use std::collections::HashMap;
use super::layer::LayerManager;

#[derive(Debug, Clone)]
pub struct Frame {
    pub id: u32,
    pub layer_manager: LayerManager,
    pub is_keyframe: bool,
}

impl Frame {
    pub fn new(id: u32, width: u32, height: u32) -> Self {
        Self {
            id,
            layer_manager: LayerManager::new(width, height),
            is_keyframe: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FrameManager {
    pub frames: Vec<Frame>,
    pub current_frame: u32,
    pub width: u32,
    pub height: u32,
}

impl FrameManager {
    pub fn new(width: u32, height: u32) -> Self {
        let initial_frame = Frame::new(0, width, height);
        
        Self {
            frames: vec![initial_frame],
            current_frame: 0,
            width,
            height,
        }
    }
    
    pub fn current_layer_manager(&self) -> &LayerManager {
        &self.frames[self.current_frame as usize].layer_manager
    }
    
    pub fn current_layer_manager_mut(&mut self) -> &mut LayerManager {
        &mut self.frames[self.current_frame as usize].layer_manager
    }
    
    pub fn go_to_frame(&mut self, frame_idx: u32) {
        if frame_idx < self.frames.len() as u32 {
            self.current_frame = frame_idx;
        }
    }
    
    pub fn next_frame(&mut self) -> bool {
        if self.current_frame < self.frames.len() as u32 - 1 {
            self.current_frame += 1;
            true
        } else {
            false
        }
    }
    
    pub fn prev_frame(&mut self) -> bool {
        if self.current_frame > 0 {
            self.current_frame -= 1;
            true
        } else {
            false
        }
    }
    
    pub fn first_frame(&mut self) {
        self.current_frame = 0;
    }
    
    pub fn last_frame(&mut self) {
        self.current_frame = self.frames.len() as u32 - 1;
    }
    
    pub fn add_frame(&mut self) -> u32 {
        let new_id = self.frames.len() as u32;
        let new_frame = Frame::new(new_id, self.width, self.height);
        self.frames.push(new_frame);
        new_id
    }
    
    pub fn insert_frame(&mut self, at_index: usize) -> Option<u32> {
        if at_index > self.frames.len() {
            return None;
        }
        
        let new_frame = Frame::new(at_index as u32, self.width, self.height);
        self.frames.insert(at_index, new_frame);
        
        for (i, frame) in self.frames.iter_mut().enumerate() {
            frame.id = i as u32;
        }
        
        if at_index <= self.current_frame as usize {
            self.current_frame += 1;
        }
        
        Some(at_index as u32)
    }
    
    pub fn delete_frame(&mut self, frame_idx: u32) -> bool {
        if self.frames.len() <= 1 || frame_idx >= self.frames.len() as u32 {
            return false;
        }
        
        self.frames.remove(frame_idx as usize);
        
        for (i, frame) in self.frames.iter_mut().enumerate() {
            frame.id = i as u32;
        }
        
        if self.current_frame >= self.frames.len() as u32 {
            self.current_frame = self.frames.len() as u32 - 1;
        }
        
        true
    }
    
    pub fn duplicate_frame(&mut self, frame_idx: u32) -> Option<u32> {
        if frame_idx >= self.frames.len() as u32 {
            return None;
        }
        
        let source_frame = &self.frames[frame_idx as usize];
        let new_id = self.frames.len() as u32;
        
        let mut new_frame = Frame::new(new_id, self.width, self.height);
        new_frame.layer_manager = source_frame.layer_manager.clone();
        new_frame.is_keyframe = true;
        
        self.frames.push(new_frame);
        
        Some(new_id)
    }
    
    pub fn total_frames(&self) -> u32 {
        self.frames.len() as u32
    }
    
    pub fn is_first_frame(&self) -> bool {
        self.current_frame == 0
    }
    
    pub fn is_last_frame(&self) -> bool {
        self.current_frame == self.frames.len() as u32 - 1
    }
}

impl Default for FrameManager {
    fn default() -> Self {
        Self::new(1920, 1080)
    }
}
