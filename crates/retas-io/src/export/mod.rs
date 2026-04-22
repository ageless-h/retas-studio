//! Export and import functionality for RETAS Studio documents.
//!
//! This module provides exporters for various image, video, and vector formats,
//! as well as importers for bringing external assets into RETAS documents.
//!
//! # Supported Formats
//!
//! ## Image Export
//! - PNG, JPEG, BMP, TIFF, WebP, GIF
//!
//! ## Video Export  
//! - Animated GIF (native)
//! - MP4, WebM, AVI, MOV (via frame sequence + ffmpeg)
//!
//! ## Vector Export
//! - SVG (Scalable Vector Graphics)
//! - SWF (Flash animation format)

use retas_core::{Document, Layer, RasterLayer, Color8};
use retas_vector::{Path, BezierCurve};
use thiserror::Error;

/// Errors that can occur during export or import operations.
#[derive(Debug, Error)]
pub enum ExportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),
    #[error("GIF encoding error: {0}")]
    Gif(#[from] gif::EncodingError),
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    #[error("Invalid document: {0}")]
    InvalidDocument(String),
}

/// Supported image export formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Bmp,
    Tiff,
    WebP,
    Gif,
}

impl ImageFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Bmp => "bmp",
            ImageFormat::Tiff => "tiff",
            ImageFormat::WebP => "webp",
            ImageFormat::Gif => "gif",
        }
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "png" => Some(ImageFormat::Png),
            "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
            "bmp" => Some(ImageFormat::Bmp),
            "tiff" | "tif" => Some(ImageFormat::Tiff),
            "webp" => Some(ImageFormat::WebP),
            "gif" => Some(ImageFormat::Gif),
            _ => None,
        }
    }
}

/// Supported video export formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VideoFormat {
    Mp4,
    WebM,
    Avi,
    Mov,
    GifAnimation,
}

impl VideoFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            VideoFormat::Mp4 => "mp4",
            VideoFormat::WebM => "webm",
            VideoFormat::Avi => "avi",
            VideoFormat::Mov => "mov",
            VideoFormat::GifAnimation => "gif",
        }
    }
}

/// Configuration options for image export operations.
#[derive(Debug, Clone)]
pub struct ImageExportOptions {
    pub format: ImageFormat,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub quality: u8,
    pub background: Option<Color8>,
}

impl Default for ImageExportOptions {
    fn default() -> Self {
        Self {
            format: ImageFormat::Png,
            width: None,
            height: None,
            quality: 95,
            background: None,
        }
    }
}

impl ImageExportOptions {
    pub fn new(format: ImageFormat) -> Self {
        Self {
            format,
            ..Default::default()
        }
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    pub fn with_quality(mut self, quality: u8) -> Self {
        self.quality = quality;
        self
    }

    pub fn with_background(mut self, color: Color8) -> Self {
        self.background = Some(color);
        self
    }
}

/// Configuration options for video/animation export operations.
#[derive(Debug, Clone)]
pub struct VideoExportOptions {
    pub format: VideoFormat,
    pub width: u32,
    pub height: u32,
    pub frame_rate: f64,
    pub quality: u8,
    pub start_frame: u32,
    pub end_frame: u32,
    pub loop_count: u32,
}

impl Default for VideoExportOptions {
    fn default() -> Self {
        Self {
            format: VideoFormat::Mp4,
            width: 1920,
            height: 1080,
            frame_rate: 24.0,
            quality: 80,
            start_frame: 0,
            end_frame: 100,
            loop_count: 0,
        }
    }
}

impl VideoExportOptions {
    pub fn new(format: VideoFormat, width: u32, height: u32, frame_rate: f64) -> Self {
        Self {
            format,
            width,
            height,
            frame_rate,
            ..Default::default()
        }
    }

    pub fn with_range(mut self, start: u32, end: u32) -> Self {
        self.start_frame = start;
        self.end_frame = end;
        self
    }

    pub fn with_quality(mut self, quality: u8) -> Self {
        self.quality = quality;
        self
    }
}

/// Image export and import implementation.
pub mod image_exporter;
/// Video and animation export implementation.
pub mod video;
/// SVG vector export implementation.
pub mod svg;
/// SWF (Flash) export implementation.
pub mod swf;

pub use image_exporter::{ImageExporter, ImageImporter};
pub use video::VideoExporter;
pub use svg::SvgExporter;
pub use swf::{SwfExporter, SwfExportOptions};
