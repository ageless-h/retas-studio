use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{LayerId, Rect};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CutFolder {
    pub id: u64,
    pub name: String,
    pub cuts: Vec<Cut>,
    pub direction: CutDirection,
    pub notes: String,
}

impl CutFolder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: 0,
            name: name.into(),
            cuts: Vec::new(),
            direction: CutDirection::default(),
            notes: String::new(),
        }
    }

    pub fn add_cut(&mut self, cut: Cut) {
        self.cuts.push(cut);
    }

    pub fn remove_cut(&mut self, id: u64) -> bool {
        let len = self.cuts.len();
        self.cuts.retain(|c| c.id != id);
        self.cuts.len() != len
    }

    pub fn get_cut(&self, id: u64) -> Option<&Cut> {
        self.cuts.iter().find(|c| c.id == id)
    }

    pub fn total_duration(&self) -> u32 {
        self.cuts.iter().map(|c| c.duration_frames).sum()
    }

    pub fn sort_by_time(&mut self) {
        self.cuts.sort_by_key(|c| c.start_frame);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cut {
    pub id: u64,
    pub name: String,
    pub start_frame: u32,
    pub end_frame: u32,
    pub duration_frames: u32,
    pub layers: Vec<LayerId>,
    pub bounds: Option<Rect>,
    pub transition_in: Option<Transition>,
    pub transition_out: Option<Transition>,
    pub notes: String,
    pub color: [u8; 4],
}

impl Cut {
    pub fn new(name: impl Into<String>, start: u32, end: u32) -> Self {
        Self {
            id: 0,
            name: name.into(),
            start_frame: start,
            end_frame: end,
            duration_frames: end.saturating_sub(start) + 1,
            layers: Vec::new(),
            bounds: None,
            transition_in: None,
            transition_out: None,
            notes: String::new(),
            color: [255, 255, 255, 255],
        }
    }

    pub fn contains_frame(&self, frame: u32) -> bool {
        frame >= self.start_frame && frame <= self.end_frame
    }

    pub fn set_range(&mut self, start: u32, end: u32) {
        self.start_frame = start;
        self.end_frame = end;
        self.duration_frames = end.saturating_sub(start) + 1;
    }

    pub fn add_layer(&mut self, layer_id: LayerId) {
        if !self.layers.contains(&layer_id) {
            self.layers.push(layer_id);
        }
    }

    pub fn remove_layer(&mut self, layer_id: LayerId) -> bool {
        let len = self.layers.len();
        self.layers.retain(|&id| id != layer_id);
        self.layers.len() != len
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CutDirection {
    pub direction: Direction,
    pub angle: f64,
    pub speed: f64,
    pub intensity: f64,
}

impl Default for CutDirection {
    fn default() -> Self {
        Self {
            direction: Direction::None,
            angle: 0.0,
            speed: 1.0,
            intensity: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    None,
    Left,
    Right,
    Up,
    Down,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
    Custom,
}

impl Direction {
    pub fn to_vector(&self) -> (f64, f64) {
        match self {
            Direction::None => (0.0, 0.0),
            Direction::Left => (-1.0, 0.0),
            Direction::Right => (1.0, 0.0),
            Direction::Up => (0.0, -1.0),
            Direction::Down => (0.0, 1.0),
            Direction::UpLeft => (-0.707, -0.707),
            Direction::UpRight => (0.707, -0.707),
            Direction::DownLeft => (-0.707, 0.707),
            Direction::DownRight => (0.707, 0.707),
            Direction::Custom => {
                let rad = std::f64::consts::PI / 180.0;
                (rad.cos(), rad.sin())
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub transition_type: TransitionType,
    pub duration_frames: u32,
    pub params: HashMap<String, f64>,
}

impl Transition {
    pub fn new(transition_type: TransitionType, duration: u32) -> Self {
        Self {
            transition_type,
            duration_frames: duration,
            params: HashMap::new(),
        }
    }

    pub fn fade(duration: u32) -> Self {
        Self::new(TransitionType::Fade, duration)
    }

    pub fn wipe(duration: u32, angle: f64) -> Self {
        let mut t = Self::new(TransitionType::Wipe, duration);
        t.params.insert("angle".to_string(), angle);
        t
    }

    pub fn dissolve(duration: u32) -> Self {
        Self::new(TransitionType::Dissolve, duration)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransitionType {
    Cut,
    Fade,
    Wipe,
    Dissolve,
    Slide,
    Push,
    Zoom,
    Spin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CutManager {
    folders: HashMap<u64, CutFolder>,
    current_folder_id: Option<u64>,
    next_id: u64,
}

impl CutManager {
    pub fn new() -> Self {
        Self {
            folders: HashMap::new(),
            current_folder_id: None,
            next_id: 1,
        }
    }

    fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn create_folder(&mut self, name: impl Into<String>) -> u64 {
        let id = self.next_id();
        let mut folder = CutFolder::new(name);
        folder.id = id;
        self.folders.insert(id, folder);
        id
    }

    pub fn delete_folder(&mut self, id: u64) -> bool {
        if self.current_folder_id == Some(id) {
            self.current_folder_id = None;
        }
        self.folders.remove(&id).is_some()
    }

    pub fn get_folder(&self, id: u64) -> Option<&CutFolder> {
        self.folders.get(&id)
    }

    pub fn get_folder_mut(&mut self, id: u64) -> Option<&mut CutFolder> {
        self.folders.get_mut(&id)
    }

    pub fn set_current_folder(&mut self, id: u64) -> bool {
        if self.folders.contains_key(&id) {
            self.current_folder_id = Some(id);
            true
        } else {
            false
        }
    }

    pub fn current_folder(&self) -> Option<&CutFolder> {
        self.current_folder_id.and_then(|id| self.folders.get(&id))
    }

    pub fn current_folder_mut(&mut self) -> Option<&mut CutFolder> {
        self.current_folder_id.and_then(|id| self.folders.get_mut(&id))
    }

    pub fn add_cut_to_folder(&mut self, folder_id: u64, mut cut: Cut) -> Option<u64> {
        let cut_id = self.next_id();
        cut.id = cut_id;
        let folder = self.folders.get_mut(&folder_id)?;
        folder.add_cut(cut);
        Some(cut_id)
    }

    pub fn remove_cut_from_folder(&mut self, folder_id: u64, cut_id: u64) -> bool {
        if let Some(folder) = self.folders.get_mut(&folder_id) {
            folder.remove_cut(cut_id)
        } else {
            false
        }
    }

    pub fn get_all_folders(&self) -> Vec<&CutFolder> {
        self.folders.values().collect()
    }

    pub fn find_cut_at_frame(&self, frame: u32) -> Option<(&CutFolder, &Cut)> {
        for folder in self.folders.values() {
            for cut in &folder.cuts {
                if cut.contains_frame(frame) {
                    return Some((folder, cut));
                }
            }
        }
        None
    }

    pub fn get_cuts_in_range(&self, start: u32, end: u32) -> Vec<(&CutFolder, &Cut)> {
        let mut result = Vec::new();
        for folder in self.folders.values() {
            for cut in &folder.cuts {
                if cut.start_frame <= end && cut.end_frame >= start {
                    result.push((folder, cut));
                }
            }
        }
        result
    }
}

impl Default for CutManager {
    fn default() -> Self {
        Self::new()
    }
}
