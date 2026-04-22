#[cfg(test)]
mod tests {
    use retas_core::{Color8, Color16, ColorF};

    #[test]
    fn test_color8_creation() {
        let c = Color8::new(255, 128, 64, 255);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 64);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn test_color8_from_rgb() {
        let c = Color8::from_rgb(255, 128, 64);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 64);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn test_color8_from_hex() {
        let c = Color8::from_hex(0xFF8040);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 64);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn test_color8_to_rgba_f32() {
        let c = Color8::new(255, 128, 64, 255);
        let rgba = c.to_rgba_f32();
        assert!((rgba[0] - 1.0).abs() < 0.001);
        assert!((rgba[1] - 0.5).abs() < 0.01);
        assert!((rgba[2] - 0.25).abs() < 0.01);
        assert!((rgba[3] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_color8_to_color16() {
        let c8 = Color8::new(255, 128, 64, 255);
        let c16 = Color16::from(c8);
        assert_eq!(c16.r, 65535);
    }

    #[test]
    fn test_color16_to_color8() {
        let c16 = Color16::new(32768, 16384, 8192, 65535);
        let c8 = Color8::from(c16);
        assert_eq!(c8.r, 128);
        assert_eq!(c8.g, 64);
        assert_eq!(c8.b, 32);
    }

    #[test]
    fn test_colorf_from_hsv() {
        let c = ColorF::from_hsv(0.0, 1.0, 1.0);
        assert!((c.r - 1.0).abs() < 0.001);
        assert!(c.g.abs() < 0.001);
        assert!(c.b.abs() < 0.001);

        let c2 = ColorF::from_hsv(120.0, 1.0, 1.0);
        assert!(c2.r.abs() < 0.001);
        assert!((c2.g - 1.0).abs() < 0.001);
        assert!(c2.b.abs() < 0.001);
    }

    #[test]
    fn test_colorf_to_hsv() {
        let c = ColorF::new(1.0, 0.0, 0.0, 1.0);
        let (h, s, v) = c.to_hsv();
        assert!((h - 0.0).abs() < 0.001);
        assert!((s - 1.0).abs() < 0.001);
        assert!((v - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_colorf_lerp() {
        let c1 = ColorF::new(0.0, 0.0, 0.0, 1.0);
        let c2 = ColorF::new(1.0, 1.0, 1.0, 1.0);
        let mid = c1.lerp(&c2, 0.5);
        assert!((mid.r - 0.5).abs() < 0.001);
        assert!((mid.g - 0.5).abs() < 0.001);
        assert!((mid.b - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_color8_to_colorf() {
        let c8 = Color8::new(255, 128, 64, 255);
        let cf = ColorF::from(c8);
        assert!((cf.r - 1.0).abs() < 0.001);
        assert!((cf.g - 0.5).abs() < 0.01);
        assert!((cf.b - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_colorf_to_color8() {
        let cf = ColorF::new(1.0, 0.5, 0.25, 1.0);
        let c8 = Color8::from(cf);
        assert_eq!(c8.r, 255);
        assert_eq!(c8.g, 128);
        assert_eq!(c8.b, 64);
    }
}
