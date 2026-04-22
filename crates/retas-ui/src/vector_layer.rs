use retas_core::{Color8, Point};
use retas_vector::{BezierCurve, BezierControlPoint};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VectorLayerType {
    Line,
    Paint,
}

#[derive(Debug, Clone)]
pub struct VectorPath {
    pub id: u64,
    pub curve: BezierCurve,
    pub fill_color: Option<Color8>,
    pub stroke_color: Option<Color8>,
    pub stroke_width: f32,
    pub visible: bool,
    pub selected: bool,
}

impl VectorPath {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            curve: BezierCurve::new(),
            fill_color: None,
            stroke_color: Some(Color8::BLACK),
            stroke_width: 2.0,
            visible: true,
            selected: false,
        }
    }
    
    pub fn add_point(&mut self, point: Point) {
        self.curve.points.push(BezierControlPoint::corner(point));
    }
    
    pub fn close_path(&mut self) {
        self.curve.closed = true;
    }
    
    pub fn is_empty(&self) -> bool {
        self.curve.points.is_empty()
    }
    
    pub fn point_count(&self) -> usize {
        self.curve.points.len()
    }
}

#[derive(Debug, Clone)]
pub struct VectorLayer {
    pub id: u64,
    pub name: String,
    pub layer_type: VectorLayerType,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,
    pub paths: Vec<VectorPath>,
    pub selected_path: Option<usize>,
}

