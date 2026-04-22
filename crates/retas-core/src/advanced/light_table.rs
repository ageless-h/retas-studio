use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{Point, Color8, LayerId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightTable {
    pub enabled: bool,
    pub reference_layers: Vec<ReferenceLayer>,
    pub onion_skin: OnionSkinSettings,
    pub opacity: f64,
}

impl Default for LightTable {
    fn default() -> Self {
        Self {
            enabled: false,
            reference_layers: Vec::new(),
            onion_skin: OnionSkinSettings::default(),
            opacity: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceLayer {
    pub id: LayerId,
    pub name: String,
    pub image_path: Option<String>,
    pub opacity: f64,
    pub tint: Option<Color8>,
    pub position: Point,
    pub scale: f64,
    pub rotation: f64,
    pub visible: bool,
    pub locked: bool,
}

impl ReferenceLayer {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: LayerId::new(),
            name: name.into(),
            image_path: None,
            opacity: 1.0,
            tint: None,
            position: Point::ZERO,
            scale: 1.0,
            rotation: 0.0,
            visible: true,
            locked: false,
        }
    }

    pub fn from_image(path: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: LayerId::new(),
            name: name.into(),
            image_path: Some(path.into()),
            opacity: 1.0,
            tint: None,
            position: Point::ZERO,
            scale: 1.0,
            rotation: 0.0,
            visible: true,
            locked: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnionSkinSettings {
    pub enabled: bool,
    pub frames_before: u32,
    pub frames_after: u32,
    pub opacity_before: f64,
    pub opacity_after: f64,
    pub color_before: Color8,
    pub color_after: Color8,
    pub blend_mode: OnionBlendMode,
}

impl Default for OnionSkinSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            frames_before: 1,
            frames_after: 1,
            opacity_before: 0.3,
            opacity_after: 0.3,
            color_before: Color8::new(255, 0, 0, 255),
            color_after: Color8::new(0, 255, 0, 255),
            blend_mode: OnionBlendMode::Tint,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OnionBlendMode {
    Tint,
    Overlay,
    Difference,
    Normal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightTableManager {
    light_tables: HashMap<u32, LightTable>,
    current_frame: u32,
}

impl LightTableManager {
    pub fn new() -> Self {
        Self {
            light_tables: HashMap::new(),
            current_frame: 0,
        }
    }

    pub fn get_or_create(&mut self, frame: u32) -> &mut LightTable {
        self.light_tables.entry(frame).or_insert_with(LightTable::default)
    }

    pub fn get(&self, frame: u32) -> Option<&LightTable> {
        self.light_tables.get(&frame)
    }

    pub fn set_current_frame(&mut self, frame: u32) {
        self.current_frame = frame;
    }

    pub fn current(&self) -> Option<&LightTable> {
        self.light_tables.get(&self.current_frame)
    }

    pub fn current_mut(&mut self) -> Option<&mut LightTable> {
        self.light_tables.get_mut(&self.current_frame)
    }

    pub fn add_reference(&mut self, frame: u32, reference: ReferenceLayer) {
        let table = self.get_or_create(frame);
        table.reference_layers.push(reference);
    }

    pub fn remove_reference(&mut self, frame: u32, id: LayerId) -> bool {
        if let Some(table) = self.light_tables.get_mut(&frame) {
            let len = table.reference_layers.len();
            table.reference_layers.retain(|r| r.id != id);
            table.reference_layers.len() != len
        } else {
            false
        }
    }

    pub fn toggle_onion_skin(&mut self, frame: u32, enabled: bool) {
        let table = self.get_or_create(frame);
        table.onion_skin.enabled = enabled;
    }

    pub fn set_onion_skin_frames(&mut self, frame: u32, before: u32, after: u32) {
        let table = self.get_or_create(frame);
        table.onion_skin.frames_before = before;
        table.onion_skin.frames_after = after;
    }

    pub fn get_visible_references(&self, frame: u32) -> Vec<&ReferenceLayer> {
        self.light_tables
            .get(&frame)
            .map(|t| t.reference_layers.iter().filter(|r| r.visible).collect())
            .unwrap_or_default()
    }

    pub fn get_onion_skin_frames(&self, current_frame: u32) -> Vec<(u32, f64, Color8)> {
        if let Some(table) = self.light_tables.get(&current_frame) {
            if !table.onion_skin.enabled {
                return Vec::new();
            }

            let mut frames = Vec::new();
            
            for i in 1..=table.onion_skin.frames_before {
                let frame = current_frame.saturating_sub(i);
                let opacity = table.onion_skin.opacity_before * (1.0 - (i as f64 - 1.0) / table.onion_skin.frames_before as f64);
                frames.push((frame, opacity, table.onion_skin.color_before));
            }
            
            for i in 1..=table.onion_skin.frames_after {
                let frame = current_frame + i;
                let opacity = table.onion_skin.opacity_after * (1.0 - (i as f64 - 1.0) / table.onion_skin.frames_after as f64);
                frames.push((frame, opacity, table.onion_skin.color_after));
            }
            
            frames
        } else {
            Vec::new()
        }
    }
}

impl Default for LightTableManager {
    fn default() -> Self {
        Self::new()
    }
}
