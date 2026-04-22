use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};
use retas_core::uuid::Uuid;

use super::{AiError, AiPipeline};

#[derive(Clone, Debug)]
pub enum AiJobType {
    AutoColor,
    Inbetween,
    StyleTransfer,
    #[allow(dead_code)]
    LineCleanup,
    #[allow(dead_code)]
    BackgroundGen,
}

pub struct AiJob {
    pub _id: String,
    pub job_type: AiJobType,
    pub input_data: Vec<u8>,
    pub params: serde_json::Value,
    pub response: oneshot::Sender<JobResult>,
}

pub type JobResult = Result<Vec<u8>, AiError>;

pub struct AiQueue {
    tx: mpsc::Sender<AiJob>,
    pending_count: Arc<Mutex<usize>>,
    max_size: usize,
}

impl AiQueue {
    pub fn new(max_size: usize, pipeline: Arc<AiPipeline>) -> Self {
        let (tx, mut rx) = mpsc::channel::<AiJob>(max_size);
        let pending_count = Arc::new(Mutex::new(0));
        let pending = pending_count.clone();

        tokio::spawn(async move {
            while let Some(job) = rx.recv().await {
                {
                    let mut count = pending.lock().await;
                    *count += 1;
                }

                let result = Self::process_job(&pipeline, job).await;

                {
                    let mut count = pending.lock().await;
                    *count -= 1;
                }

                let _ = result;
            }
        });

        Self {
            tx,
            pending_count,
            max_size,
        }
    }

    pub async fn submit(
        &self,
        job_type: AiJobType,
        input_data: Vec<u8>,
        params: serde_json::Value,
    ) -> Result<JobResult, AiError> {
        let (tx, rx) = oneshot::channel();

        let job = AiJob {
            _id: Uuid::new_v4().to_string(),
            job_type,
            input_data,
            params,
            response: tx,
        };

        self.tx
            .send(job)
            .await
            .map_err(|_| AiError::QueueFull)?;

        rx.await.map_err(|_| AiError::ApiError("Job cancelled".to_string()))
    }

    pub async fn pending_count(&self) -> usize {
        *self.pending_count.lock().await
    }

    pub fn is_full(&self) -> bool {
        self.pending_count.try_lock().map(|c| *c >= self.max_size).unwrap_or(true)
    }

    async fn process_job(pipeline: &AiPipeline, job: AiJob) -> Result<(), ()> {
        let result = match job.job_type {
            AiJobType::AutoColor => {
                let palette = vec![(255, 0, 0), (0, 255, 0), (0, 0, 255)];
                pipeline.auto_color(&job.input_data, &palette).await
            }
            AiJobType::Inbetween => {
                let next = job
                    .params
                    .get("next_frame")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_u64().map(|n| n as u8))
                            .collect::<Vec<u8>>()
                    })
                    .unwrap_or_default();
                pipeline.generate_inbetween(&job.input_data, &next).await
            }
            AiJobType::StyleTransfer => {
                let style = job
                    .params
                    .get("style")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_u64().map(|n| n as u8))
                            .collect::<Vec<u8>>()
                    })
                    .unwrap_or_default();
                pipeline.style_transfer(&job.input_data, &style).await
            }
            _ => Err(AiError::InvalidInput),
        };

        let _ = job.response.send(result);
        Ok(())
    }
}
