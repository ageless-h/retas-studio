use iced::{Color, Theme};

pub fn dark_theme() -> Theme {
    Theme::custom(
        "RETAS Dark".to_string(),
        iced::theme::Palette {
            background: Color::from_rgb8(30, 30, 30),
            text: Color::from_rgb8(220, 220, 220),
            primary: Color::from_rgb8(70, 130, 180),
            success: Color::from_rgb8(76, 175, 80),
            danger: Color::from_rgb8(244, 67, 54),
            warning: Color::from_rgb8(255, 193, 7),
        },
    )
}

pub fn light_theme() -> Theme {
    Theme::custom(
        "RETAS Light".to_string(),
        iced::theme::Palette {
            background: Color::from_rgb8(240, 240, 240),
            text: Color::from_rgb8(30, 30, 30),
            primary: Color::from_rgb8(70, 130, 180),
            success: Color::from_rgb8(76, 175, 80),
            danger: Color::from_rgb8(244, 67, 54),
            warning: Color::from_rgb8(255, 193, 7),
        },
    )
}
