use reqwest::{Client, Response};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

use super::AiError;

#[derive(Clone, Debug)]
pub struct ApiConfig {
    pub max_concurrent: usize,
    pub pool_max_idle: usize,
    pub pool_idle_timeout_secs: u64,
    pub request_timeout_secs: u64,
    pub base_url: String,
    pub api_key: Option<String>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 10,
            pool_max_idle: 2,
            pool_idle_timeout_secs: 30,
            request_timeout_secs: 120,
            base_url: String::new(),
            api_key: None,
        }
    }
}

#[derive(Clone)]
pub struct AiApiClient {
    client: Client,
    semaphore: Arc<Semaphore>,
    config: ApiConfig,
}

impl AiApiClient {
    pub fn new(config: ApiConfig) -> Result<Self, AiError> {
        let client = Client::builder()
            .pool_max_idle_per_host(config.pool_max_idle)
            .pool_idle_timeout(Duration::from_secs(config.pool_idle_timeout_secs))
            .connect_timeout(Duration::from_secs(10))
            .read_timeout(Duration::from_secs(config.request_timeout_secs))
            .timeout(Duration::from_secs(config.request_timeout_secs + 60))
            .build()
            .map_err(|e| AiError::ApiError(e.to_string()))?;

        let semaphore = Arc::new(Semaphore::new(config.max_concurrent));

        Ok(Self {
            client,
            semaphore,
            config,
        })
    }

    pub async fn generate_image(&self, prompt: &str, width: u32, height: u32) -> Result<Vec<u8>, AiError> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| AiError::ApiError("Semaphore closed".to_string()))?;

        let body = json!({
            "prompt": prompt,
            "width": width,
            "height": height,
            "response_format": "b64_json"
        });

        let mut request = self
            .client
            .post(format!("{}/generate", self.config.base_url))
            .json(&body);

        if let Some(ref key) = self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }

        let response = request
            .send()
            .await
            .map_err(|e| AiError::ApiError(e.to_string()))?;

        self.handle_response(response).await
    }

    pub async fn inbetween_frames(
        &self,
        prev_frame: &[u8],
        next_frame: &[u8],
        count: u32,
    ) -> Result<Vec<Vec<u8>>, AiError> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| AiError::ApiError("Semaphore closed".to_string()))?;

        let body = json!({
            "prev_frame": base64::encode(prev_frame),
            "next_frame": base64::encode(next_frame),
            "count": count
        });

        let mut request = self
            .client
            .post(format!("{}/inbetween", self.config.base_url))
            .json(&body);

        if let Some(ref key) = self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }

        let response = request
            .send()
            .await
            .map_err(|e| AiError::ApiError(e.to_string()))?;

        let bytes = response
            .bytes()
            .await
            .map_err(|e| AiError::ApiError(e.to_string()))?;

        let result: Vec<Vec<u8>> = serde_json::from_slice(&bytes)
            .map_err(|e| AiError::ApiError(format!("JSON parse error: {}", e)))?;

        Ok(result)
    }

    pub async fn health_check(&self) -> Result<bool, AiError> {
        let response = self
            .client
            .get(format!("{}/health", self.config.base_url))
            .send()
            .await
            .map_err(|e| AiError::ApiError(e.to_string()))?;

        Ok(response.status().is_success())
    }

    async fn handle_response(&self, response: Response) -> Result<Vec<u8>, AiError> {
        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AiError::ApiError(format!("HTTP {}: {}", status, text)));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| AiError::ApiError(e.to_string()))?;

        Ok(bytes.to_vec())
    }
}

mod base64 {
    pub fn encode(input: &[u8]) -> String {
        use std::fmt::Write;
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut result = String::new();
        for chunk in input.chunks(3) {
            let b = match chunk.len() {
                1 => [chunk[0], 0, 0],
                2 => [chunk[0], chunk[1], 0],
                _ => [chunk[0], chunk[1], chunk[2]],
            };
            let n = (b[0] as usize) << 16 | (b[1] as usize) << 8 | (b[2] as usize);
            let _ = write!(result, "{}{}{}{}",
                CHARS[(n >> 18) & 63] as char,
                CHARS[(n >> 12) & 63] as char,
                if chunk.len() > 1 { CHARS[(n >> 6) & 63] as char } else { '=' },
                if chunk.len() > 2 { CHARS[n & 63] as char } else { '=' }
            );
        }
        result
    }
}
