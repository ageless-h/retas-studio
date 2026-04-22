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

            let w1 = stroke.interpolate_width(i, 0.0) / 2.0;
            let w2 = stroke.interpolate_width(i + 1, 0.0) / 2.0;

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

    fn flatten_stroke(&self, stroke: &Stroke) -> Vec<crate::PressurePoint> {
        let mut result = Vec::new();

        for point in &stroke.points {
            result.push(point.clone());
        }

        result
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

pub fn tessellate_stroke(stroke: &Stroke) -> Mesh {
    Tessellator::new().tessellate_stroke(stroke)
}

pub fn tessellate_path(path: &Path) -> Mesh {
    Tessellator::new().tessellate_path_fill(path)
}

pub fn tessellate_curve(curve: &BezierCurve, width: f64) -> Mesh {
    Tessellator::new().tessellate_curve(curve, width)
}
