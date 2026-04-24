use retas_core::{Color8, Point};
use retas_core::advanced::{BrushBlendMode, BrushPoint, BrushSettings, BrushStroke, BrushType, BrushDynamics};

#[test]
fn test_brush_settings_default() {
    let settings = BrushSettings::default();
    assert_eq!(settings.brush_type, BrushType::Round);
    assert_eq!(settings.size, 10.0);
    assert_eq!(settings.opacity, 1.0);
    assert_eq!(settings.hardness, 0.8);
}

#[test]
fn test_brush_settings_builder() {
    let settings = BrushSettings::new(20.0, Color8::RED)
        .with_opacity(0.5)
        .with_hardness(0.9)
        .with_type(BrushType::Flat)
        .with_blend_mode(BrushBlendMode::Multiply);

    assert_eq!(settings.size, 20.0);
    assert_eq!(settings.color, Color8::RED);
    assert_eq!(settings.opacity, 0.5);
    assert_eq!(settings.hardness, 0.9);
    assert_eq!(settings.brush_type, BrushType::Flat);
    assert_eq!(settings.blend_mode, BrushBlendMode::Multiply);
}

#[test]
fn test_calculate_size_no_dynamics() {
    let mut settings = BrushSettings::new(10.0, Color8::BLACK);
    settings.dynamics.pressure_size = false;
    assert_eq!(settings.calculate_size(0.0, 0.0), 10.0);
    assert_eq!(settings.calculate_size(1.0, 0.0), 10.0);
}

#[test]
fn test_calculate_size_pressure_dynamics() {
    let settings = BrushSettings::new(10.0, Color8::BLACK)
        .with_pressure_dynamics(true, false);

    assert_eq!(settings.calculate_size(0.0, 0.0), 1.0);
    assert_eq!(settings.calculate_size(1.0, 0.0), 10.0);
    assert!(settings.calculate_size(0.5, 0.0) > 1.0 && settings.calculate_size(0.5, 0.0) < 10.0);
}

#[test]
fn test_calculate_size_clamps_pressure() {
    let settings = BrushSettings::new(10.0, Color8::BLACK)
        .with_pressure_dynamics(true, false);

    assert_eq!(settings.calculate_size(-1.0, 0.0), 1.0);
    assert_eq!(settings.calculate_size(2.0, 0.0), 10.0);
}

#[test]
fn test_calculate_size_velocity_dynamics() {
    let mut settings = BrushSettings::new(10.0, Color8::BLACK);
    settings.dynamics.pressure_size = false;
    settings.dynamics.velocity_size = true;

    let slow = settings.calculate_size(0.0, 0.0);
    let fast = settings.calculate_size(0.0, 1.0);
    assert!(fast < slow, "fast velocity should reduce size: fast={} slow={}", fast, slow);
}

#[test]
fn test_calculate_opacity_no_dynamics() {
    let mut settings = BrushSettings::new(10.0, Color8::BLACK);
    settings.dynamics.pressure_opacity = false;
    assert_eq!(settings.calculate_opacity(0.0, 0.0), 1.0);
}

#[test]
fn test_calculate_opacity_pressure_dynamics() {
    let settings = BrushSettings::new(10.0, Color8::BLACK)
        .with_pressure_dynamics(false, true);

    assert_eq!(settings.calculate_opacity(0.0, 0.0), 0.0);
    assert_eq!(settings.calculate_opacity(1.0, 0.0), 1.0);
}

#[test]
fn test_calculate_opacity_clamps() {
    let mut settings = BrushSettings::new(10.0, Color8::BLACK);
    settings.dynamics.pressure_opacity = false;
    assert_eq!(settings.calculate_opacity(0.0, 0.0), 1.0);

    let mut high = BrushSettings::new(10.0, Color8::BLACK);
    high.dynamics.pressure_opacity = false;
    high.opacity = 2.0;
    assert_eq!(high.calculate_opacity(0.0, 0.0), 1.0);
}

#[test]
fn test_calculate_flow() {
    let settings = BrushSettings::new(10.0, Color8::BLACK);
    assert_eq!(settings.calculate_flow(0.0), 1.0);
    assert_eq!(settings.calculate_flow(1.0), 1.0);
}

#[test]
fn test_calculate_angle_fixed() {
    let mut settings = BrushSettings::new(10.0, Color8::BLACK);
    settings.shape.angle = 45.0;
    settings.shape.angle_control = retas_core::advanced::brush::AngleControl::Fixed;
    assert_eq!(settings.calculate_angle(90.0), 45.0);
}

