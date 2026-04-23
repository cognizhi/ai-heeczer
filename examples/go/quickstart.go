// Quickstart: submit an event to the ingestion service via the Go SDK.
//
// Prereq: ingestion service running locally (cargo run -p heeczer-ingest).
//
// Run from the repository root:
//
//	(cd examples/go && go run .)
//
// (`examples/go/` is its own Go module with a `replace` directive pointing
// at the local SDK source so this example builds without first publishing
// the module. Single-file `go run examples/go/quickstart.go` from the repo
// root would NOT pick up that nested go.mod.)
package main

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"runtime"
	"time"

	heeczer "github.com/cognizhi/ai-heeczer/bindings/heeczer-go"
)

func main() {
	_, thisFile, _, _ := runtime.Caller(0)
	root := filepath.Join(filepath.Dir(thisFile), "..", "..")
	body, err := os.ReadFile(filepath.Join(root, "examples", "event.json"))
	if err != nil {
		log.Fatalf("read event.json: %v", err)
	}

	var event map[string]any
	if err := json.Unmarshal(body, &event); err != nil {
		log.Fatalf("parse event.json: %v", err)
	}

	base := os.Getenv("HEECZER_BASE_URL")
	if base == "" {
		base = "http://127.0.0.1:8080"
	}

	client, err := heeczer.New(base,
		heeczer.WithAPIKey(os.Getenv("HEECZER_API_KEY")))
	if err != nil {
		log.Fatalf("client: %v", err)
	}

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	if v, err := client.Version(ctx); err != nil {
		log.Fatalf("version: %v", err)
	} else {
		fmt.Printf("» service version: %+v\n", v)
	}

	resp, err := client.IngestEvent(ctx, "ws_default", event)
	if err != nil {
		var apiErr *heeczer.APIError
		if errors.As(err, &apiErr) {
			log.Fatalf("SDK error: kind=%s status=%d message=%s", apiErr.Kind, apiErr.Status, apiErr.Message)
		}
		log.Fatalf("ingest: %v", err)
	}

	fmt.Printf("» event %s ingested\n", resp.EventID)
	fmt.Printf("» summary: %s\n", resp.Score.HumanSummary)
	fmt.Printf("» minutes=%s band=%s\n", resp.Score.FinalEstimatedMinutes, resp.Score.ConfidenceBand)
}
