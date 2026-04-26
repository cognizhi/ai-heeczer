import { MetricCard } from "@/components/metric-card";
import { StatusPill } from "@/components/status-pill";
import { getDashboardData } from "@/lib/dashboard-data";

export default async function QueuePage() {
  const data = await getDashboardData();
  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold tracking-tight">Queue Health</h1>
      <section className="grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
        {data.queue.map((metric) => (
          <div key={metric.label} className="rounded-md border p-4">
            <div className="flex items-center justify-between"><span className="text-xs font-medium uppercase text-gray-500">{metric.label}</span><StatusPill state={metric.state} /></div>
            <p className="mt-2 text-2xl font-bold">{metric.value}</p>
          </div>
        ))}
      </section>
      <MetricCard label="Worker mode" value="image" unit="HTTP" />
    </div>
  );
}
