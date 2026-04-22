use std::io::{Read, Write, Seek};
use std::path::Path;
use crate::IoError;

pub trait FileReader: Sized {
    fn read<R: Read + Seek>(reader: &mut R) -> Result<Self, IoError>;
    
    fn read_file<P: AsRef<Path>>(path: P) -> Result<Self, IoError> {
        let mut file = std::fs::File::open(path)?;
        Self::read(&mut file)
    }
}

pub trait FileWriter {
    fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<(), IoError>;
    
    fn write_file<P: AsRef<Path>>(&self, path: P) -> Result<(), IoError> {
        let mut file = std::fs::File::create(path)?;
        self.write(&mut file)
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

fn read_f64_le<R: Read>(reader: &mut R) -> Result<f64, IoError> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(f64::from_le_bytes(buf))
}

fn read_string<R: Read>(reader: &mut R, len: usize) -> Result<String, IoError> {
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    let end = buf.iter().position(|&b| b == 0).unwrap_or(len);
    String::from_utf8(buf[..end].to_vec())
        .map_err(|e| IoError::InvalidFormat(format!("Invalid UTF-8: {}", e)))
}

fn read_magic<R: Read>(reader: &mut R, expected: &[u8; 4]) -> Result<(), IoError> {
    let mut actual = [0u8; 4];
    reader.read_exact(&mut actual)?;
    if &actual != expected {
        return Err(IoError::InvalidMagic {
            expected: *expected,
            actual,
        });
    }
    Ok(())
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

fn write_f64_le<W: Write>(writer: &mut W, value: f64) -> Result<(), IoError> {
    writer.write_all(&value.to_le_bytes())?;
    Ok(())
}

fn write_string<W: Write>(writer: &mut W, s: &str, len: usize) -> Result<(), IoError> {
    let bytes = s.as_bytes();
    writer.write_all(&bytes[..bytes.len().min(len)])?;
    for _ in bytes.len()..len {
        writer.write_all(&[0])?;
    }
    Ok(())
}
