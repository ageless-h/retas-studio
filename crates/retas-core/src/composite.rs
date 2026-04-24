use crate::{Color8, ColorF, BlendMode};

pub fn blend_pixels(
    base: Color8,
    blend: Color8,
    mode: BlendMode,
    opacity: f64,
) -> Color8 {
    if opacity <= 0.0 {
        return base;
    }

    let base_f = ColorF::from(base);
    let blend_f = ColorF::from(blend);

    let result = match mode {
        BlendMode::Normal => blend_normal(base_f, blend_f),
        BlendMode::Multiply => blend_multiply(base_f, blend_f),
        BlendMode::Screen => blend_screen(base_f, blend_f),
        BlendMode::Overlay => blend_overlay(base_f, blend_f),
        BlendMode::Darken => blend_darken(base_f, blend_f),
        BlendMode::Lighten => blend_lighten(base_f, blend_f),
        BlendMode::ColorDodge => blend_color_dodge(base_f, blend_f),
        BlendMode::ColorBurn => blend_color_burn(base_f, blend_f),
        BlendMode::HardLight => blend_hard_light(base_f, blend_f),
        BlendMode::SoftLight => blend_soft_light(base_f, blend_f),
        BlendMode::Difference => blend_difference(base_f, blend_f),
        BlendMode::Exclusion => blend_exclusion(base_f, blend_f),
        BlendMode::Hue => blend_hue(base_f, blend_f),
        BlendMode::Saturation => blend_saturation(base_f, blend_f),
        BlendMode::Color => blend_color_mode(base_f, blend_f),
        BlendMode::Luminosity => blend_luminosity(base_f, blend_f),
    };

    let blended = blend_normal(base_f, ColorF::new(result.r, result.g, result.b, blend_f.a * opacity));
    Color8::from(blended)
}

pub fn blend_pixels_rgba(
    base: &[u8],
    blend: &[u8],
    output: &mut [u8],
    mode: BlendMode,
    opacity: f64,
) {
    let len = base.len().min(blend.len()).min(output.len());
    
    for i in (0..len).step_by(4) {
        let base_color = Color8::new(base[i], base[i + 1], base[i + 2], base[i + 3]);
        let blend_color = Color8::new(blend[i], blend[i + 1], blend[i + 2], blend[i + 3]);
        let result = blend_pixels(base_color, blend_color, mode, opacity);
        
        output[i] = result.r;
        output[i + 1] = result.g;
        output[i + 2] = result.b;
        output[i + 3] = result.a;
    }
}

fn blend_normal(base: ColorF, blend: ColorF) -> ColorF {
    let alpha = blend.a;
    let inv_alpha = 1.0 - alpha;
    
    ColorF::new(
        blend.r * alpha + base.r * inv_alpha,
        blend.g * alpha + base.g * inv_alpha,
        blend.b * alpha + base.b * inv_alpha,
        alpha + base.a * inv_alpha,
    )
}

fn blend_multiply(base: ColorF, blend: ColorF) -> ColorF {
    ColorF::new(
        base.r * blend.r,
        base.g * blend.g,
        base.b * blend.b,
        blend.a,
    )
}

fn blend_screen(base: ColorF, blend: ColorF) -> ColorF {
    ColorF::new(
        1.0 - (1.0 - base.r) * (1.0 - blend.r),
        1.0 - (1.0 - base.g) * (1.0 - blend.g),
        1.0 - (1.0 - base.b) * (1.0 - blend.b),
        blend.a,
    )
}

fn blend_overlay(base: ColorF, blend: ColorF) -> ColorF {
    ColorF::new(
        if base.r < 0.5 { 2.0 * base.r * blend.r } else { 1.0 - 2.0 * (1.0 - base.r) * (1.0 - blend.r) },
        if base.g < 0.5 { 2.0 * base.g * blend.g } else { 1.0 - 2.0 * (1.0 - base.g) * (1.0 - blend.g) },
        if base.b < 0.5 { 2.0 * base.b * blend.b } else { 1.0 - 2.0 * (1.0 - base.b) * (1.0 - blend.b) },
        blend.a,
    )
}

fn blend_darken(base: ColorF, blend: ColorF) -> ColorF {
    ColorF::new(
        base.r.min(blend.r),
        base.g.min(blend.g),
        base.b.min(blend.b),
        blend.a,
    )
}

fn blend_lighten(base: ColorF, blend: ColorF) -> ColorF {
    ColorF::new(
        base.r.max(blend.r),
        base.g.max(blend.g),
        base.b.max(blend.b),
        blend.a,
    )
}

fn blend_color_dodge(base: ColorF, blend: ColorF) -> ColorF {
    ColorF::new(
        if blend.r == 1.0 { 1.0 } else { (base.r / (1.0 - blend.r)).min(1.0) },
        if blend.g == 1.0 { 1.0 } else { (base.g / (1.0 - blend.g)).min(1.0) },
        if blend.b == 1.0 { 1.0 } else { (base.b / (1.0 - blend.b)).min(1.0) },
        blend.a,
    )
}

