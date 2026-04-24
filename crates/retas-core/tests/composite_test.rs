use retas_core::{Color8, BlendMode};
use retas_core::composite::{blend_pixels, blend_pixels_rgba, composite_layers, apply_mask, create_checkerboard, fill_rect, draw_circle, draw_line, flood_fill};

#[test]
fn test_blend_pixels_normal_opaque() {
    let base = Color8::new(0, 0, 0, 255);
    let blend = Color8::new(255, 0, 0, 255);
    let result = blend_pixels(base, blend, BlendMode::Normal, 1.0);
    assert_eq!(result, Color8::new(255, 0, 0, 255));
}

#[test]
fn test_blend_pixels_normal_zero_opacity() {
    let base = Color8::new(0, 0, 0, 255);
    let blend = Color8::new(255, 0, 0, 255);
    let result = blend_pixels(base, blend, BlendMode::Normal, 0.0);
    assert_eq!(result, base);
}

#[test]
fn test_blend_pixels_normal_half_opacity() {
    let base = Color8::new(0, 0, 0, 255);
    let blend = Color8::new(255, 255, 255, 255);
    let result = blend_pixels(base, blend, BlendMode::Normal, 0.5);
    assert!(result.r > 120 && result.r < 135, "red should be ~128, got {}", result.r);
    assert!(result.g > 120 && result.g < 135, "green should be ~128, got {}", result.g);
    assert!(result.b > 120 && result.b < 135, "blue should be ~128, got {}", result.b);
}

#[test]
fn test_blend_pixels_multiply() {
    let base = Color8::new(128, 128, 128, 255);
    let blend = Color8::new(255, 0, 0, 255);
    let result = blend_pixels(base, blend, BlendMode::Multiply, 1.0);
    assert!(result.r < 130, "multiply should darken: r={}", result.r);
    assert_eq!(result.g, 0, "multiply with 0 should be 0");
}

#[test]
fn test_blend_pixels_screen() {
    let base = Color8::new(128, 128, 128, 255);
    let blend = Color8::new(255, 255, 255, 255);
    let result = blend_pixels(base, blend, BlendMode::Screen, 1.0);
    assert!(result.r > 200, "screen with white should lighten: r={}", result.r);
}

#[test]
fn test_blend_pixels_overlay() {
    let base = Color8::new(128, 128, 128, 255);
    let blend = Color8::new(255, 255, 255, 255);
    let result = blend_pixels(base, blend, BlendMode::Overlay, 1.0);
    assert!(result.r > 128, "overlay with white should lighten gray: r={}", result.r);
}

#[test]
fn test_blend_pixels_darken() {
    let base = Color8::new(200, 200, 200, 255);
    let blend = Color8::new(100, 250, 100, 255);
    let result = blend_pixels(base, blend, BlendMode::Darken, 1.0);
    assert_eq!(result.r, 100);
    assert_eq!(result.g, 200);
    assert_eq!(result.b, 100);
}

#[test]
fn test_blend_pixels_lighten() {
    let base = Color8::new(200, 200, 200, 255);
    let blend = Color8::new(100, 250, 100, 255);
    let result = blend_pixels(base, blend, BlendMode::Lighten, 1.0);
    assert_eq!(result.r, 200);
    assert_eq!(result.g, 250);
    assert_eq!(result.b, 200);
}

#[test]
fn test_blend_pixels_difference() {
    let base = Color8::new(255, 128, 0, 255);
    let blend = Color8::new(128, 255, 128, 255);
    let result = blend_pixels(base, blend, BlendMode::Difference, 1.0);
    assert!(result.r > 120 && result.r < 135, "difference of 255 and 128 should be ~127");
}

#[test]
fn test_blend_pixels_rgba_batch() {
    let base = vec![0u8, 0, 0, 255, 255, 255, 255, 255];
    let blend = vec![255u8, 0, 0, 255, 0, 255, 0, 128];
    let mut output = vec![0u8; 8];
    blend_pixels_rgba(&base, &blend, &mut output, BlendMode::Normal, 1.0);

    assert_eq!(output[0..4], [255, 0, 0, 255]);
    assert!(output[5] > 120, "second pixel should blend green");
}

