use iced::Color;
use retas_core::Color8;
use std::sync::atomic::{AtomicU64, Ordering};

static LAYER_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LayerId(pub u64);

impl LayerId {
    pub fn new() -> Self {
        Self(LAYER_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

impl Default for LayerId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerType {
    Raster,
    Vector,
    Group,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Add,
    Subtract,
}

impl BlendMode {
    pub fn blend(&self, base: Color8, blend: Color8, opacity: f32) -> Color8 {
        let alpha = opacity * (blend.a as f32 / 255.0);
        let inv_alpha = 1.0 - alpha;
        
        match self {
            BlendMode::Normal => Color8 {
                r: (base.r as f32 * inv_alpha + blend.r as f32 * alpha) as u8,
                g: (base.g as f32 * inv_alpha + blend.g as f32 * alpha) as u8,
                b: (base.b as f32 * inv_alpha + blend.b as f32 * alpha) as u8,
                a: (base.a as f32 * inv_alpha + blend.a as f32 * alpha) as u8,
            },
            BlendMode::Multiply => {
                let r = (base.r as f32 * blend.r as f32 / 255.0);
                let g = (base.g as f32 * blend.g as f32 / 255.0);
                let b = (base.b as f32 * blend.b as f32 / 255.0);
                Color8 {
                    r: (base.r as f32 * inv_alpha + r * alpha) as u8,
                    g: (base.g as f32 * inv_alpha + g * alpha) as u8,
                    b: (base.b as f32 * inv_alpha + b * alpha) as u8,
                    a: base.a,
                }
            }
            BlendMode::Screen => {
                let r = 255.0 - ((255.0 - base.r as f32) * (255.0 - blend.r as f32) / 255.0);
                let g = 255.0 - ((255.0 - base.g as f32) * (255.0 - blend.g as f32) / 255.0);
                let b = 255.0 - ((255.0 - base.b as f32) * (255.0 - blend.b as f32) / 255.0);
                Color8 {
                    r: (base.r as f32 * inv_alpha + r * alpha) as u8,
                    g: (base.g as f32 * inv_alpha + g * alpha) as u8,
                    b: (base.b as f32 * inv_alpha + b * alpha) as u8,
                    a: base.a,
                }
            }
            BlendMode::Add => {
                let r = (base.r as f32 + blend.r as f32).min(255.0);
                let g = (base.g as f32 + blend.g as f32).min(255.0);
                let b = (base.b as f32 + blend.b as f32).min(255.0);
                Color8 {
                    r: (base.r as f32 * inv_alpha + r * alpha) as u8,
                    g: (base.g as f32 * inv_alpha + g * alpha) as u8,
                    b: (base.b as f32 * inv_alpha + b * alpha) as u8,
                    a: base.a,
                }
            }
            _ => base,
        }
    }
}

impl Default for BlendMode {
    fn default() -> Self {
        BlendMode::Normal
    }
}

#[derive(Debug, Clone)]
pub struct Layer {
    pub id: LayerId,
    pub name: String,
    pub layer_type: LayerType,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub pixel_data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl Layer {
    pub fn new_raster(name: impl Into<String>, width: u32, height: u32) -> Self {
        let pixel_count = (width * height * 4) as usize;
        Self {
            id: LayerId::new(),
            name: name.into(),
            layer_type: LayerType::Raster,
            visible: true,
            locked: false,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            pixel_data: vec![255u8; pixel_count],
            width,
            height,
        }
    }
    
    pub fn clear(&mut self) {
        self.pixel_data.fill(255);
    }
    
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<Color8> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        if idx + 3 < self.pixel_data.len() {
            Some(Color8 {
                r: self.pixel_data[idx],
                g: self.pixel_data[idx + 1],
                b: self.pixel_data[idx + 2],
                a: self.pixel_data[idx + 3],
            })
        } else {
            None
        }
    }
    
    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color8) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        if idx + 3 < self.pixel_data.len() {
            self.pixel_data[idx] = color.r;
            self.pixel_data[idx + 1] = color.g;
            self.pixel_data[idx + 2] = color.b;
            self.pixel_data[idx + 3] = color.a;
        }
    }
}

#[derive(Debug, Clone)]
pub struct LayerManager {
    pub layers: Vec<Layer>,
    pub active_layer_id: Option<LayerId>,
}

impl LayerManager {
    pub fn new(width: u32, height: u32) -> Self {
        let base_layer = Layer::new_raster("Layer 1", width, height);
        let active_id = base_layer.id;
        Self {
            layers: vec![base_layer],
            active_layer_id: Some(active_id),
        }
    }
    
