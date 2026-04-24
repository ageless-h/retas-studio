import { describe, it, expect, beforeEach } from "vitest";
import {
  mockDrawStroke,
  mockGetLayers,
  mockAddLayer,
  mockToggleLayerVisibility,
  mockToggleLayerLock,
  mockUndo,
  mockRedo,
  mockCanUndo,
  mockCanRedo,
  mockCompositeLayers,
} from "./api.mock";

describe("Mock API", () => {
  it("should get initial layers", async () => {
    const layers = await mockGetLayers();
    expect(layers.length).toBe(2);
    expect(layers[0].name).toBe("背景");
    expect(layers[1].name).toBe("图层 1");
  });

  it("should add a layer", async () => {
    const before = await mockGetLayers();
    const beforeCount = before.length;

    await mockAddLayer("新图层");
    const after = await mockGetLayers();
    expect(after.length).toBe(beforeCount + 1);
    expect(after[after.length - 1].name).toBe("新图层");
  });

  it("should toggle layer visibility", async () => {
    const layers = await mockGetLayers();
    const layerId = layers[0].id;
    const beforeVisible = layers[0].visible;

    const afterVisible = await mockToggleLayerVisibility(layerId);
    expect(afterVisible).toBe(!beforeVisible);
  });

  it("should toggle layer lock", async () => {
    const layers = await mockGetLayers();
    const layerId = layers[0].id;
    const beforeLocked = layers[0].locked;

    const afterLocked = await mockToggleLayerLock(layerId);
    expect(afterLocked).toBe(!beforeLocked);
  });

  it("should draw stroke on current layer", async () => {
    await mockDrawStroke({
      tool: "brush",
      points: [[100, 100], [150, 150]],
      color: [255, 0, 0],
      size: 5,
      layerId: "current",
    });

    const composite = await mockCompositeLayers();
    expect(composite.length).toBeGreaterThan(0);
  });

  it("should support undo/redo after state change", async () => {
    const initialUndo = await mockCanUndo();
    expect(initialUndo).toBe(true);

    await mockAddLayer("test-layer");
    const canUndoAfterAdd = await mockCanUndo();
    expect(canUndoAfterAdd).toBe(true);

    await mockUndo();
    const canRedoAfterUndo = await mockCanRedo();
    expect(canRedoAfterUndo).toBe(true);

    await mockRedo();
    const canRedoAfterRedo = await mockCanRedo();
    expect(canRedoAfterRedo).toBe(false);
  });
});
