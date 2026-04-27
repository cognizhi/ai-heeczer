package main

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"

	heeczer "github.com/cognizhi/ai-heeczer/bindings/heeczer-go"
)

func requiredEnv(name string) string {
	value := os.Getenv(name)
	if value == "" {
		panic(fmt.Sprintf("%s is required", name))
	}
	return value
}

func main() {
	repoRoot, err := repositoryRoot()
	if err != nil {
		panic(err)
	}
	fixtureDir := os.Getenv("HEECZER_PARITY_FIXTURE_DIR")
	if fixtureDir == "" {
		fixtureDir = filepath.Join(repoRoot, "core", "schema", "fixtures", "events", "valid")
	}
	referenceDir := requiredEnv("HEECZER_PARITY_REFERENCE_DIR")
	baseURL := requiredEnv("HEECZER_PARITY_BASE_URL")

	client, err := heeczer.New(baseURL, heeczer.WithRetry(3, 50*time.Millisecond))
	if err != nil {
		panic(err)
	}

	fixtures, err := filepath.Glob(filepath.Join(fixtureDir, "*.json"))
	if err != nil {
		panic(err)
	}
	sort.Strings(fixtures)
	if len(fixtures) == 0 {
		panic(fmt.Sprintf("no valid fixtures found in %s", fixtureDir))
	}

	var failures []string
	for _, fixturePath := range fixtures {
		body, err := os.ReadFile(fixturePath)
		if err != nil {
			panic(err)
		}
		var event json.RawMessage = body
		ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
		response, err := client.TestScorePipeline(ctx, heeczer.TestPipelineRequest{Event: event})
		cancel()
		if err != nil {
			failures = append(failures, fmt.Sprintf("%s: %v", filepath.Base(fixturePath), err))
			continue
		}

		referencePath := filepath.Join(referenceDir, strings.TrimSuffix(filepath.Base(fixturePath), ".json")+".json")
		expectedBytes, err := os.ReadFile(referencePath)
		if err != nil {
			panic(err)
		}
		actualBytes, err := json.Marshal(response.Score)
		if err != nil {
			panic(err)
		}
		if string(actualBytes) != strings.TrimRight(string(expectedBytes), "\r\n") {
			failures = append(failures, filepath.Base(fixturePath)+": score JSON differed from Rust reference")
		}
	}

	if len(failures) > 0 {
		panic(strings.Join(failures, "\n"))
	}
	fmt.Printf("Go SDK parity passed for %d fixture(s)\n", len(fixtures))
}

func repositoryRoot() (string, error) {
	current, err := os.Getwd()
	if err != nil {
		return "", err
	}
	for {
		if _, err := os.Stat(filepath.Join(current, "Cargo.toml")); err == nil {
			return current, nil
		}
		parent := filepath.Dir(current)
		if parent == current {
			return "", fmt.Errorf("repository root with Cargo.toml not found")
		}
		current = parent
	}
}
