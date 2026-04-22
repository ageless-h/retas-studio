use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Eq for Point {}

impl std::hash::Hash for Point {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
    }
}

impl Point {
    pub const ZERO: Point = Point { x: 0.0, y: 0.0 };

    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Point) -> f64 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn midpoint(&self, other: &Point) -> Point {
        Point::new((self.x + other.x) / 2.0, (self.y + other.y) / 2.0)
    }

    pub fn lerp(&self, other: &Point, t: f64) -> Point {
        Point::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Size {
    pub const ZERO: Size = Size { width: 0.0, height: 0.0 };

    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    pub fn area(&self) -> f64 {
        self.width * self.height
    }

    pub fn aspect_ratio(&self) -> f64 {
        if self.height == 0.0 {
            return 0.0;
        }
        self.width / self.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub const ZERO: Rect = Rect {
        origin: Point::ZERO,
        size: Size::ZERO,
    };

    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            origin: Point::new(x, y),
            size: Size::new(width, height),
        }
    }

    pub fn from_points(p1: Point, p2: Point) -> Self {
        let x = p1.x.min(p2.x);
        let y = p1.y.min(p2.y);
        let width = (p2.x - p1.x).abs();
        let height = (p2.y - p1.y).abs();
        Self::new(x, y, width, height)
    }

    pub fn left(&self) -> f64 {
        self.origin.x
    }

    pub fn top(&self) -> f64 {
        self.origin.y
    }

    pub fn right(&self) -> f64 {
        self.origin.x + self.size.width
    }

    pub fn bottom(&self) -> f64 {
        self.origin.y + self.size.height
    }

    pub fn center(&self) -> Point {
        Point::new(
            self.origin.x + self.size.width / 2.0,
            self.origin.y + self.size.height / 2.0,
        )
    }

    pub fn contains(&self, point: &Point) -> bool {
        point.x >= self.left()
            && point.x <= self.right()
            && point.y >= self.top()
            && point.y <= self.bottom()
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.left() < other.right()
            && self.right() > other.left()
            && self.top() < other.bottom()
            && self.bottom() > other.top()
    }

    pub fn union(&self, other: &Rect) -> Rect {
        let x = self.left().min(other.left());
        let y = self.top().min(other.top());
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        Rect::new(x, y, right - x, bottom - y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector2D {
    pub dx: f64,
    pub dy: f64,
}

impl Vector2D {
    pub const ZERO: Vector2D = Vector2D { dx: 0.0, dy: 0.0 };

    pub fn new(dx: f64, dy: f64) -> Self {
        Self { dx, dy }
    }

    pub fn from_points(from: &Point, to: &Point) -> Self {
        Self::new(to.x - from.x, to.y - from.y)
    }

    pub fn length(&self) -> f64 {
        (self.dx * self.dx + self.dy * self.dy).sqrt()
    }

    pub fn normalize(&self) -> Vector2D {
        let len = self.length();
        if len == 0.0 {
            return Vector2D::ZERO;
        }
        Vector2D::new(self.dx / len, self.dy / len)
    }

    pub fn dot(&self, other: &Vector2D) -> f64 {
        self.dx * other.dx + self.dy * other.dy
    }

    pub fn cross(&self, other: &Vector2D) -> f64 {
        self.dx * other.dy - self.dy * other.dx
    }
}
