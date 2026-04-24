use retas_core::{Point, Color8};
use crate::{Path, Stroke, BezierCurve};

#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: Point,
    pub normal: Point,
    pub color: Color8,
    pub uv: (f32, f32),
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    pub fn add_triangle(&mut self, v0: Vertex, v1: Vertex, v2: Vertex) {
        let base = self.vertices.len() as u32;
        self.vertices.extend_from_slice(&[v0, v1, v2]);
        self.indices.extend_from_slice(&[base, base + 1, base + 2]);
    }
}

impl Default for Mesh {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Tessellator {
    tolerance: f64,
}

impl Tessellator {
    pub fn new() -> Self {
        Self { tolerance: 0.5 }
    }

    pub fn with_tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance = tolerance;
        self
    }

    pub fn tessellate_stroke(&self, stroke: &Stroke) -> Mesh {
        let mut mesh = Mesh::new();

        if stroke.points.len() < 2 {
            return mesh;
        }

        let flattened = self.flatten_stroke(stroke);

        for i in 0..flattened.len() - 1 {
            let p1 = &flattened[i];
            let p2 = &flattened[i + 1];

            let dx = p2.position.x - p1.position.x;
            let dy = p2.position.y - p1.position.y;
            let len = (dx * dx + dy * dy).sqrt();

            if len < 0.001 {
                continue;
            }

            let nx = -dy / len;
            let ny = dx / len;

            let w1 = stroke.style.width * p1.pressure / 2.0;
            let w2 = stroke.style.width * p2.pressure / 2.0;

            let v0 = Vertex {
                position: Point::new(p1.position.x + nx * w1, p1.position.y + ny * w1),
                normal: Point::new(nx, ny),
                color: stroke.style.color,
                uv: (0.0, 0.0),
            };
            let v1 = Vertex {
                position: Point::new(p1.position.x - nx * w1, p1.position.y - ny * w1),
                normal: Point::new(-nx, -ny),
                color: stroke.style.color,
                uv: (1.0, 0.0),
            };
            let v2 = Vertex {
                position: Point::new(p2.position.x + nx * w2, p2.position.y + ny * w2),
                normal: Point::new(nx, ny),
                color: stroke.style.color,
                uv: (0.0, 1.0),
            };
            let v3 = Vertex {
                position: Point::new(p2.position.x - nx * w2, p2.position.y - ny * w2),
                normal: Point::new(-nx, -ny),
                color: stroke.style.color,
                uv: (1.0, 1.0),
            };

            let base = mesh.vertices.len() as u32;
            mesh.vertices.extend_from_slice(&[v0, v1, v2, v3]);
            mesh.indices.extend_from_slice(&[base, base + 1, base + 2, base + 1, base + 3, base + 2]);
        }

        mesh
    }

    pub fn flatten_stroke(&self, stroke: &Stroke) -> Vec<crate::PressurePoint> {
        if stroke.points.len() < 2 {
            return stroke.points.clone();
        }

        let mut result = Vec::new();
        result.push(stroke.points[0].clone());

        for i in 0..stroke.points.len() - 1 {
            let start = &stroke.points[i];
            let end = &stroke.points[i + 1];
            let (c1, c2) = self.catmull_rom_controls(i, stroke);
            self.flatten_segment(start.position, c1, c2, end.position, start, end, &mut result);
        }

        result
    }

    fn flatten_segment(
        &self,
        p0: Point,
        p1: Point,
        p2: Point,
        p3: Point,
        start_pt: &crate::PressurePoint,
        end_pt: &crate::PressurePoint,
        result: &mut Vec<crate::PressurePoint>,
    ) {
        let curve_mid = cubic_bezier(p0, p1, p2, p3, 0.5);
        let line_mid = Point::new((p0.x + p3.x) / 2.0, (p0.y + p3.y) / 2.0);
        let deviation = curve_mid.distance_to(&line_mid);

        if deviation > self.tolerance {
            let (left, right) = split_cubic(p0, p1, p2, p3, 0.5);

            let mid_pressure = (start_pt.pressure + end_pt.pressure) / 2.0;
            let mid_tilt = match (start_pt.tilt, end_pt.tilt) {
                (Some((tx1, ty1)), Some((tx2, ty2))) => {
                    Some(((tx1 + tx2) / 2.0, (ty1 + ty2) / 2.0))
                }
                _ => start_pt.tilt.or(end_pt.tilt),
            };
            let mid_timestamp = (start_pt.timestamp + end_pt.timestamp) / 2.0;

            let mid_pt = crate::PressurePoint {
                position: curve_mid,
                pressure: mid_pressure,
                tilt: mid_tilt,
                timestamp: mid_timestamp,
            };

            self.flatten_segment(left.0, left.1, left.2, left.3, start_pt, &mid_pt, result);
            self.flatten_segment(right.0, right.1, right.2, right.3, &mid_pt, end_pt, result);
        } else {
            result.push(end_pt.clone());
        }
    }

