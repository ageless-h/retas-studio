use std::io::{Read, Seek};
use std::path::Path;
use retas_core::{Document, Layer, RasterLayer, LayerId, Size, Color8, Color16};
use crate::{IoError, FileReader};

const CEL_MAGIC: [u8; 4] = *b"CEL\x00";

#[derive(Debug, Clone)]
pub struct CelHeader {
    pub version: u16,
    pub flags: u16,
    pub width: u32,
    pub height: u32,
    pub bits_per_pixel: u16,
    pub color_depth: u16,
    pub frame_count: u32,
    pub layer_count: u32,
    pub palette_offset: u32,
    pub layer_table_offset: u32,
    pub frame_table_offset: u32,
    pub image_data_offset: u32,
    pub dpi_x: f32,
    pub dpi_y: f32,
    pub background_color: u32,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct CelLayer {
    pub id: u32,
    pub name: String,
    pub layer_type: u16,
    pub blend_mode: u16,
    pub opacity: u8,
    pub visible: bool,
    pub locked: bool,
    pub first_frame: u32,
    pub frame_count: u32,
}

#[derive(Debug, Clone)]
pub struct CelFrame {
    pub frame_number: u32,
    pub layer_id: u32,
    pub data_offset: u32,
    pub data_size: u32,
    pub uncompressed_size: u32,
    pub compression: u16,
    pub image_format: u16,
    pub offset_x: i32,
    pub offset_y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct CelFile {
    pub header: CelHeader,
    pub layers: Vec<CelLayer>,
    pub frames: Vec<CelFrame>,
    pub palette: Vec<Color16>,
    pub image_data: Vec<u8>,
}

impl CelFile {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, IoError> {
        Self::read_file(path)
    }

    pub fn get_frame_data(&self, frame: &CelFrame) -> Result<Vec<u8>, IoError> {
        let start = frame.data_offset as usize;
        let end = start + frame.data_size as usize;
        
        if end > self.image_data.len() {
            return Err(IoError::InvalidOffset(frame.data_offset));
        }

        let compressed = &self.image_data[start..end];

        match frame.compression {
            0 => Ok(compressed.to_vec()),
            1 => Self::decompress_rle(compressed, frame.uncompressed_size as usize),
            2 => Self::decompress_zlib(compressed),
            _ => Err(IoError::Compression(format!("Unknown compression type: {}", frame.compression))),
        }
    }

    fn decompress_rle(data: &[u8], expected_size: usize) -> Result<Vec<u8>, IoError> {
        let mut result = Vec::with_capacity(expected_size);
        let mut i = 0;

        while i < data.len() && result.len() < expected_size {
            let control = data[i];
            i += 1;

            if control & 0x80 != 0 {
                let count = (control & 0x7F) as usize;
                if i >= data.len() {
                    return Err(IoError::Corrupted("RLE run without value".to_string()));
                }
                let value = data[i];
                i += 1;
                result.extend(std::iter::repeat(value).take(count));
            } else {
                let count = control as usize;
                if i + count > data.len() {
                    return Err(IoError::Corrupted("RLE literal overrun".to_string()));
                }
                result.extend_from_slice(&data[i..i + count]);
                i += count;
            }
        }

        Ok(result)
    }

    fn decompress_zlib(data: &[u8]) -> Result<Vec<u8>, IoError> {
        use std::io::Read;
        let mut decoder = flate2::read::ZlibDecoder::new(data);
        let mut result = Vec::new();
        decoder.read_to_end(&mut result)
            .map_err(|e| IoError::Compression(e.to_string()))?;
        Ok(result)
    }

    pub fn to_document(&self) -> Document {
        let mut doc = Document::new(
            &self.header.name,
            self.header.width as f64,
            self.header.height as f64,
        );

        doc.settings.background_color = Color8::from(Color16::new(
            ((self.header.background_color >> 16) & 0xFFFF) as u16,
            ((self.header.background_color) & 0xFFFF) as u16,
            ((self.header.background_color >> 24) & 0xFFFF) as u16,
            65535,
        ));

        for cel_layer in &self.layers {
            let mut layer = RasterLayer::new(&cel_layer.name);
            layer.base.opacity = cel_layer.opacity as f64 / 255.0;
            layer.base.visible = cel_layer.visible;
            layer.base.locked = cel_layer.locked;
            
            let layer = Layer::Raster(layer);
            doc.add_layer(layer);
        }

        doc.timeline.frame_rate = 24.0;
        doc.timeline.end_frame = self.header.frame_count;

        doc
    }
}

impl FileReader for CelFile {
    fn read<R: Read + Seek>(reader: &mut R) -> Result<Self, IoError> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        
        if magic != CEL_MAGIC && magic != *b"CEL1" {
            return Err(IoError::InvalidMagic {
                expected: CEL_MAGIC,
                actual: magic,
            });
        }

