use iced::widget::{button, column, container, row, slider, text, Space};
use iced::{Element, Length, Color, Fill, Alignment};
use super::Message;
use retas_core::advanced::effects::{Effect, EffectType, EffectParameters, EffectStack};

pub struct EffectsPanelState {
    pub effect_stack: EffectStack,
    pub selected_effect: Option<usize>,
    pub show_panel: bool,
}

impl Default for EffectsPanelState {
    fn default() -> Self {
        Self {
            effect_stack: EffectStack::new(),
            selected_effect: None,
            show_panel: true,
        }
    }
}

impl EffectsPanelState {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn toggle_panel(&mut self) {
        self.show_panel = !self.show_panel;
    }
    
    pub fn add_effect(&mut self, effect_type: EffectType) {
        let effect = Effect::new(effect_type);
        self.effect_stack.add(effect);
        self.selected_effect = Some(self.effect_stack.effects.len() - 1);
    }
    
    pub fn remove_effect(&mut self, index: usize) {
        if index < self.effect_stack.effects.len() {
            self.effect_stack.effects.remove(index);
            if let Some(selected) = self.selected_effect {
                if selected >= index && selected > 0 {
                    self.selected_effect = Some(selected - 1);
                } else if selected >= self.effect_stack.effects.len() {
                    self.selected_effect = None;
                }
            }
        }
    }
    
    pub fn toggle_effect(&mut self, index: usize) {
        if let Some(effect) = self.effect_stack.effects.get_mut(index) {
            effect.enabled = !effect.enabled;
        }
    }
    
    pub fn select_effect(&mut self, index: usize) {
        if index < self.effect_stack.effects.len() {
            self.selected_effect = Some(index);
        }
    }
}

#[derive(Debug, Clone)]
pub enum EffectsMessage {
    AddEffect(EffectType),
    RemoveEffect(usize),
    ToggleEffect(usize),
    SelectEffect(usize),
    UpdateEffectParam(usize, EffectParamUpdate),
}

#[derive(Debug, Clone)]
pub enum EffectParamUpdate {
    Opacity(f64),
    BlurRadius(f64),
    Brightness(f64),
    Contrast(f64),
}

pub fn view(state: &EffectsPanelState) -> Element<Message> {
    if !state.show_panel {
        return container(Space::new().width(Length::Shrink).height(Length::Shrink))
            .into();
    }
    
    let header = row![
        text("特效").size(14),
        Space::new().width(Fill),
    ]
    .spacing(8)
    .align_y(Alignment::Center);
    
    let add_buttons = row![
        button(text("模糊").size(10))
            .on_press(Message::EffectsChanged(EffectsMessage::AddEffect(EffectType::GaussianBlur))),
        button(text("发光").size(10))
            .on_press(Message::EffectsChanged(EffectsMessage::AddEffect(EffectType::Glow))),
        button(text("亮度/对比度").size(10))
            .on_press(Message::EffectsChanged(EffectsMessage::AddEffect(EffectType::BrightnessContrast))),
    ]
    .spacing(4);
    
    let effect_list: Vec<Element<Message>> = state
        .effect_stack
        .effects
        .iter()
        .enumerate()
        .map(|(i, effect)| {
            let is_selected = state.selected_effect == Some(i);
            let is_enabled = effect.enabled;
            
            let name = match &effect.effect_type {
                EffectType::GaussianBlur => "高斯模糊",
                EffectType::Glow => "发光",
                EffectType::BrightnessContrast => "亮度/对比度",
                _ => "未知特效",
            };
            let label = if is_enabled {
                format!("✓ {}", name)
            } else {
                format!("☐ {}", name)
            };
            
            let row = row![
                button(text(label).size(10))
                    .on_press(Message::EffectsChanged(EffectsMessage::ToggleEffect(i))),
                Space::new().width(Fill),
                button(text("×").size(10))
                    .on_press(Message::EffectsChanged(EffectsMessage::RemoveEffect(i)))
                    .width(Length::Fixed(24.0)),
            ]
            .spacing(4)
            .align_y(Alignment::Center);
            
            let bg_color = if is_selected {
                Color::from_rgb8(60, 80, 120)
            } else {
                Color::from_rgb8(45, 45, 48)
            };
            
            container(row)
                .width(Fill)
                .padding(6)
                .style(move |_theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(bg_color)),
                    ..Default::default()
                })
                .into()
        })
        .collect();
    
    let param_controls = if let Some(idx) = state.selected_effect {
        if let Some(effect) = state.effect_stack.effects.get(idx) {
            Some(view_effect_params(effect, idx))
        } else {
            None
        }
    } else {
        None
    };
    
    let mut content = column![
        header,
        add_buttons,
        column(effect_list).spacing(2),
    ]
    .spacing(8)
    .padding(8);
    
    if let Some(params) = param_controls {
        content = content.push(params);
    }
    
    container(content)
        .width(Fill)
        .style(|_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::from_rgb8(40, 40, 45))),
            border: iced::Border {
                color: Color::from_rgb8(60, 60, 65),
                width: 1.0,
                radius: iced::border::Radius::new(4.0),
            },
            ..Default::default()
        })
        .into()
}

fn view_effect_params(effect: &Effect, idx: usize) -> Element<Message> {
    let opacity_control = column![
        text(format!("不透明度: {:.0}%", effect.opacity * 100.0)).size(11),
        slider(0.0..=100.0, effect.opacity * 100.0, move |v| {
            Message::EffectsChanged(EffectsMessage::UpdateEffectParam(idx, EffectParamUpdate::Opacity(v / 100.0)))
        }),
    ]
    .spacing(4);
    
    let mut params = column![opacity_control].spacing(8);
    
    match &effect.parameters {
        EffectParameters::GaussianBlur { radius } => {
            let blur_control = column![
                text(format!("半径: {:.1}", radius)).size(11),
                slider(0.0..=50.0, *radius, move |v| {
                    Message::EffectsChanged(EffectsMessage::UpdateEffectParam(idx, EffectParamUpdate::BlurRadius(v)))
                }),
            ]
            .spacing(4);
            params = params.push(blur_control);
        }
        EffectParameters::BrightnessContrast { brightness, contrast } => {
            let brightness_control = column![
                text(format!("亮度: {:.0}", brightness)).size(11),
                slider(-100.0..=100.0, *brightness, move |v| {
                    Message::EffectsChanged(EffectsMessage::UpdateEffectParam(idx, EffectParamUpdate::Brightness(v)))
                }),
            ]
            .spacing(4);
            let contrast_control = column![
                text(format!("对比度: {:.0}", contrast)).size(11),
                slider(-100.0..=100.0, *contrast, move |v| {
                    Message::EffectsChanged(EffectsMessage::UpdateEffectParam(idx, EffectParamUpdate::Contrast(v)))
                }),
            ]
            .spacing(4);
            params = params.push(brightness_control);
            params = params.push(contrast_control);
        }
        _ => {}
    }
    
    container(params)
        .width(Fill)
        .padding(8)
        .style(|_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::from_rgb8(50, 50, 55))),
            ..Default::default()
        })
        .into()
}
