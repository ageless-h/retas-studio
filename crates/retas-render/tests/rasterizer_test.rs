use retas_core::{Color8, Point};
use retas_core::advanced::{BrushBlendMode, BrushPoint, BrushSettings};
use retas_render::StrokeRasterizer;

fn pixel_at(pixels: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let idx = ((y * width + x) * 4) as usize;
    [pixels[idx], pixels[idx + 1], pixels[idx + 2], pixels[idx + 3]]
}

#[test]
fn test_rasterizer_new_and_dimensions() {
    let r = StrokeRasterizer::new(100, 50);
    assert_eq!(r.width(), 100);
    assert_eq!(r.height(), 50);
    assert_eq!(r.pixels().len(), 100 * 50 * 4);
}

#[test]
fn test_rasterizer_from_rgba() {
    let data = vec![255u8; 10 * 10 * 4];
    let r = StrokeRasterizer::from_rgba(10, 10, &data);
    assert_eq!(r.pixels(), &data[..]);
}

#[test]
fn test_clear_with_color() {
    let mut r = StrokeRasterizer::new(10, 10);
    r.clear(Some(Color8::new(255, 0, 0, 255)));
    for pixel in r.pixels().chunks_exact(4) {
        assert_eq!(pixel, [255, 0, 0, 255]);
    }
}

#[test]
fn test_clear_default_transparent() {
    let mut r = StrokeRasterizer::new(10, 10);
    r.clear(None);
    for pixel in r.pixels().chunks_exact(4) {
        assert_eq!(pixel, [0, 0, 0, 0]);
    }
}

#[test]
fn test_set_pixel_in_bounds() {
    let mut r = StrokeRasterizer::new(10, 10);
    r.set_pixel(5, 5, Color8::new(255, 128, 64, 255));
    assert_eq!(pixel_at(r.pixels(), 10, 5, 5), [255, 128, 64, 255]);
}

#[test]
fn test_set_pixel_out_of_bounds_is_noop() {
    let mut r = StrokeRasterizer::new(10, 10);
    r.set_pixel(-1, 5, Color8::new(255, 0, 0, 255));
    r.set_pixel(5, -1, Color8::new(255, 0, 0, 255));
    r.set_pixel(10, 5, Color8::new(255, 0, 0, 255));
    r.set_pixel(5, 10, Color8::new(255, 0, 0, 255));
    assert_eq!(pixel_at(r.pixels(), 10, 5, 5), [0, 0, 0, 0]);
}

#[test]
fn test_blend_pixel_normal_opaque_over_transparent() {
    let mut r = StrokeRasterizer::new(10, 10);
    r.blend_pixel(5, 5, Color8::new(255, 0, 0, 255), 1.0, BrushBlendMode::Normal);
    assert_eq!(pixel_at(r.pixels(), 10, 5, 5), [255, 0, 0, 255]);
}

#[test]
fn test_blend_pixel_normal_half_opacity_over_transparent() {
    let mut r = StrokeRasterizer::new(10, 10);
    r.blend_pixel(5, 5, Color8::new(255, 0, 0, 255), 0.5, BrushBlendMode::Normal);
    let p = pixel_at(r.pixels(), 10, 5, 5);
    assert_eq!(p[0], 255, "red channel should be full");
    assert!(p[3] > 120 && p[3] < 135, "alpha should be ~128, got {}", p[3]);
}

#[test]
fn test_blend_pixel_normal_half_opacity_over_white() {
    let mut r = StrokeRasterizer::new(10, 10);
    r.set_pixel(5, 5, Color8::new(255, 255, 255, 255));
    r.blend_pixel(5, 5, Color8::new(255, 0, 0, 255), 0.5, BrushBlendMode::Normal);
    let p = pixel_at(r.pixels(), 10, 5, 5);
    assert!(p[0] == 255, "red stays max");
    assert!(p[1] > 120 && p[1] < 135, "green should be ~128, got {}", p[1]);
    assert!(p[2] > 120 && p[2] < 135, "blue should be ~128, got {}", p[2]);
}

