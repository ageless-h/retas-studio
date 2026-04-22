use super::{ExportError, ImageFormat, ImageExportOptions, VideoFormat, VideoExportOptions};
use retas_core::Document;

pub struct VideoExporter;

impl VideoExporter {
    pub fn export_animation(
        document: &Document,
        path: &std::path::Path,
        options: &VideoExportOptions,
    ) -> Result<(), ExportError> {
        let total_frames = options.end_frame - options.start_frame + 1;
        
        match options.format {
            VideoFormat::GifAnimation => {
                Self::export_gif(document, path, options)
            }
            _ => {
                for frame in options.start_frame..=options.end_frame {
                    let frame_path = path.with_extension(format!("frame_{:04}.png", frame));
                    let img_options = ImageExportOptions::new(ImageFormat::Png)
                        .with_size(options.width, options.height);
                    
                    crate::export::ImageExporter::export_document(document, frame, &frame_path, &img_options)?;
                }
                
                Err(ExportError::UnsupportedFormat(
                    "Video export requires external ffmpeg. Frames have been exported.".to_string()
                ))
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
