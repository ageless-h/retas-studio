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
  blendMode?: string;
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

export interface SparsePixel {
  x: number;
  y: number;
  r: number;
  g: number;
  b: number;
  a: number;
}

export async function applyStrokePixels(strokePixels: SparsePixel[]): Promise<string> {
  if (!isTauri) {
    return Promise.resolve("Mock: stroke pixels applied");
  }
  return safeInvoke("apply_stroke_pixels_sparse", { strokePixels });
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

export async function renameLayer(id: string, name: string): Promise<void> {
  if (!isTauri) return Promise.resolve();
  return safeInvoke("rename_layer", { id, name });
}

export async function setLayerOpacity(id: string, opacity: number): Promise<void> {
  if (!isTauri) return Promise.resolve();
  return safeInvoke("set_layer_opacity", { id, opacity });
}

export async function moveLayer(id: string, newIndex: number): Promise<void> {
  if (!isTauri) return Promise.resolve();
  return safeInvoke("move_layer", { id, newIndex });
}

export async function setLayerBlendMode(id: string, blendMode: string): Promise<void> {
  if (!isTauri) return Promise.resolve();
  return safeInvoke("set_layer_blend_mode", { id, blendMode });
}

export async function newDocument(name: string, width: number, height: number, fps: number, totalFrames: number): Promise<DocumentInfo> {
  if (!isTauri) return { name, width, height, frame_rate: fps, total_frames: totalFrames };
  return safeInvoke<DocumentInfo>("new_document", { name, width, height, fps, totalFrames });
}

export async function duplicateLayer(id: string): Promise<LayerInfo> {
  if (!isTauri) return { id: "mock", name: "Copy", visible: true, locked: false, opacity: 1.0 };
  return safeInvoke<LayerInfo>("duplicate_layer", { id });
}

export interface EffectParams {
  effectType: string;
  radius?: number;
  brightness?: number;
  contrast?: number;
  hue?: number;
  saturation?: number;
  lightness?: number;
  opacity?: number;
}

export async function applyEffect(effect: EffectParams): Promise<void> {
  if (!isTauri) return Promise.resolve();
  return safeInvoke("apply_effect", { effect });
}

export async function copySelectionPixels(): Promise<number[]> {
  if (!isTauri) return [];
  return safeInvoke<number[]>("copy_selection_pixels");
}

export async function pastePixels(data: number[], x: number, y: number): Promise<void> {
  if (!isTauri) return Promise.resolve();
  return safeInvoke("paste_pixels", { data, x: Math.floor(x), y: Math.floor(y) });
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
  // Backend uses update_selection for both create and modify
  return safeInvoke("update_selection", { selection });
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

export async function applySelectionToLayer(layerId: string, operation: string): Promise<void> {
  if (!isTauri) {
    return Promise.resolve();
  }
  return safeInvoke("apply_selection_to_layer", { layerId, operation });
}

export async function floodFillLayer(
  x: number,
  y: number,
  color: [number, number, number, number],
  tolerance: number
): Promise<void> {
  if (!isTauri) {
    return Promise.resolve();
  }
  return safeInvoke("flood_fill_layer", { x, y, color, tolerance });
}

export async function getDocumentInfo(): Promise<DocumentInfo> {
  if (!isTauri) {
    return { name: "Untitled", width: 1920, height: 1080, frame_rate: 24, total_frames: 100 };
  }
  return safeInvoke<DocumentInfo>("get_document_info");
}

export async function exportImage(
  outputPath: string,
  format: string,
  frame?: number
): Promise<void> {
  if (!isTauri) {
    return Promise.resolve();
  }
  return safeInvoke("export_image", { outputPath, format, frame });
}

export async function exportFrameSequence(
  outputDir: string,
  format: string,
  startFrame: number,
  endFrame: number
): Promise<void> {
  if (!isTauri) {
    return Promise.resolve();
  }
  return safeInvoke("export_frame_sequence", {
    outputDir,
    format,
    startFrame: startFrame - 1,
    endFrame: endFrame - 1,
  });
}

export async function pickColor(
  x: number,
  y: number,
  layerId: string,
  frame: number
): Promise<[number, number, number, number]> {
  if (!isTauri) {
    return [0, 0, 0, 255];
  }
  return safeInvoke("pick_color", { x, y, layerId, frame });
}

export async function createLayerGroup(name: string): Promise<LayerInfo> {
  if (!isTauri) {
    return { id: crypto.randomUUID(), name, visible: true, locked: false, opacity: 1.0, layer_type: "Group", blend_mode: "normal" };
  }
  return safeInvoke("create_layer_group", { name });
}

export async function setLayerParent(layerId: string, parentId: string | null): Promise<void> {
  if (!isTauri) return;
  return safeInvoke("set_layer_parent", { layerId, parentId });
}

export async function getCompositedFrame(frame: number): Promise<number[]> {
  if (!isTauri) {
    return Array(1920 * 1080 * 4).fill(255);
  }
  return safeInvoke("get_composited_frame", { frame });
}

export async function resizeDocument(width: number, height: number): Promise<void> {
  if (!isTauri) return;
  return safeInvoke("resize_document", { width, height });
}

export async function moveLayerPixels(layerId: string, dx: number, dy: number): Promise<void> {
  if (!isTauri) return;
  return safeInvoke("move_layer_pixels", { layerId, dx, dy });
}

export async function flipLayer(layerId: string, horizontal: boolean): Promise<void> {
  if (!isTauri) return;
  return safeInvoke("flip_layer", { layerId, horizontal });
}

export async function rotateLayer90(layerId: string, clockwise: boolean): Promise<void> {
  if (!isTauri) return;
  return safeInvoke("rotate_layer_90", { layerId, clockwise });
}

export interface RenderJobInfo {
  id: number;
  name: string;
  frame_range: [number, number];
  format: string;
  quality: string;
  status: string;
  progress: number;
}

export async function addRenderJob(
  name: string, startFrame: number, endFrame: number,
  outputDir: string, format: string, quality: string
): Promise<RenderJobInfo> {
  if (!isTauri) {
    return { id: 1, name, frame_range: [startFrame, endFrame], format, quality, status: "queued", progress: 0 };
  }
  return safeInvoke("add_render_job", { name, startFrame, endFrame, outputDir, format, quality });
}

export async function getRenderJobs(): Promise<RenderJobInfo[]> {
  if (!isTauri) return [];
  return safeInvoke("get_render_jobs", {});
}

export async function cancelRenderJob(jobId: number): Promise<boolean> {
  if (!isTauri) return false;
  return safeInvoke("cancel_render_job", { jobId });
}

export async function clearCompletedJobs(): Promise<void> {
  if (!isTauri) return;
  return safeInvoke("clear_completed_jobs", {});
}

export async function clearFrame(layerId: string, frame: number): Promise<void> {
  if (!isTauri) return;
  return safeInvoke("clear_frame", { layerId, frame });
}

export async function fillFrame(layerId: string, frame: number, color: [number, number, number, number]): Promise<void> {
  if (!isTauri) return;
  return safeInvoke("fill_frame", { layerId, frame, color });
}

export async function colorReplace(
  layerId: string, frame: number,
  sourceColor: [number, number, number], targetColor: [number, number, number],
  tolerance: number
): Promise<number> {
  if (!isTauri) return 0;
  return safeInvoke("color_replace", { layerId, frame, sourceColor, targetColor, tolerance });
}

// ─── Light Table ─────────────────────────────────────────────────

export interface OnionSkinInfo {
  enabled: boolean;
  frames_before: number;
  frames_after: number;
  opacity_before: number;
  opacity_after: number;
  color_before: [number, number, number, number];
  color_after: [number, number, number, number];
  blend_mode: string;
}

export interface ReferenceLayerInfo {
  id: string;
  name: string;
  image_path: string | null;
  opacity: number;
  visible: boolean;
  locked: boolean;
}

export interface LightTableInfo {
  enabled: boolean;
  opacity: number;
  onion_skin: OnionSkinInfo;
  references: ReferenceLayerInfo[];
}

export async function getLightTable(frame: number): Promise<LightTableInfo | null> {
  if (!isTauri) return null;
  return safeInvoke("get_light_table", { frame });
}

export async function setOnionSkin(
  frame: number, enabled: boolean,
  framesBefore: number, framesAfter: number,
  opacityBefore: number, opacityAfter: number
): Promise<void> {
  if (!isTauri) return;
  return safeInvoke("set_onion_skin", { frame, enabled, framesBefore, framesAfter, opacityBefore, opacityAfter });
}

export async function setOnionSkinColors(
  frame: number,
  colorBefore: [number, number, number, number],
  colorAfter: [number, number, number, number],
  blendMode: string
): Promise<void> {
  if (!isTauri) return;
  return safeInvoke("set_onion_skin_colors", { frame, colorBefore, colorAfter, blendMode });
}

export async function addReferenceLayer(
  frame: number, name: string, imagePath?: string
): Promise<string> {
  if (!isTauri) return "";
  return safeInvoke("add_reference_layer", { frame, name, imagePath: imagePath ?? null });
}

export async function removeReferenceLayer(frame: number, referenceId: string): Promise<boolean> {
  if (!isTauri) return false;
  return safeInvoke("remove_reference_layer", { frame, referenceId });
}

export async function setReferenceOpacity(
  frame: number, referenceId: string, opacity: number
): Promise<void> {
  if (!isTauri) return;
  return safeInvoke("set_reference_opacity", { frame, referenceId, opacity });
}

export async function toggleReferenceVisibility(frame: number, referenceId: string): Promise<void> {
  if (!isTauri) return;
  return safeInvoke("toggle_reference_visibility", { frame, referenceId });
}

export async function getOnionSkinFrames(frame: number): Promise<[number, number, [number, number, number, number]][]> {
  if (!isTauri) return [];
  return safeInvoke("get_onion_skin_frames", { frame });
}

// ─── Motion Check ────────────────────────────────────────────────

export interface MotionCheckInfo {
  enabled: boolean;
  mode: string;
  comparison_frames: number[];
  overlay_opacity: number;
  show_trails: boolean;
  trail_length: number;
}

export interface TrailPointInfo {
  x: number;
  y: number;
  frame: number;
  size: number;
}

export interface TrailInfo {
  layer_id: string;
  points: TrailPointInfo[];
  visible: boolean;
}

export async function getMotionCheck(frame: number): Promise<MotionCheckInfo | null> {
  if (!isTauri) return null;
  return safeInvoke("get_motion_check", { frame });
}

export async function setMotionCheck(
  frame: number, enabled: boolean, mode: string,
  comparisonFrames: number[], overlayOpacity: number
): Promise<void> {
  if (!isTauri) return;
  return safeInvoke("set_motion_check", { frame, enabled, mode, comparisonFrames, overlayOpacity });
}

export async function toggleMotionTrails(frame: number, show: boolean, trailLength: number): Promise<void> {
  if (!isTauri) return;
  return safeInvoke("toggle_motion_trails", { frame, show, trailLength });
}

export async function addTrailPoint(
  layerId: string, x: number, y: number, frame: number, size: number
): Promise<void> {
  if (!isTauri) return;
  return safeInvoke("add_trail_point", { layerId, x, y, frame, size });
}

export async function getTrail(layerId: string): Promise<TrailInfo | null> {
  if (!isTauri) return null;
  return safeInvoke("get_trail", { layerId });
}

export async function clearTrails(): Promise<void> {
  if (!isTauri) return;
  return safeInvoke("clear_trails", {});
}

// ─── Selection Transform ─────────────────────────────────────────

export async function offsetSelectionPixels(
  layerId: string, frame: number, dx: number, dy: number
): Promise<void> {
  if (!isTauri) return;
  return safeInvoke("offset_selection_pixels", { layerId, frame, dx, dy });
}

export async function scaleSelectionPixels(
  layerId: string, frame: number, scaleX: number, scaleY: number
): Promise<void> {
  if (!isTauri) return;
  return safeInvoke("scale_selection_pixels", { layerId, frame, scaleX, scaleY });
}

// ─── Batch Queue ─────────────────────────────────────────────────

export interface BatchItemInfo {
  id: number;
  operation: string;
  status: string;
  priority: string;
}

export async function addBatchExport(
  outputDir: string, format: string, startFrame: number, endFrame: number, priority: string
): Promise<number> {
  if (!isTauri) return 0;
  return safeInvoke("add_batch_export", { outputDir, format, startFrame, endFrame, priority });
}

export async function addBatchColorReplace(
  sourceColor: [number, number, number], targetColor: [number, number, number],
  tolerance: number, priority: string
): Promise<number> {
  if (!isTauri) return 0;
  return safeInvoke("add_batch_color_replace", { sourceColor, targetColor, tolerance, priority });
}

export async function getBatchQueue(): Promise<BatchItemInfo[]> {
  if (!isTauri) return [];
  return safeInvoke("get_batch_queue", {});
}

export async function cancelBatchItem(id: number): Promise<boolean> {
  if (!isTauri) return false;
  return safeInvoke("cancel_batch_item", { id });
}

export async function clearBatchCompleted(): Promise<void> {
  if (!isTauri) return;
  return safeInvoke("clear_batch_completed", {});
}

export async function getBatchStats(): Promise<[number, number, number, number]> {
  if (!isTauri) return [0, 0, 0, 0];
  return safeInvoke("get_batch_stats", {});
}

// ─── Vectorize ───────────────────────────────────────────────────

export interface VectorPointInfo {
  x: number;
  y: number;
  control_in: [number, number] | null;
  control_out: [number, number] | null;
  is_corner: boolean;
}

export interface VectorPathInfo {
  points: VectorPointInfo[];
  is_closed: boolean;
  color: [number, number, number, number];
  stroke_width: number;
}

export interface VectorizeResultInfo {
  paths: VectorPathInfo[];
  width: number;
  height: number;
  processing_time_ms: number;
}

export async function vectorizeLayer(
  layerId: string, frame: number,
  threshold: number, smoothing: number, minPathLength: number
): Promise<VectorizeResultInfo | null> {
  if (!isTauri) return null;
  return safeInvoke("vectorize_layer", { layerId, frame, threshold, smoothing, minPathLength });
}

// ─── Cut System ──────────────────────────────────────────────────

export interface CutInfo {
  id: number;
  name: string;
  start_frame: number;
  end_frame: number;
  duration_frames: number;
  layers: string[];
  notes: string;
  color: [number, number, number, number];
}

export interface CutFolderInfo {
  id: number;
  name: string;
  cuts: CutInfo[];
  notes: string;
}

export async function createCutFolder(name: string): Promise<number> {
  if (!isTauri) return 0;
  return safeInvoke("create_cut_folder", { name });
}

export async function deleteCutFolder(folderId: number): Promise<boolean> {
  if (!isTauri) return false;
  return safeInvoke("delete_cut_folder", { folderId });
}

export async function getCutFolders(): Promise<CutFolderInfo[]> {
  if (!isTauri) return [];
  return safeInvoke("get_cut_folders", {});
}

export async function addCut(
  folderId: number, name: string, startFrame: number, endFrame: number
): Promise<number> {
  if (!isTauri) return 0;
  return safeInvoke("add_cut", { folderId, name, startFrame, endFrame });
}

export async function removeCut(folderId: number, cutId: number): Promise<boolean> {
  if (!isTauri) return false;
  return safeInvoke("remove_cut", { folderId, cutId });
}

export async function setCurrentCutFolder(folderId: number): Promise<boolean> {
  if (!isTauri) return false;
  return safeInvoke("set_current_cut_folder", { folderId });
}

export async function findCutAtFrame(frame: number): Promise<[CutFolderInfo, CutInfo] | null> {
  if (!isTauri) return null;
  return safeInvoke("find_cut_at_frame", { frame });
}

// ─── Coloring Engine ─────────────────────────────────────────────

export async function smartFill(
  layerId: string, frame: number,
  startX: number, startY: number,
  fillColor: [number, number, number, number],
  mode: string, tolerance: number, gapRadius: number
): Promise<void> {
  if (!isTauri) return;
  return safeInvoke("smart_fill", { layerId, frame, startX, startY, fillColor, mode, tolerance, gapRadius });
}

// ─── Keyframe Animation ─────────────────────────────────────────

export async function addTransformKeyframe(
  layerId: string, frame: number,
  translateX: number, translateY: number,
  rotation: number, scaleX: number, scaleY: number,
  interpolation: string
): Promise<void> {
  if (!isTauri) return;
  return safeInvoke("add_transform_keyframe", {
    layerId, frame, translateX, translateY, rotation, scaleX, scaleY, interpolation,
  });
}

export async function getInterpolationTypes(): Promise<string[]> {
  if (!isTauri) return [];
  return safeInvoke("get_interpolation_types", {});
}

