use image::{DynamicImage, RgbaImage};
use std::sync::Arc;

use super::{AiApiClient, AiEngine, AiError, InferenceResult};

#[derive(Clone, Debug)]
pub struct PipelineConfig {
    pub use_local_first: bool,
    pub max_image_size: u32,
    pub output_format: String,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            use_local_first: true,
            max_image_size: 2048,
            output_format: "png".to_string(),
        }
    }
}

pub struct AiPipeline {
    pub local_engine: Option<Arc<dyn AiEngine>>,
    pub api_client: Option<AiApiClient>,
    pub config: PipelineConfig,
}

impl AiPipeline {
    pub fn new(config: PipelineConfig) -> Self {
        Self {
            local_engine: None,
            api_client: None,
            config,
        }
    }

    pub fn with_local_engine(mut self, engine: Arc<dyn AiEngine>) -> Self {
        self.local_engine = Some(engine);
        self
    }

    pub fn with_api_client(mut self, client: AiApiClient) -> Self {
        self.api_client = Some(client);
        self
    }

    pub async fn auto_color(
        &self,
        line_art: &[u8],
        palette: &[(u8, u8, u8)],
    ) -> Result<Vec<u8>, AiError> {
        if self.config.use_local_first {
            if let Some(ref engine) = self.local_engine {
                if engine.is_model_loaded() {
                    let result = tokio::task::spawn_blocking({
                        let engine = engine.clone();
                        let input = line_art.to_vec();
                        move || {
                            let img = image::load_from_memory(&input)
                                .map_err(|_e| AiError::InvalidInput)?;
                            let rgba = img.to_rgba8();
                            engine.infer(
                                rgba.as_raw(),
                                rgba.width(),
                                rgba.height(),
                            )
                        }
                    })
                    .await
                    .map_err(|e| AiError::InferenceError(e.to_string()))?;

                    return match result {
                        Ok(inference) => self.encode_output(&inference),
                        Err(e) if self.api_client.is_some() => {
                            let _ = e;
                            self.fallback_api_auto_color(line_art, palette).await
                        }
                        Err(e) => Err(e),
                    };
                }
            }
        }

        if self.api_client.is_some() {
            self.fallback_api_auto_color(line_art, palette).await
        } else {
            Err(AiError::ModelNotLoaded)
        }
    }

    pub async fn generate_inbetween(
        &self,
        prev_frame: &[u8],
        next_frame: &[u8],
    ) -> Result<Vec<u8>, AiError> {
        if let Some(ref client) = self.api_client {
            let results = client.inbetween_frames(prev_frame, next_frame, 1).await?;
            results.into_iter().next().ok_or(AiError::InvalidInput)
        } else {
            Err(AiError::ApiError("No API client configured".to_string()))
        }
    }

    pub async fn style_transfer(
        &self,
        _source: &[u8],
        _style_reference: &[u8],
    ) -> Result<Vec<u8>, AiError> {
        if let Some(ref client) = self.api_client {
            let prompt = "style transfer animation frame";
            let result = client.generate_image(prompt, 512, 512).await?;
            Ok(result)
        } else {
            Err(AiError::ApiError("No API client configured".to_string()))
        }
    }

    async fn fallback_api_auto_color(
        &self,
        _line_art: &[u8],
        _palette: &[(u8, u8, u8)],
    ) -> Result<Vec<u8>, AiError> {
        if let Some(ref client) = self.api_client {
            let prompt = "auto color animation frame";
            client.generate_image(prompt, 512, 512).await
        } else {
            Err(AiError::ApiError("No API client available".to_string()))
        }
    }

    fn encode_output(&self, result: &InferenceResult) -> Result<Vec<u8>, AiError> {
        let img = RgbaImage::from_raw(result.width, result.height, result.data.clone())
            .ok_or(AiError::InvalidInput)?;

        let mut output = Vec::new();
        let dynamic = DynamicImage::ImageRgba8(img);

        match self.config.output_format.as_str() {
            "png" => {
                dynamic
                    .write_to(&mut std::io::Cursor::new(&mut output), image::ImageFormat::Png)
                    .map_err(|e| AiError::InferenceError(e.to_string()))?;
            }
            "jpeg" | "jpg" => {
                let rgb = dynamic.to_rgb8();
                rgb.write_to(
                    &mut std::io::Cursor::new(&mut output),
                    image::ImageFormat::Jpeg,
                )
                .map_err(|e| AiError::InferenceError(e.to_string()))?;
            }
            _ => return Err(AiError::InvalidInput),
        }

        Ok(output)
    }
}
