use serde::{Deserialize, Serialize};
use crate::{Point, Color8};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BrushType {
    Round,
    Flat,
    Calligraphy,
    Airbrush,
    Pencil,
    Marker,
    Watercolor,
    Oil,
    Custom(u32),
}

impl Default for BrushType {
    fn default() -> Self {
        BrushType::Round
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BrushBlendMode {
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
}

impl Default for BrushBlendMode {
    fn default() -> Self {
        BrushBlendMode::Normal
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrushDynamics {
    pub pressure_size: bool,
    pub pressure_opacity: bool,
    pub pressure_flow: bool,
    pub pressure_hardness: bool,
    pub pressure_scatter: bool,
    pub tilt_size: bool,
    pub tilt_angle: bool,
    pub velocity_size: bool,
    pub velocity_opacity: bool,
}

impl Default for BrushDynamics {
    fn default() -> Self {
        Self {
            pressure_size: true,
            pressure_opacity: true,
            pressure_flow: false,
            pressure_hardness: false,
            pressure_scatter: false,
            tilt_size: false,
            tilt_angle: false,
            velocity_size: false,
            velocity_opacity: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrushTexture {
    pub enabled: bool,
    pub texture_id: Option<u32>,
    pub scale: f64,
    pub brightness: f64,
    pub contrast: f64,
    pub blend_mode: BrushBlendMode,
    pub invert: bool,
    pub protect_texture: bool,
}

impl Default for BrushTexture {
    fn default() -> Self {
        Self {
            enabled: false,
            texture_id: None,
            scale: 1.0,
            brightness: 0.0,
            contrast: 1.0,
            blend_mode: BrushBlendMode::Normal,
            invert: false,
            protect_texture: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrushScatter {
    pub enabled: bool,
    pub scatter_x: f64,
    pub scatter_y: f64,
    pub count: u32,
    pub count_jitter: f64,
    pub scatter_pressure: bool,
}

impl Default for BrushScatter {
    fn default() -> Self {
        Self {
            enabled: false,
            scatter_x: 0.0,
            scatter_y: 0.0,
            count: 1,
            count_jitter: 0.0,
            scatter_pressure: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrushShape {
    pub angle: f64,
    pub angle_jitter: f64,
    pub angle_control: AngleControl,
    pub roundness: f64,
    pub roundness_jitter: f64,
    pub flip_x: bool,
    pub flip_y: bool,
}

impl Default for BrushShape {
    fn default() -> Self {
        Self {
            angle: 0.0,
            angle_jitter: 0.0,
            angle_control: AngleControl::Fixed,
            roundness: 1.0,
            roundness_jitter: 0.0,
            flip_x: false,
            flip_y: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AngleControl {
    Fixed,
    InitialDirection,
    Direction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrushSettings {
    pub brush_type: BrushType,
    pub size: f64,
    pub min_size: f64,
    pub max_size: f64,
    pub opacity: f64,
    pub min_opacity: f64,
    pub flow: f64,
    pub min_flow: f64,
    pub hardness: f64,
    pub spacing: f64,
    pub smoothing: f64,
    pub stabilization: f64,
    pub color: Color8,
    pub blend_mode: BrushBlendMode,
    pub dynamics: BrushDynamics,
    pub texture: BrushTexture,
    pub scatter: BrushScatter,
    pub shape: BrushShape,
    pub wet_edges: bool,
    pub build_up: bool,
    pub anti_aliasing: bool,
    pub _sample_all_layers: bool,
}

impl Default for BrushSettings {
    fn default() -> Self {
        Self {
            brush_type: BrushType::Round,
            size: 10.0,
            min_size: 1.0,
            max_size: 100.0,
            opacity: 1.0,
            min_opacity: 0.0,
            flow: 1.0,
            min_flow: 0.0,
            hardness: 0.8,
            spacing: 0.1,
            smoothing: 0.5,
            stabilization: 0.0,
            color: Color8::BLACK,
            blend_mode: BrushBlendMode::Normal,
            dynamics: BrushDynamics::default(),
            texture: BrushTexture::default(),
            scatter: BrushScatter::default(),
            shape: BrushShape::default(),
            wet_edges: false,
            build_up: false,
            anti_aliasing: true,
            _sample_all_layers: false,
        }
    }
}

impl BrushSettings {
    pub fn new(size: f64, color: Color8) -> Self {
        Self {
            size,
            color,
            ..Default::default()
        }
    }

    pub fn with_opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn with_hardness(mut self, hardness: f64) -> Self {
        self.hardness = hardness;
        self
    }

    pub fn with_type(mut self, brush_type: BrushType) -> Self {
        self.brush_type = brush_type;
        self
    }

    pub fn with_blend_mode(mut self, mode: BrushBlendMode) -> Self {
        self.blend_mode = mode;
        self
    }

    pub fn with_pressure_dynamics(mut self, size: bool, opacity: bool) -> Self {
        self.dynamics.pressure_size = size;
        self.dynamics.pressure_opacity = opacity;
        self
    }

    pub fn calculate_size(&self, pressure: f64, velocity: f64) -> f64 {
        let mut size = self.size;
        
        if self.dynamics.pressure_size {
            let pressure_factor = pressure.max(0.0).min(1.0);
            let min = self.min_size;
            size = min + (size - min) * pressure_factor;
        }
        
        if self.dynamics.velocity_size {
            let velocity_factor = velocity.max(0.0).min(1.0);
            size *= 1.0 - velocity_factor * 0.5;
        }
        
        size.max(1.0)
    }

    pub fn calculate_opacity(&self, pressure: f64, velocity: f64) -> f64 {
        let mut opacity = self.opacity;
        
        if self.dynamics.pressure_opacity {
            let pressure_factor = pressure.max(0.0).min(1.0);
            let min = self.min_opacity;
            opacity = min + (opacity - min) * pressure_factor;
        }
        
        if self.dynamics.velocity_opacity {
            let velocity_factor = velocity.max(0.0).min(1.0);
            opacity *= 1.0 - velocity_factor * 0.3;
        }
        
        opacity.max(0.0).min(1.0)
    }

    pub fn calculate_flow(&self, pressure: f64) -> f64 {
        let mut flow = self.flow;
        
        if self.dynamics.pressure_flow {
            let pressure_factor = pressure.max(0.0).min(1.0);
            let min = self.min_flow;
            flow = min + (flow - min) * pressure_factor;
        }
        
        flow.max(0.0).min(1.0)
    }

    pub fn calculate_angle(&self, direction: f64) -> f64 {
        match self.shape.angle_control {
            AngleControl::Fixed => self.shape.angle,
            AngleControl::InitialDirection => self.shape.angle,
            AngleControl::Direction => self.shape.angle + direction.to_degrees(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrushStroke {
    pub settings: BrushSettings,
    pub points: Vec<BrushPoint>,
    pub is_drawing: bool,
}

impl BrushStroke {
    pub fn new(settings: BrushSettings) -> Self {
        Self {
            settings,
            points: Vec::new(),
            is_drawing: false,
        }
    }

    pub fn add_point(&mut self, point: BrushPoint) {
        self.points.push(point);
    }

    pub fn finish(&mut self) {
        self.is_drawing = false;
    }

    pub fn calculate_interpolated_points(&self, tolerance: f64) -> Vec<BrushPoint> {
        if self.points.len() < 2 {
            return self.points.clone();
        }

        let mut result = Vec::new();
        let mut i = 0;

        while i < self.points.len() - 1 {
            let p1 = &self.points[i];
            let p2 = &self.points[i + 1];
            
            let distance = p1.position.distance_to(&p2.position);
            let spacing = self.settings.spacing * self.settings.size;
            
            if distance < tolerance {
                result.push(p1.clone());
                i += 1;
                continue;
            }

            let steps = (distance / spacing).ceil() as usize;
            
            for j in 0..steps {
                let t = j as f64 / steps as f64;
                result.push(BrushPoint {
                    position: p1.position.lerp(&p2.position, t),
                    pressure: p1.pressure + (p2.pressure - p1.pressure) * t,
                    tilt_x: p1.tilt_x + (p2.tilt_x - p1.tilt_x) * t,
                    tilt_y: p1.tilt_y + (p2.tilt_y - p1.tilt_y) * t,
                    velocity: p1.velocity + (p2.velocity - p1.velocity) * t,
                    direction: p1.direction,
                    timestamp: p1.timestamp,
                });
            }
            
            i += 1;
        }

        if let Some(last) = self.points.last() {
            result.push(last.clone());
        }

        result
    }

    pub fn smooth(&mut self, strength: f64) {
        if self.points.len() < 3 || strength <= 0.0 {
            return;
        }

        let smoothed = self.points.clone();
        let weight = strength / 2.0;

        for i in 1..self.points.len() - 1 {
            let prev = &smoothed[i - 1];
            let curr = &smoothed[i];
            let next = &smoothed[i + 1];

            self.points[i].position = Point::new(
                curr.position.x * (1.0 - strength) + (prev.position.x + next.position.x) * weight,
                curr.position.y * (1.0 - strength) + (prev.position.y + next.position.y) * weight,
            );
        }
    }

    pub fn apply_stabilization(&mut self, points: &[BrushPoint]) {
        if self.settings.stabilization <= 0.0 || points.is_empty() {
            return;
        }

        let window_size = (self.settings.stabilization * 10.0) as usize;
        let window_size = window_size.max(2).min(50);

        let mut smoothed = points.to_vec();

        for i in 1..smoothed.len() {
            let start = i.saturating_sub(window_size);
            let window = &points[start..=i];
            
            let avg_x: f64 = window.iter().map(|p| p.position.x).sum::<f64>() / window.len() as f64;
            let avg_y: f64 = window.iter().map(|p| p.position.y).sum::<f64>() / window.len() as f64;
            
            smoothed[i].position = Point::new(avg_x, avg_y);
        }

        self.points = smoothed;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrushPoint {
    pub position: Point,
    pub pressure: f64,
    pub tilt_x: f64,
    pub tilt_y: f64,
    pub velocity: f64,
    pub direction: f64,
    pub timestamp: f64,
}

impl BrushPoint {
    pub fn new(position: Point) -> Self {
        Self {
            position,
            pressure: 1.0,
            tilt_x: 0.0,
            tilt_y: 0.0,
            velocity: 0.0,
            direction: 0.0,
            timestamp: 0.0,
        }
    }

    pub fn with_pressure(mut self, pressure: f64) -> Self {
        self.pressure = pressure;
        self
    }

    pub fn with_tilt(mut self, x: f64, y: f64) -> Self {
        self.tilt_x = x;
        self.tilt_y = y;
        self
    }

    pub fn with_velocity(mut self, velocity: f64) -> Self {
        self.velocity = velocity;
        self
    }

    pub fn with_direction(mut self, direction: f64) -> Self {
        self.direction = direction;
        self
    }

    pub fn with_timestamp(mut self, timestamp: f64) -> Self {
        self.timestamp = timestamp;
        self
    }
}

pub struct BrushEngine {
    current_stroke: Option<BrushStroke>,
    last_point: Option<BrushPoint>,
    velocity_buffer: Vec<f64>,
    velocity_buffer_size: usize,
}

impl BrushEngine {
    pub fn new() -> Self {
        Self {
            current_stroke: None,
            last_point: None,
            velocity_buffer: Vec::new(),
            velocity_buffer_size: 5,
        }
    }

    pub fn start_stroke(&mut self, settings: BrushSettings, point: BrushPoint) {
        self.current_stroke = Some(BrushStroke::new(settings));
        self.last_point = Some(point.clone());
        self.velocity_buffer.clear();
        
        if let Some(stroke) = &mut self.current_stroke {
            stroke.add_point(point);
            stroke.is_drawing = true;
        }
    }

    pub fn add_point(&mut self, point: BrushPoint) {
        if let Some(stroke) = &mut self.current_stroke {
            let velocity = if let Some(last) = &self.last_point {
                let dist = last.position.distance_to(&point.position);
                let time = (point.timestamp - last.timestamp).max(0.001);
                dist / time
            } else {
                0.0
            };

            self.velocity_buffer.push(velocity);
            if self.velocity_buffer.len() > self.velocity_buffer_size {
                self.velocity_buffer.remove(0);
            }

            let avg_velocity = self.velocity_buffer.iter().sum::<f64>() 
                / self.velocity_buffer.len().max(1) as f64;
            let normalized_velocity = (avg_velocity / 1000.0).min(1.0);

            let direction = if let Some(last) = &self.last_point {
                (point.position.y - last.position.y).atan2(point.position.x - last.position.x)
            } else {
                0.0
            };

            let smoothed_point = BrushPoint {
                position: point.position,
                pressure: point.pressure,
                tilt_x: point.tilt_x,
                tilt_y: point.tilt_y,
                velocity: normalized_velocity,
                direction,
                timestamp: point.timestamp,
            };

            stroke.add_point(smoothed_point.clone());
            self.last_point = Some(smoothed_point);
        }
    }

    pub fn end_stroke(&mut self) -> Option<BrushStroke> {
        if let Some(mut stroke) = self.current_stroke.take() {
            stroke.finish();
            stroke.smooth(stroke.settings.smoothing);
            Some(stroke)
        } else {
            None
        }
    }

    pub fn cancel_stroke(&mut self) {
        self.current_stroke = None;
        self.last_point = None;
        self.velocity_buffer.clear();
    }

    pub fn is_drawing(&self) -> bool {
        self.current_stroke.as_ref().map(|s| s.is_drawing).unwrap_or(false)
    }

    pub fn current_stroke(&self) -> Option<&BrushStroke> {
        self.current_stroke.as_ref()
    }
}

impl Default for BrushEngine {
    fn default() -> Self {
        Self::new()
    }
}
