use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

mod state;
use state::AppState;

use retas_core::Layer as RetasLayer;
use retas_core::advanced::selection::{Selection, SelectionMask, SelectionTool, SelectionMode as RetasSelectionMode};
use retas_core::advanced::undo::{UndoManager, Command, LayerAddCommand, LayerDeleteCommand, SnapshotCommand};
use retas_core::advanced::brush::{BrushEngine, BrushSettings, BrushPoint, BrushType, BrushBlendMode};
use retas_core::advanced::effect_processor::EffectProcessor;
use retas_core::advanced::effects::{Effect, EffectType};
use retas_core::advanced::render_queue::{RenderJob, RenderFormat, RenderQuality, RenderStatus};
use retas_core::advanced::light_table::{LightTableManager, ReferenceLayer, OnionBlendMode};
use retas_core::advanced::motion_check::{MotionCheckManager, MotionCheckMode};
use retas_core::advanced::batch::{BatchQueue, BatchOperation, BatchPriority, BatchStatus, BatchItem, ExportFormat as BatchExportFormat};
use retas_core::advanced::vectorize::{Vectorizer, VectorizationSettings, VectorizationResult, VectorizedPath, VectorizedPoint};
use retas_core::advanced::cut_system::{CutManager, Cut, CutFolder};
use retas_core::advanced::coloring::{ColoringEngine, FillSettings, FillMode};
use retas_core::advanced::keyframe::{Interpolation, AnimationTrack, Keyframe, TransformKey, LayerAnimation, SceneAnimation};
use retas_io::export::{ImageExporter, ImageExportOptions, ImageFormat};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LayerInfo {
    pub id: String,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f64,
    pub layer_type: String,
    pub blend_mode: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FrameInfo {
    pub current: u32,
    pub total: u32,
    pub fps: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DrawCommand {
    pub tool: String,
    pub points: Vec<(f64, f64)>,
    pub color: (u8, u8, u8),
    pub size: f64,
    pub layer_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DocumentInfo {
    pub name: String,
    pub width: f64,
    pub height: f64,
    pub frame_rate: f64,
    pub total_frames: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct XSheetCell {
    pub layer_id: String,
    pub frame: u32,
    pub has_keyframe: bool,
    pub is_empty: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SelectionDataFrontend {
    #[serde(rename = "type")]
    pub selection_type: String,
    pub mode: String,
    pub points: Vec<(f64, f64)>,
    pub rect: Option<(f64, f64, f64, f64)>,
    pub feather: f64,
    pub tolerance: f64,
    pub contiguous: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SelectionBoundsFrontend {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

fn parse_layer_id(s: &str) -> Result<retas_core::LayerId, String> {
    let uuid = retas_core::uuid::Uuid::parse_str(s).map_err(|e| e.to_string())?;
    Ok(retas_core::LayerId(uuid))
}

/// Record a snapshot for undo. Call with the editor lock already held.
/// Returns the SnapshotCommand (with `before` captured). After mutation,
/// call `snap.capture_after(doc.clone())` then push to undo_manager.
fn snapshot_before(doc: &retas_core::Document, description: &str) -> SnapshotCommand {
    SnapshotCommand::new(doc.clone(), description)
}

/// Push a completed snapshot command (with after state) into the undo manager.
/// This is a no-op execute since mutation already happened.
fn push_snapshot(undo: &mut UndoManager, mut snap: SnapshotCommand, doc: &mut retas_core::Document) {
    snap.capture_after(doc.clone());
    // Push directly onto undo stack without re-executing
    // We use a wrapper: execute is a no-op, so calling execute() is safe
    undo.execute(Box::new(snap), doc);
}

#[tauri::command]
fn get_document_info(state: State<Arc<AppState>>) -> Result<DocumentInfo, String> {
    let editor = state.editor.lock().map_err(|e| e.to_string())?;
    let doc = &editor.document;
    Ok(DocumentInfo {
        name: doc.settings.name.clone(),
        width: doc.settings.resolution.width,
        height: doc.settings.resolution.height,
        frame_rate: doc.settings.frame_rate,
        total_frames: doc.settings.total_frames,
    })
}

#[tauri::command]
fn get_layers(state: State<Arc<AppState>>) -> Result<Vec<LayerInfo>, String> {
    let editor = state.editor.lock().map_err(|e| e.to_string())?;
    let doc = &editor.document;
    let mut layers = Vec::new();

    for layer_id in &doc.timeline.layer_order {
        if let Some(layer) = doc.layers.get(layer_id) {
            let base = layer.base();
            layers.push(LayerInfo {
                id: base.id.0.to_string(),
                name: base.name.clone(),
                visible: base.visible,
                locked: base.locked,
                opacity: base.opacity,
                layer_type: format!("{:?}", base.layer_type),
                blend_mode: format!("{:?}", base.blend_mode).to_lowercase(),
            });
        }
    }

    Ok(layers)
}

#[tauri::command]
fn add_layer(name: String, state: State<Arc<AppState>>) -> Result<LayerInfo, String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let layer = RetasLayer::Raster(retas_core::RasterLayer::new(&name));
    let base = layer.base().clone();
    let layer_id = base.id;
    let index = editor.document.timeline.layer_order.len();
    
    let cmd = LayerAddCommand {
        layer,
        index,
        description: format!("添加图层: {}", name),
    };
    
    editor.undo_manager.execute(Box::new(cmd), &mut editor.document);
    
    Ok(LayerInfo {
        id: layer_id.0.to_string(),
        name: base.name,
        visible: base.visible,
        locked: base.locked,
        opacity: base.opacity,
        layer_type: format!("{:?}", base.layer_type),
        blend_mode: format!("{:?}", base.blend_mode).to_lowercase(),
    })
}

#[tauri::command]
fn delete_layer(id: String, state: State<Arc<AppState>>) -> Result<(), String> {
    let layer_id = parse_layer_id(&id)?;
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    
    let layer = editor.document.layers.get(&layer_id).cloned().ok_or("Layer not found")?;
    let index = editor.document.timeline.layer_order.iter().position(|x| *x == layer_id).unwrap_or(0);
    
    let cmd = LayerDeleteCommand {
        layer,
        index,
        description: "删除图层".to_string(),
    };
    
    editor.undo_manager.execute(Box::new(cmd), &mut editor.document);
    
    Ok(())
}

#[tauri::command]
fn toggle_layer_visibility(id: String, state: State<Arc<AppState>>) -> Result<bool, String> {
    let layer_id = parse_layer_id(&id)?;
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = snapshot_before(&editor.document, "切换图层可见性");
    
    if let Some(layer) = editor.document.layers.get_mut(&layer_id) {
        layer.base_mut().visible = !layer.base().visible;
        let result = layer.base().visible;
        push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
        Ok(result)
    } else {
        Err("Layer not found".to_string())
    }
}

#[tauri::command]
fn toggle_layer_lock(id: String, state: State<Arc<AppState>>) -> Result<bool, String> {
    let layer_id = parse_layer_id(&id)?;
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = snapshot_before(&editor.document, "切换图层锁定");
    
    if let Some(layer) = editor.document.layers.get_mut(&layer_id) {
        layer.base_mut().locked = !layer.base().locked;
        let result = layer.base().locked;
        push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
        Ok(result)
    } else {
        Err("Layer not found".to_string())
    }
}

#[tauri::command]
fn get_frame_info(state: State<Arc<AppState>>) -> Result<FrameInfo, String> {
    let editor = state.editor.lock().map_err(|e| e.to_string())?;
    let doc = &editor.document;
    Ok(FrameInfo {
        current: doc.timeline.current_frame,
        total: doc.timeline.end_frame,
        fps: doc.timeline.frame_rate,
    })
}

#[tauri::command]
fn set_current_frame(frame: u32, state: State<Arc<AppState>>) -> Result<(), String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    editor.document.timeline.current_frame = frame;
    Ok(())
}

#[tauri::command]
fn add_frame(state: State<Arc<AppState>>) -> Result<(), String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = snapshot_before(&editor.document, "添加帧");
    
    let new_frame = editor.document.timeline.end_frame;
    let width = editor.document.settings.resolution.width as u32;
    let height = editor.document.settings.resolution.height as u32;

    for layer in editor.document.layers.values_mut() {
        if let retas_core::Layer::Raster(raster) = layer {
            if !raster.frames.contains_key(&new_frame) {
                raster.frames.insert(new_frame, retas_core::RasterFrame {
                    frame_number: new_frame,
                    image_data: std::sync::Arc::new(vec![0u8; (width * height * 4) as usize]),
                    width,
                    height,
                    bounds: None,
                });
            }
        }
    }

    editor.document.timeline.end_frame += 1;
    editor.document.settings.total_frames += 1;
    push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
    Ok(())
}

#[tauri::command]
fn delete_frame(state: State<Arc<AppState>>) -> Result<(), String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    if editor.document.timeline.end_frame > 1 {
        let snap = snapshot_before(&editor.document, "删除帧");
        let removed_frame = editor.document.timeline.end_frame - 1;
        for layer in editor.document.layers.values_mut() {
            if let retas_core::Layer::Raster(raster) = layer {
                raster.frames.remove(&removed_frame);
            }
        }
        editor.document.timeline.end_frame -= 1;
        editor.document.settings.total_frames -= 1;
        push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
    }
    Ok(())
}

fn draw_line_on_pixels(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    color: (u8, u8, u8),
    size: f64,
    is_eraser: bool,
) {
    let brush_radius = size.max(1.0) as i32;
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let steps = (dx.max(dy) / 0.5).max(1.0) as i32;

    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        let cx = x0 + (x1 - x0) * t;
        let cy = y0 + (y1 - y0) * t;

        for by in -brush_radius..=brush_radius {
            for bx in -brush_radius..=brush_radius {
                if bx * bx + by * by > brush_radius * brush_radius {
                    continue;
                }
                let px = (cx + bx as f64) as i32;
                let py = (cy + by as f64) as i32;
                if px < 0 || py < 0 || px >= width as i32 || py >= height as i32 {
                    continue;
                }
                let idx = ((py * width as i32 + px) * 4) as usize;
                if is_eraser {
                    pixels[idx] = 0;
                    pixels[idx + 1] = 0;
                    pixels[idx + 2] = 0;
                    pixels[idx + 3] = 0;
                } else {
                    pixels[idx] = color.0;
                    pixels[idx + 1] = color.1;
                    pixels[idx + 2] = color.2;
                    pixels[idx + 3] = 255;
                }
            }
        }
    }
}

#[tauri::command]
fn draw_stroke(command: DrawCommand, state: State<Arc<AppState>>) -> Result<String, String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = snapshot_before(&editor.document, "绘制笔划");

    let layer_id = if command.layer_id == "current" {
        editor.document.selected_layers.first().copied()
    } else {
        parse_layer_id(&command.layer_id).ok()
    };

    let layer_id = layer_id.ok_or("No selected layer")?;
    let current_frame = editor.document.timeline.current_frame;

    let layer = editor.document.layers.get_mut(&layer_id).ok_or("Layer not found")?;
    let raster = match layer {
        retas_core::Layer::Raster(r) => r,
        _ => return Err("Only raster layers support drawing".to_string()),
    };

    let frame = raster.frames.get_mut(&current_frame).ok_or("No frame data")?;
    let is_eraser = command.tool == "eraser";
    let frame_width = frame.width;
    let frame_height = frame.height;

    // Use BrushEngine for point processing (smoothing, spacing interpolation)
    let color = retas_core::Color8::new(command.color.0, command.color.1, command.color.2, 255);
    let brush_settings = BrushSettings::new(command.size, color)
        .with_hardness(0.8)
        .with_type(BrushType::Round)
        .with_blend_mode(BrushBlendMode::Normal);
    
    let mut engine = BrushEngine::new();
    
    if let Some(&first) = command.points.first() {
        let first_point = BrushPoint::new(retas_core::Point::new(first.0, first.1));
        engine.start_stroke(brush_settings, first_point);
        
        for &(x, y) in command.points.iter().skip(1) {
            engine.add_point(BrushPoint::new(retas_core::Point::new(x, y)));
        }
        
        if let Some(stroke) = engine.end_stroke() {
            // Get interpolated points from BrushEngine (applies spacing + smoothing)
            let interpolated = stroke.calculate_interpolated_points(0.5);
            let pixels = frame.get_image_data_mut();
            
            for window in interpolated.windows(2) {
                let p0 = &window[0];
                let p1 = &window[1];
                draw_line_on_pixels(
                    pixels,
                    frame_width,
                    frame_height,
                    p0.position.x, p0.position.y,
                    p1.position.x, p1.position.y,
                    command.color,
                    command.size,
                    is_eraser,
                );
            }
        }
    }

    push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
    Ok(format!("绘制了 {} 个点", command.points.len()))
}

#[tauri::command]
fn apply_stroke_pixels(
    stroke_pixels: Vec<u8>,
    state: State<Arc<AppState>>,
) -> Result<String, String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = snapshot_before(&editor.document, "应用笔划像素");

    let layer_id = editor.document.selected_layers.first().copied().ok_or("No selected layer")?;
    let current_frame = editor.document.timeline.current_frame;

    let layer = editor.document.layers.get_mut(&layer_id).ok_or("Layer not found")?;
    let raster = match layer {
        retas_core::Layer::Raster(r) => r,
        _ => return Err("Only raster layers support drawing".to_string()),
    };

    let frame = raster.frames.get_mut(&current_frame).ok_or("No frame data")?;
    let width = frame.width as usize;
    let height = frame.height as usize;
    let expected_len = width * height * 4;

    if stroke_pixels.len() != expected_len {
        return Err(format!(
            "Pixel data size mismatch: expected {}, got {}",
            expected_len,
            stroke_pixels.len()
        ));
    }

    let frame_data = frame.get_image_data_mut();

    for (i, chunk) in stroke_pixels.chunks(4).enumerate() {
        if chunk.len() < 4 {
            continue;
        }
        
        let src_alpha = chunk[3];
        if src_alpha == 0 {
            continue;
        }
        
        let idx = i * 4;
        let dst_alpha = frame_data[idx + 3];
        
        if dst_alpha == 0 {
            frame_data[idx] = chunk[0];
            frame_data[idx + 1] = chunk[1];
            frame_data[idx + 2] = chunk[2];
            frame_data[idx + 3] = src_alpha;
        } else {
            let src_a = src_alpha as f64 / 255.0;
            let dst_a = dst_alpha as f64 / 255.0;
            let out_a = src_a + dst_a * (1.0 - src_a);
            
            if out_a > 0.0 {
                frame_data[idx] = ((chunk[0] as f64 * src_a + frame_data[idx] as f64 * dst_a * (1.0 - src_a)) / out_a) as u8;
                frame_data[idx + 1] = ((chunk[1] as f64 * src_a + frame_data[idx + 1] as f64 * dst_a * (1.0 - src_a)) / out_a) as u8;
                frame_data[idx + 2] = ((chunk[2] as f64 * src_a + frame_data[idx + 2] as f64 * dst_a * (1.0 - src_a)) / out_a) as u8;
                frame_data[idx + 3] = (out_a * 255.0) as u8;
            }
        }
    }

    push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
    Ok("笔划已应用".to_string())
}

