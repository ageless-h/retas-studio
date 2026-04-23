import type {
  DrawCommand,
  LayerInfo,
  FrameInfo,
  DocumentInfo,
} from "./api";

interface RasterFrame {
  frame_number: number;
  image_data: Uint8Array;
  width: number;
  height: number;
}

interface RasterLayer {
  id: string;
  name: string;
  visible: boolean;
  locked: boolean;
  opacity: number;
  frames: Map<number, RasterFrame>;
}

interface MockDocument {
  name: string;
  width: number;
  height: number;
  frame_rate: number;
  total_frames: number;
  layers: Map<string, RasterLayer>;
  layer_order: string[];
  current_frame: number;
  selected_layers: string[];
}

function generateId(): string {
  return "mock-" + Math.random().toString(36).substring(2, 11);
}

function createMockDocument(): MockDocument {
  const width = 1920;
  const height = 1080;
  const pixelCount = width * height * 4;

  const layer1Id = generateId();
  const layer2Id = generateId();

  const layer1: RasterLayer = {
    id: layer1Id,
    name: "背景",
    visible: true,
    locked: false,
    opacity: 1.0,
    frames: new Map(),
  };

  const layer2: RasterLayer = {
    id: layer2Id,
    name: "图层 1",
    visible: true,
    locked: false,
    opacity: 1.0,
    frames: new Map(),
  };

  layer1.frames.set(0, {
    frame_number: 0,
    image_data: new Uint8Array(pixelCount).fill(255),
    width,
    height,
  });

  layer2.frames.set(0, {
    frame_number: 0,
    image_data: new Uint8Array(pixelCount).fill(0),
    width,
    height,
  });

  const layers = new Map<string, RasterLayer>();
  layers.set(layer1Id, layer1);
  layers.set(layer2Id, layer2);

  return {
    name: "未命名",
    width,
    height,
    frame_rate: 24,
    total_frames: 1,
    layers,
    layer_order: [layer1Id, layer2Id],
    current_frame: 0,
    selected_layers: [layer2Id],
  };
}

function getPersistedMockDoc(): MockDocument | undefined {
  if (typeof window !== "undefined" && (window as any).__RETAS_MOCK_DOC__) {
    return (window as any).__RETAS_MOCK_DOC__;
  }
  return undefined;
}

function setPersistedMockDoc(doc: MockDocument) {
  if (typeof window !== "undefined") {
    (window as any).__RETAS_MOCK_DOC__ = doc;
  }
}

function getPersistedHistory(): { history: MockDocument[]; historyIndex: number } | undefined {
  if (typeof window !== "undefined" && (window as any).__RETAS_MOCK_HISTORY__) {
    return (window as any).__RETAS_MOCK_HISTORY__;
  }
  return undefined;
}

function setPersistedHistory(h: MockDocument[], idx: number) {
  if (typeof window !== "undefined") {
    (window as any).__RETAS_MOCK_HISTORY__ = { history: h, historyIndex: idx };
  }
}

let mockDoc: MockDocument = getPersistedMockDoc() || createMockDocument();
const persisted = getPersistedHistory();
const history: MockDocument[] = persisted ? persisted.history : [];
let historyIndex = persisted ? persisted.historyIndex : -1;

if (!getPersistedMockDoc()) {
  setPersistedMockDoc(mockDoc);
}

if (history.length === 0) {
  const initialClone: MockDocument = {
    ...mockDoc,
    layers: new Map(),
    layer_order: [...mockDoc.layer_order],
    selected_layers: [...mockDoc.selected_layers],
  };
  mockDoc.layers.forEach((layer, id) => {
    const clonedLayer: RasterLayer = {
      ...layer,
      frames: new Map(),
    };
    layer.frames.forEach((frame, num) => {
      clonedLayer.frames.set(num, {
        ...frame,
        image_data: new Uint8Array(frame.image_data),
      });
    });
    initialClone.layers.set(id, clonedLayer);
  });
  history.push(initialClone);
  historyIndex = 0;
  setPersistedHistory(history, historyIndex);
}

function recordHistory() {
  const cloned: MockDocument = {
    ...mockDoc,
    layers: new Map(),
    layer_order: [...mockDoc.layer_order],
    selected_layers: [...mockDoc.selected_layers],
  };
  mockDoc.layers.forEach((layer, id) => {
    const clonedLayer: RasterLayer = {
      ...layer,
      frames: new Map(),
    };
    layer.frames.forEach((frame, num) => {
      clonedLayer.frames.set(num, {
        ...frame,
        image_data: new Uint8Array(frame.image_data),
      });
    });
    cloned.layers.set(id, clonedLayer);
  });
  history.splice(historyIndex + 1);
  history.push(cloned);
  if (history.length > 50) {
    history.shift();
    historyIndex = Math.max(0, historyIndex - 1);
  } else {
    historyIndex++;
  }
  setPersistedHistory(history, historyIndex);
}

