use iced::widget::{container, column, row, text, slider, button, Space, scrollable};
use iced::{Element, Length, Color, Theme, Task, window, Alignment, Fill};
use retas_core::{Project, Color8};
use retas_core::advanced::brush::BrushPoint;
use crate::canvas::CanvasState;
use crate::timeline::TimelineState;
use crate::layer_panel::LayerPanelState;
use crate::vector_layer_panel::VectorLayerPanelState;
use crate::fill_tool::{FillToolState, FillToolMessage};
use crate::effects_panel::{EffectsPanelState, EffectsMessage, EffectParamUpdate};
use crate::export_panel::{ExportPanelState, ExportMessage};
use crate::color_picker::ColorPickerState;
use crate::messages::{Tool, Message, ColorMessage, ToolMessage, LayerMessage, VectorLayerMessage, TimelineMessage, CanvasMessage};

pub struct RetasApp {
    project: Project,
    theme: Theme,
    canvas_state: CanvasState,
    timeline_state: TimelineState,
    layer_panel_state: LayerPanelState,
    vector_layer_panel_state: VectorLayerPanelState,
    fill_tool_state: FillToolState,
    effects_panel_state: EffectsPanelState,
    export_panel_state: ExportPanelState,
    color_picker_state: ColorPickerState,
    current_tool: Tool,
    undo_stack: Vec<Vec<Vec<BrushPoint>>>,
    redo_stack: Vec<Vec<Vec<BrushPoint>>>,
}

