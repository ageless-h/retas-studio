#[cfg(test)]
mod tests {
    use retas_core::{Point, Matrix2D, CameraKey};

    #[test]
    fn test_camera_identity_transform() {
        let camera = CameraKey::identity();
        let matrix = camera.to_matrix((1920.0, 1080.0));
        let p = Point::new(100.0, 200.0);
        let transformed = matrix.transform_point(&p);
        assert_eq!(transformed.x, 100.0);
        assert_eq!(transformed.y, 200.0);
    }

    #[test]
    fn test_camera_zoom_transform() {
        let camera = CameraKey::new(Point::ZERO, 2.0, 0.0);
        let matrix = camera.to_matrix((1920.0, 1080.0));
        let p = Point::new(100.0, 200.0);
        let transformed = matrix.transform_point(&p);
        assert_eq!(transformed.x, -860.0 * 2.0 + 960.0);
        assert_eq!(transformed.y, -340.0 * 2.0 + 540.0);
    }

    #[test]
    fn test_camera_translate_transform() {
        let camera = CameraKey::new(Point::new(50.0, -30.0), 1.0, 0.0);
        let matrix = camera.to_matrix((1920.0, 1080.0));
        let p = Point::new(100.0, 200.0);
        let transformed = matrix.transform_point(&p);
        assert_eq!(transformed.x, 150.0);
        assert_eq!(transformed.y, 170.0);
    }

    #[test]
    fn test_camera_zoom_and_translate() {
        let camera = CameraKey::new(Point::new(100.0, 50.0), 2.0, 0.0);
        let matrix = camera.to_matrix((1920.0, 1080.0));
        let p = Point::new(100.0, 100.0);
        let transformed = matrix.transform_point(&p);
        assert_eq!(transformed.x, -660.0);
        assert_eq!(transformed.y, -290.0);
    }

    #[test]
    fn test_camera_screen_to_canvas_roundtrip() {
        let camera = CameraKey::new(Point::new(100.0, 50.0), 2.0, 0.0);
        let matrix = camera.to_matrix((1920.0, 1080.0));
        let inverse = matrix.inverse().unwrap();

        let canvas_point = Point::new(500.0, 400.0);
        let screen_point = matrix.transform_point(&canvas_point);
        let back_to_canvas = inverse.transform_point(&screen_point);

        assert!((back_to_canvas.x - canvas_point.x).abs() < 0.0001);
        assert!((back_to_canvas.y - canvas_point.y).abs() < 0.0001);
    }

    #[test]
    fn test_camera_screen_to_canvas_at_different_zoom_levels() {
        let canvas_point = Point::new(960.0, 540.0);

        for zoom in [0.5, 1.0, 2.0, 4.0] {
            let camera = CameraKey::new(Point::ZERO, zoom, 0.0);
            let matrix = camera.to_matrix((1920.0, 1080.0));
            let inverse = matrix.inverse().unwrap();

            let screen = matrix.transform_point(&canvas_point);
            let back = inverse.transform_point(&screen);

            assert!((back.x - canvas_point.x).abs() < 0.0001, "Failed at zoom {}", zoom);
            assert!((back.y - canvas_point.y).abs() < 0.0001, "Failed at zoom {}", zoom);
        }
    }

    #[test]
    fn test_camera_screen_to_canvas_with_offset() {
        let camera = CameraKey::new(Point::new(200.0, 100.0), 1.5, 0.0);
        let matrix = camera.to_matrix((1920.0, 1080.0));
        let inverse = matrix.inverse().unwrap();

        let screen_point = Point::new(500.0, 400.0);
        let canvas_point = inverse.transform_point(&screen_point);
        let back_to_screen = matrix.transform_point(&canvas_point);

        assert!((back_to_screen.x - screen_point.x).abs() < 0.0001);
        assert!((back_to_screen.y - screen_point.y).abs() < 0.0001);
    }

    #[test]
    fn test_camera_rotate_90_degrees() {
        let camera = CameraKey::new(Point::ZERO, 1.0, std::f64::consts::PI / 2.0);
        let matrix = camera.to_matrix((1920.0, 1080.0));
        let p = Point::new(100.0, 0.0);
        let transformed = matrix.transform_point(&p);
        assert!((transformed.x - 1500.0).abs() < 0.0001);
        assert!((transformed.y - (-320.0)).abs() < 0.0001);
    }

    #[test]
    fn test_camera_rotate_and_zoom_roundtrip() {
        let camera = CameraKey::new(Point::new(50.0, 50.0), 2.0, std::f64::consts::PI / 4.0);
        let matrix = camera.to_matrix((1920.0, 1080.0));
        let inverse = matrix.inverse().unwrap();

        let canvas = Point::new(300.0, 200.0);
        let screen = matrix.transform_point(&canvas);
        let back = inverse.transform_point(&screen);

        assert!((back.x - canvas.x).abs() < 0.0001);
        assert!((back.y - canvas.y).abs() < 0.0001);
    }

    #[test]
    fn test_matrix_identity_screen_to_canvas() {
        let matrix = Matrix2D::identity();
        let p = Point::new(500.0, 300.0);
        let transformed = matrix.transform_point(&p);
        assert_eq!(transformed.x, 500.0);
        assert_eq!(transformed.y, 300.0);
    }

    #[test]
    fn test_matrix_scale_screen_to_canvas() {
        let mut matrix = Matrix2D::identity();
        matrix.scale(2.0, 3.0);
        let p = Point::new(100.0, 100.0);
        let screen = matrix.transform_point(&p);
        assert_eq!(screen.x, 200.0);
        assert_eq!(screen.y, 300.0);

        let inverse = matrix.inverse().unwrap();
        let canvas = inverse.transform_point(&screen);
        assert!((canvas.x - 100.0).abs() < 0.0001);
        assert!((canvas.y - 100.0).abs() < 0.0001);
    }

    #[test]
    fn test_matrix_translate_screen_to_canvas() {
        let matrix = Matrix2D::translation(100.0, -50.0);
        let p = Point::new(0.0, 0.0);
        let screen = matrix.transform_point(&p);
        assert_eq!(screen.x, 100.0);
        assert_eq!(screen.y, -50.0);

        let inverse = matrix.inverse().unwrap();
        let canvas = inverse.transform_point(&screen);
        assert!((canvas.x - 0.0).abs() < 0.0001);
        assert!((canvas.y - 0.0).abs() < 0.0001);
    }

    #[test]
    fn test_camera_interpolate_position() {
        let start = CameraKey::new(Point::new(0.0, 0.0), 1.0, 0.0);
        let end = CameraKey::new(Point::new(100.0, 200.0), 2.0, 0.0);
        let mid = start.interpolate(&end, 0.5);

        assert_eq!(mid.position.x, 50.0);
        assert_eq!(mid.position.y, 100.0);
        assert_eq!(mid.zoom, 1.5);
    }

    #[test]
    fn test_camera_interpolate_zoom() {
        let start = CameraKey::new(Point::ZERO, 1.0, 0.0);
        let end = CameraKey::new(Point::ZERO, 3.0, 0.0);
        let quarter = start.interpolate(&end, 0.25);

        assert_eq!(quarter.zoom, 1.5);
    }

    #[test]
    fn test_camera_with_anchor() {
        let camera = CameraKey::identity().with_anchor(Point::new(960.0, 540.0));
        let matrix = camera.to_matrix((1920.0, 1080.0));
        let p = Point::new(960.0, 540.0);
        let transformed = matrix.transform_point(&p);
        assert_eq!(transformed.x, 960.0);
        assert_eq!(transformed.y, 540.0);
    }
}
