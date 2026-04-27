import Link from "next/link";
import { canAdmin, getDashboardSession } from "@/lib/session";

const sections = [
  { title: "Tier Management" },
  { title: "Scoring Profile Management" },
  { title: "Rate Management" },
  { title: "Audit Log" },
  { title: "Calibration Guide", href: "/admin/calibration" },
  { title: "Workspace Overrides" },
  { title: "RBAC Actions" },
];

export default async function AdminPage() {
  const session = await getDashboardSession();
  const allowed = canAdmin(session.role);
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between gap-4">
        <h1 className="text-2xl font-bold tracking-tight">Admin Console</h1>
        <span className="rounded-md border px-3 py-1 text-sm">Role: {session.role}</span>
      </div>
      {!allowed && <p role="alert" className="rounded-md border border-amber-200 bg-amber-50 p-3 text-sm text-amber-900">Admin or owner role required</p>}
      <div className="grid gap-4 lg:grid-cols-2">
        {sections.map((section) => (
          <section key={section.title} className="rounded-md border p-4">
            <div className="flex items-center justify-between gap-4">
              <h2 className="text-sm font-semibold">{section.title}</h2>
              {section.href && allowed ? (
                <Link
                  href={section.href}
                  className="rounded bg-gray-900 px-3 py-1.5 text-sm font-medium text-white"
                >
                  Open
                </Link>
              ) : (
                <button disabled={!allowed} className="rounded bg-gray-900 px-3 py-1.5 text-sm font-medium text-white disabled:cursor-not-allowed disabled:opacity-40">
                  Open
                </button>
              )}
            </div>
          </section>
        ))}
      </div>
    </div>
  );
}
