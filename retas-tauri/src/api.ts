import { invoke } from "@tauri-apps/api/core";
import {
  mockDrawStroke,
  mockGetLayers,
  mockAddLayer,
  mockDeleteLayer,
  mockToggleLayerVisibility,
  mockToggleLayerLock,
  mockSelectLayer,
  mockGetFrameInfo,
  mockSetCurrentFrame,
  mockAddFrame,
  mockDeleteFrame,
  mockGetLayerPixels,
  mockCompositeLayers,
  mockUndo,
  mockRedo,
  mockCanUndo,
  mockCanRedo,
  mockOpenDocument,
  mockSaveDocument,
} from "./api.mock";

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

export interface XSheetCell {
  layerId: string;
  frame: number;
  hasKeyframe: boolean;
  isEmpty: boolean;
}

export async function drawStroke(command: DrawCommand): Promise<string> {
  if (!isTauri) return mockDrawStroke(command);
  return safeInvoke("draw_stroke", { command });
}

export async function getLayers(): Promise<LayerInfo[]> {
  if (!isTauri) return mockGetLayers();
  return safeInvoke<LayerInfo[]>("get_layers");
}

export async function addLayer(name: string): Promise<LayerInfo> {
  if (!isTauri) return mockAddLayer(name);
  return safeInvoke<LayerInfo>("add_layer", { name });
}

export async function deleteLayer(id: string): Promise<void> {
  if (!isTauri) return mockDeleteLayer(id);
  return safeInvoke("delete_layer", { id });
}

export async function toggleLayerVisibility(id: string): Promise<boolean> {
  if (!isTauri) return mockToggleLayerVisibility(id);
  return safeInvoke<boolean>("toggle_layer_visibility", { id });
}

export async function toggleLayerLock(id: string): Promise<boolean> {
  if (!isTauri) return mockToggleLayerLock(id);
  return safeInvoke<boolean>("toggle_layer_lock", { id });
}

export async function selectLayer(id: string): Promise<void> {
  if (!isTauri) return mockSelectLayer(id);
  return safeInvoke("select_layer", { id });
}

export async function getFrameInfo(): Promise<FrameInfo> {
  if (!isTauri) return mockGetFrameInfo();
  const info = await safeInvoke<FrameInfo>("get_frame_info");
  return { ...info, current: info.current + 1 };
}

export async function setCurrentFrame(frame: number): Promise<void> {
  if (!isTauri) {
    return mockSetCurrentFrame(frame);
  }
  return safeInvoke("set_current_frame", { frame: frame - 1 });
}

export async function addFrame(): Promise<void> {
  if (!isTauri) return mockAddFrame();
  return safeInvoke("add_frame");
}

export async function deleteFrame(): Promise<void> {
  if (!isTauri) return mockDeleteFrame();
  return safeInvoke("delete_frame");
}

export async function getLayerPixels(layerId: string): Promise<Uint8Array> {
  if (!isTauri) return mockGetLayerPixels(layerId);
  return safeInvoke("get_layer_pixels", { layerId });
}

export async function compositeLayers(): Promise<Uint8Array> {
  if (!isTauri) return mockCompositeLayers();
  return safeInvoke("composite_layers");
}

export async function undo(): Promise<boolean> {
  if (!isTauri) return mockUndo();
  return safeInvoke<boolean>("undo");
}

export async function redo(): Promise<boolean> {
  if (!isTauri) return mockRedo();
  return safeInvoke<boolean>("redo");
}

export async function canUndo(): Promise<boolean> {
  if (!isTauri) return mockCanUndo();
  return safeInvoke<boolean>("can_undo");
}

export async function canRedo(): Promise<boolean> {
  if (!isTauri) return mockCanRedo();
  return safeInvoke<boolean>("can_redo");
}

export async function openDocument(path: string): Promise<DocumentInfo> {
  if (!isTauri) return mockOpenDocument(path);
  return safeInvoke<DocumentInfo>("open_document", { path });
}

export async function saveDocument(path: string): Promise<void> {
  if (!isTauri) return mockSaveDocument(path);
  return safeInvoke("save_document", { path });
}

export async function getXSheetData(): Promise<XSheetCell[]> {
  if (!isTauri) {
    return Promise.resolve([]);
  }
  return safeInvoke<XSheetCell[]>("get_xsheet_data");
}

export async function toggleKeyframe(layerId: string, frame: number): Promise<void> {
  if (!isTauri) {
    return Promise.resolve();
  }
  return safeInvoke("toggle_keyframe", { layerId, frame: frame - 1 });
}

export async function insertFrames(atFrame: number, count: number): Promise<void> {
  if (!isTauri) {
    return Promise.resolve();
  }
  return safeInvoke("insert_frames", { atFrame: atFrame - 1, count });
}

export async function deleteFrames(atFrame: number, count: number): Promise<void> {
  if (!isTauri) {
    return Promise.resolve();
  }
  return safeInvoke("delete_frames", { atFrame: atFrame - 1, count });
}

export async function copyFrame(layerId: string, fromFrame: number, toFrame: number): Promise<void> {
  if (!isTauri) {
    return Promise.resolve();
  }
  return safeInvoke("copy_frame", { layerId, fromFrame: fromFrame - 1, toFrame: toFrame - 1 });
}

// Selection API types
export type SelectionToolType = "rect" | "ellipse" | "lasso" | "magicWand";
export type SelectionMode = "replace" | "add" | "subtract" | "intersect";

export interface SelectionData {
  type: SelectionToolType;
  mode: SelectionMode;
  points: { x: number; y: number }[];
  rect?: { x: number; y: number; width: number; height: number };
  feather: number;
  tolerance: number;
  contiguous: boolean;
}

export interface SelectionBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

export async function createSelection(selection: SelectionData): Promise<void> {
  if (!isTauri) {
    return Promise.resolve();
  }
  return safeInvoke("create_selection", { selection });
}

export async function updateSelection(selection: SelectionData): Promise<void> {
  if (!isTauri) {
    return Promise.resolve();
  }
  return safeInvoke("update_selection", { selection });
}

export async function clearSelection(): Promise<void> {
  if (!isTauri) {
    return Promise.resolve();
  }
  return safeInvoke("clear_selection");
}

export async function getSelection(): Promise<SelectionData | null> {
  if (!isTauri) {
    return Promise.resolve(null);
  }
  return safeInvoke<SelectionData | null>("get_selection");
}

export async function invertSelection(): Promise<void> {
  if (!isTauri) {
    return Promise.resolve();
  }
  return safeInvoke("invert_selection");
}

export async function getSelectionBounds(): Promise<SelectionBounds | null> {
  if (!isTauri) {
    return Promise.resolve(null);
  }
  return safeInvoke<SelectionBounds | null>("get_selection_bounds");
}

export async function applySelectionToLayer(layerId: string): Promise<void> {
  if (!isTauri) {
    return Promise.resolve();
  }
  return safeInvoke("apply_selection_to_layer", { layerId });
}


