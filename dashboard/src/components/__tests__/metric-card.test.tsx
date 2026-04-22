import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { MetricCard } from "@/components/metric-card";

describe("MetricCard", () => {
  it("renders label and value", () => {
    render(<MetricCard label="Total Tasks" value="42" unit="tasks" />);
    expect(screen.getByText("Total Tasks")).toBeInTheDocument();
    expect(screen.getByText("42")).toBeInTheDocument();
    expect(screen.getByText("tasks")).toBeInTheDocument();
  });

  it("renders disclaimer when provided", () => {
    render(
      <MetricCard
        label="Cost"
        value="100"
        unit="USD"
        disclaimer="labor-equivalent estimate"
      />
    );
    expect(
      screen.getByText("labor-equivalent estimate")
    ).toBeInTheDocument();
  });

  it("omits disclaimer when not provided", () => {
    render(<MetricCard label="Tasks" value="0" />);
    expect(
      screen.queryByText("labor-equivalent estimate")
    ).not.toBeInTheDocument();
  });
});
