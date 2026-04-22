#[cfg(test)]
mod tests {
    use retas_core::Point;
    use retas_vector::{BezierCurve, BezierControlPoint, BezierPointType};

    #[test]
    fn test_bezier_curve_creation() {
        let curve = BezierCurve::new();
        assert!(curve.points.is_empty());
        assert!(!curve.closed);
    }

    #[test]
    fn test_bezier_corner_point() {
        let p = BezierControlPoint::corner(Point::new(10.0, 20.0));
        assert_eq!(p.point.x, 10.0);
        assert_eq!(p.point.y, 20.0);
        assert!(p.in_handle.is_none());
        assert!(p.out_handle.is_none());
        assert_eq!(p.point_type, BezierPointType::Corner);
    }

    #[test]
    fn test_bezier_smooth_point() {
        let p = BezierControlPoint::smooth(
            Point::new(10.0, 20.0),
            Point::new(5.0, 15.0),
            Point::new(15.0, 25.0),
        );
        assert_eq!(p.point.x, 10.0);
        assert!(p.in_handle.is_some());
        assert!(p.out_handle.is_some());
        assert_eq!(p.point_type, BezierPointType::Smooth);
    }

    #[test]
    fn test_bezier_add_point() {
        let mut curve = BezierCurve::new();
        curve.add_point(BezierControlPoint::corner(Point::new(0.0, 0.0)));
        curve.add_point(BezierControlPoint::corner(Point::new(10.0, 10.0)));
        assert_eq!(curve.points.len(), 2);
    }

    #[test]
    fn test_bezier_close() {
        let mut curve = BezierCurve::new();
        curve.close();
        assert!(curve.closed);
    }

    #[test]
    fn test_bezier_from_points() {
        let curve = BezierCurve::from_points(vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(20.0, 0.0),
        ]);
        assert_eq!(curve.points.len(), 3);
        assert!(!curve.closed);
    }

    #[test]
    fn test_bezier_closed() {
        let curve = BezierCurve::closed(vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(20.0, 0.0),
        ]);
        assert!(curve.closed);
    }

    #[test]
    fn test_bezier_bounds() {
        let curve = BezierCurve::from_points(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 50.0),
            Point::new(200.0, 100.0),
        ]);
        let bounds = curve.bounds().unwrap();
        assert_eq!(bounds.origin.x, 0.0);
        assert_eq!(bounds.origin.y, 0.0);
        assert_eq!(bounds.size.width, 200.0);
        assert_eq!(bounds.size.height, 100.0);
    }

    #[test]
    fn test_bezier_evaluate() {
        let mut curve = BezierCurve::new();
        curve.add_point(BezierControlPoint::corner(Point::new(0.0, 0.0)));
        curve.add_point(BezierControlPoint::corner(Point::new(10.0, 0.0)));
        
        let p = curve.evaluate(0.0).unwrap();
        assert_eq!(p.x, 0.0);
        
        let p = curve.evaluate(1.0).unwrap();
        assert_eq!(p.x, 10.0);
    }

    #[test]
    fn test_bezier_flatten() {
        let curve = BezierCurve::from_points(vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(20.0, 0.0),
        ]);
        let points = curve.flatten(1.0);
        assert!(points.len() >= 2);
    }
}
