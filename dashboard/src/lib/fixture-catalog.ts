import type { ConfidenceBand } from "./dashboard-data";

export interface FixtureRecord {
  name: string;
  category: "valid" | "edge";
  event: Record<string, unknown>;
  golden: Record<string, unknown>;
}

const baseEvent = {
  spec_version: "1.0",
  event_id: "00000000-0000-4000-8000-000000000101",
  timestamp: "2026-04-22T10:00:00Z",
  framework_source: "test",
  workspace_id: "ws_test",
  task: {
    name: "prd_canonical",
    category: "summarization",
    outcome: "success",
  },
  metrics: { duration_ms: 3200, workflow_steps: 4, tool_call_count: 2 },
  context: { human_in_loop: true, review_required: true, risk_class: "medium" },
  meta: { sdk_language: "test", sdk_version: "0.5.1" },
};

function golden(
  minutes: string,
  confidence: ConfidenceBand,
): Record<string, unknown> {
  return {
    scoring_version: "1.0.0",
    spec_version: "1.0",
    scoring_profile: "default-v1",
    category: "summarization",
    final_estimated_minutes: minutes,
    estimated_hours: (Number(minutes) / 60).toFixed(2),
    estimated_days: (Number(minutes) / 480).toFixed(2),
    financial_equivalent_cost: (Number(minutes) * 1.5).toFixed(2),
    confidence_score:
      confidence === "High"
        ? "0.91"
        : confidence === "Medium"
          ? "0.74"
          : "0.48",
    confidence_band: confidence,
    human_summary: "Golden fixture score.",
  };
}

export const FIXTURES: FixtureRecord[] = [
  {
    name: "valid/01-prd-canonical.json",
    category: "valid",
    event: baseEvent,
    golden: golden("64.00", "High"),
  },
  {
    name: "edge/01-minimum-required.json",
    category: "edge",
    event: {
      ...baseEvent,
      event_id: "00000000-0000-4000-8000-000000000201",
      task: { name: "minimum", outcome: "success" },
      metrics: { duration_ms: 50 },
      context: undefined,
    },
    golden: golden("1.00", "Medium"),
  },
  {
    name: "edge/02-missing-category.json",
    category: "edge",
    event: {
      ...baseEvent,
      event_id: "00000000-0000-4000-8000-000000000202",
      task: { name: "missing_category", outcome: "partial_success" },
    },
    golden: golden("12.00", "Medium"),
  },
  {
    name: "edge/03-extensions-passthrough.json",
    category: "edge",
    event: {
      ...baseEvent,
      event_id: "00000000-0000-4000-8000-000000000203",
      meta: {
        sdk_language: "test",
        sdk_version: "0.5.1",
        extensions: { source: "dashboard" },
      },
    },
    golden: golden("18.00", "High"),
  },
  {
    name: "edge/04-unicode.json",
    category: "edge",
    event: {
      ...baseEvent,
      event_id: "00000000-0000-4000-8000-000000000204",
      task: {
        name: "unicode_summary",
        category: "summarization",
        outcome: "success",
      },
    },
    golden: golden("16.00", "High"),
  },
];

export function findFixture(name: string | null): FixtureRecord | null {
  if (!name) return null;
  return FIXTURES.find((fixture) => fixture.name === name) ?? null;
}
