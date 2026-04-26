"use client";

import { useEffect, useState } from "react";

export function SettingsPanel() {
  const [density, setDensity] = useState("comfortable");
  const [range, setRange] = useState("7d");

  useEffect(() => {
    setDensity(localStorage.getItem("heeczer.density") ?? "comfortable");
    setRange(localStorage.getItem("heeczer.range") ?? "7d");
  }, []);

  function persist(nextDensity: string, nextRange: string) {
    setDensity(nextDensity);
    setRange(nextRange);
    localStorage.setItem("heeczer.density", nextDensity);
    localStorage.setItem("heeczer.range", nextRange);
  }

  return (
    <section className="rounded-md border p-4 space-y-4">
      <h1 className="text-2xl font-bold tracking-tight">Settings</h1>
      <label className="block text-sm font-medium">
        Date range
        <select
          className="mt-1 block w-full rounded-md border px-3 py-2 text-sm"
          value={range}
          onChange={(event) => persist(density, event.target.value)}
        >
          <option value="7d">7 days</option>
          <option value="30d">30 days</option>
          <option value="90d">90 days</option>
        </select>
      </label>
      <label className="block text-sm font-medium">
        Table density
        <select
          className="mt-1 block w-full rounded-md border px-3 py-2 text-sm"
          value={density}
          onChange={(event) => persist(event.target.value, range)}
        >
          <option value="comfortable">Comfortable</option>
          <option value="compact">Compact</option>
        </select>
      </label>
      <p className="text-sm text-gray-500" aria-live="polite">
        Saved: {range} / {density}
      </p>
    </section>
  );
}
