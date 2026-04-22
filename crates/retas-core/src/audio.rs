use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudioFormat {
    Wav,
    Mp3,
    Ogg,
    Flac,
    Aac,
    Unknown,
}

impl Default for AudioFormat {
    fn default() -> Self {
        AudioFormat::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioInfo {
    pub format: AudioFormat,
    pub sample_rate: u32,
    pub channels: u16,
    pub bits_per_sample: u16,
    pub duration_seconds: f64,
    pub total_samples: u64,
}

impl AudioInfo {
    pub fn new(format: AudioFormat, sample_rate: u32, channels: u16, duration_seconds: f64) -> Self {
        let total_samples = (sample_rate as f64 * duration_seconds) as u64;
        Self {
            format,
            sample_rate,
            channels,
            bits_per_sample: 16,
            duration_seconds,
            total_samples,
        }
    }

    pub fn bytes_per_second(&self) -> u32 {
        self.sample_rate * self.channels as u32 * (self.bits_per_sample / 8) as u32
    }

    pub fn total_bytes(&self) -> u64 {
        self.total_samples * self.channels as u64 * (self.bits_per_sample / 8) as u64
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioClip {
    pub id: u64,
    pub name: String,
    pub file_path: Option<String>,
    pub audio_data: Option<Vec<u8>>,
    pub info: AudioInfo,
}

impl AudioClip {
    pub fn from_file(path: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: rand_audio_id(),
            name: name.into(),
            file_path: Some(path.into()),
            audio_data: None,
            info: AudioInfo::new(AudioFormat::Unknown, 44100, 2, 0.0),
        }
    }

    pub fn from_memory(data: Vec<u8>, info: AudioInfo, name: impl Into<String>) -> Self {
        Self {
            id: rand_audio_id(),
            name: name.into(),
            file_path: None,
            audio_data: Some(data),
            info,
        }
    }

    pub fn duration_frames(&self, frame_rate: f64) -> u32 {
        (self.info.duration_seconds * frame_rate).ceil() as u32
    }

    pub fn sample_at_time(&self, time_seconds: f64) -> Option<f64> {
        if time_seconds < 0.0 || time_seconds >= self.info.duration_seconds {
            return None;
        }

        if let Some(ref data) = self.audio_data {
            let sample_index = (time_seconds * self.info.sample_rate as f64) as usize;
            let byte_index = sample_index * self.info.channels as usize * 2;
            
            if byte_index + 1 < data.len() {
                let sample = i16::from_le_bytes([data[byte_index], data[byte_index + 1]]);
                return Some(sample as f64 / 32768.0);
            }
        }

        None
    }
}

fn rand_audio_id() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundLayerData {
    pub clips: HashMap<u64, AudioClip>,
    pub clip_positions: HashMap<u64, SoundClipPosition>,
    pub volume: f64,
    pub pan: f64,
    pub muted: bool,
    pub solo: bool,
}

impl SoundLayerData {
    pub fn new() -> Self {
        Self {
            clips: HashMap::new(),
            clip_positions: HashMap::new(),
            volume: 1.0,
            pan: 0.0,
            muted: false,
            solo: false,
        }
    }

    pub fn add_clip(&mut self, clip: AudioClip, start_frame: u32, frame_rate: f64) {
        let id = clip.id;
        let duration = clip.duration_frames(frame_rate);
        self.clips.insert(id, clip);
        self.clip_positions.insert(id, SoundClipPosition {
            start_frame,
            duration_frames: duration,
            trim_start: 0.0,
            trim_end: 0.0,
            loop_enabled: false,
        });
    }

    pub fn remove_clip(&mut self, clip_id: u64) {
        self.clips.remove(&clip_id);
        self.clip_positions.remove(&clip_id);
    }

    pub fn get_clip(&self, clip_id: u64) -> Option<&AudioClip> {
        self.clips.get(&clip_id)
    }

    pub fn get_clip_at_frame(&self, frame: u32) -> Vec<&AudioClip> {
        self.clip_positions
            .iter()
            .filter(|(_, pos)| frame >= pos.start_frame && frame < pos.start_frame + pos.duration_frames)
            .filter_map(|(id, _)| self.clips.get(id))
            .collect()
    }

    pub fn move_clip(&mut self, clip_id: u64, new_start_frame: u32) {
        if let Some(pos) = self.clip_positions.get_mut(&clip_id) {
            pos.start_frame = new_start_frame;
        }
    }

    pub fn trim_clip(&mut self, clip_id: u64, trim_start: f64, trim_end: f64) {
        if let Some(pos) = self.clip_positions.get_mut(&clip_id) {
            pos.trim_start = trim_start.max(0.0);
            pos.trim_end = trim_end.max(0.0);
        }
    }

    pub fn set_clip_loop(&mut self, clip_id: u64, enabled: bool) {
        if let Some(pos) = self.clip_positions.get_mut(&clip_id) {
            pos.loop_enabled = enabled;
        }
    }

    pub fn total_duration_frames(&self) -> u32 {
        self.clip_positions
            .values()
            .map(|p| p.start_frame + p.duration_frames)
            .max()
            .unwrap_or(0)
    }

    pub fn render_audio_frame(&self, frame: u32, frame_rate: f64) -> f64 {
        if self.muted {
            return 0.0;
        }

        let mut sample = 0.0;

        for (clip_id, pos) in &self.clip_positions {
            if frame >= pos.start_frame && frame < pos.start_frame + pos.duration_frames {
                if let Some(clip) = self.clips.get(clip_id) {
                    let frame_offset = frame - pos.start_frame;
                    let time = frame_offset as f64 / frame_rate + pos.trim_start;
                    
                    if let Some(s) = clip.sample_at_time(time) {
                        sample += s * self.volume;
                    }
                }
            }
        }

        let pan_factor = (self.pan + 1.0) / 2.0;
        sample * pan_factor
    }
}

impl Default for SoundLayerData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundClipPosition {
    pub start_frame: u32,
    pub duration_frames: u32,
    pub trim_start: f64,
    pub trim_end: f64,
    pub loop_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMixer {
    pub master_volume: f64,
    pub layers: HashMap<u32, AudioMixerLayer>,
}

impl AudioMixer {
    pub fn new() -> Self {
        Self {
            master_volume: 1.0,
            layers: HashMap::new(),
        }
    }

    pub fn add_layer(&mut self, layer_id: u32) {
        self.layers.insert(layer_id, AudioMixerLayer::default());
    }

    pub fn remove_layer(&mut self, layer_id: u32) {
        self.layers.remove(&layer_id);
    }

    pub fn set_layer_volume(&mut self, layer_id: u32, volume: f64) {
        if let Some(layer) = self.layers.get_mut(&layer_id) {
            layer.volume = volume.clamp(0.0, 2.0);
        }
    }

    pub fn set_layer_pan(&mut self, layer_id: u32, pan: f64) {
        if let Some(layer) = self.layers.get_mut(&layer_id) {
            layer.pan = pan.clamp(-1.0, 1.0);
        }
    }

    pub fn mute_layer(&mut self, layer_id: u32, muted: bool) {
        if let Some(layer) = self.layers.get_mut(&layer_id) {
            layer.muted = muted;
        }
    }

    pub fn solo_layer(&mut self, layer_id: u32, solo: bool) {
        if let Some(layer) = self.layers.get_mut(&layer_id) {
            layer.solo = solo;
        }
    }

    pub fn has_solo(&self) -> bool {
        self.layers.values().any(|l| l.solo)
    }

    pub fn is_layer_audible(&self, layer_id: u32) -> bool {
        if let Some(layer) = self.layers.get(&layer_id) {
            if layer.muted {
                return false;
            }
            
            if self.has_solo() && !layer.solo {
                return false;
            }
            
            return true;
        }
        false
    }

    pub fn mix_samples(&self, samples: &[(u32, f64)]) -> f64 {
        let mut result = 0.0;
        let mut count = 0;

        for (layer_id, sample) in samples {
            if self.is_layer_audible(*layer_id) {
                if let Some(layer) = self.layers.get(layer_id) {
                    result += sample * layer.volume;
                    count += 1;
                }
            }
        }

        if count > 0 {
            result = result / count as f64 * self.master_volume;
        }

        result.clamp(-1.0, 1.0)
    }
}

impl Default for AudioMixer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMixerLayer {
    pub volume: f64,
    pub pan: f64,
    pub muted: bool,
    pub solo: bool,
    pub effects: Vec<AudioEffect>,
}

impl Default for AudioMixerLayer {
    fn default() -> Self {
        Self {
            volume: 1.0,
            pan: 0.0,
            muted: false,
            solo: false,
            effects: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioEffect {
    Gain { amount: f64 },
    FadeIn { duration_seconds: f64 },
    FadeOut { duration_seconds: f64 },
    Normalize { target_db: f64 },
    Compressor { threshold: f64, ratio: f64, attack: f64, release: f64 },
    Reverb { room_size: f64, damping: f64, wet_level: f64 },
    Delay { delay_seconds: f64, feedback: f64, mix: f64 },
    Equalizer { bands: Vec<EqBand> },
    PitchShift { semitones: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqBand {
    pub frequency: f64,
    pub gain: f64,
    pub q: f64,
}

impl EqBand {
    pub fn new(frequency: f64, gain: f64) -> Self {
        Self {
            frequency,
            gain,
            q: 1.0,
        }
    }
}
