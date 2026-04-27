import Link from "next/link";

import { canAdmin, getDashboardSession } from "@/lib/session";

const metrics = [
  {
    name: "total_items",
    description: "How many benchmark items were scored in the run.",
  },
  {
    name: "within_expected_range",
    description: "How many items landed inside their expected human-effort range.",
  },
  {
    name: "confidence_band_matches",
    description: "How many items matched the expected confidence band from the pack.",
  },
  {
    name: "rmse_minutes",
    description: "Root-mean-square error versus each benchmark item's expected midpoint.",
  },
  {
    name: "mae_range_minutes",
    description: "Mean absolute distance from each expected range. Zero means every item landed in range.",
  },
  {
    name: "mae_midpoint_minutes",
    description: "Mean absolute distance from each benchmark item's expected midpoint.",
  },
  {
    name: "bias_minutes",
    description: "Signed mean error. Positive values indicate systematic over-estimation.",
  },
  {
    name: "r_squared",
    description: "Goodness of fit against expected midpoints. Higher is better.",
  },
];

const storageRows = [
  "heec_benchmark_packs stores the pack definition used for the run.",
  "heec_calibration_runs stores the completed report JSON and profile linkage.",
  "heec_scoring_profiles stores the suggested profile as a new patch version.",
  "heec_audit_log records both scoring_profile_calibrated and calibration_run_completed.",
];

export default async function AdminCalibrationPage() {
  const session = await getDashboardSession();
  const allowed = canAdmin(session.role);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between gap-4">
        <div className="space-y-1">
          <p className="text-sm uppercase tracking-[0.2em] text-gray-500">
            Dashboard Help
          </p>
          <h1 className="text-2xl font-bold tracking-tight">Calibration Guide</h1>
        </div>
        <span className="rounded-md border px-3 py-1 text-sm">Role: {session.role}</span>
      </div>

      {!allowed && (
        <p
          role="alert"
          className="rounded-md border border-amber-200 bg-amber-50 p-3 text-sm text-amber-900"
        >
          Admin or owner role required
        </p>
      )}

      <section className="rounded-md border p-5">
        <div className="space-y-2">
          <h2 className="text-lg font-semibold">Run the reference pack</h2>
          <p className="text-sm text-gray-600">
            The shipped workflow scores deterministic synthetic events from the
            embedded reference pack, reports per-item deltas, and suggests
            category multiplier updates without mutating the source profile.
          </p>
          <p className="text-sm text-gray-600">
            This page documents the CLI workflow. The interactive dashboard
            calibration page with pack picker, run history, and delta charts is
            still tracked separately in Plan 15.
          </p>
          <pre className="overflow-x-auto rounded-md bg-gray-950 p-4 text-sm text-gray-100">
            <code>{`heec calibrate run \
  --pack reference-pack \
  --profile default \
  --output-profile ./default.calibrated.json \
  --database-url sqlite:///tmp/heec.sqlite?mode=rwc \
  --workspace default`}</code>
          </pre>
        </div>
      </section>

      <div className="grid gap-4 lg:grid-cols-2">
        <section className="rounded-md border p-5">
          <h2 className="text-lg font-semibold">Interpret the report</h2>
          <div className="mt-4 space-y-3">
            {metrics.map((metric) => (
              <div key={metric.name} className="rounded-md border border-dashed p-3">
                <p className="text-sm font-semibold">{metric.name}</p>
                <p className="text-sm text-gray-600">{metric.description}</p>
              </div>
            ))}
          </div>
        </section>

        <section className="rounded-md border p-5">
          <h2 className="text-lg font-semibold">Persisted artifacts</h2>
          <div className="mt-4 space-y-3 text-sm text-gray-600">
            {storageRows.map((row) => (
              <p key={row} className="rounded-md border border-dashed p-3">
                {row}
              </p>
            ))}
          </div>
        </section>
      </div>

      <section className="rounded-md border p-5">
        <h2 className="text-lg font-semibold">Operational notes</h2>
        <div className="mt-4 space-y-3 text-sm text-gray-600">
          <p className="rounded-md border border-dashed p-3">
            Suggested profile files are emitted as a new patch version. The CLI
            never mutates the source profile in place.
          </p>
          <p className="rounded-md border border-dashed p-3">
            The current persistence path is SQLite-backed because the CLI uses
            the embedded storage migrator directly.
          </p>
          <p className="rounded-md border border-dashed p-3">
            Return to the <Link href="/admin" className="font-medium text-gray-900 underline">Admin Console</Link> to access other operational workflows.
          </p>
        </div>
      </section>
    </div>
  );
}
