"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { clsx } from "clsx";

const navItems = [
  { href: "/", label: "Overview" },
  { href: "/trends", label: "Trends" },
  { href: "/leaderboards", label: "Leaderboards" },
  { href: "/queue", label: "Queue" },
  { href: "/admin", label: "Admin" },
  { href: "/test-orchestration", label: "Test Orchestration" },
  { href: "/settings", label: "Settings" },
] as const;

export function Navigation() {
  const pathname = usePathname();

  return (
    <nav className="border-b bg-white dark:bg-gray-900">
      <div className="container mx-auto px-4 flex items-center h-14 gap-6">
        <span className="font-semibold text-sm tracking-tight">
          ai-heeczer
        </span>
        <div className="flex gap-1 overflow-x-auto">
          {navItems.map(({ href, label }) => (
            <Link
              key={href}
              href={href}
              className={clsx(
                "px-3 py-1.5 rounded text-sm font-medium transition-colors",
                pathname === href
                  ? "bg-gray-100 dark:bg-gray-800 text-gray-900 dark:text-white"
                  : "text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
              )}
            >
              {label}
            </Link>
          ))}
        </div>
      </div>
    </nav>
  );
}
