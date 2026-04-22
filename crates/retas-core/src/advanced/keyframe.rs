use serde::{Deserialize, Serialize};
use crate::{Matrix2D, LayerId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Interpolation {
    None,
    Linear,
    Smooth,
    Bezier,
}

impl Default for Interpolation {
    fn default() -> Self {
        Interpolation::Linear
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyframe<T> {
    pub frame: u32,
    pub value: T,
    pub interpolation: Interpolation,
    pub in_handle: Option<(f64, f64)>,
    pub out_handle: Option<(f64, f64)>,
}

impl<T: Clone> Keyframe<T> {
    pub fn new(frame: u32, value: T) -> Self {
        Self {
            frame,
            value,
            interpolation: Interpolation::Linear,
            in_handle: None,
            out_handle: None,
        }
    }

    pub fn with_interpolation(mut self, interpolation: Interpolation) -> Self {
        self.interpolation = interpolation;
        self
    }

    pub fn with_handles(mut self, in_handle: (f64, f64), out_handle: (f64, f64)) -> Self {
        self.in_handle = Some(in_handle);
        self.out_handle = Some(out_handle);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationTrack<T> {
    pub name: String,
    pub keyframes: Vec<Keyframe<T>>,
    pub enabled: bool,
    pub locked: bool,
}

impl<T: Clone> AnimationTrack<T> {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            keyframes: Vec::new(),
            enabled: true,
            locked: false,
        }
    }

    pub fn add_keyframe(&mut self, keyframe: Keyframe<T>) {
        let pos = self.keyframes.binary_search_by_key(&keyframe.frame, |k| k.frame);
        match pos {
            Ok(idx) => self.keyframes[idx] = keyframe,
            Err(idx) => self.keyframes.insert(idx, keyframe),
        }
    }

    pub fn remove_keyframe(&mut self, frame: u32) -> Option<Keyframe<T>> {
        let pos = self.keyframes.binary_search_by_key(&frame, |k| k.frame);
        pos.ok().map(|idx| self.keyframes.remove(idx))
    }

    pub fn get_value(&self, frame: u32) -> Option<&T> {
        self.keyframes
            .binary_search_by_key(&frame, |k| k.frame)
            .ok()
            .map(|idx| &self.keyframes[idx].value)
    }

    pub fn get_keyframe(&self, frame: u32) -> Option<&Keyframe<T>> {
        self.keyframes
            .binary_search_by_key(&frame, |k| k.frame)
            .ok()
            .map(|idx| &self.keyframes[idx])
    }

    pub fn get_keyframe_mut(&mut self, frame: u32) -> Option<&mut Keyframe<T>> {
        let pos = self.keyframes.binary_search_by_key(&frame, |k| k.frame);
        pos.ok().map(|idx| &mut self.keyframes[idx])
    }

    pub fn has_keyframe_at(&self, frame: u32) -> bool {
        self.keyframes.binary_search_by_key(&frame, |k| k.frame).is_ok()
    }

    pub fn keyframe_count(&self) -> usize {
        self.keyframes.len()
    }
}

pub type TransformTrack = AnimationTrack<TransformKey>;
pub type OpacityTrack = AnimationTrack<f64>;
pub type VisibilityTrack = AnimationTrack<bool>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TransformKey {
    pub translation: (f64, f64),
    pub rotation: f64,
    pub scale: (f64, f64),
    pub skew: (f64, f64),
}

impl TransformKey {
    pub fn identity() -> Self {
        Self {
            translation: (0.0, 0.0),
            rotation: 0.0,
            scale: (1.0, 1.0),
            skew: (0.0, 0.0),
        }
    }

    pub fn translation(x: f64, y: f64) -> Self {
        Self {
            translation: (x, y),
            ..Self::identity()
        }
    }

    pub fn rotation(angle: f64) -> Self {
        Self {
            rotation: angle,
            ..Self::identity()
        }
    }

    pub fn scale(sx: f64, sy: f64) -> Self {
        Self {
            scale: (sx, sy),
            ..Self::identity()
        }
    }

    pub fn to_matrix(&self) -> Matrix2D {
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();
        
        Matrix2D::new(
            self.scale.0 * cos_r + self.skew.1 * sin_r,
            self.scale.0 * sin_r,
            self.skew.0 * cos_r - self.scale.1 * sin_r,
            self.skew.0 * sin_r + self.scale.1 * cos_r,
            self.translation.0,
            self.translation.1,
        )
    }
}

impl Default for TransformKey {
    fn default() -> Self {
        Self::identity()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerAnimation {
    pub layer_id: LayerId,
    pub transform: TransformTrack,
    pub opacity: OpacityTrack,
    pub visibility: VisibilityTrack,
    pub effects: Vec<AnimationTrack<f64>>,
}

impl LayerAnimation {
    pub fn new(layer_id: LayerId) -> Self {
        Self {
            layer_id,
            transform: TransformTrack::new("Transform"),
            opacity: OpacityTrack::new("Opacity"),
            visibility: VisibilityTrack::new("Visibility"),
            effects: Vec::new(),
        }
    }

    pub fn evaluate(&self, frame: u32) -> LayerAnimationState {
        let transform = if let Some(key) = self.transform.get_keyframe(frame) {
            key.value.clone()
        } else {
            self.interpolate_transform(frame)
        };

        let opacity = if let Some(&opacity) = self.opacity.get_value(frame) {
            opacity
        } else {
            self.interpolate_opacity(frame)
        };

        let visible = self.visibility.get_value(frame).copied().unwrap_or(true);

        LayerAnimationState {
            transform,
            opacity,
            visible,
        }
    }

    fn interpolate_transform(&self, frame: u32) -> TransformKey {
        if self.transform.keyframes.is_empty() {
            return TransformKey::identity();
        }

        let pos = self.transform.keyframes.binary_search_by_key(&frame, |k| k.frame);
        
        match pos {
            Ok(idx) => self.transform.keyframes[idx].value.clone(),
            Err(0) => self.transform.keyframes[0].value.clone(),
            Err(idx) if idx >= self.transform.keyframes.len() => {
                self.transform.keyframes.last().unwrap().value.clone()
            }
            Err(idx) => {
                let k1 = &self.transform.keyframes[idx - 1];
                let k2 = &self.transform.keyframes[idx];
                let t = (frame - k1.frame) as f64 / (k2.frame - k1.frame) as f64;
                
                TransformKey {
                    translation: (
                        k1.value.translation.0.lerp(k2.value.translation.0, t),
                        k1.value.translation.1.lerp(k2.value.translation.1, t),
                    ),
                    rotation: k1.value.rotation + (k2.value.rotation - k1.value.rotation) * t,
                    scale: (
                        k1.value.scale.0 + (k2.value.scale.0 - k1.value.scale.0) * t,
                        k1.value.scale.1 + (k2.value.scale.1 - k1.value.scale.1) * t,
                    ),
                    skew: (
                        k1.value.skew.0 + (k2.value.skew.0 - k1.value.skew.0) * t,
                        k1.value.skew.1 + (k2.value.skew.1 - k1.value.skew.1) * t,
                    ),
                }
            }
        }
    }

    fn interpolate_opacity(&self, frame: u32) -> f64 {
        if self.opacity.keyframes.is_empty() {
            return 1.0;
        }

        let pos = self.opacity.keyframes.binary_search_by_key(&frame, |k| k.frame);
        
        match pos {
            Ok(idx) => self.opacity.keyframes[idx].value,
            Err(0) => self.opacity.keyframes[0].value,
            Err(idx) if idx >= self.opacity.keyframes.len() => {
                self.opacity.keyframes.last().unwrap().value
            }
            Err(idx) => {
                let k1 = &self.opacity.keyframes[idx - 1];
                let k2 = &self.opacity.keyframes[idx];
                let t = (frame - k1.frame) as f64 / (k2.frame - k1.frame) as f64;
                k1.value + (k2.value - k1.value) * t
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerAnimationState {
    pub transform: TransformKey,
    pub opacity: f64,
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneAnimation {
    pub layer_animations: std::collections::HashMap<LayerId, LayerAnimation>,
    pub frame_rate: f64,
    pub start_frame: u32,
    pub end_frame: u32,
}

impl SceneAnimation {
    pub fn new() -> Self {
        Self {
            layer_animations: std::collections::HashMap::new(),
            frame_rate: 24.0,
            start_frame: 0,
            end_frame: 100,
        }
    }

    pub fn add_layer(&mut self, layer_id: LayerId) {
        self.layer_animations.insert(layer_id, LayerAnimation::new(layer_id));
    }

    pub fn remove_layer(&mut self, layer_id: LayerId) {
        self.layer_animations.remove(&layer_id);
    }

    pub fn evaluate(&self, frame: u32) -> std::collections::HashMap<LayerId, LayerAnimationState> {
        self.layer_animations
            .iter()
            .map(|(id, anim)| (*id, anim.evaluate(frame)))
            .collect()
    }

    pub fn duration_seconds(&self) -> f64 {
        (self.end_frame - self.start_frame) as f64 / self.frame_rate
    }

    pub fn frame_to_time(&self, frame: u32) -> f64 {
        (frame - self.start_frame) as f64 / self.frame_rate
    }

    pub fn time_to_frame(&self, time: f64) -> u32 {
        (time * self.frame_rate) as u32 + self.start_frame
    }
}

impl Default for SceneAnimation {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Interpolate: Clone {
    fn interpolate(&self, other: &Self, t: f64) -> Self;
}

impl Interpolate for f64 {
    fn interpolate(&self, other: &Self, t: f64) -> Self {
        self + (other - self) * t
    }
}

impl Interpolate for bool {
    fn interpolate(&self, _other: &Self, _t: f64) -> Self {
        *self
    }
}

impl Interpolate for TransformKey {
    fn interpolate(&self, other: &Self, t: f64) -> Self {
        TransformKey {
            translation: (
                self.translation.0 + (other.translation.0 - self.translation.0) * t,
                self.translation.1 + (other.translation.1 - self.translation.1) * t,
            ),
            rotation: self.rotation + (other.rotation - self.rotation) * t,
            scale: (
                self.scale.0 + (other.scale.0 - self.scale.0) * t,
                self.scale.1 + (other.scale.1 - self.scale.1) * t,
            ),
            skew: (
                self.skew.0 + (other.skew.0 - self.skew.0) * t,
                self.skew.1 + (other.skew.1 - self.skew.1) * t,
            ),
        }
    }
}

trait Lerp {
    fn lerp(self, other: Self, t: f64) -> Self;
}

impl Lerp for f64 {
    fn lerp(self, other: Self, t: f64) -> Self {
        self + (other - self) * t
    }
}
