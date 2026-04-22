use iced::widget::{button, column, container, row, text, scrollable, Space};
use iced::{Element, Length, Color, Alignment, Fill};
use super::{Message, LayerMessage};

pub struct LayerPanelState {
    pub layers: Vec<LayerEntry>,
    pub selected: Option<usize>,
}

#[derive(Clone)]
pub struct LayerEntry {
    pub name: String,
    pub visible: bool,
    pub locked: bool,
}

impl Default for LayerPanelState {
    fn default() -> Self {
        Self {
            layers: vec![
                LayerEntry { name: "图层 1".to_string(), visible: true, locked: false },
            ],
            selected: Some(0),
        }
    }
}

impl LayerPanelState {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn add_layer(&mut self, name: impl Into<String>) -> usize {
        let entry = LayerEntry {
            name: name.into(),
            visible: true,
            locked: false,
        };
        self.layers.push(entry);
        let idx = self.layers.len() - 1;
        self.selected = Some(idx);
        idx
    }
    
    pub fn remove_layer(&mut self, index: usize) {
        if index < self.layers.len() {
            self.layers.remove(index);
            if self.layers.is_empty() {
                self.selected = None;
            } else if self.selected.unwrap_or(0) >= self.layers.len() {
                self.selected = Some(self.layers.len() - 1);
            }
        }
    }
    
    pub fn move_layer_up(&mut self, index: usize) {
        if index > 0 && index < self.layers.len() {
            self.layers.swap(index, index - 1);
            self.selected = Some(index - 1);
        }
    }
    
    pub fn move_layer_down(&mut self, index: usize) {
        if index < self.layers.len() - 1 {
            self.layers.swap(index, index + 1);
            self.selected = Some(index + 1);
        }
    }
    
    pub fn toggle_visibility(&mut self, index: usize) {
        if let Some(layer) = self.layers.get_mut(index) {
            layer.visible = !layer.visible;
        }
    }
    
    pub fn toggle_lock(&mut self, index: usize) {
        if let Some(layer) = self.layers.get_mut(index) {
            layer.locked = !layer.locked;
        }
    }
    
    pub fn select_layer(&mut self, index: usize) {
        if index < self.layers.len() {
            self.selected = Some(index);
        }
    }
    
    pub fn duplicate_layer(&mut self, index: usize) {
        if let Some(layer) = self.layers.get(index).cloned() {
            let new_name = format!("{} 副本", layer.name);
            let mut new_layer = layer;
            new_layer.name = new_name;
            self.layers.insert(index + 1, new_layer);
            self.selected = Some(index + 1);
        }
    }
}

pub fn view(state: &LayerPanelState) -> Element<'static, Message> {
    let header = row![
        text("图层").size(14),
        Space::new().width(Fill),
        button(text("+").size(12)).on_press(Message::LayerSelected(LayerMessage::Add)),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let layers: Vec<Element<'static, Message>> = state
        .layers
        .iter()
        .enumerate()
        .map(|(i, layer)| {
            let is_selected = state.selected == Some(i);
            
            let visibility_btn = button(
                text(if layer.visible { "👁" } else { "🚫" }).size(10)
            )
            .on_press(Message::LayerSelected(LayerMessage::ToggleVisibility(i)))
            .width(Length::Fixed(24.0));
            
            let lock_btn = button(
                text(if layer.locked { "🔒" } else { "🔓" }).size(10)
            )
            .on_press(Message::LayerSelected(LayerMessage::ToggleLock(i)))
            .width(Length::Fixed(24.0));
            
            let layer_name = text(layer.name.clone()).size(11);
            
            let layer_row = row![
                visibility_btn,
                lock_btn,
                layer_name,
            ]
            .spacing(4)
            .align_y(Alignment::Center);

            container(layer_row)
                .width(Fill)
                .padding(6)
                .style(if is_selected { selected_layer_style } else { layer_style })
                .into()
        })
        .collect();

    let layer_list = scrollable(column(layers).spacing(2))
        .width(Fill)
        .height(Fill);
    
    let action_buttons = if let Some(selected) = state.selected {
        row![
            button(text("↑").size(10))
                .on_press(Message::LayerSelected(LayerMessage::MoveUp))
                .width(Length::Fixed(28.0)),
            button(text("↓").size(10))
                .on_press(Message::LayerSelected(LayerMessage::MoveDown))
                .width(Length::Fixed(28.0)),
            button(text("复制").size(10))
                .on_press(Message::LayerSelected(LayerMessage::Duplicate))
                .width(Length::Fixed(40.0)),
            button(text("删除").size(10))
                .on_press(Message::LayerSelected(LayerMessage::Delete))
                .width(Length::Fixed(40.0)),
        ]
        .spacing(4)
    } else {
        row![]
    };

    let content = column![
        header,
        layer_list,
        action_buttons,
    ]
    .spacing(8);

    container(content)
        .width(Length::Fixed(200.0))
        .height(Length::Fill)
        .padding(8)
        .style(panel_style)
        .into()
}

fn layer_style(_: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(iced::Background::Color(Color::from_rgb8(45, 45, 48))),
        border: iced::Border {
            color: Color::from_rgb8(60, 60, 60),
            width: 1.0,
            radius: iced::border::Radius::default(),
        },
        ..Default::default()
    }
}

fn selected_layer_style(_: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(iced::Background::Color(Color::from_rgb8(0, 120, 212))),
        border: iced::Border {
            color: Color::from_rgb8(0, 150, 255),
            width: 1.0,
            radius: iced::border::Radius::default(),
        },
        ..Default::default()
    }
}

fn panel_style(_: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(iced::Background::Color(Color::from_rgb8(35, 35, 38))),
        border: iced::Border {
            color: Color::from_rgb8(50, 50, 50),
            width: 1.0,
            radius: iced::border::Radius::default(),
        },
        ..Default::default()
    }
}
