use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use crate::{LayerId, Color8};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItem {
    pub id: u64,
    pub operation: BatchOperation,
    pub status: BatchStatus,
    pub priority: BatchPriority,
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BatchStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Skipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BatchPriority {
    Low,
    Normal,
    High,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchOperation {
    ExportSequence {
        output_dir: PathBuf,
        format: ExportFormat,
        start_frame: u32,
        end_frame: u32,
    },
    ConvertLayer {
        source_layer: LayerId,
        target_type: LayerConversionType,
    },
    ApplyEffect {
        layer: LayerId,
        effect: BatchEffect,
    },
    ResizeDocument {
        new_width: u32,
        new_height: u32,
        interpolation: ResizeInterpolation,
    },
    ColorReplace {
        source_color: Color8,
        target_color: Color8,
        tolerance: u8,
    },
    LineSmooth {
        layer: LayerId,
        strength: f64,
        iterations: u32,
    },
    LineVolume {
        layer: LayerId,
        scale_factor: f64,
    },
    FillConsecutive {
        color: Color8,
        tolerance: u8,
    },
    TraceVectorize {
        input_path: PathBuf,
        output_path: PathBuf,
        threshold: u8,
        smooth: bool,
    },
    Custom {
        name: String,
        params: Vec<u8>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExportFormat {
    Png,
    Jpeg,
    Tiff,
    Bmp,
    Tga,
    Gif,
    Avi,
    Mov,
    Swf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LayerConversionType {
    ToRaster,
    ToVector,
    ToPaint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResizeInterpolation {
    Nearest,
    Bilinear,
    Bicubic,
    Lanczos,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchEffect {
    Blur { radius: f64 },
    Sharpen { amount: f64 },
    Brightness { value: f64 },
    Contrast { value: f64 },
    HueShift { degrees: f64 },
    Saturation { factor: f64 },
    Invert,
    Grayscale,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchQueue {
    items: VecDeque<BatchItem>,
    current_id: u64,
    max_concurrent: usize,
    running_count: usize,
}

impl BatchQueue {
    pub fn new() -> Self {
        Self {
            items: VecDeque::new(),
            current_id: 0,
            max_concurrent: 4,
            running_count: 0,
        }
    }

    pub fn with_max_concurrent(max: usize) -> Self {
        Self {
            items: VecDeque::new(),
            current_id: 0,
            max_concurrent: max,
            running_count: 0,
        }
    }

    pub fn add(&mut self, operation: BatchOperation, priority: BatchPriority) -> u64 {
        let id = self.current_id;
        self.current_id += 1;
        
        let item = BatchItem {
            id,
            operation,
            status: BatchStatus::Pending,
            priority,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            started_at: None,
            completed_at: None,
            error: None,
        };
        
        match priority {
            BatchPriority::Urgent | BatchPriority::High => {
                self.items.push_front(item);
            }
            _ => {
                self.items.push_back(item);
            }
        }
        
        id
    }

    pub fn get_next(&mut self) -> Option<BatchItem> {
        if self.running_count >= self.max_concurrent {
            return None;
        }
        
        let mut sorted: Vec<_> = self.items.iter().enumerate().collect();
        sorted.sort_by(|a, b| b.1.priority.cmp(&a.1.priority));
        
        if let Some((idx, _)) = sorted.first() {
            let item = self.items.remove(*idx)?;
            self.running_count += 1;
            Some(item)
        } else {
            None
        }
    }

    pub fn complete(&mut self, id: u64, result: Result<(), String>) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            self.running_count = self.running_count.saturating_sub(1);
            match result {
                Ok(()) => {
                    item.status = BatchStatus::Completed;
                    item.completed_at = Some(std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0));
                }
                Err(e) => {
                    item.status = BatchStatus::Failed;
                    item.error = Some(e);
                }
            }
        }
    }

    pub fn cancel(&mut self, id: u64) -> bool {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            if item.status == BatchStatus::Pending || item.status == BatchStatus::Running {
                item.status = BatchStatus::Cancelled;
                return true;
            }
        }
        false
    }

    pub fn cancel_all(&mut self) {
        for item in self.items.iter_mut() {
            if item.status == BatchStatus::Pending {
                item.status = BatchStatus::Cancelled;
            }
        }
    }

    pub fn pending_count(&self) -> usize {
        self.items.iter().filter(|i| i.status == BatchStatus::Pending).count()
    }

    pub fn running_count(&self) -> usize {
        self.running_count
    }

    pub fn completed_count(&self) -> usize {
        self.items.iter().filter(|i| i.status == BatchStatus::Completed).count()
    }

    pub fn failed_count(&self) -> usize {
        self.items.iter().filter(|i| i.status == BatchStatus::Failed).count()
    }

    pub fn total_count(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn clear_completed(&mut self) {
        self.items.retain(|i| i.status != BatchStatus::Completed);
    }

    pub fn get_by_id(&self, id: u64) -> Option<&BatchItem> {
        self.items.iter().find(|i| i.id == id)
    }

    pub fn get_all(&self) -> &VecDeque<BatchItem> {
        &self.items
    }

    pub fn get_pending(&self) -> Vec<&BatchItem> {
        self.items.iter().filter(|i| i.status == BatchStatus::Pending).collect()
    }

    pub fn get_running(&self) -> Vec<&BatchItem> {
        self.items.iter().filter(|i| i.status == BatchStatus::Running).collect()
    }

    pub fn get_failed(&self) -> Vec<&BatchItem> {
        self.items.iter().filter(|i| i.status == BatchStatus::Failed).collect()
    }
}

impl Default for BatchQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchPreset {
    pub name: String,
    pub operations: Vec<BatchOperation>,
}

impl BatchPreset {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            operations: Vec::new(),
        }
    }

    pub fn add_operation(&mut self, operation: BatchOperation) {
        self.operations.push(operation);
    }
}

pub fn create_export_preset(output_dir: PathBuf, format: ExportFormat, start: u32, end: u32) -> BatchPreset {
    let mut preset = BatchPreset::new("Export Sequence");
    preset.add_operation(BatchOperation::ExportSequence {
        output_dir,
        format,
        start_frame: start,
        end_frame: end,
    });
    preset
}

pub fn create_color_replace_preset(source: Color8, target: Color8, tolerance: u8) -> BatchPreset {
    let mut preset = BatchPreset::new("Color Replace");
    preset.add_operation(BatchOperation::ColorReplace {
        source_color: source,
        target_color: target,
        tolerance,
    });
    preset
}
