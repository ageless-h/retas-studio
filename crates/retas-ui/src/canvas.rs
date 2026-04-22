use iced::widget::{container, text, Column, Space, canvas};
use iced::{Element, Length, Color, Fill, Rectangle, Renderer, Theme};
use iced::mouse::{self, Cursor};
use retas_core::Color8;
use retas_core::advanced::brush::{BrushEngine, BrushSettings, BrushPoint, BrushType};
use retas_vector::{BezierCurve, BezierControlPoint};
use retas_core::Point;
use std::time::Instant;

use super::app::CanvasMessage;
use super::Message;
use super::layer::{LayerManager, LayerId};
use super::vector_layer::{VectorDocument, VectorLayer, VectorPath, PenToolState, PenToolMode};
use super::vector_render::VectorRenderer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasTool {
    Brush,
    Eraser,
    Pen,
    Select,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionState {
    Idle,
    Selecting,
    Selected,
}

#[derive(Debug, Clone)]
pub struct Selection {
    pub state: SelectionState,
    pub start: (f32, f32),
    pub end: (f32, f32),
}

impl Selection {
    pub fn new() -> Self {
        Self {
            state: SelectionState::Idle,
            start: (0.0, 0.0),
            end: (0.0, 0.0),
        }
    }
    
    pub fn start_selection(&mut self, x: f32, y: f32) {
        self.state = SelectionState::Selecting;
        self.start = (x, y);
        self.end = (x, y);
    }
    
    pub fn update_selection(&mut self, x: f32, y: f32) {
        self.end = (x, y);
    }
    
    pub fn finish_selection(&mut self) {
        if self.state == SelectionState::Selecting {
            self.state = SelectionState::Selected;
        }
    }
    
    pub fn clear(&mut self) {
        self.state = SelectionState::Idle;
    }
    
    pub fn is_active(&self) -> bool {
        matches!(self.state, SelectionState::Selecting | SelectionState::Selected)
    }
    
    pub fn rect(&self) -> Option<(f32, f32, f32, f32)> {
        if !self.is_active() {
            return None;
        }
        
        let min_x = self.start.0.min(self.end.0);
        let min_y = self.start.1.min(self.end.1);
        let max_x = self.start.0.max(self.end.0);
        let max_y = self.start.1.max(self.end.1);
        
        Some((min_x, min_y, max_x - min_x, max_y - min_y))
    }
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
    pub dynamic_brush: super::stylus::DynamicBrush,
    pub brush_preset_manager: super::stylus::BrushPresetManager,
    pub last_update: Option<Instant>,
    pub strokes: Vec<Vec<BrushPoint>>,
    pub layer_manager: LayerManager,
    pub pen_paths: Vec<PenPath>,
    pub current_pen_path: Option<PenPath>,
    pub last_mouse_pos: (f32, f32),
    pub selection: Selection,
    pub vector_document: super::vector_layer::VectorDocument,
    pub is_vector_mode: bool,
    pub cached_handle: Option<iced::widget::image::Handle>,
    pub cache_dirty: bool,
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
            dynamic_brush: super::stylus::DynamicBrush::new(),
            brush_preset_manager: super::stylus::BrushPresetManager::new(),
            last_update: None,
            strokes: Vec::new(),
            layer_manager,
            pen_paths: Vec::new(),
            current_pen_path: None,
            last_mouse_pos: (0.0, 0.0),
            selection: Selection::new(),
            vector_document: super::vector_layer::VectorDocument::new(),
            is_vector_mode: false,
            cached_handle: None,
            cache_dirty: true,
        }
    }
    
    pub fn ensure_cache(&mut self) -> iced::widget::image::Handle {
        if self.cache_dirty || self.cached_handle.is_none() {
            let composited = self.layer_manager.composite_layers();
            let handle = iced::widget::image::Handle::from_rgba(
                self.layer_manager.width(),
                self.layer_manager.height(),
                composited,
            );
            self.cached_handle = Some(handle.clone());
            self.cache_dirty = false;
            handle
        } else {
            self.cached_handle.clone().unwrap()
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
    
    pub fn set_brush_size(&mut self, size: f64) {
        self.brush_settings.size = size;
    }
    
    pub fn set_brush_color(&mut self, color: Color8) {
        self.brush_settings.color = color;
    }
    
    pub fn clear_canvas(&mut self) {
        if let Some(layer) = self.layer_manager.get_active_layer_mut() {
            layer.clear();
        }
        self.strokes.clear();
        self.cache_dirty = true;
    }
    
    pub fn start_pen_path(&mut self, x: f64, y: f64) {
        if self.vector_document.layers.is_empty() {
            self.vector_document.create_layer("矢量图层 1");
        }
        
        let stroke_color = self.brush_settings.color;
        let path_id = self.vector_document.create_path(stroke_color, self.brush_settings.size as f32);
        self.vector_document.add_point_to_path(path_id, Point::new(x, y));
        self.is_vector_mode = true;
    }
    
    pub fn add_pen_point(&mut self, x: f64, y: f64) {
        if let Some(layer) = self.vector_document.get_active_layer() {
            if let Some(last_path) = layer.paths.last() {
                let path_id = last_path.id;
                self.vector_document.add_point_to_path(path_id, Point::new(x, y));
            }
        }
    }
    
    pub fn finish_pen_path(&mut self) {
        self.vector_document.pen_state.finish_path();
        self.is_vector_mode = true;
    }
    
    pub fn handle_pen_mouse_move(&mut self, x: f64, y: f64) {
        let pos = Point::new(x, y);
        let pen_state = &self.vector_document.pen_state;
        
        let is_dragging = pen_state.drag_handle_path.is_some();
        
        if is_dragging {
            let active_idx = self.vector_document.active_layer;
            if let Some(idx) = active_idx {
                if let Some(layer) = self.vector_document.layers.get_mut(idx) {
                    self.vector_document.pen_state.move_selected_point(layer, pos);
                }
            }
        } else if let Some(layer) = self.vector_document.get_active_layer() {
            let hovered = self.vector_document.pen_state.find_nearest_point(layer, pos, 10.0);
            self.vector_document.pen_state.hovered_point = hovered;
        }
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
    
    pub fn reset_view(&mut self) {
        self.zoom = 1.0;
        self.offset = (0.0, 0.0);
    }
    
    pub fn screen_to_canvas(&self, screen_x: f32, screen_y: f32) -> (f64, f64) {
        let canvas_x = (screen_x / self.zoom) - self.offset.0;
        let canvas_y = (screen_y / self.zoom) - self.offset.1;
        (canvas_x as f64, canvas_y as f64)
    }
    
    pub fn render_stroke_to_pixels(&mut self, points: &[BrushPoint]) {
        if points.len() < 2 {
            return;
        }
        
        let layer_id = match self.layer_manager.active_layer_id {
            Some(id) => id,
            None => return,
        };
        
        let settings = self.brush_settings.clone();
        let mut modified = false;
        
        for window in points.windows(2) {
            let p1 = &window[0];
            let p2 = &window[1];
            
            if let Some(layer) = self.layer_manager.get_layer_mut(layer_id) {
                if layer.locked {
                    continue;
                }
                draw_line_on_layer(
                    layer,
                    p1.position.x as i32,
                    p1.position.y as i32,
                    p2.position.x as i32,
                    p2.position.y as i32,
                    settings.size as i32,
                    settings.color,
                );
                modified = true;
            }
        }
        
        if modified {
            self.cache_dirty = true;
        }
    }
    
    pub fn render_all_strokes(&mut self) {
        let layer_id = match self.layer_manager.active_layer_id {
            Some(id) => id,
            None => return,
        };
        
        if let Some(layer) = self.layer_manager.get_layer_mut(layer_id) {
            layer.clear();
        }
        
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

fn draw_line_on_layer(layer: &mut super::layer::Layer, x0: i32, y0: i32, x1: i32, y1: i32, radius: i32, color: Color8) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut x = x0;
    let mut y = y0;
    
    loop {
        draw_circle_on_layer(layer, x, y, radius, color, 1.0);
        
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

fn draw_circle_on_layer(layer: &mut super::layer::Layer, cx: i32, cy: i32, radius: i32, color: Color8, opacity: f64) {
    let width = layer.width as i32;
    let height = layer.height as i32;
    
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx * dx + dy * dy <= radius * radius {
                let x = cx + dx;
                let y = cy + dy;
                
                if x >= 0 && x < width && y >= 0 && y < height {
                    let idx = ((y * width + x) * 4) as usize;
                    let alpha = opacity;
                    let inv_alpha = 1.0 - alpha;
                    
                    if idx + 3 < layer.pixel_data.len() {
                        layer.pixel_data[idx] = (color.r as f64 * alpha + layer.pixel_data[idx] as f64 * inv_alpha) as u8;
                        layer.pixel_data[idx + 1] = (color.g as f64 * alpha + layer.pixel_data[idx + 1] as f64 * inv_alpha) as u8;
                        layer.pixel_data[idx + 2] = (color.b as f64 * alpha + layer.pixel_data[idx + 2] as f64 * inv_alpha) as u8;
                    }
                }
            }
        }
    }
}

impl Default for CanvasState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CanvasProgram {
    image_handle: iced::widget::image::Handle,
    width: u32,
    height: u32,
    zoom: f32,
    selection: Option<(f32, f32, f32, f32)>,
    vector_document: VectorDocument,
    current_tool: CanvasTool,
    pen_state: PenToolState,
}

impl CanvasProgram {
    pub fn new(state: &CanvasState) -> Self {
        let handle = state.cached_handle.clone().unwrap_or_else(|| {
            let composited = state.layer_manager.composite_layers();
            iced::widget::image::Handle::from_rgba(
                state.layer_manager.width(),
                state.layer_manager.height(),
                composited,
            )
        });
        Self {
            image_handle: handle,
            width: state.layer_manager.width(),
            height: state.layer_manager.height(),
            zoom: state.zoom,
            selection: state.selection.rect(),
            vector_document: state.vector_document.clone(),
            current_tool: state.current_tool,
            pen_state: state.vector_document.pen_state.clone(),
        }
    }
}

impl Clone for CanvasProgram {
    fn clone(&self) -> Self {
        Self {
            image_handle: self.image_handle.clone(),
            width: self.width,
            height: self.height,
            zoom: self.zoom,
            selection: self.selection,
            vector_document: self.vector_document.clone(),
            current_tool: self.current_tool,
            pen_state: self.pen_state.clone(),
        }
    }
}

impl canvas::Program<Message> for CanvasProgram {
    type State = ();
    
    fn update(
        &self,
        _state: &mut (),
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Option<canvas::Action<Message>> {
        let cursor_position = cursor.position_in(bounds)?;
        let x = cursor_position.x;
        let y = cursor_position.y;
        
        match event {
            canvas::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) => {
                Some(canvas::Action::publish(Message::CanvasEvent(CanvasMessage::MouseDown(x, y))))
            }
            canvas::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                Some(canvas::Action::publish(Message::CanvasEvent(CanvasMessage::MouseUp(x, y))))
            }
            canvas::Event::Mouse(iced::mouse::Event::CursorMoved { .. }) => {
                Some(canvas::Action::publish(Message::CanvasEvent(CanvasMessage::MouseMoved(x, y))))
            }
            canvas::Event::Mouse(iced::mouse::Event::WheelScrolled { delta }) => {
                let delta_y = match *delta {
                    iced::mouse::ScrollDelta::Lines { y, .. } => y,
                    iced::mouse::ScrollDelta::Pixels { y, .. } => y / 50.0,
                };
                Some(canvas::Action::publish(Message::CanvasEvent(CanvasMessage::MouseWheel(delta_y))))
            }
            _ => None,
        }
    }
    
    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        
        let background = canvas::Path::rectangle(
            iced::Point::new(0.0, 0.0),
            iced::Size::new(bounds.width, bounds.height),
        );
        frame.fill(&background, Color::from_rgb8(45, 45, 48));
        
        if self.width > 0 && self.height > 0 {
            let scaled_width = self.width as f32 * self.zoom;
            let scaled_height = self.height as f32 * self.zoom;
            
            let offset_x = (bounds.width - scaled_width) / 2.0;
            let offset_y = (bounds.height - scaled_height) / 2.0;
            
            let canvas_image = canvas::Image::from(&self.image_handle)
                .snap(true);
            
            frame.draw_image(
                iced::Rectangle::new(
                    iced::Point::new(offset_x, offset_y),
                    iced::Size::new(scaled_width, scaled_height),
                ),
                canvas_image,
            );
            
            let border = canvas::Path::rectangle(
                iced::Point::new(offset_x, offset_y),
                iced::Size::new(scaled_width, scaled_height),
            );
            frame.stroke(
                &border,
                canvas::Stroke::default()
                    .with_color(Color::from_rgb8(60, 60, 60))
                    .with_width(1.0),
            );
            
            if let Some((sel_x, sel_y, sel_w, sel_h)) = self.selection {
                if sel_w > 0.0 && sel_h > 0.0 {
                    let sel_rect = canvas::Path::rectangle(
                        iced::Point::new(offset_x + sel_x * self.zoom, offset_y + sel_y * self.zoom),
                        iced::Size::new(sel_w * self.zoom, sel_h * self.zoom),
                    );
                    frame.stroke(
                        &sel_rect,
                        canvas::Stroke::default()
                            .with_color(Color::from_rgb8(0, 120, 255))
                            .with_width(2.0),
                    );
                }
            }
            
            if !self.vector_document.layers.is_empty() {
                VectorRenderer::render_document(
                    &self.vector_document,
                    &mut frame,
                    (offset_x, offset_y),
                    self.zoom,
                );
            }
        }
        
        vec![frame.into_geometry()]
    }
    
    fn mouse_interaction(
        &self,
        _state: &(),
        _bounds: Rectangle,
        cursor: Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(_bounds) {
            mouse::Interaction::Crosshair
        } else {
            mouse::Interaction::default()
        }
    }
}

pub fn view(state: &CanvasState) -> Element<'_, Message> {
    let zoom_pct = state.zoom * 100.0;
    let width = state.layer_manager.width();
    let height = state.layer_manager.height();
    let layer_count = state.layer_manager.layer_count();

    let info_bar = container(
        text(format!(
            "画布: {}x{} | 缩放: {:.0}% | 图层: {}",
            width, height, zoom_pct, layer_count
        ))
        .size(12)
    )
    .padding(6)
    .style(info_bar_style);

    let canvas_program = CanvasProgram::new(state);
    let canvas_widget = canvas(canvas_program)
        .width(Fill)
        .height(Fill);

    let content = Column::new()
        .push(canvas_widget)
        .push(info_bar);

    container(content)
        .width(Fill)
        .height(Fill)
        .style(canvas_border_style)
        .into()
}

fn info_bar_style(_: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(iced::Background::Color(Color::from_rgb8(35, 35, 38))),
        text_color: Some(Color::from_rgb8(200, 200, 200)),
        ..Default::default()
    }
}

fn canvas_border_style(_: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        border: iced::Border {
            color: Color::from_rgb8(60, 60, 60),
            width: 1.0,
            radius: iced::border::Radius::default(),
        },
        ..Default::default()
    }
}
