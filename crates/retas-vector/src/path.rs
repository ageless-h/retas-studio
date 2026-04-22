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
                PathCommand::ArcTo { end, .. } => {
                    min_x = min_x.min(end.x);
                    min_y = min_y.min(end.y);
                    max_x = max_x.max(end.x);
                    max_y = max_y.max(end.y);
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
                PathCommand::ArcTo { .. } => {}
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