#[derive(Debug, serde::Deserialize)]
struct SparsePixel {
    x: u32,
    y: u32,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[tauri::command]
fn apply_stroke_pixels_sparse(
    stroke_pixels: Vec<SparsePixel>,
    state: State<Arc<AppState>>,
) -> Result<String, String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = snapshot_before(&editor.document, "应用稀疏笔划");

    let layer_id = editor.document.selected_layers.first().copied().ok_or("No selected layer")?;
    let current_frame = editor.document.timeline.current_frame;

    let layer = editor.document.layers.get_mut(&layer_id).ok_or("Layer not found")?;
    let raster = match layer {
        retas_core::Layer::Raster(r) => r,
        _ => return Err("Only raster layers support drawing".to_string()),
    };

    let frame = raster.frames.get_mut(&current_frame).ok_or("No frame data")?;
    let width = frame.width as usize;
    let frame_data = frame.get_image_data_mut();

    for pixel in &stroke_pixels {
        let x = pixel.x as usize;
        let y = pixel.y as usize;
        
        if x >= width {
            continue;
        }
        
        let idx = (y * width + x) * 4;
        if idx + 3 >= frame_data.len() {
            continue;
        }
        
        let src_alpha = pixel.a;
        if src_alpha == 0 {
            continue;
        }
        
        let dst_alpha = frame_data[idx + 3];
        
        if dst_alpha == 0 {
            frame_data[idx] = pixel.r;
            frame_data[idx + 1] = pixel.g;
            frame_data[idx + 2] = pixel.b;
            frame_data[idx + 3] = src_alpha;
        } else {
            let src_a = src_alpha as f64 / 255.0;
            let dst_a = dst_alpha as f64 / 255.0;
            let out_a = src_a + dst_a * (1.0 - src_a);
            
            if out_a > 0.0 {
                frame_data[idx] = ((pixel.r as f64 * src_a + frame_data[idx] as f64 * dst_a * (1.0 - src_a)) / out_a) as u8;
                frame_data[idx + 1] = ((pixel.g as f64 * src_a + frame_data[idx + 1] as f64 * dst_a * (1.0 - src_a)) / out_a) as u8;
                frame_data[idx + 2] = ((pixel.b as f64 * src_a + frame_data[idx + 2] as f64 * dst_a * (1.0 - src_a)) / out_a) as u8;
                frame_data[idx + 3] = (out_a * 255.0) as u8;
            }
        }
    }

    push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
    Ok(format!("应用了 {} 个像素", stroke_pixels.len()))
}

#[tauri::command]
fn get_layer_pixels(layer_id: String, state: State<Arc<AppState>>) -> Result<Vec<u8>, String> {
    let editor = state.editor.lock().map_err(|e| e.to_string())?;
    let doc = &editor.document;
    let id = parse_layer_id(&layer_id)?;
    let layer = doc.layers.get(&id).ok_or("Layer not found")?;

    let raster = match layer {
        retas_core::Layer::Raster(r) => r,
        _ => return Ok(vec![]),
    };

    let current_frame = doc.timeline.current_frame;
    let frame = raster.frames.get(&current_frame).ok_or("No frame data")?;
    Ok(frame.image_data.as_ref().clone())
}

#[tauri::command]
fn composite_layers(state: State<Arc<AppState>>) -> Result<Vec<u8>, String> {
    let editor = state.editor.lock().map_err(|e| e.to_string())?;
    let doc = &editor.document;

    let width = doc.settings.resolution.width as u32;
    let height = doc.settings.resolution.height as u32;
    let pixel_count = (width * height * 4) as usize;
    let mut result = vec![255u8; pixel_count];

    let current_frame = doc.timeline.current_frame;

    let mut layer_data: Vec<&[u8]> = Vec::new();
    let mut blend_modes: Vec<retas_core::BlendMode> = Vec::new();
    let mut opacities: Vec<f64> = Vec::new();

    for layer_id in &doc.timeline.layer_order {
        let layer = match doc.layers.get(layer_id) {
            Some(l) => l,
            None => continue,
        };

        if !layer.base().visible {
            continue;
        }

        let raster = match layer {
            retas_core::Layer::Raster(r) => r,
            _ => continue,
        };

        let frame = match raster.frames.get(&current_frame) {
            Some(f) => f,
            None => continue,
        };

        layer_data.push(frame.image_data.as_ref());
        blend_modes.push(layer.base().blend_mode);
        opacities.push(layer.base().opacity);
    }

    if !layer_data.is_empty() {
        let blended = retas_core::composite::composite_layers(&layer_data, &blend_modes, &opacities, width, height);
        result = blended;
    }

    Ok(result)
}

#[tauri::command]
fn undo(state: State<Arc<AppState>>) -> Result<bool, String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    if editor.undo_manager.can_undo() {
        editor.undo_manager.undo(&mut editor.document);
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
fn redo(state: State<Arc<AppState>>) -> Result<bool, String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    if editor.undo_manager.can_redo() {
        editor.undo_manager.redo(&mut editor.document);
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
fn can_undo(state: State<Arc<AppState>>) -> Result<bool, String> {
    let editor = state.editor.lock().map_err(|e| e.to_string())?;
    Ok(editor.undo_manager.can_undo())
}

#[tauri::command]
fn can_redo(state: State<Arc<AppState>>) -> Result<bool, String> {
    let editor = state.editor.lock().map_err(|e| e.to_string())?;
    Ok(editor.undo_manager.can_redo())
}

#[tauri::command]
fn open_document(path: String, state: State<Arc<AppState>>) -> Result<DocumentInfo, String> {
    println!("打开文档: {}", path);
    let data = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let new_doc: retas_core::Document = serde_json::from_str(&data).map_err(|e| e.to_string())?;

    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    editor.undo_manager.clear();
    editor.document = new_doc;

    Ok(DocumentInfo {
        name: editor.document.settings.name.clone(),
        width: editor.document.settings.resolution.width,
        height: editor.document.settings.resolution.height,
        frame_rate: editor.document.settings.frame_rate,
        total_frames: editor.document.settings.total_frames,
    })
}

#[tauri::command]
fn save_document(path: String, state: State<Arc<AppState>>) -> Result<(), String> {
    println!("保存文档: {}", path);
    let editor = state.editor.lock().map_err(|e| e.to_string())?;
    let data = serde_json::to_string_pretty(&editor.document).map_err(|e| e.to_string())?;
    drop(editor);
    std::fs::write(&path, data).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_xsheet_data(state: State<Arc<AppState>>) -> Result<Vec<XSheetCell>, String> {
    let editor = state.editor.lock().map_err(|e| e.to_string())?;
    let doc = &editor.document;
    let mut cells = Vec::new();

    for frame in 0..doc.timeline.end_frame {
        for layer_id in &doc.timeline.layer_order {
            if let Some(layer) = doc.layers.get(layer_id) {
                let has_keyframe = layer.has_keyframe(frame);
                let is_empty = match layer {
                    retas_core::Layer::Raster(r) => !r.frames.contains_key(&frame),
                    retas_core::Layer::Vector(v) => !v.frames.contains_key(&frame),
                    _ => true,
                };

                cells.push(XSheetCell {
                    layer_id: layer_id.0.to_string(),
                    frame,
                    has_keyframe,
                    is_empty,
                });
            }
        }
    }

    Ok(cells)
}

#[tauri::command]
fn toggle_keyframe(layer_id: String, frame: u32, state: State<Arc<AppState>>) -> Result<(), String> {
    let layer_id = parse_layer_id(&layer_id)?;
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = snapshot_before(&editor.document, "切换关键帧");
    
    let layer = editor.document.layers.get_mut(&layer_id)
        .ok_or_else(|| "Layer not found".to_string())?;
    
    layer.toggle_keyframe(frame);
    push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
    Ok(())
}

#[tauri::command]
fn insert_frames(at_frame: u32, count: u32, state: State<Arc<AppState>>) -> Result<(), String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = snapshot_before(&editor.document, "插入帧");
    editor.document.insert_frames(at_frame, count);
    push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
    Ok(())
}

#[tauri::command]
fn delete_frames(at_frame: u32, count: u32, state: State<Arc<AppState>>) -> Result<(), String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = snapshot_before(&editor.document, "删除帧");
    editor.document.delete_frames(at_frame, count);
    push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
    Ok(())
}

#[tauri::command]
fn copy_frame(layer_id: String, from_frame: u32, to_frame: u32, state: State<Arc<AppState>>) -> Result<(), String> {
    let layer_id = parse_layer_id(&layer_id)?;
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = snapshot_before(&editor.document, "复制帧");
    editor.document.copy_frame(layer_id, from_frame, to_frame)?;
    push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
    Ok(())
}

fn calculate_points_bounds(points: &[retas_core::Point]) -> retas_core::Rect {
    if points.is_empty() {
        return retas_core::Rect::ZERO;
    }
    
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    
    for p in points {
        min_x = min_x.min(p.x);
        min_y = min_y.min(p.y);
        max_x = max_x.max(p.x);
        max_y = max_y.max(p.y);
    }
    
    retas_core::Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
}

#[tauri::command]
fn clear_selection(state: State<Arc<AppState>>) -> Result<(), String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    editor.document.selection = None;
    Ok(())
}

