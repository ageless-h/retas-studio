use serde::{Deserialize, Serialize};
use crate::{LayerId, Point};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ToolId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolType {
    // Selection tools
    Selection,
    Lasso,
    MagicWand,
    // Move/Transform tools
    Move,
    Rotate,
    Scale,
    // Drawing tools
    Brush,
    Eraser,
    Pen,
    Pencil,
    Marker,
    Airbrush,
    // Fill tools
    Fill,
    GradientFill,
    PatternFill,
    // Color tools
    Eyedropper,
    // Navigation tools
    Zoom,
    Hand,
    // Shape tools
    Shape,
    Line,
    Curve,
    Rectangle,
    Ellipse,
    Polygon,
    Polyline,
    // Text tool
    Text,
    // Blur/Smudge tools
    Blur,
    Smudge,
    Sharpen,
    // Line processing tools (Stylos)
    LinePush,
    LineSmooth,
    LineVolume,
    LineTaper,
    LineConnect,
    LineBreak,
    // Smart drawing tools
    SmartDraw,
    SmartInk,
    LineThin,
    // Special tools
    Stamp,
    Dust,
    Texture,
    Clone,
    Heal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModifierKey {
    Shift,
    Control,
    Alt,
    Command,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEvent {
    pub position: Point,
    pub button: MouseButton,
    pub modifiers: Vec<ModifierKey>,
    pub pressure: f64,
    pub tilt: Option<(f64, f64)>,
    pub is_start: bool,
    pub is_end: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContext {
    pub current_layer: Option<LayerId>,
    pub current_frame: u32,
    pub primary_color: crate::Color8,
    pub secondary_color: crate::Color8,
    pub brush_size: f64,
    pub brush_opacity: f64,
}

impl Default for ToolContext {
    fn default() -> Self {
        Self {
            current_layer: None,
            current_frame: 0,
            primary_color: crate::Color8::BLACK,
            secondary_color: crate::Color8::WHITE,
            brush_size: 10.0,
            brush_opacity: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolAction {
    pub action_type: ToolActionType,
    pub affected_layer: Option<LayerId>,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolActionType {
    None,
    Draw,
    Erase,
    Fill,
    Move,
    Transform,
    Select,
    Deselect,
}

pub trait Tool: Send + Sync {
    fn id(&self) -> ToolId;
    fn name(&self) -> &str;
    fn tool_type(&self) -> ToolType;
    
    fn on_event(&mut self, event: &ToolEvent, context: &ToolContext) -> Option<ToolAction>;
    fn on_activate(&mut self, _context: &ToolContext) {}
    fn on_deactivate(&mut self, _context: &ToolContext) {}
    fn on_cursor_change(&self, _context: &ToolContext) -> Option<CursorShape> {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CursorShape {
    Default,
    Crosshair,
    Hand,
    Move,
    Text,
    Wait,
    Grab,
    Grabbing,
    ZoomIn,
    ZoomOut,
    Custom(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrushSettings {
    pub size: f64,
    pub min_size: f64,
    pub opacity: f64,
    pub min_opacity: f64,
    pub hardness: f64,
    pub spacing: f64,
    pub flow: f64,
    pub smoothing: f64,
    pub pressure_sensitivity: bool,
    pub size_pressure: bool,
    pub opacity_pressure: bool,
}

impl Default for BrushSettings {
    fn default() -> Self {
        Self {
            size: 10.0,
            min_size: 1.0,
            opacity: 1.0,
            min_opacity: 0.0,
            hardness: 0.8,
            spacing: 0.1,
            flow: 1.0,
            smoothing: 0.5,
            pressure_sensitivity: true,
            size_pressure: true,
            opacity_pressure: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineProcessingSettings {
    pub strength: f64,
    pub radius: f64,
    pub preserve_ends: bool,
    pub smooth_iterations: u32,
}

impl Default for LineProcessingSettings {
    fn default() -> Self {
        Self {
            strength: 0.5,
            radius: 20.0,
            preserve_ends: true,
            smooth_iterations: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinePushSettings {
    pub radius: f64,
    pub strength: f64,
    pub falloff: f64,
}

impl Default for LinePushSettings {
    fn default() -> Self {
        Self {
            radius: 30.0,
            strength: 1.0,
            falloff: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineVolumeSettings {
    pub scale_factor: f64,
    pub affect_tips: bool,
    pub min_thickness: f64,
    pub max_thickness: f64,
}

impl Default for LineVolumeSettings {
    fn default() -> Self {
        Self {
            scale_factor: 1.0,
            affect_tips: true,
            min_thickness: 0.5,
            max_thickness: 10.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartDrawSettings {
    pub auto_smooth: bool,
    pub auto_connect: bool,
    pub connect_distance: f64,
    pub thin_lines: bool,
    pub target_thickness: f64,
}

impl Default for SmartDrawSettings {
    fn default() -> Self {
        Self {
            auto_smooth: true,
            auto_connect: true,
            connect_distance: 5.0,
            thin_lines: false,
            target_thickness: 2.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StampSettings {
    pub source_point: Option<Point>,
    pub size: f64,
    pub opacity: f64,
    pub aligned: bool,
    pub _sample_all_layers: bool,
}

impl Default for StampSettings {
    fn default() -> Self {
        Self {
            source_point: None,
            size: 50.0,
            opacity: 1.0,
            aligned: true,
            _sample_all_layers: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DustSettings {
    pub radius: f64,
    pub threshold: f64,
    pub mode: DustMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DustMode {
    Remove,
    Protect,
    Highlight,
}

impl Default for DustSettings {
    fn default() -> Self {
        Self {
            radius: 5.0,
            threshold: 0.1,
            mode: DustMode::Remove,
        }
    }
}
