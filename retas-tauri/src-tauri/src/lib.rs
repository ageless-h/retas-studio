use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

mod state;
use state::AppState;

use retas_core::Layer as RetasLayer;
use retas_core::advanced::selection::{Selection, SelectionMask, SelectionTool, SelectionMode as RetasSelectionMode};
use retas_core::advanced::undo::{UndoManager, Command, LayerAddCommand, LayerDeleteCommand, SnapshotCommand};
use retas_core::advanced::brush::{BrushEngine, BrushSettings, BrushPoint, BrushType, BrushBlendMode};
use retas_io::export::{ImageExporter, ImageExportOptions, ImageFormat};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LayerInfo {
    pub id: String,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f64,
    pub layer_type: String,
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
            export_image,
            export_frame_sequence,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
