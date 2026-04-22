#[cfg(test)]
mod tests {
    use retas_vector::{Path, PathCommand, FillRule};
    use retas_core::Point;

    #[test]
    fn test_path_creation() {
        let path = Path::new();
        assert!(path.commands.is_empty());
    }

    #[test]
    fn test_path_move_to() {
        let path = Path::new().move_to(Point::new(10.0, 20.0));
        assert_eq!(path.commands.len(), 1);
        match &path.commands[0] {
            PathCommand::MoveTo(p) => {
                assert_eq!(p.x, 10.0);
                assert_eq!(p.y, 20.0);
            }
            _ => panic!("Expected MoveTo"),
        }
    }

    #[test]
    fn test_path_line_to() {
        let path = Path::new()
            .move_to(Point::new(0.0, 0.0))
            .line_to(Point::new(10.0, 10.0));
        assert_eq!(path.commands.len(), 2);
    }

    #[test]
    fn test_path_curve_to() {
        let path = Path::new()
            .move_to(Point::new(0.0, 0.0))
            .curve_to(
                Point::new(5.0, 0.0),
                Point::new(10.0, 5.0),
                Point::new(10.0, 10.0),
            );
        assert_eq!(path.commands.len(), 2);
    }

    #[test]
    fn test_path_close() {
        let path = Path::new()
            .move_to(Point::new(0.0, 0.0))
            .line_to(Point::new(10.0, 0.0))
            .close();
        assert_eq!(path.commands.len(), 3);
        match &path.commands[2] {
            PathCommand::Close => {}
            _ => panic!("Expected Close"),
        }
    }

    #[test]
    fn test_path_rect() {
        let path = Path::rect(0.0, 0.0, 100.0, 50.0);
        let bounds = path.bounds().unwrap();
        assert_eq!(bounds.size.width, 100.0);
        assert_eq!(bounds.size.height, 50.0);
    }

    #[test]
    fn test_path_ellipse() {
        let path = Path::ellipse(50.0, 50.0, 25.0, 25.0);
        assert!(path.bounds().is_some());
    }

    #[test]
    fn test_path_circle() {
        let path = Path::circle(50.0, 50.0, 25.0);
        assert!(path.bounds().is_some());
    }

    #[test]
    fn test_path_to_bezier_curves() {
        let path = Path::rect(0.0, 0.0, 100.0, 100.0);
        let curves = path.to_bezier_curves();
        assert_eq!(curves.len(), 1);
    }
}