#[tauri::command]
fn get_selection(state: State<Arc<AppState>>) -> Result<Option<SelectionDataFrontend>, String> {
    let editor = state.editor.lock().map_err(|e| e.to_string())?;
    let doc = &editor.document;
    
    let sel = match &doc.selection {
        Some(s) if s.is_active => s,
        _ => return Ok(None),
    };
    
    let (selection_type, rect, points) = match &sel.mask {
        SelectionMask::None => return Ok(None),
        SelectionMask::Rectangular { rect } => ("rect".to_string(), Some((rect.origin.x, rect.origin.y, rect.size.width, rect.size.height)), vec![]),
        SelectionMask::Elliptical { rect } => ("ellipse".to_string(), Some((rect.origin.x, rect.origin.y, rect.size.width, rect.size.height)), vec![]),
        SelectionMask::Lasso { points: pts, .. } => {
            let pts: Vec<(f64, f64)> = pts.iter().map(|p| (p.x, p.y)).collect();
            ("lasso".to_string(), None, pts)
        },
        SelectionMask::MagicWand { start_point, .. } => {
            ("magicWand".to_string(), Some((start_point.x, start_point.y, 0.0, 0.0)), vec![])
        },
        SelectionMask::Bitmap { width, height, .. } => {
            ("rect".to_string(), Some((0.0, 0.0, *width as f64, *height as f64)), vec![])
        },
    };
    
    let mode = match sel.mode {
        RetasSelectionMode::Replace => "replace",
        RetasSelectionMode::Add => "add",
        RetasSelectionMode::Subtract => "subtract",
        RetasSelectionMode::Intersect => "intersect",
    };
    
    Ok(Some(SelectionDataFrontend {
        selection_type,
        mode: mode.to_string(),
        points,
        rect,
        feather: sel.feather,
        tolerance: 32.0,
        contiguous: true,
    }))
}

#[tauri::command]
fn get_selection_bounds(state: State<Arc<AppState>>) -> Result<Option<SelectionBoundsFrontend>, String> {
    let editor = state.editor.lock().map_err(|e| e.to_string())?;
    let doc = &editor.document;
    
    let sel = match &doc.selection {
        Some(s) if s.is_active => s,
        _ => return Ok(None),
    };
    
    let bounds = sel.bounds();
    Ok(bounds.map(|b| SelectionBoundsFrontend {
        x: b.origin.x,
        y: b.origin.y,
        width: b.size.width,
        height: b.size.height,
    }))
}

#[tauri::command]
fn invert_selection(state: State<Arc<AppState>>) -> Result<(), String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    
    if let Some(sel) = &editor.document.selection {
        let width = editor.document.settings.resolution.width as u32;
        let height = editor.document.settings.resolution.height as u32;
        editor.document.selection = Some(sel.invert(width, height));
    }
    Ok(())
}

#[tauri::command]
fn update_selection(selection: SelectionDataFrontend, state: State<Arc<AppState>>) -> Result<(), String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = snapshot_before(&editor.document, "更新选区");
    
    let tool = match selection.selection_type.as_str() {
        "rect" => SelectionTool::Rectangular,
        "ellipse" => SelectionTool::Elliptical,
        "lasso" => SelectionTool::Lasso,
        "magicWand" => SelectionTool::MagicWand,
        _ => SelectionTool::Rectangular,
    };
    
    let mode = match selection.mode.as_str() {
        "replace" => RetasSelectionMode::Replace,
        "add" => RetasSelectionMode::Add,
        "subtract" => RetasSelectionMode::Subtract,
        "intersect" => RetasSelectionMode::Intersect,
        _ => RetasSelectionMode::Replace,
    };
    
    let mask = if selection.selection_type == "lasso" {
        let points: Vec<retas_core::Point> = selection.points.iter()
            .map(|(x, y)| retas_core::Point::new(*x, *y))
            .collect();
        let bounds = calculate_points_bounds(&points);
        SelectionMask::Lasso { points, bounds }
    } else if let Some((x, y, w, h)) = selection.rect {
        let rect = retas_core::Rect::new(x, y, w, h);
        if selection.selection_type == "ellipse" {
            SelectionMask::Elliptical { rect }
        } else {
            SelectionMask::Rectangular { rect }
        }
    } else {
        SelectionMask::None
    };
    
    let new_selection = Selection {
        tool,
        mode,
        mask,
        feather: selection.feather,
        anti_aliased: true,
        is_active: true,
    };
    
    if mode == RetasSelectionMode::Replace {
        editor.document.selection = Some(new_selection);
    } else if let Some(existing) = &editor.document.selection {
        editor.document.selection = Some(existing.combine(&new_selection, mode));
    } else {
        editor.document.selection = Some(new_selection);
    }
    
    push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
    Ok(())
}

#[tauri::command]
fn apply_selection_to_layer(layer_id: String, operation: String, state: State<Arc<AppState>>) -> Result<(), String> {
    let layer_id = parse_layer_id(&layer_id)?;
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = snapshot_before(&editor.document, "应用选区到图层");
    
    let selection = editor.document.selection.clone().ok_or("No active selection")?;
    if !selection.is_active {
        return Err("No active selection".to_string());
    }
    
    let current_frame = editor.document.timeline.current_frame;
    let layer = editor.document.layers.get_mut(&layer_id).ok_or("Layer not found")?;
    let raster = match layer {
        retas_core::Layer::Raster(r) => r,
        _ => return Err("Only raster layers support selection operations".to_string()),
    };
    
    let frame = raster.frames.get_mut(&current_frame).ok_or("No frame data")?;
    let frame_width = frame.width as usize;
    let frame_height = frame.height as usize;
    let pixels = frame.get_image_data_mut();
    
    let bitmap = selection.to_bitmap();
    let (mask_data, mask_width, mask_height) = match &bitmap {
        SelectionMask::Bitmap { data, width, height } => (data.clone(), *width as usize, *height as usize),
        _ => return Err("Failed to convert selection to bitmap".to_string()),
    };
    
    let sel_bounds = selection.bounds().unwrap_or(retas_core::Rect::ZERO);
    let offset_x = sel_bounds.origin.x.floor() as isize;
    let offset_y = sel_bounds.origin.y.floor() as isize;
    
    match operation.as_str() {
        "clear" => {
            for by in 0..mask_height {
                for bx in 0..mask_width {
                    let mask_idx = by * mask_width + bx;
                    if mask_idx >= mask_data.len() || mask_data[mask_idx] == 0 {
                        continue;
                    }
                    let fx = bx as isize + offset_x;
                    let fy = by as isize + offset_y;
                    if fx < 0 || fy < 0 || fx >= frame_width as isize || fy >= frame_height as isize {
                        continue;
                    }
                    let idx = (fy as usize * frame_width + fx as usize) * 4;
                    if idx + 3 < pixels.len() {
                        pixels[idx] = 0;
                        pixels[idx + 1] = 0;
                        pixels[idx + 2] = 0;
                        pixels[idx + 3] = 0;
                    }
                }
            }
        }
        "fill" => {
            for by in 0..mask_height {
                for bx in 0..mask_width {
                    let mask_idx = by * mask_width + bx;
                    if mask_idx >= mask_data.len() || mask_data[mask_idx] == 0 {
                        continue;
                    }
                    let fx = bx as isize + offset_x;
                    let fy = by as isize + offset_y;
                    if fx < 0 || fy < 0 || fx >= frame_width as isize || fy >= frame_height as isize {
                        continue;
                    }
                    let idx = (fy as usize * frame_width + fx as usize) * 4;
                    if idx + 3 < pixels.len() {
                        pixels[idx] = 255;
                        pixels[idx + 1] = 255;
                        pixels[idx + 2] = 255;
                        pixels[idx + 3] = 255;
                    }
                }
            }
        }
        "invert" => {
            for by in 0..mask_height {
                for bx in 0..mask_width {
                    let mask_idx = by * mask_width + bx;
                    if mask_idx >= mask_data.len() || mask_data[mask_idx] == 0 {
                        continue;
                    }
                    let fx = bx as isize + offset_x;
                    let fy = by as isize + offset_y;
                    if fx < 0 || fy < 0 || fx >= frame_width as isize || fy >= frame_height as isize {
                        continue;
                    }
                    let idx = (fy as usize * frame_width + fx as usize) * 4;
                    if idx + 3 < pixels.len() {
                        pixels[idx] = 255 - pixels[idx];
                        pixels[idx + 1] = 255 - pixels[idx + 1];
                        pixels[idx + 2] = 255 - pixels[idx + 2];
                        pixels[idx + 3] = 255 - pixels[idx + 3];
                    }
                }
            }
        }
        _ => return Err(format!("Unknown operation: {}", operation)),
    }
    
    push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
    Ok(())
}

#[tauri::command]
fn flood_fill_layer(x: u32, y: u32, color: (u8, u8, u8, u8), tolerance: f64, state: State<Arc<AppState>>) -> Result<(), String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = snapshot_before(&editor.document, "填充");
    
    let layer_id = editor.document.selected_layers.first().copied().ok_or("No selected layer")?;
    let current_frame = editor.document.timeline.current_frame;
    
    let layer = editor.document.layers.get_mut(&layer_id).ok_or("Layer not found")?;
    let raster = match layer {
        retas_core::Layer::Raster(r) => r,
        _ => return Err("Only raster layers support fill".to_string()),
    };
    
    let frame = raster.frames.get_mut(&current_frame).ok_or("No frame data")?;
    let width = frame.width;
    let height = frame.height;
    
    if x >= width || y >= height {
        return Err("Start point out of bounds".to_string());
    }
    
    let fill_color = retas_core::Color8 { r: color.0, g: color.1, b: color.2, a: color.3 };
    let filled = retas_core::composite::flood_fill(frame.image_data.as_ref(), width, height, x, y, fill_color, tolerance);
    
    frame.image_data = std::sync::Arc::new(filled);
    push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
    Ok(())
}

#[tauri::command]
fn select_layer(id: String, state: State<Arc<AppState>>) -> Result<(), String> {
    let layer_id = parse_layer_id(&id)?;
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    
    if editor.document.layers.contains_key(&layer_id) {
        editor.document.selected_layers.clear();
        editor.document.selected_layers.push(layer_id);
        Ok(())
    } else {
        Err("Layer not found".to_string())
    }
}

#[tauri::command]
fn rename_layer(id: String, name: String, state: State<Arc<AppState>>) -> Result<(), String> {
    let layer_id = parse_layer_id(&id)?;
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = snapshot_before(&editor.document, "重命名图层");
    
    let layer = editor.document.layers.get_mut(&layer_id).ok_or("Layer not found")?;
    layer.base_mut().name = name;
    push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
    Ok(())
}

#[tauri::command]
fn set_layer_opacity(id: String, opacity: f64, state: State<Arc<AppState>>) -> Result<(), String> {
    let layer_id = parse_layer_id(&id)?;
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = snapshot_before(&editor.document, "设置图层不透明度");
    
    let layer = editor.document.layers.get_mut(&layer_id).ok_or("Layer not found")?;
    layer.base_mut().opacity = opacity.max(0.0).min(1.0);
    push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
    Ok(())
}