#[test]
fn test_composite_layers_two_layers() {
    let layer1 = vec![255u8; 4];
    let layer2 = vec![0u8, 0, 255, 128];
    let layers: Vec<&[u8]> = vec![&layer1, &layer2];
    let modes = vec![BlendMode::Normal, BlendMode::Normal];
    let opacities = vec![1.0, 1.0];

    let result = composite_layers(&layers, &modes, &opacities, 1, 1);
    assert_eq!(result.len(), 4);
}

#[test]
fn test_composite_layers_empty() {
    let layers: Vec<&[u8]> = vec![];
    let modes: Vec<BlendMode> = vec![];
    let opacities: Vec<f64> = vec![];
    let result = composite_layers(&layers, &modes, &opacities, 1, 1);
    assert_eq!(result, vec![0u8; 4]);
}

#[test]
fn test_apply_mask() {
    let mut image = vec![255u8; 4];
    let mask = vec![128u8];
    apply_mask(&mut image, &mask, 1, 1);
    assert_eq!(image[3], 128);
}

#[test]
fn test_create_checkerboard() {
    let result = create_checkerboard(4, 4, 2, Color8::WHITE, Color8::BLACK);
    assert_eq!(result.len(), 4 * 4 * 4);
    assert_eq!(result[0..4], [255, 255, 255, 255]);
    assert_eq!(result[8..12], [0, 0, 0, 255]);
}

#[test]
fn test_fill_rect() {
    let mut image = vec![0u8; 10 * 10 * 4];
    let rect = retas_core::Rect::new(2.0, 2.0, 5.0, 5.0);
    fill_rect(&mut image, 10, 10, rect, Color8::new(255, 0, 0, 255));

    assert_eq!(image[(2 * 10 + 2) * 4], 255);
    assert_eq!(image[(6 * 10 + 6) * 4], 255);
    assert_eq!(image[0], 0);
}

#[test]
fn test_draw_circle_filled() {
    let mut image = vec![0u8; 10 * 10 * 4];
    draw_circle(&mut image, 10, 10, 5.0, 5.0, 3.0, Color8::new(0, 255, 0, 255), true);
    assert!(image[(5 * 10 + 5) * 4 + 1] == 255, "center should be green");
}

#[test]
fn test_draw_circle_outlined() {
    let mut image = vec![0u8; 10 * 10 * 4];
    draw_circle(&mut image, 10, 10, 5.0, 5.0, 3.0, Color8::new(0, 255, 0, 255), false);
    assert!(image[(5 * 10 + 5) * 4 + 1] == 0, "center should not be filled");
}

#[test]
fn test_draw_line() {
    let mut image = vec![0u8; 20 * 20 * 4];
    draw_line(&mut image, 20, 20, 5.0, 5.0, 15.0, 15.0, Color8::new(255, 0, 0, 255), 2.0);

    let mut found = false;
    for y in 0..20 {
        for x in 0..20 {
            if image[(y * 20 + x) * 4] == 255 {
                found = true;
            }
        }
    }
    assert!(found, "line should have drawn red pixels");
}

#[test]
fn test_flood_fill_basic() {
    let mut image = vec![255u8; 10 * 10 * 4];
    image[(5 * 10 + 5) * 4] = 0;
    image[(5 * 10 + 5) * 4 + 1] = 0;
    image[(5 * 10 + 5) * 4 + 2] = 0;

    let result = flood_fill(&image, 10, 10, 1, 1, Color8::new(255, 0, 0, 255), 1.0);
    assert_eq!(result[(1 * 10 + 1) * 4], 255);
    assert_eq!(result[(1 * 10 + 1) * 4 + 1], 0);
    assert_eq!(result[(5 * 10 + 5) * 4], 0);
}

#[test]
fn test_flood_fill_same_color_returns_original() {
    let image = vec![255u8; 4];
    let result = flood_fill(&image, 1, 1, 0, 0, Color8::new(255, 255, 255, 255), 1.0);
    assert_eq!(result, image);
}
