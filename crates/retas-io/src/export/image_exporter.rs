use super::{ExportError, ImageFormat, ImageExportOptions};
use retas_core::{Layer, RasterLayer};

pub struct ImageExporter;

impl ImageExporter {
    pub fn export_pixels(
        pixels: &[u8],
        width: u32,
        height: u32,
        path: &std::path::Path,
    ) -> Result<(), ExportError> {
        let img = image::RgbaImage::from_raw(width, height, pixels.to_vec())
            .ok_or_else(|| ExportError::InvalidDocument("Failed to create image buffer".to_string()))?;
        
        img.save(path)?;
        Ok(())
    }
    
    pub fn export_layer(
        layer: &Layer,
        path: &std::path::Path,
        options: &ImageExportOptions,
    ) -> Result<(), ExportError> {
        match layer {
            Layer::Raster(raster_layer) => {
                Self::export_raster_layer(raster_layer, path, options)
            }
            _ => Err(ExportError::UnsupportedFormat("Only raster layers can be exported as images".to_string())),
        }
    }

    pub fn export_raster_layer(
        layer: &RasterLayer,
        path: &std::path::Path,
        options: &ImageExportOptions,
    ) -> Result<(), ExportError> {
        let (width, height) = if let Some(frame) = layer.frames.get(&layer.current_frame) {
            (frame.width.max(1), frame.height.max(1))
        } else {
            (640, 480)
        };
        let width = options.width.unwrap_or(width);
        let height = options.height.unwrap_or(height);
        
        let mut buffer = vec![0u8; (width * height * 4) as usize];
        
        if let Some(bg) = options.background {
            for pixel in buffer.chunks_exact_mut(4) {
                pixel[0] = bg.r;
                pixel[1] = bg.g;
                pixel[2] = bg.b;
                pixel[3] = bg.a;
            }
        }

        let img = image::RgbaImage::from_raw(width, height, buffer)
            .ok_or_else(|| ExportError::InvalidDocument("Failed to create image buffer".to_string()))?;

        Self::save_image(&img, path, options)
    }

    pub fn export_document(
        document: &retas_core::Document,
        frame: u32,
        path: &std::path::Path,
        options: &ImageExportOptions,
    ) -> Result<(), ExportError> {
        let width = options.width.unwrap_or(document.settings.resolution.width as u32);
        let height = options.height.unwrap_or(document.settings.resolution.height as u32);
        
        let mut buffer = vec![0u8; (width * height * 4) as usize];
        
        if let Some(bg) = options.background {
            for pixel in buffer.chunks_exact_mut(4) {
                pixel[0] = bg.r;
                pixel[1] = bg.g;
                pixel[2] = bg.b;
                pixel[3] = bg.a;
            }
        } else {
            let bg = document.settings.background_color;
            for pixel in buffer.chunks_exact_mut(4) {
                pixel[0] = bg.r;
                pixel[1] = bg.g;
                pixel[2] = bg.b;
                pixel[3] = bg.a;
            }
        }

        for layer_id in document.timeline.layer_order.iter().rev() {
            if let Some(layer) = document.layers.get(layer_id) {
                if !layer.base().visible {
                    continue;
                }
                
                Self::composite_layer(&mut buffer, width, height, layer, frame);
            }
        }

        let img = image::RgbaImage::from_raw(width, height, buffer)
            .ok_or_else(|| ExportError::InvalidDocument("Failed to create image buffer".to_string()))?;

        Self::save_image(&img, path, options)
    }

