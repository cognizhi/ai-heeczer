"use client";

/**
 * GoldenDiff — displays the scoring result and highlights deviations from
 * the golden record. ADR-0012.
 */
interface GoldenDiffProps {
  result: Record<string, unknown>;
}

export function GoldenDiff({ result }: GoldenDiffProps) {
  return (
    <div className="rounded-lg border overflow-hidden">
      <div className="bg-gray-50 dark:bg-gray-900 px-4 py-2 border-b flex items-center justify-between">
        <h2 className="text-sm font-semibold">Score Result</h2>
        <span className="text-xs text-gray-500">
          Golden diff — coming once golden fixtures are wired
        </span>
      </div>
      <pre className="p-4 text-xs font-mono overflow-auto max-h-96 bg-white dark:bg-gray-950">
        {JSON.stringify(result, null, 2)}
      </pre>
    </div>
  );
}
