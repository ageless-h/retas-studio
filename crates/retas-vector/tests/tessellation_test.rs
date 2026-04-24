#[cfg(test)]
mod tests {
    use retas_core::{Point, Color8};
    use retas_vector::{Stroke, StrokeStyle, PressurePoint, Tessellator};

    fn straight_stroke() -> Stroke {
        let mut stroke = Stroke::new(StrokeStyle::new(Color8::BLACK, 4.0));
        stroke.add_point(PressurePoint::new(Point::new(0.0, 0.0), 1.0));
        stroke.add_point(PressurePoint::new(Point::new(100.0, 0.0), 1.0));
        stroke
    }

    fn curved_stroke() -> Stroke {
        let mut stroke = Stroke::new(StrokeStyle::new(Color8::BLACK, 4.0));
        stroke.add_point(PressurePoint::new(Point::new(0.0, 0.0), 1.0));
        stroke.add_point(PressurePoint::new(Point::new(50.0, 10.0), 0.8));
        stroke.add_point(PressurePoint::new(Point::new(100.0, 0.0), 1.0));
        stroke
    }

    fn sharp_curved_stroke() -> Stroke {
        let mut stroke = Stroke::new(StrokeStyle::new(Color8::BLACK, 4.0));
        stroke.add_point(PressurePoint::new(Point::new(0.0, 0.0), 1.0));
        stroke.add_point(PressurePoint::new(Point::new(50.0, 50.0), 0.8));
        stroke.add_point(PressurePoint::new(Point::new(100.0, 0.0), 1.0));
        stroke
    }

    #[test]
    fn test_flatten_straight_line_minimal_subdivision() {
        let stroke = straight_stroke();
        let tess = Tessellator::new().with_tolerance(1.0);
        let flat = tess.flatten_stroke(&stroke);

        assert_eq!(flat.len(), 2);
        assert_eq!(flat[0].position.x, 0.0);
        assert_eq!(flat[1].position.x, 100.0);
    }

    #[test]
    fn test_flatten_curved_stroke_produces_more_points() {
        let stroke = curved_stroke();
        let tess = Tessellator::new().with_tolerance(1.0);
        let flat = tess.flatten_stroke(&stroke);

        assert!(
            flat.len() > 3,
            "Curved stroke should be subdivided into more than 3 points, got {}",
            flat.len()
        );
    }

    #[test]
    fn test_flatten_sharp_curve_more_points_than_gentle_curve() {
        let gentle = curved_stroke();
        let sharp = sharp_curved_stroke();

        let tess = Tessellator::new().with_tolerance(1.0);
        let gentle_flat = tess.flatten_stroke(&gentle);
        let sharp_flat = tess.flatten_stroke(&sharp);

        assert!(
            sharp_flat.len() > gentle_flat.len(),
            "Sharp curve (len={}) should produce more points than gentle curve (len={})",
            sharp_flat.len(),
            gentle_flat.len()
        );
    }

    #[test]
    fn test_flatten_lower_tolerance_more_points() {
        let stroke = sharp_curved_stroke();

        let coarse = Tessellator::new().with_tolerance(10.0).flatten_stroke(&stroke);
        let fine = Tessellator::new().with_tolerance(0.1).flatten_stroke(&stroke);

        assert!(
            fine.len() > coarse.len(),
            "Fine tolerance (len={}) should produce more points than coarse (len={})",
            fine.len(),
            coarse.len()
        );
    }

    #[test]
    fn test_flatten_preserves_endpoints() {
        let stroke = sharp_curved_stroke();
        let tess = Tessellator::new().with_tolerance(0.5);
        let flat = tess.flatten_stroke(&stroke);

        assert_eq!(flat.first().unwrap().position.x, 0.0);
        assert_eq!(flat.first().unwrap().position.y, 0.0);
        assert_eq!(flat.last().unwrap().position.x, 100.0);
        assert_eq!(flat.last().unwrap().position.y, 0.0);
    }

    #[test]
    fn test_flatten_interpolates_pressure() {
        let mut stroke = Stroke::new(StrokeStyle::new(Color8::BLACK, 4.0));
        stroke.add_point(PressurePoint::new(Point::new(0.0, 0.0), 1.0));
        stroke.add_point(PressurePoint::new(Point::new(50.0, 50.0), 0.5));
        stroke.add_point(PressurePoint::new(Point::new(100.0, 0.0), 1.0));

        let tess = Tessellator::new().with_tolerance(1.0);
        let flat = tess.flatten_stroke(&stroke);

        let interpolated_exists = flat.iter().any(|p| p.pressure > 0.5 && p.pressure < 1.0);

        assert!(
            interpolated_exists,
            "At least one subdivided point should have interpolated pressure between 0.5 and 1.0"
        );
    }

    #[test]
    fn test_tessellate_stroke_produces_mesh() {
        let stroke = sharp_curved_stroke();
        let mesh = Tessellator::new().with_tolerance(2.0).tessellate_stroke(&stroke);

        assert!(!mesh.vertices.is_empty(), "Mesh should have vertices");
        assert!(!mesh.indices.is_empty(), "Mesh should have indices");
        assert_eq!(mesh.indices.len() % 6, 0, "Indices should form triangles in groups of 6");
    }

    #[test]
    fn test_tessellate_single_point_returns_empty() {
        let mut stroke = Stroke::new(StrokeStyle::new(Color8::BLACK, 4.0));
        stroke.add_point(PressurePoint::new(Point::new(0.0, 0.0), 1.0));

        let mesh = Tessellator::new().tessellate_stroke(&stroke);
        assert!(mesh.vertices.is_empty());
        assert!(mesh.indices.is_empty());
    }
}
