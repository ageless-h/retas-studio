use super::AiError;

pub struct InferenceResult {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub processing_time_ms: u64,
}

pub trait AiEngine: Send + Sync {
    fn load_model(&mut self, model_path: &str) -> Result<(), AiError>;
    fn unload_model(&mut self);
    fn is_model_loaded(&self) -> bool;
    fn infer(&self, input: &[u8], width: u32, height: u32) -> Result<InferenceResult, AiError>;
    fn memory_usage_mb(&self) -> usize;
}

pub struct LocalEngine {
    model_loaded: bool,
    model_path: Option<String>,
}

impl LocalEngine {
    pub fn new() -> Self {
        Self {
            model_loaded: false,
            model_path: None,
        }
    }
}

impl AiEngine for LocalEngine {
    fn load_model(&mut self, model_path: &str) -> Result<(), AiError> {
        self.model_path = Some(model_path.to_string());
        self.model_loaded = true;
        Ok(())
    }

    fn unload_model(&mut self) {
        self.model_loaded = false;
        self.model_path = None;
    }

    fn is_model_loaded(&self) -> bool {
        self.model_loaded
    }

    fn infer(&self, _input: &[u8], width: u32, height: u32) -> Result<InferenceResult, AiError> {
        if !self.model_loaded {
            return Err(AiError::ModelNotLoaded);
        }
        Ok(InferenceResult {
            data: vec![255; (width * height * 4) as usize],
            width,
            height,
            processing_time_ms: 0,
        })
    }

    fn memory_usage_mb(&self) -> usize {
        if self.model_loaded { 512 } else { 0 }
    }
}

pub struct OnnxEngine {
    model_loaded: bool,
}

impl OnnxEngine {
    pub fn new() -> Self {
        Self {
            model_loaded: false,
        }
    }
}

impl AiEngine for OnnxEngine {
    fn load_model(&mut self, _model_path: &str) -> Result<(), AiError> {
        self.model_loaded = true;
        Ok(())
    }

    fn unload_model(&mut self) {
        self.model_loaded = false;
    }

    fn is_model_loaded(&self) -> bool {
        self.model_loaded
    }

    fn infer(&self, input: &[u8], width: u32, height: u32) -> Result<InferenceResult, AiError> {
        if !self.model_loaded {
            return Err(AiError::ModelNotLoaded);
        }
        Ok(InferenceResult {
            data: input.to_vec(),
            width,
            height,
            processing_time_ms: 0,
        })
    }

    fn memory_usage_mb(&self) -> usize {
        if self.model_loaded { 1024 } else { 0 }
    }
}
