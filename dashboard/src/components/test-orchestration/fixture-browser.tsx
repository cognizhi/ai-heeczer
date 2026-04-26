"use client";

import { FIXTURES } from "@/lib/fixture-catalog";

interface FixtureBrowserProps {
  selected: string | null;
  onSelect: (name: string) => void;
}

export function FixtureBrowser({ selected, onSelect }: FixtureBrowserProps) {
  return (
    <div className="rounded-lg border overflow-hidden">
      <div className="bg-gray-50 dark:bg-gray-900 px-4 py-2 border-b">
        <h2 className="text-sm font-semibold">Fixtures</h2>
      </div>
      <ul className="divide-y">
        {FIXTURES.map((fixture) => (
          <li key={fixture.name}>
            <button
              className={`w-full text-left px-4 py-2.5 text-sm transition-colors hover:bg-gray-50 dark:hover:bg-gray-800 ${
                selected === fixture.name
                  ? "bg-blue-50 dark:bg-blue-900/20 font-medium text-blue-700 dark:text-blue-400"
                  : ""
              }`}
              onClick={() => onSelect(fixture.name)}
            >
              {fixture.name}
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}
