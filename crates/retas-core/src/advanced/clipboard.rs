use serde::{Deserialize, Serialize};
use crate::{LayerId, Point, Rect, Color8};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardData {
    pub data_type: ClipboardDataType,
    pub layers: Vec<ClipboardLayer>,
    pub bounds: Rect,
    pub source_document: Option<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClipboardDataType {
    Empty,
    Raster,
    Vector,
    Text,
    Mixed,
    FileReference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardLayer {
    pub original_id: LayerId,
    pub name: String,
    pub layer_type: ClipboardLayerType,
    pub data: Vec<u8>,
    pub offset: Point,
    pub opacity: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClipboardLayerType {
    Raster {
        width: u32,
        height: u32,
        has_alpha: bool,
    },
    Vector {
        stroke_count: u32,
    },
    Text {
        text: String,
        font: String,
        size: f64,
    },
}

impl ClipboardData {
    pub fn empty() -> Self {
        Self {
            data_type: ClipboardDataType::Empty,
            layers: Vec::new(),
            bounds: Rect::ZERO,
            source_document: None,
            timestamp: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.data_type == ClipboardDataType::Empty || self.layers.is_empty()
    }

    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    pub fn has_raster(&self) -> bool {
        self.layers.iter().any(|l| matches!(l.layer_type, ClipboardLayerType::Raster { .. }))
    }

    pub fn has_vector(&self) -> bool {
        self.layers.iter().any(|l| matches!(l.layer_type, ClipboardLayerType::Vector { .. }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clipboard {
    data: ClipboardData,
    history: Vec<ClipboardData>,
    max_history: usize,
}

impl Clipboard {
    pub fn new() -> Self {
        Self {
            data: ClipboardData::empty(),
            history: Vec::new(),
            max_history: 10,
        }
    }

    pub fn set(&mut self, data: ClipboardData) {
        if !self.data.is_empty() {
            self.history.push(self.data.clone());
            if self.history.len() > self.max_history {
                self.history.remove(0);
            }
        }
        self.data = data;
    }

    pub fn get(&self) -> &ClipboardData {
        &self.data
    }

    pub fn clear(&mut self) {
        self.data = ClipboardData::empty();
    }

    pub fn history(&self) -> &[ClipboardData] {
        &self.history
    }

    pub fn restore_from_history(&mut self, index: usize) -> bool {
        if index < self.history.len() {
            self.data = self.history[index].clone();
            true
        } else {
            false
        }
    }
}

impl Default for Clipboard {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DragDropData {
    pub source: DragDropSource,
    pub data: DragDropPayload,
    pub allowed_operations: Vec<DropOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DragDropSource {
    Internal {
        document: String,
        layer_ids: Vec<LayerId>,
    },
    ExternalFile {
        paths: Vec<String>,
    },
    ExternalImage {
        path: String,
        width: u32,
        height: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DragDropPayload {
    Layers {
        layers: Vec<ClipboardLayer>,
    },
    Files {
        paths: Vec<String>,
    },
    Image {
        data: Vec<u8>,
        format: ImageFormat,
    },
    Color {
        color: Color8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Bmp,
    Tiff,
    Gif,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DropOperation {
    Copy,
    Move,
    Link,
    Reference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropTarget {
    pub position: Point,
    pub target_layer: Option<LayerId>,
    pub target_frame: u32,
    pub operation: DropOperation,
}

impl DragDropData {
    pub fn from_files(paths: Vec<String>) -> Self {
        Self {
            source: DragDropSource::ExternalFile { paths: paths.clone() },
            data: DragDropPayload::Files { paths },
            allowed_operations: vec![DropOperation::Copy, DropOperation::Link],
        }
    }

    pub fn from_layers(document: String, layer_ids: Vec<LayerId>, layers: Vec<ClipboardLayer>) -> Self {
        Self {
            source: DragDropSource::Internal { document, layer_ids },
            data: DragDropPayload::Layers { layers },
            allowed_operations: vec![DropOperation::Copy, DropOperation::Move],
        }
    }

    pub fn from_color(color: Color8) -> Self {
        Self {
            source: DragDropSource::Internal {
                document: String::new(),
                layer_ids: Vec::new(),
            },
            data: DragDropPayload::Color { color },
            allowed_operations: vec![DropOperation::Copy],
        }
    }

    pub fn can_perform(&self, operation: DropOperation) -> bool {
        self.allowed_operations.contains(&operation)
    }
}
