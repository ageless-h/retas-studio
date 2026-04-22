use serde::{Deserialize, Serialize};
use crate::{Point, Color8};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorizationSettings {
    pub threshold: u8,
    pub smoothing: f64,
    pub corner_threshold: f64,
    pub optimization_tolerance: f64,
    pub min_path_length: f64,
    pub preserve_holes: bool,
    pub output_stroke_width: f64,
}

impl Default for VectorizationSettings {
    fn default() -> Self {
        Self {
            threshold: 128,
            smoothing: 1.0,
            corner_threshold: 45.0,
            optimization_tolerance: 0.5,
            min_path_length: 5.0,
            preserve_holes: true,
            output_stroke_width: 2.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorizedPath {
    pub points: Vec<VectorizedPoint>,
    pub is_closed: bool,
    pub color: Color8,
    pub stroke_width: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorizedPoint {
    pub position: Point,
    pub control_in: Option<Point>,
    pub control_out: Option<Point>,
    pub is_corner: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorizationResult {
    pub paths: Vec<VectorizedPath>,
    pub width: u32,
    pub height: u32,
    pub processing_time_ms: u64,
}

pub struct Vectorizer {
    settings: VectorizationSettings,
}

impl Vectorizer {
    pub fn new(settings: VectorizationSettings) -> Self {
        Self { settings }
    }

    pub fn with_default_settings() -> Self {
        Self {
            settings: VectorizationSettings::default(),
        }
    }

    pub fn settings(&self) -> &VectorizationSettings {
        &self.settings
    }

    pub fn set_settings(&mut self, settings: VectorizationSettings) {
        self.settings = settings;
    }

    pub fn vectorize_bitmap(&self, pixels: &[u8], width: u32, height: u32) -> VectorizationResult {
        let start = std::time::Instant::now();
        
        let binary = self.binarize(pixels, width, height);
        let contours = self.trace_contours(&binary, width, height);
        let paths = self.contours_to_paths(contours);
        let optimized = self.optimize_paths(paths);
        
        VectorizationResult {
            paths: optimized,
            width,
            height,
            processing_time_ms: start.elapsed().as_millis() as u64,
        }
    }

    pub fn vectorize_grayscale(&self, pixels: &[u8], width: u32, height: u32) -> VectorizationResult {
        let start = std::time::Instant::now();
        
        let binary = self.binarize_grayscale(pixels, width, height);
        let contours = self.trace_contours(&binary, width, height);
        let paths = self.contours_to_paths(contours);
        let optimized = self.optimize_paths(paths);
        
        VectorizationResult {
            paths: optimized,
            width,
            height,
            processing_time_ms: start.elapsed().as_millis() as u64,
        }
    }

    fn binarize(&self, pixels: &[u8], width: u32, height: u32) -> Vec<bool> {
        let mut binary = vec![false; (width * height) as usize];
        
        for (i, pixel) in pixels.chunks(4).enumerate() {
            let gray = if pixel.len() >= 3 {
                (pixel[0] as f64 * 0.299 + pixel[1] as f64 * 0.587 + pixel[2] as f64 * 0.114) as u8
            } else {
                pixel[0]
            };
            binary[i] = gray < self.settings.threshold;
        }
        
        binary
    }

    fn binarize_grayscale(&self, pixels: &[u8], _width: u32, _height: u32) -> Vec<bool> {
        pixels.iter().map(|&p| p < self.settings.threshold).collect()
    }

    fn trace_contours(&self, binary: &[bool], width: u32, height: u32) -> Vec<Vec<(i32, i32)>> {
        let mut contours = Vec::new();
        let mut visited = vec![false; binary.len()];
        
        for y in 0..height as i32 {
            for x in 0..width as i32 {
                let idx = (y * width as i32 + x) as usize;
                
                if !binary[idx] || visited[idx] {
                    continue;
                }
                
                if x > 0 && binary[((y * width as i32 + x - 1)) as usize] {
                    continue;
                }
                
                let contour = self.trace_single_contour(binary, &mut visited, x, y, width, height);
                if contour.len() >= 3 {
                    contours.push(contour);
                }
            }
        }
        
        contours
    }

    fn trace_single_contour(
        &self,
        binary: &[bool],
        visited: &mut [bool],
        start_x: i32,
        start_y: i32,
        width: u32,
        height: u32,
    ) -> Vec<(i32, i32)> {
        let mut contour = Vec::new();
        let mut x = start_x;
        let mut y = start_y;
        let mut dir = 0;
        
        let dx = [1, 1, 0, -1, -1, -1, 0, 1];
        let dy = [0, 1, 1, 1, 0, -1, -1, -1];
        
        loop {
            let idx = (y * width as i32 + x) as usize;
            if idx < visited.len() {
                visited[idx] = true;
            }
            contour.push((x, y));
            
            let mut found = false;
            for i in 0..8 {
                let new_dir = (dir + i) % 8;
                let nx = x + dx[new_dir];
                let ny = y + dy[new_dir];
                
                if nx < 0 || nx >= width as i32 || ny < 0 || ny >= height as i32 {
                    continue;
                }
                
                let nidx = (ny * width as i32 + nx) as usize;
                if binary[nidx] {
                    x = nx;
                    y = ny;
                    dir = (new_dir + 5) % 8;
                    found = true;
                    break;
                }
            }
            
            if !found || (x == start_x && y == start_y && contour.len() > 2) {
                break;
            }
        }
        
        contour
    }

    fn contours_to_paths(&self, contours: Vec<Vec<(i32, i32)>>) -> Vec<VectorizedPath> {
        contours
            .into_iter()
            .filter(|c| {
                let len = self.calculate_contour_length(c);
                len >= self.settings.min_path_length
            })
            .map(|contour| self.contour_to_path(contour))
            .collect()
    }

    fn calculate_contour_length(&self, contour: &[(i32, i32)]) -> f64 {
        if contour.len() < 2 {
            return 0.0;
        }
        
        let mut length = 0.0;
        for i in 1..contour.len() {
            let dx = (contour[i].0 - contour[i - 1].0) as f64;
            let dy = (contour[i].1 - contour[i - 1].1) as f64;
            length += (dx * dx + dy * dy).sqrt();
        }
        
        let dx = (contour[0].0 - contour[contour.len() - 1].0) as f64;
        let dy = (contour[0].1 - contour[contour.len() - 1].1) as f64;
        length += (dx * dx + dy * dy).sqrt();
        
        length
    }

    fn contour_to_path(&self, contour: Vec<(i32, i32)>) -> VectorizedPath {
        let points: Vec<VectorizedPoint> = contour
            .into_iter()
            .map(|(x, y)| VectorizedPoint {
                position: Point::new(x as f64, y as f64),
                control_in: None,
                control_out: None,
                is_corner: false,
            })
            .collect();
        
        let is_closed = points.len() > 2;
        
        VectorizedPath {
            points,
            is_closed,
            color: Color8::new(0, 0, 0, 255),
            stroke_width: self.settings.output_stroke_width,
        }
    }

    fn optimize_paths(&self, paths: Vec<VectorizedPath>) -> Vec<VectorizedPath> {
        paths
            .into_iter()
            .map(|path| self.optimize_path(path))
            .collect()
    }

    fn optimize_path(&self, mut path: VectorizedPath) -> VectorizedPath {
        if path.points.len() < 3 {
            return path;
        }
        
        let mut optimized = Vec::new();
        optimized.push(path.points[0].clone());
        
        for i in 1..path.points.len() - 1 {
            let prev = &path.points[i - 1].position;
            let curr = &path.points[i].position;
            let next = &path.points[i + 1].position;
            
            let v1 = Point::new(prev.x - curr.x, prev.y - curr.y);
            let v2 = Point::new(next.x - curr.x, next.y - curr.y);
            
            let len1 = (v1.x * v1.x + v1.y * v1.y).sqrt();
            let len2 = (v2.x * v2.x + v2.y * v2.y).sqrt();
            
            if len1 > 0.0 && len2 > 0.0 {
                let dot = (v1.x * v2.x + v1.y * v2.y) / (len1 * len2);
                let angle = dot.acos().to_degrees();
                
                let mut point = path.points[i].clone();
                point.is_corner = angle < self.settings.corner_threshold;
                optimized.push(point);
            }
        }
        
        optimized.push(path.points[path.points.len() - 1].clone());
        path.points = optimized;
        
        path
    }
}

impl Default for Vectorizer {
    fn default() -> Self {
        Self::with_default_settings()
    }
}
