import { describe, it, expect } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import ColorPanel from "../components/ColorPanel";

describe("ColorPanel", () => {
  it("renders color presets", () => {
    render(
      <ColorPanel
        color="#000000"
        onColorChange={() => {}}
        brushSize={2}
        onBrushSizeChange={() => {}}
      />
    );

    expect(screen.getByText("颜色")).toBeInTheDocument();
    expect(screen.getByTestId("current-color-preview")).toBeInTheDocument();
  });

  it("calls onColorChange when preset is clicked", () => {
    let changedColor = "";
    render(
      <ColorPanel
        color="#000000"
        onColorChange={(c) => { changedColor = c; }}
        brushSize={2}
        onBrushSizeChange={() => {}}
      />
    );

    const redPreset = screen.getByTestId("color-preset-FF0000");
    fireEvent.click(redPreset);
    expect(changedColor).toBe("#FF0000");
  });

  it("displays current color value", () => {
    render(
      <ColorPanel
        color="#FF8040"
        onColorChange={() => {}}
        brushSize={2}
        onBrushSizeChange={() => {}}
      />
    );

    expect(screen.getByText("#FF8040")).toBeInTheDocument();
  });
});
