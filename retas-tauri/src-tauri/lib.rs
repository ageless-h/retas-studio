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
                id: format!("{:?}", base.id),
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
    let mut doc = state.document.lock().map_err(|e| e.to_string())?;
    let layer = RetasLayer::Raster(retas_core::RasterLayer::new(&name));
    let id = layer.id();
    let base = layer.base().clone();
    
    doc.layers.insert(id, layer);
    doc.timeline.layer_order.push(id);
    
    Ok(LayerInfo {
        id: format!("{:?}", id),
        name: base.name,
        visible: base.visible,
        locked: base.locked,
        opacity: base.opacity,
        layer_type: format!("{:?}", base.layer_type),
    })
}

#[tauri::command]
fn delete_layer(_id: String, _state: State<Arc<AppState>>) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
fn toggle_layer_visibility(_id: String, _state: State<Arc<AppState>>) -> Result<bool, String> {
    Ok(true)
}

#[tauri::command]
fn toggle_layer_lock(_id: String, _state: State<Arc<AppState>>) -> Result<bool, String> {
    Ok(false)
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
    let mut doc = state.document.lock().map_err(|e| e.to_string())?;
    doc.timeline.end_frame += 1;
    doc.settings.total_frames += 1;
    Ok(())
}

#[tauri::command]
fn delete_frame(state: State<Arc<AppState>>) -> Result<(), String> {
    let mut doc = state.document.lock().map_err(|e| e.to_string())?;
    if doc.timeline.end_frame > 1 {
        doc.timeline.end_frame -= 1;
        doc.settings.total_frames -= 1;
    }
    Ok(())
}

#[tauri::command]
fn draw_stroke(command: DrawCommand, _state: State<Arc<AppState>>) -> Result<String, String> {
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = Arc::new(AppState::new());
    
    tauri::Builder::default()
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
            delete_frame
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
