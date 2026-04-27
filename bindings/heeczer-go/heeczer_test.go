package heeczer_test

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"reflect"
	"runtime"
	"strings"
	"testing"

	"github.com/cognizhi/ai-heeczer/bindings/heeczer-go"
)

func newTestClient(t *testing.T, h http.HandlerFunc, opts ...heeczer.Option) (*heeczer.Client, *httptest.Server) {
	t.Helper()
	srv := httptest.NewServer(h)
	t.Cleanup(srv.Close)
	c, err := heeczer.New(srv.URL, opts...)
	if err != nil {
		t.Fatalf("new client: %v", err)
	}
	return c, srv
}

func TestNewRequiresBaseURL(t *testing.T) {
	if _, err := heeczer.New(""); err == nil {
		t.Fatal("expected error on empty baseURL")
	}
}

func TestNativeModeFailsFastUntilCgoBindingShips(t *testing.T) {
	_, err := heeczer.New("https://api.example.com", heeczer.WithMode(heeczer.ModeNative))
	if err == nil || !strings.Contains(err.Error(), "native mode") {
		t.Fatalf("expected native mode error, got %v", err)
	}
}

func TestHealthzReturnsTrueOn2xx(t *testing.T) {
	c, _ := newTestClient(t, func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/healthz" {
			t.Fatalf("path = %s", r.URL.Path)
		}
		w.WriteHeader(http.StatusOK)
	})
	ok, err := c.Healthz(context.Background())
	if err != nil || !ok {
		t.Fatalf("healthz: ok=%v err=%v", ok, err)
	}
}

func TestVersionEnvelope(t *testing.T) {
	c, _ := newTestClient(t, func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("content-type", "application/json")
		_, _ = w.Write([]byte(`{"ok":true,"envelope_version":"1","scoring_version":"1.0.0","spec_version":"1.0","service":"0.1.0"}`))
	})
	v, err := c.Version(context.Background())
	if err != nil {
		t.Fatalf("version: %v", err)
	}
	if v.ScoringVersion != "1.0.0" || v.SpecVersion != "1.0" {
		t.Fatalf("unexpected: %+v", v)
	}
}

func TestIngestEventCanonicalBody(t *testing.T) {
	var captured map[string]any
	c, _ := newTestClient(t, func(w http.ResponseWriter, r *http.Request) {
		_ = json.NewDecoder(r.Body).Decode(&captured)
		if got := r.Header.Get("x-heeczer-api-key"); got != "k_secret" {
			t.Fatalf("api key header = %q", got)
		}
		w.Header().Set("content-type", "application/json")
		_, _ = w.Write([]byte(`{"ok":true,"envelope_version":"1","event_id":"evt-1","score":{"scoring_version":"1.0.0","spec_version":"1.0","scoring_profile":"default","category":"uncategorized","final_estimated_minutes":"1","estimated_hours":"0.02","estimated_days":"0.0025","financial_equivalent_cost":"1","confidence_score":"0.5","confidence_band":"Medium","human_summary":"ok"}}`))
	}, heeczer.WithAPIKey("k_secret"))

	resp, err := c.IngestEvent(context.Background(), "ws_test", map[string]string{"event_id": "evt-1"})
	if err != nil {
		t.Fatalf("ingest: %v", err)
	}
	if resp.EventID != "evt-1" || resp.Score.ConfidenceBand != heeczer.ConfidenceMedium {
		t.Fatalf("unexpected: %+v", resp)
	}
	if captured["workspace_id"] != "ws_test" {
		t.Fatalf("workspace_id = %v", captured["workspace_id"])
	}
}

func TestErrorEnvelopeMapsToTypedError(t *testing.T) {
	c, _ := newTestClient(t, func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("content-type", "application/json")
		w.WriteHeader(http.StatusBadRequest)
		_, _ = w.Write([]byte(`{"ok":false,"envelope_version":"1","error":{"kind":"schema","message":"missing field event_id"}}`))
	})
	_, err := c.IngestEvent(context.Background(), "ws", map[string]string{})
	if err == nil {
		t.Fatal("expected error")
	}
	if !heeczer.IsKind(err, heeczer.ErrSchema) {
		t.Fatalf("expected schema kind, got: %v", err)
	}
}

func TestNonJSONErrorFallsBackToUnknown(t *testing.T) {
	c, _ := newTestClient(t, func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusGatewayTimeout)
		_, _ = w.Write([]byte("upstream timeout"))
	})
	_, err := c.Version(context.Background())
	if !heeczer.IsKind(err, heeczer.ErrUnknown) {
		t.Fatalf("expected unknown kind, got: %v", err)
	}
}