impl RetasApp {
    pub fn handle_canvas_message(&mut self, msg: CanvasMessage) {
        use retas_core::advanced::brush::BrushPoint;
        use retas_core::Point as RetasPoint;
        use std::time::Instant;
        
        match msg {
            CanvasMessage::MouseDown(x, y) => {
                match self.current_tool {
                    Tool::Brush => {
                        self.save_undo_state();
                        let (canvas_x, canvas_y) = self.canvas_state.screen_to_canvas(x, y);
                        let point = BrushPoint::new(RetasPoint::new(canvas_x, canvas_y))
                            .with_timestamp(Instant::now().elapsed().as_secs_f64());
                        self.canvas_state.brush_engine.start_stroke(
                            self.canvas_state.brush_settings.clone(),
                            point,
                        );
                        self.canvas_state.is_drawing = true;
                        self.canvas_state.last_update = Some(Instant::now());
                    }
                    Tool::Pen => {
                        let (canvas_x, canvas_y) = self.canvas_state.screen_to_canvas(x, y);
                        let pos = retas_core::Point::new(canvas_x, canvas_y);
                        
                        if let Some(layer) = self.canvas_state.vector_document.get_active_layer() {
                            let pen_state = &self.canvas_state.vector_document.pen_state;
                            
                            match pen_state.mode {
                                super::vector_layer::PenToolMode::Create => {
                                    if self.canvas_state.vector_document.pen_state.current_path.is_none() {
                                        self.save_undo_state();
                                        self.canvas_state.start_pen_path(canvas_x, canvas_y);
                                    } else {
                                        self.canvas_state.add_pen_point(canvas_x, canvas_y);
                                    }
                                }
                                super::vector_layer::PenToolMode::Edit => {
                                    if let Some((path_idx, point_idx)) = pen_state.find_nearest_point(layer, pos, 10.0) {
                                        self.canvas_state.vector_document.pen_state.start_drag_point(path_idx, point_idx);
                                        self.canvas_state.vector_document.pen_state.select_point(path_idx, point_idx);
                                    }
                                }
                                _ => {}
                            }
                        } else {
                            self.save_undo_state();
                            self.canvas_state.start_pen_path(canvas_x, canvas_y);
                        }
                    }
                    Tool::Eraser => {
                        self.save_undo_state();
                        let (canvas_x, canvas_y) = self.canvas_state.screen_to_canvas(x, y);
                        let point = BrushPoint::new(RetasPoint::new(canvas_x, canvas_y))
                            .with_timestamp(Instant::now().elapsed().as_secs_f64());
                        self.canvas_state.brush_engine.start_stroke(
                            self.canvas_state.brush_settings.clone(),
                            point,
                        );
                        self.canvas_state.is_drawing = true;
                    }
                    Tool::Fill => {
                        self.save_undo_state();
                        let (canvas_x, canvas_y) = self.canvas_state.screen_to_canvas(x, y);
                        let px = canvas_x as i32;
                        let py = canvas_y as i32;
                        
                        if let Some(layer) = self.canvas_state.layer_manager.get_active_layer_mut() {
                            if !layer.locked && px >= 0 && py >= 0 && px < layer.width as i32 && py < layer.height as i32 {
                                let fill_color = self.canvas_state.brush_settings.color;
                                let engine = retas_core::advanced::coloring::ColoringEngine::new();
                                let filled = engine.smart_fill(
                                    &layer.pixel_data,
                                    layer.width,
                                    layer.height,
                                    px as u32,
                                    py as u32,
                                    fill_color,
                                );
                                layer.pixel_data = filled;
                            }
                        }
                    }
                    Tool::Zoom => {
                        self.canvas_state.zoom_in();
                    }
                    Tool::Hand => {
                    }
                    Tool::Select => {
                        let (canvas_x, canvas_y) = self.canvas_state.screen_to_canvas(x, y);
                        self.canvas_state.selection.start_selection(canvas_x as f32, canvas_y as f32);
                    }
                    _ => {}
                }
            }
            CanvasMessage::MouseUp(_x, _y) => {
                if self.canvas_state.is_drawing {
                    if let Some(stroke) = self.canvas_state.brush_engine.end_stroke() {
                        self.canvas_state.render_stroke_to_pixels(&stroke.points);
                        self.canvas_state.strokes.push(stroke.points);
                    }
                    self.canvas_state.is_drawing = false;
                    self.canvas_state.brush_settings.color = Color8::BLACK;
                    self.canvas_state.ensure_cache();
                }
                
                if self.current_tool == Tool::Select {
                    self.canvas_state.selection.finish_selection();
                }
                
                if self.current_tool == Tool::Pen {
                    self.canvas_state.vector_document.pen_state.end_drag();
                }
            }
            CanvasMessage::PenFinish => {
                self.canvas_state.finish_pen_path();
            }
            CanvasMessage::MouseMoved(x, y) => {
                if self.canvas_state.is_drawing {
                    let (canvas_x, canvas_y) = self.canvas_state.screen_to_canvas(x, y);
                    let point = BrushPoint::new(RetasPoint::new(canvas_x, canvas_y))
                        .with_timestamp(Instant::now().elapsed().as_secs_f64());
                    self.canvas_state.brush_engine.add_point(point);
                    self.canvas_state.last_update = Some(Instant::now());
                }
                
                if self.current_tool == Tool::Select && self.canvas_state.selection.is_active() {
                    let (canvas_x, canvas_y) = self.canvas_state.screen_to_canvas(x, y);
                    self.canvas_state.selection.update_selection(canvas_x as f32, canvas_y as f32);
                }
                
                if self.current_tool == Tool::Pen {
                    let (canvas_x, canvas_y) = self.canvas_state.screen_to_canvas(x, y);
                    self.canvas_state.handle_pen_mouse_move(canvas_x, canvas_y);
                }
            }
            CanvasMessage::MouseWheel(delta) => {
                if delta > 0.0 {
                    self.canvas_state.zoom_in();
                } else {
                    self.canvas_state.zoom_out();
                }
            }
            CanvasMessage::Pan(dx, dy) => {
                self.canvas_state.offset.0 += dx;
                self.canvas_state.offset.1 += dy;
            }
            CanvasMessage::KeyPress(key) => {
                match key.to_lowercase().as_str() {
                    "z" => {
                        if let Some(previous_state) = self.undo_stack.pop() {
                            self.redo_stack.push(self.canvas_state.strokes.clone());
                            self.canvas_state.strokes = previous_state;
                            self.canvas_state.render_all_strokes();
                        }
                    }
                    "y" => {
                        if let Some(next_state) = self.redo_stack.pop() {
                            self.undo_stack.push(self.canvas_state.strokes.clone());
                            self.canvas_state.strokes = next_state;
                            self.canvas_state.render_all_strokes();
                        }
                    }
                    "b" => {
                        self.current_tool = Tool::Brush;
                        self.canvas_state.set_tool(crate::canvas::CanvasTool::Brush);
                    }
                    "e" => {
                        self.current_tool = Tool::Eraser;
                        self.canvas_state.set_tool(crate::canvas::CanvasTool::Eraser);
                    }
                    "p" => {
                        self.current_tool = Tool::Pen;
                        self.canvas_state.set_tool(crate::canvas::CanvasTool::Pen);
                        self.canvas_state.set_brush_type(retas_core::advanced::brush::BrushType::Pencil);
                    }
                    "f" => {
                        self.current_tool = Tool::Fill;
                    }
                    "v" => {
                        self.current_tool = Tool::Move;
                    }
                    "h" => {
                        self.current_tool = Tool::Hand;
                    }
                    "+" | "=" => {
                        self.canvas_state.zoom_in();
                    }
                    "-" => {
                        self.canvas_state.zoom_out();
                    }
                    "0" => {
                        self.canvas_state.reset_view();
                    }
                    "[" => {
                        let new_size = (self.canvas_state.brush_settings.size - 1.0).max(1.0);
                        self.canvas_state.set_brush_size(new_size);
                    }
                    "]" => {
                        let new_size = (self.canvas_state.brush_settings.size + 1.0).min(100.0);
                        self.canvas_state.set_brush_size(new_size);
                    }
                    "delete" | "backspace" => {
                        self.save_undo_state();
                        self.canvas_state.clear_canvas();
                    }
                    "return" | "enter" => {
                        if self.current_tool == Tool::Pen {
                            self.canvas_state.finish_pen_path();
                        }
                    }
                    _ => {}
                }
            }
            CanvasMessage::KeyRelease(_) => {}
        }
    }
    
