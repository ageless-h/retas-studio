use retas_core::{Document, Layer, RasterLayer, Color8};
use retas_vector::{Path, BezierCurve};
use thiserror::Error;

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
        // Get dimensions from the current frame or use defaults
        let (width, height) = if let Some(frame) = layer.frames.get(&layer.current_frame) {
            (frame.width.max(1), frame.height.max(1))
        } else {
            (640, 480) // Default dimensions
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
        document: &Document,
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
                encoder.encode(&rgb_img, img.width(), img.height(), image::ColorType::Rgb8)?;
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
                    
                    ImageExporter::export_document(document, frame, &frame_path, &img_options)?;
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
                    
                    if let Layer::Raster(raster) = layer {
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
                &buffer,
                None,
            );
            
            let mut gif_frame = gif_frame;
            gif_frame.delay = frame_delay;
            encoder.write_frame(&gif_frame)?;
        }

        Ok(())
    }
}

pub struct SvgExporter;

impl SvgExporter {
    pub fn export_path(path: &Path, output_path: &std::path::Path) -> Result<(), ExportError> {
        let mut svg = String::new();
        svg.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        svg.push_str("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1920\" height=\"1080\">\n");
        
        svg.push_str(&Self::path_to_svg(path));
        
        svg.push_str("</svg>\n");
        
        std::fs::write(output_path, svg)?;
        Ok(())
    }

    pub fn export_bezier(curve: &BezierCurve, output_path: &std::path::Path) -> Result<(), ExportError> {
        let mut svg = String::new();
        svg.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        svg.push_str("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1920\" height=\"1080\">\n");
        
        svg.push_str(&Self::bezier_to_svg(curve));
        
        svg.push_str("</svg>\n");
        
        std::fs::write(output_path, svg)?;
        Ok(())
    }

    fn path_to_svg(path: &Path) -> String {
        let mut d = String::new();
        
        for cmd in &path.commands {
            match cmd {
                retas_vector::PathCommand::MoveTo(p) => {
                    d.push_str(&format!("M {} {} ", p.x, p.y));
                }
                retas_vector::PathCommand::LineTo(p) => {
                    d.push_str(&format!("L {} {} ", p.x, p.y));
                }
                retas_vector::PathCommand::CurveTo(c1, c2, end) => {
                    d.push_str(&format!("C {} {} {} {} {} {} ", c1.x, c1.y, c2.x, c2.y, end.x, end.y));
                }
                retas_vector::PathCommand::QuadTo(ctrl, end) => {
                    d.push_str(&format!("Q {} {} {} {} ", ctrl.x, ctrl.y, end.x, end.y));
                }
                retas_vector::PathCommand::ArcTo { radii, rotation, large_arc, sweep, end } => {
                    let la = if *large_arc { 1 } else { 0 };
                    let sw = if *sweep { 1 } else { 0 };
                    d.push_str(&format!("A {} {} {} {} {} {} {} ", radii.0, radii.1, rotation, la, sw, end.x, end.y));
                }
                retas_vector::PathCommand::Close => {
                    d.push_str("Z ");
                }
            }
        }
        
        format!("  <path d=\"{}\" fill=\"none\" stroke=\"black\" stroke-width=\"1\"/>\n", d.trim())
    }

    fn bezier_to_svg(curve: &BezierCurve) -> String {
        if curve.points.is_empty() {
            return String::new();
        }

        let mut d = String::new();
        let first = &curve.points[0];
        d.push_str(&format!("M {} {} ", first.point.x, first.point.y));

        for i in 0..curve.points.len() - 1 {
            let p0 = &curve.points[i];
            let p1 = &curve.points[i + 1];
            
            if let (Some(out), Some(in_)) = (p0.out_handle, p1.in_handle) {
                d.push_str(&format!("C {} {} {} {} {} {} ", out.x, out.y, in_.x, in_.y, p1.point.x, p1.point.y));
            } else {
                d.push_str(&format!("L {} {} ", p1.point.x, p1.point.y));
            }
        }

        if curve.closed {
            d.push_str("Z");
        }

        format!("  <path d=\"{}\" fill=\"none\" stroke=\"black\" stroke-width=\"1\"/>\n", d.trim())
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
            image_data: data,
            width,
            height,
            bounds: None,
        };
        layer.frames.insert(0, frame);
        
        Ok(layer)
    }
}

#[derive(Debug, Clone)]
pub struct SwfExportOptions {
    pub width: u32,
    pub height: u32,
    pub frame_rate: f64,
    pub background_color: Color8,
    pub compress: bool,
    pub version: u8,
}

impl Default for SwfExportOptions {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            frame_rate: 24.0,
            background_color: Color8::new(255, 255, 255, 255),
            compress: true,
            version: 10,
        }
    }
}

