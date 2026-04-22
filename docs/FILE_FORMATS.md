# RETAS STUDIO File Format Specifications

## Overview

RETAS STUDIO uses several proprietary file formats for storing animation data:

| Extension | Name | Purpose | Module |
|-----------|------|---------|--------|
| `.cel` | Cel File | Raster animation frames | Stylos/PaintMan |
| `.dga` | DGA File | Vector drawing data | Stylos |
| `.scs` | Scene File | Scene composition | CoreRETAS |
| `.rsh` | Resource File | Resource references | All |
| `.pmt` | Palette File | Color palette | PaintMan |
| `.tmg` | Timing File | Timing/exposure sheet | All |

---

## 1. CEL File Format (.cel)

The CEL file is the primary raster animation format used by Stylos and PaintMan.

### File Structure

```
┌─────────────────────────────────┐
│         File Header             │  128 bytes
├─────────────────────────────────┤
│         Layer Table             │  Variable
├─────────────────────────────────┤
│         Frame Table             │  Variable
├─────────────────────────────────┤
│         Image Data              │  Variable
├─────────────────────────────────┤
│         Palette Data            │  Variable
└─────────────────────────────────┘
```

### Header Structure (128 bytes)

| Offset | Size | Type | Description |
|--------|------|------|-------------|
| 0x00 | 4 | char[4] | Magic: "CEL\x00" or "CEL1" |
| 0x04 | 2 | uint16 | Version (typically 0x0100 = 1.0) |
| 0x06 | 2 | uint16 | Flags |
| 0x08 | 4 | uint32 | Image width |
| 0x0C | 4 | uint32 | Image height |
| 0x10 | 2 | uint16 | Bits per pixel (8, 16, 24, 32) |
| 0x12 | 2 | uint16 | Color depth (indexed/RGB/RGBA) |
| 0x14 | 4 | uint32 | Frame count |
| 0x18 | 4 | uint32 | Layer count |
| 0x1C | 4 | uint32 | Palette offset |
| 0x20 | 4 | uint32 | Layer table offset |
| 0x24 | 4 | uint32 | Frame table offset |
| 0x28 | 4 | uint32 | Image data offset |
| 0x2C | 4 | float | DPI X |
| 0x30 | 4 | float | DPI Y |
| 0x34 | 4 | uint32 | Background color (ARGB) |
| 0x38 | 64 | char[64] | Document name (UTF-8) |
| 0x78 | 8 | reserved | Reserved for future use |

### Layer Entry (32 bytes each)

| Offset | Size | Type | Description |
|--------|------|------|-------------|
| 0x00 | 4 | uint32 | Layer ID |
| 0x04 | 32 | char[32] | Layer name |
| 0x24 | 2 | uint16 | Layer type (0=raster, 1=vector, 2=guide) |
| 0x26 | 2 | uint16 | Blend mode |
| 0x28 | 1 | uint8 | Opacity (0-255) |
| 0x29 | 1 | uint8 | Visible flag |
| 0x2A | 1 | uint8 | Locked flag |
| 0x2B | 1 | uint8 | Reserved |
| 0x2C | 4 | uint32 | First frame index |
| 0x30 | 4 | uint32 | Frame count |

### Frame Entry (48 bytes each)

| Offset | Size | Type | Description |
|--------|------|------|-------------|
| 0x00 | 4 | uint32 | Frame number |
| 0x04 | 4 | uint32 | Layer ID |
| 0x08 | 4 | uint32 | Data offset (relative to image data) |
| 0x0C | 4 | uint32 | Data size (compressed) |
| 0x10 | 4 | uint32 | Uncompressed size |
| 0x14 | 2 | uint16 | Compression type (0=none, 1=RLE, 2=ZLIB) |
| 0x16 | 2 | uint16 | Image format |
| 0x18 | 4 | int32 | Offset X (relative to document) |
| 0x1C | 4 | int32 | Offset Y |
| 0x20 | 4 | uint32 | Width |
| 0x24 | 4 | uint32 | Height |
| 0x28 | 8 | reserved | Reserved |

### Compression Types

| Value | Name | Description |
|-------|------|-------------|
| 0 | None | Uncompressed raw pixel data |
| 1 | RLE | Run-length encoding |
| 2 | ZLIB | zlib/DEFLATE compression |
| 3 | RLE+ZLIB | RLE followed by ZLIB |

### Blend Modes

| Value | Name | Description |
|-------|------|-------------|
| 0 | Normal | Standard alpha blending |
| 1 | Multiply | Darken by multiplying |
| 2 | Screen | Lighten by screening |
| 3 | Overlay | Combine multiply and screen |
| 4 | Darken | Keep darker pixels |
| 5 | Lighten | Keep lighter pixels |
| 6 | ColorDodge | Brighten to white |
| 7 | ColorBurn | Darken to black |
| 8 | HardLight | Like overlay, more contrast |
| 9 | SoftLight | Like overlay, less contrast |
| 10 | Difference | Absolute difference |
| 11 | Exclusion | Like difference, less contrast |

