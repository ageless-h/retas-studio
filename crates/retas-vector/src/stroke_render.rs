use retas_core::{Point, Color8, Rect};
use retas_core::advanced::{BrushSettings, BrushPoint, BrushStroke, BrushType};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStroke {
    pub id: u64,
    pub points: Vec<VectorPoint>,
    pub color: Color8,
    pub width: f64,
    pub opacity: f64,
    pub cap: StrokeCap,
    pub join: StrokeJoin,
    pub miter_limit: f64,
    pub closed: bool,
    pub brush_type: BrushType,
}

impl VectorStroke {
    pub fn new(color: Color8, width: f64) -> Self {
        Self {
            id: rand_id(),
            points: Vec::new(),
            color,
            width,
            opacity: 1.0,
            cap: StrokeCap::Round,
            join: StrokeJoin::Round,
            miter_limit: 4.0,
            closed: false,
            brush_type: BrushType::Round,
        }
    }

    pub fn from_brush_stroke(stroke: &BrushStroke) -> Self {
        let points: Vec<VectorPoint> = stroke.points.iter().map(|p| VectorPoint {
            position: p.position,
            pressure: p.pressure,
            tilt: (p.tilt_x, p.tilt_y),
            direction: p.direction,
            width: stroke.settings.calculate_size(p.pressure, p.velocity),
            opacity: stroke.settings.calculate_opacity(p.pressure, p.velocity),
        }).collect();

        Self {
            id: rand_id(),
            points,
            color: stroke.settings.color,
            width: stroke.settings.size,
            opacity: stroke.settings.opacity,
            cap: StrokeCap::Round,
            join: StrokeJoin::Round,
            miter_limit: 4.0,
            closed: false,
            brush_type: stroke.settings.brush_type,
        }
    }

    pub fn add_point(&mut self, point: VectorPoint) {
        self.points.push(point);
    }

    pub fn close(&mut self) {
        self.closed = true;
    }

    pub fn bounds(&self) -> Option<Rect> {
        if self.points.is_empty() {
            return None;
        }

        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for p in &self.points {
            let hw = p.width / 2.0 + 1.0;
            min_x = min_x.min(p.position.x - hw);
            min_y = min_y.min(p.position.y - hw);
            max_x = max_x.max(p.position.x + hw);
            max_y = max_y.max(p.position.y + hw);
        }

        Some(Rect::new(min_x, min_y, max_x - min_x, max_y - min_y))
    }

    pub fn length(&self) -> f64 {
        if self.points.len() < 2 {
            return 0.0;
        }

        let mut length = 0.0;
        for i in 0..self.points.len() - 1 {
            length += self.points[i].position.distance_to(&self.points[i + 1].position);
        }
        length
    }

    pub fn simplify(&mut self, tolerance: f64) {
        if self.points.len() < 3 {
            return;
        }

        let mut result = vec![self.points[0].clone()];
        self.ramer_douglas_peucker(0, self.points.len() - 1, tolerance, &mut result);
        result.push(self.points.last().expect("simplify checked len >= 3").clone());
        self.points = result;
    }

