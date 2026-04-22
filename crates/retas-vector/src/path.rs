use serde::{Deserialize, Serialize};
use retas_core::{Point, Rect};
use crate::BezierCurve;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FillRule {
    NonZero,
    EvenOdd,
}

impl Default for FillRule {
    fn default() -> Self {
        FillRule::NonZero
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PathCommand {
    MoveTo(Point),
    LineTo(Point),
    CurveTo(Point, Point, Point),
    QuadTo(Point, Point),
    ArcTo {
        radii: (f64, f64),
        rotation: f64,
        large_arc: bool,
        sweep: bool,
        end: Point,
    },
    Close,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Path {
    pub commands: Vec<PathCommand>,
    pub fill_rule: FillRule,
}

impl Path {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            fill_rule: FillRule::NonZero,
        }
    }

    pub fn move_to(mut self, point: Point) -> Self {
        self.commands.push(PathCommand::MoveTo(point));
        self
    }

    pub fn line_to(mut self, point: Point) -> Self {
        self.commands.push(PathCommand::LineTo(point));
        self
    }

    pub fn curve_to(mut self, ctrl1: Point, ctrl2: Point, end: Point) -> Self {
        self.commands.push(PathCommand::CurveTo(ctrl1, ctrl2, end));
        self
    }

    pub fn quad_to(mut self, ctrl: Point, end: Point) -> Self {
        self.commands.push(PathCommand::QuadTo(ctrl, end));
        self
    }

    pub fn arc_to(
        mut self,
        rx: f64,
        ry: f64,
        rotation: f64,
        large_arc: bool,
        sweep: bool,
        end: Point,
    ) -> Self {
        self.commands.push(PathCommand::ArcTo {
            radii: (rx, ry),
            rotation,
            large_arc,
            sweep,
            end,
        });
        self
    }

    pub fn close(mut self) -> Self {
        self.commands.push(PathCommand::Close);
        self
    }

    pub fn rect(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self::new()
            .move_to(Point::new(x, y))
            .line_to(Point::new(x + width, y))
            .line_to(Point::new(x + width, y + height))
            .line_to(Point::new(x, y + height))
            .close()
    }

    pub fn ellipse(cx: f64, cy: f64, rx: f64, ry: f64) -> Self {
        Self::new()
            .move_to(Point::new(cx + rx, cy))
            .arc_to(rx, ry, 0.0, false, true, Point::new(cx - rx, cy))
            .arc_to(rx, ry, 0.0, false, true, Point::new(cx + rx, cy))
            .close()
    }

    pub fn circle(cx: f64, cy: f64, r: f64) -> Self {
        Self::ellipse(cx, cy, r, r)
    }

    pub fn bounds(&self) -> Option<Rect> {
        let mut current = Point::ZERO;
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;
        let mut has_points = false;

        for cmd in &self.commands {
            match cmd {
                PathCommand::MoveTo(p) | PathCommand::LineTo(p) => {
                    current = *p;
                    min_x = min_x.min(p.x);
                    min_y = min_y.min(p.y);
                    max_x = max_x.max(p.x);
                    max_y = max_y.max(p.y);
                    has_points = true;
                }
                PathCommand::CurveTo(c1, c2, end) => {
                    min_x = min_x.min(c1.x).min(c2.x).min(end.x);
                    min_y = min_y.min(c1.y).min(c2.y).min(end.y);
                    max_x = max_x.max(c1.x).max(c2.x).max(end.x);
                    max_y = max_y.max(c1.y).max(c2.y).max(end.y);
                    current = *end;
                    has_points = true;
                }
                PathCommand::QuadTo(ctrl, end) => {
                    min_x = min_x.min(ctrl.x).min(end.x);
                    min_y = min_y.min(ctrl.y).min(end.y);
                    max_x = max_x.max(ctrl.x).max(end.x);
                    max_y = max_y.max(ctrl.y).max(end.y);
                    current = *end;
                    has_points = true;
                }
                PathCommand::ArcTo { radii, rotation, large_arc, sweep, end } => {
                    let center = Point::new(
                        (current.x + end.x) / 2.0,
                        (current.y + end.y) / 2.0,
                    );
                    min_x = min_x.min(center.x - radii.0).min(end.x);
                    min_y = min_y.min(center.y - radii.1).min(end.y);
                    max_x = max_x.max(center.x + radii.0).max(end.x);
                    max_y = max_y.max(center.y + radii.1).max(end.y);
                    current = *end;
                    has_points = true;
                }
                PathCommand::Close => {}
            }
        }

        if has_points {
            Some(Rect::new(min_x, min_y, max_x - min_x, max_y - min_y))
        } else {
            None
        }
    }

    pub fn to_bezier_curves(&self) -> Vec<BezierCurve> {
        let mut curves = Vec::new();
        let mut current_curve = BezierCurve::new();
        let mut current = Point::ZERO;

        for cmd in &self.commands {
            match cmd {
                PathCommand::MoveTo(p) => {
                    if !current_curve.points.is_empty() {
                        curves.push(current_curve.clone());
                    }
                    current_curve = BezierCurve::new();
                    current_curve.add_point(crate::BezierControlPoint::corner(*p));
                    current = *p;
                }
                PathCommand::LineTo(p) => {
                    current_curve.add_point(crate::BezierControlPoint::corner(*p));
                    current = *p;
                }
                PathCommand::CurveTo(c1, c2, end) => {
                    if let Some(last) = current_curve.points.last_mut() {
                        last.out_handle = Some(*c1);
                    }
                    current_curve.add_point(crate::BezierControlPoint::smooth(*end, *c2, Point::ZERO));
                    current = *end;
                }
                PathCommand::QuadTo(ctrl, end) => {
                    let c1 = Point::new(
                        current.x + 2.0 * (ctrl.x - current.x) / 3.0,
                        current.y + 2.0 * (ctrl.y - current.y) / 3.0,
                    );
                    let c2 = Point::new(
                        end.x + 2.0 * (ctrl.x - end.x) / 3.0,
                        end.y + 2.0 * (ctrl.y - end.y) / 3.0,
                    );
                    if let Some(last) = current_curve.points.last_mut() {
                        last.out_handle = Some(c1);
                    }
                    current_curve.add_point(crate::BezierControlPoint::smooth(*end, c2, Point::ZERO));
                    current = *end;
                }
                PathCommand::ArcTo { radii, rotation, large_arc, sweep, end } => {
                    let arc_curves = arc_to_bezier(current, *end, *radii, *rotation, *large_arc, *sweep);
                    for (c1, c2, p) in arc_curves {
                        if let Some(last) = current_curve.points.last_mut() {
                            last.out_handle = Some(c1);
                        }
                        current_curve.add_point(crate::BezierControlPoint::smooth(p, c2, Point::ZERO));
                        current = p;
                    }
                }
                PathCommand::Close => {
                    current_curve.close();
                }
            }
        }

        if !current_curve.points.is_empty() {
            curves.push(current_curve);
        }

        curves
    }
}

impl Default for Path {
    fn default() -> Self {
        Self::new()
    }
}

fn arc_to_bezier(
    start: Point,
    end: Point,
    radii: (f64, f64),
    rotation: f64,
    large_arc: bool,
    sweep: bool,
) -> Vec<(Point, Point, Point)> {
    let (rx, ry) = radii;
    if rx == 0.0 || ry == 0.0 {
        return vec![(start, end, end)];
    }

    let phi = rotation.to_radians();
    let cos_phi = phi.cos();
    let sin_phi = phi.sin();

    let dx2 = (start.x - end.x) / 2.0;
    let dy2 = (start.y - end.y) / 2.0;

    let x1p = cos_phi * dx2 + sin_phi * dy2;
    let y1p = -sin_phi * dx2 + cos_phi * dy2;

    let rx_sq = rx * rx;
    let ry_sq = ry * ry;
    let x1p_sq = x1p * x1p;
    let y1p_sq = y1p * y1p;

    let mut radii_check = x1p_sq / rx_sq + y1p_sq / ry_sq;
    let (rx, ry) = if radii_check > 1.0 {
        let scale = radii_check.sqrt();
        (rx * scale, ry * scale)
    } else {
        (rx, ry)
    };

    let rx_sq = rx * rx;
    let ry_sq = ry * ry;

    let mut sign = -1.0f64;
    if large_arc == sweep {
        sign = 1.0;
    }

    let sq = ((rx_sq * ry_sq - rx_sq * y1p_sq - ry_sq * x1p_sq) / (rx_sq * y1p_sq + ry_sq * x1p_sq)).max(0.0);
    let coeff = sign * sq.sqrt();

    let cxp = coeff * (rx * y1p / ry);
    let cyp = coeff * -(ry * x1p / rx);

    let cx = cos_phi * cxp - sin_phi * cyp + (start.x + end.x) / 2.0;
    let cy = sin_phi * cxp + cos_phi * cyp + (start.y + end.y) / 2.0;

    let ux = (x1p - cxp) / rx;
    let uy = (y1p - cyp) / ry;
    let vx = (-x1p - cxp) / rx;
    let vy = (-y1p - cyp) / ry;

    let mut start_angle = uy.atan2(ux);
    let mut delta_angle = vx.atan2(vy) - start_angle;

    if !sweep && delta_angle > 0.0 {
        delta_angle -= 2.0 * std::f64::consts::PI;
    } else if sweep && delta_angle < 0.0 {
        delta_angle += 2.0 * std::f64::consts::PI;
    }

    let num_segments = ((delta_angle.abs() / (std::f64::consts::PI / 2.0)).ceil() as usize).max(1);
    let segment_angle = delta_angle / num_segments as f64;

    let mut curves = Vec::new();
    let mut current_angle = start_angle;

    for i in 0..num_segments {
        let next_angle = current_angle + segment_angle;
        let curve = arc_segment_to_bezier(cx, cy, rx, ry, phi, current_angle, next_angle);
        curves.push(curve);
        current_angle = next_angle;
    }

    curves
}

fn arc_segment_to_bezier(
    cx: f64,
    cy: f64,
    rx: f64,
    ry: f64,
    phi: f64,
    start_angle: f64,
    end_angle: f64,
) -> (Point, Point, Point) {
    let cos_phi = phi.cos();
    let sin_phi = phi.sin();

    let alpha = (end_angle - start_angle) / 2.0;
    let cos_alpha = alpha.cos();
    let sin_alpha = alpha.sin();

    let kappa = (4.0 / 3.0) * ((1.0 - cos_alpha) / sin_alpha);

    let cos_start = start_angle.cos();
    let sin_start = start_angle.sin();
    let cos_end = end_angle.cos();
    let sin_end = end_angle.sin();

    let x1 = cx + rx * cos_phi * cos_start - ry * sin_phi * sin_start;
    let y1 = cy + rx * sin_phi * cos_start + ry * cos_phi * sin_start;

    let x2 = cx + rx * cos_phi * cos_end - ry * sin_phi * sin_end;
    let y2 = cy + rx * sin_phi * cos_end + ry * cos_phi * sin_end;

    let dx1 = -rx * cos_phi * sin_start - ry * sin_phi * cos_start;
    let dy1 = -rx * sin_phi * sin_start + ry * cos_phi * cos_start;

    let dx2 = -rx * cos_phi * sin_end - ry * sin_phi * cos_end;
    let dy2 = -rx * sin_phi * sin_end + ry * cos_phi * cos_end;

    let c1x = x1 + kappa * dx1;
    let c1y = y1 + kappa * dy1;

    let c2x = x2 - kappa * dx2;
    let c2y = y2 - kappa * dy2;

    (Point::new(c1x, c1y), Point::new(c2x, c2y), Point::new(x2, y2))
}