#[test]
fn test_calculate_angle_direction() {
    let mut settings = BrushSettings::new(10.0, Color8::BLACK);
    settings.shape.angle = 0.0;
    settings.shape.angle_control = retas_core::advanced::brush::AngleControl::Direction;
    assert_eq!(settings.calculate_angle(std::f64::consts::PI / 2.0), 90.0);
}

#[test]
fn test_brush_point_builder() {
    let point = BrushPoint::new(Point::new(10.0, 20.0))
        .with_pressure(0.5)
        .with_velocity(1.0)
        .with_direction(45.0);

    assert_eq!(point.position.x, 10.0);
    assert_eq!(point.position.y, 20.0);
    assert_eq!(point.pressure, 0.5);
    assert_eq!(point.velocity, 1.0);
    assert_eq!(point.direction, 45.0);
}

#[test]
fn test_brush_stroke_new() {
    let settings = BrushSettings::new(10.0, Color8::BLACK);
    let stroke = BrushStroke::new(settings.clone());
    assert!(stroke.points.is_empty());
    assert!(!stroke.is_drawing);
    assert_eq!(stroke.settings.size, 10.0);
}

#[test]
fn test_brush_stroke_add_point() {
    let settings = BrushSettings::new(10.0, Color8::BLACK);
    let mut stroke = BrushStroke::new(settings);
    stroke.add_point(BrushPoint::new(Point::new(1.0, 2.0)));
    stroke.add_point(BrushPoint::new(Point::new(3.0, 4.0)));
    assert_eq!(stroke.points.len(), 2);
}

#[test]
fn test_brush_stroke_calculate_interpolated_points_empty() {
    let settings = BrushSettings::new(10.0, Color8::BLACK);
    let stroke = BrushStroke::new(settings);
    let result = stroke.calculate_interpolated_points(0.1);
    assert!(result.is_empty());
}

#[test]
fn test_brush_stroke_calculate_interpolated_points_single() {
    let settings = BrushSettings::new(10.0, Color8::BLACK);
    let mut stroke = BrushStroke::new(settings);
    stroke.add_point(BrushPoint::new(Point::new(0.0, 0.0)));
    let result = stroke.calculate_interpolated_points(0.1);
    assert_eq!(result.len(), 1);
}

#[test]
fn test_brush_stroke_calculate_interpolated_points_multiple() {
    let mut settings = BrushSettings::new(10.0, Color8::BLACK);
    settings.spacing = 0.5;
    let mut stroke = BrushStroke::new(settings);
    stroke.add_point(BrushPoint::new(Point::new(0.0, 0.0)));
    stroke.add_point(BrushPoint::new(Point::new(10.0, 0.0)));
    let result = stroke.calculate_interpolated_points(0.1);
    assert!(result.len() >= 2, "should interpolate between points, got {}", result.len());
}

#[test]
fn test_brush_stroke_smooth() {
    let settings = BrushSettings::new(10.0, Color8::BLACK);
    let mut stroke = BrushStroke::new(settings);
    stroke.add_point(BrushPoint::new(Point::new(0.0, 0.0)));
    stroke.add_point(BrushPoint::new(Point::new(10.0, 10.0)));
    stroke.add_point(BrushPoint::new(Point::new(20.0, 0.0)));

    stroke.smooth(0.5);
    assert_eq!(stroke.points.len(), 3);
}

#[test]
fn test_brush_stroke_smooth_too_few_points() {
    let settings = BrushSettings::new(10.0, Color8::BLACK);
    let mut stroke = BrushStroke::new(settings);
    stroke.add_point(BrushPoint::new(Point::new(0.0, 0.0)));
    stroke.smooth(0.5);
    assert_eq!(stroke.points.len(), 1);
}

#[test]
fn test_brush_stroke_apply_stabilization() {
    let mut settings = BrushSettings::new(10.0, Color8::BLACK);
    settings.stabilization = 0.5;
    let mut stroke = BrushStroke::new(settings);
    let points = vec![
        BrushPoint::new(Point::new(0.0, 0.0)),
        BrushPoint::new(Point::new(10.0, 0.0)),
        BrushPoint::new(Point::new(20.0, 0.0)),
    ];
    stroke.apply_stabilization(&points);
    assert_eq!(stroke.points.len(), 3);
}

#[test]
fn test_brush_stroke_apply_stabilization_empty() {
    let settings = BrushSettings::new(10.0, Color8::BLACK);
    let mut stroke = BrushStroke::new(settings);
    stroke.apply_stabilization(&[]);
    assert!(stroke.points.is_empty());
}

#[test]
fn test_brush_dynamics_default() {
    let dynamics = BrushDynamics::default();
    assert!(dynamics.pressure_size);
    assert!(dynamics.pressure_opacity);
    assert!(!dynamics.velocity_size);
}
