// Package heeczer is a thin Go client for the ai-heeczer ingestion
// service (plan 0007). It speaks the envelope_version=1 contract
// documented in ADR-0011 and surfaces typed errors via *APIError so
// callers do not pattern-match on strings.
package heeczer

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"strings"
	"time"
)

// maxResponseBytes caps the amount of data read from any single API response
// to prevent memory exhaustion from malicious or misconfigured servers.
const maxResponseBytes = 4 * 1024 * 1024 // 4 MiB

// Version is the SDK version. Kept in lockstep with the npm + PyPI
// siblings.
const Version = "0.1.0"

// ConfidenceBand mirrors the Rust ConfidenceBand enum.
type ConfidenceBand string

const (
	ConfidenceLow    ConfidenceBand = "Low"
	ConfidenceMedium ConfidenceBand = "Medium"
	ConfidenceHigh   ConfidenceBand = "High"
)

// ScoreResult is the typed surface of the engine's score result. Extra
// fields are tolerated via Extra.
type ScoreResult struct {
	ScoringVersion          string         `json:"scoring_version"`
	SpecVersion             string         `json:"spec_version"`
	ScoringProfile          string         `json:"scoring_profile"`
	Category                string         `json:"category"`
	FinalEstimatedMinutes   string         `json:"final_estimated_minutes"`
	EstimatedHours          string         `json:"estimated_hours"`
	EstimatedDays           string         `json:"estimated_days"`
	FinancialEquivalentCost string         `json:"financial_equivalent_cost"`
	ConfidenceScore         string         `json:"confidence_score"`
	ConfidenceBand          ConfidenceBand `json:"confidence_band"`
	HumanSummary            string         `json:"human_summary"`
}

// IngestEventResponse is returned by Client.IngestEvent.
type IngestEventResponse struct {
	OK              bool        `json:"ok"`
	EnvelopeVersion string      `json:"envelope_version"`
	EventID         string      `json:"event_id"`
	Score           ScoreResult `json:"score"`
}

// VersionResponse is returned by Client.Version.
type VersionResponse struct {
	OK              bool   `json:"ok"`
	EnvelopeVersion string `json:"envelope_version"`
	ScoringVersion  string `json:"scoring_version"`
	SpecVersion     string `json:"spec_version"`
	Service         string `json:"service"`
}

// TestPipelineResponse is returned by Client.TestScorePipeline.
type TestPipelineResponse struct {
	OK              bool        `json:"ok"`
	EnvelopeVersion string      `json:"envelope_version"`
	Score           ScoreResult `json:"score"`
}

// ErrorKind is the closed enum mirrored from the ingestion service
// envelope (services/heeczer-ingest/src/error.rs).
type ErrorKind string

const (
	ErrSchema          ErrorKind = "schema"
	ErrBadRequest      ErrorKind = "bad_request"
	ErrScoring         ErrorKind = "scoring"
	ErrStorage         ErrorKind = "storage"
	ErrNotFound        ErrorKind = "not_found"
	ErrForbidden       ErrorKind = "forbidden"
	ErrFeatureDisabled ErrorKind = "feature_disabled"
	ErrUnknown         ErrorKind = "unknown"
)

// APIError is returned by every Client method on a non-2xx response.
type APIError struct {
	Status  int
	Kind    ErrorKind
	Message string
}

func (e *APIError) Error() string {
	return fmt.Sprintf("heeczer %d %s: %s", e.Status, e.Kind, e.Message)
}

// IsKind reports whether err is an *APIError with the given kind.
func IsKind(err error, kind ErrorKind) bool {
	var api *APIError
	return errors.As(err, &api) && api.Kind == kind
}

// Doer is the minimal interface we need from *http.Client; callers can
// inject a fake for tests.
type Doer interface {
	Do(req *http.Request) (*http.Response, error)
}

// Client talks to the ai-heeczer ingestion service.
type Client struct {
	baseURL string
	apiKey  string
	http    Doer
}

// Option configures a Client.
type Option func(*Client)

// WithAPIKey sets the x-heeczer-api-key header.
func WithAPIKey(k string) Option {
	return func(c *Client) { c.apiKey = k }
}

// WithHTTPClient injects a custom http.Client (or any Doer). Useful for
// tests against httptest.NewServer.
func WithHTTPClient(d Doer) Option {
	return func(c *Client) { c.http = d }
}

// New constructs a Client for the given base URL.
func New(baseURL string, opts ...Option) (*Client, error) {
	if baseURL == "" {
		return nil, errors.New("heeczer: baseURL is required")
	}
	c := &Client{
		baseURL: strings.TrimRight(baseURL, "/"),
		http:    &http.Client{Timeout: 10 * time.Second},
	}
	for _, o := range opts {
		o(c)
	}
	return c, nil
}

