package heeczer_test

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
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