    pub fn add_layer(&mut self, name: impl Into<String>) -> LayerId {
        let layer = Layer::new_raster(name, self.width(), self.height());
        let id = layer.id;
        self.layers.push(layer);
        self.active_layer_id = Some(id);
        id
    }
    
    pub fn add_layer_at(&mut self, index: usize, name: impl Into<String>) -> LayerId {
        let layer = Layer::new_raster(name, self.width(), self.height());
        let id = layer.id;
        let index = index.min(self.layers.len());
        self.layers.insert(index, layer);
        self.active_layer_id = Some(id);
        id
    }
    
    pub fn delete_layer(&mut self, id: LayerId) -> bool {
        if self.layers.len() <= 1 {
            return false;
        }
        
        if let Some(index) = self.layers.iter().position(|l| l.id == id) {
            self.layers.remove(index);
            
            if self.active_layer_id == Some(id) {
                self.active_layer_id = self.layers.get(index.saturating_sub(1))
                    .or_else(|| self.layers.last())
                    .map(|l| l.id);
            }
            return true;
        }
        false
    }
    
    pub fn get_layer(&self, id: LayerId) -> Option<&Layer> {
        self.layers.iter().find(|l| l.id == id)
    }
    
    pub fn get_layer_mut(&mut self, id: LayerId) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|l| l.id == id)
    }
    
    pub fn get_active_layer(&self) -> Option<&Layer> {
        self.active_layer_id.and_then(|id| self.get_layer(id))
    }
    
    pub fn get_active_layer_mut(&mut self) -> Option<&mut Layer> {
        self.active_layer_id.and_then(|id| self.get_layer_mut(id))
    }
    
    pub fn set_active_layer(&mut self, id: LayerId) {
        if self.layers.iter().any(|l| l.id == id) {
            self.active_layer_id = Some(id);
        }
    }
    
    pub fn move_layer_up(&mut self, id: LayerId) -> bool {
        if let Some(index) = self.layers.iter().position(|l| l.id == id) {
            if index < self.layers.len() - 1 {
                self.layers.swap(index, index + 1);
                return true;
            }
        }
        false
    }
    
    pub fn move_layer_down(&mut self, id: LayerId) -> bool {
        if let Some(index) = self.layers.iter().position(|l| l.id == id) {
            if index > 0 {
                self.layers.swap(index, index - 1);
                return true;
            }
        }
        false
    }
    
    pub fn rename_layer(&mut self, id: LayerId, name: impl Into<String>) -> bool {
        if let Some(layer) = self.get_layer_mut(id) {
            layer.name = name.into();
            return true;
        }
        false
    }
    
    pub fn composite_layers(&self) -> Vec<u8> {
        let width = self.width();
        let height = self.height();
        let pixel_count = (width * height) as usize;
        let mut result = vec![255u8; pixel_count * 4];
        
        for y in 0..height {
            for x in 0..width {
                let mut base_color = Color8::WHITE;
                
                for layer in &self.layers {
                    if !layer.visible || layer.opacity <= 0.0 {
                        continue;
                    }
                    
                    if let Some(pixel) = layer.get_pixel(x, y) {
                        base_color = layer.blend_mode.blend(base_color, pixel, layer.opacity);
                    }
                }
                
                let idx = ((y * width + x) * 4) as usize;
                result[idx] = base_color.r;
                result[idx + 1] = base_color.g;
                result[idx + 2] = base_color.b;
                result[idx + 3] = base_color.a;
            }
        }
        
        result
    }
    
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }
    
    pub fn width(&self) -> u32 {
        self.layers.first().map(|l| l.width).unwrap_or(1920)
    }
    
    pub fn height(&self) -> u32 {
        self.layers.first().map(|l| l.height).unwrap_or(1080)
    }
}

impl Default for LayerManager {
    fn default() -> Self {
        Self::new(1920, 1080)
    }
}
