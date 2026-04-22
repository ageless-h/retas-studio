use crate::Color8;
use super::effects::{Effect, EffectParameters, EffectType};

pub struct EffectProcessor;

impl EffectProcessor {
    pub fn apply_effect(image: &[u8], width: u32, height: u32, effect: &Effect) -> Vec<u8> {
        if !effect.enabled {
            return image.to_vec();
        }

        let result = match effect.effect_type {
            EffectType::Blur | EffectType::GaussianBlur => {
                Self::apply_blur(image, width, height, &effect.parameters)
            }
            EffectType::BrightnessContrast 
            | EffectType::HueSaturation
            | EffectType::Saturation
            | EffectType::Invert
            | EffectType::Threshold
            | EffectType::Posterize
            | EffectType::ColorOverlay => {
                Self::apply_color_effect(image, &effect.parameters)
            }
            EffectType::Glow | EffectType::OuterGlow => {
                Self::apply_glow(image, width, height, &effect.parameters)
            }
            _ => image.to_vec(),
        };

        if effect.opacity < 1.0 {
            Self::blend_with_original(image, &result, effect.opacity)
        } else {
            result
        }
    }

    pub fn apply_blur(image: &[u8], width: u32, height: u32, params: &EffectParameters) -> Vec<u8> {
        let radius = match params {
            EffectParameters::Blur { radius } => *radius,
            EffectParameters::GaussianBlur { radius } => *radius,
            _ => 5.0,
        };

        super::effects::apply_blur_kernel(image, width, height, radius)
    }

    pub fn apply_color_effect(image: &[u8], params: &EffectParameters) -> Vec<u8> {
        let mut result = image.to_vec();

        for i in (0..result.len()).step_by(4) {
            if i + 3 >= result.len() {
                break;
            }

            let color = Color8::new(result[i], result[i + 1], result[i + 2], result[i + 3]);
            let adjusted = super::effects::apply_color_adjustment(color, params);

            result[i] = adjusted.r;
            result[i + 1] = adjusted.g;
            result[i + 2] = adjusted.b;
            result[i + 3] = adjusted.a;
        }

        result
    }

    pub fn apply_glow(image: &[u8], width: u32, height: u32, params: &EffectParameters) -> Vec<u8> {
        let (radius, color, _spread) = match params {
            EffectParameters::Glow { radius, color, spread } => (*radius, *color, *spread),
            EffectParameters::OuterGlow { radius, color, spread, .. } => (*radius, *color, *spread),
            _ => (10.0, Color8::new(255, 255, 200, 255), 0.0),
        };

        let mut glow_layer = vec![0u8; image.len()];
        
        for i in (0..image.len()).step_by(4) {
            if i + 3 >= image.len() {
                break;
            }

            let alpha = image[i + 3] as f64 / 255.0;
            if alpha > 0.1 {
                glow_layer[i] = color.r;
                glow_layer[i + 1] = color.g;
                glow_layer[i + 2] = color.b;
                glow_layer[i + 3] = (alpha * 255.0) as u8;
            }
        }

        let blurred = super::effects::apply_blur_kernel(&glow_layer, width, height, radius);

        Self::blend_screen(image, &blurred)
    }

    fn blend_with_original(original: &[u8], effect: &[u8], opacity: f64) -> Vec<u8> {
        let mut result = vec![0u8; original.len()];

        for i in (0..original.len()).step_by(4) {
            if i + 3 >= original.len() {
                break;
            }

            let orig_alpha = original[i + 3] as f64 / 255.0;
            let effect_alpha = effect[i + 3] as f64 / 255.0 * opacity;

            let out_alpha = orig_alpha + effect_alpha * (1.0 - orig_alpha);

            if out_alpha > 0.0 {
                for c in 0..3 {
                    let orig_c = original[i + c] as f64 / 255.0;
                    let effect_c = effect[i + c] as f64 / 255.0;
                    let out_c = (orig_c * orig_alpha + effect_c * effect_alpha * (1.0 - orig_alpha)) / out_alpha;
                    result[i + c] = (out_c * 255.0).round().clamp(0.0, 255.0) as u8;
                }
            }

            result[i + 3] = (out_alpha * 255.0).round().clamp(0.0, 255.0) as u8;
        }

        result
    }

    fn blend_screen(base: &[u8], blend: &[u8]) -> Vec<u8> {
        let mut result = vec![0u8; base.len()];

        for i in (0..base.len()).step_by(4) {
            if i + 3 >= base.len() {
                break;
            }

            let base_alpha = base[i + 3] as f64 / 255.0;
            let blend_alpha = blend[i + 3] as f64 / 255.0;

            for c in 0..3 {
                let b = base[i + c] as f64 / 255.0;
                let l = blend[i + c] as f64 / 255.0;
                let out = b + l - b * l;
                result[i + c] = (out * 255.0).round().clamp(0.0, 255.0) as u8;
            }

            result[i + 3] = ((base_alpha + blend_alpha - base_alpha * blend_alpha) * 255.0).round().clamp(0.0, 255.0) as u8;
        }

        result
    }

    pub fn apply_effect_stack(image: &[u8], width: u32, height: u32, effects: &[Effect]) -> Vec<u8> {
        let mut result = image.to_vec();

        for effect in effects.iter().filter(|e| e.enabled) {
            result = Self::apply_effect(&result, width, height, effect);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brightness_contrast() {
        let image = vec![128u8; 16];
        let effect = Effect::new(EffectType::BrightnessContrast);
        let result = EffectProcessor::apply_effect(&image, 2, 2, &effect);
        assert_eq!(result.len(), image.len());
    }

    #[test]
    fn test_blur() {
        let image = vec![255u8, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255];
        let params = EffectParameters::GaussianBlur { radius: 2.0 };
        let result = EffectProcessor::apply_blur(&image, 2, 2, &params);
        assert_eq!(result.len(), image.len());
    }
}
