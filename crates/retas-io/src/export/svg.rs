use super::ExportError;
use retas_vector::{Path, BezierCurve};

pub struct SvgExporter;

impl SvgExporter {
    pub fn export_path(path: &Path, output_path: &std::path::Path) -> Result<(), ExportError> {
        let mut svg = String::new();
        svg.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        svg.push_str("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1920\" height=\"1080\">\n");
        
        svg.push_str(&Self::path_to_svg(path));
        
        svg.push_str("</svg>\n");
        
        std::fs::write(output_path, svg)?;
        Ok(())
    }

    pub fn export_bezier(curve: &BezierCurve, output_path: &std::path::Path) -> Result<(), ExportError> {
        let mut svg = String::new();
        svg.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        svg.push_str("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1920\" height=\"1080\">\n");
        
        svg.push_str(&Self::bezier_to_svg(curve));
        
        svg.push_str("</svg>\n");
        
        std::fs::write(output_path, svg)?;
        Ok(())
    }

    fn path_to_svg(path: &Path) -> String {
        let mut d = String::new();
        
        for cmd in &path.commands {
            match cmd {
                retas_vector::PathCommand::MoveTo(p) => {
                    d.push_str(&format!("M {} {} ", p.x, p.y));
                }
                retas_vector::PathCommand::LineTo(p) => {
                    d.push_str(&format!("L {} {} ", p.x, p.y));
                }
                retas_vector::PathCommand::CurveTo(c1, c2, end) => {
                    d.push_str(&format!("C {} {} {} {} {} {} ", c1.x, c1.y, c2.x, c2.y, end.x, end.y));
                }
                retas_vector::PathCommand::QuadTo(ctrl, end) => {
                    d.push_str(&format!("Q {} {} {} {} ", ctrl.x, ctrl.y, end.x, end.y));
                }
                retas_vector::PathCommand::ArcTo { radii, rotation, large_arc, sweep, end } => {
                    let la = if *large_arc { 1 } else { 0 };
                    let sw = if *sweep { 1 } else { 0 };
                    d.push_str(&format!("A {} {} {} {} {} {} {} ", radii.0, radii.1, rotation, la, sw, end.x, end.y));
                }
                retas_vector::PathCommand::Close => {
                    d.push_str("Z ");
                }
            }
        }
        
        format!("  <path d=\"{}\" fill=\"none\" stroke=\"black\" stroke-width=\"1\"/>\n", d.trim())
    }

    fn bezier_to_svg(curve: &BezierCurve) -> String {
        if curve.points.is_empty() {
            return String::new();
        }

        let mut d = String::new();
        let first = &curve.points[0];
        d.push_str(&format!("M {} {} ", first.point.x, first.point.y));

        for i in 0..curve.points.len() - 1 {
            let p0 = &curve.points[i];
            let p1 = &curve.points[i + 1];
            
            if let (Some(out), Some(in_)) = (p0.out_handle, p1.in_handle) {
                d.push_str(&format!("C {} {} {} {} {} {} ", out.x, out.y, in_.x, in_.y, p1.point.x, p1.point.y));
            } else {
                d.push_str(&format!("L {} {} ", p1.point.x, p1.point.y));
            }
        }

        if curve.closed {
            d.push_str("Z");
        }

        format!("  <path d=\"{}\" fill=\"none\" stroke=\"black\" stroke-width=\"1\"/>\n", d.trim())
    }
}
