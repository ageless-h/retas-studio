#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::fmt;

pub mod api_client;
pub mod engine;
pub mod pipeline;
pub mod queue;

pub use api_client::AiApiClient;
pub use engine::{AiEngine, InferenceResult};
pub use pipeline::{AiPipeline, PipelineConfig};
pub use queue::{AiJobType, AiQueue};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiFeature {
    AutoColor,
    Inbetween,
    StyleTransfer,
    LineCleanup,
    BackgroundGen,
    PoseEstimate,
}

#[derive(Debug, Clone)]
pub enum AiError {
    ApiError(String),
    InferenceError(String),
    QueueFull,
    InvalidInput,
    ModelNotLoaded,
    MemoryExceeded,
    Cancelled,
}

impl fmt::Display for AiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AiError::ApiError(msg) => write!(f, "API error: {}", msg),
            AiError::InferenceError(msg) => write!(f, "Inference error: {}", msg),
            AiError::QueueFull => write!(f, "AI job queue is full"),
            AiError::InvalidInput => write!(f, "Invalid input for AI processing"),
            AiError::ModelNotLoaded => write!(f, "AI model not loaded"),
            AiError::MemoryExceeded => write!(f, "Memory limit exceeded"),
            AiError::Cancelled => write!(f, "AI job was cancelled"),
        }
    }
}

impl std::error::Error for AiError {}
