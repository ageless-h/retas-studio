use std::io::{Write, Seek, SeekFrom};
use std::path::Path;
use retas_core::{Document, Layer, RasterLayer, LayerId};
use crate::{IoError, FileWriter};
use flate2::write::ZlibEncoder;
use flate2::Compression;

pub struct CelWriter {
    document: Document,
}

impl CelWriter {
    pub fn new(document: Document) -> Self {
        Self { document }
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), IoError> {
        self.write_file(path)
    }
}

impl FileWriter for CelWriter {
    fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<(), IoError> {
        // Magic number "CEL\0"
        write_u32_le(writer, 0x004C4543)?;

        // Version (1.0)
        write_u16_le(writer, 0x0100)?;
        
        // Flags
        write_u16_le(writer, 0)?;

        // Width and height
        write_u32_le(writer, self.document.settings.resolution.width as u32)?;
        write_u32_le(writer, self.document.settings.resolution.height as u32)?;
        
        // Bits per pixel (32 for RGBA)
        write_u16_le(writer, 32)?;
        
        // Color depth (1 = RGBA)
        write_u16_le(writer, 1)?;
        
        // Frame count
        write_u32_le(writer, self.document.timeline.end_frame)?;
        
        // Layer count
        write_u32_le(writer, self.document.layers.len() as u32)?;

        // Calculate offsets (header is 128 bytes)
        let layer_table_offset = 128u32;
        let layer_table_size = self.document.layers.len() as u32 * 48; // Each layer entry is 48 bytes
        let frame_table_offset = layer_table_offset + layer_table_size;
        
        // Calculate total frames (sum of all layers' frames)
        let total_frame_entries: u32 = self.document.layers.len() as u32 * self.document.timeline.end_frame;
        let frame_table_size = total_frame_entries * 40; // Each frame entry is 40 bytes
        let image_data_offset = frame_table_offset + frame_table_size;
        
        // Palette offset (0 = no palette for RGBA mode)
        write_u32_le(writer, 0)?;
        
        // Layer table offset
        write_u32_le(writer, layer_table_offset)?;
        
        // Frame table offset
        write_u32_le(writer, frame_table_offset)?;
        
        // Image data offset
        write_u32_le(writer, image_data_offset)?;

        // DPI (72.0)
        write_f32_le(writer, 72.0)?;
        write_f32_le(writer, 72.0)?;

        // Background color (ARGB)
        let bg_color = &self.document.settings.background_color;
        let bg_value = ((bg_color.a as u32) << 24) 
            | ((bg_color.r as u32) << 16) 
            | ((bg_color.g as u32) << 8) 
            | (bg_color.b as u32);
        write_u32_le(writer, bg_value)?;

        // Document name (64 bytes)
        write_string(writer, &self.document.settings.name, 64)?;

        // Pad header to 128 bytes
        let current_pos = writer.stream_position()?;
        for _ in current_pos..128 {
            write_u8(writer, 0)?;
        }

        // Write layer table
        let mut layer_id_map: std::collections::HashMap<LayerId, u32> = std::collections::HashMap::new();
        let mut layer_index = 0u32;
        
        for (id, layer) in &self.document.layers {
            layer_id_map.insert(*id, layer_index);
            
            // Layer ID
            write_u32_le(writer, layer_index)?;
            
            // Layer name (32 bytes)
            write_string(writer, layer.name(), 32)?;
            
            // Layer type (0 = raster, 1 = vector)
            let layer_type = match layer {
                Layer::Raster(_) => 0u16,
                Layer::Vector(_) => 1u16,
                _ => 0u16, // Other types default to raster
            };
            write_u16_le(writer, layer_type)?;
            
            // Blend mode (0 = normal)
            write_u16_le(writer, 0)?;
            
            // Opacity (0-255)
            write_u8(writer, 255)?;
            
            // Visible
            write_u8(writer, if layer.base().visible { 1 } else { 0 })?;
            
            // Locked
            write_u8(writer, if layer.base().locked { 1 } else { 0 })?;
            
            // Reserved
            write_u8(writer, 0)?;
            
            // First frame
            write_u32_le(writer, 0)?;
            
            // Frame count
            write_u32_le(writer, self.document.timeline.end_frame)?;
            
            layer_index += 1;
        }

        // Write frame table and image data
        let mut current_image_offset = image_data_offset;
        let width = self.document.settings.resolution.width as u32;
        let height = self.document.settings.resolution.height as u32;
        let pixel_count = (width * height * 4) as usize;
        
        // Collect image data to write later
        let mut image_data_to_write: Vec<(u32, Vec<u8>)> = Vec::new();
        
        for layer_id in &self.document.timeline.layer_order {
            let layer_internal_id = layer_id_map.get(layer_id).copied().unwrap_or(0);
            
            for frame_num in 0..self.document.timeline.end_frame {
                // Frame number (1-based)
                write_u32_le(writer, frame_num + 1)?;
                
                // Layer ID
                write_u32_le(writer, layer_internal_id)?;
                
                // Data offset (will be the same for now, storing actual offset)
                write_u32_le(writer, current_image_offset)?;
                
                // Get layer pixel data for current frame
                let pixel_data = if let Some(layer) = self.document.layers.get(layer_id) {
                    match layer {
                        Layer::Raster(raster) => {
                            // Get pixel data from raster layer for current frame
                            if let Some(frame) = raster.frames.get(&frame_num) {
                                if frame.image_data.is_empty() {
                                    vec![255u8; pixel_count]
                                } else {
                                    (*frame.image_data).clone()
                                }
                            } else {
                                vec![255u8; pixel_count]
                            }
                        }
                        _ => {
                            vec![255u8; pixel_count]
                        }
                    }
                } else {
                    vec![255u8; pixel_count]
                };
                
                // Compress pixel data
                let compressed = compress_pixels(&pixel_data)?;
                let data_size = compressed.len() as u32;
                let uncompressed_size = pixel_data.len() as u32;
                
                // Data size (compressed)
                write_u32_le(writer, data_size)?;
                
                // Uncompressed size
                write_u32_le(writer, uncompressed_size)?;
                
                // Compression (1 = zlib)
                write_u16_le(writer, 1)?;
                
                // Image format (0 = raw RGBA)
                write_u16_le(writer, 0)?;
                
                // Offset X
                write_i32_le(writer, 0)?;
                
                // Offset Y
                write_i32_le(writer, 0)?;
                
                // Width
                write_u32_le(writer, width)?;
                
                // Height
                write_u32_le(writer, height)?;
                
                // Reserved (8 bytes)
                for _ in 0..8 {
                    write_u8(writer, 0)?;
                }
                
                // Store image data to write later
                image_data_to_write.push((current_image_offset, compressed));
                current_image_offset += data_size;
            }
        }
        
        // Write image data
        for (_offset, data) in image_data_to_write {
            writer.write_all(&data)?;
        }

        Ok(())
    }
}