fn blend_color_burn(base: ColorF, blend: ColorF) -> ColorF {
    ColorF::new(
        if blend.r == 0.0 { 0.0 } else { 1.0 - ((1.0 - base.r) / blend.r).min(1.0) },
        if blend.g == 0.0 { 0.0 } else { 1.0 - ((1.0 - base.g) / blend.g).min(1.0) },
        if blend.b == 0.0 { 0.0 } else { 1.0 - ((1.0 - base.b) / blend.b).min(1.0) },
        blend.a,
    )
}

fn blend_hard_light(base: ColorF, blend: ColorF) -> ColorF {
    blend_overlay(blend, base)
}

fn blend_soft_light(base: ColorF, blend: ColorF) -> ColorF {
    fn soft_light_channel(base: f64, blend: f64) -> f64 {
        if blend < 0.5 {
            base - (1.0 - 2.0 * blend) * base * (1.0 - base)
        } else {
            let d = if base <= 0.25 {
                ((16.0 * base - 12.0) * base + 4.0) * base
            } else {
                base.sqrt()
            };
            base + (2.0 * blend - 1.0) * (d - base)
        }
    }
    
    ColorF::new(
        soft_light_channel(base.r, blend.r),
        soft_light_channel(base.g, blend.g),
        soft_light_channel(base.b, blend.b),
        blend.a,
    )
}

fn blend_difference(base: ColorF, blend: ColorF) -> ColorF {
    ColorF::new(
        (base.r - blend.r).abs(),
        (base.g - blend.g).abs(),
        (base.b - blend.b).abs(),
        blend.a,
    )
}

fn blend_exclusion(base: ColorF, blend: ColorF) -> ColorF {
    ColorF::new(
        base.r + blend.r - 2.0 * base.r * blend.r,
        base.g + blend.g - 2.0 * base.g * blend.g,
        base.b + blend.b - 2.0 * base.b * blend.b,
        blend.a,
    )
}

fn blend_hue(base: ColorF, blend: ColorF) -> ColorF {
    let (_, base_s, base_v) = base.to_hsv();
    let (blend_h, _, _) = blend.to_hsv();
    ColorF::from_hsv(blend_h, base_s, base_v)
}

fn blend_saturation(base: ColorF, blend: ColorF) -> ColorF {
    let (base_h, _, base_v) = base.to_hsv();
    let (_, blend_s, _) = blend.to_hsv();
    ColorF::from_hsv(base_h, blend_s, base_v)
}

fn blend_color_mode(base: ColorF, blend: ColorF) -> ColorF {
    let (_, base_v) = (base.to_hsv().0, base.to_hsv().2);
    let (blend_h, blend_s, _) = blend.to_hsv();
    ColorF::from_hsv(blend_h, blend_s, base_v)
}

fn blend_luminosity(base: ColorF, blend: ColorF) -> ColorF {
    let (base_h, base_s, _) = base.to_hsv();
    let (_, _, blend_v) = blend.to_hsv();
    ColorF::from_hsv(base_h, base_s, blend_v)
}

pub fn composite_layers(
    layers: &[&[u8]],
    blend_modes: &[BlendMode],
    opacities: &[f64],
    width: u32,
    height: u32,
) -> Vec<u8> {
    let pixel_count = (width * height * 4) as usize;
    
    if layers.is_empty() {
        // No visible layers: return fully transparent
        return vec![0u8; pixel_count];
    }
    
    let mut buffer_a = vec![0u8; pixel_count];
    let mut buffer_b = vec![0u8; pixel_count];
    let mut use_a_as_result = true;

    for (i, ((layer, mode), opacity)) in layers.iter().zip(blend_modes.iter()).zip(opacities.iter()).enumerate() {
        let (src, dst) = if use_a_as_result {
            (&buffer_a[..], &mut buffer_b[..])
        } else {
            (&buffer_b[..], &mut buffer_a[..])
        };
        
        if i == 0 {
            for j in 0..pixel_count.min(layer.len()) {
                dst[j] = layer[j];
            }
        } else {
            blend_pixels_rgba(src, layer, dst, *mode, *opacity);
        }
        
        use_a_as_result = !use_a_as_result;
    }

    if use_a_as_result {
        buffer_a
    } else {
        buffer_b
    }
}

pub fn apply_mask(image: &mut [u8], mask: &[u8], width: u32, height: u32) {
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            let img_idx = idx * 4;
            
            if img_idx + 3 < image.len() && idx < mask.len() {
                let mask_alpha = mask[idx] as f64 / 255.0;
                image[img_idx + 3] = (image[img_idx + 3] as f64 * mask_alpha) as u8;
            }
        }
    }
}

pub fn create_checkerboard(width: u32, height: u32, checker_size: u32, color1: Color8, color2: Color8) -> Vec<u8> {
    let mut result = vec![0u8; (width * height * 4) as usize];
    
    for y in 0..height {
        for x in 0..width {
            let checker_x = x / checker_size;
            let checker_y = y / checker_size;
            let is_light = (checker_x + checker_y) % 2 == 0;
            
            let color = if is_light { color1 } else { color2 };
            let idx = (y * width + x) as usize * 4;
            
            result[idx] = color.r;
            result[idx + 1] = color.g;
            result[idx + 2] = color.b;
            result[idx + 3] = 255;
        }
    }
    
    result
}

