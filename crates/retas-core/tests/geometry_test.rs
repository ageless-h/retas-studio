#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_creation() {
        let p = Point::new(10.0, 20.0);
        assert_eq!(p.x, 10.0);
        assert_eq!(p.y, 20.0);
    }

    #[test]
    fn test_point_zero() {
        let p = Point::ZERO;
        assert_eq!(p.x, 0.0);
        assert_eq!(p.y, 0.0);
    }

    #[test]
    fn test_point_distance() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(3.0, 4.0);
        assert_eq!(p1.distance_to(&p2), 5.0);
    }

    #[test]
    fn test_point_midpoint() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(10.0, 20.0);
        let mid = p1.midpoint(&p2);
        assert_eq!(mid.x, 5.0);
        assert_eq!(mid.y, 10.0);
    }

    #[test]
    fn test_point_lerp() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(10.0, 20.0);
        let result = p1.lerp(&p2, 0.5);
        assert_eq!(result.x, 5.0);
        assert_eq!(result.y, 10.0);
    }

    #[test]
    fn test_size_area() {
        let s = Size::new(10.0, 20.0);
        assert_eq!(s.area(), 200.0);
    }

    #[test]
    fn test_size_aspect_ratio() {
        let s = Size::new(16.0, 9.0);
        assert!((s.aspect_ratio() - 16.0 / 9.0).abs() < 0.0001);
    }

    #[test]
    fn test_rect_contains() {
        let r = Rect::new(0.0, 0.0, 100.0, 100.0);
        assert!(r.contains(&Point::new(50.0, 50.0)));
        assert!(r.contains(&Point::new(0.0, 0.0)));
        assert!(r.contains(&Point::new(100.0, 100.0)));
        assert!(!r.contains(&Point::new(101.0, 50.0)));
    }

    #[test]
    fn test_rect_intersects() {
        let r1 = Rect::new(0.0, 0.0, 100.0, 100.0);
        let r2 = Rect::new(50.0, 50.0, 100.0, 100.0);
        assert!(r1.intersects(&r2));
        
        let r3 = Rect::new(200.0, 200.0, 100.0, 100.0);
        assert!(!r1.intersects(&r3));
    }

    #[test]
    fn test_rect_union() {
        let r1 = Rect::new(0.0, 0.0, 100.0, 100.0);
        let r2 = Rect::new(50.0, 50.0, 100.0, 100.0);
        let union = r1.union(&r2);
        assert_eq!(union.left(), 0.0);
        assert_eq!(union.top(), 0.0);
        assert_eq!(union.right(), 150.0);
        assert_eq!(union.bottom(), 150.0);
    }

    #[test]
    fn test_vector_length() {
        let v = Vector2D::new(3.0, 4.0);
        assert_eq!(v.length(), 5.0);
    }

    #[test]
    fn test_vector_normalize() {
        let v = Vector2D::new(3.0, 4.0);
        let n = v.normalize();
        assert!((n.length() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_vector_dot() {
        let v1 = Vector2D::new(1.0, 0.0);
        let v2 = Vector2D::new(0.0, 1.0);
        assert_eq!(v1.dot(&v2), 0.0);
        
        let v3 = Vector2D::new(1.0, 1.0);
        assert_eq!(v1.dot(&v3), 1.0);
    }

    #[test]
    fn test_vector_cross() {
        let v1 = Vector2D::new(1.0, 0.0);
        let v2 = Vector2D::new(0.0, 1.0);
        assert_eq!(v1.cross(&v2), 1.0);
    }
}
