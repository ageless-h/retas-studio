use serde::{Deserialize, Serialize};
use crate::{Point, Rect};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SelectionMode {
    Replace,
    Add,
    Subtract,
    Intersect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SelectionTool {
    Rectangular,
    Elliptical,
    Lasso,
    PolygonLasso,
    MagicWand,
    QuickSelection,
}

impl Default for SelectionTool {
    fn default() -> Self {
        SelectionTool::Rectangular
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Selection {
    pub tool: SelectionTool,
    pub mode: SelectionMode,
    pub mask: SelectionMask,
    pub feather: f64,
    pub anti_aliased: bool,
    pub is_active: bool,
}

impl Default for Selection {
    fn default() -> Self {
        Self {
            tool: SelectionTool::Rectangular,
            mode: SelectionMode::Replace,
            mask: SelectionMask::None,
            feather: 0.0,
            anti_aliased: true,
            is_active: false,
        }
    }
}

impl Selection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rectangular(rect: Rect) -> Self {
        Self {
            tool: SelectionTool::Rectangular,
            mode: SelectionMode::Replace,
            mask: SelectionMask::Rectangular { rect },
            feather: 0.0,
            anti_aliased: true,
            is_active: true,
        }
    }

    pub fn elliptical(rect: Rect) -> Self {
        Self {
            tool: SelectionTool::Elliptical,
            mode: SelectionMode::Replace,
            mask: SelectionMask::Elliptical { rect },
            feather: 0.0,
            anti_aliased: true,
            is_active: true,
        }
    }

    pub fn lasso(points: Vec<Point>) -> Self {
        let bounds = Self::calculate_bounds(&points);
        Self {
            tool: SelectionTool::Lasso,
            mode: SelectionMode::Replace,
            mask: SelectionMask::Lasso { points, bounds },
            feather: 0.0,
            anti_aliased: true,
            is_active: true,
        }
    }

    pub fn magic_wand(
        start_point: Point,
        tolerance: f64,
        contiguous: bool,
        _sample_all_layers: bool,
    ) -> Self {
        Self {
            tool: SelectionTool::MagicWand,
            mode: SelectionMode::Replace,
            mask: SelectionMask::MagicWand {
                start_point,
                tolerance,
                contiguous,
                _sample_all_layers: false,
                pixels: HashSet::new(),
            },
            feather: 0.0,
            anti_aliased: true,
            is_active: true,
        }
    }

    pub fn none() -> Self {
        Self {
            tool: SelectionTool::Rectangular,
            mode: SelectionMode::Replace,
            mask: SelectionMask::None,
            feather: 0.0,
            anti_aliased: true,
            is_active: false,
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self.mask, SelectionMask::None)
    }

    pub fn bounds(&self) -> Option<Rect> {
        match &self.mask {
            SelectionMask::None => None,
            SelectionMask::Rectangular { rect } => Some(*rect),
            SelectionMask::Elliptical { rect } => Some(*rect),
            SelectionMask::Lasso { bounds, .. } => Some(*bounds),
            SelectionMask::MagicWand { .. } => None,
            SelectionMask::Bitmap { width, height, .. } => {
                Some(Rect::new(0.0, 0.0, *width as f64, *height as f64))
            }
        }
    }

    pub fn contains(&self, point: Point) -> bool {
        match &self.mask {
            SelectionMask::None => false,
            SelectionMask::Rectangular { rect } => rect.contains(&point),
            SelectionMask::Elliptical { rect } => {
                let cx = rect.origin.x + rect.size.width / 2.0;
                let cy = rect.origin.y + rect.size.height / 2.0;
                let rx = rect.size.width / 2.0;
                let ry = rect.size.height / 2.0;
                if rx == 0.0 || ry == 0.0 {
                    return false;
                }
                let dx = (point.x - cx) / rx;
                let dy = (point.y - cy) / ry;
                dx * dx + dy * dy <= 1.0
            }
            SelectionMask::Lasso { points, .. } => {
                Self::point_in_polygon(point, points)
            }
            SelectionMask::MagicWand { pixels, .. } => {
                let px = point.x as i32;
                let py = point.y as i32;
                pixels.contains(&(px, py))
            }
            SelectionMask::Bitmap { data, width, .. } => {
                let px = point.x as usize;
                let py = point.y as usize;
                let w = *width as usize;
                if px < w && py * w + px < data.len() {
                    data[py * w + px] > 127
                } else {
                    false
                }
            }
        }
    }

    fn point_in_polygon(point: Point, polygon: &[Point]) -> bool {
        if polygon.len() < 3 {
            return false;
        }

        let mut inside = false;
        let mut j = polygon.len() - 1;

        for i in 0..polygon.len() {
            let pi = polygon[i];
            let pj = polygon[j];

            if ((pi.y > point.y) != (pj.y > point.y))
                && (point.x < (pj.x - pi.x) * (point.y - pi.y) / (pj.y - pi.y) + pi.x)
            {
                inside = !inside;
            }
            j = i;
        }

        inside
    }

    fn calculate_bounds(points: &[Point]) -> Rect {
        if points.is_empty() {
            return Rect::ZERO;
        }

        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for p in points {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }

        Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    pub fn combine(&self, other: &Selection, mode: SelectionMode) -> Selection {
        match mode {
            SelectionMode::Replace => other.clone(),
            SelectionMode::Add => self.add(other),
            SelectionMode::Subtract => self.subtract(other),
            SelectionMode::Intersect => self.intersect(other),
        }
    }

    fn add(&self, other: &Selection) -> Selection {
        if self.is_empty() {
            return other.clone();
        }
        if other.is_empty() {
            return self.clone();
        }

        let mut result = self.clone();
        result.mask = match (&self.mask, &other.mask) {
            (SelectionMask::None, m) | (m, SelectionMask::None) => m.clone(),
            (SelectionMask::Bitmap { .. }, _) | (_, SelectionMask::Bitmap { .. }) => {
                self.to_bitmap().merge_bitmap(&other.to_bitmap(), MergeOp::Add)
            }
            _ => self.to_bitmap().merge_bitmap(&other.to_bitmap(), MergeOp::Add),
        };
        result
    }

    fn subtract(&self, other: &Selection) -> Selection {
        if self.is_empty() || other.is_empty() {
            return self.clone();
        }

        let mut result = self.clone();
        result.mask = self.to_bitmap().merge_bitmap(&other.to_bitmap(), MergeOp::Subtract);
        result
    }

    fn intersect(&self, other: &Selection) -> Selection {
        if self.is_empty() || other.is_empty() {
            return Selection::none();
        }

        let mut result = self.clone();
        result.mask = self.to_bitmap().merge_bitmap(&other.to_bitmap(), MergeOp::Intersect);
        result
    }

    pub fn invert(&self, width: u32, height: u32) -> Selection {
        if self.is_empty() {
            return Selection::rectangular(Rect::new(0.0, 0.0, width as f64, height as f64));
        }

        let mut result = self.clone();
        result.mask = self.to_bitmap().invert(width, height);
        result
    }

    pub fn feather(&mut self, radius: f64) {
        self.feather = radius;
    }

    pub fn to_bitmap(&self) -> SelectionMask {
        match &self.mask {
            SelectionMask::Bitmap { .. } => self.mask.clone(),
            SelectionMask::None => SelectionMask::Bitmap {
                data: Vec::new(),
                width: 0,
                height: 0,
            },
            _ => {
                let bounds = self.bounds().unwrap_or(Rect::new(0.0, 0.0, 100.0, 100.0));
                let width = bounds.size.width.ceil() as u32;
                let height = bounds.size.height.ceil() as u32;
                
                let mut data = vec![0u8; (width * height) as usize];
                
                for y in 0..height {
                    for x in 0..width {
                        let point = Point::new(
                            bounds.origin.x + x as f64,
                            bounds.origin.y + y as f64,
                        );
                        if self.contains(point) {
                            data[(y * width + x) as usize] = 255;
                        }
                    }
                }

                SelectionMask::Bitmap { data, width, height }
            }
        }
    }

    pub fn to_mask_image(&self) -> Option<Vec<u8>> {
        match &self.mask {
            SelectionMask::Bitmap { data, width, height } => {
                let mut rgba = Vec::with_capacity((width * height * 4) as usize);
                for &alpha in data {
                    rgba.extend_from_slice(&[255, 255, 255, alpha]);
                }
                Some(rgba)
            }
            _ => {
                let bitmap = self.to_bitmap();
                if let SelectionMask::Bitmap { data, width, height } = bitmap {
                    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
                    for alpha in data {
                        rgba.extend_from_slice(&[255, 255, 255, alpha]);
                    }
                    Some(rgba)
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectionMask {
    None,
    Rectangular { rect: Rect },
    Elliptical { rect: Rect },
    Lasso { points: Vec<Point>, bounds: Rect },
    MagicWand {
        start_point: Point,
        tolerance: f64,
        contiguous: bool,
        _sample_all_layers: bool,
        pixels: HashSet<(i32, i32)>,
    },
    Bitmap {
        data: Vec<u8>,
        width: u32,
        height: u32,
    },
}

impl SelectionMask {
    fn merge_bitmap(&self, other: &SelectionMask, op: MergeOp) -> SelectionMask {
        let b1 = match self {
            SelectionMask::Bitmap { data, width, height } => (data.clone(), *width, *height),
            _ => return self.clone(),
        };
        let b2 = match other {
            SelectionMask::Bitmap { data, width, height } => (data.clone(), *width, *height),
            _ => return self.clone(),
        };

        let width = b1.1.max(b2.1);
        let height = b1.2.max(b2.2);
        let mut result = vec![0u8; (width * height) as usize];

        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                let v1 = if x < b1.1 && y < b1.2 {
                    b1.0[(y * b1.1 + x) as usize]
                } else {
                    0
                };
                let v2 = if x < b2.1 && y < b2.2 {
                    b2.0[(y * b2.1 + x) as usize]
                } else {
                    0
                };

                result[idx] = match op {
                    MergeOp::Add => (v1 as u16).saturating_add(v2 as u16) as u8,
                    MergeOp::Subtract => v1.saturating_sub(v2),
                    MergeOp::Intersect => v1.min(v2),
                };
            }
        }

        SelectionMask::Bitmap { data: result, width, height }
    }

    fn invert(&self, width: u32, height: u32) -> SelectionMask {
        let bitmap = match self {
            SelectionMask::Bitmap { data, width: w, height: h } => {
                (data.clone(), *w, *h)
            }
            _ => return self.clone(),
        };

        let mut result = vec![255u8; (width * height) as usize];

        for y in 0..bitmap.2.min(height) {
            for x in 0..bitmap.1.min(width) {
                let src_idx = (y * bitmap.1 + x) as usize;
                let dst_idx = (y * width + x) as usize;
                result[dst_idx] = 255 - bitmap.0[src_idx];
            }
        }

        SelectionMask::Bitmap { data: result, width, height }
    }
}

#[derive(Debug, Clone, Copy)]
enum MergeOp {
    Add,
    Subtract,
    Intersect,
}

pub struct MagicWandSelector {
    tolerance: f64,
    contiguous: bool,
    _sample_all_layers: bool,
}

impl MagicWandSelector {
    pub fn new(tolerance: f64, contiguous: bool) -> Self {
        Self {
            tolerance,
            contiguous,
            _sample_all_layers: false,
        }
    }

    pub fn select(
        &self,
        image: &[u8],
        width: u32,
        height: u32,
        start_x: u32,
        start_y: u32,
    ) -> HashSet<(i32, i32)> {
        let start_idx = (start_y * width + start_x) as usize * 4;
        if start_idx + 3 >= image.len() {
            return HashSet::new();
        }

        let target_r = image[start_idx];
        let target_g = image[start_idx + 1];
        let target_b = image[start_idx + 2];
        let target_a = image[start_idx + 3];

        let mut selected = HashSet::new();
        let mut visited = vec![false; (width * height) as usize];
        let mut queue = std::collections::VecDeque::new();

        queue.push_back((start_x, start_y));
        visited[(start_y * width + start_x) as usize] = true;

        while let Some((x, y)) = queue.pop_front() {
            let idx = (y * width + x) as usize * 4;
            
            if idx + 3 < image.len() {
                let r = image[idx];
                let g = image[idx + 1];
                let b = image[idx + 2];
                let a = image[idx + 3];

                let diff = Self::color_distance(r, g, b, a, target_r, target_g, target_b, target_a);
                
                if diff <= self.tolerance {
                    selected.insert((x as i32, y as i32));

                    if self.contiguous {
                        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                            let nx = x as i32 + dx;
                            let ny = y as i32 + dy;
                            
                            if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                                let nidx = (ny as u32 * width + nx as u32) as usize;
                                if !visited[nidx] {
                                    visited[nidx] = true;
                                    queue.push_back((nx as u32, ny as u32));
                                }
                            }
                        }
                    }
                }
            }

            if !self.contiguous {
                break;
            }
        }

        if !self.contiguous {
            for y in 0..height {
                for x in 0..width {
                    let idx = (y * width + x) as usize * 4;
                    if idx + 3 < image.len() {
                        let r = image[idx];
                        let g = image[idx + 1];
                        let b = image[idx + 2];
                        let a = image[idx + 3];
                        
                        let diff = Self::color_distance(r, g, b, a, target_r, target_g, target_b, target_a);
                        
                        if diff <= self.tolerance {
                            selected.insert((x as i32, y as i32));
                        }
                    }
                }
            }
        }

        selected
    }

    fn color_distance(r1: u8, g1: u8, b1: u8, a1: u8, r2: u8, g2: u8, b2: u8, a2: u8) -> f64 {
        let dr = (r1 as i32 - r2 as i32).abs() as f64;
        let dg = (g1 as i32 - g2 as i32).abs() as f64;
        let db = (b1 as i32 - b2 as i32).abs() as f64;
        let da = (a1 as i32 - a2 as i32).abs() as f64;
        
        (dr * dr + dg * dg + db * db + da * da).sqrt()
    }
}
