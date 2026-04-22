import { invoke } from "@tauri-apps/api/core";

const isTauri = typeof window !== "undefined" && !!(window as any).__TAURI__;

function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri) {
    return Promise.reject(new Error("Tauri not available"));
  }
  return invoke(cmd, args);
}

export interface DrawCommand {
  tool: string;
  points: [number, number][];
  color: [number, number, number];
  size: number;
  layerId: string;
}

export interface LayerInfo {
  id: string;
  name: string;
  visible: boolean;
  locked: boolean;
  opacity: number;
}

export interface FrameInfo {
  current: number;
  total: number;
  fps: number;
}

export interface DocumentInfo {
  name: string;
  width: number;
  height: number;
  frame_rate: number;
  total_frames: number;
}

const mockLayers: LayerInfo[] = [
  { id: "bg-1", name: "背景", visible: true, locked: false, opacity: 1.0 },
  { id: "layer-1", name: "图层 1", visible: true, locked: false, opacity: 1.0 },
];

const mockHistory = {
  undoStack: [] as LayerInfo[][],
  redoStack: [] as LayerInfo[][],
};

const mockFrameInfo: FrameInfo = { current: 1, total: 100, fps: 24 };

export async function drawStroke(command: DrawCommand): Promise<string> {
  return safeInvoke("draw_stroke", { command });
}

export async function getLayers(): Promise<LayerInfo[]> {
  try {
    return await safeInvoke<LayerInfo[]>("get_layers");
  } catch {
    return [...mockLayers];
  }
}

export async function addLayer(name: string): Promise<LayerInfo> {
  try {
    return await safeInvoke<LayerInfo>("add_layer", { name });
  } catch {
    mockHistory.undoStack.push(mockLayers.map(l => ({ ...l })));
    mockHistory.redoStack = [];
    const newLayer: LayerInfo = {
      id: `layer-${mockLayers.length + 1}`,
      name,
      visible: true,
      locked: false,
      opacity: 1.0,
    };
    mockLayers.push(newLayer);
    return newLayer;
  }
}

export async function deleteLayer(id: string): Promise<void> {
  try {
    return await safeInvoke("delete_layer", { id });
  } catch {
    mockHistory.undoStack.push(mockLayers.map(l => ({ ...l })));
    mockHistory.redoStack = [];
    const idx = mockLayers.findIndex(l => l.id === id);
    if (idx >= 0) mockLayers.splice(idx, 1);
  }
}

export async function toggleLayerVisibility(id: string): Promise<boolean> {
  try {
    return await safeInvoke<boolean>("toggle_layer_visibility", { id });
  } catch {
    mockHistory.undoStack.push(mockLayers.map(l => ({ ...l })));
    mockHistory.redoStack = [];
    const layer = mockLayers.find(l => l.id === id);
    if (layer) layer.visible = !layer.visible;
    return layer?.visible ?? true;
  }
}

export async function toggleLayerLock(id: string): Promise<boolean> {
  try {
    return await safeInvoke<boolean>("toggle_layer_lock", { id });
  } catch {
    mockHistory.undoStack.push(mockLayers.map(l => ({ ...l })));
    mockHistory.redoStack = [];
    const layer = mockLayers.find(l => l.id === id);
    if (layer) layer.locked = !layer.locked;
    return layer?.locked ?? false;
  }
}

export async function getFrameInfo(): Promise<FrameInfo> {
  try {
    return await safeInvoke<FrameInfo>("get_frame_info");
  } catch {
    return { ...mockFrameInfo };
  }
}

export async function setCurrentFrame(frame: number): Promise<void> {
  try {
    return await safeInvoke("set_current_frame", { frame });
  } catch {
    mockFrameInfo.current = frame;
  }
}

export async function addFrame(): Promise<void> {
  return safeInvoke("add_frame");
}

export async function deleteFrame(): Promise<void> {
  return safeInvoke("delete_frame");
}

export async function getLayerPixels(layerId: string): Promise<Uint8Array> {
  return safeInvoke("get_layer_pixels", { layerId });
}

export async function compositeLayers(): Promise<Uint8Array> {
  return safeInvoke("composite_layers");
}

export async function undo(): Promise<boolean> {
  try {
    return await safeInvoke<boolean>("undo");
  } catch {
    if (mockHistory.undoStack.length === 0) return false;
    mockHistory.redoStack.push(mockLayers.map(l => ({ ...l })));
    const prev = mockHistory.undoStack.pop()!;
    mockLayers.length = 0;
    mockLayers.push(...prev);
    return true;
  }
}

export async function redo(): Promise<boolean> {
  try {
    return await safeInvoke<boolean>("redo");
  } catch {
    if (mockHistory.redoStack.length === 0) return false;
    mockHistory.undoStack.push(mockLayers.map(l => ({ ...l })));
    const next = mockHistory.redoStack.pop()!;
    mockLayers.length = 0;
    mockLayers.push(...next);
    return true;
  }
}

export async function canUndo(): Promise<boolean> {
  try {
    return await safeInvoke<boolean>("can_undo");
  } catch {
    return mockHistory.undoStack.length > 0;
  }
}

export async function canRedo(): Promise<boolean> {
  try {
    return await safeInvoke<boolean>("can_redo");
  } catch {
    return mockHistory.redoStack.length > 0;
  }
}

const mockDocument: DocumentInfo = {
  name: "未命名",
  width: 1920,
  height: 1080,
  frame_rate: 24,
  total_frames: 100,
};

export async function openDocument(path: string): Promise<DocumentInfo> {
  try {
    return await safeInvoke<DocumentInfo>("open_document", { path });
  } catch {
    console.log("Mock open document:", path);
    return { ...mockDocument };
  }
}

export async function saveDocument(path: string): Promise<void> {
  try {
    return await safeInvoke("save_document", { path });
  } catch {
    console.log("Mock save document:", path);
  }
}

export interface AiProcessRequest {
  feature: string;
  imageData: number[];
  params: Record<string, unknown>;
}

export interface AiProcessResult {
  success: boolean;
  imageData: number[] | null;
  error: string | null;
  processingTimeMs: number;
}

export interface AiQueueStatus {
  pending: number;
  maxSize: number;
  isFull: boolean;
}

export async function aiAutoColor(imageData: Uint8Array, params: Record<string, unknown> = {}): Promise<AiProcessResult> {
  return safeInvoke("ai_auto_color", {
    request: {
      feature: "auto_color",
      imageData: Array.from(imageData),
      params,
    },
  });
}

export async function aiInbetween(prevFrame: Uint8Array, nextFrame: Uint8Array): Promise<AiProcessResult> {
  return safeInvoke("ai_inbetween", {
    prevFrame: Array.from(prevFrame),
    nextFrame: Array.from(nextFrame),
  });
}

export async function aiStyleTransfer(imageData: Uint8Array, styleData: Uint8Array): Promise<AiProcessResult> {
  return safeInvoke("ai_style_transfer", {
    request: {
      feature: "style_transfer",
      imageData: Array.from(imageData),
      params: { style: Array.from(styleData) },
    },
  });
}

export async function aiQueueStatus(): Promise<AiQueueStatus> {
  try {
    return await safeInvoke<AiQueueStatus>("ai_queue_status");
  } catch {
    return { pending: 0, maxSize: 100, isFull: false };
  }
}