    pub fn save_undo_state(&mut self) {
        self.undo_stack.push(self.canvas_state.strokes.clone());
        if self.undo_stack.len() > 50 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }
    
    pub fn view_toolbar(&self) -> Element<'static, Message> {
        let file_buttons = row![
            button("新建").on_press(Message::NewDocument),
            button("导出PNG").on_press(Message::ExportPng),
        ]
        .spacing(4);

        let edit_buttons = row![
            button("撤销").on_press(Message::Undo),
            button("重做").on_press(Message::Redo),
            button("清空").on_press(Message::ClearCanvas),
        ]
        .spacing(4);

        let tool_buttons = row![
            button("画笔").on_press(Message::ToolSelected(ToolMessage::Brush)),
            button("钢笔").on_press(Message::ToolSelected(ToolMessage::Pen)),
            button("橡皮").on_press(Message::ToolSelected(ToolMessage::Eraser)),
            button("填充").on_press(Message::ToolSelected(ToolMessage::Fill)),
            button("移动").on_press(Message::ToolSelected(ToolMessage::Move)),
            button("缩放").on_press(Message::ToolSelected(ToolMessage::Zoom)),
            button("抓手").on_press(Message::ToolSelected(ToolMessage::Hand)),
        ]
        .spacing(4);

        let current_tool_text = text(format!("当前工具: {}", self.current_tool.display_name())).size(12);
        let zoom_text = text(format!("缩放: {:.0}%", self.canvas_state.zoom * 100.0)).size(12);