    fn catmull_rom_controls(&self, index: usize, stroke: &Stroke) -> (Point, Point) {
        let n = stroke.points.len();
        let p1 = stroke.points[index].position;
        let p2 = stroke.points[index + 1].position;

        let p0 = if index > 0 {
            stroke.points[index - 1].position
        } else {
            Point::new(2.0 * p1.x - p2.x, 2.0 * p1.y - p2.y)
        };

        let p3 = if index + 2 < n {
            stroke.points[index + 2].position
        } else {
            Point::new(2.0 * p2.x - p1.x, 2.0 * p2.y - p1.y)
        };

        let c1 = Point::new(p1.x + (p2.x - p0.x) / 6.0, p1.y + (p2.y - p0.y) / 6.0);
        let c2 = Point::new(p2.x - (p3.x - p1.x) / 6.0, p2.y - (p3.y - p1.y) / 6.0);

        (c1, c2)
    }

    pub fn tessellate_path_fill(&self, path: &Path) -> Mesh {
        let mut mesh = Mesh::new();
        let curves = path.to_bezier_curves();

        for curve in curves {
            self.tessellate_curve_fill(&curve, &mut mesh);
        }

        mesh
    }

    fn tessellate_curve_fill(&self, curve: &BezierCurve, mesh: &mut Mesh) {
        let points = curve.flatten(self.tolerance);

        if points.len() < 3 {
            return;
        }

        let center = points.iter().fold(Point::ZERO, |acc, p| Point::new(acc.x + p.x, acc.y + p.y));
        let center = Point::new(center.x / points.len() as f64, center.y / points.len() as f64);

        let center_vertex = Vertex {
            position: center,
            normal: Point::ZERO,
            color: Color8::WHITE,
            uv: (0.5, 0.5),
        };

        let base = mesh.vertices.len() as u32;
        mesh.vertices.push(center_vertex);

        for point in &points {
            mesh.vertices.push(Vertex {
                position: *point,
                normal: Point::ZERO,
                color: Color8::WHITE,
                uv: (point.x as f32, point.y as f32),
            });
        }

        let n = points.len() as u32;
        for i in 0..n {
            mesh.indices.push(base);
            mesh.indices.push(base + 1 + i);
            mesh.indices.push(base + 1 + (i + 1) % n);
        }
    }

    pub fn tessellate_curve(&self, curve: &BezierCurve, width: f64) -> Mesh {
        let mut mesh = Mesh::new();
        let points = curve.flatten(self.tolerance);

        if points.len() < 2 {
            return mesh;
        }

        for i in 0..points.len() - 1 {
            let p1 = &points[i];
            let p2 = &points[i + 1];

            let dx = p2.x - p1.x;
            let dy = p2.y - p1.y;
            let len = (dx * dx + dy * dy).sqrt();

            if len < 0.001 {
                continue;
            }

            let nx = -dy / len;
            let ny = dx / len;
            let hw = width / 2.0;

            let v0 = Vertex {
                position: Point::new(p1.x + nx * hw, p1.y + ny * hw),
                normal: Point::new(nx, ny),
                color: Color8::WHITE,
                uv: (0.0, 0.0),
            };
            let v1 = Vertex {
                position: Point::new(p1.x - nx * hw, p1.y - ny * hw),
                normal: Point::new(-nx, -ny),
                color: Color8::WHITE,
                uv: (1.0, 0.0),
            };
            let v2 = Vertex {
                position: Point::new(p2.x + nx * hw, p2.y + ny * hw),
                normal: Point::new(nx, ny),
                color: Color8::WHITE,
                uv: (0.0, 1.0),
            };
            let v3 = Vertex {
                position: Point::new(p2.x - nx * hw, p2.y - ny * hw),
                normal: Point::new(-nx, -ny),
                color: Color8::WHITE,
                uv: (1.0, 1.0),
            };

            let base = mesh.vertices.len() as u32;
            mesh.vertices.extend_from_slice(&[v0, v1, v2, v3]);
            mesh.indices.extend_from_slice(&[base, base + 1, base + 2, base + 1, base + 3, base + 2]);
        }

        mesh
    }
}

impl Default for Tessellator {
    fn default() -> Self {
        Self::new()
    }
}

fn cubic_bezier(p0: Point, p1: Point, p2: Point, p3: Point, t: f64) -> Point {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;

    Point::new(
        mt3 * p0.x + 3.0 * mt2 * t * p1.x + 3.0 * mt * t2 * p2.x + t3 * p3.x,
        mt3 * p0.y + 3.0 * mt2 * t * p1.y + 3.0 * mt * t2 * p2.y + t3 * p3.y,
    )
}

fn split_cubic(p0: Point, p1: Point, p2: Point, p3: Point, t: f64) -> ((Point, Point, Point, Point), (Point, Point, Point, Point)) {
    let p01 = p0.lerp(&p1, t);
    let p12 = p1.lerp(&p2, t);
    let p23 = p2.lerp(&p3, t);
    let p012 = p01.lerp(&p12, t);
    let p123 = p12.lerp(&p23, t);
    let p0123 = p012.lerp(&p123, t);

    ((p0, p01, p012, p0123), (p0123, p123, p23, p3))
}

pub fn tessellate_stroke(stroke: &Stroke) -> Mesh {
    Tessellator::new().tessellate_stroke(stroke)
}

pub fn tessellate_path(path: &Path) -> Mesh {
    Tessellator::new().tessellate_path_fill(path)
}

pub fn tessellate_curve(curve: &BezierCurve, width: f64) -> Mesh {
    Tessellator::new().tessellate_curve(curve, width)
}
