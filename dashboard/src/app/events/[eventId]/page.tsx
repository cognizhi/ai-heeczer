import { notFound } from "next/navigation";
import { ConfidenceBadge } from "@/components/confidence-badge";
import { MetricCard } from "@/components/metric-card";
import { getScore } from "@/lib/dashboard-data";

export default async function EventDrilldownPage({ params }: { params: Promise<{ eventId: string }> }) {
  const { eventId } = await params;
  const score = await getScore(eventId);
  if (!score) notFound();
  return (
    <div className="space-y-6">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">{score.task}</h1>
          <p className="text-sm text-gray-500">{score.eventId}</p>
        </div>
        <ConfidenceBadge band={score.confidence} />
      </div>
      <section className="grid gap-4 sm:grid-cols-3">
        <MetricCard label="Effort" value={(score.minutes / 60).toFixed(1)} unit="hours" disclaimer="labor-equivalent estimate" />
        <MetricCard label="Financial Equivalent Cost" value={`$${score.fec.toLocaleString()}`} unit="USD" disclaimer="labor-equivalent estimate" />
        <MetricCard label="Category" value={score.category} />
      </section>
      <section className="rounded-md border p-4">
        <h2 className="mb-3 text-sm font-semibold">Explainability Trace</h2>
        <dl className="divide-y text-sm">
          {score.trace.map((item) => (
            <div key={item.label} className="grid gap-2 py-3 sm:grid-cols-3">
              <dt className="font-medium text-gray-600">{item.label}</dt>
              <dd className="sm:col-span-2">{item.value}</dd>
            </div>
          ))}
        </dl>
      </section>
    </div>
  );
}