#[test]
fn test_blend_pixel_multiply() {
    let mut r = StrokeRasterizer::new(10, 10);
    r.set_pixel(5, 5, Color8::new(128, 128, 128, 255));
    r.blend_pixel(5, 5, Color8::new(255, 0, 0, 255), 1.0, BrushBlendMode::Multiply);
    let p = pixel_at(r.pixels(), 10, 5, 5);
    assert!(p[0] < 130, "multiply with red should darken red: {}", p[0]);
}

#[test]
fn test_blend_pixel_screen() {
    let mut r = StrokeRasterizer::new(10, 10);
    r.set_pixel(5, 5, Color8::new(128, 128, 128, 255));
    r.blend_pixel(5, 5, Color8::new(255, 255, 255, 255), 1.0, BrushBlendMode::Screen);
    let p = pixel_at(r.pixels(), 10, 5, 5);
    assert!(p[0] > 200, "screen with white should lighten: {}", p[0]);
}

#[test]
fn test_blend_pixel_darken() {
    let mut r = StrokeRasterizer::new(10, 10);
    r.set_pixel(5, 5, Color8::new(200, 200, 200, 255));
    r.blend_pixel(5, 5, Color8::new(100, 250, 100, 255), 1.0, BrushBlendMode::Darken);
    let p = pixel_at(r.pixels(), 10, 5, 5);
    assert_eq!(p[0], 100);
    assert_eq!(p[1], 200);
}

#[test]
fn test_blend_pixel_lighten() {
    let mut r = StrokeRasterizer::new(10, 10);
    r.set_pixel(5, 5, Color8::new(200, 200, 200, 255));
    r.blend_pixel(5, 5, Color8::new(100, 250, 100, 255), 1.0, BrushBlendMode::Lighten);
    let p = pixel_at(r.pixels(), 10, 5, 5);
    assert_eq!(p[0], 200);
    assert_eq!(p[1], 250);
}

#[test]
fn test_blend_pixel_out_of_bounds_is_noop() {
    let mut r = StrokeRasterizer::new(10, 10);
    r.blend_pixel(-1, 5, Color8::new(255, 0, 0, 255), 1.0, BrushBlendMode::Normal);
    r.blend_pixel(5, 10, Color8::new(255, 0, 0, 255), 1.0, BrushBlendMode::Normal);
    assert_eq!(pixel_at(r.pixels(), 10, 5, 5), [0, 0, 0, 0]);
}

#[test]
fn test_draw_circle_basic() {
    let mut r = StrokeRasterizer::new(20, 20);
    r.draw_circle(10.0, 10.0, 3.0, Color8::new(255, 0, 0, 255), 1.0, 1.0, BrushBlendMode::Normal);

    let mut found = false;
    for y in 0..20 {
        for x in 0..20 {
            if pixel_at(r.pixels(), 20, x, y)[0] == 255 {
                found = true;
            }
        }
    }
    assert!(found, "circle should have drawn red pixels");
}

#[test]
fn test_draw_circle_out_of_bounds_partial() {
    let mut r = StrokeRasterizer::new(10, 10);
    r.draw_circle(-2.0, 5.0, 5.0, Color8::new(255, 0, 0, 255), 1.0, 1.0, BrushBlendMode::Normal);
    assert!(pixel_at(r.pixels(), 10, 0, 5)[0] == 255, "partial circle should be visible at edge");
}

#[test]
fn test_draw_ellipse_basic() {
    let mut r = StrokeRasterizer::new(30, 30);
    r.draw_ellipse(15.0, 15.0, 5.0, 3.0, 0.0, Color8::new(0, 255, 0, 255), 1.0, 1.0, BrushBlendMode::Normal);

    let mut found = false;
    for y in 0..30 {
        for x in 0..30 {
            if pixel_at(r.pixels(), 30, x, y)[1] == 255 {
                found = true;
            }
        }
    }
    assert!(found, "ellipse should have drawn green pixels");
}

#[test]
fn test_draw_line_basic() {
    let mut r = StrokeRasterizer::new(20, 20);
    r.draw_line(5.0, 5.0, 15.0, 15.0, Color8::new(0, 0, 255, 255), 2.0, 1.0);

    let mut found_start = false;
    let mut found_end = false;
    for y in 0..20 {
        for x in 0..20 {
            if pixel_at(r.pixels(), 20, x, y)[2] == 255 {
                if x <= 6 && y <= 6 { found_start = true; }
                if x >= 14 && y >= 14 { found_end = true; }
            }
        }
    }
    assert!(found_start, "line should start near (5,5)");
    assert!(found_end, "line should end near (15,15)");
}

