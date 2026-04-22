/**
 * Test Orchestration view — fixture browser → pipeline runner → golden diff.
 * ADR-0012 / plan 0010.
 */
"use client";

import { useState } from "react";
import { PipelineRunner } from "@/components/test-orchestration/pipeline-runner";
import { FixtureBrowser } from "@/components/test-orchestration/fixture-browser";
import { GoldenDiff } from "@/components/test-orchestration/golden-diff";

export default function TestOrchestrationPage() {
  const [selectedFixture, setSelectedFixture] = useState<string | null>(null);
  const [result, setResult] = useState<Record<string, unknown> | null>(null);

  return (
    <div className="space-y-6">
      <section>
        <h1 className="text-2xl font-bold tracking-tight mb-1">
          Test Orchestration
        </h1>
        <p className="text-sm text-gray-500">
          Browse fixtures, run the scoring pipeline, and diff against the golden
          record. ADR-0012.
        </p>
      </section>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-1">
          <FixtureBrowser
            onSelect={(name) => {
              setSelectedFixture(name);
              setResult(null);
            }}
            selected={selectedFixture}
          />
        </div>
        <div className="lg:col-span-2 space-y-4">
          <PipelineRunner
            fixture={selectedFixture}
            onResult={setResult}
          />
          {result !== null && <GoldenDiff result={result} />}
        </div>
      </div>
    </div>
  );
}
