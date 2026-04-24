use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

mod state;
use state::AppState;

use retas_core::Layer as RetasLayer;
use retas_core::advanced::selection::{Selection, SelectionMask, SelectionTool, SelectionMode as RetasSelectionMode};

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

fn record_history(state: &State<Arc<AppState>>) -> Result<(), String> {
    let doc = state.document.lock().map_err(|e| e.to_string())?;
    let snapshot = doc.clone();
    drop(doc);
    state.history.lock().map_err(|e| e.to_string())?.record(&snapshot);
    Ok(())
}

#[tauri::command]
fn get_document_info(state: State<Arc<AppState>>) -> Result<DocumentInfo, String> {
    let doc = state.document.lock().map_err(|e| e.to_string())?;
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
    let doc = state.document.lock().map_err(|e| e.to_string())?;
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
    record_history(&state)?;
    let mut doc = state.document.lock().map_err(|e| e.to_string())?;
    let layer = RetasLayer::Raster(retas_core::RasterLayer::new(&name));
    let id = layer.id();
    let base = layer.base().clone();

    doc.layers.insert(id, layer);
    doc.timeline.layer_order.push(id);

    Ok(LayerInfo {
        id: base.id.0.to_string(),
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
    record_history(&state)?;
    let mut doc = state.document.lock().map_err(|e| e.to_string())?;
    doc.remove_layer(layer_id);
    Ok(())
}

#[tauri::command]
fn toggle_layer_visibility(id: String, state: State<Arc<AppState>>) -> Result<bool, String> {
    let layer_id = parse_layer_id(&id)?;
    record_history(&state)?;
    let mut doc = state.document.lock().map_err(|e| e.to_string())?;
    if let Some(layer) = doc.layers.get_mut(&layer_id) {
        layer.base_mut().visible = !layer.base().visible;
        Ok(layer.base().visible)
    } else {
        Err("Layer not found".to_string())
    }
}

#[tauri::command]
fn toggle_layer_lock(id: String, state: State<Arc<AppState>>) -> Result<bool, String> {
    let layer_id = parse_layer_id(&id)?;
    record_history(&state)?;
    let mut doc = state.document.lock().map_err(|e| e.to_string())?;
    if let Some(layer) = doc.layers.get_mut(&layer_id) {
        layer.base_mut().locked = !layer.base().locked;
        Ok(layer.base().locked)
    } else {
        Err("Layer not found".to_string())
    }
}

#[tauri::command]
fn get_frame_info(state: State<Arc<AppState>>) -> Result<FrameInfo, String> {
    let doc = state.document.lock().map_err(|e| e.to_string())?;
    Ok(FrameInfo {
        current: doc.timeline.current_frame,
        total: doc.timeline.end_frame,
        fps: doc.timeline.frame_rate,
    })
}

#[tauri::command]
fn set_current_frame(frame: u32, state: State<Arc<AppState>>) -> Result<(), String> {
    let mut doc = state.document.lock().map_err(|e| e.to_string())?;
    doc.timeline.current_frame = frame;
    Ok(())
}

#[tauri::command]
fn add_frame(state: State<Arc<AppState>>) -> Result<(), String> {
    record_history(&state)?;
    let mut doc = state.document.lock().map_err(|e| e.to_string())?;
    let new_frame = doc.timeline.end_frame;
    let width = doc.settings.resolution.width as u32;
    let height = doc.settings.resolution.height as u32;

    for layer in doc.layers.values_mut() {
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

    doc.timeline.end_frame += 1;
    doc.settings.total_frames += 1;
    Ok(())
}

#[tauri::command]
fn delete_frame(state: State<Arc<AppState>>) -> Result<(), String> {
    record_history(&state)?;
    let mut doc = state.document.lock().map_err(|e| e.to_string())?;
    if doc.timeline.end_frame > 1 {
        let removed_frame = doc.timeline.end_frame - 1;
        for layer in doc.layers.values_mut() {
            if let retas_core::Layer::Raster(raster) = layer {
                raster.frames.remove(&removed_frame);
            }
        }
        doc.timeline.end_frame -= 1;
        doc.settings.total_frames -= 1;
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
    record_history(&state)?;
    let mut doc = state.document.lock().map_err(|e| e.to_string())?;

    let layer_id = if command.layer_id == "current" {
        doc.selected_layers.first().copied()
    } else {
        parse_layer_id(&command.layer_id).ok()
    };

    let layer_id = layer_id.ok_or("No selected layer")?;
    let current_frame = doc.timeline.current_frame;

    let layer = doc.layers.get_mut(&layer_id).ok_or("Layer not found")?;
    let raster = match layer {
        retas_core::Layer::Raster(r) => r,
        _ => return Err("Only raster layers support drawing".to_string()),
    };

    let frame = raster.frames.get_mut(&current_frame).ok_or("No frame data")?;
    let is_eraser = command.tool == "eraser";
    let frame_width = frame.width;
    let frame_height = frame.height;

    for window in command.points.windows(2) {
        let (x0, y0) = window[0];
        let (x1, y1) = window[1];
        draw_line_on_pixels(
            frame.get_image_data_mut(),
            frame_width,
            frame_height,
            x0, y0, x1, y1,
            command.color,
            command.size,
            is_eraser,
        );
    }

    Ok(format!("绘制了 {} 个点", command.points.len()))
}

#[tauri::command]
fn apply_stroke_pixels(
    stroke_pixels: Vec<u8>,
    state: State<Arc<AppState>>,
) -> Result<String, String> {
    record_history(&state)?;
    let mut doc = state.document.lock().map_err(|e| e.to_string())?;

    let layer_id = doc.selected_layers.first().copied().ok_or("No selected layer")?;
    let current_frame = doc.timeline.current_frame;

    let layer = doc.layers.get_mut(&layer_id).ok_or("Layer not found")?;
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

    Ok("笔划已应用".to_string())
}

#[tauri::command]
fn get_layer_pixels(layer_id: String, state: State<Arc<AppState>>) -> Result<Vec<u8>, String> {
    let doc = state.document.lock().map_err(|e| e.to_string())?;
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
    let doc = state.document.lock().map_err(|e| e.to_string())?;

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
    let snapshot = {
        let doc = state.document.lock().map_err(|e| e.to_string())?;
        let current = doc.clone();
        let mut history = state.history.lock().map_err(|e| e.to_string())?;
        history.undo(&current)
    };
    if let Some(doc) = snapshot {
        *state.document.lock().map_err(|e| e.to_string())? = doc;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
fn redo(state: State<Arc<AppState>>) -> Result<bool, String> {
    let snapshot = {
        let doc = state.document.lock().map_err(|e| e.to_string())?;
        let current = doc.clone();
        let mut history = state.history.lock().map_err(|e| e.to_string())?;
        history.redo(&current)
    };
    if let Some(doc) = snapshot {
        *state.document.lock().map_err(|e| e.to_string())? = doc;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
fn can_undo(state: State<Arc<AppState>>) -> Result<bool, String> {
    Ok(state.history.lock().map_err(|e| e.to_string())?.can_undo())
}

#[tauri::command]
fn can_redo(state: State<Arc<AppState>>) -> Result<bool, String> {
    Ok(state.history.lock().map_err(|e| e.to_string())?.can_redo())
}

#[tauri::command]
fn open_document(path: String, state: State<Arc<AppState>>) -> Result<DocumentInfo, String> {
    println!("打开文档: {}", path);
    let data = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let new_doc: retas_core::Document = serde_json::from_str(&data).map_err(|e| e.to_string())?;

    {
        let mut history = state.history.lock().map_err(|e| e.to_string())?;
        history.clear();
    }

    let mut doc = state.document.lock().map_err(|e| e.to_string())?;
    *doc = new_doc;

    Ok(DocumentInfo {
        name: doc.settings.name.clone(),
        width: doc.settings.resolution.width,
        height: doc.settings.resolution.height,
        frame_rate: doc.settings.frame_rate,
        total_frames: doc.settings.total_frames,
    })
}

#[tauri::command]
fn save_document(path: String, state: State<Arc<AppState>>) -> Result<(), String> {
    println!("保存文档: {}", path);
    let doc = state.document.lock().map_err(|e| e.to_string())?;
    let data = serde_json::to_string_pretty(&*doc).map_err(|e| e.to_string())?;
    drop(doc);
    std::fs::write(&path, data).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_xsheet_data(state: State<Arc<AppState>>) -> Result<Vec<XSheetCell>, String> {
    let doc = state.document.lock().map_err(|e| e.to_string())?;
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
    record_history(&state)?;
    let mut doc = state.document.lock().map_err(|e| e.to_string())?;
    
    let layer = doc.layers.get_mut(&layer_id)
        .ok_or_else(|| "Layer not found".to_string())?;
    
    layer.toggle_keyframe(frame);
    Ok(())
}

#[tauri::command]
fn insert_frames(at_frame: u32, count: u32, state: State<Arc<AppState>>) -> Result<(), String> {
    record_history(&state)?;
    let mut doc = state.document.lock().map_err(|e| e.to_string())?;
    doc.insert_frames(at_frame, count);
    Ok(())
}

#[tauri::command]
fn delete_frames(at_frame: u32, count: u32, state: State<Arc<AppState>>) -> Result<(), String> {
    record_history(&state)?;
    let mut doc = state.document.lock().map_err(|e| e.to_string())?;
    doc.delete_frames(at_frame, count);
    Ok(())
}

#[tauri::command]
fn copy_frame(layer_id: String, from_frame: u32, to_frame: u32, state: State<Arc<AppState>>) -> Result<(), String> {
    let layer_id = parse_layer_id(&layer_id)?;
    record_history(&state)?;
    let mut doc = state.document.lock().map_err(|e| e.to_string())?;
    doc.copy_frame(layer_id, from_frame, to_frame)
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
    let mut doc = state.document.lock().map_err(|e| e.to_string())?;
    doc.selection = None;
    Ok(())
}

#[tauri::command]
fn get_selection(state: State<Arc<AppState>>) -> Result<Option<SelectionDataFrontend>, String> {
    let doc = state.document.lock().map_err(|e| e.to_string())?;
    
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
    let doc = state.document.lock().map_err(|e| e.to_string())?;
    
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
    let mut doc = state.document.lock().map_err(|e| e.to_string())?;
    
    if let Some(sel) = &doc.selection {
        let width = doc.settings.resolution.width as u32;
        let height = doc.settings.resolution.height as u32;
        doc.selection = Some(sel.invert(width, height));
    }
    Ok(())
}

#[tauri::command]
fn update_selection(selection: SelectionDataFrontend, state: State<Arc<AppState>>) -> Result<(), String> {
    record_history(&state)?;
    let mut doc = state.document.lock().map_err(|e| e.to_string())?;
    
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
        doc.selection = Some(new_selection);
    } else if let Some(existing) = &doc.selection {
        doc.selection = Some(existing.combine(&new_selection, mode));
    } else {
        doc.selection = Some(new_selection);
    }
    
    Ok(())
}

#[tauri::command]
fn apply_selection_to_layer(layer_id: String, operation: String, state: State<Arc<AppState>>) -> Result<(), String> {
    let layer_id = parse_layer_id(&layer_id)?;
    record_history(&state)?;
    let mut doc = state.document.lock().map_err(|e| e.to_string())?;
    
    let selection = doc.selection.clone().ok_or("No active selection")?;
    if !selection.is_active {
        return Err("No active selection".to_string());
    }
    
    let current_frame = doc.timeline.current_frame;
    let layer = doc.layers.get_mut(&layer_id).ok_or("Layer not found")?;
    let raster = match layer {
        retas_core::Layer::Raster(r) => r,
        _ => return Err("Only raster layers support selection operations".to_string()),
    };
    
    let frame = raster.frames.get_mut(&current_frame).ok_or("No frame data")?;
    let _width = frame.width;
    let _height = frame.height;
    let pixels = frame.get_image_data_mut();
    
    let bitmap = selection.to_bitmap();
    let mask_data = match &bitmap {
        SelectionMask::Bitmap { data, .. } => data.clone(),
        _ => return Err("Failed to convert selection to bitmap".to_string()),
    };
    
    match operation.as_str() {
        "clear" => {
            for (i, &alpha) in mask_data.iter().enumerate() {
                if alpha > 0 {
                    let idx = i * 4;
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
            for (i, &alpha) in mask_data.iter().enumerate() {
                if alpha > 0 {
                    let idx = i * 4;
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
            for (i, &alpha) in mask_data.iter().enumerate() {
                if alpha > 0 {
                    let idx = i * 4;
                    if idx + 3 < pixels.len() {
                        let current_alpha = pixels[idx + 3];
                        pixels[idx + 3] = 255 - current_alpha;
                    }
                }
            }
        }
        _ => return Err(format!("Unknown operation: {}", operation)),
    }
    
    Ok(())
}

#[tauri::command]
fn flood_fill_layer(x: u32, y: u32, color: (u8, u8, u8, u8), tolerance: f64, state: State<Arc<AppState>>) -> Result<(), String> {
    record_history(&state)?;
    let mut doc = state.document.lock().map_err(|e| e.to_string())?;
    
    let layer_id = doc.selected_layers.first().copied().ok_or("No selected layer")?;
    let current_frame = doc.timeline.current_frame;
    
    let layer = doc.layers.get_mut(&layer_id).ok_or("Layer not found")?;
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
    Ok(())
}

#[tauri::command]
fn select_layer(id: String, state: State<Arc<AppState>>) -> Result<(), String> {
    let layer_id = parse_layer_id(&id)?;
    let mut doc = state.document.lock().map_err(|e| e.to_string())?;
    
    if doc.layers.contains_key(&layer_id) {
        doc.selected_layers.clear();
        doc.selected_layers.push(layer_id);
        Ok(())
    } else {
        Err("Layer not found".to_string())
    }
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
