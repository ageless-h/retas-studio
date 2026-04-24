#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use retas_core::advanced::undo::*;
    use retas_core::{Document, RasterLayer, Layer};
    use retas_core::advanced::selection::{Selection, SelectionMask, SelectionTool, SelectionMode};

    fn create_test_document() -> Document {
        let mut doc = Document::new("test".to_string(), 1920.0, 1080.0);
        let layer = RasterLayer::new("Layer 1");
        let layer_id = layer.base.id;
        doc.layers.insert(layer_id, Layer::Raster(layer));
        doc.timeline.layer_order.push(layer_id);
        doc
    }

    #[test]
    fn test_undo_manager_new() {
        let manager = UndoManager::new();
        assert_eq!(manager.undo_count(), 0);
        assert_eq!(manager.redo_count(), 0);
        assert!(!manager.can_undo());
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_undo_manager_execute_and_undo() {
        let mut manager = UndoManager::new();
        let mut doc = create_test_document();
        
        let layer_id = doc.timeline.layer_order[0];
        let cmd = Box::new(TransformCommand {
            layer_id,
            old_offset: (0.0, 0.0),
            new_offset: (10.0, 20.0),
            description: "Move layer".to_string(),
        });
        
        manager.execute(cmd, &mut doc);
        
        assert_eq!(manager.undo_count(), 1);
        assert!(manager.can_undo());
        assert!(!manager.can_redo());
        
        let desc = manager.undo(&mut doc);
        assert!(desc.is_some());
        assert_eq!(desc.unwrap(), "Move layer");
        assert!(manager.can_redo());
    }

    #[test]
    fn test_undo_manager_redo() {
        let mut manager = UndoManager::new();
        let mut doc = create_test_document();
        
        let layer_id = doc.timeline.layer_order[0];
        let cmd = Box::new(TransformCommand {
            layer_id,
            old_offset: (0.0, 0.0),
            new_offset: (10.0, 20.0),
            description: "Move layer".to_string(),
        });
        
        manager.execute(cmd, &mut doc);
        manager.undo(&mut doc);
        
        assert!(manager.can_redo());
        let desc = manager.redo(&mut doc);
        assert!(desc.is_some());
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_undo_manager_clear() {
        let mut manager = UndoManager::new();
        let mut doc = create_test_document();
        
        let layer_id = doc.timeline.layer_order[0];
        let cmd = Box::new(TransformCommand {
            layer_id,
            old_offset: (0.0, 0.0),
            new_offset: (10.0, 20.0),
            description: "Move layer".to_string(),
        });
        
        manager.execute(cmd, &mut doc);
        manager.clear();
        
        assert_eq!(manager.undo_count(), 0);
        assert_eq!(manager.redo_count(), 0);
        assert!(!manager.can_undo());
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_undo_manager_max_levels() {
        let mut manager = UndoManager::new().with_max_levels(3);
        let mut doc = create_test_document();
        let layer_id = doc.timeline.layer_order[0];
        
        for i in 0..5 {
            let cmd = Box::new(TransformCommand {
                layer_id,
                old_offset: (0.0, 0.0),
                new_offset: (i as f64, 0.0),
                description: format!("Move {}", i),
            });
            manager.execute(cmd, &mut doc);
        }
        
        assert_eq!(manager.undo_count(), 3);
    }

    #[test]
    fn test_stroke_command_execute_undo() {
        let mut doc = create_test_document();
        let layer_id = doc.timeline.layer_order[0];
        
        let mut cmd = StrokeCommand {
            layer_id,
            stroke_id: 1,
            stroke_data: vec![255, 0, 0, 255],
            previous_pixel_data: Vec::new(),
            bounds: (0, 0, 1, 1),
            blend_mode: retas_core::layer::BlendMode::Normal,
            opacity: 1.0,
            description: "Brush stroke".to_string(),
        };
        
        cmd.execute(&mut doc);
        
        if let Layer::Raster(raster) = &doc.layers[&layer_id] {
            if let Some(frame) = raster.frames.get(&raster.current_frame) {
                assert_eq!(*frame.image_data, vec![255, 0, 0, 255]);
            }
        }
        
        cmd.undo(&mut doc);
        
        if let Layer::Raster(raster) = &doc.layers[&layer_id] {
            if let Some(frame) = raster.frames.get(&raster.current_frame) {
                assert!(frame.image_data.is_empty());
            }
        }
    }

    #[test]
    fn test_transform_command_execute_undo() {
        let mut doc = create_test_document();
        let layer_id = doc.timeline.layer_order[0];
        
        let mut cmd = TransformCommand {
            layer_id,
            old_offset: (0.0, 0.0),
            new_offset: (100.0, 200.0),
            description: "Move layer".to_string(),
        };
        
        cmd.execute(&mut doc);
        
        if let Layer::Raster(raster) = &doc.layers[&layer_id] {
            assert_eq!(raster.offset.x, 100.0);
            assert_eq!(raster.offset.y, 200.0);
        }
        
        cmd.undo(&mut doc);
        
        if let Layer::Raster(raster) = &doc.layers[&layer_id] {
            assert_eq!(raster.offset.x, 0.0);
            assert_eq!(raster.offset.y, 0.0);
        }
    }

    #[test]
    fn test_layer_property_command() {
        let mut doc = create_test_document();
        let layer_id = doc.timeline.layer_order[0];
        
        let mut cmd = LayerPropertyCommand {
            layer_id,
            property_name: "opacity".to_string(),
            old_value: PropertyValue::F64(1.0),
            new_value: PropertyValue::F64(0.5),
            description: "Change opacity".to_string(),
        };
        
        cmd.execute(&mut doc);
        
        if let Some(layer) = doc.layers.get(&layer_id) {
            assert_eq!(layer.base().opacity, 0.5);
        }
        
        cmd.undo(&mut doc);
        
        if let Some(layer) = doc.layers.get(&layer_id) {
            assert_eq!(layer.base().opacity, 1.0);
        }
    }

    #[test]
    fn test_selection_command() {
        let mut doc = create_test_document();
        
        let selection = Selection {
            tool: SelectionTool::Rectangular,
            mode: SelectionMode::Replace,
            mask: SelectionMask::Rectangular {
                rect: retas_core::Rect::new(0.0, 0.0, 100.0, 100.0),
            },
            feather: 0.0,
            anti_aliased: true,
            is_active: true,
        };
        
        let mut cmd = SelectionCommand {
            old_selection: None,
            new_selection: Some(selection.clone()),
            description: "Select area".to_string(),
        };
        
        cmd.execute(&mut doc);
        assert!(doc.selection.is_some());
        
        cmd.undo(&mut doc);
        assert!(doc.selection.is_none());
    }

    #[test]
    fn test_fill_command_execute_undo() {
        let mut doc = create_test_document();
        let layer_id = doc.timeline.layer_order[0];
        
        if let retas_core::Layer::Raster(raster) = doc.layers.get_mut(&layer_id).unwrap() {
            if let Some(frame) = raster.frames.get_mut(&raster.current_frame) {
                frame.image_data = Arc::new(vec![100, 150, 200, 255].repeat(4));
            }
        }
        
        let mut cmd = FillCommand {
            layer_id,
            selection: None,
            old_pixel_data: Vec::new(),
            fill_color: retas_core::Color8::new(255, 0, 0, 255),
            tolerance: 0.0,
            description: "Fill red".to_string(),
        };
        
        cmd.execute(&mut doc);
        
        if let retas_core::Layer::Raster(raster) = &doc.layers[&layer_id] {
            if let Some(frame) = raster.frames.get(&raster.current_frame) {
                assert_eq!(*frame.image_data, vec![255, 0, 0, 255].repeat(4));
            }
        }
        
        cmd.undo(&mut doc);
        
        if let retas_core::Layer::Raster(raster) = &doc.layers[&layer_id] {
            if let Some(frame) = raster.frames.get(&raster.current_frame) {
                assert_eq!(*frame.image_data, vec![100, 150, 200, 255].repeat(4));
            }
        }
    }

    #[test]
    fn test_frame_command_execute_undo() {
        let mut doc = create_test_document();
        let layer_id = doc.timeline.layer_order[0];
        
        let new_data = vec![255, 0, 0, 255].repeat(4);
        
        let mut cmd = FrameCommand {
            layer_id,
            frame_number: 0,
            old_frame_data: None,
            new_frame_data: Some(new_data.clone()),
            description: "Update frame".to_string(),
        };
        
        cmd.execute(&mut doc);
        
        if let retas_core::Layer::Raster(raster) = &doc.layers[&layer_id] {
            if let Some(frame) = raster.frames.get(&0) {
                assert_eq!(*frame.image_data, new_data);
            }
        }
        
        cmd.undo(&mut doc);
        
        if let retas_core::Layer::Raster(raster) = &doc.layers[&layer_id] {
            if let Some(frame) = raster.frames.get(&0) {
                assert!(frame.image_data.is_empty());
            }
        }
    }
}