func TestTestScorePipelineSendsTesterHeader(t *testing.T) {
	c, _ := newTestClient(t, func(w http.ResponseWriter, r *http.Request) {
		if r.Header.Get("x-heeczer-tester") != "1" {
			t.Fatalf("missing tester header")
		}
		w.Header().Set("content-type", "application/json")
		_, _ = w.Write([]byte(`{"ok":true,"envelope_version":"1","score":{"scoring_version":"1.0.0","spec_version":"1.0","scoring_profile":"default","category":"uncategorized","final_estimated_minutes":"1","estimated_hours":"0.02","estimated_days":"0.0025","financial_equivalent_cost":"1","confidence_score":"0.5","confidence_band":"Medium","human_summary":"ok"}}`))
	})
	_, err := c.TestScorePipeline(context.Background(), heeczer.TestPipelineRequest{Event: map[string]string{"event_id": "evt"}})
	if err != nil {
		t.Fatalf("test pipeline: %v", err)
	}
}

func TestScoreResultPreservesAdditiveFieldsOnMarshal(t *testing.T) {
	raw := []byte(`{"scoring_version":"1.0.0","spec_version":"1.0","scoring_profile":"default","bcu_breakdown":{"tokens":"1"},"category":"uncategorized","final_estimated_minutes":"1","estimated_hours":"0.02","estimated_days":"0.00","financial_equivalent_cost":"1","confidence_score":"0.5","confidence_band":"Medium","human_summary":"ok"}`)
	var score heeczer.ScoreResult
	if err := json.Unmarshal(raw, &score); err != nil {
		t.Fatalf("unmarshal: %v", err)
	}
	encoded, err := json.Marshal(score)
	if err != nil {
		t.Fatalf("marshal: %v", err)
	}
	if string(encoded) != string(raw) {
		t.Fatalf("score result did not preserve raw JSON\nwant: %s\n got: %s", raw, encoded)
	}
	if score.JSON() != string(raw) {
		t.Fatalf("ScoreResult.JSON() = %s", score.JSON())
	}
}

func TestBaseURLTrailingSlashStripped(t *testing.T) {
	var seenPath string
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		seenPath = r.URL.Path
		w.WriteHeader(http.StatusOK)
	}))
	t.Cleanup(srv.Close)
	c, err := heeczer.New(srv.URL + "/")
	if err != nil {
		t.Fatalf("new: %v", err)
	}
	if _, err := c.Healthz(context.Background()); err != nil {
		t.Fatalf("healthz: %v", err)
	}
	if !strings.HasSuffix(seenPath, "/healthz") || strings.Contains(seenPath, "//") {
		t.Fatalf("path = %q", seenPath)
	}
}

func TestRetryTransientStatus(t *testing.T) {
	calls := 0
	c, _ := newTestClient(t, func(w http.ResponseWriter, r *http.Request) {
		calls++
		if calls == 1 {
			w.WriteHeader(http.StatusServiceUnavailable)
			_, _ = w.Write([]byte(`{"ok":false,"error":{"kind":"unavailable","message":"warming"}}`))
			return
		}
		w.Header().Set("content-type", "application/json")
		_, _ = w.Write([]byte(`{"ok":true,"envelope_version":"1"}`))
	}, heeczer.WithRetry(2, 0))
	ok, err := c.Healthz(context.Background())
	if err != nil || !ok {
		t.Fatalf("healthz: ok=%v err=%v", ok, err)
	}
	if calls != 2 {
		t.Fatalf("calls=%d, want 2", calls)
	}
}

// ── Contract tests (plan 0001 / ADR-0002) ────────────────────────────────────

// fixtureDir returns the absolute path to core/schema/fixtures/events/valid
// regardless of where `go test` is invoked from.
func fixtureDir() string {
	_, thisFile, _, _ := runtime.Caller(0)
	// thisFile = .../bindings/heeczer-go/heeczer_test.go
	return filepath.Join(filepath.Dir(thisFile), "../../core/schema/fixtures/events/valid")
}

func loadValidFixtures(t *testing.T) []struct{ name, body string } {
	t.Helper()
	dir := fixtureDir()
	entries, err := os.ReadDir(dir)
	if err != nil {
		t.Fatalf("read fixture dir %s: %v", dir, err)
	}
	var out []struct{ name, body string }
	for _, e := range entries {
		if filepath.Ext(e.Name()) != ".json" {
			continue
		}
		raw, err := os.ReadFile(filepath.Join(dir, e.Name()))
		if err != nil {
			t.Fatalf("read fixture %s: %v", e.Name(), err)
		}
		out = append(out, struct{ name, body string }{e.Name(), string(raw)})
	}
	return out
}

