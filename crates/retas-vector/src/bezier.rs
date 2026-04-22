use serde::{Deserialize, Serialize};
use retas_core::{Point, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BezierPointType {
    Corner,
    Smooth,
    Symmetric,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BezierControlPoint {
    pub point: Point,
    pub in_handle: Option<Point>,
    pub out_handle: Option<Point>,
    pub point_type: BezierPointType,
}

impl BezierControlPoint {
    pub fn corner(point: Point) -> Self {
        Self {
            point,
            in_handle: None,
            out_handle: None,
            point_type: BezierPointType::Corner,
        }
    }

    pub fn smooth(point: Point, in_handle: Point, out_handle: Point) -> Self {
        Self {
            point,
            in_handle: Some(in_handle),
            out_handle: Some(out_handle),
            point_type: BezierPointType::Smooth,
        }
    }

    pub fn symmetric(point: Point, handle_offset: Point) -> Self {
        Self {
            point,
            in_handle: Some(Point::new(
                point.x - handle_offset.x,
                point.y - handle_offset.y,
            )),
            out_handle: Some(Point::new(
                point.x + handle_offset.x,
                point.y + handle_offset.y,
            )),
            point_type: BezierPointType::Symmetric,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BezierCurve {
    pub points: Vec<BezierControlPoint>,
    pub closed: bool,
}

impl BezierCurve {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            closed: false,
        }
    }

    pub fn from_points(points: Vec<Point>) -> Self {
        Self {
            points: points.into_iter().map(BezierControlPoint::corner).collect(),
            closed: false,
        }
    }

    pub fn closed(points: Vec<Point>) -> Self {
        Self {
            points: points.into_iter().map(BezierControlPoint::corner).collect(),
            closed: true,
        }
    }

    pub fn add_point(&mut self, point: BezierControlPoint) {
        self.points.push(point);
    }

    pub fn close(&mut self) {
        self.closed = true;
    }

    pub fn open(&mut self) {
        self.closed = false;
    }

    pub fn bounds(&self) -> Option<Rect> {
        if self.points.is_empty() {
            return None;
        }

        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for cp in &self.points {
            min_x = min_x.min(cp.point.x);
            min_y = min_y.min(cp.point.y);
            max_x = max_x.max(cp.point.x);
            max_y = max_y.max(cp.point.y);

            if let Some(h) = cp.in_handle {
                min_x = min_x.min(h.x);
                min_y = min_y.min(h.y);
                max_x = max_x.max(h.x);
                max_y = max_y.max(h.y);
            }
            if let Some(h) = cp.out_handle {
                min_x = min_x.min(h.x);
                min_y = min_y.min(h.y);
                max_x = max_x.max(h.x);
                max_y = max_y.max(h.y);
            }
        }

        Some(Rect::new(min_x, min_y, max_x - min_x, max_y - min_y))
    }

    pub fn evaluate(&self, t: f64) -> Option<Point> {
        if self.points.len() < 2 {
            return self.points.first().map(|cp| cp.point);
        }

        let n = self.points.len();
        let segment_count = if self.closed { n } else { n - 1 };
        let total_t = t * segment_count as f64;
        let segment_idx = total_t.floor() as usize;
        let local_t = total_t - segment_idx as f64;

        let i0 = segment_idx % n;
        let i1 = (segment_idx + 1) % n;

        let p0 = self.points[i0].point;
        let p1 = self.points[i0].out_handle.unwrap_or(p0);
        let p2 = self.points[i1].in_handle.unwrap_or(self.points[i1].point);
        let p3 = self.points[i1].point;

        Some(self.cubic_bezier(p0, p1, p2, p3, local_t))
    }

    fn cubic_bezier(&self, p0: Point, p1: Point, p2: Point, p3: Point, t: f64) -> Point {
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

    pub fn flatten(&self, tolerance: f64) -> Vec<Point> {
        if self.points.len() < 2 {
            return self.points.iter().map(|cp| cp.point).collect();
        }

        let mut result = Vec::new();
        let n = self.points.len();
        let segments = if self.closed { n } else { n - 1 };

        for i in 0..segments {
            let i0 = i;
            let i1 = (i + 1) % n;

            let p0 = self.points[i0].point;
            let p1 = self.points[i0].out_handle.unwrap_or(p0);
            let p2 = self.points[i1].in_handle.unwrap_or(self.points[i1].point);
            let p3 = self.points[i1].point;

            self.flatten_segment(p0, p1, p2, p3, tolerance, &mut result);
        }

        result
    }

    fn flatten_segment(&self, p0: Point, p1: Point, p2: Point, p3: Point, tolerance: f64, result: &mut Vec<Point>) {
        result.push(p0);
        self.flatten_recursive(p0, p1, p2, p3, tolerance, result);
    }

    fn flatten_recursive(&self, p0: Point, p1: Point, p2: Point, p3: Point, tolerance: f64, result: &mut Vec<Point>) {
        let mid = self.cubic_bezier(p0, p1, p2, p3, 0.5);
        
        let expected_x = (p0.x + p3.x) / 2.0;
        let expected_y = (p0.y + p3.y) / 2.0;
        let dist = ((mid.x - expected_x).powi(2) + (mid.y - expected_y).powi(2)).sqrt();

        if dist > tolerance {
            let (left, right) = self.split_cubic(p0, p1, p2, p3, 0.5);
            self.flatten_recursive(left.0, left.1, left.2, left.3, tolerance, result);
            self.flatten_recursive(right.0, right.1, right.2, right.3, tolerance, result);
        } else {
            result.push(p3);
        }
    }

    fn split_cubic(&self, p0: Point, p1: Point, p2: Point, p3: Point, t: f64) -> ((Point, Point, Point, Point), (Point, Point, Point, Point)) {
        let p01 = p0.lerp(&p1, t);
        let p12 = p1.lerp(&p2, t);
        let p23 = p2.lerp(&p3, t);
        let p012 = p01.lerp(&p12, t);
        let p123 = p12.lerp(&p23, t);
        let p0123 = p012.lerp(&p123, t);

        ((p0, p01, p012, p0123), (p0123, p123, p23, p3))
    }
}

impl Default for BezierCurve {
    fn default() -> Self {
        Self::new()
    }
}
