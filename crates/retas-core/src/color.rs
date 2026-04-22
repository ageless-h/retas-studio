use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Color8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color8 {
    pub const TRANSPARENT: Color8 = Color8 { r: 0, g: 0, b: 0, a: 0 };
    pub const BLACK: Color8 = Color8 { r: 0, g: 0, b: 0, a: 255 };
    pub const WHITE: Color8 = Color8 { r: 255, g: 255, b: 255, a: 255 };
    pub const RED: Color8 = Color8 { r: 255, g: 0, b: 0, a: 255 };
    pub const GREEN: Color8 = Color8 { r: 0, g: 255, b: 0, a: 255 };
    pub const BLUE: Color8 = Color8 { r: 0, g: 0, b: 255, a: 255 };

    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as u8,
            g: ((hex >> 8) & 0xFF) as u8,
            b: (hex & 0xFF) as u8,
            a: 255,
        }
    }

    pub fn to_rgba_f32(&self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }

    pub fn to_rgba_f64(&self) -> [f64; 4] {
        [
            self.r as f64 / 255.0,
            self.g as f64 / 255.0,
            self.b as f64 / 255.0,
            self.a as f64 / 255.0,
        ]
    }
}

impl From<Color16> for Color8 {
    fn from(c: Color16) -> Self {
        Self {
            r: (c.r >> 8) as u8,
            g: (c.g >> 8) as u8,
            b: (c.b >> 8) as u8,
            a: (c.a >> 8) as u8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Color16 {
    pub r: u16,
    pub g: u16,
    pub b: u16,
    pub a: u16,
}

impl Color16 {
    pub const TRANSPARENT: Color16 = Color16 { r: 0, g: 0, b: 0, a: 0 };
    pub const BLACK: Color16 = Color16 { r: 0, g: 0, b: 0, a: 65535 };
    pub const WHITE: Color16 = Color16 { r: 65535, g: 65535, b: 65535, a: 65535 };

    pub fn new(r: u16, g: u16, b: u16, a: u16) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_rgb(r: u16, g: u16, b: u16) -> Self {
        Self { r, g, b, a: 65535 }
    }
}

impl From<Color8> for Color16 {
    fn from(c: Color8) -> Self {
        Self {
            r: (c.r as u16) << 8 | c.r as u16,
            g: (c.g as u16) << 8 | c.g as u16,
            b: (c.b as u16) << 8 | c.b as u16,
            a: (c.a as u16) << 8 | c.a as u16,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ColorF {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl ColorF {
    pub const TRANSPARENT: ColorF = ColorF { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    pub const BLACK: ColorF = ColorF { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const WHITE: ColorF = ColorF { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

    pub fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_rgb(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn from_hsv(h: f64, s: f64, v: f64) -> Self {
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r, g, b) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        Self::new(r + m, g + m, b + m, 1.0)
    }

    pub fn to_hsv(&self) -> (f64, f64, f64) {
        let max = self.r.max(self.g).max(self.b);
        let min = self.r.min(self.g).min(self.b);
        let delta = max - min;

        let h = if delta == 0.0 {
            0.0
        } else if max == self.r {
            60.0 * (((self.g - self.b) / delta) % 6.0)
        } else if max == self.g {
            60.0 * ((self.b - self.r) / delta + 2.0)
        } else {
            60.0 * ((self.r - self.g) / delta + 4.0)
        };

        let s = if max == 0.0 { 0.0 } else { delta / max };

        (h, s, max)
    }

    pub fn lerp(&self, other: &ColorF, t: f64) -> ColorF {
        ColorF::new(
            self.r + (other.r - self.r) * t,
            self.g + (other.g - self.g) * t,
            self.b + (other.b - self.b) * t,
            self.a + (other.a - self.a) * t,
        )
    }
}

impl From<Color8> for ColorF {
    fn from(c: Color8) -> Self {
        Self {
            r: c.r as f64 / 255.0,
            g: c.g as f64 / 255.0,
            b: c.b as f64 / 255.0,
            a: c.a as f64 / 255.0,
        }
    }
}

impl From<ColorF> for Color8 {
    fn from(c: ColorF) -> Self {
        Self {
            r: (c.r.clamp(0.0, 1.0) * 255.0).round() as u8,
            g: (c.g.clamp(0.0, 1.0) * 255.0).round() as u8,
            b: (c.b.clamp(0.0, 1.0) * 255.0).round() as u8,
            a: (c.a.clamp(0.0, 1.0) * 255.0).round() as u8,
        }
    }
}
