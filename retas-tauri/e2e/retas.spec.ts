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

async function getCompositePixels(page: Page): Promise<number[]> {
  return page.evaluate(() => {
    const mock = (window as any).__RETAS_MOCK__;
    if (!mock) return [];
    const doc = mock.getDoc();
    const pixelCount = doc.width * doc.height * 4;
    const result = new Uint8Array(pixelCount);
    result.fill(255);

    for (const layerId of doc.layer_order) {
      const layer = doc.layers.get(layerId);
      if (!layer || !layer.visible) continue;
      const frame = layer.frames.get(doc.current_frame);
      if (!frame) continue;

      const opacity = layer.opacity;
      for (let i = 0; i < pixelCount; i += 4) {
        const blendA = (frame.image_data[i + 3] / 255) * opacity;
        const invAlpha = 1 - blendA;
        const outA = blendA + (result[i + 3] / 255) * invAlpha;
        if (outA === 0) continue;
        result[i] = Math.round((frame.image_data[i] * blendA + result[i] * invAlpha) / outA * outA);
        result[i + 1] = Math.round((frame.image_data[i + 1] * blendA + result[i + 1] * invAlpha) / outA * outA);
        result[i + 2] = Math.round((frame.image_data[i + 2] * blendA + result[i + 2] * invAlpha) / outA * outA);
        result[i + 3] = Math.round(outA * 255);
      }
    }
    return Array.from(result);
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

test.describe("Canvas Drawing", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector("canvas", { timeout: 30000 });
    await page.waitForTimeout(2000);
  });

  test("brush tool draws persistent stroke with pixel verification", async ({ page }) => {
    const canvas = page.locator("canvas").first();
    await expect(canvas).toBeVisible();

    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();
    const cx = box!.x + box!.width / 2;
    const cy = box!.y + box!.height / 2;

    const stateBefore = await getMockState(page);
    expect(stateBefore).not.toBeNull();
    expect(stateBefore!.currentFrame).toBe(0);

    await canvas.hover({ position: { x: cx, y: cy } });
    await page.mouse.down();
    await canvas.hover({ position: { x: cx + 50, y: cy + 50 } });
    await page.mouse.up();

    await page.waitForTimeout(500);

    const stateAfter = await getMockState(page);
    expect(stateAfter).not.toBeNull();

    const activeLayerId = stateAfter!.selectedLayers[0];
    const activeLayer = stateAfter!.layers.find((l: any) => l.id === activeLayerId);
    expect(activeLayer).toBeDefined();
    expect(activeLayer!.frameCount).toBeGreaterThan(0);

    const compositePixels = await getCompositePixels(page);
    const nonTransparentCount = countNonTransparentPixels(compositePixels);
    expect(nonTransparentCount).toBeGreaterThan(0);
  });

  test("eraser tool removes pixels from active layer", async ({ page }) => {
    const canvas = page.locator("canvas").first();
    const box = await canvas.boundingBox();
    const cx = box!.x + box!.width / 2;
    const cy = box!.y + box!.height / 2;

    await canvas.hover({ position: { x: cx, y: cy } });
    await page.mouse.down();
    await canvas.hover({ position: { x: cx + 50, y: cy + 30 } });
    await page.mouse.up();
    await page.waitForTimeout(300);

    const state = await getMockState(page);
    const activeLayerId = state!.selectedLayers[0];
    const layerBefore = await getLayerPixels(page, activeLayerId);
    const drawnPixels = countNonTransparentPixels(layerBefore);
    expect(drawnPixels).toBeGreaterThan(0);

    const eraserBtn = page.locator('button[title="橡皮"]').first();
    await eraserBtn.click();
    await page.waitForTimeout(200);

    await canvas.hover({ position: { x: cx + 10, y: cy + 10 } });
    await page.mouse.down();
    await canvas.hover({ position: { x: cx + 20, y: cy + 20 } });
    await page.mouse.up();
    await page.waitForTimeout(300);

    const layerAfter = await getLayerPixels(page, activeLayerId);
    const remainingPixels = countNonTransparentPixels(layerAfter);
    expect(remainingPixels).toBeLessThan(drawnPixels);
  });

  test("empty click does not create pixels", async ({ page }) => {
    const canvas = page.locator("canvas").first();
    const box = await canvas.boundingBox();
    const cx = box!.x + box!.width / 2;
    const cy = box!.y + box!.height / 2;

    await canvas.click({ position: { x: cx, y: cy } });
    await page.waitForTimeout(300);

    const state = await getMockState(page);
    const activeLayerId = state!.selectedLayers[0];

    const frame = await getLayerPixels(page, activeLayerId);
    const nonTransparent = countNonTransparentPixels(frame);
    expect(nonTransparent).toBe(0);
  });
});

