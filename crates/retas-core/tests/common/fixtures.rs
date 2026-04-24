use retas_core::{Document, Layer, RasterLayer, VectorLayer, Color8, BlendMode, Matrix2D, Point, Size, Rect};
use retas_core::advanced::{BrushSettings, BrushPoint, BrushType, BrushBlendMode};

pub fn create_test_document() -> Document {
    Document::new("Test Document", 1920.0, 1080.0)
}

pub fn create_test_raster_layer(name: &str) -> RasterLayer {
    let mut layer = RasterLayer::new(name);
    layer.base.opacity = 1.0;
    layer.base.blend_mode = BlendMode::Normal;
    layer
}

pub fn create_test_vector_layer(name: &str) -> VectorLayer {
    let mut layer = VectorLayer::new(name);
    layer.base.opacity = 1.0;
    layer.base.blend_mode = BlendMode::Normal;
    layer
}

pub fn create_red_brush_settings(size: f64) -> BrushSettings {
    BrushSettings::new(size, Color8::RED)
        .with_hardness(1.0)
        .with_type(BrushType::Round)
        .with_blend_mode(BrushBlendMode::Normal)
}

pub fn create_blue_brush_settings(size: f64) -> BrushSettings {
    BrushSettings::new(size, Color8::new(0, 0, 255, 255))
        .with_hardness(1.0)
        .with_type(BrushType::Round)
        .with_blend_mode(BrushBlendMode::Normal)
}

pub fn create_line_points(x1: f64, y1: f64, x2: f64, y2: f64, count: usize) -> Vec<BrushPoint> {
    let mut points = Vec::new();
    for i in 0..count {
        let t = i as f64 / (count - 1).max(1) as f64;
        let x = x1 + (x2 - x1) * t;
        let y = y1 + (y2 - y1) * t;
        points.push(BrushPoint::new(Point::new(x, y)));
    }
    points
}

pub fn count_non_transparent_pixels(pixels: &[u8]) -> usize {
    pixels.chunks_exact(4).filter(|c| c[3] > 0).count()
}

pub fn pixel_color_at(pixels: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let idx = ((y * width + x) * 4) as usize;
    [pixels[idx], pixels[idx + 1], pixels[idx + 2], pixels[idx + 3]]
}

pub fn create_white_canvas(width: u32, height: u32) -> Vec<u8> {
    vec![255u8; (width * height * 4) as usize]
}

pub fn create_transparent_canvas(width: u32, height: u32) -> Vec<u8> {
    vec![0u8; (width * height * 4) as usize]
}
