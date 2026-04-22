use crate::Color8;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FillMode {
    Normal,
    Smart,
    GapClosing,
}

#[derive(Debug, Clone)]
pub struct FillSettings {
    pub mode: FillMode,
    pub tolerance: f64,
    pub gap_closing_radius: u32,
    pub anti_aliasing: bool,
    pub fill_behind_lines: bool,
}

impl Default for FillSettings {
    fn default() -> Self {
        Self {
            mode: FillMode::Smart,
            tolerance: 32.0,
            gap_closing_radius: 2,
            anti_aliasing: true,
            fill_behind_lines: true,
        }
    }
}

impl FillSettings {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance = tolerance.clamp(0.0, 255.0);
        self
    }
    
    pub fn with_gap_radius(mut self, radius: u32) -> Self {
        self.gap_closing_radius = radius.clamp(0, 10);
        self
    }
}

pub struct ColoringEngine {
    pub settings: FillSettings,
}

impl ColoringEngine {
    pub fn new() -> Self {
        Self {
            settings: FillSettings::default(),
        }
    }
    
    pub fn smart_fill(
        &self,
        image: &[u8],
        width: u32,
        height: u32,
        start_x: u32,
        start_y: u32,
        fill_color: Color8,
    ) -> Vec<u8> {
        match self.settings.mode {
            FillMode::Normal => {
                self.scanline_fill(image, width, height, start_x, start_y, fill_color)
            }
            FillMode::Smart => {
                self.smart_fill_with_detection(image, width, height, start_x, start_y, fill_color)
            }
            FillMode::GapClosing => {
                self.gap_closing_fill(image, width, height, start_x, start_y, fill_color)
            }
        }
    }
    
    fn scanline_fill(
        &self,
        image: &[u8],
        width: u32,
        height: u32,
        start_x: u32,
        start_y: u32,
        fill_color: Color8,
    ) -> Vec<u8> {
        let mut result = image.to_vec();
        
        if start_x >= width || start_y >= height {
            return result;
        }
        
        let start_idx = ((start_y * width + start_x) * 4) as usize;
        if start_idx + 3 >= image.len() {
            return result;
        }
        
        let target_color = self.get_pixel(image, width, start_x, start_y);
        
        if self.color_distance(&target_color, &fill_color) < 1.0 {
            return result;
        }
        
        let mut queue = VecDeque::new();
        queue.push_back((start_x, start_y));
        
        let mut visited = vec![false; (width * height) as usize];
        
        while let Some((x, y)) = queue.pop_front() {
            let idx = (y * width + x) as usize;
            
            if visited[idx] {
                continue;
            }
            visited[idx] = true;
            
            let current_color = self.get_pixel(&result, width, x, y);
            
            if self.color_distance(&current_color, &target_color) > self.settings.tolerance {
                continue;
            }
            
            self.set_pixel(&mut result, width, x, y, fill_color);
            
            if y > 0 {
                queue.push_back((x, y - 1));
            }
            if y < height - 1 {
                queue.push_back((x, y + 1));
            }
            if x > 0 {
                queue.push_back((x - 1, y));
            }
            if x < width - 1 {
                queue.push_back((x + 1, y));
            }
        }
        
        result
    }
    
