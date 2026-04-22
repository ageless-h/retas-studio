use iced::widget::{button, column, container, row, text, scrollable, Space, slider};
use iced::{Element, Length, Color, Alignment, Fill};
use super::{Message, VectorLayerMessage};
use super::vector_layer::{VectorDocument, VectorLayer, VectorPath};

pub struct VectorLayerPanelState {
    pub document: VectorDocument,
}

impl Default for VectorLayerPanelState {
    fn default() -> Self {
        Self {
            document: VectorDocument::new(),
        }
    }
}

impl VectorLayerPanelState {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn update_from_document(&mut self, document: &VectorDocument) {
        self.document = document.clone();
    }
    
    pub fn create_layer(&mut self, name: impl Into<String>) -> usize {
        self.document.create_layer(name)
    }
    
    pub fn delete_layer(&mut self, index: usize) {
        self.document.delete_layer(index);
    }
    
    pub fn select_layer(&mut self, index: usize) {
        self.document.set_active_layer(index);
    }
    
    pub fn toggle_visibility(&mut self, index: usize) {
        if let Some(layer) = self.document.layers.get_mut(index) {
            layer.visible = !layer.visible;
        }
    }
    
    pub fn move_layer_up(&mut self, index: usize) {
        if index > 0 && index < self.document.layers.len() {
            self.document.layers.swap(index, index - 1);
            if let Some(active) = self.document.active_layer {
                if active == index {
                    self.document.active_layer = Some(index - 1);
                } else if active == index - 1 {
                    self.document.active_layer = Some(index);
                }
            }
        }
    }
    
    pub fn move_layer_down(&mut self, index: usize) {
        if index < self.document.layers.len().saturating_sub(1) {
            self.document.layers.swap(index, index + 1);
            if let Some(active) = self.document.active_layer {
                if active == index {
                    self.document.active_layer = Some(index + 1);
                } else if active == index + 1 {
                    self.document.active_layer = Some(index);
                }
            }
        }
    }
}

pub fn view(state: &VectorLayerPanelState) -> Element<'static, Message> {
    let header = row![
        text("矢量图层").size(14),
        Space::new().width(Fill),
        button(text("+").size(12)).on_press(Message::VectorLayerSelected(VectorLayerMessage::Add)),
    ]
    .spacing(8)
    .align_y(Alignment::Center);
    
    let pen_mode = state.document.pen_state.mode;
    let create_label = if pen_mode == super::vector_layer::PenToolMode::Create {
        "[创建]"
    } else {
        "创建"
    };
    let edit_label = if pen_mode == super::vector_layer::PenToolMode::Edit {
        "[编辑]"
    } else {
        "编辑"
    };
    let mode_buttons = row![
        button(text(create_label).size(10))
            .on_press(Message::VectorLayerSelected(VectorLayerMessage::SetPenMode(super::vector_layer::PenToolMode::Create))),
        button(text(edit_label).size(10))
            .on_press(Message::VectorLayerSelected(VectorLayerMessage::SetPenMode(super::vector_layer::PenToolMode::Edit))),
    ]
    .spacing(4);
    
    let layers: Vec<Element<'static, Message>> = state
        .document
        .layers
        .iter()
        .enumerate()
        .map(|(i, layer)| {
            let is_active = state.document.active_layer == Some(i);
            
            let visibility_btn = button(
                text(if layer.visible { "👁" } else { "🚫" }).size(10)
            )
            .on_press(Message::VectorLayerSelected(VectorLayerMessage::ToggleVisibility(i)))
            .width(Length::Fixed(24.0));
            
            let layer_name = text(format!("{}（{} 条路径）", layer.name, layer.paths.len())).size(11);
            
            let layer_row = row![
                visibility_btn,
                layer_name,
            ]
            .spacing(4)
            .align_y(Alignment::Center);
            
            let bg_color = if is_active {
                Color::from_rgb8(60, 80, 120)
            } else {
                Color::from_rgb8(45, 45, 48)
            };
            
            container(layer_row)
                .width(Fill)
                .padding(6)
                .style(move |_theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(bg_color)),
                    ..Default::default()
                })
                .into()
        })
        .collect();
    
    let content = column![
        header,
        mode_buttons,
        scrollable(column(layers).spacing(2))
            .height(Length::Fill)
            .width(Fill),
    ]
    .spacing(8)
    .padding(8);
    
    container(content)
        .width(Fill)
        .height(Length::Fill)
        .style(|_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::from_rgb8(35, 35, 38))),
            ..Default::default()
        })
        .into()
}
