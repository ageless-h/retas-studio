use std::io::{Read, Seek};
use std::path::Path;
use retas_core::Size;
use crate::{IoError, FileReader};

const SCS_MAGIC: [u8; 4] = *b"SCS\x00";

#[derive(Debug, Clone)]
pub struct ScsHeader {
    pub version: u16,
    pub flags: u16,
    pub layer_count: u32,
    pub total_frames: u32,
    pub frame_rate: f32,
    pub resolution_x: u32,
    pub resolution_y: u32,
    pub aspect_ratio: f32,
    pub layer_table_offset: u32,
    pub timeline_offset: u32,
    pub effect_offset: u32,
    pub camera_offset: u32,
    pub safe_area: (f32, f32, f32, f32),
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct ScsLayer {
    pub id: u32,
    pub name: String,
    pub layer_type: u8,
    pub blend_mode: u8,
    pub opacity: f32,
    pub visible: bool,
    pub locked: bool,
    pub parent_id: Option<u32>,
    pub transform: [f32; 6],
    pub source_path: String,
    pub start_frame: u32,
    pub end_frame: u32,
}

#[derive(Debug, Clone)]
pub struct ScsCamera {
    pub id: u32,
    pub name: String,
    pub position: (f32, f32),
    pub zoom: f32,
    pub rotation: f32,
    pub resolution: (u32, u32),
}

#[derive(Debug, Clone)]
pub struct ScsFile {
    pub header: ScsHeader,
    pub layers: Vec<ScsLayer>,
    pub cameras: Vec<ScsCamera>,
}

impl ScsFile {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, IoError> {
        Self::read_file(path)
    }
}

impl FileReader for ScsFile {
    fn read<R: Read + Seek>(reader: &mut R) -> Result<Self, IoError> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        
        if magic != SCS_MAGIC {
            return Err(IoError::InvalidMagic {
                expected: SCS_MAGIC,
                actual: magic,
            });
        }

        let version = read_u16_le(reader)?;
        let flags = read_u16_le(reader)?;
        let layer_count = read_u32_le(reader)?;
        let total_frames = read_u32_le(reader)?;
        let frame_rate = read_f32_le(reader)?;
        let resolution_x = read_u32_le(reader)?;
        let resolution_y = read_u32_le(reader)?;
        let aspect_ratio = read_f32_le(reader)?;
        let layer_table_offset = read_u32_le(reader)?;
        let timeline_offset = read_u32_le(reader)?;
        let effect_offset = read_u32_le(reader)?;
        let camera_offset = read_u32_le(reader)?;
        let safe_left = read_f32_le(reader)?;
        let safe_top = read_f32_le(reader)?;
        let safe_right = read_f32_le(reader)?;
        let safe_bottom = read_f32_le(reader)?;
        let name = read_string(reader, 128)?;
        
        let mut _reserved = [0u8; 128];
        reader.read_exact(&mut _reserved)?;

        let header = ScsHeader {
            version,
            flags,
            layer_count,
            total_frames,
            frame_rate,
            resolution_x,
            resolution_y,
            aspect_ratio,
            layer_table_offset,
            timeline_offset,
            effect_offset,
            camera_offset,
            safe_area: (safe_left, safe_top, safe_right, safe_bottom),
            name,
        };

        reader.seek(std::io::SeekFrom::Start(layer_table_offset as u64))?;
        let mut layers = Vec::with_capacity(layer_count as usize);
        for _ in 0..layer_count {
            let id = read_u32_le(reader)?;
            let name = read_string(reader, 64)?;
            let layer_type = read_u8(reader)?;
            let blend_mode = read_u8(reader)?;
            let _reserved = read_u8(reader)?;
            let _reserved2 = read_u8(reader)?;
            let opacity = read_f32_le(reader)?;
            let visible = read_u8(reader)? != 0;
            let locked = read_u8(reader)? != 0;
            let _reserved3 = read_u8(reader)?;
            let _reserved4 = read_u8(reader)?;
            
            let parent_id_raw = read_u32_le(reader)?;
            let parent_id = if parent_id_raw == 0xFFFFFFFF { None } else { Some(parent_id_raw) };
            
            let mut transform = [0f32; 6];
            for i in 0..6 {
                transform[i] = read_f32_le(reader)?;
            }
            
            let source_path = read_string(reader, 256)?;
            let start_frame = read_u32_le(reader)?;
            let end_frame = read_u32_le(reader)?;

            layers.push(ScsLayer {
                id,
                name,
                layer_type,
                blend_mode,
                opacity,
                visible,
                locked,
                parent_id,
                transform,
                source_path,
                start_frame,
                end_frame,
            });
        }

        let cameras = if camera_offset > 0 {
            reader.seek(std::io::SeekFrom::Start(camera_offset as u64))?;
            let camera_count = read_u32_le(reader)?;
            let mut cameras = Vec::with_capacity(camera_count as usize);
            for _ in 0..camera_count {
                let id = read_u32_le(reader)?;
                let name = read_string(reader, 32)?;
                let pos_x = read_f32_le(reader)?;
                let pos_y = read_f32_le(reader)?;
                let zoom = read_f32_le(reader)?;
                let rotation = read_f32_le(reader)?;
                let res_x = read_u32_le(reader)?;
                let res_y = read_u32_le(reader)?;

                cameras.push(ScsCamera {
                    id,
                    name,
                    position: (pos_x, pos_y),
                    zoom,
                    rotation,
                    resolution: (res_x, res_y),
                });
            }
            cameras
        } else {
            Vec::new()
        };

        Ok(ScsFile {
            header,
            layers,
            cameras,
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
