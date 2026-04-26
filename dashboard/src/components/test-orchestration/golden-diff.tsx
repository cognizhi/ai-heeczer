"use client";

/**
 * GoldenDiff — displays the scoring result and highlights deviations from
 * the golden record. ADR-0012.
 */
interface GoldenDiffProps {
  result: Record<string, unknown>;
  golden?: Record<string, unknown> | null;
}

interface DiffRow {
  path: string;
  expected: unknown;
  actual: unknown;
}

function scoreBody(result: Record<string, unknown>): Record<string, unknown> {
  const score = result["score"];
  return score && typeof score === "object" && !Array.isArray(score)
    ? (score as Record<string, unknown>)
    : result;
}

function diff(expected: unknown, actual: unknown, path = "$", rows: DiffRow[] = []): DiffRow[] {
  if (Object.is(expected, actual)) return rows;
  if (
    expected &&
    actual &&
    typeof expected === "object" &&
    typeof actual === "object" &&
    !Array.isArray(expected) &&
    !Array.isArray(actual)
  ) {
    const keys = new Set([...Object.keys(expected), ...Object.keys(actual)]);
    for (const key of keys) {
      diff(
        (expected as Record<string, unknown>)[key],
        (actual as Record<string, unknown>)[key],
        `${path}.${key}`,
        rows,
      );
    }
    return rows;
  }
  rows.push({ path, expected, actual });
  return rows;
}

export function GoldenDiff({ result, golden }: GoldenDiffProps) {
  const actual = scoreBody(result);
  const rows = golden ? diff(golden, actual) : [];
  return (
    <div className="rounded-lg border overflow-hidden">
      <div className="bg-gray-50 dark:bg-gray-900 px-4 py-2 border-b flex items-center justify-between">
        <h2 className="text-sm font-semibold">Score Result</h2>
        <span className="text-xs text-gray-500">
          {golden ? `${rows.length} mismatched paths` : "No golden selected"}
        </span>
      </div>
      {golden && rows.length > 0 && (
        <div className="border-b bg-amber-50 p-3 text-xs text-amber-950">
          {rows.slice(0, 8).map((row) => (
            <div key={row.path} className="grid gap-2 py-1 sm:grid-cols-3">
              <span className="font-mono">{row.path}</span>
              <span>expected {JSON.stringify(row.expected)}</span>
              <span>actual {JSON.stringify(row.actual)}</span>
            </div>
          ))}
        </div>
      )}
      <pre className="p-4 text-xs font-mono overflow-auto max-h-96 bg-white dark:bg-gray-950">
        {JSON.stringify(result, null, 2)}
      </pre>
    </div>
  );
}