pub fn fill_rect(
    image: &mut [u8],
    width: u32,
    height: u32,
    rect: crate::Rect,
    color: Color8,
) {
    let x_start = rect.origin.x.max(0.0) as u32;
    let y_start = rect.origin.y.max(0.0) as u32;
    let x_end = (rect.origin.x + rect.size.width).min(width as f64) as u32;
    let y_end = (rect.origin.y + rect.size.height).min(height as f64) as u32;

    for y in y_start..y_end {
        for x in x_start..x_end {
            let idx = (y * width + x) as usize * 4;
            if idx + 3 < image.len() {
                image[idx] = color.r;
                image[idx + 1] = color.g;
                image[idx + 2] = color.b;
                image[idx + 3] = color.a;
            }
        }
    }
}

pub fn draw_circle(
    image: &mut [u8],
    width: u32,
    height: u32,
    cx: f64,
    cy: f64,
    radius: f64,
    color: Color8,
    filled: bool,
) {
    let x_start = (cx - radius).max(0.0) as u32;
    let y_start = (cy - radius).max(0.0) as u32;
    let x_end = (cx + radius).min(width as f64) as u32;
    let y_end = (cy + radius).min(height as f64) as u32;
    
    for y in y_start..y_end {
        for x in x_start..x_end {
            let dx = x as f64 - cx;
            let dy = y as f64 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            
            let should_draw = if filled {
                dist <= radius
            } else {
                dist <= radius && dist >= radius - 1.0
            };
            
            if should_draw {
                let idx = (y * width + x) as usize * 4;
                if idx + 3 < image.len() {
                    image[idx] = color.r;
                    image[idx + 1] = color.g;
                    image[idx + 2] = color.b;
                    image[idx + 3] = color.a;
                }
            }
        }
    }
}

pub fn draw_line(
    image: &mut [u8],
    width: u32,
    height: u32,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    color: Color8,
    thickness: f64,
) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1.0 } else { -1.0 };
    let sy = if y0 < y1 { 1.0 } else { -1.0 };
    let mut err = dx - dy;
    
    let mut x = x0;
    let mut y = y0;
    
    loop {
        draw_circle(image, width, height, x, y, thickness / 2.0, color, true);
        
        if (x - x1).abs() < 0.5 && (y - y1).abs() < 0.5 {
            break;
        }
        
        let e2 = 2.0 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

pub fn flood_fill(
    image: &[u8],
    width: u32,
    height: u32,
    start_x: u32,
    start_y: u32,
    fill_color: Color8,
    tolerance: f64,
) -> Vec<u8> {
    let mut result = image.to_vec();
    let start_idx = (start_y * width + start_x) as usize * 4;
    
    if start_idx + 3 >= image.len() {
        return result;
    }
    
    let target_r = image[start_idx];
    let target_g = image[start_idx + 1];
    let target_b = image[start_idx + 2];
    let target_a = image[start_idx + 3];
    
    let fill_r = fill_color.r;
    let fill_g = fill_color.g;
    let fill_b = fill_color.b;
    let fill_a = fill_color.a;
    
    if (target_r as i32 - fill_r as i32).abs() <= tolerance as i32
        && (target_g as i32 - fill_g as i32).abs() <= tolerance as i32
        && (target_b as i32 - fill_b as i32).abs() <= tolerance as i32
    {
        return result;
    }
    
    let mut stack = vec![(start_x, start_y)];
    let mut visited = vec![false; (width * height) as usize];
    
    while let Some((x, y)) = stack.pop() {
        let idx = (y * width + x) as usize;
        let pixel_idx = idx * 4;
        
        if visited[idx] {
            continue;
        }
        visited[idx] = true;
        
        if pixel_idx + 3 >= result.len() {
            continue;
        }
        
        let r = result[pixel_idx];
        let g = result[pixel_idx + 1];
        let b = result[pixel_idx + 2];
        let a = result[pixel_idx + 3];
        
        let diff = ((r as i32 - target_r as i32).abs() as f64
            + (g as i32 - target_g as i32).abs() as f64
            + (b as i32 - target_b as i32).abs() as f64
            + (a as i32 - target_a as i32).abs() as f64)
            / 4.0;
        
        if diff <= tolerance {
            result[pixel_idx] = fill_r;
            result[pixel_idx + 1] = fill_g;
            result[pixel_idx + 2] = fill_b;
            result[pixel_idx + 3] = fill_a;
            
            if x > 0 {
                stack.push((x - 1, y));
            }
            if x < width - 1 {
                stack.push((x + 1, y));
            }
            if y > 0 {
                stack.push((x, y - 1));
            }
            if y < height - 1 {
                stack.push((x, y + 1));
            }
        }
    }
    
    result
}
