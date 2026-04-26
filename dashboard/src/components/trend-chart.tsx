"use client";

import { Line, LineChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import type { TrendPoint } from "@/lib/dashboard-data";

export function TrendChart({ points }: { points: TrendPoint[] }) {
  return (
    <div className="h-72 w-full">
      <ResponsiveContainer width="100%" height="100%">
        <LineChart data={points} margin={{ left: 8, right: 16, top: 12, bottom: 8 }}>
          <XAxis dataKey="date" tickLine={false} axisLine={false} tick={{ fontSize: 12 }} />
          <YAxis tickLine={false} axisLine={false} tick={{ fontSize: 12 }} />
          <Tooltip formatter={(value, name) => [String(value), String(name)]} />
          <Line type="monotone" dataKey="tasks" stroke="#2563eb" strokeWidth={2} dot={false} />
          <Line type="monotone" dataKey="hours" stroke="#059669" strokeWidth={2} dot={false} />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}
