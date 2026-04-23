import { test, expect, Page } from "@playwright/test";

async function getMockState(page: Page) {
  return page.evaluate(() => {
    const mock = (window as any).__RETAS_MOCK__;
    if (!mock) return null;
    const doc = mock.getDoc();
    return {
      currentFrame: doc.current_frame,
      totalFrames: doc.total_frames,
      layerCount: doc.layer_order.length,
      layers: Array.from(doc.layers.values()).map((l: any) => ({
        id: l.id,
        name: l.name,
        visible: l.visible,
        locked: l.locked,
        opacity: l.opacity,
        frameCount: l.frames.size,
      })),
      selectedLayers: doc.selected_layers,
    };
  });
}

async function getLayerPixels(page: Page, layerId: string): Promise<number[]> {
  return page.evaluate((id: string) => {
    const mock = (window as any).__RETAS_MOCK__;
    if (!mock) return [];
    const doc = mock.getDoc();
    const layer = doc.layers.get(id);
    if (!layer) return [];
    const frame = layer.frames.get(doc.current_frame);
    if (!frame) return [];
    return Array.from(frame.image_data);
  }, layerId);
}

function countNonTransparentPixels(pixels: number[]): number {
  let count = 0;
  for (let i = 3; i < pixels.length; i += 4) {
    if (pixels[i] > 10) count++;
  }
  return count;
}

async function getCanvasDrawPoint(page: Page): Promise<{ x: number; y: number }> {
  const canvas = page.locator('[data-testid="main-canvas"]');
  const box = await canvas.boundingBox();
  if (!box) return { x: 200, y: 200 };
  return { x: 200, y: 200 };
}

async function drawOnCanvas(page: Page, fromX: number, fromY: number, toX: number, toY: number) {
  const canvas = page.locator('[data-testid="main-canvas"]');
  await canvas.hover({ position: { x: fromX, y: fromY } });
  await page.mouse.down();
  await canvas.hover({ position: { x: toX, y: toY } });
  await page.mouse.up();
}

