use iced::widget::{button, column, container, row, slider, text, Space};
use iced::{Element, Length, Color, Fill, Alignment};
use super::Message;
use retas_core::advanced::coloring::{FillMode, FillSettings};

pub struct FillToolState {
    pub settings: FillSettings,
    pub show_panel: bool,
}

impl Default for FillToolState {
    fn default() -> Self {
        Self {
            settings: FillSettings::default(),
            show_panel: true,
        }
    }
}

impl FillToolState {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn toggle_panel(&mut self) {
        self.show_panel = !self.show_panel;
    }
    
    pub fn set_mode(&mut self, mode: FillMode) {
        self.settings.mode = mode;
    }
    
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.settings.tolerance = tolerance.clamp(0.0, 255.0);
    }
    
    pub fn set_gap_radius(&mut self, radius: u32) {
        self.settings.gap_closing_radius = radius.clamp(0, 10);
    }
    
    pub fn toggle_anti_aliasing(&mut self) {
        self.settings.anti_aliasing = !self.settings.anti_aliasing;
    }
    
    pub fn toggle_fill_behind_lines(&mut self) {
        self.settings.fill_behind_lines = !self.settings.fill_behind_lines;
    }
}

#[derive(Debug, Clone)]
pub enum FillToolMessage {
    SetMode(FillMode),
    SetTolerance(f64),
    SetGapRadius(u32),
    ToggleAntiAliasing,
    ToggleFillBehindLines,
}

pub fn view(state: &FillToolState) -> Element<Message> {
    if !state.show_panel {
        return container(Space::new().width(Length::Shrink).height(Length::Shrink))
            .into();
    }
    
    let header = container(
        text("填充工具")
            .size(14)
    )
    .padding(8)
    .style(header_style);
    
    let mode_buttons = {
        let modes = vec![FillMode::Normal, FillMode::Smart, FillMode::GapClosing];
        let mut buttons = row![].spacing(4);
        
        for mode in modes {
            let label = match mode {
                FillMode::Normal => "普通",
                FillMode::Smart => "智能",
                FillMode::GapClosing => "缝隙",
            };
            let is_active = state.settings.mode == mode;
            let btn = button(text(label).size(10))
                .on_press(Message::FillToolChanged(FillToolMessage::SetMode(mode)));
            buttons = buttons.push(btn);
        }
        
        column![
            text("填充模式").size(11),
            buttons,
        ]
        .spacing(4)
    };
    
    let tolerance_control = column![
        text(format!("容差: {:.0}", state.settings.tolerance)).size(11),
        slider(
            0.0..=255.0,
            state.settings.tolerance,
            |v| Message::FillToolChanged(FillToolMessage::SetTolerance(v))
        ),
    ]
    .spacing(4);
    
    let gap_radius_control = if state.settings.mode == FillMode::GapClosing {
        Some(column![
            text(format!("缝隙半径: {}", state.settings.gap_closing_radius)).size(11),
            slider(
                0.0..=10.0,
                state.settings.gap_closing_radius as f64,
                |v| Message::FillToolChanged(FillToolMessage::SetGapRadius(v as u32))
            ),
        ]
        .spacing(4))
    } else {
        None
    };
    
    let anti_alias_btn = button(
        text(if state.settings.anti_aliasing { "✓ 抗锯齿" } else { "☐ 抗锯齿" }).size(10)
    )
    .on_press(Message::FillToolChanged(FillToolMessage::ToggleAntiAliasing));
    
    let behind_lines_btn = button(
        text(if state.settings.fill_behind_lines { "✓ 线稿下方填充" } else { "☐ 线稿下方填充" }).size(10)
    )
    .on_press(Message::FillToolChanged(FillToolMessage::ToggleFillBehindLines));
    
    let mut content = column![
        header,
        mode_buttons,
        tolerance_control,
    ]
    .spacing(12)
    .padding(8);
    
    if let Some(gap_control) = gap_radius_control {
        content = content.push(gap_control);
    }
    
    content = content.push(anti_alias_btn);
    content = content.push(behind_lines_btn);
    
    container(content)
        .width(Fill)
        .style(panel_style)
        .into()
}

fn header_style(_: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(iced::Background::Color(Color::from_rgb8(50, 50, 55))),
        text_color: Some(Color::WHITE),
        ..Default::default()
    }
}

fn panel_style(_: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(iced::Background::Color(Color::from_rgb8(40, 40, 45))),
        border: iced::Border {
            color: Color::from_rgb8(60, 60, 65),
            width: 1.0,
            radius: iced::border::Radius::new(4.0),
        },
        ..Default::default()
    }
}
