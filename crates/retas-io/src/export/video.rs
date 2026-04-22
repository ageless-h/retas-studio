use super::{ExportError, ImageFormat, ImageExportOptions, VideoFormat, VideoExportOptions};
use retas_core::Document;

pub struct VideoExporter;

impl VideoExporter {
    pub fn export_animation(
        document: &Document,
        path: &std::path::Path,
        options: &VideoExportOptions,
    ) -> Result<(), ExportError> {
        match options.format {
            VideoFormat::GifAnimation => {
                Self::export_gif(document, path, options)
            }
            _ => {
                Self::export_video_with_ffmpeg(document, path, options)
            }
        }
    }

    fn export_video_with_ffmpeg(
        document: &Document,
        path: &std::path::Path,
        options: &VideoExportOptions,
    ) -> Result<(), ExportError> {
        let temp_dir = std::env::temp_dir().join("retas_video_export");
        std::fs::create_dir_all(&temp_dir)?;

        let frame_files: Vec<std::path::PathBuf> = (options.start_frame..=options.end_frame)
            .map(|frame| {
                let frame_path = temp_dir.join(format!("frame_{:04}.png", frame));
                let img_options = ImageExportOptions::new(ImageFormat::Png)
                    .with_size(options.width, options.height);
                (frame, frame_path, img_options)
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|(frame, frame_path, img_options)| {
                crate::export::ImageExporter::export_document(document, frame, &frame_path, &img_options)
                    .map(|_| frame_path)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let input_pattern = temp_dir.join("frame_%04d.png");
        let codec = match options.format {
            VideoFormat::Mp4 => "libx264",
            VideoFormat::WebM => "libvpx-vp9",
            VideoFormat::Avi => "libxvid",
            VideoFormat::Mov => "libx264",
            _ => "libx264",
        };

        let pix_fmt = match options.format {
            VideoFormat::WebM => "yuv420p",
            _ => "yuv420p",
        };

        let crf = match options.quality {
            0..=30 => 30,
            31..=60 => 23,
            61..=90 => 18,
            _ => 15,
        };

        let output = std::process::Command::new("ffmpeg")
            .args(&[
                "-y",
                "-framerate", &format!("{}", options.frame_rate),
                "-i", &input_pattern.to_string_lossy(),
                "-c:v", codec,
                "-pix_fmt", pix_fmt,
                "-crf", &crf.to_string(),
                "-preset", "medium",
                "-movflags", "+faststart",
                &path.to_string_lossy(),
            ])
            .output();

        for frame_file in &frame_files {
            std::fs::remove_file(frame_file).ok();
        }
        std::fs::remove_dir(&temp_dir).ok();

        match output {
            Ok(result) if result.status.success() => Ok(()),
            Ok(result) => {
                let stderr = String::from_utf8_lossy(&result.stderr);
                Err(ExportError::InvalidDocument(format!(
                    "FFmpeg failed: {}. Ensure ffmpeg is installed and in PATH.",
                    stderr.lines().next().unwrap_or("unknown error")
                )))
            }
            Err(e) => {
                Err(ExportError::InvalidDocument(format!(
                    "Failed to spawn FFmpeg: {}. Ensure ffmpeg is installed and in PATH.", e
                )))
            }
        }
    }

    fn export_gif(
        document: &Document,
        path: &std::path::Path,
        options: &VideoExportOptions,
    ) -> Result<(), ExportError> {
        use std::fs::File;
        use std::io::BufWriter;
        
        let file = File::create(path)?;
        let mut encoder = gif::Encoder::new(BufWriter::new(file), options.width as u16, options.height as u16, &[])?;
        encoder.set_repeat(gif::Repeat::Infinite)?;

        for frame in options.start_frame..=options.end_frame {
            let mut buffer = vec![0u8; (options.width * options.height) as usize];
            
            let bg = document.settings.background_color;
            let bg_gray = (bg.r as u16 * 299 + bg.g as u16 * 587 + bg.b as u16 * 114) / 1000;
            for pixel in buffer.iter_mut() {
                *pixel = bg_gray as u8;
            }

            for layer_id in document.timeline.layer_order.iter().rev() {
                if let Some(layer) = document.layers.get(layer_id) {
                    if !layer.base().visible {
                        continue;
                    }
                    
                    if let retas_core::Layer::Raster(raster) = layer {
                        if let Some(frame_data) = raster.frames.get(&frame) {
                            for y in 0..frame_data.height.min(options.height) {
                                for x in 0..frame_data.width.min(options.width) {
                                    let src_idx = (y * frame_data.width + x) as usize * 4;
                                    let dst_idx = (y * options.width + x) as usize;
                                    
                                    if src_idx + 2 < frame_data.image_data.len() {
                                        let r = frame_data.image_data[src_idx] as u16;
                                        let g = frame_data.image_data[src_idx + 1] as u16;
                                        let b = frame_data.image_data[src_idx + 2] as u16;
                                        let gray = (r * 299 + g * 587 + b * 114) / 1000;
                                        buffer[dst_idx] = gray as u8;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let frame_delay = (100.0 / options.frame_rate) as u16;
            let gif_frame = gif::Frame::from_indexed_pixels(
                options.width as u16,
                options.height as u16,
                buffer,
                None,
            );
            
            let mut gif_frame = gif_frame;
            gif_frame.delay = frame_delay;
            encoder.write_frame(&gif_frame)?;
        }

        Ok(())
    }
}