function drawLineOnPixels(
  pixels: Uint8Array,
  width: number,
  height: number,
  x0: number,
  y0: number,
  x1: number,
  y1: number,
  color: [number, number, number],
  size: number,
  isEraser: boolean
) {
  const brushRadius = Math.max(1, Math.round(size));
  const dx = Math.abs(x1 - x0);
  const dy = Math.abs(y1 - y0);
  const steps = Math.max(1, Math.round(Math.max(dx, dy) / 0.5));

  for (let i = 0; i <= steps; i++) {
    const t = i / steps;
    const cx = x0 + (x1 - x0) * t;
    const cy = y0 + (y1 - y0) * t;

    for (let by = -brushRadius; by <= brushRadius; by++) {
      for (let bx = -brushRadius; bx <= brushRadius; bx++) {
        if (bx * bx + by * by > brushRadius * brushRadius) {
          continue;
        }
        const px = Math.round(cx + bx);
        const py = Math.round(cy + by);
        if (px < 0 || py < 0 || px >= width || py >= height) {
          continue;
        }
        const idx = (py * width + px) * 4;
        if (isEraser) {
          pixels[idx] = 0;
          pixels[idx + 1] = 0;
          pixels[idx + 2] = 0;
          pixels[idx + 3] = 0;
        } else {
          pixels[idx] = color[0];
          pixels[idx + 1] = color[1];
          pixels[idx + 2] = color[2];
          pixels[idx + 3] = 255;
        }
      }
    }
  }
}

function blendNormal(
  baseR: number,
  baseG: number,
  baseB: number,
  baseA: number,
  blendR: number,
  blendG: number,
  blendB: number,
  blendA: number,
  opacity: number
): [number, number, number, number] {
  const alpha = (blendA / 255) * opacity;
  const invAlpha = 1 - alpha;
  const outA = alpha + (baseA / 255) * invAlpha;
  if (outA === 0) return [0, 0, 0, 0];
  const outR = (blendR * alpha + baseR * invAlpha) / outA;
  const outG = (blendG * alpha + baseG * invAlpha) / outA;
  const outB = (blendB * alpha + baseB * invAlpha) / outA;
  return [
    Math.round(outR * outA),
    Math.round(outG * outA),
    Math.round(outB * outA),
    Math.round(outA * 255),
  ];
}

export async function mockDrawStroke(command: DrawCommand): Promise<string> {
  recordHistory();
  let layerId = command.layerId;
  if (layerId === "current") {
    layerId = mockDoc.selected_layers[0];
  }
  if (!layerId) {
    throw new Error("No selected layer");
  }

  const layer = mockDoc.layers.get(layerId);
  if (!layer) {
    throw new Error("Layer not found");
  }
  if (layer.locked) {
    throw new Error("Layer is locked");
  }

  let frame = layer.frames.get(mockDoc.current_frame);
  if (!frame) {
    const pixelCount = mockDoc.width * mockDoc.height * 4;
    frame = {
      frame_number: mockDoc.current_frame,
      image_data: new Uint8Array(pixelCount).fill(0),
      width: mockDoc.width,
      height: mockDoc.height,
    };
    layer.frames.set(mockDoc.current_frame, frame);
  }

  const isEraser = command.tool === "eraser";
  const points = command.points;

  for (let i = 0; i < points.length - 1; i++) {
    const [x0, y0] = points[i];
    const [x1, y1] = points[i + 1];
    drawLineOnPixels(
      frame.image_data,
      frame.width,
      frame.height,
      x0,
      y0,
      x1,
      y1,
      command.color as [number, number, number],
      command.size,
      isEraser
    );
  }

  return `绘制了 ${points.length} 个点`;
}

export async function mockGetLayers(): Promise<LayerInfo[]> {
  const result: LayerInfo[] = [];
  for (const layerId of mockDoc.layer_order) {
    const layer = mockDoc.layers.get(layerId);
    if (layer) {
      result.push({
        id: layer.id,
        name: layer.name,
        visible: layer.visible,
        locked: layer.locked,
        opacity: layer.opacity,
      });
    }
  }
  return result;
}

export async function mockAddLayer(name: string): Promise<LayerInfo> {
  recordHistory();
  const id = generateId();
  const layer: RasterLayer = {
    id,
    name,
    visible: true,
    locked: false,
    opacity: 1.0,
    frames: new Map(),
  };
  mockDoc.layers.set(id, layer);
  mockDoc.layer_order.push(id);
  return {
    id,
    name,
    visible: true,
    locked: false,
    opacity: 1.0,
  };
}

export async function mockDeleteLayer(id: string): Promise<void> {
  recordHistory();
  mockDoc.layers.delete(id);
  mockDoc.layer_order = mockDoc.layer_order.filter((lid) => lid !== id);
  mockDoc.selected_layers = mockDoc.selected_layers.filter((lid) => lid !== id);
}

