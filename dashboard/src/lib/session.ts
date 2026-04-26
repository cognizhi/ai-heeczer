export type DashboardRole = "viewer" | "tester" | "admin";

export interface DashboardSession {
  user: string;
  role: DashboardRole;
  authProvider: "local" | "oidc";
}

export async function getDashboardSession(): Promise<DashboardSession> {
  const role = process.env["HEECZER_DASHBOARD_ROLE"] as
    | DashboardRole
    | undefined;
  const provider = process.env["HEECZER_OIDC_ISSUER"] ? "oidc" : "local";
  return {
    user: process.env["HEECZER_DASHBOARD_USER"] ?? "local-viewer",
    role: role === "admin" || role === "tester" ? role : "viewer",
    authProvider: provider,
  };
}

export function canAdmin(role: DashboardRole): boolean {
  return role === "admin";
}
