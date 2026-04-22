use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub use uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LayerId(pub Uuid);

impl LayerId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for LayerId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LayerType {
    Raster,
    Vector,
    Camera,
    Text,
    Shape,
    Guide,
    Sound,
    Adjustment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

impl Default for BlendMode {
    fn default() -> Self {
        BlendMode::Normal
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerBase {
    pub id: LayerId,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f64,
    pub blend_mode: BlendMode,
    pub layer_type: LayerType,
    pub parent: Option<LayerId>,
    pub children: Vec<LayerId>,
}

impl LayerBase {
    pub fn new(name: impl Into<String>, layer_type: LayerType) -> Self {
        Self {
            id: LayerId::new(),
            name: name.into(),
            visible: true,
            locked: false,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            layer_type,
            parent: None,
            children: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RasterFrame {
    pub frame_number: u32,
    pub image_data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub bounds: Option<super::Rect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RasterLayer {
    pub base: LayerBase,
    pub frames: HashMap<u32, RasterFrame>,
    pub current_frame: u32,
    pub offset: super::Point,
}

impl RasterLayer {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: LayerBase::new(name, LayerType::Raster),
            frames: HashMap::new(),
            current_frame: 0,
            offset: super::Point::ZERO,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrokePoint {
    pub position: super::Point,
    pub pressure: f64,
    pub tilt: Option<(f64, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stroke {
    pub points: Vec<StrokePoint>,
    pub brush_size: f64,
    pub color: super::Color8,
    pub opacity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorFrame {
    pub frame_number: u32,
    pub strokes: Vec<Stroke>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorLayer {
    pub base: LayerBase,
    pub frames: HashMap<u32, VectorFrame>,
    pub current_frame: u32,
    pub antialiasing: bool,
}

impl VectorLayer {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: LayerBase::new(name, LayerType::Vector),
            frames: HashMap::new(),
            current_frame: 0,
            antialiasing: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraLayer {
    pub base: LayerBase,
    pub position: super::Point,
    pub zoom: f64,
    pub rotation: f64,
    pub resolution: (u32, u32),
    pub frame_rate: f64,
}

impl CameraLayer {
    pub fn new(name: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            base: LayerBase::new(name, LayerType::Camera),
            position: super::Point::ZERO,
            zoom: 1.0,
            rotation: 0.0,
            resolution: (width, height),
            frame_rate: 24.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextLayer {
    pub base: LayerBase,
    pub text: String,
    pub font_family: String,
    pub font_size: f64,
    pub position: super::Point,
}

impl TextLayer {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: LayerBase::new(name, LayerType::Text),
            text: String::new(),
            font_family: "Arial".to_string(),
            font_size: 24.0,
            position: super::Point::ZERO,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundLayer {
    pub base: LayerBase,
    pub audio_data: Vec<u8>,
    pub sample_rate: u32,
    pub channels: u16,
    pub start_frame: u32,
    pub duration_frames: u32,
}

impl SoundLayer {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: LayerBase::new(name, LayerType::Sound),
            audio_data: Vec::new(),
            sample_rate: 44100,
            channels: 2,
            start_frame: 0,
            duration_frames: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Layer {
    Raster(RasterLayer),
    Vector(VectorLayer),
    Camera(CameraLayer),
    Text(TextLayer),
    Sound(SoundLayer),
}

impl Layer {
    pub fn base(&self) -> &LayerBase {
        match self {
            Layer::Raster(l) => &l.base,
            Layer::Vector(l) => &l.base,
            Layer::Camera(l) => &l.base,
            Layer::Text(l) => &l.base,
            Layer::Sound(l) => &l.base,
        }
    }

    pub fn base_mut(&mut self) -> &mut LayerBase {
        match self {
            Layer::Raster(l) => &mut l.base,
            Layer::Vector(l) => &mut l.base,
            Layer::Camera(l) => &mut l.base,
            Layer::Text(l) => &mut l.base,
            Layer::Sound(l) => &mut l.base,
        }
    }

    pub fn id(&self) -> LayerId {
        self.base().id
    }

    pub fn name(&self) -> &str {
        &self.base().name
    }

    pub fn layer_type(&self) -> LayerType {
        self.base().layer_type
    }
}
