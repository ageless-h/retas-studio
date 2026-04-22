#[cfg(test)]
mod tests {
    use retas_core::Matrix2D;

    #[test]
    fn test_matrix_identity() {
        let m = Matrix2D::identity();
        assert_eq!(m.a, 1.0);
        assert_eq!(m.b, 0.0);
        assert_eq!(m.c, 0.0);
        assert_eq!(m.d, 1.0);
        assert_eq!(m.tx, 0.0);
        assert_eq!(m.ty, 0.0);
    }

    #[test]
    fn test_matrix_translation() {
        let m = Matrix2D::translation(10.0, 20.0);
        assert_eq!(m.tx, 10.0);
        assert_eq!(m.ty, 20.0);
    }

    #[test]
    fn test_matrix_scale() {
        let m = Matrix2D::scale(2.0, 3.0);
        assert_eq!(m.a, 2.0);
        assert_eq!(m.d, 3.0);
    }

    #[test]
    fn test_matrix_rotation() {
        let m = Matrix2D::rotation(0.0);
        assert!((m.a - 1.0).abs() < 0.0001);
        assert!(m.b.abs() < 0.0001);
    }

    #[test]
    fn test_matrix_transform_point() {
        let m = Matrix2D::translation(10.0, 20.0);
        let p = retas_core::Point::new(0.0, 0.0);
        let result = m.transform_point(&p);
        assert_eq!(result.x, 10.0);
        assert_eq!(result.y, 20.0);
    }

    #[test]
    fn test_matrix_multiply() {
        let m1 = Matrix2D::translation(10.0, 0.0);
        let m2 = Matrix2D::translation(0.0, 20.0);
        let result = m1.multiply(&m2);
        assert_eq!(result.tx, 10.0);
        assert_eq!(result.ty, 20.0);
    }

    #[test]
    fn test_matrix_determinant() {
        let m = Matrix2D::scale(2.0, 3.0);
        assert_eq!(m.determinant(), 6.0);
    }

    #[test]
    fn test_matrix_inverse() {
        let m = Matrix2D::scale(2.0, 2.0);
        let inv = m.inverse().unwrap();
        let result = m.multiply(&inv);
        
        assert!((result.a - 1.0).abs() < 0.0001);
        assert!((result.d - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_matrix_inverse_singular() {
        let m = Matrix2D::ZERO;
        assert!(m.inverse().is_none());
    }

    #[test]
    fn test_matrix_to_array() {
        let m = Matrix2D::identity();
        let arr = m.to_array();
        assert_eq!(arr[0], 1.0);
        assert_eq!(arr[5], 0.0);
    }
}
