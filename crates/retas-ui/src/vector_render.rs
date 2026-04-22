use iced::widget::canvas;
use iced::{Color, Point as IcedPoint, Rectangle, Renderer, Theme};
use retas_core::Point;
use retas_vector::BezierControlPoint;
use super::vector_layer::{VectorDocument, VectorPath, VectorLayer, PenToolState, PenToolMode};

pub struct VectorRenderer;

impl VectorRenderer {
    pub fn render_path(
        path: &VectorPath,
        frame: &mut canvas::Frame,
        offset: (f32, f32),
        zoom: f32,
        is_selected: bool,
        path_idx: usize,
        pen_state: &PenToolState,
    ) {
        if !path.visible || path.curve.points.is_empty() {
            return;
        }
        
        let points: Vec<IcedPoint> = path.curve.points.iter()
            .map(|p| IcedPoint::new(
                offset.0 + p.point.x as f32 * zoom,
                offset.1 + p.point.y as f32 * zoom,
            ))
            .collect();
        
        if points.len() < 2 {
            return;
        }
        
        if let Some(fill_color) = path.fill_color {
            let fill_iced_color = Color::from_rgba8(fill_color.r, fill_color.g, fill_color.b, fill_color.a as f32 / 255.0);
            
            let fill_path = Self::create_fill_path(&path.curve, offset, zoom);
            frame.fill(&fill_path, fill_iced_color);
        }
        
        if let Some(stroke_color) = path.stroke_color {
            let stroke_iced_color = Color::from_rgba8(stroke_color.r, stroke_color.g, stroke_color.b, stroke_color.a as f32 / 255.0);
            let stroke_width = path.stroke_width * zoom;
            
            let stroke_path = Self::create_stroke_path(&path.curve, offset, zoom);
            frame.stroke(
                &stroke_path,
                canvas::Stroke::default()
                    .with_color(stroke_iced_color)
                    .with_width(stroke_width)
                    .with_line_cap(canvas::LineCap::Round)
                    .with_line_join(canvas::LineJoin::Round),
            );
        }
        
        if is_selected {
            Self::render_control_points(&path.curve, frame, offset, zoom, path_idx, pen_state);
        }
    }
    
    fn create_fill_path(
        curve: &retas_vector::BezierCurve,
        offset: (f32, f32),
        zoom: f32,
    ) -> canvas::Path {
        let mut builder = canvas::path::Builder::new();
        
        if let Some(first) = curve.points.first() {
            builder.move_to(IcedPoint::new(
                offset.0 + first.point.x as f32 * zoom,
                offset.1 + first.point.y as f32 * zoom,
            ));
            
            for i in 1..curve.points.len() {
                let p1 = &curve.points[i - 1];
                let p2 = &curve.points[i];
                
                Self::add_bezier_segment(&mut builder, p1, p2, offset, zoom);
            }
            
            if curve.closed && curve.points.len() > 1 {
                let last = curve.points.last().expect("len > 1 guarantees last");
                let first = curve.points.first().expect("len > 1 guarantees first");
                Self::add_bezier_segment(&mut builder, last, first, offset, zoom);
                builder.close();
            }
        }

        builder.build()
    }

    fn create_stroke_path(
        curve: &retas_vector::BezierCurve,
        offset: (f32, f32),
        zoom: f32,
    ) -> canvas::Path {
        let mut builder = canvas::path::Builder::new();

        if let Some(first) = curve.points.first() {
            builder.move_to(IcedPoint::new(
                offset.0 + first.point.x as f32 * zoom,
                offset.1 + first.point.y as f32 * zoom,
            ));

            for i in 1..curve.points.len() {
                let p1 = &curve.points[i - 1];
                let p2 = &curve.points[i];

                Self::add_bezier_segment(&mut builder, p1, p2, offset, zoom);
            }

            if curve.closed && curve.points.len() > 1 {
                let last = curve.points.last().expect("len > 1 guarantees last");
                let first = curve.points.first().expect("len > 1 guarantees first");
                Self::add_bezier_segment(&mut builder, last, first, offset, zoom);
                builder.close();
            }
        }
        
        builder.build()
    }
    