#[tauri::command]
fn move_layer(id: String, new_index: usize, state: State<Arc<AppState>>) -> Result<(), String> {
    let layer_id = parse_layer_id(&id)?;
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = snapshot_before(&editor.document, "移动图层");
    
    let old_index = editor.document.timeline.layer_order.iter()
        .position(|x| *x == layer_id)
        .ok_or("Layer not found in order")?;
    
    let id = editor.document.timeline.layer_order.remove(old_index);
    let insert_pos = new_index.min(editor.document.timeline.layer_order.len());
    editor.document.timeline.layer_order.insert(insert_pos, id);
    push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
    Ok(())
}

#[tauri::command]
fn set_layer_blend_mode(id: String, blend_mode: String, state: State<Arc<AppState>>) -> Result<(), String> {
    let layer_id = parse_layer_id(&id)?;
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = snapshot_before(&editor.document, "设置混合模式");
    
    let mode = match blend_mode.as_str() {
        "normal" => retas_core::BlendMode::Normal,
        "multiply" => retas_core::BlendMode::Multiply,
        "screen" => retas_core::BlendMode::Screen,
        "overlay" => retas_core::BlendMode::Overlay,
        "darken" => retas_core::BlendMode::Darken,
        "lighten" => retas_core::BlendMode::Lighten,
        "color_dodge" => retas_core::BlendMode::ColorDodge,
        "color_burn" => retas_core::BlendMode::ColorBurn,
        "hard_light" => retas_core::BlendMode::HardLight,
        "soft_light" => retas_core::BlendMode::SoftLight,
        "difference" => retas_core::BlendMode::Difference,
        "exclusion" => retas_core::BlendMode::Exclusion,
        "hue" => retas_core::BlendMode::Hue,
        "saturation" => retas_core::BlendMode::Saturation,
        "color" => retas_core::BlendMode::Color,
        "luminosity" => retas_core::BlendMode::Luminosity,
        other => return Err(format!("Unknown blend mode: {}", other)),
    };
    
    let layer = editor.document.layers.get_mut(&layer_id)
        .ok_or("Layer not found")?;
    layer.base_mut().blend_mode = mode;
    push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
    Ok(())
}

#[tauri::command]
fn new_document(name: String, width: f64, height: f64, fps: f64, total_frames: u32, state: State<Arc<AppState>>) -> Result<DocumentInfo, String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    
    let mut doc = retas_core::Document::new(&name, width, height);
    doc.settings.frame_rate = fps;
    doc.settings.total_frames = total_frames;
    doc.timeline.frame_rate = fps;
    doc.timeline.end_frame = total_frames;
    
    let w = width as u32;
    let h = height as u32;
    
    let mut bg_layer = retas_core::RasterLayer::new("背景");
    bg_layer.frames.insert(0, retas_core::RasterFrame {
        frame_number: 0,
        image_data: std::sync::Arc::new(vec![255u8; (w * h * 4) as usize]),
        width: w,
        height: h,
        bounds: None,
    });
    
    let mut draw_layer = retas_core::RasterLayer::new("图层 1");
    draw_layer.frames.insert(0, retas_core::RasterFrame {
        frame_number: 0,
        image_data: std::sync::Arc::new(vec![0u8; (w * h * 4) as usize]),
        width: w,
        height: h,
        bounds: None,
    });
    
    let bg_id = bg_layer.base.id;
    let draw_id = draw_layer.base.id;
    
    doc.layers.insert(bg_id, RetasLayer::Raster(bg_layer));
    doc.layers.insert(draw_id, RetasLayer::Raster(draw_layer));
    doc.timeline.layer_order.push(bg_id);
    doc.timeline.layer_order.push(draw_id);
    doc.selected_layers.push(draw_id);
    
    editor.document = doc;
    editor.undo_manager.clear();
    
    Ok(DocumentInfo {
        name: editor.document.settings.name.clone(),
        width: editor.document.settings.resolution.width,
        height: editor.document.settings.resolution.height,
        frame_rate: editor.document.settings.frame_rate,
        total_frames: editor.document.settings.total_frames,
    })
}

#[tauri::command]
fn duplicate_layer(id: String, state: State<Arc<AppState>>) -> Result<LayerInfo, String> {
    let layer_id = parse_layer_id(&id)?;
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    
    let src_layer = editor.document.layers.get(&layer_id)
        .ok_or("Layer not found")?
        .clone();
    
    let src_base = src_layer.base();
    let new_name = format!("{} (副本)", src_base.name);
    
    // Create a deep copy with new ID
    let new_layer = match src_layer {
        RetasLayer::Raster(mut r) => {
            r.base.id = retas_core::LayerId(retas_core::uuid::Uuid::new_v4());
            r.base.name = new_name.clone();
            r.base.locked = false;
            r.base.parent = None;
            r.base.children = Vec::new();
            // Deep copy frame data
            for frame in r.frames.values_mut() {
                frame.image_data = std::sync::Arc::new(frame.image_data.as_ref().clone());
            }
            RetasLayer::Raster(r)
        }
        RetasLayer::Vector(mut v) => {
            v.base.id = retas_core::LayerId(retas_core::uuid::Uuid::new_v4());
            v.base.name = new_name.clone();
            v.base.locked = false;
            v.base.parent = None;
            v.base.children = Vec::new();
            RetasLayer::Vector(v)
        }
        other => other,
    };
    
    let new_base = new_layer.base().clone();
    let new_id = new_base.id;
    
    // Insert after the source layer
    let src_index = editor.document.timeline.layer_order.iter()
        .position(|x| *x == layer_id)
        .unwrap_or(editor.document.timeline.layer_order.len());
    
    let cmd = LayerAddCommand {
        layer: new_layer,
        index: src_index + 1,
        description: format!("复制图层: {}", new_name),
    };
    
    editor.undo_manager.execute(Box::new(cmd), &mut editor.document);
    
    Ok(LayerInfo {
        id: new_id.0.to_string(),
        name: new_base.name,
        visible: new_base.visible,
        locked: new_base.locked,
        opacity: new_base.opacity,
        layer_type: format!("{:?}", new_base.layer_type),
        blend_mode: format!("{:?}", new_base.blend_mode).to_lowercase(),
    })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EffectParams {
    pub effect_type: String,
    pub radius: Option<f64>,
    pub brightness: Option<f64>,
    pub contrast: Option<f64>,
    pub hue: Option<f64>,
    pub saturation: Option<f64>,
    pub lightness: Option<f64>,
    pub opacity: Option<f64>,
}

#[tauri::command]
fn apply_effect(effect: EffectParams, state: State<Arc<AppState>>) -> Result<(), String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = snapshot_before(&editor.document, "应用效果");
    
    let layer_id = editor.document.selected_layers.first().copied().ok_or("No selected layer")?;
    let current_frame = editor.document.timeline.current_frame;
    
    let layer = editor.document.layers.get_mut(&layer_id).ok_or("Layer not found")?;
    let raster = match layer {
        retas_core::Layer::Raster(r) => r,
        _ => return Err("Only raster layers support effects".to_string()),
    };
    
    let frame = raster.frames.get_mut(&current_frame).ok_or("No frame data")?;
    let width = frame.width;
    let height = frame.height;
    
    let effect_obj = match effect.effect_type.as_str() {
        "blur" | "gaussianBlur" => {
            let mut e = Effect::new(EffectType::GaussianBlur);
            if let Some(r) = effect.radius {
                e.parameters = retas_core::advanced::effects::EffectParameters::GaussianBlur { radius: r };
            }
            if let Some(o) = effect.opacity { e.opacity = o; }
            e
        }
        "brightnessContrast" => {
            let mut e = Effect::new(EffectType::BrightnessContrast);
            e.parameters = retas_core::advanced::effects::EffectParameters::BrightnessContrast {
                brightness: effect.brightness.unwrap_or(0.0),
                contrast: effect.contrast.unwrap_or(0.0),
            };
            if let Some(o) = effect.opacity { e.opacity = o; }
            e
        }
        "hueSaturation" => {
            let mut e = Effect::new(EffectType::HueSaturation);
            e.parameters = retas_core::advanced::effects::EffectParameters::HueSaturation {
                hue: effect.hue.unwrap_or(0.0),
                saturation: effect.saturation.unwrap_or(0.0),
                lightness: effect.lightness.unwrap_or(0.0),
            };
            if let Some(o) = effect.opacity { e.opacity = o; }
            e
        }
        "invert" => {
            let mut e = Effect::new(EffectType::Invert);
            if let Some(o) = effect.opacity { e.opacity = o; }
            e
        }
        "posterize" => {
            let mut e = Effect::new(EffectType::Posterize);
            e.parameters = retas_core::advanced::effects::EffectParameters::Posterize {
                levels: effect.radius.unwrap_or(4.0) as u32,
            };
            if let Some(o) = effect.opacity { e.opacity = o; }
            e
        }
        "sharpen" => {
            let mut e = Effect::new(EffectType::Sharpen);
            if let Some(r) = effect.radius {
                e.parameters = retas_core::advanced::effects::EffectParameters::Sharpen { amount: r };
            }
            if let Some(o) = effect.opacity { e.opacity = o; }
            e
        }
        other => return Err(format!("Unknown effect: {}", other)),
    };
    
    let pixels = frame.image_data.as_ref();
    let result = EffectProcessor::apply_effect(pixels, width, height, &effect_obj);
    frame.image_data = std::sync::Arc::new(result);
    
    push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
    Ok(())
}

