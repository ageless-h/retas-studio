use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::any::Any;
use crate::{Document, LayerId, Layer, Color8, Point, Rect};

pub trait Command: std::fmt::Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn execute(&mut self, document: &mut Document);
    fn undo(&mut self, document: &mut Document);
    fn redo(&mut self, document: &mut Document) {
        self.execute(document);
    }
    fn description(&self) -> &str;
    fn merge(&mut self, _other: &dyn Command) -> bool {
        false
    }
    fn can_merge(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoState {
    pub description: String,
    pub timestamp: f64,
    pub is_merged: bool,
}

pub struct UndoManager {
    undo_stack: VecDeque<Box<dyn Command>>,
    redo_stack: VecDeque<Box<dyn Command>>,
    max_undo_levels: usize,
    last_command_time: f64,
    merge_time_window: f64,
}

impl UndoManager {
    pub fn new() -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            max_undo_levels: 100,
            last_command_time: 0.0,
            merge_time_window: 0.5,
        }
    }

    pub fn with_max_levels(mut self, max: usize) -> Self {
        self.max_undo_levels = max;
        self
    }

    pub fn execute(&mut self, mut command: Box<dyn Command>, document: &mut Document) {
        command.execute(document);
        
        let current_time = Self::current_time();
        
        if let Some(last) = self.undo_stack.back_mut() {
            let can_merge = last.can_merge() 
                && command.can_merge()
                && (current_time - self.last_command_time) < self.merge_time_window;
            
            if can_merge && last.merge(command.as_ref()) {
                self.last_command_time = current_time;
                return;
            }
        }

        self.undo_stack.push_back(command);
        self.redo_stack.clear();
        self.last_command_time = current_time;

        while self.undo_stack.len() > self.max_undo_levels {
            self.undo_stack.pop_front();
        }
    }

    pub fn undo(&mut self, document: &mut Document) -> Option<String> {
        if let Some(mut command) = self.undo_stack.pop_back() {
            let description = command.description().to_string();
            command.undo(document);
            self.redo_stack.push_back(command);
            Some(description)
        } else {
            None
        }
    }

    pub fn redo(&mut self, document: &mut Document) -> Option<String> {
        if let Some(mut command) = self.redo_stack.pop_back() {
            let description = command.description().to_string();
            command.redo(document);
            self.undo_stack.push_back(command);
            Some(description)
        } else {
            None
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    pub fn get_undo_history(&self) -> Vec<UndoState> {
        self.undo_stack
            .iter()
            .rev()
            .map(|c| UndoState {
                description: c.description().to_string(),
                timestamp: 0.0,
                is_merged: false,
            })
            .collect()
    }

    pub fn get_redo_history(&self) -> Vec<UndoState> {
        self.redo_stack
            .iter()
            .rev()
            .map(|c| UndoState {
                description: c.description().to_string(),
                timestamp: 0.0,
                is_merged: false,
            })
            .collect()
    }

    fn current_time() -> f64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
    }
}

impl Default for UndoManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrokeCommand {
    pub layer_id: LayerId,
    pub stroke_id: u64,
    pub stroke_data: Vec<u8>,
    pub previous_pixel_data: Vec<u8>,
    pub bounds: (u32, u32, u32, u32),
    pub blend_mode: crate::layer::BlendMode,
    pub opacity: f64,
    pub description: String,
}

impl Command for StrokeCommand {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn execute(&mut self, document: &mut Document) {
        if let Some(layer) = document.layers.get_mut(&self.layer_id) {
            if let crate::Layer::Raster(raster) = layer {
                if let Some(frame) = raster.frames.get_mut(&raster.current_frame) {
                    if self.previous_pixel_data.is_empty() {
                        self.previous_pixel_data = frame.image_data.clone();
                    }
                    if !self.stroke_data.is_empty() {
                        frame.image_data = self.stroke_data.clone();
                    }
                }
            }
        }
    }

