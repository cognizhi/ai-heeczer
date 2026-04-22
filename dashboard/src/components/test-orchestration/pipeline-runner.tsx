"use client";

import { useState } from "react";

interface PipelineRunnerProps {
  fixture: string | null;
  onResult: (result: Record<string, unknown>) => void;
}

/**
 * PipelineRunner — submits the selected fixture to the scoring pipeline
 * and streams back the ScoreResult. ADR-0012.
 */
export function PipelineRunner({ fixture, onResult }: PipelineRunnerProps) {
  const [running, setRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const ingestUrl =
    process.env["NEXT_PUBLIC_INGEST_URL"] ?? "http://localhost:8080";

  async function handleRun() {
    if (!fixture) return;
    setRunning(true);
    setError(null);
    try {
      const res = await fetch(`${ingestUrl}/v1/test/score-pipeline`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "x-heeczer-tester": "1",
        },
        body: JSON.stringify({ fixture }),
      });
      if (!res.ok) {
        const body = (await res.json()) as {
          error?: { kind?: string; message?: string };
        };
        const msg = body.error?.message ?? res.statusText;
        throw new Error(msg.slice(0, 200));
      }
      const data = (await res.json()) as Record<string, unknown>;
      onResult(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setRunning(false);
    }
  }

  return (
    <div className="rounded-lg border p-4 space-y-3">
      <div className="flex items-center justify-between">
        <h2 className="text-sm font-semibold">Pipeline Runner</h2>
        <button
          onClick={() => void handleRun()}
          disabled={!fixture || running}
          className="px-3 py-1.5 text-sm font-medium rounded bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {running ? "Running…" : "Run"}
        </button>
      </div>
      {fixture && (
        <p className="text-xs text-gray-500">
          Fixture: <code className="font-mono">{fixture}</code>
        </p>
      )}
      {error && (
        <p className="text-xs text-red-600 dark:text-red-400" role="alert">
          {error}
        </p>
      )}
    </div>
  );
}
