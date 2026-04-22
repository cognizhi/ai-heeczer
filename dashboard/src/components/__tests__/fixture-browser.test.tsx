import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { FixtureBrowser } from "@/components/test-orchestration/fixture-browser";

describe("FixtureBrowser", () => {
  it("lists all bundled fixtures including edge cases", () => {
    const onSelect = vi.fn();
    render(<FixtureBrowser selected={null} onSelect={onSelect} />);
    expect(
      screen.getByText("valid/01-prd-canonical.json")
    ).toBeInTheDocument();
    expect(
      screen.getByText("edge/01-minimum-required.json")
    ).toBeInTheDocument();
  });

  it("calls onSelect with the fixture name on click", () => {
    const onSelect = vi.fn();
    render(<FixtureBrowser selected={null} onSelect={onSelect} />);
    fireEvent.click(screen.getByText("valid/01-prd-canonical.json"));
    expect(onSelect).toHaveBeenCalledWith("valid/01-prd-canonical.json");
  });

  it("highlights the selected fixture", () => {
    const onSelect = vi.fn();
    render(
      <FixtureBrowser
        selected="valid/01-prd-canonical.json"
        onSelect={onSelect}
      />
    );
    const btn = screen.getByText("valid/01-prd-canonical.json");
    expect(btn.className).toContain("bg-blue-50");
  });
});