// TestContractAtLeastOneValidFixture verifies the fixture directory is reachable.
func TestContractAtLeastOneValidFixture(t *testing.T) {
	fixtures := loadValidFixtures(t)
	if len(fixtures) == 0 {
		t.Fatalf("no valid fixtures found in %s", fixtureDir())
	}
}

// TestContractValidFixtureRoundTrips checks every valid fixture deserializes
// into CanonicalEvent and re-serializes to semantically equal JSON.
func TestContractValidFixtureRoundTrips(t *testing.T) {
	for _, fix := range loadValidFixtures(t) {
		fix := fix // capture
		t.Run(fix.name, func(t *testing.T) {
			var event heeczer.CanonicalEvent
			if err := json.Unmarshal([]byte(fix.body), &event); err != nil {
				t.Fatalf("unmarshal: %v", err)
			}

			reserialised, err := json.Marshal(&event)
			if err != nil {
				t.Fatalf("marshal: %v", err)
			}

			// Compare semantically: parse both as map[string]any.
			var original, roundtripped map[string]any
			if err := json.Unmarshal([]byte(fix.body), &original); err != nil {
				t.Fatalf("unmarshal original: %v", err)
			}
			if err := json.Unmarshal(reserialised, &roundtripped); err != nil {
				t.Fatalf("unmarshal roundtripped: %v", err)
			}
			if !reflect.DeepEqual(original, roundtripped) {
				t.Fatalf("round-trip mismatch for %s:\noriginal   : %v\nroundtripped: %v",
					fix.name, original, roundtripped)
			}
		})
	}
}

// TestContractExtensionsSurviveRoundTrip verifies meta.extensions are preserved.
func TestContractExtensionsSurviveRoundTrip(t *testing.T) {
	extensions := json.RawMessage(`{"custom_key":42,"nested":{"x":true}}`)
	event := heeczer.CanonicalEvent{
		SpecVersion:     "1.0",
		EventID:         "00000000-0000-4000-8000-aabbccddeeff",
		Timestamp:       "2026-04-22T10:00:00Z",
		FrameworkSource: "test",
		WorkspaceID:     "ws_ext",
		Task:            heeczer.EventTask{Name: "ext_test", Outcome: heeczer.OutcomeSuccess},
		Metrics:         heeczer.EventMetrics{DurationMS: 100},
		Meta: heeczer.EventMeta{
			SDKLanguage: "go",
			SDKVersion:  "0.1.0",
			Extensions:  extensions,
		},
	}

	raw, err := json.Marshal(&event)
	if err != nil {
		t.Fatalf("marshal: %v", err)
	}

	var back heeczer.CanonicalEvent
	if err := json.Unmarshal(raw, &back); err != nil {
		t.Fatalf("unmarshal: %v", err)
	}

	var ext map[string]any
	if err := json.Unmarshal(back.Meta.Extensions, &ext); err != nil {
		t.Fatalf("unmarshal extensions: %v", err)
	}
	if v, ok := ext["custom_key"].(float64); !ok || v != 42 {
		t.Fatalf("extensions.custom_key = %v, want 42", ext["custom_key"])
	}
	nested, ok := ext["nested"].(map[string]any)
	if !ok {
		t.Fatalf("extensions.nested is not an object: %v", ext["nested"])
	}
	if nested["x"] != true {
		t.Fatalf("extensions.nested.x = %v, want true", nested["x"])
	}
}

// TestContractUnknownTopLevelFieldRejected verifies that DisallowUnknownFields
// causes decoding to fail when an unknown top-level field is present.
func TestContractUnknownTopLevelFieldRejected(t *testing.T) {
	bad := `{
		"spec_version":"1.0",
		"event_id":"00000000-0000-4000-8000-aabbccddeeff",
		"timestamp":"2026-04-22T10:00:00Z",
		"framework_source":"test",
		"workspace_id":"ws_strict",
		"task":{"name":"t","outcome":"success"},
		"metrics":{"duration_ms":100},
		"meta":{"sdk_language":"go","sdk_version":"0.1.0"},
		"forbidden_extra_field":"value"
	}`

	dec := json.NewDecoder(strings.NewReader(bad))
	dec.DisallowUnknownFields()
	var event heeczer.CanonicalEvent
	if err := dec.Decode(&event); err == nil {
		t.Fatal("expected error when unknown top-level field is present")
	}
}
