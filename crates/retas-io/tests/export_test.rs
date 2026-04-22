#[cfg(test)]
mod tests {
    use retas_io::export::*;
    use std::path::PathBuf;

    #[test]
    fn test_image_format_extension() {
        assert_eq!(ImageFormat::Png.extension(), "png");
        assert_eq!(ImageFormat::Jpeg.extension(), "jpg");
        assert_eq!(ImageFormat::Gif.extension(), "gif");
        assert_eq!(ImageFormat::WebP.extension(), "webp");
    }

    #[test]
    fn test_image_format_from_extension() {
        assert_eq!(ImageFormat::from_extension("png"), Some(ImageFormat::Png));
        assert_eq!(ImageFormat::from_extension("jpg"), Some(ImageFormat::Jpeg));
        assert_eq!(ImageFormat::from_extension("jpeg"), Some(ImageFormat::Jpeg));
        assert_eq!(ImageFormat::from_extension("bmp"), Some(ImageFormat::Bmp));
        assert_eq!(ImageFormat::from_extension("webp"), Some(ImageFormat::WebP));
        assert_eq!(ImageFormat::from_extension("unknown"), None);
    }

    #[test]
    fn test_video_format_extension() {
        assert_eq!(VideoFormat::Mp4.extension(), "mp4");
        assert_eq!(VideoFormat::WebM.extension(), "webm");
        assert_eq!(VideoFormat::GifAnimation.extension(), "gif");
    }

    #[test]
    fn test_image_export_options_default() {
        let opts = ImageExportOptions::default();
        assert_eq!(opts.format, ImageFormat::Png);
        assert_eq!(opts.width, None);
        assert_eq!(opts.height, None);
        assert_eq!(opts.quality, 95);
        assert_eq!(opts.background, None);
    }

    #[test]
    fn test_image_export_options_builder() {
        let opts = ImageExportOptions::new(ImageFormat::Jpeg)
            .with_size(1920, 1080)
            .with_quality(85);
        
        assert_eq!(opts.format, ImageFormat::Jpeg);
        assert_eq!(opts.width, Some(1920));
        assert_eq!(opts.height, Some(1080));
        assert_eq!(opts.quality, 85);
    }

    #[test]
    fn test_video_export_options_default() {
        let opts = VideoExportOptions::default();
        assert_eq!(opts.format, VideoFormat::Mp4);
        assert_eq!(opts.width, 1920);
        assert_eq!(opts.height, 1080);
        assert_eq!(opts.frame_rate, 24.0);
        assert_eq!(opts.quality, 80);
    }

    #[test]
    fn test_video_export_options_builder() {
        let opts = VideoExportOptions::new(VideoFormat::GifAnimation, 800, 600, 30.0)
            .with_range(0, 100)
            .with_quality(90);
        
        assert_eq!(opts.format, VideoFormat::GifAnimation);
        assert_eq!(opts.width, 800);
        assert_eq!(opts.height, 600);
        assert_eq!(opts.frame_rate, 30.0);
        assert_eq!(opts.start_frame, 0);
        assert_eq!(opts.end_frame, 100);
        assert_eq!(opts.quality, 90);
    }

    #[test]
    fn test_swf_export_options_default() {
        let opts = SwfExportOptions::default();
        assert_eq!(opts.width, 800);
        assert_eq!(opts.height, 600);
        assert_eq!(opts.frame_rate, 24.0);
        assert_eq!(opts.compress, true);
        assert_eq!(opts.version, 10);
    }

    #[test]
    fn test_export_error_display() {
        let err = ExportError::UnsupportedFormat("test".to_string());
        assert_eq!(err.to_string(), "Unsupported format: test");
        
        let err2 = ExportError::InvalidDocument("bad doc".to_string());
        assert_eq!(err2.to_string(), "Invalid document: bad doc");
    }

    #[test]
    fn test_export_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let export_err: ExportError = io_err.into();
        assert!(matches!(export_err, ExportError::Io(_)));
    }
}