    fn smart_fill_with_detection(
        &self,
        image: &[u8],
        width: u32,
        height: u32,
        start_x: u32,
        start_y: u32,
        fill_color: Color8,
    ) -> Vec<u8> {
        let mut result = image.to_vec();
        
        if start_x >= width || start_y >= height {
            return result;
        }
        
        let boundary_mask = self.detect_boundaries(image, width, height);
        let closed_area = self.find_closed_area(&boundary_mask, width, height, start_x, start_y);
        
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                if closed_area[idx] {
                    let is_boundary = boundary_mask[idx];
                    
                    if self.settings.fill_behind_lines && is_boundary {
                        continue;
                    }
                    
                    if self.settings.anti_aliasing && is_boundary {
                        let current = self.get_pixel(&result, width, x, y);
                        let blended = self.blend_colors(&current, &fill_color, 0.5);
                        self.set_pixel(&mut result, width, x, y, blended);
                    } else {
                        self.set_pixel(&mut result, width, x, y, fill_color);
                    }
                }
            }
        }
        
        result
    }
    
    fn gap_closing_fill(
        &self,
        image: &[u8],
        width: u32,
        height: u32,
        start_x: u32,
        start_y: u32,
        fill_color: Color8,
    ) -> Vec<u8> {
        let radius = self.settings.gap_closing_radius;
        
        if radius == 0 {
            return self.scanline_fill(image, width, height, start_x, start_y, fill_color);
        }
        
        let dilated = self.dilate_boundaries(image, width, height, radius);
        
        let closed_fill = self.scanline_fill(&dilated, width, height, start_x, start_y, fill_color);
        
        self.merge_fill_result(image, &closed_fill, width, height, fill_color)
    }
    
    fn detect_boundaries(&self, image: &[u8], width: u32, height: u32) -> Vec<bool> {
        let mut boundaries = vec![false; (width * height) as usize];
        
        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let idx = (y * width + x) as usize;
                let current = self.get_pixel(image, width, x, y);
                
                let left = self.get_pixel(image, width, x - 1, y);
                let right = self.get_pixel(image, width, x + 1, y);
                let top = self.get_pixel(image, width, x, y - 1);
                let bottom = self.get_pixel(image, width, x, y + 1);
                
                let threshold = self.settings.tolerance;
                
                if self.color_distance(&current, &left) > threshold
                    || self.color_distance(&current, &right) > threshold
                    || self.color_distance(&current, &top) > threshold
                    || self.color_distance(&current, &bottom) > threshold
                {
                    boundaries[idx] = true;
                }
                
                if current.a > 10 {
                    boundaries[idx] = true;
                }
            }
        }
        
        boundaries
    }
    
    fn find_closed_area(
        &self,
        boundary_mask: &[bool],
        width: u32,
        height: u32,
        start_x: u32,
        start_y: u32,
    ) -> Vec<bool> {
        let mut area = vec![false; (width * height) as usize];
        let mut queue = VecDeque::new();
        
        let start_idx = (start_y * width + start_x) as usize;
        if boundary_mask[start_idx] {
            return area;
        }
        
        queue.push_back((start_x, start_y));
        area[start_idx] = true;
        
        while let Some((x, y)) = queue.pop_front() {
            let neighbors = [(x + 1, y), (x.saturating_sub(1), y), (x, y + 1), (x, y.saturating_sub(1))];
            
            for (nx, ny) in neighbors {
                if nx >= width || ny >= height {
                    continue;
                }
                
                let idx = (ny * width + nx) as usize;
                
                if !area[idx] && !boundary_mask[idx] {
                    area[idx] = true;
                    queue.push_back((nx, ny));
                }
            }
        }
        
        area
    }
    
    fn dilate_boundaries(
        &self,
        image: &[u8],
        width: u32,
        height: u32,
        radius: u32,
    ) -> Vec<u8> {
        let mut result = image.to_vec();
        let boundaries = self.detect_boundaries(image, width, height);
        
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                
                if boundaries[idx] {
                    for dy in 0..=radius * 2 {
                        for dx in 0..=radius * 2 {
                            let nx = x.saturating_add(dx).saturating_sub(radius);
                            let ny = y.saturating_add(dy).saturating_sub(radius);
                            
                            if nx < width && ny < height {
                                let nidx = (ny * width + nx) as usize;
                                let pixel_idx = nidx * 4;
                                
                                if pixel_idx + 3 < result.len() {
                                    result[pixel_idx] = 0;
                                    result[pixel_idx + 1] = 0;
                                    result[pixel_idx + 2] = 0;
                                    result[pixel_idx + 3] = 255;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        result
    }
    
    fn merge_fill_result(
        &self,
        original: &[u8],
        filled: &[u8],
        width: u32,
        height: u32,
        fill_color: Color8,
    ) -> Vec<u8> {
        let mut result = original.to_vec();
        
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                
                if idx + 3 >= result.len() {
                    continue;
                }
                
                let filled_pixel = &filled[idx..idx + 4];
                let is_filled = filled_pixel[0] == fill_color.r
                    && filled_pixel[1] == fill_color.g
                    && filled_pixel[2] == fill_color.b;
                
                if is_filled {
                    let original_alpha = result[idx + 3];
                    
                    if original_alpha < 10 || self.settings.fill_behind_lines {
                        result[idx] = fill_color.r;
                        result[idx + 1] = fill_color.g;
                        result[idx + 2] = fill_color.b;
                        result[idx + 3] = fill_color.a;
                    }
                }
            }
        }
        
        result
    }
    
    fn get_pixel(&self, image: &[u8], width: u32, x: u32, y: u32) -> Color8 {
        let idx = ((y * width + x) * 4) as usize;
        Color8::new(image[idx], image[idx + 1], image[idx + 2], image[idx + 3])
    }
    
    fn set_pixel(&self, image: &mut [u8], width: u32, x: u32, y: u32, color: Color8) {
        let idx = ((y * width + x) * 4) as usize;
        image[idx] = color.r;
        image[idx + 1] = color.g;
        image[idx + 2] = color.b;
        image[idx + 3] = color.a;
    }
    
    fn color_distance(&self, a: &Color8, b: &Color8) -> f64 {
        let dr = (a.r as i32 - b.r as i32).abs() as f64;
        let dg = (a.g as i32 - b.g as i32).abs() as f64;
        let db = (a.b as i32 - b.b as i32).abs() as f64;
        let da = (a.a as i32 - b.a as i32).abs() as f64;
        
        (dr + dg + db + da) / 4.0
    }
    
    fn blend_colors(&self, base: &Color8, blend: &Color8, alpha: f64) -> Color8 {
        let inv_alpha = 1.0 - alpha;
        
        Color8::new(
            (base.r as f64 * inv_alpha + blend.r as f64 * alpha) as u8,
            (base.g as f64 * inv_alpha + blend.g as f64 * alpha) as u8,
            (base.b as f64 * inv_alpha + blend.b as f64 * alpha) as u8,
            (base.a as f64 * inv_alpha + blend.a as f64 * alpha) as u8,
        )
    }
}

impl Default for ColoringEngine {
    fn default() -> Self {
        Self::new()
    }
}