    fn composite_layer(
        buffer: &mut [u8],
        width: u32,
        height: u32,
        layer: &Layer,
        _frame: u32,
    ) {
        let opacity = layer.base().opacity;
        
        match layer {
            Layer::Raster(raster) => {
                if let Some(frame_data) = raster.frames.get(&raster.current_frame) {
                    let layer_width = frame_data.width;
                    let layer_height = frame_data.height;
                    let offset_x = frame_data.bounds.as_ref().map(|b| b.origin.x as i32).unwrap_or(0);
                    let offset_y = frame_data.bounds.as_ref().map(|b| b.origin.y as i32).unwrap_or(0);
                    
                    for y in 0..layer_height {
                        for x in 0..layer_width {
                            let dst_x = x as i32 + offset_x;
                            let dst_y = y as i32 + offset_y;
                            
                            if dst_x < 0 || dst_x >= width as i32 || dst_y < 0 || dst_y >= height as i32 {
                                continue;
                            }
                            
                            let src_idx = (y * layer_width + x) as usize * 4;
                            let dst_idx = (dst_y as u32 * width + dst_x as u32) as usize * 4;
                            
                            if src_idx + 3 < frame_data.image_data.len() && dst_idx + 3 < buffer.len() {
                                let fg_r = frame_data.image_data[src_idx] as f64 / 255.0;
                                let fg_g = frame_data.image_data[src_idx + 1] as f64 / 255.0;
                                let fg_b = frame_data.image_data[src_idx + 2] as f64 / 255.0;
                                let fg_a = frame_data.image_data[src_idx + 3] as f64 / 255.0 * opacity;
                                
                                let bg_r = buffer[dst_idx] as f64 / 255.0;
                                let bg_g = buffer[dst_idx + 1] as f64 / 255.0;
                                let bg_b = buffer[dst_idx + 2] as f64 / 255.0;
                                let bg_a = buffer[dst_idx + 3] as f64 / 255.0;
                                
                                let out_a = fg_a + bg_a * (1.0 - fg_a);
                                
                                if out_a > 0.0 {
                                    buffer[dst_idx] = ((fg_r * fg_a + bg_r * bg_a * (1.0 - fg_a)) / out_a * 255.0) as u8;
                                    buffer[dst_idx + 1] = ((fg_g * fg_a + bg_g * bg_a * (1.0 - fg_a)) / out_a * 255.0) as u8;
                                    buffer[dst_idx + 2] = ((fg_b * fg_a + bg_b * bg_a * (1.0 - fg_a)) / out_a * 255.0) as u8;
                                    buffer[dst_idx + 3] = (out_a * 255.0) as u8;
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn save_image(
        img: &image::RgbaImage,
        path: &std::path::Path,
        options: &ImageExportOptions,
    ) -> Result<(), ExportError> {
        match options.format {
            ImageFormat::Png => {
                img.save(path)?;
            }
            ImageFormat::Jpeg => {
                let rgb_img = image::DynamicImage::ImageRgba8(img.clone()).to_rgb8();
                let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                    std::fs::File::create(path)?,
                    options.quality,
                );
                encoder.encode(&rgb_img, img.width(), img.height(), image::ExtendedColorType::Rgb8)?;
            }
            ImageFormat::Bmp => {
                img.save(path)?;
            }
            ImageFormat::WebP => {
                img.save(path)?;
            }
            ImageFormat::Gif => {
                img.save(path)?;
            }
            ImageFormat::Tiff => {
                img.save(path)?;
            }
        }
        Ok(())
    }
}

pub struct ImageImporter;

impl ImageImporter {
    pub fn import(path: &std::path::Path) -> Result<(Vec<u8>, u32, u32), ExportError> {
        let img = image::open(path)?;
        let rgba = img.to_rgba8();
        let width = rgba.width();
        let height = rgba.height();
        let data = rgba.into_raw();
        
        Ok((data, width, height))
    }

    pub fn import_as_layer(
        path: &std::path::Path,
        name: impl Into<String>,
    ) -> Result<RasterLayer, ExportError> {
        let (data, width, height) = Self::import(path)?;
        
        let mut layer = RasterLayer::new(name);
        let frame = retas_core::RasterFrame {
            frame_number: 0,
            image_data: std::sync::Arc::new(data),
            width,
            height,
            bounds: None,
        };
        layer.frames.insert(0, frame);
        
        Ok(layer)
    }
}