---

## 2. DGA File Format (.dga)

DGA (Drawing) files store vector drawing data with bezier curves.

### File Structure

```
┌─────────────────────────────────┐
│         File Header             │  64 bytes
├─────────────────────────────────┤
│         Stroke Table            │  Variable
├─────────────────────────────────┤
│         Point Data              │  Variable
├─────────────────────────────────┤
│         Attribute Data          │  Variable
└─────────────────────────────────┘
```

### Header Structure (64 bytes)

| Offset | Size | Type | Description |
|--------|------|------|-------------|
| 0x00 | 4 | char[4] | Magic: "DGA\x00" |
| 0x04 | 2 | uint16 | Version |
| 0x06 | 2 | uint16 | Flags |
| 0x08 | 4 | uint32 | Stroke count |
| 0x0C | 4 | uint32 | Total points |
| 0x10 | 4 | uint32 | Stroke table offset |
| 0x14 | 4 | uint32 | Point data offset |
| 0x18 | 4 | float | Document width |
| 0x1C | 4 | float | Document height |
| 0x20 | 4 | float | DPI |
| 0x24 | 4 | uint32 | Background color |
| 0x28 | 32 | char[32] | Document name |

### Stroke Entry (40 bytes each)

| Offset | Size | Type | Description |
|--------|------|------|-------------|
| 0x00 | 4 | uint32 | Stroke ID |
| 0x04 | 4 | uint32 | Point count |
| 0x08 | 4 | uint32 | First point index |
| 0x0C | 4 | uint32 | Color (ARGB) |
| 0x10 | 4 | float | Line width |
| 0x14 | 1 | uint8 | Line cap |
| 0x15 | 1 | uint8 | Line join |
| 0x16 | 1 | uint8 | Pressure sensitive |
| 0x17 | 1 | uint8 | Reserved |
| 0x18 | 4 | float | Opacity |
| 0x1C | 4 | float | Smoothing |
| 0x20 | 8 | reserved | Reserved |

### Point Entry (24 bytes each)

| Offset | Size | Type | Description |
|--------|------|------|-------------|
| 0x00 | 4 | float | X coordinate |
| 0x04 | 4 | float | Y coordinate |
| 0x08 | 4 | float | Pressure (0.0-1.0) |
| 0x0C | 4 | float | Tilt X |
| 0x10 | 4 | float | Tilt Y |
| 0x14 | 4 | float | Timestamp |

---

## 3. SCS File Format (.scs)

SCS (Scene) files store scene composition data for CoreRETAS.

### File Structure

```
┌─────────────────────────────────┐
│         File Header             │  256 bytes
├─────────────────────────────────┤
│         Layer Hierarchy         │  Variable
├─────────────────────────────────┤
│         Timeline Data           │  Variable
├─────────────────────────────────┤
│         Effect Stack            │  Variable
├─────────────────────────────────┤
│         Camera Data             │  Variable
└─────────────────────────────────┘
```

### Header Structure (256 bytes)

| Offset | Size | Type | Description |
|--------|------|------|-------------|
| 0x00 | 4 | char[4] | Magic: "SCS\x00" |
| 0x04 | 2 | uint16 | Version |
| 0x06 | 2 | uint16 | Flags |
| 0x08 | 4 | uint32 | Layer count |
| 0x0C | 4 | uint32 | Total frames |
| 0x10 | 4 | float | Frame rate |
| 0x14 | 4 | uint32 | Resolution X |
| 0x18 | 4 | uint32 | Resolution Y |
| 0x1C | 4 | float | Aspect ratio |
| 0x20 | 4 | uint32 | Layer table offset |
| 0x24 | 4 | uint32 | Timeline offset |
| 0x28 | 4 | uint32 | Effect offset |
| 0x2C | 4 | uint32 | Camera offset |
| 0x30 | 4 | float | Safe area left |
| 0x34 | 4 | float | Safe area top |
| 0x38 | 4 | float | Safe area right |
| 0x3C | 4 | float | Safe area bottom |
| 0x40 | 128 | char[128] | Scene name |
| 0xC0 | 128 | reserved | Reserved |

### Layer Types

| Value | Name | Description |
|-------|------|-------------|
| 0 | Raster | Bitmap image layer |
| 1 | Vector | Vector drawing layer |
| 2 | Camera | Camera/viewport layer |
| 3 | Text | Text layer |
| 4 | Shape | Shape layer |
| 5 | Guide | Guide layer |
| 6 | Sound | Audio layer |
| 7 | Adjustment | Adjustment layer |
| 8 | 3D | 3D layer (CoreRETAS) |

---

## 4. Palette File Format (.pmt)

PMT files store color palettes for PaintMan.

### Header Structure (32 bytes)

