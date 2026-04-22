"use client";

/**
 * FixtureBrowser — lists available scoring fixtures by name.
 * In production, fetches from the ingestion service's fixture index.
 */

// Bundled fixture names (static placeholder; replace with API call post-scaffold).
const FIXTURE_NAMES = [
  "valid/01-prd-canonical.json",
  "valid/02-min-payload.json",
  "valid/03-unicode-names.json",
  "valid/04-all-optional-fields.json",
  "valid/05-extension-passthrough.json",
] as const;

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
        {FIXTURE_NAMES.map((name) => (
          <li key={name}>
            <button
              className={`w-full text-left px-4 py-2.5 text-sm transition-colors hover:bg-gray-50 dark:hover:bg-gray-800 ${
                selected === name
                  ? "bg-blue-50 dark:bg-blue-900/20 font-medium text-blue-700 dark:text-blue-400"
                  : ""
              }`}
              onClick={() => onSelect(name)}
            >
              {name}
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}