// Healthz returns true if the service responds 2xx to /healthz.
func (c *Client) Healthz(ctx context.Context) (bool, error) {
	resp, err := c.do(ctx, http.MethodGet, "/healthz", nil, nil)
	if err != nil {
		return false, err
	}
	defer resp.Body.Close()
	return resp.StatusCode >= 200 && resp.StatusCode < 300, nil
}

// Version returns the engine + spec versions advertised by the service.
func (c *Client) Version(ctx context.Context) (*VersionResponse, error) {
	var out VersionResponse
	if err := c.getJSON(ctx, "/v1/version", &out); err != nil {
		return nil, err
	}
	return &out, nil
}

// IngestEvent validates, scores, and persists a single canonical event.
// event must be a JSON-serialisable Go value (struct, map, json.RawMessage).
func (c *Client) IngestEvent(
	ctx context.Context, workspaceID string, event any,
) (*IngestEventResponse, error) {
	body := map[string]any{"workspace_id": workspaceID, "event": event}
	var out IngestEventResponse
	if err := c.postJSON(ctx, "/v1/events", body, nil, &out); err != nil {
		return nil, err
	}
	return &out, nil
}

// TestPipelineRequest configures TestScorePipeline.
type TestPipelineRequest struct {
	Event        any
	Profile      any
	TierSet      any
	TierOverride string
}

// TestScorePipeline runs the scoring pipeline back-to-back without
// persisting. Always sends the x-heeczer-tester header.
func (c *Client) TestScorePipeline(
	ctx context.Context, req TestPipelineRequest,
) (*TestPipelineResponse, error) {
	body := map[string]any{"event": req.Event}
	if req.Profile != nil {
		body["profile"] = req.Profile
	}
	if req.TierSet != nil {
		body["tier_set"] = req.TierSet
	}
	if req.TierOverride != "" {
		body["tier_override"] = req.TierOverride
	}
	headers := map[string]string{"x-heeczer-tester": "1"}
	var out TestPipelineResponse
	if err := c.postJSON(ctx, "/v1/test/score-pipeline", body, headers, &out); err != nil {
		return nil, err
	}
	return &out, nil
}

func (c *Client) getJSON(ctx context.Context, path string, out any) error {
	resp, err := c.do(ctx, http.MethodGet, path, nil, nil)
	if err != nil {
		return err
	}
	return c.handle(resp, out)
}

func (c *Client) postJSON(
	ctx context.Context, path string, body any, extraHeaders map[string]string, out any,
) error {
	buf, err := json.Marshal(body)
	if err != nil {
		return fmt.Errorf("heeczer: marshal body: %w", err)
	}
	headers := map[string]string{"content-type": "application/json"}
	for k, v := range extraHeaders {
		headers[k] = v
	}
	resp, err := c.do(ctx, http.MethodPost, path, bytes.NewReader(buf), headers)
	if err != nil {
		return err
	}
	return c.handle(resp, out)
}

func (c *Client) do(
	ctx context.Context, method, path string, body io.Reader, headers map[string]string,
) (*http.Response, error) {
	req, err := http.NewRequestWithContext(ctx, method, c.baseURL+path, body)
	if err != nil {
		return nil, fmt.Errorf("heeczer: build request: %w", err)
	}
	if c.apiKey != "" {
		req.Header.Set("x-heeczer-api-key", c.apiKey)
	}
	for k, v := range headers {
		req.Header.Set(k, v)
	}
	resp, err := c.http.Do(req)
	if err != nil {
		return nil, fmt.Errorf("heeczer: send request: %w", err)
	}
	return resp, nil
}

func (c *Client) handle(resp *http.Response, out any) error {
	defer resp.Body.Close()
	raw, err := io.ReadAll(io.LimitReader(resp.Body, maxResponseBytes))
	if err != nil {
		return fmt.Errorf("heeczer: read body: %w", err)
	}
	if int64(len(raw)) == maxResponseBytes {
		return fmt.Errorf("heeczer: response body exceeds %d bytes", maxResponseBytes)
	}
	if resp.StatusCode >= 200 && resp.StatusCode < 300 {
		if out == nil {
			return nil
		}
		if err := json.Unmarshal(raw, out); err != nil {
			return fmt.Errorf("heeczer: decode body: %w", err)
		}
		return nil
	}
	apiErr := &APIError{Status: resp.StatusCode, Kind: ErrUnknown, Message: string(raw)}
	var env struct {
		OK    bool `json:"ok"`
		Error struct {
			Kind    ErrorKind `json:"kind"`
			Message string    `json:"message"`
		} `json:"error"`
	}
	if jerr := json.Unmarshal(raw, &env); jerr == nil && !env.OK && env.Error.Kind != "" {
		apiErr.Kind = env.Error.Kind
		apiErr.Message = env.Error.Message
	} else if apiErr.Message == "" {
		apiErr.Message = resp.Status
	}
	return apiErr
}

