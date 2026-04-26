import Link from "next/link";
import { TrendChart } from "@/components/trend-chart";
import { getDashboardData } from "@/lib/dashboard-data";

export default async function TrendsPage({ searchParams }: { searchParams: Promise<{ range?: string }> }) {
  const params = await searchParams;
  const range = params.range ?? "7d";
  const data = await getDashboardData();
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between gap-4">
        <h1 className="text-2xl font-bold tracking-tight">Trends</h1>
        <div className="flex rounded-md border p-1 text-sm">
          {["7d", "30d", "90d"].map((value) => (
            <Link key={value} href={`/trends?range=${value}`} className={value === range ? "rounded bg-gray-900 px-3 py-1 text-white" : "px-3 py-1 text-gray-600"}>
              {value}
            </Link>
          ))}
        </div>
      </div>
      <section className="rounded-md border p-4">
        <TrendChart points={data.trends} />
      </section>
      <section className="rounded-md border overflow-hidden">
        <table className="w-full text-sm">
          <thead className="bg-gray-50 text-left text-xs uppercase text-gray-500">
            <tr><th className="p-3">Event</th><th className="p-3">Category</th><th className="p-3">Hours</th><th className="p-3">FEC</th></tr>
          </thead>
          <tbody className="divide-y">
            {data.scores.map((score) => (
              <tr key={score.eventId}>
                <td className="p-3"><Link className="font-medium text-blue-700" href={`/events/${score.eventId}`}>{score.task}</Link></td>
                <td className="p-3">{score.category}</td>
                <td className="p-3">{(score.minutes / 60).toFixed(1)}</td>
                <td className="p-3">${score.fec.toLocaleString()} <span className="text-xs text-gray-500">labor-equivalent estimate</span></td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>
    </div>
  );
}
