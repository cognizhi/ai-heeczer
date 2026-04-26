import { clsx } from "clsx";

export function StatusPill({ state }: { state: "good" | "warn" | "bad" }) {
  return (
    <span
      className={clsx(
        "inline-flex h-2.5 w-2.5 rounded-full",
        state === "good" && "bg-emerald-500",
        state === "warn" && "bg-amber-500",
        state === "bad" && "bg-red-500",
      )}
      aria-label={`Status: ${state}`}
    />
  );
}