    fn ramer_douglas_peucker(&self, start: usize, end: usize, tolerance: f64, result: &mut Vec<VectorPoint>) {
        if end <= start + 1 {
            return;
        }

        let mut max_dist = 0.0;
        let mut max_idx = start;

        let start_point = &self.points[start];
        let end_point = &self.points[end];

        for i in (start + 1)..end {
            let dist = point_line_distance(&self.points[i].position, &start_point.position, &end_point.position);
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

    pub fn to_triangles(&self) -> Vec<Triangle> {
        if self.points.len() < 2 {
            return Vec::new();
        }

        let mut triangles = Vec::new();
        
        for i in 0..self.points.len() - 1 {
            let p0 = &self.points[i];
            let p1 = &self.points[i + 1];
            
            let dx = p1.position.x - p0.position.x;
            let dy = p1.position.y - p0.position.y;
            let len = (dx * dx + dy * dy).sqrt();
            
            if len < 0.001 {
                continue;
            }
            
            let nx = -dy / len;
            let ny = dx / len;
            
            let w0 = p0.width / 2.0;
            let w1 = p1.width / 2.0;
            
            let v0 = Point::new(p0.position.x + nx * w0, p0.position.y + ny * w0);
            let v1 = Point::new(p0.position.x - nx * w0, p0.position.y - ny * w0);
            let v2 = Point::new(p1.position.x + nx * w1, p1.position.y + ny * w1);
            let v3 = Point::new(p1.position.x - nx * w1, p1.position.y - ny * w1);
            
            let c0 = Vertex {
                position: v0,
                color: self.color,
                opacity: p0.opacity * self.opacity,
                u: 0.0,
                v: 0.0,
            };
            let c1 = Vertex {
                position: v1,
                color: self.color,
                opacity: p0.opacity * self.opacity,
                u: 1.0,
                v: 0.0,
            };
            let c2 = Vertex {
                position: v2,
                color: self.color,
                opacity: p1.opacity * self.opacity,
                u: 0.0,
                v: 1.0,
            };
            let c3 = Vertex {
                position: v3,
                color: self.color,
                opacity: p1.opacity * self.opacity,
                u: 1.0,
                v: 1.0,
            };
            
            triangles.push(Triangle { v0: c0.clone(), v1: c1.clone(), v2: c2.clone() });
            triangles.push(Triangle { v0: c1, v1: c3, v2: c2 });
        }
        
        if self.closed && self.points.len() >= 2 {
            let p0 = self.points.last().expect("len >= 2 guarantees last");
            let p1 = &self.points[0];
            
            let dx = p1.position.x - p0.position.x;
            let dy = p1.position.y - p0.position.y;
            let len = (dx * dx + dy * dy).sqrt();
            
            if len >= 0.001 {
                let nx = -dy / len;
                let ny = dx / len;
                
                let w0 = p0.width / 2.0;
                let w1 = p1.width / 2.0;
                
                let v0 = Point::new(p0.position.x + nx * w0, p0.position.y + ny * w0);
                let v1 = Point::new(p0.position.x - nx * w0, p0.position.y - ny * w0);
                let v2 = Point::new(p1.position.x + nx * w1, p1.position.y + ny * w1);
                let v3 = Point::new(p1.position.x - nx * w1, p1.position.y - ny * w1);
                
                let c0 = Vertex { position: v0, color: self.color, opacity: p0.opacity * self.opacity, u: 0.0, v: 0.0 };
                let c1 = Vertex { position: v1, color: self.color, opacity: p0.opacity * self.opacity, u: 1.0, v: 0.0 };
                let c2 = Vertex { position: v2, color: self.color, opacity: p1.opacity * self.opacity, u: 0.0, v: 1.0 };
                let c3 = Vertex { position: v3, color: self.color, opacity: p1.opacity * self.opacity, u: 1.0, v: 1.0 };
                
                triangles.push(Triangle { v0: c0.clone(), v1: c1.clone(), v2: c2.clone() });
                triangles.push(Triangle { v0: c1, v1: c3, v2: c2 });
            }
        }

        triangles
    }

    pub fn rasterize(&self, width: u32, height: u32) -> Vec<u8> {
        let mut buffer = vec![0u8; (width * height * 4) as usize];
        let triangles = self.to_triangles();
        
        for tri in triangles {
            tri.rasterize(&mut buffer, width, height);
        }
        
        buffer
    }
}

fn rand_id() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

fn point_line_distance(point: &Point, line_start: &Point, line_end: &Point) -> f64 {
    let dx = line_end.x - line_start.x;
    let dy = line_end.y - line_start.y;
    let len_sq = dx * dx + dy * dy;

    if len_sq == 0.0 {
        return point.distance_to(line_start);
    }

    let t = ((point.x - line_start.x) * dx + (point.y - line_start.y) * dy / len_sq)
        .max(0.0)
        .min(1.0);
    let proj_x = line_start.x + t * dx;
    let proj_y = line_start.y + t * dy;

    point.distance_to(&Point::new(proj_x, proj_y))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorPoint {
    pub position: Point,
    pub pressure: f64,
    pub tilt: (f64, f64),
    pub direction: f64,
    pub width: f64,
    pub opacity: f64,
}

impl VectorPoint {
    pub fn new(position: Point) -> Self {
        Self {
            position,
            pressure: 1.0,
            tilt: (0.0, 0.0),
            direction: 0.0,
            width: 1.0,
            opacity: 1.0,
        }
    }

    pub fn with_pressure(mut self, pressure: f64) -> Self {
        self.pressure = pressure;
        self
    }

    pub fn with_width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vertex {
    pub position: Point,
    pub color: Color8,
    pub opacity: f64,
    pub u: f64,
    pub v: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Triangle {
    pub v0: Vertex,
    pub v1: Vertex,
    pub v2: Vertex,
}

impl Triangle {
    pub fn rasterize(&self, buffer: &mut [u8], width: u32, height: u32) {
        let min_x = self.v0.position.x.min(self.v1.position.x).min(self.v2.position.x).floor() as i32;
        let min_y = self.v0.position.y.min(self.v1.position.y).min(self.v2.position.y).floor() as i32;
        let max_x = self.v0.position.x.max(self.v1.position.x).max(self.v2.position.x).ceil() as i32;
        let max_y = self.v0.position.y.max(self.v1.position.y).max(self.v2.position.y).ceil() as i32;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
                    continue;
                }

                let p = Point::new(x as f64 + 0.5, y as f64 + 0.5);
                
                if let Some((u, v, w)) = self.barycentric(&p) {
                    if u >= 0.0 && v >= 0.0 && w >= 0.0 {
                        let idx = (y as u32 * width + x as u32) as usize * 4;
                        
                        if idx + 3 < buffer.len() {
                            let opacity = self.v0.opacity * u + self.v1.opacity * v + self.v2.opacity * w;
                            let alpha = (opacity * 255.0) as u8;
                            
                            let existing_alpha = buffer[idx + 3] as f64 / 255.0;
                            let new_alpha = alpha as f64 / 255.0;
                            let out_alpha = new_alpha + existing_alpha * (1.0 - new_alpha);
                            
                            if out_alpha > 0.0 {
                                buffer[idx] = self.v0.color.r;
                                buffer[idx + 1] = self.v0.color.g;
                                buffer[idx + 2] = self.v0.color.b;
                                buffer[idx + 3] = (out_alpha * 255.0) as u8;
                            }
                        }
                    }
                }
            }
        }
    }

    fn barycentric(&self, p: &Point) -> Option<(f64, f64, f64)> {
        let v0 = self.v0.position;
        let v1 = self.v1.position;
        let v2 = self.v2.position;

        let denom = (v1.y - v2.y) * (v0.x - v2.x) + (v2.x - v1.x) * (v0.y - v2.y);
        
        if denom.abs() < 0.0001 {
            return None;
        }

        let u = ((v1.y - v2.y) * (p.x - v2.x) + (v2.x - v1.x) * (p.y - v2.y)) / denom;
        let v = ((v2.y - v0.y) * (p.x - v2.x) + (v0.x - v2.x) * (p.y - v2.y)) / denom;
        let w = 1.0 - u - v;

        Some((u, v, w))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorLayerData {
    pub strokes: Vec<VectorStroke>,
    pub fill_color: Option<Color8>,
    pub fill_opacity: f64,
}

impl VectorLayerData {
    pub fn new() -> Self {
        Self {
            strokes: Vec::new(),
            fill_color: None,
            fill_opacity: 1.0,
        }
    }

    pub fn add_stroke(&mut self, stroke: VectorStroke) {
        self.strokes.push(stroke);
    }

    pub fn remove_stroke(&mut self, id: u64) -> Option<VectorStroke> {
        let pos = self.strokes.iter().position(|s| s.id == id)?;
        Some(self.strokes.remove(pos))
    }

    pub fn bounds(&self) -> Option<Rect> {
        let mut result: Option<Rect> = None;
        
        for stroke in &self.strokes {
            if let Some(stroke_bounds) = stroke.bounds() {
                result = Some(match result {
                    Some(r) => r.union(&stroke_bounds),
                    None => stroke_bounds,
                });
            }
        }
        
        result
    }

    pub fn rasterize(&self, width: u32, height: u32) -> Vec<u8> {
        let mut buffer = vec![0u8; (width * height * 4) as usize];
        
        for stroke in &self.strokes {
            let stroke_buffer = stroke.rasterize(width, height);
            
            for i in (0..buffer.len()).step_by(4) {
                let bg_r = buffer[i] as f64 / 255.0;
                let bg_g = buffer[i + 1] as f64 / 255.0;
                let bg_b = buffer[i + 2] as f64 / 255.0;
                let bg_a = buffer[i + 3] as f64 / 255.0;
                
                let fg_r = stroke_buffer[i] as f64 / 255.0;
                let fg_g = stroke_buffer[i + 1] as f64 / 255.0;
                let fg_b = stroke_buffer[i + 2] as f64 / 255.0;
                let fg_a = stroke_buffer[i + 3] as f64 / 255.0;
                
                let out_a = fg_a + bg_a * (1.0 - fg_a);
                
                if out_a > 0.0 {
                    buffer[i] = ((fg_r * fg_a + bg_r * bg_a * (1.0 - fg_a)) / out_a * 255.0) as u8;
                    buffer[i + 1] = ((fg_g * fg_a + bg_g * bg_a * (1.0 - fg_a)) / out_a * 255.0) as u8;
                    buffer[i + 2] = ((fg_b * fg_a + bg_b * bg_a * (1.0 - fg_a)) / out_a * 255.0) as u8;
                    buffer[i + 3] = (out_a * 255.0) as u8;
                }
            }
        }
        
        buffer
    }
}

impl Default for VectorLayerData {
    fn default() -> Self {
        Self::new()
    }
}
