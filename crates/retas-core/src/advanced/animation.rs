use serde::{Deserialize, Serialize};
use crate::{LayerId, Color8};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
    Recording,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineMarker {
    pub frame: u32,
    pub name: String,
    pub color: Color8,
}

impl TimelineMarker {
    pub fn new(frame: u32, name: impl Into<String>) -> Self {
        Self {
            frame,
            name: name.into(),
            color: Color8::from_rgb(255, 200, 0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnionSkinSettings {
    pub enabled: bool,
    pub frames_before: u32,
    pub frames_after: u32,
    pub opacity_before: f64,
    pub opacity_after: f64,
    pub color_before: Color8,
    pub color_after: Color8,
    pub blend_mode: crate::layer::BlendMode,
}

impl Default for OnionSkinSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            frames_before: 3,
            frames_after: 3,
            opacity_before: 0.3,
            opacity_after: 0.3,
            color_before: Color8::from_rgb(255, 0, 0),
            color_after: Color8::from_rgb(0, 255, 0),
            blend_mode: crate::layer::BlendMode::Normal,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackSettings {
    pub frame_rate: f64,
    pub loop_enabled: bool,
    pub ping_pong: bool,
    pub play_all_layers: bool,
    pub play_sound: bool,
    pub skip_frames: bool,
}

impl Default for PlaybackSettings {
    fn default() -> Self {
        Self {
            frame_rate: 24.0,
            loop_enabled: true,
            ping_pong: false,
            play_all_layers: true,
            play_sound: true,
            skip_frames: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineState {
    pub current_frame: u32,
    pub start_frame: u32,
    pub end_frame: u32,
    pub in_point: u32,
    pub out_point: u32,
    pub playback_state: PlaybackState,
    pub markers: Vec<TimelineMarker>,
    pub onion_skin: OnionSkinSettings,
    pub playback: PlaybackSettings,
    pub layer_visibility: HashMap<LayerId, bool>,
    pub layer_lock: HashMap<LayerId, bool>,
    pub selected_layers: Vec<LayerId>,
    pub active_layer: Option<LayerId>,
}

impl TimelineState {
    pub fn new() -> Self {
        Self {
            current_frame: 0,
            start_frame: 0,
            end_frame: 100,
            in_point: 0,
            out_point: 100,
            playback_state: PlaybackState::Stopped,
            markers: Vec::new(),
            onion_skin: OnionSkinSettings::default(),
            playback: PlaybackSettings::default(),
            layer_visibility: HashMap::new(),
            layer_lock: HashMap::new(),
            selected_layers: Vec::new(),
            active_layer: None,
        }
    }

    pub fn with_range(mut self, start: u32, end: u32) -> Self {
        self.start_frame = start;
        self.end_frame = end;
        self.in_point = start;
        self.out_point = end;
        self
    }

    pub fn with_frame_rate(mut self, fps: f64) -> Self {
        self.playback.frame_rate = fps;
        self
    }

    pub fn total_frames(&self) -> u32 {
        self.end_frame.saturating_sub(self.start_frame) + 1
    }

    pub fn duration_seconds(&self) -> f64 {
        self.total_frames() as f64 / self.playback.frame_rate
    }

    pub fn frame_to_time(&self, frame: u32) -> f64 {
        (frame.saturating_sub(self.start_frame)) as f64 / self.playback.frame_rate
    }

    pub fn time_to_frame(&self, time: f64) -> u32 {
        let frame = (time * self.playback.frame_rate) as u32 + self.start_frame;
        frame.clamp(self.start_frame, self.end_frame)
    }

    pub fn advance_frame(&mut self) {
        self.current_frame += 1;
        
        if self.current_frame > self.out_point {
            if self.playback.loop_enabled {
                if self.playback.ping_pong {
                    self.current_frame = self.out_point.saturating_sub(1);
                } else {
                    self.current_frame = self.in_point;
                }
            } else {
                self.current_frame = self.out_point;
                self.playback_state = PlaybackState::Stopped;
            }
        }
    }

    pub fn previous_frame(&mut self) {
        if self.current_frame > 0 {
            self.current_frame -= 1;
        }
    }

    pub fn next_frame(&mut self) {
        if self.current_frame < self.end_frame {
            self.current_frame += 1;
        }
    }

    pub fn go_to_frame(&mut self, frame: u32) {
        self.current_frame = frame.clamp(self.start_frame, self.end_frame);
    }

    pub fn go_to_start(&mut self) {
        self.current_frame = self.in_point;
    }

    pub fn go_to_end(&mut self) {
        self.current_frame = self.out_point;
    }

    pub fn play(&mut self) {
        self.playback_state = PlaybackState::Playing;
    }

    pub fn pause(&mut self) {
        self.playback_state = PlaybackState::Paused;
    }

    pub fn stop(&mut self) {
        self.playback_state = PlaybackState::Stopped;
        self.current_frame = self.in_point;
    }

    pub fn toggle_play(&mut self) {
        match self.playback_state {
            PlaybackState::Playing => self.pause(),
            PlaybackState::Paused => self.play(),
            PlaybackState::Stopped => self.play(),
            PlaybackState::Recording => {}
        }
    }

    pub fn is_playing(&self) -> bool {
        matches!(self.playback_state, PlaybackState::Playing)
    }

    pub fn add_marker(&mut self, marker: TimelineMarker) {
        let pos = self.markers.binary_search_by_key(&marker.frame, |m| m.frame);
        match pos {
            Ok(idx) => self.markers[idx] = marker,
            Err(idx) => self.markers.insert(idx, marker),
        }
    }

    pub fn remove_marker(&mut self, frame: u32) -> Option<TimelineMarker> {
        let pos = self.markers.binary_search_by_key(&frame, |m| m.frame);
        pos.ok().map(|idx| self.markers.remove(idx))
    }

    pub fn get_marker(&self, frame: u32) -> Option<&TimelineMarker> {
        self.markers.binary_search_by_key(&frame, |m| m.frame)
            .ok()
            .map(|idx| &self.markers[idx])
    }

    pub fn set_layer_visibility(&mut self, layer_id: LayerId, visible: bool) {
        self.layer_visibility.insert(layer_id, visible);
    }

    pub fn is_layer_visible(&self, layer_id: LayerId) -> bool {
        self.layer_visibility.get(&layer_id).copied().unwrap_or(true)
    }

    pub fn set_layer_lock(&mut self, layer_id: LayerId, locked: bool) {
        self.layer_lock.insert(layer_id, locked);
    }

    pub fn is_layer_locked(&self, layer_id: LayerId) -> bool {
        self.layer_lock.get(&layer_id).copied().unwrap_or(false)
    }

    pub fn select_layer(&mut self, layer_id: LayerId, add_to_selection: bool) {
        if add_to_selection {
            if let Some(pos) = self.selected_layers.iter().position(|&id| id == layer_id) {
                self.selected_layers.remove(pos);
            } else {
                self.selected_layers.push(layer_id);
            }
        } else {
            self.selected_layers.clear();
            self.selected_layers.push(layer_id);
        }
        self.active_layer = Some(layer_id);
    }

    pub fn deselect_all(&mut self) {
        self.selected_layers.clear();
        self.active_layer = None;
    }

    pub fn onion_skin_frames(&self) -> Vec<(u32, f64, Color8)> {
        if !self.onion_skin.enabled {
            return Vec::new();
        }

        let mut frames = Vec::new();

        for i in 1..=self.onion_skin.frames_before {
            let frame = self.current_frame.saturating_sub(i);
            if frame >= self.start_frame {
                let opacity = self.onion_skin.opacity_before * (1.0 - (i - 1) as f64 / self.onion_skin.frames_before as f64);
                frames.push((frame, opacity, self.onion_skin.color_before));
            }
        }

        for i in 1..=self.onion_skin.frames_after {
            let frame = self.current_frame + i;
            if frame <= self.end_frame {
                let opacity = self.onion_skin.opacity_after * (1.0 - (i - 1) as f64 / self.onion_skin.frames_after as f64);
                frames.push((frame, opacity, self.onion_skin.color_after));
            }
        }

        frames
    }
}

impl Default for TimelineState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureSheet {
    pub columns: Vec<ExposureColumn>,
    pub current_column: Option<usize>,
    pub current_row: u32,
}

impl ExposureSheet {
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            current_column: None,
            current_row: 0,
        }
    }

    pub fn add_column(&mut self, name: impl Into<String>, layer_id: LayerId) -> usize {
        let idx = self.columns.len();
        self.columns.push(ExposureColumn {
            name: name.into(),
            layer_id,
            cells: HashMap::new(),
            visible: true,
            locked: false,
        });
        idx
    }

    pub fn set_cell(&mut self, column: usize, frame: u32, cell: ExposureCell) {
        if let Some(col) = self.columns.get_mut(column) {
            col.cells.insert(frame, cell);
        }
    }

    pub fn get_cell(&self, column: usize, frame: u32) -> Option<&ExposureCell> {
        self.columns.get(column)?.cells.get(&frame)
    }

    pub fn extend_cell(&mut self, column: usize, start_frame: u32, end_frame: u32) {
        if let Some(col) = self.columns.get_mut(column) {
            if let Some(cell) = col.cells.get(&start_frame).cloned() {
                for frame in start_frame..=end_frame {
                    col.cells.insert(frame, cell.clone());
                }
            }
        }
    }
}

impl Default for ExposureSheet {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureColumn {
    pub name: String,
    pub layer_id: LayerId,
    pub cells: HashMap<u32, ExposureCell>,
    pub visible: bool,
    pub locked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureCell {
    pub name: String,
    pub duration: u32,
    pub is_keyframe: bool,
    pub has_drawing: bool,
}

impl ExposureCell {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            duration: 1,
            is_keyframe: false,
            has_drawing: false,
        }
    }

    pub fn keyframe(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            duration: 1,
            is_keyframe: true,
            has_drawing: true,
        }
    }
}

pub struct PlaybackController {
    timeline: TimelineState,
    elapsed_time: f64,
    frame_accumulator: f64,
    direction: i32,
}

impl PlaybackController {
    pub fn new(timeline: TimelineState) -> Self {
        Self {
            timeline,
            elapsed_time: 0.0,
            frame_accumulator: 0.0,
            direction: 1,
        }
    }

    pub fn timeline(&self) -> &TimelineState {
        &self.timeline
    }

    pub fn timeline_mut(&mut self) -> &mut TimelineState {
        &mut self.timeline
    }

    pub fn update(&mut self, delta_time: f64) -> Option<u32> {
        if !self.timeline.is_playing() {
            return None;
        }

        let frame_duration = 1.0 / self.timeline.playback.frame_rate;
        self.elapsed_time += delta_time;
        self.frame_accumulator += delta_time;

        let mut new_frame = None;

        while self.frame_accumulator >= frame_duration {
            self.frame_accumulator -= frame_duration;

            if self.timeline.playback.ping_pong {
                if self.direction > 0 {
                    if self.timeline.current_frame >= self.timeline.out_point {
                        self.direction = -1;
                        self.timeline.current_frame = self.timeline.out_point.saturating_sub(1);
                    } else {
                        self.timeline.current_frame += 1;
                    }
                } else {
                    if self.timeline.current_frame <= self.timeline.in_point {
                        self.direction = 1;
                        self.timeline.current_frame = self.timeline.in_point + 1;
                    } else {
                        self.timeline.current_frame -= 1;
                    }
                }
            } else {
                self.timeline.advance_frame();
            }

            new_frame = Some(self.timeline.current_frame);
        }

        new_frame
    }

    pub fn play(&mut self) {
        self.timeline.play();
        self.elapsed_time = 0.0;
        self.frame_accumulator = 0.0;
        self.direction = 1;
    }

    pub fn pause(&mut self) {
        self.timeline.pause();
    }

    pub fn stop(&mut self) {
        self.timeline.stop();
        self.elapsed_time = 0.0;
        self.frame_accumulator = 0.0;
        self.direction = 1;
    }

    pub fn toggle_play(&mut self) {
        self.timeline.toggle_play();
        if self.timeline.is_playing() {
            self.elapsed_time = 0.0;
            self.frame_accumulator = 0.0;
        }
    }

    pub fn step_forward(&mut self) {
        self.timeline.next_frame();
    }

    pub fn step_backward(&mut self) {
        self.timeline.previous_frame();
    }

    pub fn go_to_frame(&mut self, frame: u32) {
        self.timeline.go_to_frame(frame);
        self.elapsed_time = 0.0;
        self.frame_accumulator = 0.0;
    }

    pub fn current_time(&self) -> f64 {
        self.timeline.frame_to_time(self.timeline.current_frame)
    }

    pub fn progress(&self) -> f64 {
        let total = self.timeline.total_frames() as f64;
        if total > 0.0 {
            (self.timeline.current_frame - self.timeline.start_frame) as f64 / total
        } else {
            0.0
        }
    }

    pub fn set_playback_range(&mut self, in_point: u32, out_point: u32) {
        self.timeline.in_point = in_point;
        self.timeline.out_point = out_point;
    }
}

impl Default for PlaybackController {
    fn default() -> Self {
        Self::new(TimelineState::new())
    }
}
