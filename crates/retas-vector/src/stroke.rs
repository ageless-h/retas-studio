use serde::{Deserialize, Serialize};
use retas_core::{Point, Color8};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StrokeCap {
    Butt,
    Round,
    Square,
}

impl Default for StrokeCap {
    fn default() -> Self {
        StrokeCap::Round
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StrokeJoin {
    Miter,
    Round,
    Bevel,
}

impl Default for StrokeJoin {
    fn default() -> Self {
        StrokeJoin::Round
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BrushType {
    Round,
    Flat,
    Calligraphy,
    Custom(u32),
}

impl Default for BrushType {
    fn default() -> Self {
        BrushType::Round
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrokeStyle {
    pub color: Color8,
    pub width: f64,
    pub opacity: f64,
    pub cap: StrokeCap,
    pub join: StrokeJoin,
    pub miter_limit: f64,
    pub dash_pattern: Vec<f64>,
    pub dash_offset: f64,
}

impl StrokeStyle {
    pub fn new(color: Color8, width: f64) -> Self {
        Self {
            color,
            width,
            opacity: 1.0,
            cap: StrokeCap::Round,
            join: StrokeJoin::Round,
            miter_limit: 4.0,
            dash_pattern: Vec::new(),
            dash_offset: 0.0,
        }
    }

    pub fn with_opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn with_cap(mut self, cap: StrokeCap) -> Self {
        self.cap = cap;
        self
    }

    pub fn with_join(mut self, join: StrokeJoin) -> Self {
        self.join = join;
        self
    }
}

impl Default for StrokeStyle {
    fn default() -> Self {
        Self::new(Color8::BLACK, 1.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressurePoint {
    pub position: Point,
    pub pressure: f64,
    pub tilt: Option<(f64, f64)>,
    pub timestamp: f64,
}

impl PressurePoint {
    pub fn new(position: Point, pressure: f64) -> Self {
        Self {
            position,
            pressure,
            tilt: None,
            timestamp: 0.0,
        }
    }

    pub fn with_tilt(mut self, x: f64, y: f64) -> Self {
        self.tilt = Some((x, y));
        self
    }

    pub fn with_timestamp(mut self, ts: f64) -> Self {
        self.timestamp = ts;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stroke {
    pub points: Vec<PressurePoint>,
    pub style: StrokeStyle,
    pub brush_type: BrushType,
    pub brush_rotation: f64,
    pub spacing: f64,
    pub smoothed: bool,
}

impl Stroke {
    pub fn new(style: StrokeStyle) -> Self {
        Self {
            points: Vec::new(),
            style,
            brush_type: BrushType::Round,
            brush_rotation: 0.0,
            spacing: 0.05,
            smoothed: false,
        }
    }

    pub fn add_point(&mut self, point: PressurePoint) {
        self.points.push(point);
    }

    pub fn simplify(&mut self, tolerance: f64) {
        if self.points.len() < 3 {
            return;
        }

        let mut simplified = Vec::new();
        simplified.push(self.points[0].clone());

        self.ramer_douglas_peucker(0, self.points.len() - 1, tolerance, &mut simplified);

        simplified.push(self.points.last().unwrap().clone());
        simplified.sort_by_key(|p| p.timestamp as i64);
        simplified.dedup_by(|a, b| (a.position.x - b.position.x).abs() < 0.001 
            && (a.position.y - b.position.y).abs() < 0.001);
        
        self.points = simplified;
        self.smoothed = true;
    }

    fn ramer_douglas_peucker(&self, start: usize, end: usize, tolerance: f64, result: &mut Vec<PressurePoint>) {
        if end <= start + 1 {
            return;
        }

        let mut max_dist = 0.0;
        let mut max_idx = start;

        let start_point = &self.points[start];
        let end_point = &self.points[end];

        for i in (start + 1)..end {
            let dist = self.point_line_distance(&self.points[i].position, &start_point.position, &end_point.position);
            if dist > max_dist {
                max_dist = dist;
                max_idx = i;
            }
        }

        if max_dist > tolerance {
            self.ramer_douglas_peucker(start, max_idx, tolerance, result);
            result.push(self.points[max_idx].clone());
            self.ramer_douglas_peucker(max_idx, end, tolerance, result);
        }
    }

    fn point_line_distance(&self, point: &Point, line_start: &Point, line_end: &Point) -> f64 {
        let dx = line_end.x - line_start.x;
        let dy = line_end.y - line_start.y;
        let len_sq = dx * dx + dy * dy;

        if len_sq == 0.0 {
            return point.distance_to(line_start);
        }

        let t = ((point.x - line_start.x) * dx + (point.y - line_start.y) * dy / len_sq).max(0.0).min(1.0);
        let proj_x = line_start.x + t * dx;
        let proj_y = line_start.y + t * dy;

        point.distance_to(&Point::new(proj_x, proj_y))
    }

    pub fn interpolate_width(&self, index: usize, t: f64) -> f64 {
        if self.points.is_empty() {
            return self.style.width;
        }

        let i = index.min(self.points.len() - 1);
        let j = (i + 1).min(self.points.len() - 1);

        let p1 = &self.points[i];
        let p2 = &self.points[j];

        let w1 = self.style.width * p1.pressure;
        let w2 = self.style.width * p2.pressure;

        w1 + (w2 - w1) * t
    }
}