        let version = read_u16_le(reader)?;
        let flags = read_u16_le(reader)?;
        let width = read_u32_le(reader)?;
        let height = read_u32_le(reader)?;
        let bits_per_pixel = read_u16_le(reader)?;
        let color_depth = read_u16_le(reader)?;
        let frame_count = read_u32_le(reader)?;
        let layer_count = read_u32_le(reader)?;
        let palette_offset = read_u32_le(reader)?;
        let layer_table_offset = read_u32_le(reader)?;
        let frame_table_offset = read_u32_le(reader)?;
        let image_data_offset = read_u32_le(reader)?;
        let dpi_x = read_f32_le(reader)?;
        let dpi_y = read_f32_le(reader)?;
        let background_color = read_u32_le(reader)?;
        let name = read_string(reader, 64)?;

        let header = CelHeader {
            version,
            flags,
            width,
            height,
            bits_per_pixel,
            color_depth,
            frame_count,
            layer_count,
            palette_offset,
            layer_table_offset,
            frame_table_offset,
            image_data_offset,
            dpi_x,
            dpi_y,
            background_color,
            name,
        };

        reader.seek(std::io::SeekFrom::Start(layer_table_offset as u64))?;
        let mut layers = Vec::with_capacity(layer_count as usize);
        for _ in 0..layer_count {
            let id = read_u32_le(reader)?;
            let name = read_string(reader, 32)?;
            let layer_type = read_u16_le(reader)?;
            let blend_mode = read_u16_le(reader)?;
            let opacity = read_u8(reader)?;
            let visible = read_u8(reader)? != 0;
            let locked = read_u8(reader)? != 0;
            let _reserved = read_u8(reader)?;
            let first_frame = read_u32_le(reader)?;
            let frame_count = read_u32_le(reader)?;

            layers.push(CelLayer {
                id,
                name,
                layer_type,
                blend_mode,
                opacity,
                visible,
                locked,
                first_frame,
                frame_count,
            });
        }

        reader.seek(std::io::SeekFrom::Start(frame_table_offset as u64))?;
        let mut frames = Vec::with_capacity(frame_count as usize);
        for _ in 0..frame_count {
            let frame_number = read_u32_le(reader)?;
            let layer_id = read_u32_le(reader)?;
            let data_offset = read_u32_le(reader)?;
            let data_size = read_u32_le(reader)?;
            let uncompressed_size = read_u32_le(reader)?;
            let compression = read_u16_le(reader)?;
            let image_format = read_u16_le(reader)?;
            let offset_x = read_i32_le(reader)?;
            let offset_y = read_i32_le(reader)?;
            let width = read_u32_le(reader)?;
            let height = read_u32_le(reader)?;
            let mut _reserved = [0u8; 8];
            reader.read_exact(&mut _reserved)?;

            frames.push(CelFrame {
                frame_number,
                layer_id,
                data_offset,
                data_size,
                uncompressed_size,
                compression,
                image_format,
                offset_x,
                offset_y,
                width,
                height,
            });
        }

        let palette = if palette_offset > 0 {
            reader.seek(std::io::SeekFrom::Start(palette_offset as u64))?;
            let color_count = read_u32_le(reader)?;
            let mut palette = Vec::with_capacity(color_count as usize);
            for _ in 0..color_count {
                let r = read_u16_le(reader)?;
                let g = read_u16_le(reader)?;
                let b = read_u16_le(reader)?;
                let a = read_u16_le(reader)?;
                palette.push(Color16::new(r, g, b, a));
            }
            palette
        } else {
            Vec::new()
        };

        reader.seek(std::io::SeekFrom::Start(image_data_offset as u64))?;
        let mut image_data = Vec::new();
        reader.read_to_end(&mut image_data)?;

        Ok(CelFile {
            header,
            layers,
            frames,
            palette,
            image_data,
        })
    }
}

fn read_u8<R: Read>(reader: &mut R) -> Result<u8, IoError> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn read_u16_le<R: Read>(reader: &mut R) -> Result<u16, IoError> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

fn read_u32_le<R: Read>(reader: &mut R) -> Result<u32, IoError> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn read_i32_le<R: Read>(reader: &mut R) -> Result<i32, IoError> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(i32::from_le_bytes(buf))
}

fn read_f32_le<R: Read>(reader: &mut R) -> Result<f32, IoError> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(f32::from_le_bytes(buf))
}

fn read_string<R: Read>(reader: &mut R, len: usize) -> Result<String, IoError> {
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    let end = buf.iter().position(|&b| b == 0).unwrap_or(len);
    String::from_utf8(buf[..end].to_vec())
        .map_err(|e| IoError::InvalidFormat(format!("Invalid UTF-8: {}", e)))
}
