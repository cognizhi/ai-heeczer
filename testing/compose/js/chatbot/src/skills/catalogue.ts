import { readFileSync } from "node:fs";
import { join } from "node:path";

import type { ToolName } from "../tools/catalogue.js";

export interface SkillFixture {
  skill: string;
  command: string;
  mock_script: Array<{ tool: ToolName; stub_output: Record<string, unknown> }>;
  expected_event: {
    task: { category: string; sub_category: string; outcome: string };
    metrics: Record<string, number>;
    context: Record<string, unknown>;
  };
}

const SKILL_ALIASES: Record<string, string> = {
  "code-gen": "code_gen",
  "doc-summary": "doc_summary",
  "ci-triage": "ci_triage",
};

export function normalizeSkill(raw: string | undefined): string {
  const value = (raw ?? "code_gen").replace(/^\/skill\s+/, "").trim();
  return SKILL_ALIASES[value] ?? value;
}

export function loadSkill(raw: string | undefined): SkillFixture {
  const skill = normalizeSkill(raw);
  const fixtureRoot = process.env["SKILL_FIXTURE_DIR"] ?? "/fixtures/skills";
  const fixturePath = join(fixtureRoot, `${skill}.json`);
  return JSON.parse(readFileSync(fixturePath, "utf8")) as SkillFixture;
}

export function activeTools(fixture: SkillFixture): ToolName[] {
  return fixture.mock_script.map((step) => step.tool);
}
