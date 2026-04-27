export type DashboardRole = "viewer" | "analyst" | "admin" | "owner";

export interface DashboardSession {
  user: string;
  role: DashboardRole;
  authProvider: "local" | "oidc";
}

export function normalizeDashboardRole(role?: string): DashboardRole {
  switch (role) {
    case "owner":
    case "admin":
    case "analyst":
      return role;
    case "tester":
      return "analyst";
    default:
      return "viewer";
  }
}

export async function getDashboardSession(): Promise<DashboardSession> {
  const role = process.env["HEECZER_DASHBOARD_ROLE"];
  const provider = process.env["HEECZER_OIDC_ISSUER"] ? "oidc" : "local";
  return {
    user: process.env["HEECZER_DASHBOARD_USER"] ?? "local-viewer",
    role: normalizeDashboardRole(role),
    authProvider: provider,
  };
}

export function canAdmin(role: DashboardRole): boolean {
  return role === "admin" || role === "owner";
}
