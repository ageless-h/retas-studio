use serde::{Deserialize, Serialize};
use crate::{Point, Color8};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VanishingPoint {
    pub id: u64,
    pub position: Point,
    pub color: Color8,
    pub visible: bool,
    pub locked: bool,
    pub name: String,
}

impl VanishingPoint {
    pub fn new(position: Point) -> Self {
        Self {
            id: 0,
            position,
            color: Color8::new(255, 0, 255, 255),
            visible: true,
            locked: false,
            name: String::new(),
        }
    }

    pub fn with_color(position: Point, color: Color8) -> Self {
        Self {
            id: 0,
            position,
            color,
            visible: true,
            locked: false,
            name: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerspectiveGuide {
    pub id: u64,
    pub vanishing_points: Vec<VanishingPoint>,
    pub horizon_line: Option<HorizonLine>,
    pub grid_enabled: bool,
    pub grid_spacing: f64,
    pub visible: bool,
}

impl PerspectiveGuide {
    pub fn one_point() -> Self {
        Self {
            id: 0,
            vanishing_points: vec![VanishingPoint::new(Point::new(0.0, 0.0))],
            horizon_line: None,
            grid_enabled: true,
            grid_spacing: 50.0,
            visible: true,
        }
    }

    pub fn two_point() -> Self {
        Self {
            id: 0,
            vanishing_points: vec![
                VanishingPoint::new(Point::new(-500.0, 0.0)),
                VanishingPoint::new(Point::new(500.0, 0.0)),
            ],
            horizon_line: Some(HorizonLine::default()),
            grid_enabled: true,
            grid_spacing: 50.0,
            visible: true,
        }
    }

    pub fn three_point() -> Self {
        Self {
            id: 0,
            vanishing_points: vec![
                VanishingPoint::new(Point::new(-500.0, 0.0)),
                VanishingPoint::new(Point::new(500.0, 0.0)),
                VanishingPoint::new(Point::new(0.0, -500.0)),
            ],
            horizon_line: Some(HorizonLine::default()),
            grid_enabled: true,
            grid_spacing: 50.0,
            visible: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HorizonLine {
    pub y: f64,
    pub visible: bool,
    pub color: Color8,
}

impl Default for HorizonLine {
    fn default() -> Self {
        Self {
            y: 0.0,
            visible: true,
            color: Color8::new(100, 100, 255, 255),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuideLayer {
    pub id: u64,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub horizontal_guides: Vec<HorizontalGuide>,
    pub vertical_guides: Vec<VerticalGuide>,
    pub perspective_guides: Vec<PerspectiveGuide>,
}

impl GuideLayer {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: 0,
            name: name.into(),
            visible: true,
            locked: false,
            horizontal_guides: Vec::new(),
            vertical_guides: Vec::new(),
            perspective_guides: Vec::new(),
        }
    }

    pub fn add_horizontal_guide(&mut self, y: f64) -> u64 {
        let id = self.next_guide_id();
        self.horizontal_guides.push(HorizontalGuide {
            id,
            y,
            visible: true,
            color: Color8::new(0, 255, 255, 255),
        });
        id
    }

    pub fn add_vertical_guide(&mut self, x: f64) -> u64 {
        let id = self.next_guide_id();
        self.vertical_guides.push(VerticalGuide {
            id,
            x,
            visible: true,
            color: Color8::new(0, 255, 255, 255),
        });
        id
    }

    pub fn add_one_point_perspective(&mut self, vanishing_point: Point) -> u64 {
        let id = self.next_guide_id();
        let mut guide = PerspectiveGuide::one_point();
        guide.id = id;
        guide.vanishing_points[0].position = vanishing_point;
        self.perspective_guides.push(guide);
        id
    }

    pub fn add_two_point_perspective(&mut self, vp1: Point, vp2: Point, horizon_y: f64) -> u64 {
        let id = self.next_guide_id();
        let mut guide = PerspectiveGuide::two_point();
        guide.id = id;
        guide.vanishing_points[0].position = vp1;
        guide.vanishing_points[1].position = vp2;
        if let Some(ref mut horizon) = guide.horizon_line {
            horizon.y = horizon_y;
        }
        self.perspective_guides.push(guide);
        id
    }

    pub fn remove_guide(&mut self, id: u64) -> bool {
        let h_len = self.horizontal_guides.len();
        let v_len = self.vertical_guides.len();
        let p_len = self.perspective_guides.len();
        
        self.horizontal_guides.retain(|g| g.id != id);
        self.vertical_guides.retain(|g| g.id != id);
        self.perspective_guides.retain(|g| g.id != id);
        
        self.horizontal_guides.len() != h_len
            || self.vertical_guides.len() != v_len
            || self.perspective_guides.len() != p_len
    }

    fn next_guide_id(&self) -> u64 {
        let mut max_id = 0u64;
        for g in &self.horizontal_guides {
            max_id = max_id.max(g.id);
        }
        for g in &self.vertical_guides {
            max_id = max_id.max(g.id);
        }
        for g in &self.perspective_guides {
            max_id = max_id.max(g.id);
        }
        max_id + 1
    }

    pub fn get_perspective_lines(&self, canvas_width: f64, canvas_height: f64) -> Vec<PerspectiveLine> {
        let mut lines = Vec::new();
        
        for guide in &self.perspective_guides {
            if !guide.visible {
                continue;
            }
            
            if let Some(ref horizon) = guide.horizon_line {
                if horizon.visible {
                    lines.push(PerspectiveLine {
                        start: Point::new(0.0, horizon.y),
                        end: Point::new(canvas_width, horizon.y),
                        color: horizon.color,
                        line_type: PerspectiveLineType::Horizon,
                    });
                }
            }
            
            for vp in &guide.vanishing_points {
                if !vp.visible {
                    continue;
                }
                
                let num_lines = 12;
                for i in 0..num_lines {
                    let angle = std::f64::consts::PI * 2.0 * (i as f64) / (num_lines as f64);
                    let length = (canvas_width.powi(2) + canvas_height.powi(2)).sqrt();
                    
                    let end_x = vp.position.x + length * angle.cos();
                    let end_y = vp.position.y + length * angle.sin();
                    
                    lines.push(PerspectiveLine {
                        start: vp.position,
                        end: Point::new(end_x, end_y),
                        color: vp.color,
                        line_type: PerspectiveLineType::Radial,
                    });
                }
            }
        }
        
        lines
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HorizontalGuide {
    pub id: u64,
    pub y: f64,
    pub visible: bool,
    pub color: Color8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerticalGuide {
    pub id: u64,
    pub x: f64,
    pub visible: bool,
    pub color: Color8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerspectiveLine {
    pub start: Point,
    pub end: Point,
    pub color: Color8,
    pub line_type: PerspectiveLineType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PerspectiveLineType {
    Horizon,
    Radial,
    Grid,
}
