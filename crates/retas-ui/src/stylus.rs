use iced::mouse;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StylusInput {
    pub position: (f32, f32),
    pub pressure: f32,
    pub tilt_x: f32,
    pub tilt_y: f32,
    pub rotation: f32,
    pub is_eraser: bool,
    pub button: StylusButton,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StylusButton {
    None,
    Tip,
    Barrel,
    Eraser,
}

impl StylusInput {
    pub fn new(position: (f32, f32)) -> Self {
        Self {
            position,
            pressure: 1.0,
            tilt_x: 0.0,
            tilt_y: 0.0,
            rotation: 0.0,
            is_eraser: false,
            button: StylusButton::None,
        }
    }
    
    pub fn with_pressure(mut self, pressure: f32) -> Self {
        self.pressure = pressure.clamp(0.0, 1.0);
        self
    }
    
    pub fn with_tilt(mut self, x: f32, y: f32) -> Self {
        self.tilt_x = x;
        self.tilt_y = y;
        self
    }
    
    pub fn is_pen_active(&self) -> bool {
        self.pressure > 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PressureCurve {
    Linear,
    Soft,
    Hard,
    Custom([f32; 4]),
}

impl PressureCurve {
    pub fn apply(&self, pressure: f32) -> f32 {
        let pressure = pressure.clamp(0.0, 1.0);
        
        match self {
            PressureCurve::Linear => pressure,
            PressureCurve::Soft => pressure.powf(2.0),
            PressureCurve::Hard => pressure.sqrt(),
            PressureCurve::Custom(points) => {
                let p0 = points[0];
                let p1 = points[1];
                let p2 = points[2];
                let p3 = points[3];
                
                let t = pressure;
                let one_minus_t = 1.0 - t;
                
                one_minus_t.powi(3) * p0
                    + 3.0 * one_minus_t.powi(2) * t * p1
                    + 3.0 * one_minus_t * t.powi(2) * p2
                    + t.powi(3) * p3
            }
        }
    }
}

impl Default for PressureCurve {
    fn default() -> Self {
        PressureCurve::Linear
    }
}

#[derive(Debug, Clone)]
pub struct DynamicBrush {
    pub size_min: f32,
    pub size_max: f32,
    pub opacity_min: f32,
    pub opacity_max: f32,
    pub size_pressure: bool,
    pub opacity_pressure: bool,
    pub pressure_curve: PressureCurve,
}

impl DynamicBrush {
    pub fn new() -> Self {
        Self {
            size_min: 1.0,
            size_max: 20.0,
            opacity_min: 0.1,
            opacity_max: 1.0,
            size_pressure: true,
            opacity_pressure: true,
            pressure_curve: PressureCurve::Linear,
        }
    }
    
    pub fn calculate_size(&self, pressure: f32) -> f32 {
        if !self.size_pressure {
            return self.size_max;
        }
        
        let normalized = self.pressure_curve.apply(pressure);
        self.size_min + (self.size_max - self.size_min) * normalized
    }
    
    pub fn calculate_opacity(&self, pressure: f32) -> f32 {
        if !self.opacity_pressure {
            return self.opacity_max;
        }
        
        let normalized = self.pressure_curve.apply(pressure);
        self.opacity_min + (self.opacity_max - self.opacity_min) * normalized
    }
}

impl Default for DynamicBrush {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct BrushPreset {
    pub name: String,
    pub dynamic_brush: DynamicBrush,
    pub color: retas_core::Color8,
}

impl BrushPreset {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            dynamic_brush: DynamicBrush::new(),
            color: retas_core::Color8::BLACK,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BrushPresetManager {
    pub presets: Vec<BrushPreset>,
    pub current: usize,
}

impl BrushPresetManager {
    pub fn new() -> Self {
        let default_presets = vec![
            BrushPreset::new("Pencil"),
            BrushPreset::new("Ink Pen"),
            BrushPreset::new("Soft Brush"),
            BrushPreset::new("Marker"),
        ];
        
        Self {
            presets: default_presets,
            current: 0,
        }
    }
    
    pub fn add_preset(&mut self, preset: BrushPreset) {
        self.presets.push(preset);
    }
    
    pub fn remove_preset(&mut self, index: usize) {
        if self.presets.len() > 1 && index < self.presets.len() {
            self.presets.remove(index);
            if self.current >= self.presets.len() {
                self.current = self.presets.len() - 1;
            }
        }
    }
    
    pub fn current_preset(&self) -> Option<&BrushPreset> {
        self.presets.get(self.current)
    }
    
    pub fn current_preset_mut(&mut self) -> Option<&mut BrushPreset> {
        self.presets.get_mut(self.current)
    }
    
    pub fn set_current(&mut self, index: usize) {
        if index < self.presets.len() {
            self.current = index;
        }
    }
}

impl Default for BrushPresetManager {
    fn default() -> Self {
        Self::new()
    }
}