// ── Canonical event types (mirrored from core/schema/event.v1.json) ──────────
// Mirrors heeczer_core::event (Rust) and generated per plan 0001 / ADR-0002.
// Use these types to construct events type-safely before passing them to
// Client.IngestEvent.

// Outcome is the closed task-outcome enum.
type Outcome string

const (
	OutcomeSuccess        Outcome = "success"
	OutcomePartialSuccess Outcome = "partial_success"
	OutcomeFailure        Outcome = "failure"
	OutcomeTimeout        Outcome = "timeout"
)

// EventRiskClass is the closed risk-classification enum.
type EventRiskClass string

const (
	RiskLow    EventRiskClass = "low"
	RiskMedium EventRiskClass = "medium"
	RiskHigh   EventRiskClass = "high"
)

// EventIdentity is the optional identity block.
type EventIdentity struct {
	UserID         *string `json:"user_id,omitempty"`
	TeamID         *string `json:"team_id,omitempty"`
	BusinessUnitID *string `json:"business_unit_id,omitempty"`
	// TierID is resolved against the active TierSet (PRD §14.2.1).
	TierID *string `json:"tier_id,omitempty"`
}

// EventTask is the task descriptor block.
type EventTask struct {
	Name string `json:"name"`
	// Category is optional; missing/null normalises to "uncategorized" per PRD §14.2.1.
	Category    *string `json:"category,omitempty"`
	SubCategory *string `json:"sub_category,omitempty"`
	Outcome     Outcome `json:"outcome"`
}

// EventMetrics is the required telemetry metrics block.
type EventMetrics struct {
	// DurationMS is the wall-clock task duration in milliseconds (required).
	DurationMS       int64    `json:"duration_ms"`
	TokensPrompt     *int64   `json:"tokens_prompt,omitempty"`
	TokensCompletion *int64   `json:"tokens_completion,omitempty"`
	ToolCallCount    *int32   `json:"tool_call_count,omitempty"`
	WorkflowSteps    *int32   `json:"workflow_steps,omitempty"`
	Retries          *int32   `json:"retries,omitempty"`
	ArtifactCount    *int32   `json:"artifact_count,omitempty"`
	OutputSizeProxy  *float64 `json:"output_size_proxy,omitempty"`
}

// EventContext is the optional execution context block.
type EventContext struct {
	HumanInLoop    *bool           `json:"human_in_loop,omitempty"`
	ReviewRequired *bool           `json:"review_required,omitempty"`
	Temperature    *float64        `json:"temperature,omitempty"`
	RiskClass      *EventRiskClass `json:"risk_class,omitempty"`
	Tags           []string        `json:"tags,omitempty"`
}

// EventMeta is the SDK metadata block. Extensions is the sole permitted bucket
// for unknown fields (PRD §13 / ADR-0002).
type EventMeta struct {
	SDKLanguage    string          `json:"sdk_language"`
	SDKVersion     string          `json:"sdk_version"`
	ScoringProfile *string         `json:"scoring_profile,omitempty"`
	Extensions     json.RawMessage `json:"extensions,omitempty"`
}

// CanonicalEvent is the canonical ai-heeczer telemetry event (v1).
//
// Mirrors heeczer_core::Event (Rust) and the JSON Schema at
// core/schema/event.v1.json. Construct this type and pass it (or its
// json.RawMessage equivalent) to Client.IngestEvent.
//
// Example:
//
//	event := heeczer.CanonicalEvent{
//	    SpecVersion:     "1.0",
//	    EventID:         uuid.New().String(),
//	    Timestamp:       time.Now().UTC().Format(time.RFC3339Nano),
//	    FrameworkSource: "langgraph",
//	    WorkspaceID:     "ws_default",
//	    Task:   heeczer.EventTask{Name: "summarise_pr", Outcome: heeczer.OutcomeSuccess},
//	    Metrics: heeczer.EventMetrics{DurationMS: 3200},
//	    Meta:   heeczer.EventMeta{SDKLanguage: "go", SDKVersion: "0.1.0"},
//	}
type CanonicalEvent struct {
	// SpecVersion must be "1.0" for v1 events.
	SpecVersion     string         `json:"spec_version"`
	EventID         string         `json:"event_id"`
	CorrelationID   *string        `json:"correlation_id,omitempty"`
	Timestamp       string         `json:"timestamp"`
	FrameworkSource string         `json:"framework_source"`
	WorkspaceID     string         `json:"workspace_id"`
	ProjectID       *string        `json:"project_id,omitempty"`
	Identity        *EventIdentity `json:"identity,omitempty"`
	Task            EventTask      `json:"task"`
	Metrics         EventMetrics   `json:"metrics"`
	Context         *EventContext  `json:"context,omitempty"`
	Meta            EventMeta      `json:"meta"`
}