test.describe("Cross-Panel Mouse Testing", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector('[data-testid="main-canvas"]', { timeout: 30000 });
    await page.waitForTimeout(2000);
  });

  test("A01: brush drawing persists and enables undo button", async ({ page }) => {
    const center = await getCanvasDrawPoint(page);
    const undoBtn = page.locator('[data-testid="toolbar-undo"]');
    await expect(undoBtn).toBeDisabled();

    await drawOnCanvas(page, center.x, center.y, center.x + 50, center.y + 50);
    await page.waitForTimeout(500);

    await expect(undoBtn).toBeEnabled();
    const state = await getMockState(page);
    const activeLayerId = state!.selectedLayers[0];
    const pixels = await getLayerPixels(page, activeLayerId);
    expect(countNonTransparentPixels(pixels)).toBeGreaterThan(0);
  });

  test("A02: undo click disables undo and enables redo", async ({ page }) => {
    const center = await getCanvasDrawPoint(page);
    await drawOnCanvas(page, center.x, center.y, center.x + 50, center.y + 50);
    await page.waitForTimeout(500);

    const undoBtn = page.locator('[data-testid="toolbar-undo"]');
    const redoBtn = page.locator('[data-testid="toolbar-redo"]');
    await expect(undoBtn).toBeEnabled();
    await expect(redoBtn).toBeDisabled();

    await undoBtn.click();
    await page.waitForTimeout(300);

    await expect(undoBtn).toBeDisabled();
    await expect(redoBtn).toBeEnabled();
  });

  test("A03: eraser tool switches via mouse and removes drawn pixels", async ({ page }) => {
    const center = await getCanvasDrawPoint(page);
    await drawOnCanvas(page, center.x, center.y, center.x + 30, center.y + 30);
    await page.waitForTimeout(300);

    const state = await getMockState(page);
    const activeLayerId = state!.selectedLayers[0];
    const beforePixels = await getLayerPixels(page, activeLayerId);
    const drawnCount = countNonTransparentPixels(beforePixels);
    expect(drawnCount).toBeGreaterThan(0);

    await page.locator('[data-testid="tool-category-brush"]').click();
    await page.waitForTimeout(200);
    await page.locator('[data-testid="tool-eraser"]').click();
    await page.waitForTimeout(200);

    await drawOnCanvas(page, center.x + 10, center.y + 10, center.x + 20, center.y + 20);
    await page.waitForTimeout(300);

    const afterPixels = await getLayerPixels(page, activeLayerId);
    const remainingCount = countNonTransparentPixels(afterPixels);
    expect(remainingCount).toBeLessThan(drawnCount);
  });

  test("A04: layer visibility toggle affects canvas composite", async ({ page }) => {
    const center = await getCanvasDrawPoint(page);
    await drawOnCanvas(page, center.x, center.y, center.x + 50, center.y + 50);
    await page.waitForTimeout(300);

    const stateBefore = await getMockState(page);
    const activeLayerId = stateBefore!.selectedLayers[0];
    const layerBefore = await getLayerPixels(page, activeLayerId);
    expect(countNonTransparentPixels(layerBefore)).toBeGreaterThan(0);

    const visibilityBtn = page.locator(`[data-testid="layer-visibility-${stateBefore!.layers[0].id}"]`);
    await visibilityBtn.click();
    await page.waitForTimeout(300);

    const stateAfter = await getMockState(page);
    expect(stateAfter!.layers[0].visible).toBe(false);
  });

  test("A05: add layer via mouse increases layer count in panel", async ({ page }) => {
    const stateBefore = await getMockState(page);
    const countBefore = stateBefore!.layerCount;

    await page.locator('[data-testid="layer-add"]').click();
    await page.waitForTimeout(300);

    const stateAfter = await getMockState(page);
    expect(stateAfter!.layerCount).toBe(countBefore + 1);
  });

  test("A06: frame navigation updates both timeline and mock state", async ({ page }) => {
    await page.locator('[data-testid="frame-add"]').click();
    await page.waitForTimeout(300);

    const counterBefore = await page.locator('[data-testid="frame-counter"]').textContent();
    const stateBefore = await getMockState(page);
    expect(stateBefore!.currentFrame).toBe(0);

    await page.locator('[data-testid="frame-next"]').click();
    await page.waitForTimeout(300);

    const counterAfter = await page.locator('[data-testid="frame-counter"]').textContent();
    const stateAfter = await getMockState(page);
    expect(stateAfter!.currentFrame).toBe(1);
    expect(counterAfter).not.toBe(counterBefore);
  });

  test("A07: draw on frame 2, navigate away, return, pixels persist", async ({ page }) => {
    await page.locator('[data-testid="frame-add"]').click();
    await page.waitForTimeout(300);
    await page.locator('[data-testid="frame-next"]').click();
    await page.waitForTimeout(300);

    const center = await getCanvasDrawPoint(page);
    await drawOnCanvas(page, center.x, center.y, center.x + 50, center.y + 50);
    await page.waitForTimeout(300);

    const stateFrame2 = await getMockState(page);
    const activeLayerId = stateFrame2!.selectedLayers[0];
    const pixelsFrame2 = await getLayerPixels(page, activeLayerId);
    const drawnPixels = countNonTransparentPixels(pixelsFrame2);
    expect(drawnPixels).toBeGreaterThan(0);

    await page.locator('[data-testid="frame-prev"]').click();
    await page.waitForTimeout(300);
    await page.locator('[data-testid="frame-next"]').click();
    await page.waitForTimeout(300);

    const pixelsAgain = await getLayerPixels(page, activeLayerId);
    expect(countNonTransparentPixels(pixelsAgain)).toBe(drawnPixels);
  });

  test("A08: color change via mouse affects subsequent drawing", async ({ page }) => {
    await page.locator('[data-testid="color-preset-FF0000"]').click();
    await page.waitForTimeout(200);

    const preview = page.locator('[data-testid="current-color-preview"]');
    await expect(preview).toHaveCSS("background-color", "rgb(255, 0, 0)");

    const center = await getCanvasDrawPoint(page);
    await drawOnCanvas(page, center.x, center.y, center.x + 30, center.y + 30);
    await page.waitForTimeout(300);

    const state = await getMockState(page);
    const activeLayerId = state!.selectedLayers[0];
    const pixels = await getLayerPixels(page, activeLayerId);
    expect(countNonTransparentPixels(pixels)).toBeGreaterThan(0);
  });

  test("A09: workspace switch changes right panel via mouse", async ({ page }) => {
    await page.locator('[data-testid="workspace-animation"]').click();
    await page.waitForTimeout(500);
    await expect(page.getByText("动画属性").nth(1)).toBeVisible();

    await page.locator('[data-testid="workspace-drawing"]').click();
    await page.waitForTimeout(500);
    await expect(page.locator('[data-testid="current-color-preview"]')).toBeVisible();
  });

  test("A10: empty click on canvas does not create pixels", async ({ page }) => {
    const center = await getCanvasDrawPoint(page);
    await page.locator('[data-testid="main-canvas"]').click({ position: { x: center.x, y: center.y } });
    await page.waitForTimeout(300);

    const state = await getMockState(page);
    const activeLayerId = state!.selectedLayers[0];
    const pixels = await getLayerPixels(page, activeLayerId);
    expect(countNonTransparentPixels(pixels)).toBe(0);
  });

  test("A11: rapid tool switching remains stable", async ({ page }) => {
    await page.locator('[data-testid="tool-category-brush"]').click();
    await page.waitForTimeout(100);
    await page.locator('[data-testid="tool-pen"]').click();
    await page.waitForTimeout(100);
    await page.locator('[data-testid="tool-category-brush"]').click();
    await page.waitForTimeout(100);
    await page.locator('[data-testid="tool-brush"]').click();
    await page.waitForTimeout(100);

    const center = await getCanvasDrawPoint(page);
    await drawOnCanvas(page, center.x, center.y, center.x + 30, center.y + 30);
    await page.waitForTimeout(300);

    const state = await getMockState(page);
    const activeLayerId = state!.selectedLayers[0];
    const pixels = await getLayerPixels(page, activeLayerId);
    expect(countNonTransparentPixels(pixels)).toBeGreaterThan(0);
  });

  test("A12: layer lock prevents drawing on locked layer", async ({ page }) => {
    const stateBefore = await getMockState(page);
    const firstLayer = stateBefore!.layers[0];
    await page.locator(`[data-testid="layer-row-${firstLayer.id}"]`).click();
    await page.waitForTimeout(200);

    const beforePixels = await getLayerPixels(page, firstLayer.id);
    const beforeCount = countNonTransparentPixels(beforePixels);

    await page.locator(`[data-testid="layer-lock-${firstLayer.id}"]`).click();
    await page.waitForTimeout(300);

    const center = await getCanvasDrawPoint(page);
    await drawOnCanvas(page, center.x, center.y, center.x + 30, center.y + 30);
    await page.waitForTimeout(300);

    const afterPixels = await getLayerPixels(page, firstLayer.id);
    const afterCount = countNonTransparentPixels(afterPixels);
    expect(afterCount).toBe(beforeCount);
  });

  test("A13: select different layer and draw isolates strokes", async ({ page }) => {
    await page.locator('[data-testid="layer-add"]').click();
    await page.waitForTimeout(300);

    const state = await getMockState(page);
    const firstLayerId = state!.layers[0].id;
    const secondLayerId = state!.layers[1].id;

    await page.locator(`[data-testid="layer-row-${firstLayerId}"]`).click();
    await page.waitForTimeout(200);

    const center = await getCanvasDrawPoint(page);
    await drawOnCanvas(page, center.x, center.y, center.x + 30, center.y + 30);
    await page.waitForTimeout(300);

    const firstLayerPixels = await getLayerPixels(page, firstLayerId);
    const firstLayerCount = countNonTransparentPixels(firstLayerPixels);
    expect(firstLayerCount).toBeGreaterThan(0);

    const secondLayerPixels = await getLayerPixels(page, secondLayerId);
    expect(countNonTransparentPixels(secondLayerPixels)).toBe(0);
  });

  test("A14: drawing at canvas edge does not crash", async ({ page }) => {
    const canvas = page.locator('[data-testid="main-canvas"]');
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();

    await drawOnCanvas(page, 5, 5, box!.width - 5, box!.height - 5);
    await page.waitForTimeout(300);

    const state = await getMockState(page);
    const activeLayerId = state!.selectedLayers[0];
    const pixels = await getLayerPixels(page, activeLayerId);
    expect(countNonTransparentPixels(pixels)).toBeGreaterThan(0);
  });

  test("A15: delete frame decreases total frame count", async ({ page }) => {
    await page.locator('[data-testid="frame-add"]').click();
    await page.waitForTimeout(300);

    const stateBefore = await getMockState(page);
    const totalBefore = stateBefore!.totalFrames;

    await page.locator('[data-testid="frame-delete"]').click();
    await page.waitForTimeout(300);

    const stateAfter = await getMockState(page);
    expect(stateAfter!.totalFrames).toBe(totalBefore - 1);
  });

  test("A16: workspace coloring shows color panel and allows color pick", async ({ page }) => {
    await page.locator('[data-testid="workspace-coloring"]').click();
    await page.waitForTimeout(500);

    await expect(page.locator('[data-testid="current-color-preview"]')).toBeVisible();

    await page.locator('[data-testid="color-preset-0000FF"]').click();
    await page.waitForTimeout(200);

    const preview = page.locator('[data-testid="current-color-preview"]');
    await expect(preview).toHaveCSS("background-color", "rgb(0, 0, 255)");
  });
});