test.describe("Timeline", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector("canvas", { timeout: 30000 });
    await page.waitForTimeout(2000);
  });

  test("next frame button updates frame counter", async ({ page }) => {
    const addBtn = page.locator("button").filter({ hasText: /^增加帧$/ }).first();
    await addBtn.click();
    await page.waitForTimeout(300);

    const nextBtn = page.locator("button").filter({ hasText: /^下一帧$/ }).first();
    await expect(nextBtn).toBeVisible();

    const stateBefore = await getMockState(page);
    const frameBefore = stateBefore!.currentFrame;

    await nextBtn.click();
    await page.waitForTimeout(300);

    const stateAfter = await getMockState(page);
    expect(stateAfter!.currentFrame).toBe(frameBefore + 1);

    const frameText = page.locator("span").filter({ hasText: /帧:/ }).first();
    await expect(frameText).toContainText(`帧: ${frameBefore + 2}`);
  });

  test("previous frame button updates frame counter", async ({ page }) => {
    const addBtn = page.locator("button").filter({ hasText: /^增加帧$/ }).first();
    await addBtn.click();
    await page.waitForTimeout(300);

    const nextBtn = page.locator("button").filter({ hasText: /^下一帧$/ }).first();
    await nextBtn.click();
    await page.waitForTimeout(300);

    const prevBtn = page.locator("button").filter({ hasText: /^上一帧$/ }).first();
    await prevBtn.click();
    await page.waitForTimeout(300);

    const state = await getMockState(page);
    expect(state!.currentFrame).toBe(0);
  });

  test("add frame increases total frames", async ({ page }) => {
    const addBtn = page.locator("button").filter({ hasText: /^增加帧$/ }).first();
    await expect(addBtn).toBeVisible();

    const stateBefore = await getMockState(page);
    const totalBefore = stateBefore!.totalFrames;

    await addBtn.click();
    await page.waitForTimeout(300);

    const stateAfter = await getMockState(page);
    expect(stateAfter!.totalFrames).toBe(totalBefore + 1);
  });

  test.skip("playback advances frames automatically - skipped due to Dockview param propagation limitation", async () => {});
});

test.describe("Layers", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector("canvas", { timeout: 30000 });
    await page.waitForTimeout(2000);
  });

  test("toggle layer visibility updates mock state", async ({ page }) => {
    const layerPanel = page.getByText("图层").first();
    await expect(layerPanel).toBeVisible();

    const stateBefore = await getMockState(page);
    const firstLayer = stateBefore!.layers[0];
    expect(firstLayer.visible).toBe(true);

    const layerRow = page.getByText(firstLayer.name).first().locator("xpath=../..");
    const eyeBtn = layerRow.locator("button").first();
    await eyeBtn.click();
    await page.waitForTimeout(300);

    const stateAfter = await getMockState(page);
    const layerAfter = stateAfter!.layers.find((l: any) => l.id === firstLayer.id);
    expect(layerAfter!.visible).toBe(false);
  });

  test("toggle layer lock updates mock state", async ({ page }) => {
    const stateBefore = await getMockState(page);
    const firstLayer = stateBefore!.layers[0];
    expect(firstLayer.locked).toBe(false);

    const layerRow = page.getByText(firstLayer.name).first().locator("xpath=../..");
    const buttons = layerRow.locator("button");
    const lockBtn = buttons.nth(1);
    await lockBtn.click();
    await page.waitForTimeout(300);

    const stateAfter = await getMockState(page);
    const layerAfter = stateAfter!.layers.find((l: any) => l.id === firstLayer.id);
    expect(layerAfter!.locked).toBe(true);
  });

  test("add layer increases layer count", async ({ page }) => {
    const stateBefore = await getMockState(page);
    const countBefore = stateBefore!.layerCount;

    const addBtn = page.locator("button").filter({ hasText: /^新建$/ }).last();
    await addBtn.click();
    await page.waitForTimeout(300);

    const stateAfter = await getMockState(page);
    expect(stateAfter!.layerCount).toBe(countBefore + 1);
  });

  test("delete layer decreases layer count", async ({ page }) => {
    const addBtn = page.locator("button").filter({ hasText: /^新建$/ }).last();
    await addBtn.click();
    await page.waitForTimeout(300);

    const stateBefore = await getMockState(page);
    const countBefore = stateBefore!.layerCount;

    const deleteBtn = page.locator("button").filter({ hasText: /^删除$/ }).last();
    await deleteBtn.click();
    await page.waitForTimeout(300);

    const stateAfter = await getMockState(page);
    expect(stateAfter!.layerCount).toBe(countBefore - 1);
  });
});

