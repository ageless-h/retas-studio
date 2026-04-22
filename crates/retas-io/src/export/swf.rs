use super::ExportError;
use retas_core::{Document, Color8};

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
