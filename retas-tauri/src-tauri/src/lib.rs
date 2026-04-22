use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

mod state;
use state::AppState;

use retas_core::Layer as RetasLayer;

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
    record_history(&state)?;
    let mut doc = state.document.lock().map_err(|e| e.to_string())?;
    doc.timeline.current_frame = frame;
    Ok(())
}

#[tauri::command]
fn add_frame(state: State<Arc<AppState>>) -> Result<(), String> {
    record_history(&state)?;
    let mut doc = state.document.lock().map_err(|e| e.to_string())?;
    doc.timeline.end_frame += 1;
    doc.settings.total_frames += 1;
    Ok(())
}

#[tauri::command]
fn delete_frame(state: State<Arc<AppState>>) -> Result<(), String> {
    record_history(&state)?;
    let mut doc = state.document.lock().map_err(|e| e.to_string())?;
    if doc.timeline.end_frame > 1 {
        doc.timeline.end_frame -= 1;
        doc.settings.total_frames -= 1;
    }
    Ok(())
}

#[tauri::command]
fn draw_stroke(command: DrawCommand, state: State<Arc<AppState>>) -> Result<String, String> {
    record_history(&state)?;
    println!("绘制: {:?}", command);
    Ok(format!("绘制了 {} 个点", command.points.len()))
}

#[tauri::command]
fn get_layer_pixels(_layer_id: String, _state: State<Arc<AppState>>) -> Result<Vec<u8>, String> {
    Ok(vec![255; 1920 * 1080 * 4])
}

#[tauri::command]
fn composite_layers(_state: State<Arc<AppState>>) -> Result<Vec<u8>, String> {
    Ok(vec![255; 1920 * 1080 * 4])
}

#[tauri::command]
fn undo(state: State<Arc<AppState>>) -> Result<bool, String> {
    let current = state.document.lock().map_err(|e| e.to_string())?.clone();
    let prev = state.history.lock().map_err(|e| e.to_string())?.undo(&current);
    if let Some(doc) = prev {
        *state.document.lock().map_err(|e| e.to_string())? = doc;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
fn redo(state: State<Arc<AppState>>) -> Result<bool, String> {
    let current = state.document.lock().map_err(|e| e.to_string())?.clone();
    let next = state.history.lock().map_err(|e| e.to_string())?.redo(&current);
    if let Some(doc) = next {
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = Arc::new(AppState::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_document_info,
            draw_stroke,
            get_layer_pixels,
            composite_layers,
            get_layers,
            add_layer,
            delete_layer,
            toggle_layer_visibility,
            toggle_layer_lock,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