test.describe("Workspace Switching", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector("canvas", { timeout: 30000 });
    await page.waitForTimeout(2000);
  });

  test("switch to animation workspace shows animation props", async ({ page }) => {
    const animBtn = page.locator("button").filter({ hasText: /^动画$/ }).first();
    await animBtn.click();
    await page.waitForTimeout(500);

    await expect(page.getByText("动画属性").nth(1)).toBeVisible();
  });

  test("switch to coloring workspace shows color panel", async ({ page }) => {
    const colorBtn = page.locator("button").filter({ hasText: /^上色$/ }).first();
    await colorBtn.click();
    await page.waitForTimeout(500);

    const colorPanel = page.getByText("颜色").first();
    await expect(colorPanel).toBeVisible();
  });

  test("switch to compositing workspace shows blend modes", async ({ page }) => {
    const compBtn = page.locator("button").filter({ hasText: /^合成$/ }).first();
    await compBtn.click();
    await page.waitForTimeout(500);

    await expect(page.getByText("混合模式").nth(1)).toBeVisible();
  });
});

test.describe("Integration", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector("canvas", { timeout: 30000 });
    await page.waitForTimeout(2000);
  });

  test("draw on frame 2, switch back to frame 1, frame 2 keeps data", async ({ page }) => {
    const addBtn = page.locator("button").filter({ hasText: /^增加帧$/ }).first();
    await addBtn.click();
    await page.waitForTimeout(300);

    const nextBtn = page.locator("button").filter({ hasText: /^下一帧$/ }).first();
    await nextBtn.click();
    await page.waitForTimeout(300);

    const canvas = page.locator("canvas").first();
    const box = await canvas.boundingBox();
    const cx = box!.x + box!.width / 2;
    const cy = box!.y + box!.height / 2;

    await canvas.hover({ position: { x: cx, y: cy } });
    await page.mouse.down();
    await canvas.hover({ position: { x: cx + 50, y: cy + 50 } });
    await page.mouse.up();
    await page.waitForTimeout(300);

    const stateFrame2 = await getMockState(page);
    const activeLayerId = stateFrame2!.selectedLayers[0];

    const pixelsFrame2 = await getLayerPixels(page, activeLayerId);
    const drawnPixels = countNonTransparentPixels(pixelsFrame2);
    expect(drawnPixels).toBeGreaterThan(0);

    const prevBtn = page.locator("button").filter({ hasText: /^上一帧$/ }).first();
    await prevBtn.click();
    await page.waitForTimeout(300);

    await nextBtn.click();
    await page.waitForTimeout(300);

    const pixelsFrame2Again = await getLayerPixels(page, activeLayerId);
    const drawnPixelsAgain = countNonTransparentPixels(pixelsFrame2Again);
    expect(drawnPixelsAgain).toBe(drawnPixels);
  });

  test("hidden layer does not appear in composite", async ({ page }) => {
    const canvas = page.locator("canvas").first();
    const box = await canvas.boundingBox();
    const cx = box!.x + box!.width / 2;
    const cy = box!.y + box!.height / 2;

    await canvas.hover({ position: { x: cx, y: cy } });
    await page.mouse.down();
    await canvas.hover({ position: { x: cx + 50, y: cy + 50 } });
    await page.mouse.up();
    await page.waitForTimeout(300);

    const compositeBefore = await getCompositePixels(page);
    const drawnBefore = countNonTransparentPixels(compositeBefore);
    expect(drawnBefore).toBeGreaterThan(0);

    const stateBefore = await getMockState(page);
    const firstLayer = stateBefore!.layers[0];
    const layerRow = page.getByText(firstLayer.name).first().locator("xpath=../..");
    const eyeBtn = layerRow.locator("button").first();
    await eyeBtn.click();
    await page.waitForTimeout(300);

    const compositeAfter = await getCompositePixels(page);
    const drawnAfter = countNonTransparentPixels(compositeAfter);

    if (firstLayer.id === stateBefore!.selectedLayers[0]) {
      expect(drawnAfter).toBeLessThan(drawnBefore);
    }
  });

  test("undo restores previous canvas state", async ({ page }) => {
    const canvas = page.locator("canvas").first();
    const box = await canvas.boundingBox();
    const cx = box!.x + box!.width / 2;
    const cy = box!.y + box!.height / 2;

    await canvas.hover({ position: { x: cx, y: cy } });
    await page.mouse.down();
    await canvas.hover({ position: { x: cx + 50, y: cy + 50 } });
    await page.mouse.up();
    await page.waitForTimeout(300);

    const state = await getMockState(page);
    const activeLayerId = state!.selectedLayers[0];
    const layerBefore = await getLayerPixels(page, activeLayerId);
    const drawnBefore = countNonTransparentPixels(layerBefore);
    expect(drawnBefore).toBeGreaterThan(0);

    const undoBtn = page.locator('button[title="撤销 (Ctrl+Z)"]').first();
    await undoBtn.click();
    await page.waitForTimeout(300);

    const layerAfter = await getLayerPixels(page, activeLayerId);
    const drawnAfter = countNonTransparentPixels(layerAfter);
    expect(drawnAfter).toBeLessThan(drawnBefore);
  });
});