fn compress_pixels(pixels: &[u8]) -> Result<Vec<u8>, IoError> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(pixels)?;
    encoder.finish().map_err(|e| IoError::InvalidFormat(format!("Compression error: {}", e)))
}

fn write_u8<W: Write>(writer: &mut W, value: u8) -> Result<(), IoError> {
    writer.write_all(&[value])?;
    Ok(())
}

fn write_u16_le<W: Write>(writer: &mut W, value: u16) -> Result<(), IoError> {
    writer.write_all(&value.to_le_bytes())?;
    Ok(())
}

fn write_u32_le<W: Write>(writer: &mut W, value: u32) -> Result<(), IoError> {
    writer.write_all(&value.to_le_bytes())?;
    Ok(())
}

fn write_i32_le<W: Write>(writer: &mut W, value: i32) -> Result<(), IoError> {
    writer.write_all(&value.to_le_bytes())?;
    Ok(())
}

fn write_f32_le<W: Write>(writer: &mut W, value: f32) -> Result<(), IoError> {
    writer.write_all(&value.to_le_bytes())?;
    Ok(())
}

fn write_string<W: Write>(writer: &mut W, s: &str, len: usize) -> Result<(), IoError> {
    let bytes = s.as_bytes();
    writer.write_all(&bytes[..bytes.len().min(len)])?;
    for _ in bytes.len()..len {
        write_u8(writer, 0)?;
    }
    Ok(())
}
