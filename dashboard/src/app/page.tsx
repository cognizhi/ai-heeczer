/**
 * Overview page — summary metrics, HEE totals, FEC, confidence distribution.
 * PRD §21 / ADR-0008.
 */
import { ConfidenceBadge } from "@/components/confidence-badge";
import { MetricCard } from "@/components/metric-card";
import { TrendChart } from "@/components/trend-chart";
import { getDashboardData } from "@/lib/dashboard-data";

export default async function OverviewPage() {
  const data = await getDashboardData();

  return (
    <div className="space-y-8">
      <section>
        <h1 className="text-2xl font-bold tracking-tight mb-2">Overview</h1>
        <p className="text-sm text-gray-500">
          All figures are{" "}
          <strong>labor-equivalent estimates</strong> (PRD §21.3).
        </p>
      </section>

      <section className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <MetricCard
          label="Total Tasks (7d)"
          value={data.summary.totalTasks}
          unit="tasks"
        />
        <MetricCard
          label="Human-Equivalent Effort"
          value={data.summary.hours}
          unit="hours"
          disclaimer="labor-equivalent estimate"
        />
        <MetricCard
          label="Financial Equivalent Cost"
          value={`$${data.summary.fec.toLocaleString()}`}
          unit="USD"
          disclaimer="labor-equivalent estimate"
        />
        <div className="rounded-lg border p-4">
          <p className="text-xs font-medium text-gray-500 uppercase tracking-wide mb-2">
            Confidence
          </p>
          <div className="flex gap-2 flex-wrap">
            <ConfidenceBadge band="High" />
            <span className="text-sm text-gray-500">{data.summary.confidence.High}</span>
            <ConfidenceBadge band="Medium" />
            <span className="text-sm text-gray-500">{data.summary.confidence.Medium}</span>
            <ConfidenceBadge band="Low" />
            <span className="text-sm text-gray-500">{data.summary.confidence.Low}</span>
          </div>
        </div>
      </section>

      <section className="rounded-lg border p-6">
        <h2 className="font-semibold mb-4">Trends (7d)</h2>
        <TrendChart points={data.trends} />
      </section>
    </div>
  );
}