impl VectorLayer {
    pub fn new(id: u64, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            layer_type: VectorLayerType::Line,
            visible: true,
            locked: false,
            opacity: 1.0,
            paths: Vec::new(),
            selected_path: None,
        }
    }
    
    pub fn create_path(&mut self) -> usize {
        let id = self.paths.len() as u64;
        let path = VectorPath::new(id);
        self.paths.push(path);
        self.selected_path = Some(self.paths.len() - 1);
        self.paths.len() - 1
    }
    
    pub fn get_selected_path(&self) -> Option<&VectorPath> {
        self.selected_path.and_then(|idx| self.paths.get(idx))
    }
    
    pub fn get_selected_path_mut(&mut self) -> Option<&mut VectorPath> {
        self.selected_path.and_then(|idx| self.paths.get_mut(idx))
    }
    
    pub fn select_path(&mut self, idx: usize) {
        if idx < self.paths.len() {
            self.selected_path = Some(idx);
        }
    }
    
    pub fn delete_path(&mut self, idx: usize) {
        if idx < self.paths.len() {
            self.paths.remove(idx);
            if self.paths.is_empty() {
                self.selected_path = None;
            } else if self.selected_path == Some(idx) {
                self.selected_path = Some(idx.saturating_sub(1));
            }
        }
    }
    
    pub fn move_path_up(&mut self, idx: usize) -> bool {
        if idx > 0 && idx < self.paths.len() {
            self.paths.swap(idx, idx - 1);
            if self.selected_path == Some(idx) {
                self.selected_path = Some(idx - 1);
            } else if self.selected_path == Some(idx - 1) {
                self.selected_path = Some(idx);
            }
            true
        } else {
            false
        }
    }
    
    pub fn move_path_down(&mut self, idx: usize) -> bool {
        if idx < self.paths.len().saturating_sub(1) {
            self.paths.swap(idx, idx + 1);
            if self.selected_path == Some(idx) {
                self.selected_path = Some(idx + 1);
            } else if self.selected_path == Some(idx + 1) {
                self.selected_path = Some(idx);
            }
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PenToolMode {
    Create,
    Edit,
    AddPoint,
    DeletePoint,
}

#[derive(Debug, Clone)]
pub struct PenToolState {
    pub mode: PenToolMode,
    pub current_path: Option<usize>,
    pub hovered_point: Option<(usize, usize)>,
    pub selected_points: Vec<(usize, usize)>,
    pub is_dragging_handle: bool,
    pub drag_handle_path: Option<usize>,
    pub drag_handle_point: Option<usize>,
    pub drag_handle_in: bool,
}

impl PenToolState {
    pub fn new() -> Self {
        Self {
            mode: PenToolMode::Create,
            current_path: None,
            hovered_point: None,
            selected_points: Vec::new(),
            is_dragging_handle: false,
            drag_handle_path: None,
            drag_handle_point: None,
            drag_handle_in: false,
        }
    }
    
    pub fn set_mode(&mut self, mode: PenToolMode) {
        self.mode = mode;
        self.selected_points.clear();
    }
    
    pub fn start_new_path(&mut self, layer: &mut VectorLayer) {
        let path_idx = layer.create_path();
        self.current_path = Some(path_idx);
        self.mode = PenToolMode::Create;
    }
    
    pub fn add_point_to_current(&mut self, layer: &mut VectorLayer, point: Point) {
        if let Some(path_idx) = self.current_path {
            if let Some(path) = layer.paths.get_mut(path_idx) {
                path.add_point(point);
            }
        }
    }
    
    pub fn close_current_path(&mut self, layer: &mut VectorLayer) {
        if let Some(path_idx) = self.current_path {
            if let Some(path) = layer.paths.get_mut(path_idx) {
                if path.point_count() >= 3 {
                    path.close_path();
                }
            }
            self.current_path = None;
        }
    }
    
    pub fn finish_path(&mut self) {
        self.current_path = None;
        self.selected_points.clear();
    }
    
    pub fn select_point(&mut self, path_idx: usize, point_idx: usize) {
        self.selected_points.push((path_idx, point_idx));
    }
    
    pub fn clear_selection(&mut self) {
        self.selected_points.clear();
    }
    
    pub fn is_point_selected(&self, path_idx: usize, point_idx: usize) -> bool {
        self.selected_points.contains(&(path_idx, point_idx))
    }
    
    pub fn find_nearest_point(&self, layer: &VectorLayer, pos: Point, threshold: f64) -> Option<(usize, usize)> {
        let mut nearest = None;
        let mut min_dist = threshold;
        
        for (path_idx, path) in layer.paths.iter().enumerate() {
            for (point_idx, cp) in path.curve.points.iter().enumerate() {
                let dist = ((cp.point.x - pos.x).powi(2) + (cp.point.y - pos.y).powi(2)).sqrt();
                if dist < min_dist {
                    min_dist = dist;
                    nearest = Some((path_idx, point_idx));
                }
            }
        }
        
        nearest
    }
    
    pub fn find_nearest_handle(&self, layer: &VectorLayer, pos: Point, threshold: f64) -> Option<(usize, usize, bool)> {
        let mut nearest = None;
        let mut min_dist = threshold;
        
        for (path_idx, path) in layer.paths.iter().enumerate() {
            for (point_idx, cp) in path.curve.points.iter().enumerate() {
                        if let Some(in_handle) = cp.in_handle {
                            let dist = ((in_handle.x - pos.x).powi(2) + (in_handle.y - pos.y).powi(2)).sqrt();
                            if dist < min_dist {
                                min_dist = dist;
                                nearest = Some((path_idx, point_idx, true));
                            }
                        }
                        
                        if let Some(out_handle) = cp.out_handle {
                            let dist = ((out_handle.x - pos.x).powi(2) + (out_handle.y - pos.y).powi(2)).sqrt();
                            if dist < min_dist {
                                min_dist = dist;
                                nearest = Some((path_idx, point_idx, false));
                            }
                        }
            }
        }
        
        nearest
    }
    
    pub fn start_drag_point(&mut self, path_idx: usize, point_idx: usize) {
        self.drag_handle_path = Some(path_idx);
        self.drag_handle_point = Some(point_idx);
        self.is_dragging_handle = false;
        self.drag_handle_in = false;
    }
    
    pub fn start_drag_handle(&mut self, path_idx: usize, point_idx: usize, is_in: bool) {
        self.drag_handle_path = Some(path_idx);
        self.drag_handle_point = Some(point_idx);
        self.is_dragging_handle = true;
        self.drag_handle_in = is_in;
    }
    
    pub fn end_drag(&mut self) {
        self.drag_handle_path = None;
        self.drag_handle_point = None;
        self.is_dragging_handle = false;
    }
    
    pub fn move_selected_point(&self, layer: &mut VectorLayer, new_pos: Point) {
        if let Some(path_idx) = self.drag_handle_path {
            if let Some(point_idx) = self.drag_handle_point {
                if let Some(path) = layer.paths.get_mut(path_idx) {
                    if let Some(cp) = path.curve.points.get_mut(point_idx) {
                        if self.is_dragging_handle {
                            if self.drag_handle_in {
                                cp.in_handle = Some(new_pos);
                            } else {
                                cp.out_handle = Some(new_pos);
                            }
                        } else {
                            cp.point = new_pos;
                        }
                    }
                }
            }
        }
    }
}

impl Default for PenToolState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct VectorDocument {
    pub layers: Vec<VectorLayer>,
    pub active_layer: Option<usize>,
    pub pen_state: PenToolState,
}

impl VectorDocument {
    pub fn new() -> Self {
        let base_layer = VectorLayer::new(0, "矢量图层 1");
        Self {
            layers: vec![base_layer],
            active_layer: Some(0),
            pen_state: PenToolState::new(),
        }
    }
    
    pub fn create_layer(&mut self, name: impl Into<String>) -> usize {
        let id = self.layers.len() as u64;
        let layer = VectorLayer::new(id, name);
        self.layers.push(layer);
        self.active_layer = Some(self.layers.len() - 1);
        self.layers.len() - 1
    }
    
    pub fn get_active_layer(&self) -> Option<&VectorLayer> {
        self.active_layer.and_then(|idx| self.layers.get(idx))
    }
    
    pub fn get_active_layer_mut(&mut self) -> Option<&mut VectorLayer> {
        self.active_layer.and_then(|idx| self.layers.get_mut(idx))
    }
    
    pub fn set_active_layer(&mut self, idx: usize) {
        if idx < self.layers.len() {
            self.active_layer = Some(idx);
        }
    }
    
    pub fn delete_layer(&mut self, idx: usize) {
        if self.layers.len() > 1 && idx < self.layers.len() {
            self.layers.remove(idx);
            if let Some(active) = self.active_layer {
                if active >= self.layers.len() {
                    self.active_layer = Some(self.layers.len() - 1);
                } else if active == idx {
                    self.active_layer = Some(idx.saturating_sub(1));
                }
            }
        }
    }
    
    pub fn get_layer(&self, id: u64) -> Option<&VectorLayer> {
        self.layers.iter().find(|l| l.id == id)
    }
    
    pub fn get_layer_mut(&mut self, id: u64) -> Option<&mut VectorLayer> {
        self.layers.iter_mut().find(|l| l.id == id)
    }
    
    pub fn create_path(&mut self, stroke_color: Color8, stroke_width: f32) -> u64 {
        if let Some(active_layer_idx) = self.active_layer {
            if let Some(layer) = self.layers.get_mut(active_layer_idx) {
                let mut path = VectorPath::new(layer.paths.len() as u64);
                path.stroke_color = Some(stroke_color);
                path.stroke_width = stroke_width;
                let path_id = path.id;
                let path_idx = layer.paths.len();
                layer.paths.push(path);
                self.pen_state.current_path = Some(path_idx);
                self.pen_state.mode = PenToolMode::Create;
                return path_id;
            }
        }
        0
    }
    
    pub fn add_point_to_path(&mut self, path_id: u64, point: Point) {
        if let Some(layer) = self.get_active_layer_mut() {
            if let Some(path) = layer.paths.iter_mut().find(|p| p.id == path_id) {
                path.add_point(point);
            }
        }
    }
}

impl Default for VectorDocument {
    fn default() -> Self {
        Self::new()
    }
}
