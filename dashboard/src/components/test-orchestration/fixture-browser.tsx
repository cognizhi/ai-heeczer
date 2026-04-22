"use client";

/**
 * FixtureBrowser — lists available scoring fixtures by name.
 * TODO (plan 0004): replace with an API call to a fixture-index endpoint once
 * the ingestion service exposes one. For now, the list is bundled statically
 * and matches the files in core/schema/fixtures/.
 */

// Bundled fixture names (static placeholder; replace with API call post-scaffold).
// Names match the actual fixture files in core/schema/fixtures/.
const FIXTURE_NAMES = [
  "valid/01-prd-canonical.json",
  "edge/01-minimum-required.json",
  "edge/02-missing-category.json",
  "edge/03-extensions-passthrough.json",
  "edge/04-unicode.json",
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
