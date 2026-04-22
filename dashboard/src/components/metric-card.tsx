/**
 * MetricCard — displays a labeled numeric metric with optional disclaimer.
 * PRD §21.3: every financial number must be labeled "labor-equivalent estimate".
 */
interface MetricCardProps {
  label: string;
  value: string | number;
  unit?: string;
  disclaimer?: string;
}

export function MetricCard({ label, value, unit, disclaimer }: MetricCardProps) {
  return (
    <div className="rounded-lg border p-4">
      <p className="text-xs font-medium text-gray-500 uppercase tracking-wide">
        {label}
      </p>
      <p className="mt-1 text-2xl font-bold">
        {value}
        {unit && (
          <span className="ml-1 text-sm font-normal text-gray-500">{unit}</span>
        )}
      </p>
      {disclaimer && (
        <p className="mt-1 text-xs text-gray-400 italic">{disclaimer}</p>
      )}
    </div>
  );
}
