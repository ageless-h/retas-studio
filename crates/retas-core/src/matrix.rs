use serde::{Deserialize, Serialize};
use crate::Point;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Matrix2D {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub tx: f64,
    pub ty: f64,
}

impl Matrix2D {
    pub const IDENTITY: Matrix2D = Matrix2D {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        tx: 0.0,
        ty: 0.0,
    };

    pub const ZERO: Matrix2D = Matrix2D {
        a: 0.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
        tx: 0.0,
        ty: 0.0,
    };

    pub fn new(a: f64, b: f64, c: f64, d: f64, tx: f64, ty: f64) -> Self {
        Self { a, b, c, d, tx, ty }
    }

    pub fn identity() -> Self {
        Self::IDENTITY
    }

    pub fn translation(tx: f64, ty: f64) -> Self {
        Self::new(1.0, 0.0, 0.0, 1.0, tx, ty)
    }

    pub fn scaling(sx: f64, sy: f64) -> Self {
        Self::new(sx, 0.0, 0.0, sy, 0.0, 0.0)
    }

    pub fn rotation(angle_radians: f64) -> Self {
        let cos = angle_radians.cos();
        let sin = angle_radians.sin();
        Self::new(cos, sin, -sin, cos, 0.0, 0.0)
    }

    pub fn rotation_degrees(angle_degrees: f64) -> Self {
        Self::rotation(angle_degrees.to_radians())
    }

    pub fn multiply(&self, other: &Matrix2D) -> Matrix2D {
        Matrix2D::new(
            self.a * other.a + self.b * other.c,
            self.a * other.b + self.b * other.d,
            self.c * other.a + self.d * other.c,
            self.c * other.b + self.d * other.d,
            self.tx * other.a + self.ty * other.c + other.tx,
            self.tx * other.b + self.ty * other.d + other.ty,
        )
    }

    pub fn transform_point(&self, point: &Point) -> Point {
        Point::new(
            self.a * point.x + self.c * point.y + self.tx,
            self.b * point.x + self.d * point.y + self.ty,
        )
    }

    pub fn determinant(&self) -> f64 {
        self.a * self.d - self.b * self.c
    }

    pub fn is_invertible(&self) -> bool {
        self.determinant() != 0.0
    }

    pub fn inverse(&self) -> Option<Matrix2D> {
        let det = self.determinant();
        if det == 0.0 {
            return None;
        }

        let inv_det = 1.0 / det;
        Some(Matrix2D::new(
            self.d * inv_det,
            -self.b * inv_det,
            -self.c * inv_det,
            self.a * inv_det,
            (self.c * self.ty - self.d * self.tx) * inv_det,
            (self.b * self.tx - self.a * self.ty) * inv_det,
        ))
    }

    pub fn translate(&mut self, tx: f64, ty: f64) {
        self.tx += self.a * tx + self.c * ty;
        self.ty += self.b * tx + self.d * ty;
    }

    pub fn scale(&mut self, sx: f64, sy: f64) {
        self.a *= sx;
        self.b *= sx;
        self.c *= sy;
        self.d *= sy;
    }

    pub fn rotate(&mut self, angle_radians: f64) {
        let cos = angle_radians.cos();
        let sin = angle_radians.sin();
        let new_a = self.a * cos + self.c * sin;
        let new_b = self.b * cos + self.d * sin;
        let new_c = self.c * cos - self.a * sin;
        let new_d = self.d * cos - self.b * sin;
        self.a = new_a;
        self.b = new_b;
        self.c = new_c;
        self.d = new_d;
    }

    pub fn to_array(&self) -> [f64; 6] {
        [self.a, self.b, self.c, self.d, self.tx, self.ty]
    }

    pub fn to_row_major_3x3(&self) -> [f64; 9] {
        [
            self.a, self.c, self.tx,
            self.b, self.d, self.ty,
            0.0, 0.0, 1.0,
        ]
    }

    pub fn to_column_major_3x3(&self) -> [f64; 9] {
        [
            self.a, self.b, 0.0,
            self.c, self.d, 0.0,
            self.tx, self.ty, 1.0,
        ]
    }
}

impl Default for Matrix2D {
    fn default() -> Self {
        Self::IDENTITY
    }
}
