use iced::widget::{button, column, container, row, slider, text, Space};
use iced::{Element, Length, Color, Fill};
use super::{Message, ColorMessage};
use super::stylus::{BrushPresetManager, DynamicBrush, PressureCurve};
use retas_core::Color8;

pub struct BrushPanelState {
    pub show_panel: bool,
}

impl Default for BrushPanelState {
    fn default() -> Self {
        Self {
            show_panel: false,
        }
    }
}

impl BrushPanelState {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn toggle(&mut self) {
        self.show_panel = !self.show_panel;
    }
}

#[derive(Debug, Clone)]
pub enum BrushMessage {
    TogglePanel,
    SelectPreset(usize),
    SetSizeMin(f32),
    SetSizeMax(f32),
    SetOpacityMin(f32),
    SetOpacityMax(f32),
    ToggleSizePressure,
    ToggleOpacityPressure,
    SetPressureCurve(PressureCurve),
    AddPreset(String),
    DeletePreset(usize),
}

pub fn view<'a>(
    state: &'a BrushPanelState,
    preset_manager: &'a BrushPresetManager,
) -> Element<'a, Message> {
    if !state.show_panel {
        return container(Space::new().width(Length::Shrink).height(Length::Shrink))
            .into();
    }
    
    let current_preset = match preset_manager.current_preset() {
        Some(p) => p,
        None => return container(text("未选择预设")).into(),
    };
    
    let dynamic_brush = &current_preset.dynamic_brush;
    
    let header = container(
        text("笔刷设置")
            .size(14)
    )
    .padding(8)
    .style(header_style);
    
    let preset_list = {
        let mut preset_buttons = row![].spacing(4);
        for (idx, preset) in preset_manager.presets.iter().enumerate() {
            let is_active = idx == preset_manager.current;
            let btn = button(text(&preset.name).size(10))
                .on_press(Message::ColorChanged(ColorMessage::PresetSelected(
                    Color8::BLACK
                )));
            preset_buttons = preset_buttons.push(btn);
        }
        container(preset_buttons)
            .padding(4)
    };
    
    let size_section = container(
        column![
            text("大小").size(12),
            row![
                text(format!("{:.0}", dynamic_brush.size_min)).size(10),
                slider(
                    1.0..=50.0,
                    dynamic_brush.size_min,
                    |v| Message::CanvasEvent(super::messages::CanvasMessage::MouseDown(0.0, 0.0))
                )
                .step(1.0)
                .width(Length::Fixed(100.0)),
                text(format!("{:.0}", dynamic_brush.size_max)).size(10),
            ]
            .spacing(4),
            row![
                text("压感").size(10),
                button(text(if dynamic_brush.size_pressure { "开" } else { "关" }).size(9))
                    .on_press(Message::CanvasEvent(super::messages::CanvasMessage::MouseDown(0.0, 0.0))),
            ]
            .spacing(4),
        ]
        .spacing(4)
    )
    .padding(4);
    
    let opacity_section = container(
        column![
            text("不透明度").size(12),
            row![
                text(format!("{:.0}%", dynamic_brush.opacity_min * 100.0)).size(10),
                slider(
                    0.0..=1.0,
                    dynamic_brush.opacity_min,
                    |v| Message::CanvasEvent(super::messages::CanvasMessage::MouseDown(0.0, 0.0))
                )
                .step(0.05)
                .width(Length::Fixed(100.0)),
                text(format!("{:.0}%", dynamic_brush.opacity_max * 100.0)).size(10),
            ]
            .spacing(4),
            row![
                text("压感").size(10),
                button(text(if dynamic_brush.opacity_pressure { "开" } else { "关" }).size(9))
                    .on_press(Message::CanvasEvent(super::messages::CanvasMessage::MouseDown(0.0, 0.0))),
            ]
            .spacing(4),
        ]
        .spacing(4)
    )
    .padding(4);
    
    let curve_buttons = row![
        button(text("线性").size(9))
            .on_press(Message::CanvasEvent(super::messages::CanvasMessage::MouseDown(0.0, 0.0))),
        button(text("柔和").size(9))
            .on_press(Message::CanvasEvent(super::messages::CanvasMessage::MouseDown(0.0, 0.0))),
        button(text("硬边").size(9))
            .on_press(Message::CanvasEvent(super::messages::CanvasMessage::MouseDown(0.0, 0.0))),
    ]
    .spacing(2);
    
    let curve_section = container(
        column![
            text("压感曲线").size(12),
            curve_buttons,
        ]
        .spacing(4)
    )
    .padding(4);
    
    let content = column![
        header,
        preset_list,
        size_section,
        opacity_section,
        curve_section,
    ]
    .spacing(8);
    
    container(content)
        .width(Length::Fixed(200.0))
        .height(Fill)
        .padding(8)
        .style(panel_style)
        .into()
}

fn header_style(_: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(iced::Background::Color(Color::from_rgb8(50, 50, 55))),
        text_color: Some(Color::from_rgb8(200, 200, 200)),
        ..Default::default()
    }
}

fn panel_style(_: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(iced::Background::Color(Color::from_rgb8(40, 40, 45))),
        border: iced::Border {
            color: Color::from_rgb8(60, 60, 65),
            width: 1.0,
            radius: iced::border::Radius::default(),
        },
        ..Default::default()
    }
}

pub fn update(
    state: &mut BrushPanelState,
    preset_manager: &mut BrushPresetManager,
    message: BrushMessage,
) {
    match message {
        BrushMessage::TogglePanel => {
            state.toggle();
        }
        BrushMessage::SelectPreset(idx) => {
            preset_manager.set_current(idx);
        }
        BrushMessage::SetSizeMin(size) => {
            if let Some(preset) = preset_manager.current_preset_mut() {
                preset.dynamic_brush.size_min = size;
            }
        }
        BrushMessage::SetSizeMax(size) => {
            if let Some(preset) = preset_manager.current_preset_mut() {
                preset.dynamic_brush.size_max = size;
            }
        }
        BrushMessage::SetOpacityMin(opacity) => {
            if let Some(preset) = preset_manager.current_preset_mut() {
                preset.dynamic_brush.opacity_min = opacity.clamp(0.0, 1.0);
            }
        }
        BrushMessage::SetOpacityMax(opacity) => {
            if let Some(preset) = preset_manager.current_preset_mut() {
                preset.dynamic_brush.opacity_max = opacity.clamp(0.0, 1.0);
            }
        }
        BrushMessage::ToggleSizePressure => {
            if let Some(preset) = preset_manager.current_preset_mut() {
                preset.dynamic_brush.size_pressure = !preset.dynamic_brush.size_pressure;
            }
        }
        BrushMessage::ToggleOpacityPressure => {
            if let Some(preset) = preset_manager.current_preset_mut() {
                preset.dynamic_brush.opacity_pressure = !preset.dynamic_brush.opacity_pressure;
            }
        }
        BrushMessage::SetPressureCurve(curve) => {
            if let Some(preset) = preset_manager.current_preset_mut() {
                preset.dynamic_brush.pressure_curve = curve;
            }
        }
        BrushMessage::AddPreset(name) => {
            let preset = super::stylus::BrushPreset::new(name);
            preset_manager.add_preset(preset);
        }
        BrushMessage::DeletePreset(idx) => {
            preset_manager.remove_preset(idx);
        }
    }
}
