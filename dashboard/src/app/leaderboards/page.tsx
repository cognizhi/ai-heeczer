import { ConfidenceBadge } from "@/components/confidence-badge";
import { getDashboardData, type LeaderboardRow } from "@/lib/dashboard-data";

function Board({ title, rows }: { title: string; rows: LeaderboardRow[] }) {
  return (
    <section className="rounded-md border overflow-hidden">
      <h2 className="border-b bg-gray-50 px-4 py-3 text-sm font-semibold">{title}</h2>
      <table className="w-full text-sm">
        <tbody className="divide-y">
          {rows.map((row) => (
            <tr key={row.label}>
              <td className="p-3 font-medium">{row.label}</td>
              <td className="p-3 text-right">{row.tasks} tasks</td>
              <td className="p-3 text-right">{row.hours} h</td>
              <td className="p-3 text-right">${row.fec.toLocaleString()} <span className="text-xs text-gray-500">labor-equivalent estimate</span></td>
              <td className="p-3 text-right"><ConfidenceBadge band={row.confidence} /></td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}

export default async function LeaderboardsPage() {
  const data = await getDashboardData();
  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold tracking-tight">Leaderboards</h1>
      <div className="grid gap-4 xl:grid-cols-2">
        <Board title="By Team" rows={data.leaderboards.team} />
        <Board title="By Project" rows={data.leaderboards.project} />
        <Board title="By Framework" rows={data.leaderboards.framework} />
        <Board title="By Category" rows={data.leaderboards.category} />
        <Board title="By User" rows={data.leaderboards.user} />
      </div>
    </div>
  );
}