export async function mockToggleLayerVisibility(id: string): Promise<boolean> {
  recordHistory();
  const layer = mockDoc.layers.get(id);
  if (!layer) throw new Error("Layer not found");
  layer.visible = !layer.visible;
  return layer.visible;
}

export async function mockToggleLayerLock(id: string): Promise<boolean> {
  recordHistory();
  const layer = mockDoc.layers.get(id);
  if (!layer) throw new Error("Layer not found");
  layer.locked = !layer.locked;
  return layer.locked;
}

export async function mockGetFrameInfo(): Promise<FrameInfo> {
  return {
    current: mockDoc.current_frame + 1,
    total: mockDoc.total_frames,
    fps: mockDoc.frame_rate,
  };
}

export async function mockSetCurrentFrame(frame: number): Promise<void> {
  mockDoc.current_frame = frame - 1;
  setPersistedMockDoc(mockDoc);
}

export async function mockAddFrame(): Promise<void> {
  recordHistory();
  const newFrame = mockDoc.total_frames;
  const pixelCount = mockDoc.width * mockDoc.height * 4;
  for (const layer of mockDoc.layers.values()) {
    if (!layer.frames.has(newFrame)) {
      layer.frames.set(newFrame, {
        frame_number: newFrame,
        image_data: new Uint8Array(pixelCount).fill(0),
        width: mockDoc.width,
        height: mockDoc.height,
      });
    }
  }
  mockDoc.total_frames += 1;
}

export async function mockDeleteFrame(): Promise<void> {
  recordHistory();
  if (mockDoc.total_frames > 1) {
    mockDoc.total_frames -= 1;
  }
}

export async function mockGetLayerPixels(layerId: string): Promise<Uint8Array> {
  const layer = mockDoc.layers.get(layerId);
  if (!layer) throw new Error("Layer not found");
  const frame = layer.frames.get(mockDoc.current_frame);
  if (!frame) return new Uint8Array(0);
  return new Uint8Array(frame.image_data);
}

export async function mockCompositeLayers(): Promise<Uint8Array> {
  const pixelCount = mockDoc.width * mockDoc.height * 4;
  const result = new Uint8Array(pixelCount);
  result.fill(255);

  for (const layerId of mockDoc.layer_order) {
    const layer = mockDoc.layers.get(layerId);
    if (!layer || !layer.visible) continue;

    const frame = layer.frames.get(mockDoc.current_frame);
    if (!frame) continue;

    const opacity = layer.opacity;

    for (let i = 0; i < pixelCount; i += 4) {
      const [r, g, b, a] = blendNormal(
        result[i],
        result[i + 1],
        result[i + 2],
        result[i + 3],
        frame.image_data[i],
        frame.image_data[i + 1],
        frame.image_data[i + 2],
        frame.image_data[i + 3],
        opacity
      );
      result[i] = r;
      result[i + 1] = g;
      result[i + 2] = b;
      result[i + 3] = a;
    }
  }

  return result;
}

export async function mockUndo(): Promise<boolean> {
  if (historyIndex <= 0) return false;
  historyIndex--;
  mockDoc = history[historyIndex];
  setPersistedMockDoc(mockDoc);
  setPersistedHistory(history, historyIndex);
  return true;
}

export async function mockRedo(): Promise<boolean> {
  if (historyIndex >= history.length - 1) return false;
  historyIndex++;
  mockDoc = history[historyIndex];
  setPersistedMockDoc(mockDoc);
  setPersistedHistory(history, historyIndex);
  return true;
}

export async function mockCanUndo(): Promise<boolean> {
  return historyIndex > 0;
}

export async function mockCanRedo(): Promise<boolean> {
  return historyIndex < history.length - 1;
}

export async function mockGetDocumentInfo(): Promise<DocumentInfo> {
  return {
    name: mockDoc.name,
    width: mockDoc.width,
    height: mockDoc.height,
    frame_rate: mockDoc.frame_rate,
    total_frames: mockDoc.total_frames,
  };
}

export async function mockOpenDocument(_path: string): Promise<DocumentInfo> {
  mockDoc = createMockDocument();
  history.length = 0;
  historyIndex = -1;
  setPersistedMockDoc(mockDoc);
  setPersistedHistory(history, historyIndex);
  return mockGetDocumentInfo();
}

export async function mockSelectLayer(id: string): Promise<void> {
  mockDoc.selected_layers = [id];
  setPersistedMockDoc(mockDoc);
}

export async function mockSaveDocument(_path: string): Promise<void> {
}

if (typeof window !== "undefined") {
  (window as any).__RETAS_MOCK__ = {
    getDoc: () => mockDoc,
    getHistory: () => ({ index: historyIndex, length: history.length }),
  };
}
