use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{Layer, LayerId, Rect, Size};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub frame_rate: f64,
    pub start_frame: u32,
    pub end_frame: u32,
    pub current_frame: u32,
    pub layer_order: Vec<LayerId>,
    pub markers: HashMap<u32, String>,
}

impl Timeline {
    pub fn new() -> Self {
        Self {
            frame_rate: 24.0,
            start_frame: 0,
            end_frame: 100,
            current_frame: 0,
            layer_order: Vec::new(),
            markers: HashMap::new(),
        }
    }

    pub fn duration_seconds(&self) -> f64 {
        (self.end_frame - self.start_frame) as f64 / self.frame_rate
    }

    pub fn frame_to_time(&self, frame: u32) -> f64 {
        frame as f64 / self.frame_rate
    }

    pub fn time_to_frame(&self, time: f64) -> u32 {
        (time * self.frame_rate) as u32
    }
}

impl Default for Timeline {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSettings {
    pub name: String,
    pub resolution: Size,
    pub background_color: crate::Color8,
    pub frame_rate: f64,
    pub total_frames: u32,
}

impl DocumentSettings {
    pub fn new(name: impl Into<String>, width: f64, height: f64) -> Self {
        Self {
            name: name.into(),
            resolution: Size::new(width, height),
            background_color: crate::Color8::WHITE,
            frame_rate: 24.0,
            total_frames: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub settings: DocumentSettings,
    pub layers: HashMap<LayerId, Layer>,
    pub timeline: Timeline,
    pub selected_layers: Vec<LayerId>,
    pub modified: bool,
    pub selection: Option<crate::advanced::undo::SelectionData>,
}

impl Document {
    pub fn new(name: impl Into<String>, width: f64, height: f64) -> Self {
        Self {
            settings: DocumentSettings::new(name, width, height),
            layers: HashMap::new(),
            timeline: Timeline::new(),
            selected_layers: Vec::new(),
            modified: false,
            selection: None,
        }
    }

    pub fn add_layer(&mut self, layer: Layer) -> LayerId {
        let id = layer.id();
        self.timeline.layer_order.push(id);
        self.layers.insert(id, layer);
        self.modified = true;
        id
    }

    pub fn remove_layer(&mut self, id: LayerId) -> Option<Layer> {
        self.timeline.layer_order.retain(|&l| l != id);
        self.selected_layers.retain(|&l| l != id);
        self.modified = true;
        self.layers.remove(&id)
    }

    pub fn get_layer(&self, id: LayerId) -> Option<&Layer> {
        self.layers.get(&id)
    }

    pub fn get_layer_mut(&mut self, id: LayerId) -> Option<&mut Layer> {
        self.modified = true;
        self.layers.get_mut(&id)
    }

    pub fn move_layer(&mut self, id: LayerId, new_index: usize) {
        if let Some(current_index) = self.timeline.layer_order.iter().position(|&l| l == id) {
            self.timeline.layer_order.remove(current_index);
            let insert_index = new_index.min(self.timeline.layer_order.len());
            self.timeline.layer_order.insert(insert_index, id);
            self.modified = true;
        }
    }

    pub fn bounds(&self) -> Rect {
        Rect::new(
            0.0,
            0.0,
            self.settings.resolution.width,
            self.settings.resolution.height,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub documents: Vec<Document>,
    pub active_document: Option<usize>,
    pub file_path: Option<String>,
}

impl Project {
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
            active_document: None,
            file_path: None,
        }
    }

    pub fn add_document(&mut self, document: Document) -> usize {
        self.documents.push(document);
        self.active_document = Some(self.documents.len() - 1);
        self.documents.len() - 1
    }

    pub fn active_document(&self) -> Option<&Document> {
        self.active_document.and_then(|i| self.documents.get(i))
    }

    pub fn active_document_mut(&mut self) -> Option<&mut Document> {
        self.active_document.and_then(|i| self.documents.get_mut(i))
    }
}

impl Default for Project {
    fn default() -> Self {
        Self::new()
    }
}