    fn add_bezier_segment(
        builder: &mut canvas::path::Builder,
        p1: &BezierControlPoint,
        p2: &BezierControlPoint,
        offset: (f32, f32),
        zoom: f32,
    ) {
        match (p1.out_handle, p2.in_handle) {
            (Some(out_h), Some(in_h)) => {
                builder.bezier_curve_to(
                    IcedPoint::new(
                        offset.0 + out_h.x as f32 * zoom,
                        offset.1 + out_h.y as f32 * zoom,
                    ),
                    IcedPoint::new(
                        offset.0 + in_h.x as f32 * zoom,
                        offset.1 + in_h.y as f32 * zoom,
                    ),
                    IcedPoint::new(
                        offset.0 + p2.point.x as f32 * zoom,
                        offset.1 + p2.point.y as f32 * zoom,
                    ),
                );
            }
            (Some(out_h), None) => {
                builder.quadratic_curve_to(
                    IcedPoint::new(
                        offset.0 + out_h.x as f32 * zoom,
                        offset.1 + out_h.y as f32 * zoom,
                    ),
                    IcedPoint::new(
                        offset.0 + p2.point.x as f32 * zoom,
                        offset.1 + p2.point.y as f32 * zoom,
                    ),
                );
            }
            (None, Some(in_h)) => {
                builder.quadratic_curve_to(
                    IcedPoint::new(
                        offset.0 + in_h.x as f32 * zoom,
                        offset.1 + in_h.y as f32 * zoom,
                    ),
                    IcedPoint::new(
                        offset.0 + p2.point.x as f32 * zoom,
                        offset.1 + p2.point.y as f32 * zoom,
                    ),
                );
            }
            (None, None) => {
                builder.line_to(IcedPoint::new(
                    offset.0 + p2.point.x as f32 * zoom,
                    offset.1 + p2.point.y as f32 * zoom,
                ));
            }
        }
    }
    
    fn render_control_points(
        curve: &retas_vector::BezierCurve,
        frame: &mut canvas::Frame,
        offset: (f32, f32),
        zoom: f32,
        path_idx: usize,
        pen_state: &PenToolState,
    ) {
        let point_radius = 4.0 * zoom;
        let handle_radius = 3.0 * zoom;
        let handle_color = Color::from_rgb8(100, 150, 255);
        let point_color = Color::from_rgb8(255, 255, 255);
        let selected_color = Color::from_rgb8(255, 200, 0);
        let hovered_color = Color::from_rgb8(0, 255, 200);
        
        for (idx, point) in curve.points.iter().enumerate() {
            let point_pos = IcedPoint::new(
                offset.0 + point.point.x as f32 * zoom,
                offset.1 + point.point.y as f32 * zoom,
            );
            
            let is_selected = pen_state.is_point_selected(path_idx, idx);
            let is_hovered = pen_state.hovered_point == Some((path_idx, idx));
            
            let point_fill_color = if is_selected {
                selected_color
            } else if is_hovered {
                hovered_color
            } else {
                point_color
            };
            
            if let Some(in_handle) = point.in_handle {
                let handle_pos = IcedPoint::new(
                    offset.0 + in_handle.x as f32 * zoom,
                    offset.1 + in_handle.y as f32 * zoom,
                );
                
                let handle_line = canvas::Path::line(handle_pos, point_pos);
                frame.stroke(
                    &handle_line,
                    canvas::Stroke::default()
                        .with_color(handle_color)
                        .with_width(1.0),
                );
                
                let handle_circle = canvas::Path::circle(handle_pos, handle_radius);
                frame.fill(&handle_circle, handle_color);
            }
            
            if let Some(out_handle) = point.out_handle {
                let handle_pos = IcedPoint::new(
                    offset.0 + out_handle.x as f32 * zoom,
                    offset.1 + out_handle.y as f32 * zoom,
                );
                
                let handle_line = canvas::Path::line(point_pos, handle_pos);
                frame.stroke(
                    &handle_line,
                    canvas::Stroke::default()
                        .with_color(handle_color)
                        .with_width(1.0),
                );
                
                let handle_circle = canvas::Path::circle(handle_pos, handle_radius);
                frame.fill(&handle_circle, handle_color);
            }
            
            let point_circle = canvas::Path::circle(point_pos, point_radius);
            frame.fill(&point_circle, point_fill_color);
            frame.stroke(
                &point_circle,
                canvas::Stroke::default()
                    .with_color(selected_color)
                    .with_width(2.0),
            );
        }
    }
    
    pub fn render_document(
        document: &VectorDocument,
        frame: &mut canvas::Frame,
        offset: (f32, f32),
        zoom: f32,
    ) {
        let pen_state = &document.pen_state;
        
        for (layer_idx, layer) in document.layers.iter().enumerate() {
            if !layer.visible {
                continue;
            }
            
            let is_active = document.active_layer == Some(layer_idx);
            
            for (path_idx, path) in layer.paths.iter().enumerate() {
                let is_selected = is_active && layer.selected_path == Some(path_idx);
                Self::render_path(path, frame, offset, zoom, is_selected, path_idx, pen_state);
            }
        }
        
        if let Some(active_layer) = document.get_active_layer() {
            if let Some(path_idx) = active_layer.selected_path {
                if let Some(path) = active_layer.paths.get(path_idx) {
                    if !path.curve.closed && !path.curve.points.is_empty() {
                        if let Some(last_point) = path.curve.points.last() {
                            let preview_pos = IcedPoint::new(
                                offset.0 + last_point.point.x as f32 * zoom,
                                offset.1 + last_point.point.y as f32 * zoom,
                            );
                            let preview_circle = canvas::Path::circle(preview_pos, 3.0 * zoom);
                            frame.stroke(
                                &preview_circle,
                                canvas::Stroke::default()
                                    .with_color(Color::from_rgb8(200, 200, 200))
                                    .with_width(1.0),
                            );
                        }
                    }
                }
            }
        }
    }
}