| Offset | Size | Type | Description |
|--------|------|------|-------------|
| 0x00 | 4 | char[4] | Magic: "PMT\x00" |
| 0x04 | 2 | uint16 | Version |
| 0x06 | 2 | uint16 | Color count |
| 0x08 | 4 | uint32 | Color data offset |
| 0x0C | 4 | uint32 | Group count |
| 0x10 | 4 | uint32 | Group data offset |
| 0x14 | 64 | char[64] | Palette name |
| 0x54 | 8 | reserved | Reserved |

### Color Entry (16 bytes each)

| Offset | Size | Type | Description |
|--------|------|------|-------------|
| 0x00 | 2 | uint16 | Red (16-bit) |
| 0x02 | 2 | uint16 | Green (16-bit) |
| 0x04 | 2 | uint16 | Blue (16-bit) |
| 0x06 | 2 | uint16 | Alpha (16-bit) |
| 0x08 | 4 | uint32 | Color ID |
| 0x0C | 4 | char[4] | Color name/code |

---

## 5. Timing File Format (.tmg)

TMG files store timing/exposure sheet data.

### Header Structure (64 bytes)

| Offset | Size | Type | Description |
|--------|------|------|-------------|
| 0x00 | 4 | char[4] | Magic: "TMG\x00" |
| 0x04 | 2 | uint16 | Version |
| 0x06 | 2 | uint16 | Column count |
| 0x08 | 4 | uint32 | Row count |
| 0x0C | 4 | uint32 | Data offset |
| 0x10 | 4 | float | Frame rate |
| 0x14 | 4 | uint32 | Start frame |
| 0x18 | 4 | uint32 | End frame |
| 0x1C | 48 | char[48] | Sheet name |

---

## Implementation Notes

### Reading CEL Files

```rust
pub struct CelReader {
    header: CelHeader,
    layers: Vec<LayerEntry>,
    frames: Vec<FrameEntry>,
    palette: Option<Vec<ColorEntry>>,
}

impl CelReader {
    pub fn open(path: &Path) -> Result<Self, Error> {
        let mut file = File::open(path)?;
        let header = Self::read_header(&mut file)?;
        let layers = Self::read_layers(&mut file, &header)?;
        let frames = Self::read_frames(&mut file, &header)?;
        let palette = Self::read_palette(&mut file, &header)?;
        
        Ok(Self { header, layers, frames, palette })
    }
    
    pub fn get_frame(&self, frame_num: u32, layer_id: u32) -> Option<&FrameEntry> {
        self.frames.iter()
            .find(|f| f.frame_number == frame_num && f.layer_id == layer_id)
    }
    
    pub fn decode_frame(&self, frame: &FrameEntry, data: &[u8]) -> Result<ImageBuffer, Error> {
        match frame.compression {
            0 => self.decode_raw(frame, data),
            1 => self.decode_rle(frame, data),
            2 => self.decode_zlib(frame, data),
            _ => Err(Error::UnknownCompression),
        }
    }
}
```

### Writing CEL Files

```rust
pub struct CelWriter {
    header: CelHeader,
    layers: Vec<LayerEntry>,
    frames: Vec<FrameEntry>,
}

impl CelWriter {
    pub fn new(width: u32, height: u32, bpp: u16) -> Self {
        Self {
            header: CelHeader::new(width, height, bpp),
            layers: Vec::new(),
            frames: Vec::new(),
        }
    }
    
    pub fn add_layer(&mut self, layer: LayerEntry) -> u32 {
        let id = self.layers.len() as u32;
        self.layers.push(layer);
        id
    }
    
    pub fn add_frame(&mut self, frame: FrameEntry, image: &ImageBuffer) -> Result<(), Error> {
        let compressed = self.compress_image(image)?;
        // ... store frame data
        Ok(())
    }
    
    pub fn save(&self, path: &Path) -> Result<(), Error> {
        let mut file = File::create(path)?;
        self.write_header(&mut file)?;
        self.write_layers(&mut file)?;
        self.write_frames(&mut file)?;
        Ok(())
    }
}
```

---

## Reverse Engineering Notes

### Key Findings from Binary Analysis

1. **RCVOffscreen** family handles all offscreen rendering
2. **CCScoreDocument** is the main document class
3. **CCLayer** hierarchy supports nested layers with transforms
4. **CCTool** uses command pattern via **DoItem**/**DoManager**
5. **RVBezier**/**RVVertex**/**RVLoop**/**RVFace** handle vector rendering
6. Color precision is 16-bit internally (Color16)

### Compression Algorithm (RLE)

The RLE compression used in CEL files:

```
For each scanline:
    while bytes_remaining > 0:
        if run_length >= 3:
            write 0x80 | run_length
            write pixel_value
        else:
            write literal_count
            write literal_pixels
```

### Endianness

All multi-byte values are **little-endian** (x86 native).

---

## References

- RETAS STUDIO 6.6.0 CHS binary analysis
- Class reference: `docs/CLASS_REFERENCE.md`
- Architecture: `docs/CROSS_PLATFORM_ARCHITECTURE.md`
