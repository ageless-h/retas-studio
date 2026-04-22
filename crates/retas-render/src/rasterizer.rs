use retas_core::{Color8, Point};
use retas_core::advanced::{BrushSettings, BrushPoint, BrushType, BrushBlendMode};
use std::f64::consts::PI;

#[derive(Debug, Clone)]
pub struct StrokeRasterizer {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl StrokeRasterizer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0u8; (width * height * 4) as usize],
        }
    }

    pub fn from_rgba(width: u32, height: u32, data: &[u8]) -> Self {
        Self {
            width,
            height,
            pixels: data.to_vec(),
        }
    }

    pub fn clear(&mut self, color: Option<Color8>) {
        let bg = color.unwrap_or(Color8::TRANSPARENT);
        for pixel in self.pixels.chunks_exact_mut(4) {
            pixel[0] = bg.r;
            pixel[1] = bg.g;
            pixel[2] = bg.b;
            pixel[3] = bg.a;
        }
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn pixels_mut(&mut self) -> &mut [u8] {
        &mut self.pixels
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color8) {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            let idx = ((y as u32 * self.width + x as u32) * 4) as usize;
            if idx + 3 < self.pixels.len() {
                self.pixels[idx] = color.r;
                self.pixels[idx + 1] = color.g;
                self.pixels[idx + 2] = color.b;
                self.pixels[idx + 3] = color.a;
            }
        }
    }

    pub fn blend_pixel(&mut self, x: i32, y: i32, color: Color8, opacity: f64, mode: BrushBlendMode) {
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
            return;
        }

        let idx = ((y as u32 * self.width + x as u32) * 4) as usize;
        if idx + 3 >= self.pixels.len() {
            return;
        }

        let dst_r = self.pixels[idx] as f64 / 255.0;
        let dst_g = self.pixels[idx + 1] as f64 / 255.0;
        let dst_b = self.pixels[idx + 2] as f64 / 255.0;
        let dst_a = self.pixels[idx + 3] as f64 / 255.0;

        let src_r = color.r as f64 / 255.0;
        let src_g = color.g as f64 / 255.0;
        let src_b = color.b as f64 / 255.0;
        let src_a = (color.a as f64 / 255.0) * opacity;

        let (out_r, out_g, out_b, out_a) = match mode {
            BrushBlendMode::Normal => {
                let a = src_a + dst_a * (1.0 - src_a);
                if a < 0.001 {
                    (0.0, 0.0, 0.0, 0.0)
                } else {
                    let r = (src_r * src_a + dst_r * dst_a * (1.0 - src_a)) / a;
                    let g = (src_g * src_a + dst_g * dst_a * (1.0 - src_a)) / a;
                    let b = (src_b * src_a + dst_b * dst_a * (1.0 - src_a)) / a;
                    (r, g, b, a)
                }
            }
            BrushBlendMode::Multiply => {
                let a = src_a + dst_a * (1.0 - src_a);
                (
                    src_r * dst_r,
                    src_g * dst_g,
                    src_b * dst_b,
                    a
                )
            }
            BrushBlendMode::Screen => {
                let a = src_a + dst_a * (1.0 - src_a);
                (
                    1.0 - (1.0 - src_r) * (1.0 - dst_r),
                    1.0 - (1.0 - src_g) * (1.0 - dst_g),
                    1.0 - (1.0 - src_b) * (1.0 - dst_b),
                    a
                )
            }
            BrushBlendMode::Overlay => {
                let a = src_a + dst_a * (1.0 - src_a);
                let blend = |s: f64, d: f64| -> f64 {
                    if d < 0.5 {
                        2.0 * s * d
                    } else {
                        1.0 - 2.0 * (1.0 - s) * (1.0 - d)
                    }
                };
                (blend(src_r, dst_r), blend(src_g, dst_g), blend(src_b, dst_b), a)
            }
            BrushBlendMode::Darken => {
                let a = src_a + dst_a * (1.0 - src_a);
                (src_r.min(dst_r), src_g.min(dst_g), src_b.min(dst_b), a)
            }
            BrushBlendMode::Lighten => {
                let a = src_a + dst_a * (1.0 - src_a);
                (src_r.max(dst_r), src_g.max(dst_g), src_b.max(dst_b), a)
            }
            _ => {
                let a = src_a + dst_a * (1.0 - src_a);
                if a < 0.001 {
                    (0.0, 0.0, 0.0, 0.0)
                } else {
                    let r = (src_r * src_a + dst_r * dst_a * (1.0 - src_a)) / a;
                    let g = (src_g * src_a + dst_g * dst_a * (1.0 - src_a)) / a;
                    let b = (src_b * src_a + dst_b * dst_a * (1.0 - src_a)) / a;
                    (r, g, b, a)
                }
            }
        };

        self.pixels[idx] = (out_r.clamp(0.0, 1.0) * 255.0) as u8;
        self.pixels[idx + 1] = (out_g.clamp(0.0, 1.0) * 255.0) as u8;
        self.pixels[idx + 2] = (out_b.clamp(0.0, 1.0) * 255.0) as u8;
        self.pixels[idx + 3] = (out_a.clamp(0.0, 1.0) * 255.0) as u8;
    }

    pub fn draw_circle(&mut self, cx: f64, cy: f64, radius: f64, color: Color8, opacity: f64, hardness: f64, mode: BrushBlendMode) {
        let r = radius.max(0.5);
        let x_start = (cx - r - 1.0).floor() as i32;
        let x_end = (cx + r + 1.0).ceil() as i32;
        let y_start = (cy - r - 1.0).floor() as i32;
        let y_end = (cy + r + 1.0).ceil() as i32;

        for y in y_start..=y_end {
            for x in x_start..=x_end {
                let dx = x as f64 - cx;
                let dy = y as f64 - cy;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist <= r {
                    let edge_fade = if hardness >= 0.999 {
                        if dist <= r { 1.0 } else { 0.0 }
                    } else {
                        let soft_edge = r * (1.0 - hardness);
                        if dist < r - soft_edge {
                            1.0
                        } else {
                            1.0 - (dist - (r - soft_edge)) / soft_edge
                        }
                    };

                    let final_opacity = opacity * edge_fade;
                    if final_opacity > 0.001 {
                        self.blend_pixel(x, y, color, final_opacity, mode);
                    }
                }
            }
        }
    }

    pub fn draw_ellipse(&mut self, cx: f64, cy: f64, rx: f64, ry: f64, angle: f64, color: Color8, opacity: f64, hardness: f64, mode: BrushBlendMode) {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        let max_r = rx.max(ry);
        let x_start = (cx - max_r - 1.0).floor() as i32;
        let x_end = (cx + max_r + 1.0).ceil() as i32;
        let y_start = (cy - max_r - 1.0).floor() as i32;
        let y_end = (cy + max_r + 1.0).ceil() as i32;

        for y in y_start..=y_end {
            for x in x_start..=x_end {
                let dx = x as f64 - cx;
                let dy = y as f64 - cy;

                let rx_rot = dx * cos_a + dy * sin_a;
                let ry_rot = -dx * sin_a + dy * cos_a;

                let dist = ((rx_rot / rx).powi(2) + (ry_rot / ry).powi(2)).sqrt();

                if dist <= 1.0 {
                    let edge_fade = if hardness >= 0.999 {
                        if dist <= 1.0 { 1.0 } else { 0.0 }
                    } else {
                        let soft_edge = 1.0 - hardness;
                        if dist < 1.0 - soft_edge {
                            1.0
                        } else {
                            1.0 - (dist - (1.0 - soft_edge)) / soft_edge
                        }
                    };

                    let final_opacity = opacity * edge_fade;
                    if final_opacity > 0.001 {
                        self.blend_pixel(x, y, color, final_opacity, mode);
                    }
                }
            }
        }
    }

    pub fn rasterize_stroke(&mut self, points: &[BrushPoint], settings: &BrushSettings) {
        if points.is_empty() {
            return;
        }

        let mode = settings.blend_mode;
        let color = settings.color;

        for i in 0..points.len() {
            let point = &points[i];
            let size = settings.calculate_size(point.pressure, point.velocity);
            let opacity = settings.calculate_opacity(point.pressure, point.velocity);
            let angle = settings.calculate_angle(point.direction);

            let (rx, ry) = match settings.brush_type {
                BrushType::Flat => (size / 2.0, size / 4.0),
                BrushType::Calligraphy => (size / 2.0, size / 6.0),
                BrushType::Airbrush => (size / 2.0, size / 2.0),
                BrushType::Pencil => (size / 2.0 * 0.8, size / 2.0 * 0.8),
                BrushType::Marker => (size / 2.0, size / 2.0),
                _ => (size / 2.0, size / 2.0),
            };

            if settings.brush_type == BrushType::Round || settings.brush_type == BrushType::Pencil {
                self.draw_circle(
                    point.position.x,
                    point.position.y,
                    rx,
                    color,
                    opacity,
                    settings.hardness,
                    mode,
                );
            } else {
                self.draw_ellipse(
                    point.position.x,
                    point.position.y,
                    rx,
                    ry,
                    angle * PI / 180.0,
                    color,
                    opacity,
                    settings.hardness,
                    mode,
                );
            }

            if i > 0 && settings.spacing > 0.0 {
                let prev = &points[i - 1];
                let dist = point.position.distance_to(&prev.position);
                let spacing_dist = settings.spacing * size;

                if dist > spacing_dist {
                    let steps = (dist / spacing_dist).ceil() as usize;
                    for j in 1..steps {
                        let t = j as f64 / steps as f64;
                        let interp_point = BrushPoint {
                            position: Point::new(
                                prev.position.x + (point.position.x - prev.position.x) * t,
                                prev.position.y + (point.position.y - prev.position.y) * t,
                            ),
                            pressure: prev.pressure + (point.pressure - prev.pressure) * t,
                            velocity: prev.velocity + (point.velocity - prev.velocity) * t,
                            direction: prev.direction,
                            tilt_x: prev.tilt_x + (point.tilt_x - prev.tilt_x) * t,
                            tilt_y: prev.tilt_y + (point.tilt_y - prev.tilt_y) * t,
                            timestamp: prev.timestamp + (point.timestamp - prev.timestamp) * t,
                        };

                        let int_size = settings.calculate_size(interp_point.pressure, interp_point.velocity);
                        let int_opacity = settings.calculate_opacity(interp_point.pressure, interp_point.velocity);

                        self.draw_circle(
                            interp_point.position.x,
                            interp_point.position.y,
                            int_size / 2.0,
                            color,
                            int_opacity,
                            settings.hardness,
                            mode,
                        );
                    }
                }
            }
        }
    }

    pub fn flood_fill(&mut self, start_x: i32, start_y: i32, fill_color: Color8, tolerance: f64) {
        if start_x < 0 || start_x >= self.width as i32 || start_y < 0 || start_y >= self.height as i32 {
            return;
        }

        let idx = ((start_y as u32 * self.width + start_x as u32) * 4) as usize;
        let target_r = self.pixels[idx];
        let target_g = self.pixels[idx + 1];
        let target_b = self.pixels[idx + 2];
        let target_a = self.pixels[idx + 3];

        let target = Color8::new(target_r, target_g, target_b, target_a);
        if Self::color_matches(&target, &fill_color, tolerance) {
            return;
        }

        let mut stack = vec![(start_x, start_y)];
        let mut visited = vec![false; (self.width * self.height) as usize];

        while let Some((x, y)) = stack.pop() {
            if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
                continue;
            }

            let pixel_idx = (y as u32 * self.width + x as u32) as usize;
            if visited[pixel_idx] {
                continue;
            }

            let idx = pixel_idx * 4;
            let current = Color8::new(self.pixels[idx], self.pixels[idx + 1], self.pixels[idx + 2], self.pixels[idx + 3]);

            if !Self::color_matches(&current, &target, tolerance) {
                continue;
            }

            visited[pixel_idx] = true;
            self.pixels[idx] = fill_color.r;
            self.pixels[idx + 1] = fill_color.g;
            self.pixels[idx + 2] = fill_color.b;
            self.pixels[idx + 3] = fill_color.a;

            stack.push((x + 1, y));
            stack.push((x - 1, y));
            stack.push((x, y + 1));
            stack.push((x, y - 1));
        }
    }

    fn color_matches(c1: &Color8, c2: &Color8, tolerance: f64) -> bool {
        let dr = (c1.r as i32 - c2.r as i32).abs() as f64;
        let dg = (c1.g as i32 - c2.g as i32).abs() as f64;
        let db = (c1.b as i32 - c2.b as i32).abs() as f64;
        let da = (c1.a as i32 - c2.a as i32).abs() as f64;

        let diff = (dr + dg + db + da) / (4.0 * 255.0);
        diff <= tolerance
    }

    pub fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, color: Color8, width: f64, opacity: f64) {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let dist = (dx * dx + dy * dy).sqrt();
        
        if dist < 0.001 {
            self.draw_circle(x1, y1, width / 2.0, color, opacity, 0.8, BrushBlendMode::Normal);
            return;
        }

        let steps = (dist / (width * 0.5)).ceil() as usize;

        for i in 0..=steps {
            let t = i as f64 / steps as f64;
            let x = x1 + dx * t;
            let y = y1 + dy * t;
            self.draw_circle(x, y, width / 2.0, color, opacity, 0.8, BrushBlendMode::Normal);
        }
    }

    pub fn copy_to(&self, target: &mut StrokeRasterizer, src_x: i32, src_y: i32, dst_x: i32, dst_y: i32, w: u32, h: u32) {
        for y in 0..h {
            let sy = src_y + y as i32;
            let dy = dst_y + y as i32;

            if sy < 0 || sy >= self.height as i32 || dy < 0 || dy >= target.height as i32 {
                continue;
            }

            for x in 0..w {
                let sx = src_x + x as i32;
                let dx = dst_x + x as i32;

                if sx < 0 || sx >= self.width as i32 || dx < 0 || dx >= target.width as i32 {
                    continue;
                }

                let src_idx = ((sy as u32 * self.width + sx as u32) * 4) as usize;
                let dst_idx = ((dy as u32 * target.width + dx as u32) * 4) as usize;

                target.pixels[dst_idx] = self.pixels[src_idx];
                target.pixels[dst_idx + 1] = self.pixels[src_idx + 1];
                target.pixels[dst_idx + 2] = self.pixels[src_idx + 2];
                target.pixels[dst_idx + 3] = self.pixels[src_idx + 3];
            }
        }
    }
}
