use serde::{Deserialize, Serialize};
use crate::{Point, Rect, Matrix2D};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MotionBlurType {
    None,
    Linear,
    Radial,
    Zoom,
}

impl Default for MotionBlurType {
    fn default() -> Self {
        MotionBlurType::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraKey {
    pub position: Point,
    pub zoom: f64,
    pub rotation: f64,
    pub anchor: Point,
}

impl CameraKey {
    pub fn identity() -> Self {
        Self {
            position: Point::ZERO,
            zoom: 1.0,
            rotation: 0.0,
            anchor: Point::ZERO,
        }
    }

    pub fn new(position: Point, zoom: f64, rotation: f64) -> Self {
        Self {
            position,
            zoom,
            rotation,
            anchor: Point::ZERO,
        }
    }

    pub fn with_anchor(mut self, anchor: Point) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn to_matrix(&self, canvas_size: (f64, f64)) -> Matrix2D {
        let cx = canvas_size.0 / 2.0;
        let cy = canvas_size.1 / 2.0;
        
        let anchor_offset_x = cx - self.anchor.x;
        let anchor_offset_y = cy - self.anchor.y;

        let translate_to_anchor = Matrix2D::translation(-anchor_offset_x, -anchor_offset_y);
        let scale = Matrix2D::scaling(self.zoom, self.zoom);
        let rotate = Matrix2D::rotation(self.rotation);
        let translate_back = Matrix2D::translation(
            anchor_offset_x + self.position.x,
            anchor_offset_y + self.position.y,
        );

        translate_to_anchor
            .multiply(&scale)
            .multiply(&rotate)
            .multiply(&translate_back)
    }

    pub fn interpolate(&self, other: &CameraKey, t: f64) -> CameraKey {
        CameraKey {
            position: Point::new(
                self.position.x + (other.position.x - self.position.x) * t,
                self.position.y + (other.position.y - self.position.y) * t,
            ),
            zoom: self.zoom + (other.zoom - self.zoom) * t,
            rotation: self.rotation + (other.rotation - self.rotation) * t,
            anchor: Point::new(
                self.anchor.x + (other.anchor.x - self.anchor.x) * t,
                self.anchor.y + (other.anchor.y - self.anchor.y) * t,
            ),
        }
    }
}

impl Default for CameraKey {
    fn default() -> Self {
        Self::identity()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotionBlurSettings {
    pub enabled: bool,
    pub blur_type: MotionBlurType,
    pub shutter_angle: f64,
    pub shutter_phase: f64,
    pub samples: u32,
}

impl Default for MotionBlurSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            blur_type: MotionBlurType::Linear,
            shutter_angle: 180.0,
            shutter_phase: 0.0,
            samples: 8,
        }
    }
}

impl MotionBlurSettings {
    pub fn new(shutter_angle: f64, samples: u32) -> Self {
        Self {
            enabled: true,
            shutter_angle,
            samples,
            ..Default::default()
        }
    }