pub struct SwfExporter;

impl SwfExporter {
    pub fn export(
        document: &Document,
        path: &std::path::Path,
        options: &SwfExportOptions,
    ) -> Result<(), ExportError> {
        let mut swf = SwfWriter::new(options);
        swf.write_header()?;
        swf.write_background(options.background_color)?;
        swf.write_frames(document)?;
        swf.write_end_tag()?;
        swf.save(path)?;
        Ok(())
    }

    pub fn export_animation(
        frames: &[Vec<u8>],
        width: u32,
        height: u32,
        frame_rate: f64,
        path: &std::path::Path,
    ) -> Result<(), ExportError> {
        let options = SwfExportOptions {
            width,
            height,
            frame_rate,
            ..Default::default()
        };
        
        let mut swf = SwfWriter::new(&options);
        swf.write_header()?;
        swf.write_background(Color8::new(255, 255, 255, 255))?;
        
        for (i, frame_data) in frames.iter().enumerate() {
            swf.write_raw_frame(frame_data, width, height, i)?;
        }
        
        swf.write_end_tag()?;
        swf.save(path)?;
        Ok(())
    }
}

struct SwfWriter {
    buffer: Vec<u8>,
    width: u32,
    height: u32,
    frame_rate: f64,
    version: u8,
    compress: bool,
    frame_count: u16,
}

impl SwfWriter {
    fn new(options: &SwfExportOptions) -> Self {
        Self {
            buffer: Vec::new(),
            width: options.width,
            height: options.height,
            frame_rate: options.frame_rate,
            version: options.version,
            compress: options.compress,
            frame_count: 0,
        }
    }

    fn write_header(&mut self) -> Result<(), ExportError> {
        let signature = if self.compress { b"CWS" } else { b"FWS" };
        self.buffer.extend_from_slice(signature);
        self.buffer.push(self.version);
        
        self.buffer.extend_from_slice(&[0, 0, 0, 0]);
        
        let frame_size = self.encode_rect(0, 0, self.width as i32 * 20, self.height as i32 * 20);
        self.buffer.extend_from_slice(&frame_size);
        
        let frame_rate_fixed = (self.frame_rate * 256.0) as u16;
        self.buffer.extend_from_slice(&frame_rate_fixed.to_le_bytes());
        
        self.buffer.extend_from_slice(&[0, 0]);
        
        Ok(())
    }

    fn finalize_header(&mut self) {
        let frame_count = self.frame_count;
        let len = self.buffer.len();
        self.buffer[len - 2] = (frame_count & 0xFF) as u8;
        self.buffer[len - 1] = ((frame_count >> 8) & 0xFF) as u8;
    }

    fn encode_rect(&self, x_min: i32, y_min: i32, x_max: i32, y_max: i32) -> Vec<u8> {
        let mut bits = Vec::new();
        
        let n_bits = 1.max(
            Self::count_signed_bits(x_min)
                .max(Self::count_signed_bits(y_min))
                .max(Self::count_signed_bits(x_max))
                .max(Self::count_signed_bits(y_max))
        ) + 1;
        
        bits.push(n_bits as u8);
        bits.extend_from_slice(&Self::encode_signed_bits(x_min, n_bits));
        bits.extend_from_slice(&Self::encode_signed_bits(x_max, n_bits));
        bits.extend_from_slice(&Self::encode_signed_bits(y_min, n_bits));
        bits.extend_from_slice(&Self::encode_signed_bits(y_max, n_bits));
        
        Self::bits_to_bytes(&bits)
    }

    fn count_signed_bits(value: i32) -> u8 {
        if value == 0 { return 0; }
        let abs_val = value.abs();
        let mut count = 0u8;
        let mut v = abs_val;
        while v != 0 {
            count += 1;
            v >>= 1;
        }
        count
    }

    fn encode_signed_bits(value: i32, n_bits: u8) -> Vec<u8> {
        let mut bits = Vec::with_capacity(n_bits as usize);
        let is_negative = value < 0;
        let mut val = if is_negative { (-value) as u32 } else { value as u32 };
        
        if is_negative {
            val = !val;
        }
        
        for i in (0..n_bits).rev() {
            bits.push(((val >> i) & 1) as u8);
        }
        bits
    }

    fn encode_bits(value: i32, n_bits: u8) -> Vec<u8> {
        let mut bits = Vec::with_capacity(n_bits as usize);
        for i in (0..n_bits).rev() {
            bits.push(((value >> i) & 1) as u8);
        }
        bits
    }