#[test]
fn test_flood_fill_basic() {
    let mut r = StrokeRasterizer::new(10, 10);
    r.clear(Some(Color8::new(255, 255, 255, 255)));
    r.set_pixel(5, 5, Color8::new(0, 0, 0, 255));
    r.flood_fill(1, 1, Color8::new(255, 0, 0, 255), 0.01);

    assert_eq!(pixel_at(r.pixels(), 10, 1, 1), [255, 0, 0, 255]);
    assert_eq!(pixel_at(r.pixels(), 10, 5, 5), [0, 0, 0, 255]);
}

#[test]
fn test_flood_fill_out_of_bounds_is_noop() {
    let mut r = StrokeRasterizer::new(10, 10);
    r.flood_fill(-1, 5, Color8::new(255, 0, 0, 255), 0.01);
    r.flood_fill(5, 10, Color8::new(255, 0, 0, 255), 0.01);
}

#[test]
fn test_copy_to_basic() {
    let mut src = StrokeRasterizer::new(10, 10);
    src.set_pixel(0, 0, Color8::new(255, 0, 0, 255));
    src.set_pixel(1, 1, Color8::new(0, 255, 0, 255));

    let mut dst = StrokeRasterizer::new(10, 10);
    src.copy_to(&mut dst, 0, 0, 2, 2, 2, 2);

    assert_eq!(pixel_at(dst.pixels(), 10, 2, 2), [255, 0, 0, 255]);
    assert_eq!(pixel_at(dst.pixels(), 10, 3, 3), [0, 255, 0, 255]);
}

#[test]
fn test_copy_to_clamps_out_of_bounds() {
    let mut src = StrokeRasterizer::new(10, 10);
    src.set_pixel(0, 0, Color8::new(255, 0, 0, 255));

    let mut dst = StrokeRasterizer::new(10, 10);
    src.copy_to(&mut dst, 0, 0, 9, 9, 5, 5);

    assert_eq!(pixel_at(dst.pixels(), 10, 9, 9), [255, 0, 0, 255]);
}

#[test]
fn test_rasterize_stroke_empty_points() {
    let mut r = StrokeRasterizer::new(100, 100);
    let settings = BrushSettings::new(10.0, Color8::new(255, 0, 0, 255));
    r.rasterize_stroke(&[], &settings);
    assert_eq!(pixel_at(r.pixels(), 100, 50, 50), [0, 0, 0, 0]);
}

#[test]
fn test_rasterize_stroke_single_point() {
    let mut r = StrokeRasterizer::new(100, 100);
    let settings = BrushSettings::new(10.0, Color8::new(255, 0, 0, 255));
    let points = vec![BrushPoint::new(Point::new(50.0, 50.0))];
    r.rasterize_stroke(&points, &settings);

    assert!(pixel_at(r.pixels(), 100, 50, 50)[0] > 0, "should have drawn at center");
}

#[test]
fn test_rasterize_stroke_multiple_points() {
    let mut r = StrokeRasterizer::new(100, 100);
    let settings = BrushSettings::new(5.0, Color8::new(0, 0, 255, 255));
    let points = vec![
        BrushPoint::new(Point::new(20.0, 20.0)),
        BrushPoint::new(Point::new(80.0, 80.0)),
    ];
    r.rasterize_stroke(&points, &settings);

    let mut found_start = false;
    let mut found_end = false;
    for y in 0..100 {
        for x in 0..100 {
            if pixel_at(r.pixels(), 100, x, y)[2] > 0 {
                if x <= 25 && y <= 25 { found_start = true; }
                if x >= 75 && y >= 75 { found_end = true; }
            }
        }
    }
    assert!(found_start, "should have drawn near start");
    assert!(found_end, "should have drawn near end");
}

#[test]
fn test_pixels_mut() {
    let mut r = StrokeRasterizer::new(10, 10);
    {
        let pixels = r.pixels_mut();
        pixels[0] = 255;
        pixels[3] = 255;
    }
    assert_eq!(pixel_at(r.pixels(), 10, 0, 0), [255, 0, 0, 255]);
}