    pub fn calculate_motion_blur(
        &self,
        camera_start: &CameraKey,
        camera_end: &CameraKey,
        canvas_size: (f64, f64),
    ) -> Vec<Matrix2D> {
        if !self.enabled || self.samples < 2 {
            return vec![camera_start.to_matrix(canvas_size)];
        }

        let mut matrices = Vec::with_capacity(self.samples as usize);
        
        let start_offset = self.shutter_phase / 360.0;
        let end_offset = start_offset + self.shutter_angle / 360.0;

        for i in 0..self.samples {
            let t = start_offset + (end_offset - start_offset) * (i as f64 / (self.samples - 1) as f64);
            let interpolated = camera_start.interpolate(camera_end, t);
            matrices.push(interpolated.to_matrix(canvas_size));
        }

        matrices
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraTrack {
    pub keys: Vec<(u32, CameraKey)>,
    pub motion_blur: MotionBlurSettings,
}

impl CameraTrack {
    pub fn new() -> Self {
        Self {
            keys: vec![(0, CameraKey::identity())],
            motion_blur: MotionBlurSettings::default(),
        }
    }

    pub fn add_key(&mut self, frame: u32, key: CameraKey) {
        let pos = self.keys.binary_search_by_key(&frame, |k| k.0);
        match pos {
            Ok(idx) => self.keys[idx].1 = key,
            Err(idx) => self.keys.insert(idx, (frame, key)),
        }
    }

    pub fn remove_key(&mut self, frame: u32) -> Option<CameraKey> {
        let pos = self.keys.binary_search_by_key(&frame, |k| k.0);
        pos.ok().map(|idx| self.keys.remove(idx).1)
    }

    pub fn get_key(&self, frame: u32) -> Option<&CameraKey> {
        self.keys.binary_search_by_key(&frame, |k| k.0)
            .ok()
            .map(|idx| &self.keys[idx].1)
    }

    pub fn evaluate(&self, frame: u32) -> CameraKey {
        if self.keys.is_empty() {
            return CameraKey::identity();
        }

        let pos = self.keys.binary_search_by_key(&frame, |k| k.0);
        
        match pos {
            Ok(idx) => self.keys[idx].1.clone(),
            Err(0) => self.keys[0].1.clone(),
            Err(idx) if idx >= self.keys.len() => self.keys.last().expect("keys non-empty checked above").1.clone(),
            Err(idx) => {
                let (frame1, key1) = &self.keys[idx - 1];
                let (frame2, key2) = &self.keys[idx];
                let t = (frame - frame1) as f64 / (frame2 - frame1) as f64;
                key1.interpolate(key2, t)
            }
        }
    }

    pub fn get_blur_matrices(&self, frame: u32, canvas_size: (f64, f64)) -> Vec<Matrix2D> {
        if !self.motion_blur.enabled || self.motion_blur.samples < 2 {
            return vec![self.evaluate(frame).to_matrix(canvas_size)];
        }

        let camera_start = self.evaluate(frame);
        let camera_end = self.evaluate(frame + 1);
        
        self.motion_blur.calculate_motion_blur(&camera_start, &camera_end, canvas_size)
    }
}

impl Default for CameraTrack {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn hd() -> Self {
        Self::new(1280, 720)
    }

    pub fn full_hd() -> Self {
        Self::new(1920, 1080)
    }

    pub fn uhd() -> Self {
        Self::new(3840, 2160)
    }

    pub fn aspect_ratio(&self) -> f64 {
        if self.height == 0 {
            return 0.0;
        }
        self.width as f64 / self.height as f64
    }

    pub fn safe_area(&self, percent: f64) -> Rect {
        let margin_x = self.width as f64 * percent / 100.0;
        let margin_y = self.height as f64 * percent / 100.0;
        Rect::new(
            margin_x,
            margin_y,
            self.width as f64 - margin_x * 2.0,
            self.height as f64 - margin_y * 2.0,
        )
    }

    pub fn center(&self) -> Point {
        Point::new(self.width as f64 / 2.0, self.height as f64 / 2.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraLayerData {
    pub resolution: Resolution,
    pub frame_rate: f64,
    pub track: CameraTrack,
    pub background_color: crate::Color8,
    pub field_of_view: f64,
}

impl CameraLayerData {
    pub fn new(width: u32, height: u32, frame_rate: f64) -> Self {
        Self {
            resolution: Resolution::new(width, height),
            frame_rate,
            track: CameraTrack::new(),
            background_color: crate::Color8::WHITE,
            field_of_view: 90.0,
        }
    }

    pub fn full_hd(frame_rate: f64) -> Self {
        Self::new(1920, 1080, frame_rate)
    }

    pub fn add_camera_key(&mut self, frame: u32, key: CameraKey) {
        self.track.add_key(frame, key);
    }

    pub fn get_camera(&self, frame: u32) -> CameraKey {
        self.track.evaluate(frame)
    }

    pub fn get_transform(&self, frame: u32) -> Matrix2D {
        let camera = self.get_camera(frame);
        camera.to_matrix((self.resolution.width as f64, self.resolution.height as f64))
    }

    pub fn pan(&mut self, frame: u32, delta: Point) {
        let mut key = self.get_camera(frame);
        key.position = Point::new(
            key.position.x + delta.x,
            key.position.y + delta.y,
        );
        self.add_camera_key(frame, key);
    }

    pub fn zoom(&mut self, frame: u32, factor: f64) {
        let mut key = self.get_camera(frame);
        key.zoom *= factor;
        key.zoom = key.zoom.max(0.1).min(100.0);
        self.add_camera_key(frame, key);
    }

    pub fn rotate(&mut self, frame: u32, angle: f64) {
        let mut key = self.get_camera(frame);
        key.rotation += angle;
        self.add_camera_key(frame, key);
    }

    pub fn set_motion_blur(&mut self, enabled: bool, shutter_angle: f64, samples: u32) {
        self.track.motion_blur.enabled = enabled;
        self.track.motion_blur.shutter_angle = shutter_angle;
        self.track.motion_blur.samples = samples;
    }
}

impl Default for CameraLayerData {
    fn default() -> Self {
        Self::full_hd(24.0)
    }
}

pub fn apply_camera_transform(
    image: &[u8],
    image_width: u32,
    image_height: u32,
    transform: &Matrix2D,
    output_width: u32,
    output_height: u32,
) -> Vec<u8> {
    let mut result = vec![0u8; (output_width * output_height * 4) as usize];
    
    if let Some(inverse) = transform.inverse() {
        for y in 0..output_height {
            for x in 0..output_width {
                let src_point = inverse.transform_point(&Point::new(x as f64, y as f64));
                
                let src_x = src_point.x as i32;
                let src_y = src_point.y as i32;
                
                if src_x >= 0 && src_x < image_width as i32 
                    && src_y >= 0 && src_y < image_height as i32 
                {
                    let src_idx = (src_y as u32 * image_width + src_x as u32) as usize * 4;
                    let dst_idx = (y * output_width + x) as usize * 4;
                    
                    if src_idx + 3 < image.len() && dst_idx + 3 < result.len() {
                        result[dst_idx] = image[src_idx];
                        result[dst_idx + 1] = image[src_idx + 1];
                        result[dst_idx + 2] = image[src_idx + 2];
                        result[dst_idx + 3] = image[src_idx + 3];
                    }
                }
            }
        }
    }
    
    result
}

pub fn composite_with_motion_blur(
    frames: &[Vec<u8>],
    width: u32,
    height: u32,
) -> Vec<u8> {
    if frames.is_empty() {
        return vec![0u8; (width * height * 4) as usize];
    }
    
    if frames.len() == 1 {
        return frames[0].clone();
    }
    
    let mut result = vec![0.0f64; (width * height * 4) as usize];
    let weight = 1.0 / frames.len() as f64;
    
    for frame in frames {
        for (i, &pixel) in frame.iter().enumerate() {
            result[i] += pixel as f64 * weight;
        }
    }
    
    result.iter().map(|&v| v.round() as u8).collect()
}