    fn bits_to_bytes(bits: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut current_byte = 0u8;
        let mut bit_pos = 7;
        
        for bit in bits {
            current_byte |= (bit & 1) << bit_pos;
            if bit_pos == 0 {
                bytes.push(current_byte);
                current_byte = 0;
                bit_pos = 7;
            } else {
                bit_pos -= 1;
            }
        }
        
        if bit_pos != 7 {
            bytes.push(current_byte);
        }
        
        bytes
    }

    fn write_background(&mut self, color: Color8) -> Result<(), ExportError> {
        self.buffer.push(9);
        self.buffer.push(3);
        self.buffer.push(color.r);
        self.buffer.push(color.g);
        self.buffer.push(color.b);
        Ok(())
    }

    fn write_frames(&mut self, document: &Document) -> Result<(), ExportError> {
        let start_frame = document.timeline.start_frame;
        let end_frame = document.timeline.end_frame;
        
        for _frame in start_frame..=end_frame {
            self.write_show_frame()?;
            self.frame_count += 1;
        }
        
        self.finalize_header();
        Ok(())
    }

    fn write_raw_frame(&mut self, data: &[u8], width: u32, height: u32, _frame_index: usize) -> Result<(), ExportError> {
        let depth = 1u16;
        let character_id = (self.frame_count + 1) as u16;
        
        self.write_define_shape(character_id, data, width, height)?;
        self.write_place_object(character_id, depth)?;
        self.write_show_frame()?;
        
        self.frame_count += 1;
        Ok(())
    }

    fn write_define_shape(&mut self, id: u16, _data: &[u8], width: u32, height: u32) -> Result<(), ExportError> {
        let tag_type: u16 = 2;
        let tag_header = (tag_type << 6) | 63;
        
        self.buffer.extend_from_slice(&tag_header.to_le_bytes());
        
        let mut shape_data = Vec::new();
        shape_data.extend_from_slice(&id.to_le_bytes());
        
        let bounds = self.encode_rect(0, 0, width as i32 * 20, height as i32 * 20);
        shape_data.extend_from_slice(&bounds);
        
        shape_data.extend_from_slice(&bounds);
        
        shape_data.push(0);
        shape_data.push(1);
        shape_data.push(0);
        
        let shape_len = shape_data.len() as u32;
        self.buffer.extend_from_slice(&shape_len.to_le_bytes());
        self.buffer.extend_from_slice(&shape_data);
        
        Ok(())
    }

    fn write_place_object(&mut self, character_id: u16, depth: u16) -> Result<(), ExportError> {
        let tag_type: u16 = 4;
        let mut tag_data = Vec::new();
        
        tag_data.extend_from_slice(&character_id.to_le_bytes());
        tag_data.extend_from_slice(&depth.to_le_bytes());
        
        tag_data.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
        
        let tag_len = tag_data.len();
        if tag_len < 63 {
            let tag_header = (tag_type << 6) | (tag_len as u16);
            self.buffer.extend_from_slice(&tag_header.to_le_bytes());
        } else {
            let tag_header = (tag_type << 6) | 63;
            self.buffer.extend_from_slice(&tag_header.to_le_bytes());
            self.buffer.extend_from_slice(&(tag_len as u32).to_le_bytes());
        }
        
        self.buffer.extend_from_slice(&tag_data);
        Ok(())
    }

    fn write_show_frame(&mut self) -> Result<(), ExportError> {
        let tag_header: u16 = (1 << 6) | 0;
        self.buffer.extend_from_slice(&tag_header.to_le_bytes());
        Ok(())
    }

    fn write_end_tag(&mut self) -> Result<(), ExportError> {
        self.buffer.push(0);
        self.buffer.push(0);
        Ok(())
    }

    fn save(mut self, path: &std::path::Path) -> Result<(), ExportError> {
        let file_size = self.buffer.len() as u32;
        
        if self.compress {
            let uncompressed_body = self.buffer.split_off(8);
            
            let mut compressed = Vec::new();
            {
                use flate2::write::ZlibEncoder;
                use flate2::Compression;
                let mut encoder = ZlibEncoder::new(&mut compressed, Compression::default());
                std::io::Write::write_all(&mut encoder, &uncompressed_body)?;
                encoder.finish()?;
            }
            
            self.buffer.extend_from_slice(&compressed);
        }
        
        let final_size = self.buffer.len() as u32;
        self.buffer[4] = (final_size & 0xFF) as u8;
        self.buffer[5] = ((final_size >> 8) & 0xFF) as u8;
        self.buffer[6] = ((final_size >> 16) & 0xFF) as u8;
        self.buffer[7] = ((final_size >> 24) & 0xFF) as u8;
        
        std::fs::write(path, &self.buffer)?;
        Ok(())
    }
}
