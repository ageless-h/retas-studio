use std::io::{Read, Seek};
use std::path::Path;
use retas_core::{Point, Color8};
use crate::{IoError, FileReader};

const DGA_MAGIC: [u8; 4] = *b"DGA\x00";

#[derive(Debug, Clone)]
pub struct DgaHeader {
    pub version: u16,
    pub flags: u16,
    pub stroke_count: u32,
    pub total_points: u32,
    pub stroke_table_offset: u32,
    pub point_data_offset: u32,
    pub width: f32,
    pub height: f32,
    pub dpi: f32,
    pub background_color: u32,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct DgaStroke {
    pub id: u32,
    pub point_count: u32,
    pub first_point_index: u32,
    pub color: Color8,
    pub line_width: f32,
    pub line_cap: u8,
    pub line_join: u8,
    pub pressure_sensitive: bool,
    pub opacity: f32,
    pub smoothing: f32,
}

#[derive(Debug, Clone)]
pub struct DgaPoint {
    pub position: Point,
    pub pressure: f32,
    pub tilt_x: f32,
    pub tilt_y: f32,
    pub timestamp: f32,
}

#[derive(Debug, Clone)]
pub struct DgaFile {
    pub header: DgaHeader,
    pub strokes: Vec<DgaStroke>,
    pub points: Vec<DgaPoint>,
}

impl DgaFile {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, IoError> {
        Self::read_file(path)
    }
}

impl FileReader for DgaFile {
    fn read<R: Read + Seek>(reader: &mut R) -> Result<Self, IoError> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        
        if magic != DGA_MAGIC {
            return Err(IoError::InvalidMagic {
                expected: DGA_MAGIC,
                actual: magic,
            });
        }

        let version = read_u16_le(reader)?;
        let flags = read_u16_le(reader)?;
        let stroke_count = read_u32_le(reader)?;
        let total_points = read_u32_le(reader)?;
        let stroke_table_offset = read_u32_le(reader)?;
        let point_data_offset = read_u32_le(reader)?;
        let width = read_f32_le(reader)?;
        let height = read_f32_le(reader)?;
        let dpi = read_f32_le(reader)?;
        let background_color = read_u32_le(reader)?;
        let name = read_string(reader, 32)?;

        let header = DgaHeader {
            version,
            flags,
            stroke_count,
            total_points,
            stroke_table_offset,
            point_data_offset,
            width,
            height,
            dpi,
            background_color,
            name,
        };

        reader.seek(std::io::SeekFrom::Start(stroke_table_offset as u64))?;
        let mut strokes = Vec::with_capacity(stroke_count as usize);
        for _ in 0..stroke_count {
            let id = read_u32_le(reader)?;
            let point_count = read_u32_le(reader)?;
            let first_point_index = read_u32_le(reader)?;
            let color_value = read_u32_le(reader)?;
            let color = Color8::new(
                ((color_value >> 16) & 0xFF) as u8,
                ((color_value >> 8) & 0xFF) as u8,
                (color_value & 0xFF) as u8,
                ((color_value >> 24) & 0xFF) as u8,
            );
            let line_width = read_f32_le(reader)?;
            let line_cap = read_u8(reader)?;
            let line_join = read_u8(reader)?;
            let pressure_sensitive = read_u8(reader)? != 0;
            let _reserved = read_u8(reader)?;
            let opacity = read_f32_le(reader)?;
            let smoothing = read_f32_le(reader)?;
            let mut _reserved2 = [0u8; 8];
            reader.read_exact(&mut _reserved2)?;

            strokes.push(DgaStroke {
                id,
                point_count,
                first_point_index,
                color,
                line_width,
                line_cap,
                line_join,
                pressure_sensitive,
                opacity,
                smoothing,
            });
        }

        reader.seek(std::io::SeekFrom::Start(point_data_offset as u64))?;
        let mut points = Vec::with_capacity(total_points as usize);
        for _ in 0..total_points {
            let x = read_f32_le(reader)?;
            let y = read_f32_le(reader)?;
            let pressure = read_f32_le(reader)?;
            let tilt_x = read_f32_le(reader)?;
            let tilt_y = read_f32_le(reader)?;
            let timestamp = read_f32_le(reader)?;

            points.push(DgaPoint {
                position: Point::new(x as f64, y as f64),
                pressure,
                tilt_x,
                tilt_y,
                timestamp,
            });
        }

        Ok(DgaFile {
            header,
            strokes,
            points,
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