        row![
            file_buttons,
            edit_buttons,
            tool_buttons,
            Space::new().width(Fill),
            current_tool_text,
            zoom_text,
        ]
        .spacing(12)
        .padding(8)
        .align_y(Alignment::Center)
        .into()
    }
    
    pub fn view_brush_controls(&self) -> Element<'static, Message> {
        container(
            column![
                text(format!("笔刷大小: {:.0}", self.canvas_state.brush_settings.size)).size(12),
                slider(1.0..=100.0, self.canvas_state.brush_settings.size, Message::BrushSizeChanged),
            ]
            .spacing(6)
        )
        .padding(8)
        .style(|_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::from_rgb8(35, 35, 38))),
            border: iced::Border {
                color: Color::from_rgb8(50, 50, 50),
                width: 1.0,
                radius: iced::border::Radius::new(4.0),
            },
            ..Default::default()
        })
        .into()
    }

    pub fn handle_layer_message(&mut self, layer_msg: LayerMessage) {
        match layer_msg {
            LayerMessage::Add => {
                let new_name = format!("Layer {}", self.canvas_state.layer_manager.layer_count() + 1);
                self.canvas_state.layer_manager.add_layer(&new_name);
                self.layer_panel_state.add_layer(new_name);
            }
            LayerMessage::Delete => {
                if let Some(selected) = self.layer_panel_state.selected {
                    if let Some(layer) = self.canvas_state.layer_manager.layers.get(selected) {
                        let id = layer.id;
                        self.canvas_state.layer_manager.delete_layer(id);
                    }
                    self.layer_panel_state.remove_layer(selected);
                }
            }
            LayerMessage::Duplicate => {
                if let Some(selected) = self.layer_panel_state.selected {
                    let layer_data = self.canvas_state.layer_manager.layers.get(selected).map(|layer| {
                        (layer.name.clone(), layer.pixel_data.clone(), layer.opacity, layer.blend_mode)
                    });
                    if let Some((name, pixels, opacity, blend_mode)) = layer_data {
                        let new_id = self.canvas_state.layer_manager.add_layer(format!("{} Copy", name));
                        if let Some(new_layer) = self.canvas_state.layer_manager.get_layer_mut(new_id) {
                            new_layer.pixel_data = pixels;
                            new_layer.opacity = opacity;
                            new_layer.blend_mode = blend_mode;
                        }
                    }
                    self.layer_panel_state.duplicate_layer(selected);
                }
            }
            LayerMessage::MoveUp => {
                if let Some(selected) = self.layer_panel_state.selected {
                    if let Some(layer) = self.canvas_state.layer_manager.layers.get(selected) {
                        self.canvas_state.layer_manager.move_layer_up(layer.id);
                    }
                    self.layer_panel_state.move_layer_up(selected);
                }
            }
            LayerMessage::MoveDown => {
                if let Some(selected) = self.layer_panel_state.selected {
                    if let Some(layer) = self.canvas_state.layer_manager.layers.get(selected) {
                        self.canvas_state.layer_manager.move_layer_down(layer.id);
                    }
                    self.layer_panel_state.move_layer_down(selected);
                }
            }
            LayerMessage::Select(index) => {
                if let Some(layer) = self.canvas_state.layer_manager.layers.get(index) {
                    self.canvas_state.layer_manager.set_active_layer(layer.id);
                }
                self.layer_panel_state.select_layer(index);
            }
            LayerMessage::ToggleVisibility(index) => {
                if let Some(layer) = self.canvas_state.layer_manager.layers.get_mut(index) {
                    layer.visible = !layer.visible;
                }
                self.layer_panel_state.toggle_visibility(index);
            }
            LayerMessage::ToggleLock(index) => {
                if let Some(layer) = self.canvas_state.layer_manager.layers.get_mut(index) {
                    layer.locked = !layer.locked;
                }
                self.layer_panel_state.toggle_lock(index);
            }
        }
    }

    pub fn handle_vector_layer_message(&mut self, vector_msg: VectorLayerMessage) {
        match vector_msg {
            VectorLayerMessage::Add => {
                let new_name = format!("矢量图层 {}", self.canvas_state.vector_document.layers.len() + 1);
                self.canvas_state.vector_document.create_layer(&new_name);
                self.vector_layer_panel_state.create_layer(&new_name);
            }
            VectorLayerMessage::Delete(index) => {
                self.canvas_state.vector_document.delete_layer(index);
                self.vector_layer_panel_state.delete_layer(index);
            }
            VectorLayerMessage::Select(index) => {
                self.canvas_state.vector_document.set_active_layer(index);
                self.vector_layer_panel_state.select_layer(index);
            }
            VectorLayerMessage::ToggleVisibility(index) => {
                if let Some(layer) = self.canvas_state.vector_document.layers.get_mut(index) {
                    layer.visible = !layer.visible;
                }
                self.vector_layer_panel_state.toggle_visibility(index);
            }
            VectorLayerMessage::MoveUp(index) => {
                if index > 0 && index < self.canvas_state.vector_document.layers.len() {
                    self.canvas_state.vector_document.layers.swap(index, index - 1);
                    if let Some(active) = self.canvas_state.vector_document.active_layer {
                        if active == index {
                            self.canvas_state.vector_document.active_layer = Some(index - 1);
                        } else if active == index - 1 {
                            self.canvas_state.vector_document.active_layer = Some(index);
                        }
                    }
                }
                self.vector_layer_panel_state.move_layer_up(index);
            }
            VectorLayerMessage::MoveDown(index) => {
                let len = self.canvas_state.vector_document.layers.len();
                if index < len.saturating_sub(1) {
                    self.canvas_state.vector_document.layers.swap(index, index + 1);
                    if let Some(active) = self.canvas_state.vector_document.active_layer {
                        if active == index {
                            self.canvas_state.vector_document.active_layer = Some(index + 1);
                        } else if active == index + 1 {
                            self.canvas_state.vector_document.active_layer = Some(index);
                        }
                    }
                }
                self.vector_layer_panel_state.move_layer_down(index);
            }
            VectorLayerMessage::SetPenMode(mode) => {
                self.canvas_state.vector_document.pen_state.set_mode(mode);
            }
        }
    }

    pub fn handle_fill_tool_message(&mut self, fill_msg: FillToolMessage) {
        match fill_msg {
            FillToolMessage::SetMode(mode) => self.fill_tool_state.set_mode(mode),
            FillToolMessage::SetTolerance(tolerance) => self.fill_tool_state.set_tolerance(tolerance),
            FillToolMessage::SetGapRadius(radius) => self.fill_tool_state.set_gap_radius(radius),
            FillToolMessage::ToggleAntiAliasing => self.fill_tool_state.toggle_anti_aliasing(),
            FillToolMessage::ToggleFillBehindLines => self.fill_tool_state.toggle_fill_behind_lines(),
        }
    }

    pub fn handle_effects_message(&mut self, effects_msg: EffectsMessage) {
        match effects_msg {
            EffectsMessage::AddEffect(effect_type) => {
                self.effects_panel_state.add_effect(effect_type);
            }
            EffectsMessage::RemoveEffect(index) => {
                self.effects_panel_state.remove_effect(index);
            }
            EffectsMessage::ToggleEffect(index) => {
                self.effects_panel_state.toggle_effect(index);
            }
            EffectsMessage::SelectEffect(index) => {
                self.effects_panel_state.select_effect(index);
            }
            EffectsMessage::UpdateEffectParam(index, param) => {
                if let Some(effect) = self.effects_panel_state.effect_stack.effects.get_mut(index) {
                    match param {
                        EffectParamUpdate::Opacity(v) => effect.opacity = v,
                        EffectParamUpdate::BlurRadius(v) => {
                            if let retas_core::advanced::effects::EffectParameters::GaussianBlur { radius } = &mut effect.parameters {
                                *radius = v;
                            }
                        }
                        EffectParamUpdate::Brightness(v) => {
                            if let retas_core::advanced::effects::EffectParameters::BrightnessContrast { brightness, .. } = &mut effect.parameters {
                                *brightness = v;
                            }
                        }
                        EffectParamUpdate::Contrast(v) => {
                            if let retas_core::advanced::effects::EffectParameters::BrightnessContrast { contrast, .. } = &mut effect.parameters {
                                *contrast = v;
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn handle_timeline_message(&mut self, timeline_msg: TimelineMessage) {
        match timeline_msg {
            TimelineMessage::FrameChanged(frame) => {
                self.timeline_state.go_to_frame(frame);
            }
            TimelineMessage::Play => {
                self.timeline_state.toggle_play();
            }
            TimelineMessage::Pause => {
                self.timeline_state.is_playing = false;
            }
            TimelineMessage::Stop => {
                self.timeline_state.stop();
            }
            TimelineMessage::AddFrame => {
                self.timeline_state.total_frames += 1;
                self.timeline_state.end_frame += 1;
            }
            TimelineMessage::DeleteFrame => {
                if self.timeline_state.total_frames > 1 {
                    self.timeline_state.total_frames -= 1;
                    self.timeline_state.end_frame = self.timeline_state.end_frame.saturating_sub(1);
                }
            }
        }
    }

    pub fn handle_color_message(&mut self, color_msg: ColorMessage) {
        match color_msg {
            ColorMessage::PresetSelected(color) => {
                self.color_picker_state.set_color(color);
                self.canvas_state.set_brush_color(color);
            }
            ColorMessage::HueChanged(h) => {
                self.color_picker_state.set_hsv(h, self.color_picker_state.saturation, self.color_picker_state.value);
                self.canvas_state.set_brush_color(self.color_picker_state.primary_color);
            }
            ColorMessage::SaturationChanged(s) => {
                self.color_picker_state.set_hsv(self.color_picker_state.hue, s, self.color_picker_state.value);
                self.canvas_state.set_brush_color(self.color_picker_state.primary_color);
            }
            ColorMessage::ValueChanged(v) => {
                self.color_picker_state.set_hsv(self.color_picker_state.hue, self.color_picker_state.saturation, v);
                self.canvas_state.set_brush_color(self.color_picker_state.primary_color);
            }
            ColorMessage::RedChanged(r) => {
                self.color_picker_state.set_rgb(r, self.color_picker_state.green, self.color_picker_state.blue);
                self.canvas_state.set_brush_color(self.color_picker_state.primary_color);
            }
            ColorMessage::GreenChanged(g) => {
                self.color_picker_state.set_rgb(self.color_picker_state.red, g, self.color_picker_state.blue);
                self.canvas_state.set_brush_color(self.color_picker_state.primary_color);
            }
            ColorMessage::BlueChanged(b) => {
                self.color_picker_state.set_rgb(self.color_picker_state.red, self.color_picker_state.green, b);
                self.canvas_state.set_brush_color(self.color_picker_state.primary_color);
            }
        }
    }
}

fn new() -> (RetasApp, Task<Message>) {
    (
        RetasApp {
            project: Project::new(),
            theme: Theme::Dark,
            canvas_state: CanvasState::new(),
            timeline_state: TimelineState::default(),
            layer_panel_state: LayerPanelState::new(),
            vector_layer_panel_state: VectorLayerPanelState::new(),
            fill_tool_state: FillToolState::new(),
            effects_panel_state: EffectsPanelState::new(),
            export_panel_state: ExportPanelState::new(),
            color_picker_state: ColorPickerState::default(),
            current_tool: Tool::Brush,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        },
        Task::none(),
    )
}

fn title(_: &RetasApp) -> String {
    String::from("RETAS Studio")
}

fn update(app: &mut RetasApp, message: Message) -> Task<Message> {
    match message {
        Message::NewDocument => {
            let doc = retas_core::Document::new("Untitled", 1920.0, 1080.0);
            app.project.add_document(doc);
            app.canvas_state.clear_canvas();
            app.undo_stack.clear();
            app.redo_stack.clear();
        }
        Message::OpenDocument => {
            return Task::perform(open_file_dialog(), |result| {
                match result {
                    Ok(Some(path)) => Message::CanvasEvent(CanvasMessage::KeyPress(format!("opened:{}", path))),
                    Ok(None) => Message::CanvasEvent(CanvasMessage::KeyPress("cancelled".to_string())),
                    Err(_) => Message::CanvasEvent(CanvasMessage::KeyPress("error".to_string())),
                }
            });
        }
        Message::SaveDocument => {
            return Task::perform(save_file_dialog(), |result| {
                match result {
                    Ok(Some(path)) => Message::CanvasEvent(CanvasMessage::KeyPress(format!("saved:{}", path))),
                    Ok(None) => Message::CanvasEvent(CanvasMessage::KeyPress("cancelled".to_string())),
                    Err(_) => Message::CanvasEvent(CanvasMessage::KeyPress("error".to_string())),
                }
            });
        }
        Message::ExportPng => {
            let path = std::path::Path::new("canvas_export.png");
            let composited = app.canvas_state.layer_manager.composite_layers();
            let width = app.canvas_state.layer_manager.width();
            let height = app.canvas_state.layer_manager.height();
            let _ = retas_io::ImageExporter::export_pixels(
                &composited,
                width,
                height,
                path,
            );
        }
        Message::Undo => {
            if let Some(previous_state) = app.undo_stack.pop() {
                app.redo_stack.push(app.canvas_state.strokes.clone());
                app.canvas_state.strokes = previous_state;
                app.canvas_state.render_all_strokes();
            }
        }
        Message::Redo => {
            if let Some(next_state) = app.redo_stack.pop() {
                app.undo_stack.push(app.canvas_state.strokes.clone());
                app.canvas_state.strokes = next_state;
                app.canvas_state.render_all_strokes();
            }
        }
        Message::ToolSelected(ref tool_msg) => {
            app.current_tool = Tool::from(tool_msg.clone());
            match tool_msg {
                ToolMessage::Brush => {
                    app.canvas_state.set_tool(crate::canvas::CanvasTool::Brush);
                }
                ToolMessage::Pen => {
                    app.canvas_state.set_tool(crate::canvas::CanvasTool::Pen);
                    app.canvas_state.set_brush_type(retas_core::advanced::brush::BrushType::Pencil);
                }
                ToolMessage::Eraser => {
                    app.canvas_state.set_tool(crate::canvas::CanvasTool::Eraser);
                }
                _ => {}
            }
        }
        Message::LayerSelected(layer_msg) => app.handle_layer_message(layer_msg),
        Message::VectorLayerSelected(vector_msg) => app.handle_vector_layer_message(vector_msg),
        Message::FillToolChanged(fill_msg) => app.handle_fill_tool_message(fill_msg),
        Message::EffectsChanged(effects_msg) => app.handle_effects_message(effects_msg),
        Message::ExportChanged(export_msg) => {
            if let ExportMessage::QueueExport(doc_id) = export_msg {
                app.export_panel_state.queue_export(doc_id);
            }
        }
        Message::TimelineChanged(timeline_msg) => app.handle_timeline_message(timeline_msg),
        Message::CanvasEvent(canvas_msg) => {
            app.handle_canvas_message(canvas_msg);
        }
        Message::BrushSizeChanged(size) => {
            app.canvas_state.set_brush_size(size);
        }
        Message::ColorChanged(color_msg) => app.handle_color_message(color_msg),
        Message::ClearCanvas => {
            app.save_undo_state();
            app.canvas_state.clear_canvas();
            app.canvas_state.ensure_cache();
        }
    }
    Task::none()
}

fn view(app: &RetasApp) -> Element<Message> {
    let toolbar = app.view_toolbar();
    let canvas = crate::canvas::view(&app.canvas_state);
    let timeline = crate::timeline::view(&app.timeline_state);
    let layer_panel = crate::layer_panel::view(&app.layer_panel_state);
    let vector_layer_panel = crate::vector_layer_panel::view(&app.vector_layer_panel_state);
    let color_picker = crate::color_picker::view(&app.color_picker_state);
    let brush_controls = app.view_brush_controls();

    let sidebar_width = Length::Fixed(220.0);

    let left_sidebar = container(
        scrollable(
            column![
                layer_panel,
                vector_layer_panel,
            ]
            .spacing(8)
        )
    )
    .width(sidebar_width)
    .height(Fill)
    .style(|_theme| iced::widget::container::Style {
        background: Some(iced::Background::Color(Color::from_rgb8(30, 30, 33))),
        ..Default::default()
    });

    let right_sidebar = container(
        scrollable(
            column![
                color_picker,
                brush_controls,
            ]
            .spacing(8)
        )
    )
    .width(sidebar_width)
    .height(Fill)
    .style(|_theme| iced::widget::container::Style {
        background: Some(iced::Background::Color(Color::from_rgb8(30, 30, 33))),
        ..Default::default()
    });

    let main_content = row![
        left_sidebar,
        container(canvas)
            .width(Fill)
            .height(Fill)
            .style(|_theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(Color::from_rgb8(25, 25, 28))),
                ..Default::default()
            }),
        right_sidebar,
    ]
    .spacing(0)
    .height(Fill);

    let layout = column![
        toolbar,
        main_content,
        timeline,
    ]
    .spacing(0);

    container(layout)
        .width(Fill)
        .height(Fill)
        .into()
}

fn theme(_: &RetasApp) -> Theme {
    Theme::Dark
}

pub fn run() -> iced::Result {
    iced::application(new, update, view)
        .title(title)
        .theme(theme)
        .window(window::Settings {
            size: iced::Size::new(1280.0, 800.0),
            min_size: Some(iced::Size::new(800.0, 600.0)),
            ..Default::default()
        })
        .run()
}

async fn open_file_dialog() -> Result<Option<String>, ()> {
    let file = rfd::AsyncFileDialog::new()
        .add_filter("RETAS Project", &["retas"])
        .add_filter("All Files", &["*"])
        .pick_file()
        .await;
    
    Ok(file.map(|f| f.path().to_string_lossy().to_string()))
}

async fn save_file_dialog() -> Result<Option<String>, ()> {
    let file = rfd::AsyncFileDialog::new()
        .add_filter("RETAS Project", &["retas"])
        .add_filter("All Files", &["*"])
        .save_file()
        .await;
    
    Ok(file.map(|f| f.path().to_string_lossy().to_string()))
}
