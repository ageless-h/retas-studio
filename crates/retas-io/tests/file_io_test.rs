#[cfg(test)]
mod tests {
    use retas_core::{Document, RasterLayer, Layer};
    use retas_io::{
        CelFile, CelWriter,
        DgaFile,
        ScsFile,
        IoError,
        FileReader,
        ImageExporter, ImageFormat, ImageExportOptions,
    };
    use std::io::Cursor;

    fn create_test_document() -> Document {
        let mut doc = Document::new("TestDoc", 64.0, 64.0);
        let mut layer = RasterLayer::new("Layer1");
        layer.base.opacity = 1.0;

        let frame = retas_core::RasterFrame {
            frame_number: 0,
            image_data: vec![255u8; 64 * 64 * 4],
            width: 64,
            height: 64,
            bounds: None,
        };
        layer.frames.insert(0, frame);

        doc.add_layer(Layer::Raster(layer));
        doc.timeline.end_frame = 1;
        doc
    }

    #[test]
    fn test_cel_write_and_read_roundtrip() {
        let doc = create_test_document();
        let mut buffer = Vec::new();
        
        {
            let writer = CelWriter::new(doc.clone());
            let mut cursor = Cursor::new(&mut buffer);
            use retas_io::FileWriter;
            writer.write(&mut cursor).unwrap();
        }

        let mut cursor = Cursor::new(&buffer);
        let cel = CelFile::read(&mut cursor).unwrap();

        assert_eq!(cel.header.width, 64);
        assert_eq!(cel.header.height, 64);
        assert_eq!(cel.header.bits_per_pixel, 32);
        assert_eq!(cel.header.frame_count, 1);
        assert_eq!(cel.header.layer_count, 1);
        assert_eq!(cel.header.name, "TestDoc");
    }

    #[test]
    fn test_cel_invalid_magic() {
        let data = b"INVALID";
        let mut cursor = Cursor::new(data);
        let result = CelFile::read(&mut cursor);
        assert!(matches!(result, Err(IoError::InvalidMagic { .. })));
    }

    #[test]
    fn test_cel_file_open_nonexistent() {
        let result = CelFile::open("/nonexistent/path/file.cel");
        assert!(result.is_err());
    }

    #[test]
    fn test_dga_invalid_magic() {
        let data = b"NOTDGA";
        let mut cursor = Cursor::new(data);
        let result = DgaFile::read(&mut cursor);
        assert!(matches!(result, Err(IoError::InvalidMagic { .. })));
    }

    #[test]
    fn test_dga_file_open_nonexistent() {
        let result = DgaFile::open("/nonexistent/path/file.dga");
        assert!(result.is_err());
    }

    #[test]
    fn test_scs_invalid_magic() {
        let data = b"NOTSCS";
        let mut cursor = Cursor::new(data);
        let result = ScsFile::read(&mut cursor);
        assert!(matches!(result, Err(IoError::InvalidMagic { .. })));
    }

    #[test]
    fn test_scs_file_open_nonexistent() {
        let result = ScsFile::open("/nonexistent/path/file.scs");
        assert!(result.is_err());
    }

    #[test]
    fn test_image_export_png_to_file() {
        let pixels = vec![255u8; 4 * 4 * 4];
        let temp_path = std::env::temp_dir().join("retas_test.png");
        
        ImageExporter::export_pixels(&pixels, 4, 4, &temp_path).unwrap();
        
        let contents = std::fs::read(&temp_path).unwrap();
        assert_eq!(&contents[0..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
        
        std::fs::remove_file(&temp_path).ok();
    }

    #[test]
    fn test_image_export_options_builder() {
        let options = ImageExportOptions::new(ImageFormat::Png)
            .with_size(1920, 1080)
            .with_quality(90)
            .with_background(retas_core::Color8::BLACK);
        
        assert_eq!(options.format, ImageFormat::Png);
        assert_eq!(options.width, Some(1920));
        assert_eq!(options.height, Some(1080));
        assert_eq!(options.quality, 90);
        assert_eq!(options.background, Some(retas_core::Color8::BLACK));
    }

    #[test]
    fn test_image_format_extensions() {
        assert_eq!(ImageFormat::Png.extension(), "png");
        assert_eq!(ImageFormat::Jpeg.extension(), "jpg");
        assert_eq!(ImageFormat::Gif.extension(), "gif");
        assert_eq!(ImageFormat::Bmp.extension(), "bmp");
        assert_eq!(ImageFormat::Tiff.extension(), "tiff");
    }

    #[test]
    fn test_image_format_from_extension() {
        assert_eq!(ImageFormat::from_extension("png"), Some(ImageFormat::Png));
        assert_eq!(ImageFormat::from_extension("jpg"), Some(ImageFormat::Jpeg));
        assert_eq!(ImageFormat::from_extension("jpeg"), Some(ImageFormat::Jpeg));
        assert_eq!(ImageFormat::from_extension("gif"), Some(ImageFormat::Gif));
        assert_eq!(ImageFormat::from_extension("bmp"), Some(ImageFormat::Bmp));
        assert_eq!(ImageFormat::from_extension("tiff"), Some(ImageFormat::Tiff));
        assert_eq!(ImageFormat::from_extension("unknown"), None);
    }

    #[test]
    fn test_cel_header_fields() {
        let doc = create_test_document();
        let mut buffer = Vec::new();
        
        {
            let writer = CelWriter::new(doc);
            let mut cursor = Cursor::new(&mut buffer);
            use retas_io::FileWriter;
            writer.write(&mut cursor).unwrap();
        }

        let mut cursor = Cursor::new(&buffer);
        let cel = CelFile::read(&mut cursor).unwrap();

        assert!(cel.header.version > 0);
        assert_eq!(cel.header.color_depth, 1);
        assert_eq!(cel.layers.len(), 1);
        assert_eq!(cel.frames.len(), 1);
        
        let layer = &cel.layers[0];
        assert_eq!(layer.name, "Layer1");
        assert!(layer.visible);
        assert_eq!(layer.blend_mode, 0);
    }

    #[test]
    fn test_document_to_cel_layer_mapping() {
        let mut doc = Document::new("MultiLayer", 32.0, 32.0);
        doc.timeline.end_frame = 1;
        
        let mut layer1 = RasterLayer::new("Background");
        layer1.frames.insert(0, retas_core::RasterFrame {
            frame_number: 0,
            image_data: vec![255u8; 32 * 32 * 4],
            width: 32,
            height: 32,
            bounds: None,
        });
        doc.add_layer(Layer::Raster(layer1));
        
        let mut layer2 = RasterLayer::new("Foreground");
        layer2.base.blend_mode = retas_core::BlendMode::Multiply;
        layer2.frames.insert(0, retas_core::RasterFrame {
            frame_number: 0,
            image_data: vec![128u8; 32 * 32 * 4],
            width: 32,
            height: 32,
            bounds: None,
        });
        doc.add_layer(Layer::Raster(layer2));
        
        let mut buffer = Vec::new();
        {
            let writer = CelWriter::new(doc);
            let mut cursor = Cursor::new(&mut buffer);
            use retas_io::FileWriter;
            writer.write(&mut cursor).unwrap();
        }

        let mut cursor = Cursor::new(&buffer);
        let cel = CelFile::read(&mut cursor).unwrap();

        assert_eq!(cel.header.layer_count, 2);
        assert_eq!(cel.layers.len(), 2);
        assert_eq!(cel.frames.len(), 1);
    }
}
