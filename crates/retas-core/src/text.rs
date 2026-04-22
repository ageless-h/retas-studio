use serde::{Deserialize, Serialize};
use crate::{Point, Color8, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
}

impl Default for FontWeight {
    fn default() -> Self {
        FontWeight::Normal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

impl Default for FontStyle {
    fn default() -> Self {
        FontStyle::Normal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Justify,
}

impl Default for TextAlignment {
    fn default() -> Self {
        TextAlignment::Left
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TextVerticalAlignment {
    Top,
    Middle,
    Bottom,
}

impl Default for TextVerticalAlignment {
    fn default() -> Self {
        TextVerticalAlignment::Top
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TextDecoration {
    None,
    Underline,
    Strikethrough,
    Overline,
}

impl Default for TextDecoration {
    fn default() -> Self {
        TextDecoration::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStyle {
    pub font_family: String,
    pub font_size: f64,
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub color: Color8,
    pub background_color: Option<Color8>,
    pub alignment: TextAlignment,
    pub vertical_alignment: TextVerticalAlignment,
    pub line_height: f64,
    pub letter_spacing: f64,
    pub decoration: TextDecoration,
    pub shadow: Option<TextShadow>,
    pub stroke: Option<TextStroke>,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_family: "Arial".to_string(),
            font_size: 24.0,
            font_weight: FontWeight::Normal,
            font_style: FontStyle::Normal,
            color: Color8::BLACK,
            background_color: None,
            alignment: TextAlignment::Left,
            vertical_alignment: TextVerticalAlignment::Top,
            line_height: 1.2,
            letter_spacing: 0.0,
            decoration: TextDecoration::None,
            shadow: None,
            stroke: None,
        }
    }
}

impl TextStyle {
    pub fn new(font_size: f64, color: Color8) -> Self {
        Self {
            font_size,
            color,
            ..Default::default()
        }
    }

    pub fn with_font_family(mut self, family: impl Into<String>) -> Self {
        self.font_family = family.into();
        self
    }

    pub fn with_weight(mut self, weight: FontWeight) -> Self {
        self.font_weight = weight;
        self
    }

    pub fn with_alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn with_shadow(mut self, shadow: TextShadow) -> Self {
        self.shadow = Some(shadow);
        self
    }

    pub fn with_stroke(mut self, stroke: TextStroke) -> Self {
        self.stroke = Some(stroke);
        self
    }

    pub fn with_background(mut self, color: Color8) -> Self {
        self.background_color = Some(color);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextShadow {
    pub offset_x: f64,
    pub offset_y: f64,
    pub blur_radius: f64,
    pub color: Color8,
}

impl TextShadow {
    pub fn new(offset_x: f64, offset_y: f64, blur: f64, color: Color8) -> Self {
        Self {
            offset_x,
            offset_y,
            blur_radius: blur,
            color,
        }
    }

    pub fn simple(offset: f64, color: Color8) -> Self {
        Self::new(offset, offset, 2.0, color)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStroke {
    pub width: f64,
    pub color: Color8,
}

impl TextStroke {
    pub fn new(width: f64, color: Color8) -> Self {
        Self { width, color }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextRun {
    pub text: String,
    pub style: TextStyle,
    pub start_index: usize,
    pub end_index: usize,
}

impl TextRun {
    pub fn new(text: impl Into<String>, style: TextStyle) -> Self {
        let text = text.into();
        let len = text.len();
        Self {
            text,
            style,
            start_index: 0,
            end_index: len,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextLayerData {
    pub text: String,
    pub style: TextStyle,
    pub position: Point,
    pub bounds: Option<Rect>,
    pub rotation: f64,
    pub runs: Vec<TextRun>,
}

impl TextLayerData {
    pub fn new(text: impl Into<String>, style: TextStyle, position: Point) -> Self {
        Self {
            text: text.into(),
            style,
            position,
            bounds: None,
            rotation: 0.0,
            runs: Vec::new(),
        }
    }

    pub fn simple(text: impl Into<String>, font_size: f64, position: Point) -> Self {
        Self::new(text, TextStyle::new(font_size, Color8::BLACK), position)
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.bounds = None;
    }

    pub fn set_style(&mut self, style: TextStyle) {
        self.style = style;
        self.bounds = None;
    }

    pub fn set_position(&mut self, position: Point) {
        self.position = position;
    }

    pub fn add_run(&mut self, run: TextRun) {
        self.runs.push(run);
    }

    pub fn measure(&self) -> Rect {
        if let Some(bounds) = self.bounds {
            return bounds;
        }

        let line_height = self.style.font_size * self.style.line_height;
        let num_lines = self.text.lines().count().max(1);
        let height = line_height * num_lines as f64;
        
        let avg_char_width = self.style.font_size * 0.5;
        let max_line_length = self.text.lines().map(|l| l.len()).max().unwrap_or(0);
        let width = avg_char_width * max_line_length as f64;

        Rect::new(self.position.x, self.position.y, width, height)
    }

    pub fn layout_lines(&self, max_width: Option<f64>) -> Vec<TextLine> {
        let mut lines = Vec::new();
        let line_height = self.style.font_size * self.style.line_height;
        
        for (line_idx, line_text) in self.text.lines().enumerate() {
            let y = self.position.y + line_idx as f64 * line_height;
            
            let char_width = self.style.font_size * 0.5 * (1.0 + self.style.letter_spacing / 100.0);
            let text_width = line_text.len() as f64 * char_width;
            
            let x = match self.style.alignment {
                TextAlignment::Left => self.position.x,
                TextAlignment::Center => {
                    if let Some(max_w) = max_width {
                        self.position.x + (max_w - text_width) / 2.0
                    } else {
                        self.position.x
                    }
                }
                TextAlignment::Right => {
                    if let Some(max_w) = max_width {
                        self.position.x + max_w - text_width
                    } else {
                        self.position.x
                    }
                }
                TextAlignment::Justify => self.position.x,
            };
            
            lines.push(TextLine {
                text: line_text.to_string(),
                x,
                y,
                width: text_width,
                height: line_height,
            });
        }
        
        lines
    }
}

impl Default for TextLayerData {
    fn default() -> Self {
        Self::new("", TextStyle::default(), Point::ZERO)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextLine {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

pub fn rasterize_text(
    text: &TextLayerData,
    width: u32,
    height: u32,
) -> Vec<u8> {
    let mut buffer = vec![0u8; (width * height * 4) as usize];
    
    let lines = text.layout_lines(None);
    
    let char_width = text.style.font_size * 0.5;
    let char_height = text.style.font_size;
    
    for line in &lines {
        for (char_idx, _ch) in line.text.chars().enumerate() {
            let cx = (line.x + char_idx as f64 * char_width) as i32;
            let cy = line.y as i32;
            
            let start_x = cx.max(0);
            let start_y = cy.max(0);
            let end_x = (cx + char_width as i32).min(width as i32);
            let end_y = (cy + char_height as i32).min(height as i32);
            
            for py in start_y..end_y {
                for px in start_x..end_x {
                    let idx = (py as u32 * width + px as u32) as usize * 4;
                    
                    if idx + 3 < buffer.len() {
                        let dx = (px - cx) as f64 / char_width - 0.5;
                        let dy = (py - cy) as f64 / char_height - 0.5;
                        let dist = (dx * dx + dy * dy).sqrt();
                        
                        let alpha = if dist < 0.4 { 255u8 } else if dist < 0.5 { 128u8 } else { 0u8 };
                        
                        if alpha > 0 {
                            buffer[idx] = text.style.color.r;
                            buffer[idx + 1] = text.style.color.g;
                            buffer[idx + 2] = text.style.color.b;
                            buffer[idx + 3] = alpha;
                        }
                    }
                }
            }
        }
    }
    
    buffer
}
