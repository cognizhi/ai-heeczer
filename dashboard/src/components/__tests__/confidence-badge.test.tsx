import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { ConfidenceBadge } from "@/components/confidence-badge";

describe("ConfidenceBadge", () => {
  it.each(["High", "Medium", "Low"] as const)(
    "renders %s band with accessible label",
    (band) => {
      render(<ConfidenceBadge band={band} />);
      expect(screen.getByLabelText(`Confidence: ${band}`)).toBeInTheDocument();
      expect(screen.getByText(band)).toBeInTheDocument();
    }
  );
});