    fn undo(&mut self, document: &mut Document) {
        if let Some(layer) = document.layers.get_mut(&self.layer_id) {
            if let crate::Layer::Raster(raster) = layer {
                if let Some(frame) = raster.frames.get_mut(&raster.current_frame) {
                    frame.image_data = self.previous_pixel_data.clone();
                }
            }
        }
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn can_merge(&self) -> bool {
        true
    }

    fn merge(&mut self, other: &dyn Command) -> bool {
        if let Some(other_stroke) = other.as_any().downcast_ref::<StrokeCommand>() {
            if self.layer_id == other_stroke.layer_id {
                self.stroke_data.extend_from_slice(&other_stroke.stroke_data);
                self.description = "Brush Stroke (merged)".to_string();
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformCommand {
    pub layer_id: LayerId,
    pub old_offset: (f64, f64),
    pub new_offset: (f64, f64),
    pub description: String,
}

impl Command for TransformCommand {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn execute(&mut self, document: &mut Document) {
        if let Some(layer) = document.layers.get_mut(&self.layer_id) {
            match layer {
                crate::Layer::Raster(raster) => {
                    raster.offset = crate::Point::new(self.new_offset.0, self.new_offset.1);
                }
                crate::Layer::Camera(camera) => {
                    camera.position = crate::Point::new(self.new_offset.0, self.new_offset.1);
                }
                crate::Layer::Text(text) => {
                    text.position = crate::Point::new(self.new_offset.0, self.new_offset.1);
                }
                _ => {}
            }
        }
    }

    fn undo(&mut self, document: &mut Document) {
        if let Some(layer) = document.layers.get_mut(&self.layer_id) {
            match layer {
                crate::Layer::Raster(raster) => {
                    raster.offset = crate::Point::new(self.old_offset.0, self.old_offset.1);
                }
                crate::Layer::Camera(camera) => {
                    camera.position = crate::Point::new(self.old_offset.0, self.old_offset.1);
                }
                crate::Layer::Text(text) => {
                    text.position = crate::Point::new(self.old_offset.0, self.old_offset.1);
                }
                _ => {}
            }
        }
    }

    fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerAddCommand {
    pub layer: Layer,
    pub index: usize,
    pub description: String,
}

impl Command for LayerAddCommand {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn execute(&mut self, document: &mut Document) {
        let layer_id = self.layer.base().id;
        document.layers.insert(layer_id, self.layer.clone());
        if self.index < document.timeline.layer_order.len() {
            document.timeline.layer_order.insert(self.index, layer_id);
        } else {
            document.timeline.layer_order.push(layer_id);
        }
    }

    fn undo(&mut self, document: &mut Document) {
        let layer_id = self.layer.base().id;
        document.layers.remove(&layer_id);
        document.timeline.layer_order.retain(|id| *id != layer_id);
    }

    fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerDeleteCommand {
    pub layer: Layer,
    pub index: usize,
    pub description: String,
}

impl Command for LayerDeleteCommand {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn execute(&mut self, document: &mut Document) {
        let layer_id = self.layer.base().id;
        document.layers.remove(&layer_id);
        document.timeline.layer_order.retain(|id| *id != layer_id);
    }

    fn undo(&mut self, document: &mut Document) {
        let layer_id = self.layer.base().id;
        document.layers.insert(layer_id, self.layer.clone());
        if self.index < document.timeline.layer_order.len() {
            document.timeline.layer_order.insert(self.index, layer_id);
        } else {
            document.timeline.layer_order.push(layer_id);
        }
    }

    fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerMoveCommand {
    pub layer_id: LayerId,
    pub old_index: usize,
    pub new_index: usize,
    pub description: String,
}

impl Command for LayerMoveCommand {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn execute(&mut self, document: &mut Document) {
        if self.old_index < document.timeline.layer_order.len() {
            let layer_id = document.timeline.layer_order.remove(self.old_index);
            let insert_pos = if self.new_index > self.old_index {
                (self.new_index - 1).min(document.timeline.layer_order.len())
            } else {
                self.new_index
            };
            document.timeline.layer_order.insert(insert_pos, layer_id);
        }
    }

    fn undo(&mut self, document: &mut Document) {
        let new_pos = if self.new_index > self.old_index {
            (self.new_index - 1).min(document.timeline.layer_order.len().saturating_sub(1))
        } else {
            self.new_index
        };
        if new_pos < document.timeline.layer_order.len() {
            let layer_id = document.timeline.layer_order.remove(new_pos);
            document.timeline.layer_order.insert(self.old_index, layer_id);
        }
    }

    fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerPropertyCommand {
    pub layer_id: LayerId,
    pub property_name: String,
    pub old_value: PropertyValue,
    pub new_value: PropertyValue,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropertyValue {
    Bool(bool),
    F64(f64),
    String(String),
    BlendMode(crate::layer::BlendMode),
}

impl Command for LayerPropertyCommand {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn execute(&mut self, document: &mut Document) {
        if let Some(layer) = document.layers.get_mut(&self.layer_id) {
            let base = layer.base_mut();
            match (&self.property_name as &str, &self.new_value) {
                ("visible", PropertyValue::Bool(v)) => base.visible = *v,
                ("locked", PropertyValue::Bool(v)) => base.locked = *v,
                ("opacity", PropertyValue::F64(v)) => base.opacity = *v,
                ("name", PropertyValue::String(v)) => base.name = v.clone(),
                ("blend_mode", PropertyValue::BlendMode(v)) => base.blend_mode = *v,
                _ => {}
            }
        }
    }

    fn undo(&mut self, document: &mut Document) {
        if let Some(layer) = document.layers.get_mut(&self.layer_id) {
            let base = layer.base_mut();
            match (&self.property_name as &str, &self.old_value) {
                ("visible", PropertyValue::Bool(v)) => base.visible = *v,
                ("locked", PropertyValue::Bool(v)) => base.locked = *v,
                ("opacity", PropertyValue::F64(v)) => base.opacity = *v,
                ("name", PropertyValue::String(v)) => base.name = v.clone(),
                ("blend_mode", PropertyValue::BlendMode(v)) => base.blend_mode = *v,
                _ => {}
            }
        }
    }

    fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionCommand {
    pub old_selection: Option<SelectionData>,
    pub new_selection: Option<SelectionData>,
    pub description: String,
}

impl Command for SelectionCommand {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn execute(&mut self, document: &mut Document) {
        document.selection = self.new_selection.clone();
    }

    fn undo(&mut self, document: &mut Document) {
        document.selection = self.old_selection.clone();
    }

    fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillCommand {
    pub layer_id: LayerId,
    pub selection: Option<SelectionData>,
    pub old_pixel_data: Vec<u8>,
    pub fill_color: Color8,
    pub tolerance: f64,
    pub description: String,
}

impl Command for FillCommand {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn execute(&mut self, document: &mut Document) {
        if let Some(layer) = document.layers.get_mut(&self.layer_id) {
            if let crate::Layer::Raster(raster) = layer {
                if let Some(frame) = raster.frames.get_mut(&raster.current_frame) {
                    if self.old_pixel_data.is_empty() {
                        self.old_pixel_data = frame.image_data.clone();
                    }
                    let color = self.fill_color;
                    for i in (0..frame.image_data.len()).step_by(4) {
                        if i + 3 < frame.image_data.len() {
                            frame.image_data[i] = color.r;
                            frame.image_data[i + 1] = color.g;
                            frame.image_data[i + 2] = color.b;
                            frame.image_data[i + 3] = color.a;
                        }
                    }
                }
            }
        }
    }

    fn undo(&mut self, document: &mut Document) {
        if let Some(layer) = document.layers.get_mut(&self.layer_id) {
            if let crate::Layer::Raster(raster) = layer {
                if let Some(frame) = raster.frames.get_mut(&raster.current_frame) {
                    frame.image_data = self.old_pixel_data.clone();
                }
            }
        }
    }

    fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameCommand {
    pub layer_id: LayerId,
    pub frame_number: u32,
    pub old_frame_data: Option<Vec<u8>>,
    pub new_frame_data: Option<Vec<u8>>,
    pub description: String,
}

impl Command for FrameCommand {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn execute(&mut self, document: &mut Document) {
        if let Some(layer) = document.layers.get_mut(&self.layer_id) {
            if let crate::Layer::Raster(raster) = layer {
                if let Some(frame) = raster.frames.get_mut(&self.frame_number) {
                    if self.old_frame_data.is_none() {
                        self.old_frame_data = Some(frame.image_data.clone());
                    }
                    if let Some(ref data) = self.new_frame_data {
                        frame.image_data = data.clone();
                    }
                }
                raster.current_frame = self.frame_number;
            }
        }
    }

    fn undo(&mut self, document: &mut Document) {
        if let Some(layer) = document.layers.get_mut(&self.layer_id) {
            if let crate::Layer::Raster(raster) = layer {
                if let Some(frame) = raster.frames.get_mut(&self.frame_number) {
                    if let Some(ref data) = self.old_frame_data {
                        frame.image_data = data.clone();
                    }
                }
                raster.current_frame = self.frame_number;
            }
        }
    }

    fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionData {
    pub selection_type: SelectionType,
    pub points: Vec<Point>,
    pub bounds: Rect,
    pub feather: f64,
    pub anti_aliased: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SelectionType {
    None,
    Rectangular,
    Elliptical,
    Lasso,
    Polygon,
    MagicWand,
}
