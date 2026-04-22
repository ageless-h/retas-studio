use iced::widget::{container, text, image, Column, Space, mouse_area};
use iced::{Element, Length, Color};
use retas_core::Color8;
use retas_core::advanced::brush::{BrushEngine, BrushSettings, BrushPoint, BrushType};
use retas_vector::{BezierCurve, BezierControlPoint};
use retas_core::Point;
use std::time::Instant;

use super::app::CanvasMessage;
use super::Message;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasTool {
    Brush,
    Eraser,
    Pen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EraserMode {
    Hard,
    Soft,
}

#[derive(Debug, Clone)]
pub struct PenPath {
    pub curve: BezierCurve,
    pub is_drawing: bool,
}

pub struct CanvasState {
    pub zoom: f32,
    pub offset: (f32, f32),
    pub is_drawing: bool,
    pub current_tool: CanvasTool,
    pub eraser_mode: EraserMode,
    pub brush_engine: BrushEngine,
    pub brush_settings: BrushSettings,
    pub last_update: Option<Instant>,
    pub strokes: Vec<Vec<BrushPoint>>,
    pub current_layer_pixels: Vec<u8>,
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub pen_paths: Vec<PenPath>,
    pub current_pen_path: Option<PenPath>,
    pub last_mouse_pos: (f32, f32), // Track last mouse position
}

impl CanvasState {
    pub fn new() -> Self {
        let layer_manager = LayerManager::new(1920, 1080);
        
        Self {
            zoom: 1.0,
            offset: (0.0, 0.0),
            is_drawing: false,
            current_tool: CanvasTool::Brush,
            eraser_mode: EraserMode::Soft,
            brush_engine: BrushEngine::new(),
            brush_settings: BrushSettings::new(10.0, Color8::BLACK),
            last_update: None,
            strokes: Vec::new(),
            layer_manager,
            pen_paths: Vec::new(),
            current_pen_path: None,
            last_mouse_pos: (0.0, 0.0),
        }
    }
    }
    
    pub fn set_tool(&mut self, tool: CanvasTool) {
        self.current_tool = tool;
    }
    
    pub fn set_eraser_mode(&mut self, mode: EraserMode) {
        self.eraser_mode = mode;
    }
    
    pub fn set_brush_type(&mut self, brush_type: BrushType) {
        self.brush_settings.brush_type = brush_type;
    }
    
    pub fn start_pen_path(&mut self, x: f64, y: f64) {
        let curve = BezierCurve::new();
        let mut path = PenPath { curve, is_drawing: true };
        path.curve.add_point(BezierControlPoint::corner(Point::new(x, y)));
        self.current_pen_path = Some(path);
    }
    
    pub fn add_pen_point(&mut self, x: f64, y: f64) {
        if let Some(ref mut path) = self.current_pen_path {
            path.curve.add_point(BezierControlPoint::corner(Point::new(x, y)));
        }
    }
    
    pub fn finish_pen_path(&mut self) {
        if let Some(path) = self.current_pen_path.take() {
            let mut finished_path = path;
            finished_path.is_drawing = false;
            self.pen_paths.push(finished_path);
        }
    }
    
    pub fn screen_to_canvas(&self, screen_x: f32, screen_y: f32) -> (f64, f64) {
        let canvas_x = (screen_x / self.zoom) - self.offset.0;
        let canvas_y = (screen_y / self.zoom) - self.offset.1;
        (canvas_x as f64, canvas_y as f64)
    }
    
    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom * 1.1).min(5.0);
    }
    
    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom / 1.1).max(0.1);
    }
    
    pub fn zoom_reset(&mut self) {
        self.zoom = 1.0;
        self.offset = (0.0, 0.0);
    }
    
    pub fn clear_canvas(&mut self) {
        self.current_layer_pixels.fill(255);
        self.strokes.clear();
    }
    
    pub fn set_brush_size(&mut self, size: f64) {
        self.brush_settings.size = size;
    }
    
    pub fn set_brush_color(&mut self, color: Color8) {
        self.brush_settings.color = color;
    }
    
    pub fn render_stroke_to_pixels(&mut self, stroke: &[BrushPoint]) {
        if stroke.len() < 2 {
            return;
        }
        
        for i in 0..stroke.len() - 1 {
            let p1 = &stroke[i];
            let p2 = &stroke[i + 1];
            self.draw_pencil_line(
                p1.position.x,
                p1.position.y,
                p2.position.x,
                p2.position.y,
                self.brush_settings.size,
                self.brush_settings.color,
                self.brush_settings.opacity,
            );
        }
    }
    
    fn draw_pencil_line(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, thickness: f64, color: Color8, opacity: f64) {
        let width = self.canvas_width as i32;
        let height = self.canvas_height as i32;
        
        let x0 = x0 as i32;
        let y0 = y0 as i32;
        let x1 = x1 as i32;
        let y1 = y1 as i32;
        
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;
        
        let mut x = x0;
        let mut y = y0;
        
        let radius = (thickness / 2.0).max(1.0) as i32;
        
        loop {
            self.draw_circle(x, y, radius, color, opacity);
            
            if x == x1 && y == y1 {
                break;
            }
            
            let e2 = 2 * err;
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
    
    fn draw_circle(&mut self, cx: i32, cy: i32, radius: i32, color: Color8, opacity: f64) {
        let width = self.canvas_width as i32;
        let height = self.canvas_height as i32;
        
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= radius * radius {
                    let x = cx + dx;
                    let y = cy + dy;
                    
                    if x >= 0 && x < width && y >= 0 && y < height {
                        let idx = ((y * width + x) * 4) as usize;
                        let alpha = opacity;
                        let inv_alpha = 1.0 - alpha;
                        
                        if idx + 3 < self.current_layer_pixels.len() {
                            self.current_layer_pixels[idx] = (color.r as f64 * alpha + self.current_layer_pixels[idx] as f64 * inv_alpha) as u8;
                            self.current_layer_pixels[idx + 1] = (color.g as f64 * alpha + self.current_layer_pixels[idx + 1] as f64 * inv_alpha) as u8;
                            self.current_layer_pixels[idx + 2] = (color.b as f64 * alpha + self.current_layer_pixels[idx + 2] as f64 * inv_alpha) as u8;
                        }
                    }
                }
            }
        }
    }
    
    pub fn render_all_strokes(&mut self) {
        self.current_layer_pixels.fill(255);
        
        let strokes = self.strokes.clone();
        for stroke in &strokes {
            let saved_color = self.brush_settings.color;
            let saved_tool = self.current_tool;
            self.render_stroke_to_pixels(stroke);
            self.brush_settings.color = saved_color;
            self.current_tool = saved_tool;
        }
    }
}