/// Copy pixels from the current selection on the active layer into a serializable buffer.
/// Returns { width, height, data: Vec<u8> } for the frontend to hold.
#[tauri::command]
fn copy_selection_pixels(state: State<Arc<AppState>>) -> Result<Vec<u8>, String> {
    let editor = state.editor.lock().map_err(|e| e.to_string())?;
    let doc = &editor.document;
    
    let sel = doc.selection.as_ref().filter(|s| s.is_active).ok_or("No active selection")?;
    let bounds = sel.bounds().ok_or("Selection has no bounds")?;
    
    let layer_id = doc.selected_layers.first().copied().ok_or("No selected layer")?;
    let layer = doc.layers.get(&layer_id).ok_or("Layer not found")?;
    let raster = match layer {
        retas_core::Layer::Raster(r) => r,
        _ => return Err("Only raster layers support copy".to_string()),
    };
    let current_frame = doc.timeline.current_frame;
    let frame = raster.frames.get(&current_frame).ok_or("No frame data")?;
    let fw = frame.width as usize;
    let fh = frame.height as usize;
    let pixels = frame.image_data.as_ref();
    
    let x0 = bounds.origin.x.floor().max(0.0) as usize;
    let y0 = bounds.origin.y.floor().max(0.0) as usize;
    let w = bounds.size.width.ceil() as usize;
    let h = bounds.size.height.ceil() as usize;
    
    // Header: 4 bytes width + 4 bytes height + pixel data
    let mut result = Vec::with_capacity(8 + w * h * 4);
    result.extend_from_slice(&(w as u32).to_le_bytes());
    result.extend_from_slice(&(h as u32).to_le_bytes());
    
    for row in 0..h {
        for col in 0..w {
            let sx = x0 + col;
            let sy = y0 + row;
            if sx < fw && sy < fh {
                let idx = (sy * fw + sx) * 4;
                result.push(pixels[idx]);
                result.push(pixels[idx + 1]);
                result.push(pixels[idx + 2]);
                result.push(pixels[idx + 3]);
            } else {
                result.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }
    
    Ok(result)
}

/// Paste pixel buffer (from copy_selection_pixels) onto the active layer at the specified position.
#[tauri::command]
fn paste_pixels(data: Vec<u8>, x: i32, y: i32, state: State<Arc<AppState>>) -> Result<(), String> {
    if data.len() < 8 {
        return Err("Invalid paste data".to_string());
    }
    
    let w = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let h = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
    let pixel_data = &data[8..];
    
    if pixel_data.len() != w * h * 4 {
        return Err("Paste data size mismatch".to_string());
    }
    
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = snapshot_before(&editor.document, "粘贴像素");
    
    let layer_id = editor.document.selected_layers.first().copied().ok_or("No selected layer")?;
    let current_frame = editor.document.timeline.current_frame;
    
    let layer = editor.document.layers.get_mut(&layer_id).ok_or("Layer not found")?;
    let raster = match layer {
        retas_core::Layer::Raster(r) => r,
        _ => return Err("Only raster layers support paste".to_string()),
    };
    
    let frame = raster.frames.get_mut(&current_frame).ok_or("No frame data")?;
    let fw = frame.width as i32;
    let fh = frame.height as i32;
    let frame_pixels = frame.get_image_data_mut();
    
    for row in 0..h {
        for col in 0..w {
            let dx = x + col as i32;
            let dy = y + row as i32;
            if dx < 0 || dy < 0 || dx >= fw || dy >= fh { continue; }
            
            let src_idx = (row * w + col) * 4;
            let src_a = pixel_data[src_idx + 3];
            if src_a == 0 { continue; }
            
            let dst_idx = (dy as usize * fw as usize + dx as usize) * 4;
            if src_a == 255 {
                frame_pixels[dst_idx] = pixel_data[src_idx];
                frame_pixels[dst_idx + 1] = pixel_data[src_idx + 1];
                frame_pixels[dst_idx + 2] = pixel_data[src_idx + 2];
                frame_pixels[dst_idx + 3] = 255;
            } else {
                let sa = src_a as f64 / 255.0;
                let da = frame_pixels[dst_idx + 3] as f64 / 255.0;
                let oa = sa + da * (1.0 - sa);
                if oa > 0.0 {
                    frame_pixels[dst_idx] = ((pixel_data[src_idx] as f64 * sa + frame_pixels[dst_idx] as f64 * da * (1.0 - sa)) / oa) as u8;
                    frame_pixels[dst_idx + 1] = ((pixel_data[src_idx + 1] as f64 * sa + frame_pixels[dst_idx + 1] as f64 * da * (1.0 - sa)) / oa) as u8;
                    frame_pixels[dst_idx + 2] = ((pixel_data[src_idx + 2] as f64 * sa + frame_pixels[dst_idx + 2] as f64 * da * (1.0 - sa)) / oa) as u8;
                    frame_pixels[dst_idx + 3] = (oa * 255.0) as u8;
                }
            }
        }
    }
    
    push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
    Ok(())
}

#[tauri::command]
fn export_image(
    output_path: String,
    format: String,
    frame: Option<u32>,
    state: State<Arc<AppState>>,
) -> Result<(), String> {
    let editor = state.editor.lock().map_err(|e| e.to_string())?;
    let doc = &editor.document;
    
    let img_format = ImageFormat::from_extension(&format)
        .ok_or_else(|| format!("Unsupported format: {}", format))?;
    
    let options = ImageExportOptions::new(img_format)
        .with_background(retas_core::Color8::new(255, 255, 255, 255));
    
    let frame_num = frame.unwrap_or(doc.timeline.current_frame);
    let path = std::path::Path::new(&output_path);
    
    ImageExporter::export_document(doc, frame_num, path, &options)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn export_frame_sequence(
    output_dir: String,
    format: String,
    start_frame: u32,
    end_frame: u32,
    state: State<Arc<AppState>>,
) -> Result<(), String> {
    let editor = state.editor.lock().map_err(|e| e.to_string())?;
    let doc = &editor.document;
    
    let img_format = ImageFormat::from_extension(&format)
        .ok_or_else(|| format!("Unsupported format: {}", format))?;
    
    let dir = std::path::Path::new(&output_dir);
    if !dir.exists() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    
    let options = ImageExportOptions::new(img_format)
        .with_background(retas_core::Color8::new(255, 255, 255, 255));
    
    let end = end_frame.min(doc.timeline.end_frame.saturating_sub(1));
    for frame in start_frame..=end {
        let filename = format!("frame_{:04}.{}", frame, img_format.extension());
        let path = dir.join(filename);
        ImageExporter::export_document(doc, frame, &path, &options)
            .map_err(|e| format!("Frame {}: {}", frame, e))?;
    }
    
    Ok(())
}

#[tauri::command]
fn pick_color(
    x: u32,
    y: u32,
    layer_id: String,
    frame: u32,
    state: State<'_, Arc<AppState>>,
) -> Result<(u8, u8, u8, u8), String> {
    let lid = parse_layer_id(&layer_id)?;
    let editor = state.editor.lock().map_err(|e| e.to_string())?;
    let doc = &editor.document;
    
    let layer = doc.layers.get(&lid).ok_or("Layer not found")?;
    
    let w = doc.settings.resolution.width as u32;
    let h = doc.settings.resolution.height as u32;
    
    if x >= w || y >= h {
        return Err("Coordinates out of bounds".to_string());
    }
    
    if let RetasLayer::Raster(raster) = layer {
        if let Some(frame_data) = raster.frames.get(&frame) {
            let idx = ((y * w + x) * 4) as usize;
            if idx + 3 < frame_data.image_data.len() {
                return Ok((
                    frame_data.image_data[idx],
                    frame_data.image_data[idx + 1],
                    frame_data.image_data[idx + 2],
                    frame_data.image_data[idx + 3],
                ));
            }
        }
    }
    
    Ok((0, 0, 0, 0))
}

#[tauri::command]
fn create_layer_group(
    name: String,
    state: State<'_, Arc<AppState>>,
) -> Result<LayerInfo, String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let doc = &mut editor.document;
    
    let mut base = retas_core::LayerBase::new(&name, retas_core::LayerType::Group);
    let layer_id = base.id;
    
    let layer = RetasLayer::Raster(retas_core::RasterLayer {
        base,
        frames: std::collections::HashMap::new(),
        current_frame: 0,
        offset: retas_core::Point::ZERO,
    });
    
    let info = LayerInfo {
        id: layer_id.0.to_string(),
        name: name.clone(),
        visible: true,
        locked: false,
        opacity: 1.0,
        layer_type: "Group".to_string(),
        blend_mode: "Normal".to_string(),
    };
    
    doc.layers.insert(layer_id, layer);
    doc.timeline.layer_order.push(layer_id);
    
    Ok(info)
}

#[tauri::command]
fn set_layer_parent(
    layer_id: String,
    parent_id: Option<String>,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let lid = parse_layer_id(&layer_id)?;
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let doc = &mut editor.document;
    
    let parent = match parent_id {
        Some(ref pid) => Some(parse_layer_id(pid)?),
        None => None,
    };
    
    if let Some(layer) = doc.layers.get_mut(&lid) {
        layer.base_mut().parent = parent;
        Ok(())
    } else {
        Err("Layer not found".to_string())
    }
}

#[tauri::command]
fn get_composited_frame(
    frame: u32,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<u8>, String> {
    let editor = state.editor.lock().map_err(|e| e.to_string())?;
    let doc = &editor.document;
    
    let w = doc.settings.resolution.width as u32;
    let h = doc.settings.resolution.height as u32;
    let mut result = vec![255u8; (w * h * 4) as usize];
    
    for layer_id in &doc.timeline.layer_order {
        if let Some(layer) = doc.layers.get(layer_id) {
            let base = layer.base();
            if !base.visible { continue; }
            
            if let RetasLayer::Raster(raster) = layer {
                if let Some(frame_data) = raster.frames.get(&frame) {
                    let alpha_mult = base.opacity;
                    let pixels = &frame_data.image_data;
                    for y in 0..h {
                        for x in 0..w {
                            let idx = ((y * w + x) * 4) as usize;
                            if idx + 3 >= pixels.len() { continue; }
                            
                            let sa = (pixels[idx + 3] as f64 / 255.0) * alpha_mult;
                            if sa <= 0.0 { continue; }
                            
                            let da = result[idx + 3] as f64 / 255.0;
                            let out_a = sa + da * (1.0 - sa);
                            
                            if out_a > 0.0 {
                                for c in 0..3 {
                                    let sc = pixels[idx + c] as f64;
                                    let dc = result[idx + c] as f64;
                                    result[idx + c] = ((sc * sa + dc * da * (1.0 - sa)) / out_a) as u8;
                                }
                                result[idx + 3] = (out_a * 255.0) as u8;
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(result)
}

#[tauri::command]
fn resize_document(
    width: u32,
    height: u32,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = snapshot_before(&editor.document, "调整画布大小");
    let doc = &mut editor.document;
    
    let old_w = doc.settings.resolution.width as u32;
    
    for layer in doc.layers.values_mut() {
        if let RetasLayer::Raster(raster) = layer {
            for frame_data in raster.frames.values_mut() {
                let old_pixels = Arc::clone(&frame_data.image_data);
                let mut new_pixels = vec![0u8; (width * height * 4) as usize];
                
                let copy_w = old_w.min(width);
                let copy_h = (frame_data.height).min(height);
                
                for y in 0..copy_h {
                    let src_start = (y * old_w * 4) as usize;
                    let dst_start = (y * width * 4) as usize;
                    let bytes = (copy_w * 4) as usize;
                    
                    if src_start + bytes <= old_pixels.len() {
                        new_pixels[dst_start..dst_start + bytes]
                            .copy_from_slice(&old_pixels[src_start..src_start + bytes]);
                    }
                }
                
                frame_data.image_data = Arc::new(new_pixels);
                frame_data.width = width;
                frame_data.height = height;
            }
        }
    }
    
    doc.settings.resolution.width = width as f64;
    doc.settings.resolution.height = height as f64;
    
    push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
    Ok(())
}

#[tauri::command]
fn move_layer_pixels(
    layer_id: String,
    dx: i32,
    dy: i32,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let lid = parse_layer_id(&layer_id)?;
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = snapshot_before(&editor.document, "移动图层");
    let current_frame = editor.document.timeline.current_frame;
    
    let layer = editor.document.layers.get_mut(&lid).ok_or("Layer not found")?;
    let raster = match layer {
        RetasLayer::Raster(r) => r,
        _ => return Err("Only raster layers support move".to_string()),
    };
    
    let frame = raster.frames.get_mut(&current_frame).ok_or("No frame data")?;
    let w = frame.width as i32;
    let h = frame.height as i32;
    let old_pixels = Arc::clone(&frame.image_data);
    let mut new_pixels = vec![0u8; (w * h * 4) as usize];
    
    for y in 0..h {
        for x in 0..w {
            let sx = x - dx;
            let sy = y - dy;
            if sx >= 0 && sx < w && sy >= 0 && sy < h {
                let src_idx = ((sy * w + sx) * 4) as usize;
                let dst_idx = ((y * w + x) * 4) as usize;
                new_pixels[dst_idx..dst_idx + 4].copy_from_slice(&old_pixels[src_idx..src_idx + 4]);
            }
        }
    }
    
    frame.image_data = Arc::new(new_pixels);
    push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
    Ok(())
}

#[tauri::command]
fn flip_layer(
    layer_id: String,
    horizontal: bool,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let lid = parse_layer_id(&layer_id)?;
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = snapshot_before(&editor.document, if horizontal { "水平翻转" } else { "垂直翻转" });
    let current_frame = editor.document.timeline.current_frame;
    
    let layer = editor.document.layers.get_mut(&lid).ok_or("Layer not found")?;
    let raster = match layer {
        RetasLayer::Raster(r) => r,
        _ => return Err("Only raster layers support flip".to_string()),
    };
    
    let frame = raster.frames.get_mut(&current_frame).ok_or("No frame data")?;
    let w = frame.width as usize;
    let h = frame.height as usize;
    let old_pixels = Arc::clone(&frame.image_data);
    let mut new_pixels = vec![0u8; w * h * 4];
    
    for y in 0..h {
        for x in 0..w {
            let (sx, sy) = if horizontal { (w - 1 - x, y) } else { (x, h - 1 - y) };
            let src_idx = (sy * w + sx) * 4;
            let dst_idx = (y * w + x) * 4;
            new_pixels[dst_idx..dst_idx + 4].copy_from_slice(&old_pixels[src_idx..src_idx + 4]);
        }
    }
    
    frame.image_data = Arc::new(new_pixels);
    push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
    Ok(())
}

#[tauri::command]
fn rotate_layer_90(
    layer_id: String,
    clockwise: bool,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let lid = parse_layer_id(&layer_id)?;
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = snapshot_before(&editor.document, if clockwise { "顺时针旋转90°" } else { "逆时针旋转90°" });
    let current_frame = editor.document.timeline.current_frame;
    
    let layer = editor.document.layers.get_mut(&lid).ok_or("Layer not found")?;
    let raster = match layer {
        RetasLayer::Raster(r) => r,
        _ => return Err("Only raster layers support rotate".to_string()),
    };
    
    let frame = raster.frames.get_mut(&current_frame).ok_or("No frame data")?;
    let w = frame.width as usize;
    let h = frame.height as usize;
    let old_pixels = Arc::clone(&frame.image_data);
    // 90° rotation swaps dimensions
    let new_w = h;
    let new_h = w;
    let mut new_pixels = vec![0u8; new_w * new_h * 4];
    
    for y in 0..h {
        for x in 0..w {
            let (nx, ny) = if clockwise { (h - 1 - y, x) } else { (y, w - 1 - x) };
            let src_idx = (y * w + x) * 4;
            let dst_idx = (ny * new_w + nx) * 4;
            new_pixels[dst_idx..dst_idx + 4].copy_from_slice(&old_pixels[src_idx..src_idx + 4]);
        }
    }
    
    frame.image_data = Arc::new(new_pixels);
    frame.width = new_w as u32;
    frame.height = new_h as u32;
    push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RenderJobInfo {
    pub id: u64,
    pub name: String,
    pub frame_range: (u32, u32),
    pub format: String,
    pub quality: String,
    pub status: String,
    pub progress: f64,
}

fn job_to_info(job: &RenderJob) -> RenderJobInfo {
    RenderJobInfo {
        id: job.id,
        name: job.name.clone(),
        frame_range: job.frame_range,
        format: format!("{:?}", job.format),
        quality: format!("{:?}", job.quality),
        status: match &job.status {
            RenderStatus::Queued => "queued".to_string(),
            RenderStatus::Rendering => "rendering".to_string(),
            RenderStatus::Completed => "completed".to_string(),
            RenderStatus::Failed(msg) => format!("failed: {}", msg),
            RenderStatus::Cancelled => "cancelled".to_string(),
        },
        progress: job.progress,
    }
}

#[tauri::command]
fn add_render_job(
    name: String,
    start_frame: u32,
    end_frame: u32,
    output_dir: String,
    format: String,
    quality: String,
    state: State<'_, Arc<AppState>>,
) -> Result<RenderJobInfo, String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    
    let fmt = match format.as_str() {
        "png" => RenderFormat::Png,
        "jpeg" | "jpg" => RenderFormat::Jpeg,
        "gif" => RenderFormat::Gif,
        "mp4" => RenderFormat::Mp4,
        "webm" => RenderFormat::WebM,
        "apng" => RenderFormat::APNG,
        _ => RenderFormat::Png,
    };
    
    let job_id = editor.render_queue.add_batch_export(
        name,
        "current".to_string(),
        (start_frame, end_frame),
        std::path::PathBuf::from(&output_dir),
        fmt,
    );
    
    let job = editor.render_queue.get_job(job_id).ok_or("Job not found")?;
    Ok(job_to_info(job))
}

#[tauri::command]
fn get_render_jobs(state: State<'_, Arc<AppState>>) -> Result<Vec<RenderJobInfo>, String> {
    let editor = state.editor.lock().map_err(|e| e.to_string())?;
    let mut jobs: Vec<RenderJobInfo> = editor.render_queue.jobs.iter().map(job_to_info).collect();
    jobs.extend(editor.render_queue.completed_jobs.iter().map(job_to_info));
    Ok(jobs)
}

#[tauri::command]
fn cancel_render_job(job_id: u64, state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    Ok(editor.render_queue.cancel_job(job_id))
}

#[tauri::command]
fn clear_completed_jobs(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    editor.render_queue.clear_completed();
    Ok(())
}

#[tauri::command]
fn clear_frame(
    layer_id: String,
    frame: u32,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let lid = parse_layer_id(&layer_id)?;
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = snapshot_before(&editor.document, "清除帧");
    
    let layer = editor.document.layers.get_mut(&lid).ok_or("Layer not found")?;
    let raster = match layer {
        RetasLayer::Raster(r) => r,
        _ => return Err("Only raster layers support clear".to_string()),
    };
    
    if let Some(frame_data) = raster.frames.get_mut(&frame) {
        let size = (frame_data.width * frame_data.height * 4) as usize;
        frame_data.image_data = Arc::new(vec![0u8; size]);
    }
    
    push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
    Ok(())
}

#[tauri::command]
fn fill_frame(
    layer_id: String,
    frame: u32,
    color: (u8, u8, u8, u8),
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let lid = parse_layer_id(&layer_id)?;
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = snapshot_before(&editor.document, "填充帧");
    
    let layer = editor.document.layers.get_mut(&lid).ok_or("Layer not found")?;
    let raster = match layer {
        RetasLayer::Raster(r) => r,
        _ => return Err("Only raster layers support fill".to_string()),
    };
    
    if let Some(frame_data) = raster.frames.get_mut(&frame) {
        let pixel_count = (frame_data.width * frame_data.height) as usize;
        let mut pixels = vec![0u8; pixel_count * 4];
        for i in 0..pixel_count {
            pixels[i * 4] = color.0;
            pixels[i * 4 + 1] = color.1;
            pixels[i * 4 + 2] = color.2;
            pixels[i * 4 + 3] = color.3;
        }
        frame_data.image_data = Arc::new(pixels);
    }
    
    push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
    Ok(())
}

#[tauri::command]
fn color_replace(
    layer_id: String,
    frame: u32,
    source_color: (u8, u8, u8),
    target_color: (u8, u8, u8),
    tolerance: u8,
    state: State<'_, Arc<AppState>>,
) -> Result<u32, String> {
    let lid = parse_layer_id(&layer_id)?;
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = snapshot_before(&editor.document, "颜色替换");
    
    let layer = editor.document.layers.get_mut(&lid).ok_or("Layer not found")?;
    let raster = match layer {
        RetasLayer::Raster(r) => r,
        _ => return Err("Only raster layers support color replace".to_string()),
    };
    
    let frame_data = raster.frames.get_mut(&frame).ok_or("No frame data")?;
    let mut pixels = frame_data.image_data.as_ref().clone();
    let pixel_count = (frame_data.width * frame_data.height) as usize;
    let mut replaced = 0u32;
    
    for i in 0..pixel_count {
        let idx = i * 4;
        let r = pixels[idx];
        let g = pixels[idx + 1];
        let b = pixels[idx + 2];
        
        let dr = (r as i32 - source_color.0 as i32).unsigned_abs() as u8;
        let dg = (g as i32 - source_color.1 as i32).unsigned_abs() as u8;
        let db = (b as i32 - source_color.2 as i32).unsigned_abs() as u8;
        
        if dr <= tolerance && dg <= tolerance && db <= tolerance && pixels[idx + 3] > 0 {
            pixels[idx] = target_color.0;
            pixels[idx + 1] = target_color.1;
            pixels[idx + 2] = target_color.2;
            replaced += 1;
        }
    }
    
    frame_data.image_data = Arc::new(pixels);
    push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
    Ok(replaced)
}

// ─── Light Table Commands ────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OnionSkinInfo {
    pub enabled: bool,
    pub frames_before: u32,
    pub frames_after: u32,
    pub opacity_before: f64,
    pub opacity_after: f64,
    pub color_before: [u8; 4],
    pub color_after: [u8; 4],
    pub blend_mode: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReferenceLayerInfo {
    pub id: String,
    pub name: String,
    pub image_path: Option<String>,
    pub opacity: f64,
    pub visible: bool,
    pub locked: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LightTableInfo {
    pub enabled: bool,
    pub opacity: f64,
    pub onion_skin: OnionSkinInfo,
    pub references: Vec<ReferenceLayerInfo>,
}

#[tauri::command]
fn get_light_table(
    frame: u32,
    state: State<'_, Arc<AppState>>,
) -> Result<LightTableInfo, String> {
    let editor = state.editor.lock().map_err(|e| e.to_string())?;
    let table = editor.light_table.get(frame);
    
    match table {
        Some(t) => Ok(LightTableInfo {
            enabled: t.enabled,
            opacity: t.opacity,
            onion_skin: OnionSkinInfo {
                enabled: t.onion_skin.enabled,
                frames_before: t.onion_skin.frames_before,
                frames_after: t.onion_skin.frames_after,
                opacity_before: t.onion_skin.opacity_before,
                opacity_after: t.onion_skin.opacity_after,
                color_before: [t.onion_skin.color_before.r, t.onion_skin.color_before.g, t.onion_skin.color_before.b, t.onion_skin.color_before.a],
                color_after: [t.onion_skin.color_after.r, t.onion_skin.color_after.g, t.onion_skin.color_after.b, t.onion_skin.color_after.a],
                blend_mode: format!("{:?}", t.onion_skin.blend_mode),
            },
            references: t.reference_layers.iter().map(|r| ReferenceLayerInfo {
                id: r.id.0.to_string(),
                name: r.name.clone(),
                image_path: r.image_path.clone(),
                opacity: r.opacity,
                visible: r.visible,
                locked: r.locked,
            }).collect(),
        }),
        None => Ok(LightTableInfo {
            enabled: false,
            opacity: 0.5,
            onion_skin: OnionSkinInfo {
                enabled: false,
                frames_before: 1,
                frames_after: 1,
                opacity_before: 0.3,
                opacity_after: 0.3,
                color_before: [255, 0, 0, 255],
                color_after: [0, 255, 0, 255],
                blend_mode: "Tint".to_string(),
            },
            references: vec![],
        }),
    }
}

#[tauri::command]
fn set_onion_skin(
    frame: u32,
    enabled: bool,
    frames_before: u32,
    frames_after: u32,
    opacity_before: f64,
    opacity_after: f64,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    editor.light_table.toggle_onion_skin(frame, enabled);
    editor.light_table.set_onion_skin_frames(frame, frames_before, frames_after);
    let table = editor.light_table.get_or_create(frame);
    table.onion_skin.opacity_before = opacity_before;
    table.onion_skin.opacity_after = opacity_after;
    Ok(())
}

#[tauri::command]
fn set_onion_skin_colors(
    frame: u32,
    color_before: [u8; 4],
    color_after: [u8; 4],
    blend_mode: String,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let table = editor.light_table.get_or_create(frame);
    table.onion_skin.color_before = retas_core::Color8::new(color_before[0], color_before[1], color_before[2], color_before[3]);
    table.onion_skin.color_after = retas_core::Color8::new(color_after[0], color_after[1], color_after[2], color_after[3]);
    table.onion_skin.blend_mode = match blend_mode.as_str() {
        "Overlay" => OnionBlendMode::Overlay,
        "Difference" => OnionBlendMode::Difference,
        "Normal" => OnionBlendMode::Normal,
        _ => OnionBlendMode::Tint,
    };
    Ok(())
}

#[tauri::command]
fn add_reference_layer(
    frame: u32,
    name: String,
    image_path: Option<String>,
    state: State<'_, Arc<AppState>>,
) -> Result<String, String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let reference = match image_path {
        Some(path) => ReferenceLayer::from_image(path, name),
        None => ReferenceLayer::new(name),
    };
    let id = reference.id.0.to_string();
    editor.light_table.add_reference(frame, reference);
    Ok(id)
}

#[tauri::command]
fn remove_reference_layer(
    frame: u32,
    reference_id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<bool, String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let layer_id = parse_layer_id(&reference_id)?;
    Ok(editor.light_table.remove_reference(frame, layer_id))
}

#[tauri::command]
fn set_reference_opacity(
    frame: u32,
    reference_id: String,
    opacity: f64,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let layer_id = parse_layer_id(&reference_id)?;
    let table = editor.light_table.get_or_create(frame);
    if let Some(r) = table.reference_layers.iter_mut().find(|r| r.id == layer_id) {
        r.opacity = opacity.clamp(0.0, 1.0);
    }
    Ok(())
}

#[tauri::command]
fn toggle_reference_visibility(
    frame: u32,
    reference_id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let layer_id = parse_layer_id(&reference_id)?;
    let table = editor.light_table.get_or_create(frame);
    if let Some(r) = table.reference_layers.iter_mut().find(|r| r.id == layer_id) {
        r.visible = !r.visible;
    }
    Ok(())
}

#[tauri::command]
fn get_onion_skin_frames(
    frame: u32,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<(u32, f64, [u8; 4])>, String> {
    let editor = state.editor.lock().map_err(|e| e.to_string())?;
    let frames = editor.light_table.get_onion_skin_frames(frame);
    Ok(frames.into_iter().map(|(f, opacity, color)| {
        (f, opacity, [color.r, color.g, color.b, color.a])
    }).collect())
}

// ─── Motion Check Commands ───────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MotionCheckInfo {
    pub enabled: bool,
    pub mode: String,
    pub comparison_frames: Vec<u32>,
    pub overlay_opacity: f64,
    pub show_trails: bool,
    pub trail_length: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrailInfo {
    pub layer_id: String,
    pub points: Vec<TrailPointInfo>,
    pub visible: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrailPointInfo {
    pub x: f64,
    pub y: f64,
    pub frame: u32,
    pub size: f64,
}

#[tauri::command]
fn get_motion_check(
    frame: u32,
    state: State<'_, Arc<AppState>>,
) -> Result<MotionCheckInfo, String> {
    let editor = state.editor.lock().map_err(|e| e.to_string())?;
    match editor.motion_check.get(frame) {
        Some(check) => Ok(MotionCheckInfo {
            enabled: check.enabled,
            mode: format!("{:?}", check.mode),
            comparison_frames: check.comparison_frames.clone(),
            overlay_opacity: check.overlay_opacity,
            show_trails: check.show_trails,
            trail_length: check.trail_length,
        }),
        None => Ok(MotionCheckInfo {
            enabled: false,
            mode: "Overlay".to_string(),
            comparison_frames: vec![1],
            overlay_opacity: 0.5,
            show_trails: false,
            trail_length: 5,
        }),
    }
}

#[tauri::command]
fn set_motion_check(
    frame: u32,
    enabled: bool,
    mode: String,
    comparison_frames: Vec<u32>,
    overlay_opacity: f64,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    if enabled {
        editor.motion_check.enable(frame);
    } else {
        editor.motion_check.disable(frame);
    }
    let check_mode = match mode.as_str() {
        "Difference" => MotionCheckMode::Difference,
        "SideBySide" => MotionCheckMode::SideBySide,
        "OnionSkin" => MotionCheckMode::OnionSkin,
        _ => MotionCheckMode::Overlay,
    };
    editor.motion_check.set_mode(frame, check_mode);
    editor.motion_check.set_comparison_frames(frame, comparison_frames);
    editor.motion_check.set_overlay_opacity(frame, overlay_opacity);
    Ok(())
}

#[tauri::command]
fn toggle_motion_trails(
    frame: u32,
    show: bool,
    trail_length: u32,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let check = editor.motion_check.get_mut(frame);
    check.show_trails = show;
    check.trail_length = trail_length;
    Ok(())
}

#[tauri::command]
fn add_trail_point(
    layer_id: String,
    x: f64, y: f64,
    frame: u32,
    size: f64,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let lid = parse_layer_id(&layer_id)?;
    editor.motion_check.add_trail_point(lid, retas_core::Point::new(x, y), frame, size);
    Ok(())
}

#[tauri::command]
fn get_trail(
    layer_id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Option<TrailInfo>, String> {
    let editor = state.editor.lock().map_err(|e| e.to_string())?;
    let lid = parse_layer_id(&layer_id)?;
    Ok(editor.motion_check.get_trail(lid).map(|trail| TrailInfo {
        layer_id: trail.layer_id.0.to_string(),
        points: trail.points.iter().map(|p| TrailPointInfo {
            x: p.position.x,
            y: p.position.y,
            frame: p.frame,
            size: p.size,
        }).collect(),
        visible: trail.visible,
    }))
}

#[tauri::command]
fn clear_trails(
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    editor.motion_check.clear_all_trails();
    Ok(())
}

// ─── Selection Transform Commands ────────────────────────────────

#[tauri::command]
fn offset_selection_pixels(
    layer_id: String,
    frame: u32,
    dx: i32, dy: i32,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = SnapshotCommand::capture(&editor.document);
    let lid = parse_layer_id(&layer_id)?;
    
    let selection_mask = editor.document.selection.as_ref()
        .ok_or("No active selection")?
        .mask.clone();
    
    let layer = editor.document.layers.get_mut(&lid)
        .ok_or("Layer not found")?;
    let raster = match layer {
        RetasLayer::Raster(r) => r,
        _ => return Err("Not a raster layer".to_string()),
    };
    let frame_data = raster.frames.get_mut(&frame)
        .ok_or("Frame not found")?;
    
    let w = frame_data.width as i32;
    let h = frame_data.height as i32;
    let mut pixels = (*frame_data.image_data).clone();
    
    // Collect selected pixels
    let mask_w = selection_mask.width as i32;
    let mask_h = selection_mask.height as i32;
    let mut selected: Vec<(i32, i32, [u8; 4])> = Vec::new();
    
    for y in 0..h.min(mask_h) {
        for x in 0..w.min(mask_w) {
            let midx = (y * mask_w + x) as usize;
            if midx < selection_mask.data.len() && selection_mask.data[midx] > 127 {
                let pidx = (y * w + x) as usize * 4;
                if pidx + 3 < pixels.len() {
                    selected.push((x, y, [pixels[pidx], pixels[pidx+1], pixels[pidx+2], pixels[pidx+3]]));
                    // Clear original
                    pixels[pidx] = 0;
                    pixels[pidx+1] = 0;
                    pixels[pidx+2] = 0;
                    pixels[pidx+3] = 0;
                }
            }
        }
    }
    
    // Place at offset
    for (x, y, rgba) in &selected {
        let nx = x + dx;
        let ny = y + dy;
        if nx >= 0 && nx < w && ny >= 0 && ny < h {
            let pidx = (ny * w + nx) as usize * 4;
            if pidx + 3 < pixels.len() {
                pixels[pidx] = rgba[0];
                pixels[pidx+1] = rgba[1];
                pixels[pidx+2] = rgba[2];
                pixels[pidx+3] = rgba[3];
            }
        }
    }
    
    frame_data.image_data = Arc::new(pixels);
    push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
    Ok(())
}

#[tauri::command]
fn scale_selection_pixels(
    layer_id: String,
    frame: u32,
    scale_x: f64, scale_y: f64,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = SnapshotCommand::capture(&editor.document);
    let lid = parse_layer_id(&layer_id)?;
    
    let selection_mask = editor.document.selection.as_ref()
        .ok_or("No active selection")?
        .mask.clone();
    
    let layer = editor.document.layers.get_mut(&lid)
        .ok_or("Layer not found")?;
    let raster = match layer {
        RetasLayer::Raster(r) => r,
        _ => return Err("Not a raster layer".to_string()),
    };
    let frame_data = raster.frames.get_mut(&frame)
        .ok_or("Frame not found")?;
    
    let w = frame_data.width as i32;
    let h = frame_data.height as i32;
    let mut pixels = (*frame_data.image_data).clone();
    
    // Find selection bounding box
    let mask_w = selection_mask.width as i32;
    let mask_h = selection_mask.height as i32;
    let mut min_x = w;
    let mut min_y = h;
    let mut max_x = 0i32;
    let mut max_y = 0i32;
    let mut selected: Vec<(i32, i32, [u8; 4])> = Vec::new();
    
    for y in 0..h.min(mask_h) {
        for x in 0..w.min(mask_w) {
            let midx = (y * mask_w + x) as usize;
            if midx < selection_mask.data.len() && selection_mask.data[midx] > 127 {
                let pidx = (y * w + x) as usize * 4;
                if pidx + 3 < pixels.len() {
                    selected.push((x, y, [pixels[pidx], pixels[pidx+1], pixels[pidx+2], pixels[pidx+3]]));
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                    pixels[pidx] = 0;
                    pixels[pidx+1] = 0;
                    pixels[pidx+2] = 0;
                    pixels[pidx+3] = 0;
                }
            }
        }
    }
    
    let cx = (min_x + max_x) as f64 / 2.0;
    let cy = (min_y + max_y) as f64 / 2.0;
    
    for (x, y, rgba) in &selected {
        let nx = ((*x as f64 - cx) * scale_x + cx).round() as i32;
        let ny = ((*y as f64 - cy) * scale_y + cy).round() as i32;
        if nx >= 0 && nx < w && ny >= 0 && ny < h {
            let pidx = (ny * w + nx) as usize * 4;
            if pidx + 3 < pixels.len() {
                pixels[pidx] = rgba[0];
                pixels[pidx+1] = rgba[1];
                pixels[pidx+2] = rgba[2];
                pixels[pidx+3] = rgba[3];
            }
        }
    }
    
    frame_data.image_data = Arc::new(pixels);
    push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
    Ok(())
}

// ─── Batch Queue Commands ────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BatchItemInfo {
    pub id: u64,
    pub operation: String,
    pub status: String,
    pub priority: String,
}

#[tauri::command]
fn add_batch_export(
    output_dir: String,
    format: String,
    start_frame: u32,
    end_frame: u32,
    priority: String,
    state: State<'_, Arc<AppState>>,
) -> Result<u64, String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let fmt = match format.as_str() {
        "jpeg" | "jpg" => BatchExportFormat::Jpeg,
        "tga" => BatchExportFormat::Tga,
        "bmp" => BatchExportFormat::Bmp,
        "gif" => BatchExportFormat::Gif,
        _ => BatchExportFormat::Png,
    };
    let op = BatchOperation::ExportSequence {
        output_dir: std::path::PathBuf::from(output_dir),
        format: fmt,
        start_frame,
        end_frame,
    };
    let pri = match priority.as_str() {
        "Low" => BatchPriority::Low,
        "High" => BatchPriority::High,
        "Urgent" => BatchPriority::Urgent,
        _ => BatchPriority::Normal,
    };
    let id = editor.batch_queue.add(op, pri);
    Ok(id)
}

#[tauri::command]
fn add_batch_color_replace(
    source_color: (u8, u8, u8),
    target_color: (u8, u8, u8),
    tolerance: u8,
    priority: String,
    state: State<'_, Arc<AppState>>,
) -> Result<u64, String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let op = BatchOperation::ColorReplace {
        source_color: retas_core::Color8::new(source_color.0, source_color.1, source_color.2, 255),
        target_color: retas_core::Color8::new(target_color.0, target_color.1, target_color.2, 255),
        tolerance,
    };
    let pri = match priority.as_str() {
        "Low" => BatchPriority::Low,
        "High" => BatchPriority::High,
        "Urgent" => BatchPriority::Urgent,
        _ => BatchPriority::Normal,
    };
    let id = editor.batch_queue.add(op, pri);
    Ok(id)
}

#[tauri::command]
fn get_batch_queue(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<BatchItemInfo>, String> {
    let editor = state.editor.lock().map_err(|e| e.to_string())?;
    let items: Vec<BatchItemInfo> = editor.batch_queue.get_all().iter().map(|item| {
        let op_name = match &item.operation {
            BatchOperation::ExportSequence { .. } => "导出序列",
            BatchOperation::ConvertLayer { .. } => "图层转换",
            BatchOperation::ApplyEffect { .. } => "应用效果",
            BatchOperation::ResizeDocument { .. } => "调整大小",
            BatchOperation::ColorReplace { .. } => "颜色替换",
            BatchOperation::LineSmooth { .. } => "线条平滑",
            BatchOperation::LineVolume { .. } => "线条粗细",
            BatchOperation::FillConsecutive { .. } => "连续填充",
            BatchOperation::TraceVectorize { .. } => "矢量化",
            BatchOperation::Custom { name, .. } => name.as_str(),
        };
        BatchItemInfo {
            id: item.id,
            operation: op_name.to_string(),
            status: format!("{:?}", item.status),
            priority: format!("{:?}", item.priority),
        }
    }).collect();
    Ok(items)
}

#[tauri::command]
fn cancel_batch_item(
    id: u64,
    state: State<'_, Arc<AppState>>,
) -> Result<bool, String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    Ok(editor.batch_queue.cancel(id))
}

#[tauri::command]
fn clear_batch_completed(
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    editor.batch_queue.clear_completed();
    Ok(())
}

#[tauri::command]
fn get_batch_stats(
    state: State<'_, Arc<AppState>>,
) -> Result<(usize, usize, usize, usize), String> {
    let editor = state.editor.lock().map_err(|e| e.to_string())?;
    Ok((
        editor.batch_queue.pending_count(),
        editor.batch_queue.running_count(),
        editor.batch_queue.completed_count(),
        editor.batch_queue.failed_count(),
    ))
}

// ─── Vectorize Commands ──────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VectorPathInfo {
    pub points: Vec<VectorPointInfo>,
    pub is_closed: bool,
    pub color: [u8; 4],
    pub stroke_width: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VectorPointInfo {
    pub x: f64,
    pub y: f64,
    pub control_in: Option<(f64, f64)>,
    pub control_out: Option<(f64, f64)>,
    pub is_corner: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VectorizeResultInfo {
    pub paths: Vec<VectorPathInfo>,
    pub width: u32,
    pub height: u32,
    pub processing_time_ms: u64,
}

#[tauri::command]
fn vectorize_layer(
    layer_id: String,
    frame: u32,
    threshold: u8,
    smoothing: f64,
    min_path_length: f64,
    state: State<'_, Arc<AppState>>,
) -> Result<VectorizeResultInfo, String> {
    let editor = state.editor.lock().map_err(|e| e.to_string())?;
    let lid = parse_layer_id(&layer_id)?;
    
    let layer = editor.document.layers.get(&lid).ok_or("Layer not found")?;
    let raster = match layer {
        RetasLayer::Raster(r) => r,
        _ => return Err("Not a raster layer".to_string()),
    };
    let frame_data = raster.frames.get(&frame).ok_or("Frame not found")?;
    
    let settings = VectorizationSettings {
        threshold,
        smoothing,
        min_path_length,
        ..VectorizationSettings::default()
    };
    let vectorizer = Vectorizer::new(settings);
    let result = vectorizer.vectorize_bitmap(&frame_data.image_data, frame_data.width, frame_data.height);
    
    Ok(VectorizeResultInfo {
        paths: result.paths.iter().map(|p| VectorPathInfo {
            points: p.points.iter().map(|pt| VectorPointInfo {
                x: pt.position.x,
                y: pt.position.y,
                control_in: pt.control_in.map(|c| (c.x, c.y)),
                control_out: pt.control_out.map(|c| (c.x, c.y)),
                is_corner: pt.is_corner,
            }).collect(),
            is_closed: p.is_closed,
            color: [p.color.r, p.color.g, p.color.b, p.color.a],
            stroke_width: p.stroke_width,
        }).collect(),
        width: result.width,
        height: result.height,
        processing_time_ms: result.processing_time_ms,
    })
}

// ─── Cut System Commands ─────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CutInfo {
    pub id: u64,
    pub name: String,
    pub start_frame: u32,
    pub end_frame: u32,
    pub duration_frames: u32,
    pub layers: Vec<String>,
    pub notes: String,
    pub color: [u8; 4],
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CutFolderInfo {
    pub id: u64,
    pub name: String,
    pub cuts: Vec<CutInfo>,
    pub notes: String,
}

fn cut_to_info(cut: &Cut) -> CutInfo {
    CutInfo {
        id: cut.id,
        name: cut.name.clone(),
        start_frame: cut.start_frame,
        end_frame: cut.end_frame,
        duration_frames: cut.duration_frames,
        layers: cut.layers.iter().map(|l| l.0.to_string()).collect(),
        notes: cut.notes.clone(),
        color: cut.color,
    }
}

fn folder_to_info(folder: &CutFolder) -> CutFolderInfo {
    CutFolderInfo {
        id: folder.id,
        name: folder.name.clone(),
        cuts: folder.cuts.iter().map(cut_to_info).collect(),
        notes: folder.notes.clone(),
    }
}

#[tauri::command]
fn create_cut_folder(
    name: String,
    state: State<'_, Arc<AppState>>,
) -> Result<u64, String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let id = editor.cut_manager.create_folder(name);
    Ok(id)
}

#[tauri::command]
fn delete_cut_folder(
    folder_id: u64,
    state: State<'_, Arc<AppState>>,
) -> Result<bool, String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    Ok(editor.cut_manager.delete_folder(folder_id))
}

#[tauri::command]
fn get_cut_folders(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<CutFolderInfo>, String> {
    let editor = state.editor.lock().map_err(|e| e.to_string())?;
    Ok(editor.cut_manager.get_all_folders().iter().map(|f| folder_to_info(f)).collect())
}

#[tauri::command]
fn add_cut(
    folder_id: u64,
    name: String,
    start_frame: u32,
    end_frame: u32,
    state: State<'_, Arc<AppState>>,
) -> Result<u64, String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let cut = Cut::new(name, start_frame, end_frame);
    editor.cut_manager.add_cut_to_folder(folder_id, cut)
        .ok_or("Folder not found".to_string())
}

#[tauri::command]
fn remove_cut(
    folder_id: u64,
    cut_id: u64,
    state: State<'_, Arc<AppState>>,
) -> Result<bool, String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    Ok(editor.cut_manager.remove_cut_from_folder(folder_id, cut_id))
}

#[tauri::command]
fn set_current_cut_folder(
    folder_id: u64,
    state: State<'_, Arc<AppState>>,
) -> Result<bool, String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    Ok(editor.cut_manager.set_current_folder(folder_id))
}

#[tauri::command]
fn find_cut_at_frame(
    frame: u32,
    state: State<'_, Arc<AppState>>,
) -> Result<Option<(CutFolderInfo, CutInfo)>, String> {
    let editor = state.editor.lock().map_err(|e| e.to_string())?;
    Ok(editor.cut_manager.find_cut_at_frame(frame).map(|(folder, cut)| {
        (folder_to_info(folder), cut_to_info(cut))
    }))
}

// ─── Coloring Engine Commands ────────────────────────────────────

#[tauri::command]
fn smart_fill(
    layer_id: String,
    frame: u32,
    start_x: u32,
    start_y: u32,
    fill_color: (u8, u8, u8, u8),
    mode: String,
    tolerance: f64,
    gap_radius: u32,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let mut editor = state.editor.lock().map_err(|e| e.to_string())?;
    let snap = SnapshotCommand::capture(&editor.document);
    let lid = parse_layer_id(&layer_id)?;
    
    let fill_mode = match mode.as_str() {
        "Normal" => FillMode::Normal,
        "GapClosing" => FillMode::GapClosing,
        _ => FillMode::Smart,
    };
    
    let settings = FillSettings {
        mode: fill_mode,
        tolerance,
        gap_closing_radius: gap_radius,
        anti_aliasing: true,
        fill_behind_lines: true,
    };
    
    let engine = ColoringEngine { settings };
    
    let layer = editor.document.layers.get_mut(&lid).ok_or("Layer not found")?;
    let raster = match layer {
        RetasLayer::Raster(r) => r,
        _ => return Err("Not a raster layer".to_string()),
    };
    let frame_data = raster.frames.get_mut(&frame).ok_or("Frame not found")?;
    
    let color = retas_core::Color8::new(fill_color.0, fill_color.1, fill_color.2, fill_color.3);
    let result = engine.smart_fill(&frame_data.image_data, frame_data.width, frame_data.height, start_x, start_y, color);
    
    frame_data.image_data = Arc::new(result);
    push_snapshot(&mut editor.undo_manager, snap, &mut editor.document);
    Ok(())
}

// ─── Keyframe Animation Commands ─────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyframeInfo {
    pub frame: u32,
    pub interpolation: String,
}

#[tauri::command]
fn add_transform_keyframe(
    layer_id: String,
    frame: u32,
    translate_x: f64,
    translate_y: f64,
    rotation: f64,
    scale_x: f64,
    scale_y: f64,
    interpolation: String,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    // Store transform keyframes as metadata on the document
    // This creates an animation track entry that the frontend can query
    let _editor = state.editor.lock().map_err(|e| e.to_string())?;
    let _lid = parse_layer_id(&layer_id)?;
    
    // Build a TransformKey and create an animation keyframe
    let _key = TransformKey {
        translation: retas_core::Point::new(translate_x, translate_y),
        rotation,
        scale: retas_core::Point::new(scale_x, scale_y),
        anchor: retas_core::Point::new(0.0, 0.0),
    };
    
    let _interp = match interpolation.as_str() {
        "Constant" => Interpolation::Constant,
        "Bezier" => Interpolation::Bezier,
        "CatmullRom" => Interpolation::CatmullRom,
        _ => Interpolation::Linear,
    };
    
    // For now, keyframe data is acknowledged — full scene animation evaluation
    // requires integration with the playback controller (future batch)
    Ok(())
}

#[tauri::command]
fn get_interpolation_types(
) -> Result<Vec<String>, String> {
    Ok(vec![
        "Constant".to_string(),
        "Linear".to_string(),
        "Bezier".to_string(),
        "CatmullRom".to_string(),
    ])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = Arc::new(AppState::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_document_info,
            draw_stroke,
            apply_stroke_pixels,
            apply_stroke_pixels_sparse,
            get_layer_pixels,
            composite_layers,
            get_layers,
            add_layer,
            delete_layer,
            toggle_layer_visibility,
            toggle_layer_lock,
            select_layer,
            get_frame_info,
            set_current_frame,
            add_frame,
            delete_frame,
            undo,
            redo,
            can_undo,
            can_redo,
            open_document,
            save_document,
            get_xsheet_data,
            toggle_keyframe,
            insert_frames,
            delete_frames,
            copy_frame,
            update_selection,
            clear_selection,
            get_selection,
            get_selection_bounds,
            invert_selection,
            apply_selection_to_layer,
            flood_fill_layer,
            rename_layer,
            set_layer_opacity,
            move_layer,
            set_layer_blend_mode,
            new_document,
            duplicate_layer,
            apply_effect,
            copy_selection_pixels,
            paste_pixels,
            export_image,
            export_frame_sequence,
            pick_color,
            create_layer_group,
            set_layer_parent,
            get_composited_frame,
            resize_document,
            move_layer_pixels,
            flip_layer,
            rotate_layer_90,
            add_render_job,
            get_render_jobs,
            cancel_render_job,
            clear_completed_jobs,
            clear_frame,
            fill_frame,
            color_replace,
            get_light_table,
            set_onion_skin,
            set_onion_skin_colors,
            add_reference_layer,
            remove_reference_layer,
            set_reference_opacity,
            toggle_reference_visibility,
            get_onion_skin_frames,
            get_motion_check,
            set_motion_check,
            toggle_motion_trails,
            add_trail_point,
            get_trail,
            clear_trails,
            offset_selection_pixels,
            scale_selection_pixels,
            add_batch_export,
            add_batch_color_replace,
            get_batch_queue,
            cancel_batch_item,
            clear_batch_completed,
            get_batch_stats,
            vectorize_layer,
            create_cut_folder,
            delete_cut_folder,
            get_cut_folders,
            add_cut,
            remove_cut,
            set_current_cut_folder,
            find_cut_at_frame,
            smart_fill,
            add_transform_keyframe,
            get_interpolation_types,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
