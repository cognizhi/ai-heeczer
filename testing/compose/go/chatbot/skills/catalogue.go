package skills

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
)

// Fixture mirrors testing/tests/fixtures/skills/*.json.
type Fixture struct {
	Skill         string           `json:"skill"`
	Command       string           `json:"command"`
	MockScript    []MockScriptStep `json:"mock_script"`
	ExpectedEvent ExpectedEvent    `json:"expected_event"`
}

type MockScriptStep struct {
	Tool       string         `json:"tool"`
	StubOutput map[string]any `json:"stub_output"`
}

type ExpectedEvent struct {
	Task    map[string]any     `json:"task"`
	Metrics map[string]float64 `json:"metrics"`
	Context map[string]any     `json:"context"`
}

var aliases = map[string]string{
	"code-gen":    "code_gen",
	"doc-summary": "doc_summary",
	"ci-triage":   "ci_triage",
}

func NormalizeSkill(rawSkill string) string {
	selected := strings.TrimSpace(strings.TrimPrefix(rawSkill, "/skill "))
	if selected == "" {
		selected = "code_gen"
	}
	if alias, ok := aliases[selected]; ok {
		return alias
	}
	return selected
}

func Load(rawSkill string) (*Fixture, error) {
	skillName := NormalizeSkill(rawSkill)
	fixtureRoot := os.Getenv("SKILL_FIXTURE_DIR")
	if fixtureRoot == "" {
		fixtureRoot = "/fixtures/skills"
	}
	body, readErr := os.ReadFile(filepath.Join(fixtureRoot, skillName+".json"))
	if readErr != nil {
		return nil, readErr
	}
	var fixture Fixture
	if jsonErr := json.Unmarshal(body, &fixture); jsonErr != nil {
		return nil, jsonErr
	}
	return &fixture, nil
}

func ActiveTools(fixture *Fixture) []string {
	activeTools := make([]string, 0, len(fixture.MockScript))
	for _, step := range fixture.MockScript {
		activeTools = append(activeTools, step.Tool)
	}
	return activeTools
}
