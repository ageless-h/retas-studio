use std::collections::VecDeque;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct RenderJob {
    pub id: u64,
    pub name: String,
    pub document_id: String,
    pub frame_range: (u32, u32),
    pub output_dir: PathBuf,
    pub format: RenderFormat,
    pub quality: RenderQuality,
    pub status: RenderStatus,
    pub progress: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderFormat {
    Png,
    Jpeg,
    Gif,
    WebM,
    Mp4,
    APNG,
}

impl RenderFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            RenderFormat::Png => "png",
            RenderFormat::Jpeg => "jpg",
            RenderFormat::Gif => "gif",
            RenderFormat::WebM => "webm",
            RenderFormat::Mp4 => "mp4",
            RenderFormat::APNG => "png",
        }
    }
    
    pub fn is_video(&self) -> bool {
        matches!(self, RenderFormat::WebM | RenderFormat::Mp4 | RenderFormat::Gif)
    }
    
    pub fn is_sequence(&self) -> bool {
        matches!(self, RenderFormat::Png | RenderFormat::Jpeg)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderQuality {
    Draft,
    Standard,
    High,
    Maximum,
}

impl RenderQuality {
    pub fn scale_factor(&self) -> f64 {
        match self {
            RenderQuality::Draft => 0.5,
            RenderQuality::Standard => 1.0,
            RenderQuality::High => 2.0,
            RenderQuality::Maximum => 4.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RenderStatus {
    Queued,
    Rendering,
    Completed,
    Failed(String),
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct RenderQueue {
    pub jobs: VecDeque<RenderJob>,
    pub completed_jobs: Vec<RenderJob>,
    pub next_id: u64,
    pub is_rendering: bool,
    pub current_job: Option<u64>,
}

impl RenderQueue {
    pub fn new() -> Self {
        Self {
            jobs: VecDeque::new(),
            completed_jobs: Vec::new(),
            next_id: 1,
            is_rendering: false,
            current_job: None,
        }
    }
    
    pub fn add_job(&mut self, mut job: RenderJob) -> u64 {
        job.id = self.next_id;
        job.status = RenderStatus::Queued;
        job.progress = 0.0;
        self.jobs.push_back(job);
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    
    pub fn add_batch_export(
        &mut self,
        name: String,
        document_id: String,
        frame_range: (u32, u32),
        output_dir: PathBuf,
        format: RenderFormat,
    ) -> u64 {
        let job = RenderJob {
            id: 0,
            name,
            document_id,
            frame_range,
            output_dir,
            format,
            quality: RenderQuality::Standard,
            status: RenderStatus::Queued,
            progress: 0.0,
        };
        self.add_job(job)
    }
    
    pub fn remove_job(&mut self, job_id: u64) -> Option<RenderJob> {
        if let Some(pos) = self.jobs.iter().position(|j| j.id == job_id) {
            if pos < self.jobs.len() {
                return self.jobs.remove(pos);
            }
        }
        None
    }
    
    pub fn cancel_job(&mut self, job_id: u64) -> bool {
        if let Some(job) = self.jobs.iter_mut().find(|j| j.id == job_id) {
            if job.status == RenderStatus::Queued {
                job.status = RenderStatus::Cancelled;
                return true;
            }
        }
        false
    }
    
    pub fn start_next(&mut self) -> Option<&mut RenderJob> {
        if let Some(job) = self.jobs.front_mut() {
            if job.status == RenderStatus::Queued {
                job.status = RenderStatus::Rendering;
                self.current_job = Some(job.id);
                self.is_rendering = true;
                return Some(job);
            }
        }
        None
    }
    
    pub fn finish_current(&mut self, success: bool, error_msg: Option<String>) {
        if let Some(job_id) = self.current_job {
            if let Some(mut job) = self.remove_job(job_id) {
                job.status = if success {
                    RenderStatus::Completed
                } else {
                    RenderStatus::Failed(error_msg.unwrap_or_else(|| "Unknown error".to_string()))
                };
                job.progress = if success { 100.0 } else { job.progress };
                self.completed_jobs.push(job);
            }
        }
        self.current_job = None;
        self.is_rendering = false;
    }
    
    pub fn update_progress(&mut self, job_id: u64, progress: f64) {
        if let Some(job) = self.jobs.iter_mut().find(|j| j.id == job_id) {
            job.progress = progress.clamp(0.0, 100.0);
        }
    }
    
    pub fn get_job(&self, job_id: u64) -> Option<&RenderJob> {
        self.jobs.iter().find(|j| j.id == job_id)
            .or_else(|| self.completed_jobs.iter().find(|j| j.id == job_id))
    }
    
    pub fn clear_completed(&mut self) {
        self.completed_jobs.retain(|j| j.status == RenderStatus::Failed("".to_string()));
    }
    
    pub fn reorder_job(&mut self, job_id: u64, new_index: usize) -> bool {
        let old_index = match self.jobs.iter().position(|j| j.id == job_id) {
            Some(idx) => idx,
            None => return false,
        };
        if old_index == new_index || new_index >= self.jobs.len() {
            return false;
        }
        
        if let Some(job) = self.jobs.remove(old_index) {
            let insert_index = if new_index > old_index { new_index } else { new_index };
            self.jobs.insert(insert_index, job);
            true
        } else {
            false
        }
    }
}

impl Default for RenderQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct BatchExportSettings {
    pub frame_range: FrameRange,
    pub output_format: RenderFormat,
    pub quality: RenderQuality,
    pub include_alpha: bool,
    pub scale: f64,
    pub naming_pattern: String,
}

impl Default for BatchExportSettings {
    fn default() -> Self {
        Self {
            frame_range: FrameRange::All,
            output_format: RenderFormat::Png,
            quality: RenderQuality::Standard,
            include_alpha: true,
            scale: 1.0,
            naming_pattern: String::from("{project}_{frame:04d}.{ext}"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FrameRange {
    All,
    Current,
    Custom(u32, u32),
    WorkArea,
}

pub struct RenderEngine {
    pub queue: RenderQueue,
}

impl RenderEngine {
    pub fn new() -> Self {
        Self {
            queue: RenderQueue::new(),
        }
    }
    
    pub fn queue_document(&mut self, doc_id: String, frame_range: FrameRange, settings: BatchExportSettings) -> u64 {
        let (start, end) = match frame_range {
            FrameRange::All => (0, 143),
            FrameRange::Current => (0, 0),
            FrameRange::Custom(s, e) => (s, e),
            FrameRange::WorkArea => (0, 143),
        };
        
        let job = RenderJob {
            id: 0,
            name: format!("Export {}", doc_id),
            document_id: doc_id,
            frame_range: (start, end),
            output_dir: PathBuf::from("./output"),
            format: settings.output_format,
            quality: settings.quality,
            status: RenderStatus::Queued,
            progress: 0.0,
        };
        
        self.queue.add_job(job)
    }
    
    pub fn start_render(&mut self) -> Option<u64> {
        self.queue.start_next().map(|j| j.id)
    }
    
    pub fn process_frame(&mut self, job_id: u64, frame: u32) -> bool {
        if let Some(job) = self.queue.get_job(job_id) {
            let total_frames = job.frame_range.1 - job.frame_range.0 + 1;
            let current_frame_in_range = frame - job.frame_range.0;
            let progress = (current_frame_in_range as f64 / total_frames as f64) * 100.0;
            self.queue.update_progress(job_id, progress);
            true
        } else {
            false
        }
    }
    
    pub fn complete_render(&mut self, _job_id: u64, success: bool) {
        self.queue.finish_current(success, None);
    }
}

impl Default for RenderEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_render_queue() {
        let mut queue = RenderQueue::new();
        
        let job_id = queue.add_batch_export(
            "Test Export".to_string(),
            "doc1".to_string(),
            (0, 10),
            PathBuf::from("./output"),
            RenderFormat::Png,
        );
        
        assert_eq!(queue.jobs.len(), 1);
        assert_eq!(job_id, 1);
        
        if let Some(job) = queue.start_next() {
            assert_eq!(job.id, job_id);
            assert_eq!(job.status, RenderStatus::Rendering);
        }
    }
    
    #[test]
    fn test_render_format() {
        assert!(RenderFormat::Png.is_sequence());
        assert!(!RenderFormat::Png.is_video());
        assert!(RenderFormat::Mp4.is_video());
        assert!(!RenderFormat::Mp4.is_sequence());
    }
}