impl Default for CanvasState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn view(state: &CanvasState) -> Element<'_, Message> {
    use iced::widget::image;
    
    let stroke_count = state.strokes.len();
    let zoom_pct = state.zoom * 100.0;
    let width = state.canvas_width;
    let height = state.canvas_height;
    
    let handle = image::Handle::from_pixels(
        width,
        height,
        state.current_layer_pixels.clone(),
    );
    
    let canvas_image = image(handle)
        .width(Length::Fixed(width as f32 * state.zoom))
        .height(Length::Fixed(height as f32 * state.zoom));
    
    let info_bar = container(
        text(format!(
            "Canvas: {}x{} | Zoom: {:.0}% | Strokes: {}",
            width, height, zoom_pct, stroke_count
        ))
        .size(12)
    )
    .padding(8)
    .style(info_bar_style);
    
    let canvas_container = container(canvas_image)
        .width(Length::Shrink)
        .height(Length::Shrink)
        .style(canvas_background_style);
    
    // Use stored last_mouse_pos for coordinates
    let last_pos = state.last_mouse_pos;
    let interactive_canvas = mouse_area(canvas_container)
        .on_press(Message::CanvasEvent(CanvasMessage::MouseDown(last_pos.0, last_pos.1)))
        .on_release(Message::CanvasEvent(CanvasMessage::MouseUp(last_pos.0, last_pos.1)));
    
    let content = Column::new()
        .push(info_bar)
        .push(Space::with_height(Length::Shrink))
        .push(
            container(interactive_canvas)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
        );
    
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(canvas_border_style)
        .into()
}

fn info_bar_style(_: &iced::Theme) -> iced::widget::container::Appearance {
    iced::widget::container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgb8(35, 35, 38))),
        text_color: Some(Color::from_rgb8(200, 200, 200)),
        ..Default::default()
    }
}

fn canvas_background_style(_: &iced::Theme) -> iced::widget::container::Appearance {
    iced::widget::container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgb8(45, 45, 48))),
        text_color: Some(Color::from_rgb8(200, 200, 200)),
        ..Default::default()
    }
}

fn canvas_border_style(_: &iced::Theme) -> iced::widget::container::Appearance {
    iced::widget::container::Appearance {
        border: iced::Border {
            color: Color::from_rgb8(60, 60, 60),
            width: 1.0,
            radius: iced::border::Radius::default(),
        },
        ..Default::default()
    }
}
