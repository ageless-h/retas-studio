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

export async function drawStroke(command: DrawCommand): Promise<string> {
  return safeInvoke("draw_stroke", { command });
}

export async function getLayers(): Promise<LayerInfo[]> {
  return safeInvoke<LayerInfo[]>("get_layers");
}

export async function addLayer(name: string): Promise<LayerInfo> {
  return safeInvoke<LayerInfo>("add_layer", { name });
}

export async function deleteLayer(id: string): Promise<void> {
  return safeInvoke("delete_layer", { id });
}

export async function toggleLayerVisibility(id: string): Promise<boolean> {
  return safeInvoke<boolean>("toggle_layer_visibility", { id });
}

export async function toggleLayerLock(id: string): Promise<boolean> {
  return safeInvoke<boolean>("toggle_layer_lock", { id });
}

export async function getFrameInfo(): Promise<FrameInfo> {
  return safeInvoke<FrameInfo>("get_frame_info");
}

export async function setCurrentFrame(frame: number): Promise<void> {
  return safeInvoke("set_current_frame", { frame });
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
  return safeInvoke<boolean>("undo");
}

export async function redo(): Promise<boolean> {
  return safeInvoke<boolean>("redo");
}

export async function canUndo(): Promise<boolean> {
  return safeInvoke<boolean>("can_undo");
}

export async function canRedo(): Promise<boolean> {
  return safeInvoke<boolean>("can_redo");
}

export async function openDocument(path: string): Promise<DocumentInfo> {
  return safeInvoke<DocumentInfo>("open_document", { path });
}

export async function saveDocument(path: string): Promise<void> {
  return safeInvoke("save_document", { path });
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
  return safeInvoke<AiQueueStatus>("ai_queue_status");
}
