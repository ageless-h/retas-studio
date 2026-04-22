use iced::widget::{column, container, row, text, slider, button};
use iced::{Element, Length, Color, Fill};
use retas_core::Color8;
use super::{Message, ColorMessage};

pub struct ColorPickerState {
    pub primary_color: Color8,
    pub secondary_color: Color8,
    pub hue: f32,
    pub saturation: f32,
    pub value: f32,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Default for ColorPickerState {
    fn default() -> Self {
        Self {
            primary_color: Color8::BLACK,
            secondary_color: Color8::WHITE,
            hue: 0.0,
            saturation: 0.0,
            value: 0.0,
            red: 0,
            green: 0,
            blue: 0,
        }
    }
}

impl ColorPickerState {
    pub fn set_color(&mut self, color: Color8) {
        self.primary_color = color;
        self.red = color.r;
        self.green = color.g;
        self.blue = color.b;
        self.hsv_from_rgb();
    }
    
    pub fn set_hsv(&mut self, h: f32, s: f32, v: f32) {
        self.hue = h;
        self.saturation = s;
        self.value = v;
        self.rgb_from_hsv();
    }
    
    pub fn set_rgb(&mut self, r: u8, g: u8, b: u8) {
        self.red = r;
        self.green = g;
        self.blue = b;
        self.primary_color = Color8::new(r, g, b, 255);
        self.hsv_from_rgb();
    }
    
    fn hsv_from_rgb(&mut self) {
        let r = self.red as f32 / 255.0;
        let g = self.green as f32 / 255.0;
        let b = self.blue as f32 / 255.0;
        
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let diff = max - min;
        
        self.value = max;
        
        if max == 0.0 {
            self.saturation = 0.0;
        } else {
            self.saturation = diff / max;
        }
        
        if diff == 0.0 {
            self.hue = 0.0;
        } else if max == r {
            self.hue = (60.0 * ((g - b) / diff) + 360.0) % 360.0;
        } else if max == g {
            self.hue = (60.0 * ((b - r) / diff) + 120.0) % 360.0;
        } else {
            self.hue = (60.0 * ((r - g) / diff) + 240.0) % 360.0;
        }
    }
    
    fn rgb_from_hsv(&mut self) {
        let h = self.hue / 60.0;
        let s = self.saturation;
        let v = self.value;
        
        let c = v * s;
        let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
        let m = v - c;
        
        let (r, g, b) = match h.floor() as i32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        
        self.red = ((r + m) * 255.0) as u8;
        self.green = ((g + m) * 255.0) as u8;
        self.blue = ((b + m) * 255.0) as u8;
        self.primary_color = Color8::new(self.red, self.green, self.blue, 255);
    }
}

pub fn view(state: &ColorPickerState) -> Element<'static, Message> {
    let color_preview = row![
        container(
            text(format!("红:{:02X}\n绿:{:02X}\n蓝:{:02X}", 
                state.primary_color.r, 
                state.primary_color.g, 
                state.primary_color.b)).size(8)
        )
            .width(Length::Fixed(50.0))
            .height(Length::Fixed(50.0))
            .style(container_style),
        container(text("样").size(10))
            .width(Length::Fixed(50.0))
            .height(Length::Fixed(50.0))
            .style(container_style),
    ]
    .spacing(4);

    let hue_slider = column![
        text(format!("色相: {:.0}", state.hue)).size(10),
        slider(0.0..=360.0, state.hue, |v| Message::ColorChanged(ColorMessage::HueChanged(v)))
            .width(Fill),
    ];

    let sat_slider = column![
        text(format!("饱和度: {:.0}%", state.saturation * 100.0)).size(10),
        slider(0.0..=1.0, state.saturation, |v| Message::ColorChanged(ColorMessage::SaturationChanged(v)))
            .width(Fill),
    ];

    let val_slider = column![
        text(format!("明度: {:.0}%", state.value * 100.0)).size(10),
        slider(0.0..=1.0, state.value, |v| Message::ColorChanged(ColorMessage::ValueChanged(v)))
            .width(Fill),
    ];

    let r_slider = column![
        text(format!("红: {}", state.red)).size(10),
        slider(0.0..=255.0, state.red as f32, |v| Message::ColorChanged(ColorMessage::RedChanged(v as u8)))
            .width(Fill),
    ];

    let g_slider = column![
        text(format!("绿: {}", state.green)).size(10),
        slider(0.0..=255.0, state.green as f32, |v| Message::ColorChanged(ColorMessage::GreenChanged(v as u8)))
            .width(Fill),
    ];

    let b_slider = column![
        text(format!("蓝: {}", state.blue)).size(10),
        slider(0.0..=255.0, state.blue as f32, |v| Message::ColorChanged(ColorMessage::BlueChanged(v as u8)))
            .width(Fill),
    ];

    let preset_colors = row![
        color_preset_button(Color8::BLACK, state.primary_color),
        color_preset_button(Color8::WHITE, state.primary_color),
        color_preset_button(Color8::new(255, 0, 0, 255), state.primary_color),
        color_preset_button(Color8::new(0, 255, 0, 255), state.primary_color),
        color_preset_button(Color8::new(0, 0, 255, 255), state.primary_color),
        color_preset_button(Color8::new(255, 255, 0, 255), state.primary_color),
        color_preset_button(Color8::new(255, 0, 255, 255), state.primary_color),
        color_preset_button(Color8::new(0, 255, 255, 255), state.primary_color),
    ]
    .spacing(2);

    let content = column![
        text("颜色").size(14),
        color_preview,
        text("预设").size(11),
        preset_colors,
        text("色相/饱和度/明度").size(11),
        hue_slider,
        sat_slider,
        val_slider,
        text("红/绿/蓝").size(11),
        r_slider,
        g_slider,
        b_slider,
    ]
    .spacing(6);

    container(content)
        .width(Length::Fixed(180.0))
        .height(Length::Fill)
        .padding(8)
        .style(panel_style)
        .into()
}

fn color_preset_button(color: Color8, _current: Color8) -> Element<'static, Message> {
    let iced_color = Color::from_rgb8(color.r, color.g, color.b);
    button("")
        .on_press(Message::ColorChanged(ColorMessage::PresetSelected(color)))
        .width(Length::Fixed(30.0))
        .height(Length::Fixed(30.0))
        .style(move |_theme, _status| iced::widget::button::Style {
            background: Some(iced::Background::Color(iced_color)),
            border: iced::Border {
                color: Color::from_rgb8(80, 80, 80),
                width: 1.0,
                radius: iced::border::Radius::new(4.0),
            },
            ..Default::default()
        })
        .into()
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

fn container_style(_: &iced::Theme) -> iced::widget::container::Style {
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
