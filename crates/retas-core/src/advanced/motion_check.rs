use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{Point, LayerId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotionCheck {
    pub enabled: bool,
    pub mode: MotionCheckMode,
    pub comparison_frames: Vec<u32>,
    pub overlay_opacity: f64,
    pub show_trails: bool,
    pub trail_length: u32,
}

impl Default for MotionCheck {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: MotionCheckMode::Overlay,
            comparison_frames: vec![1],
            overlay_opacity: 0.5,
            show_trails: false,
            trail_length: 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MotionCheckMode {
    Overlay,
    Difference,
    SideBySide,
    OnionSkin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotionTrail {
    pub layer_id: LayerId,
    pub points: Vec<TrailPoint>,
    pub color: [f32; 4],
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrailPoint {
    pub position: Point,
    pub frame: u32,
    pub size: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotionCheckManager {
    checks: HashMap<u32, MotionCheck>,
    trails: HashMap<LayerId, MotionTrail>,
    current_frame: u32,
}

impl MotionCheckManager {
    pub fn new() -> Self {
        Self {
            checks: HashMap::new(),
            trails: HashMap::new(),
            current_frame: 0,
        }
    }

    pub fn enable(&mut self, frame: u32) {
        let check = self.checks.entry(frame).or_insert_with(MotionCheck::default);
        check.enabled = true;
    }

    pub fn disable(&mut self, frame: u32) {
        if let Some(check) = self.checks.get_mut(&frame) {
            check.enabled = false;
        }
    }

    pub fn toggle(&mut self, frame: u32) {
        let check = self.checks.entry(frame).or_insert_with(MotionCheck::default);
        check.enabled = !check.enabled;
    }

    pub fn set_mode(&mut self, frame: u32, mode: MotionCheckMode) {
        let check = self.checks.entry(frame).or_insert_with(MotionCheck::default);
        check.mode = mode;
    }

    pub fn set_comparison_frames(&mut self, frame: u32, frames: Vec<u32>) {
        let check = self.checks.entry(frame).or_insert_with(MotionCheck::default);
        check.comparison_frames = frames;
    }

    pub fn set_overlay_opacity(&mut self, frame: u32, opacity: f64) {
        let check = self.checks.entry(frame).or_insert_with(MotionCheck::default);
        check.overlay_opacity = opacity.clamp(0.0, 1.0);
    }

    pub fn get(&self, frame: u32) -> Option<&MotionCheck> {
        self.checks.get(&frame)
    }

    pub fn get_mut(&mut self, frame: u32) -> &mut MotionCheck {
        self.checks.entry(frame).or_insert_with(MotionCheck::default)
    }

    pub fn set_current_frame(&mut self, frame: u32) {
        self.current_frame = frame;
    }

    pub fn add_trail_point(&mut self, layer_id: LayerId, position: Point, frame: u32, size: f64) {
        let trail = self.trails.entry(layer_id).or_insert_with(|| MotionTrail {
            layer_id,
            points: Vec::new(),
            color: [1.0, 0.0, 0.0, 0.5],
            visible: true,
        });
        
        trail.points.push(TrailPoint {
            position,
            frame,
            size,
        });
        
        if trail.points.len() > 100 {
            trail.points.remove(0);
        }
    }

    pub fn get_trail(&self, layer_id: LayerId) -> Option<&MotionTrail> {
        self.trails.get(&layer_id)
    }

    pub fn clear_trail(&mut self, layer_id: LayerId) {
        if let Some(trail) = self.trails.get_mut(&layer_id) {
            trail.points.clear();
        }
    }

    pub fn clear_all_trails(&mut self) {
        for trail in self.trails.values_mut() {
            trail.points.clear();
        }
    }

    pub fn set_trail_visible(&mut self, layer_id: LayerId, visible: bool) {
        if let Some(trail) = self.trails.get_mut(&layer_id) {
            trail.visible = visible;
        }
    }

    pub fn get_comparison_info(&self, frame: u32) -> Option<ComparisonInfo> {
        let check = self.checks.get(&frame)?;
        
        if !check.enabled {
            return None;
        }
        
        Some(ComparisonInfo {
            mode: check.mode,
            comparison_frames: check.comparison_frames.clone(),
            overlay_opacity: check.overlay_opacity,
        })
    }
}

impl Default for MotionCheckManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonInfo {
    pub mode: MotionCheckMode,
    pub comparison_frames: Vec<u32>,
    pub overlay_opacity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotionPath {
    pub id: u64,
    pub layer_id: LayerId,
    pub keyframes: Vec<MotionKeyframe>,
    pub interpolation: InterpolationType,
    pub visible: bool,
    pub color: [f32; 4],
}

impl MotionPath {
    pub fn new(layer_id: LayerId) -> Self {
        Self {
            id: 0,
            layer_id,
            keyframes: Vec::new(),
            interpolation: InterpolationType::Linear,
            visible: true,
            color: [0.0, 1.0, 1.0, 0.8],
        }
    }

    pub fn add_keyframe(&mut self, frame: u32, position: Point) {
        let keyframe = MotionKeyframe {
            frame,
            position,
            handle_in: None,
            handle_out: None,
        };
        
        let pos = self.keyframes.iter().position(|k| k.frame >= frame);
        match pos {
            Some(idx) => {
                if self.keyframes[idx].frame == frame {
                    self.keyframes[idx] = keyframe;
                } else {
                    self.keyframes.insert(idx, keyframe);
                }
            }
            None => self.keyframes.push(keyframe),
        }
    }

    pub fn remove_keyframe(&mut self, frame: u32) -> bool {
        let len = self.keyframes.len();
        self.keyframes.retain(|k| k.frame != frame);
        self.keyframes.len() != len
    }

    pub fn get_position_at(&self, frame: u32) -> Option<Point> {
        if self.keyframes.is_empty() {
            return None;
        }
        
        if frame <= self.keyframes[0].frame {
            return Some(self.keyframes[0].position);
        }
        
        if frame >= self.keyframes.last()?.frame {
            return Some(self.keyframes.last()?.position);
        }
        
        for i in 0..self.keyframes.len() - 1 {
            let k1 = &self.keyframes[i];
            let k2 = &self.keyframes[i + 1];
            
            if frame >= k1.frame && frame <= k2.frame {
                let t = (frame - k1.frame) as f64 / (k2.frame - k1.frame) as f64;
                return Some(match self.interpolation {
                    InterpolationType::Linear => Point::new(
                        k1.position.x + (k2.position.x - k1.position.x) * t,
                        k1.position.y + (k2.position.y - k1.position.y) * t,
                    ),
                    InterpolationType::Smooth => {
                        let t = t * t * (3.0 - 2.0 * t);
                        Point::new(
                            k1.position.x + (k2.position.x - k1.position.x) * t,
                            k1.position.y + (k2.position.y - k1.position.y) * t,
                        )
                    }
                    InterpolationType::Bezier => {
                        if let (Some(h1_out), Some(h2_in)) = (k1.handle_out, k2.handle_in) {
                            let t2 = t * t;
                            let t3 = t2 * t;
                            let mt = 1.0 - t;
                            let mt2 = mt * mt;
                            let mt3 = mt2 * mt;
                            
                            Point::new(
                                mt3 * k1.position.x + 3.0 * mt2 * t * h1_out.x + 3.0 * mt * t2 * h2_in.x + t3 * k2.position.x,
                                mt3 * k1.position.y + 3.0 * mt2 * t * h1_out.y + 3.0 * mt * t2 * h2_in.y + t3 * k2.position.y,
                            )
                        } else {
                            let t = t * t * (3.0 - 2.0 * t);
                            Point::new(
                                k1.position.x + (k2.position.x - k1.position.x) * t,
                                k1.position.y + (k2.position.y - k1.position.y) * t,
                            )
                        }
                    }
                });
            }
        }
        
        None
    }

    pub fn get_path_points(&self, start_frame: u32, end_frame: u32) -> Vec<Point> {
        let mut points = Vec::new();
        
        for frame in start_frame..=end_frame {
            if let Some(pos) = self.get_position_at(frame) {
                points.push(pos);
            }
        }
        
        points
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotionKeyframe {
    pub frame: u32,
    pub position: Point,
    pub handle_in: Option<Point>,
    pub handle_out: Option<Point>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InterpolationType {
    Linear,
    Smooth,
    Bezier,
}
