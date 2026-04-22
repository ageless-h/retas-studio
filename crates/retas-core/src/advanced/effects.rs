use serde::{Deserialize, Serialize};
use crate::{Color8, ColorF, Point};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectType {
    Blur,
    GaussianBlur,
    MotionBlur,
    RadialBlur,
    Sharpen,
    UnsharpMask,
    Noise,
    Glow,
    OuterGlow,
    InnerGlow,
    DropShadow,
    InnerShadow,
    BevelEmboss,
    ColorOverlay,
    GradientOverlay,
    PatternOverlay,
    Saturation,
    BrightnessContrast,
    ColorBalance,
    Curves,
    Levels,
    HueSaturation,
    Invert,
    Threshold,
    Posterize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect {
    pub effect_type: EffectType,
    pub enabled: bool,
    pub opacity: f64,
    pub blend_mode: crate::layer::BlendMode,
    pub parameters: EffectParameters,
}

impl Effect {
    pub fn new(effect_type: EffectType) -> Self {
        Self {
            effect_type,
            enabled: true,
            opacity: 1.0,
            blend_mode: crate::layer::BlendMode::Normal,
            parameters: EffectParameters::default_for(effect_type),
        }
    }

    pub fn with_opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn with_blend_mode(mut self, mode: crate::layer::BlendMode) -> Self {
        self.blend_mode = mode;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectParameters {
    Blur { radius: f64 },
    GaussianBlur { radius: f64 },
    MotionBlur { angle: f64, distance: f64 },
    RadialBlur { amount: f64, center: Point, method: RadialBlurMethod },
    Sharpen { amount: f64 },
    UnsharpMask { amount: f64, radius: f64, threshold: f64 },
    Noise { amount: f64, uniform: bool, monochromatic: bool },
    Glow { radius: f64, color: Color8, spread: f64 },
    OuterGlow { radius: f64, color: Color8, spread: f64, offset: Point },
    InnerGlow { radius: f64, color: Color8, source: GlowSource },
    DropShadow { offset: Point, radius: f64, color: Color8, spread: f64, noise: f64 },
    InnerShadow { offset: Point, radius: f64, color: Color8, spread: f64 },
    BevelEmboss { style: BevelStyle, depth: f64, direction: EmbossDirection, size: f64, soften: f64, angle: f64, altitude: f64, highlight_color: Color8, shadow_color: Color8 },
    ColorOverlay { color: Color8 },
    GradientOverlay { gradient: Gradient, angle: f64, scale: f64, offset: Point },
    PatternOverlay { pattern_id: u32, scale: f64, offset: Point },
    Saturation { amount: f64 },
    BrightnessContrast { brightness: f64, contrast: f64 },
    ColorBalance { shadows: (i32, i32, i32), midtones: (i32, i32, i32), highlights: (i32, i32, i32), preserve_luminosity: bool },
    Curves { points: Vec<CurvePoint>, channel: ColorChannel },
    Levels { input_black: f64, input_white: f64, gamma: f64, output_black: f64, output_white: f64 },
    HueSaturation { hue: f64, saturation: f64, lightness: f64 },
    Invert,
    Threshold { level: f64 },
    Posterize { levels: u32 },
}

impl EffectParameters {
    pub fn default_for(effect_type: EffectType) -> Self {
        match effect_type {
            EffectType::Blur => EffectParameters::Blur { radius: 5.0 },
            EffectType::GaussianBlur => EffectParameters::GaussianBlur { radius: 5.0 },
            EffectType::MotionBlur => EffectParameters::MotionBlur { angle: 0.0, distance: 10.0 },
            EffectType::RadialBlur => EffectParameters::RadialBlur {
                amount: 10.0,
                center: Point::new(0.5, 0.5),
                method: RadialBlurMethod::Spin,
            },
            EffectType::Sharpen => EffectParameters::Sharpen { amount: 50.0 },
            EffectType::UnsharpMask => EffectParameters::UnsharpMask {
                amount: 50.0,
                radius: 1.0,
                threshold: 0.0,
            },
            EffectType::Noise => EffectParameters::Noise {
                amount: 10.0,
                uniform: false,
                monochromatic: false,
            },
            EffectType::Glow => EffectParameters::Glow {
                radius: 10.0,
                color: Color8::WHITE,
                spread: 0.0,
            },
            EffectType::OuterGlow => EffectParameters::OuterGlow {
                radius: 10.0,
                color: Color8::WHITE,
                spread: 0.0,
                offset: Point::ZERO,
            },
            EffectType::InnerGlow => EffectParameters::InnerGlow {
                radius: 10.0,
                color: Color8::WHITE,
                source: GlowSource::Edge,
            },
            EffectType::DropShadow => EffectParameters::DropShadow {
                offset: Point::new(5.0, 5.0),
                radius: 10.0,
                color: Color8::new(0, 0, 0, 128),
                spread: 0.0,
                noise: 0.0,
            },
            EffectType::InnerShadow => EffectParameters::InnerShadow {
                offset: Point::new(5.0, 5.0),
                radius: 10.0,
                color: Color8::new(0, 0, 0, 128),
                spread: 0.0,
            },
            EffectType::BevelEmboss => EffectParameters::BevelEmboss {
                style: BevelStyle::InnerBevel,
                depth: 100.0,
                direction: EmbossDirection::Up,
                size: 5.0,
                soften: 0.0,
                angle: 120.0,
                altitude: 30.0,
                highlight_color: Color8::WHITE,
                shadow_color: Color8::BLACK,
            },
            EffectType::ColorOverlay => EffectParameters::ColorOverlay { color: Color8::WHITE },
            EffectType::GradientOverlay => EffectParameters::GradientOverlay {
                gradient: Gradient::default(),
                angle: 90.0,
                scale: 100.0,
                offset: Point::ZERO,
            },
            EffectType::PatternOverlay => EffectParameters::PatternOverlay {
                pattern_id: 0,
                scale: 100.0,
                offset: Point::ZERO,
            },
            EffectType::Saturation => EffectParameters::Saturation { amount: 0.0 },
            EffectType::BrightnessContrast => EffectParameters::BrightnessContrast {
                brightness: 0.0,
                contrast: 0.0,
            },
            EffectType::ColorBalance => EffectParameters::ColorBalance {
                shadows: (0, 0, 0),
                midtones: (0, 0, 0),
                highlights: (0, 0, 0),
                preserve_luminosity: true,
            },
            EffectType::Curves => EffectParameters::Curves {
                points: vec![
                    CurvePoint { input: 0.0, output: 0.0 },
                    CurvePoint { input: 255.0, output: 255.0 },
                ],
                channel: ColorChannel::RGB,
            },
            EffectType::Levels => EffectParameters::Levels {
                input_black: 0.0,
                input_white: 255.0,
                gamma: 1.0,
                output_black: 0.0,
                output_white: 255.0,
            },
            EffectType::HueSaturation => EffectParameters::HueSaturation {
                hue: 0.0,
                saturation: 0.0,
                lightness: 0.0,
            },
            EffectType::Invert => EffectParameters::Invert,
            EffectType::Threshold => EffectParameters::Threshold { level: 128.0 },
            EffectType::Posterize => EffectParameters::Posterize { levels: 4 },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RadialBlurMethod {
    Spin,
    Zoom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GlowSource {
    Center,
    Edge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BevelStyle {
    OuterBevel,
    InnerBevel,
    Emboss,
    PillowEmboss,
    StrokeEmboss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EmbossDirection {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ColorChannel {
    RGB,
    Red,
    Green,
    Blue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurvePoint {
    pub input: f64,
    pub output: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gradient {
    pub stops: Vec<GradientStop>,
    pub gradient_type: GradientType,
    pub interpolation: GradientInterpolation,
}

impl Default for Gradient {
    fn default() -> Self {
        Self {
            stops: vec![
                GradientStop { position: 0.0, color: Color8::BLACK },
                GradientStop { position: 1.0, color: Color8::WHITE },
            ],
            gradient_type: GradientType::Linear,
            interpolation: GradientInterpolation::Linear,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GradientType {
    Linear,
    Radial,
    Angle,
    Reflected,
    Diamond,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GradientInterpolation {
    Linear,
    Perceptual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientStop {
    pub position: f64,
    pub color: Color8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectStack {
    pub effects: Vec<Effect>,
}

impl EffectStack {
    pub fn new() -> Self {
        Self { effects: Vec::new() }
    }

    pub fn add(&mut self, effect: Effect) {
        self.effects.push(effect);
    }

    pub fn remove(&mut self, index: usize) -> Option<Effect> {
        if index < self.effects.len() {
            Some(self.effects.remove(index))
        } else {
            None
        }
    }

    pub fn reorder(&mut self, from: usize, to: usize) {
        if from < self.effects.len() && to < self.effects.len() {
            let effect = self.effects.remove(from);
            self.effects.insert(to, effect);
        }
    }

    pub fn enabled_effects(&self) -> impl Iterator<Item = &Effect> {
        self.effects.iter().filter(|e| e.enabled)
    }
}

impl Default for EffectStack {
    fn default() -> Self {
        Self::new()
    }
}

pub fn apply_color_adjustment(color: Color8, params: &EffectParameters) -> Color8 {
    let mut cf = ColorF::from(color);

    match params {
        EffectParameters::BrightnessContrast { brightness, contrast } => {
            cf.r = (cf.r + brightness / 100.0).clamp(0.0, 1.0);
            cf.g = (cf.g + brightness / 100.0).clamp(0.0, 1.0);
            cf.b = (cf.b + brightness / 100.0).clamp(0.0, 1.0);
            
            let factor = (259.0 * (contrast + 255.0)) / (255.0 * (259.0 - contrast));
            cf.r = (factor * (cf.r * 255.0 - 128.0) + 128.0) / 255.0;
            cf.g = (factor * (cf.g * 255.0 - 128.0) + 128.0) / 255.0;
            cf.b = (factor * (cf.b * 255.0 - 128.0) + 128.0) / 255.0;
        }
        EffectParameters::HueSaturation { hue, saturation, lightness } => {
            let (h, s, v) = cf.to_hsv();
            let new_h = (h + *hue).rem_euclid(360.0);
            let new_s = (s + *saturation / 100.0).clamp(0.0, 1.0);
            let new_v = (v + *lightness / 100.0).clamp(0.0, 1.0);
            cf = ColorF::from_hsv(new_h, new_s, new_v);
        }
        EffectParameters::Invert => {
            cf.r = 1.0 - cf.r;
            cf.g = 1.0 - cf.g;
            cf.b = 1.0 - cf.b;
        }
        EffectParameters::Threshold { level } => {
            let gray = 0.299 * cf.r + 0.587 * cf.g + 0.114 * cf.b;
            let threshold = level / 255.0;
            let result = if gray > threshold { 1.0 } else { 0.0 };
            cf.r = result;
            cf.g = result;
            cf.b = result;
        }
        EffectParameters::Posterize { levels } => {
            let levels = *levels as f64;
            cf.r = (cf.r * levels).floor() / levels;
            cf.g = (cf.g * levels).floor() / levels;
            cf.b = (cf.b * levels).floor() / levels;
        }
        EffectParameters::ColorOverlay { color } => {
            let overlay = ColorF::from(*color);
            cf.r = overlay.r;
            cf.g = overlay.g;
            cf.b = overlay.b;
        }
        EffectParameters::Saturation { amount } => {
            let (h, s, v) = cf.to_hsv();
            let new_s = (s + *amount / 100.0).clamp(0.0, 1.0);
            cf = ColorF::from_hsv(h, new_s, v);
        }
        _ => {}
    }

    Color8::from(cf)
}

pub fn apply_blur_kernel(data: &[u8], width: u32, height: u32, radius: f64) -> Vec<u8> {
    if radius <= 0.0 {
        return data.to_vec();
    }

    let radius = radius.ceil() as usize;
    let mut result = vec![0u8; data.len()];

    let kernel_size = radius * 2 + 1;
    let sigma = radius as f64 / 3.0;
    let mut kernel = vec![0.0f64; kernel_size];
    
    for i in 0..kernel_size {
        let x = (i as isize - radius as isize) as f64;
        kernel[i] = (-x * x / (2.0 * sigma * sigma)).exp();
    }
    
    let sum: f64 = kernel.iter().sum();
    for k in &mut kernel {
        *k /= sum;
    }

    let mut temp = vec![0.0f64; data.len()];
    
    for y in 0..height {
        for x in 0..width {
            for c in 0..4 {
                let mut sum = 0.0;
                let mut weight_sum = 0.0;
                
                for k in 0..kernel_size {
                    let sx = (x as isize + k as isize - radius as isize)
                        .max(0)
                        .min(width as isize - 1) as usize;
                    let idx = (y * width + sx as u32) as usize * 4 + c;
                    sum += data[idx] as f64 * kernel[k];
                    weight_sum += kernel[k];
                }
                
                let idx = (y * width + x) as usize * 4 + c;
                temp[idx] = sum / weight_sum;
            }
        }
    }

    for x in 0..width {
        for y in 0..height {
            for c in 0..4 {
                let mut sum = 0.0;
                let mut weight_sum = 0.0;
                
                for k in 0..kernel_size {
                    let sy = (y as isize + k as isize - radius as isize)
                        .max(0)
                        .min(height as isize - 1) as usize;
                    let idx = (sy as u32 * width + x) as usize * 4 + c;
                    sum += temp[idx] * kernel[k];
                    weight_sum += kernel[k];
                }
                
                let idx = (y * width + x) as usize * 4 + c;
                result[idx] = (sum / weight_sum).round().clamp(0.0, 255.0) as u8;
            }
        }
    }

    result
}
