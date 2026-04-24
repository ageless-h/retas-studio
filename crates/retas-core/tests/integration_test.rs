use retas_core::{
    Document, Layer, RasterLayer, VectorLayer, RasterFrame, VectorFrame, Stroke, StrokePoint,
    Color8, BlendMode, Point, LayerId, LayerType, composite_layers, fill_rect, draw_circle,
};
use retas_core::advanced::{BrushSettings, BrushType, BrushBlendMode};
use std::sync::Arc;

mod common;
use common::fixtures::{create_test_document, create_test_raster_layer, create_test_vector_layer, pixel_color_at};

#[test]
fn test_document_create_and_add_layers() {
    let mut doc = create_test_document();
    assert_eq!(doc.settings.name, "Test Document");
    assert_eq!(doc.settings.resolution.width, 1920.0);
    assert_eq!(doc.layers.len(), 0);

    let raster = create_test_raster_layer("Background");
    let raster_id = doc.add_layer(Layer::Raster(raster));
    assert_eq!(doc.layers.len(), 1);
    assert_eq!(doc.timeline.layer_order.len(), 1);
    assert!(doc.modified);

    let vector = create_test_vector_layer("Line Art");
    let vector_id = doc.add_layer(Layer::Vector(vector));
    assert_eq!(doc.layers.len(), 2);
    assert_eq!(doc.timeline.layer_order.len(), 2);

    assert_eq!(doc.timeline.layer_order[0], raster_id);
    assert_eq!(doc.timeline.layer_order[1], vector_id);

    let layer = doc.get_layer(raster_id).unwrap();
    assert_eq!(layer.name(), "Background");
    assert_eq!(layer.layer_type(), LayerType::Raster);
}

#[test]
fn test_raster_layer_draw_and_composite() {
    let mut doc = create_test_document();
    let mut raster = create_test_raster_layer("Background");

    let width = 100u32;
    let height = 100u32;
    let mut pixels = vec![0u8; (width * height * 4) as usize];

    fill_rect(&mut pixels, width, height,
        retas_core::Rect::new(10.0, 10.0, 30.0, 30.0),
        Color8::RED);

    draw_circle(&mut pixels, width, height, 70.0, 70.0, 15.0, Color8::BLUE, true);

    let frame = RasterFrame {
        frame_number: 0,
        image_data: Arc::new(pixels.clone()),
        width,
        height,
        bounds: Some(retas_core::Rect::new(0.0, 0.0, width as f64, height as f64)),
    };
    raster.frames.insert(0, frame);

    doc.add_layer(Layer::Raster(raster));

    let layer = doc.layers.values().next().unwrap();
    if let Layer::Raster(r) = layer {
        assert!(r.frames.contains_key(&0));
        let stored = &r.frames[&0].image_data;
        assert_eq!(stored.len(), (width * height * 4) as usize);

        assert_eq!(pixel_color_at(stored, width, 15, 15), [255, 0, 0, 255]);
        assert_eq!(pixel_color_at(stored, width, 70, 70), [0, 0, 255, 255]);
        assert_eq!(pixel_color_at(stored, width, 5, 5), [0, 0, 0, 0]);
    } else {
        panic!("Expected raster layer");
    }
}

#[test]
fn test_vector_layer_strokes() {
    let mut doc = create_test_document();
    let mut vector = create_test_vector_layer("Line Art");

    let stroke = Stroke {
        points: vec![
            StrokePoint { position: Point::new(10.0, 10.0), pressure: 1.0, tilt: None },
            StrokePoint { position: Point::new(50.0, 50.0), pressure: 0.8, tilt: None },
            StrokePoint { position: Point::new(90.0, 10.0), pressure: 0.5, tilt: None },
        ],
        brush_size: 5.0,
        color: Color8::GREEN,
        opacity: 1.0,
    };

    let frame = VectorFrame {
        frame_number: 0,
        strokes: vec![stroke],
    };
    vector.frames.insert(0, frame);

    let vector_id = doc.add_layer(Layer::Vector(vector));

    let layer = doc.get_layer(vector_id).unwrap();
    if let Layer::Vector(v) = layer {
        assert_eq!(v.frames.len(), 1);
        let frame = &v.frames[&0];
        assert_eq!(frame.strokes.len(), 1);
        assert_eq!(frame.strokes[0].points.len(), 3);
        assert_eq!(frame.strokes[0].color, Color8::GREEN);
        assert_eq!(frame.strokes[0].brush_size, 5.0);
    } else {
        panic!("Expected vector layer");
    }
}

#[test]
fn test_document_json_save_and_load() {
    let mut doc = create_test_document();
    doc.settings.frame_rate = 30.0;
    doc.settings.total_frames = 60;
    doc.settings.background_color = Color8::new(32, 32, 32, 255);

    let mut raster = create_test_raster_layer("Background");
    let width = 64u32;
    let height = 64u32;
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    fill_rect(&mut pixels, width, height,
        retas_core::Rect::new(10.0, 10.0, 20.0, 20.0),
        Color8::new(255, 128, 64, 255));
    raster.frames.insert(0, RasterFrame {
        frame_number: 0,
        image_data: Arc::new(pixels),
        width,
        height,
        bounds: None,
    });
    doc.add_layer(Layer::Raster(raster));

    let mut vector = create_test_vector_layer("Ink");
    vector.frames.insert(0, VectorFrame {
        frame_number: 0,
        strokes: vec![Stroke {
            points: vec![
                StrokePoint { position: Point::new(5.0, 5.0), pressure: 1.0, tilt: None },
                StrokePoint { position: Point::new(55.0, 55.0), pressure: 0.5, tilt: None },
            ],
            brush_size: 3.0,
            color: Color8::BLACK,
            opacity: 1.0,
        }],
    });
    doc.add_layer(Layer::Vector(vector));

    let layer_ids: Vec<LayerId> = doc.timeline.layer_order.clone();
    doc.selected_layers = vec![layer_ids[0]];

    let json = serde_json::to_string_pretty(&doc).expect("Failed to serialize document");
    assert!(json.contains("Test Document"));
    assert!(json.contains("Background"));
    assert!(json.contains("Ink"));

    let loaded: Document = serde_json::from_str(&json).expect("Failed to deserialize document");

    assert_eq!(loaded.settings.name, doc.settings.name);
    assert_eq!(loaded.settings.frame_rate, doc.settings.frame_rate);
    assert_eq!(loaded.settings.total_frames, doc.settings.total_frames);
    assert_eq!(loaded.settings.background_color, doc.settings.background_color);
    assert_eq!(loaded.settings.resolution.width, doc.settings.resolution.width);
    assert_eq!(loaded.settings.resolution.height, doc.settings.resolution.height);

    assert_eq!(loaded.layers.len(), 2);
    assert_eq!(loaded.timeline.layer_order.len(), 2);
    assert_eq!(loaded.selected_layers.len(), 1);
    assert_eq!(loaded.selected_layers[0], layer_ids[0]);

    let loaded_raster = loaded.layers.get(&layer_ids[0]).unwrap();
    assert_eq!(loaded_raster.name(), "Background");
    if let Layer::Raster(r) = loaded_raster {
        assert!(r.frames.contains_key(&0));
        let frame = &r.frames[&0];
        assert_eq!(frame.width, width);
        assert_eq!(frame.height, height);
        assert_eq!(frame.image_data.len(), (width * height * 4) as usize);
        assert_eq!(pixel_color_at(&frame.image_data, width, 15, 15), [255, 128, 64, 255]);
    } else {
        panic!("Expected raster layer");
    }

    let loaded_vector = loaded.layers.get(&layer_ids[1]).unwrap();
    assert_eq!(loaded_vector.name(), "Ink");
    if let Layer::Vector(v) = loaded_vector {
        assert_eq!(v.frames[&0].strokes.len(), 1);
        assert_eq!(v.frames[&0].strokes[0].color, Color8::BLACK);
        assert_eq!(v.frames[&0].strokes[0].brush_size, 3.0);
    } else {
        panic!("Expected vector layer");
    }
}

#[test]
fn test_document_layer_operations() {
    let mut doc = create_test_document();

    let id1 = doc.add_layer(Layer::Raster(create_test_raster_layer("Layer1")));
    let id2 = doc.add_layer(Layer::Raster(create_test_raster_layer("Layer2")));
    let id3 = doc.add_layer(Layer::Raster(create_test_raster_layer("Layer3")));

    assert_eq!(doc.timeline.layer_order, vec![id1, id2, id3]);

    doc.move_layer(id1, 2);
    assert_eq!(doc.timeline.layer_order, vec![id2, id3, id1]);

    doc.move_layer(id1, 0);
    assert_eq!(doc.timeline.layer_order, vec![id1, id2, id3]);

    let removed = doc.remove_layer(id2);
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().name(), "Layer2");
    assert_eq!(doc.layers.len(), 2);
    assert_eq!(doc.timeline.layer_order, vec![id1, id3]);

    assert!(!doc.selected_layers.contains(&id2));
}

#[test]
fn test_composite_document_layers() {
    let mut doc = create_test_document();

    let mut raster1 = create_test_raster_layer("Red BG");
    let width = 10u32;
    let height = 10u32;
    let mut pixels1 = vec![0u8; (width * height * 4) as usize];
    fill_rect(&mut pixels1, width, height,
        retas_core::Rect::new(0.0, 0.0, 10.0, 10.0),
        Color8::new(255, 0, 0, 255));
    raster1.frames.insert(0, RasterFrame {
        frame_number: 0,
        image_data: Arc::new(pixels1),
        width,
        height,
        bounds: None,
    });
    doc.add_layer(Layer::Raster(raster1));

    let mut raster2 = create_test_raster_layer("Blue Overlay");
    raster2.base.opacity = 0.5;
    let mut pixels2 = vec![0u8; (width * height * 4) as usize];
    fill_rect(&mut pixels2, width, height,
        retas_core::Rect::new(0.0, 0.0, 10.0, 10.0),
        Color8::new(0, 0, 255, 255));
    raster2.frames.insert(0, RasterFrame {
        frame_number: 0,
        image_data: Arc::new(pixels2),
        width,
        height,
        bounds: None,
    });
    doc.add_layer(Layer::Raster(raster2));

    let layer_refs: Vec<&[u8]> = doc.timeline.layer_order.iter()
        .filter_map(|id| {
            doc.layers.get(id).and_then(|l| {
                if let Layer::Raster(r) = l {
                    r.frames.get(&0).map(|f| f.image_data.as_slice())
                } else {
                    None
                }
            })
        })
        .collect();

    let blend_modes = vec![BlendMode::Normal, BlendMode::Normal];
    let opacities = vec![1.0, 0.5];

    let result = composite_layers(&layer_refs, &blend_modes, &opacities, width, height);

    assert_eq!(result.len(), (width * height * 4) as usize);

    let idx = (5 * width + 5) as usize * 4;
    assert!(result[idx] > 127);
    assert_eq!(result[idx + 1], 0);
    assert!(result[idx + 2] > 127);
    assert_eq!(result[idx + 3], 255);
}

#[test]
fn test_project_with_multiple_documents() {
    let mut project = retas_core::Project::new();
    assert!(project.active_document().is_none());

    let mut doc1 = create_test_document();
    doc1.settings.name = "Scene 1".to_string();
    doc1.add_layer(Layer::Raster(create_test_raster_layer("BG1")));

    let mut doc2 = create_test_document();
    doc2.settings.name = "Scene 2".to_string();
    doc2.settings.resolution = retas_core::Size::new(1280.0, 720.0);
    doc2.add_layer(Layer::Raster(create_test_raster_layer("BG2")));

    let idx1 = project.add_document(doc1);
    assert_eq!(idx1, 0);
    assert_eq!(project.active_document, Some(0));

    let idx2 = project.add_document(doc2);
    assert_eq!(idx2, 1);
    assert_eq!(project.active_document, Some(1));

    // Verify active document
    let active = project.active_document().unwrap();
    assert_eq!(active.settings.name, "Scene 2");
    assert_eq!(active.settings.resolution.width, 1280.0);

    // Verify document count
    assert_eq!(project.documents.len(), 2);

    // Serialize entire project
    let json = serde_json::to_string_pretty(&project).expect("Failed to serialize project");
    let loaded: retas_core::Project = serde_json::from_str(&json).expect("Failed to deserialize project");

    assert_eq!(loaded.documents.len(), 2);
    assert_eq!(loaded.active_document, Some(1));
    assert_eq!(loaded.documents[0].settings.name, "Scene 1");
    assert_eq!(loaded.documents[1].settings.name, "Scene 2");
}

#[test]
fn test_roundtrip_with_modifications() {
    let mut doc = Document::new("Animation", 1920.0, 1080.0);
    doc.settings.frame_rate = 24.0;

    let mut raster = RasterLayer::new("Paint");
    raster.base.opacity = 0.8;
    raster.base.blend_mode = BlendMode::Multiply;
    let raster_id = doc.add_layer(Layer::Raster(raster));

    let mut vector = VectorLayer::new("Ink");
    vector.base.visible = false;
    vector.antialiasing = false;
    let vector_id = doc.add_layer(Layer::Vector(vector));

    let camera = retas_core::CameraLayer::new("Camera", 1920, 1080);
    let camera_id = doc.add_layer(Layer::Camera(camera));

    doc.timeline.start_frame = 1;
    doc.timeline.end_frame = 120;
    doc.timeline.current_frame = 5;

    let json = serde_json::to_string(&doc).unwrap();
    let mut loaded: Document = serde_json::from_str(&json).unwrap();

    loaded.settings.frame_rate = 30.0;
    if let Some(Layer::Raster(r)) = loaded.get_layer_mut(raster_id) {
        r.base.opacity = 1.0;
    }
    if let Some(Layer::Vector(v)) = loaded.get_layer_mut(vector_id) {
        v.base.visible = true;
    }

    let json2 = serde_json::to_string(&loaded).unwrap();
    let loaded2: Document = serde_json::from_str(&json2).unwrap();

    assert_eq!(loaded2.settings.frame_rate, 30.0);
    assert_eq!(loaded2.timeline.start_frame, 1);
    assert_eq!(loaded2.timeline.end_frame, 120);

    if let Layer::Raster(r) = loaded2.layers.get(&raster_id).unwrap() {
        assert_eq!(r.base.opacity, 1.0);
        assert_eq!(r.base.blend_mode, BlendMode::Multiply);
    } else {
        panic!("Expected raster layer");
    }

    if let Layer::Vector(v) = loaded2.layers.get(&vector_id).unwrap() {
        assert!(v.base.visible);
        assert!(!v.antialiasing);
    } else {
        panic!("Expected vector layer");
    }

    if let Layer::Camera(c) = loaded2.layers.get(&camera_id).unwrap() {
        assert_eq!(c.resolution, (1920, 1080));
        assert_eq!(c.frame_rate, 24.0);
    } else {
        panic!("Expected camera layer");
    }
}

#[test]
fn test_document_bounds() {
    let doc = Document::new("Test", 800.0, 600.0);
    let bounds = doc.bounds();
    assert_eq!(bounds.origin.x, 0.0);
    assert_eq!(bounds.origin.y, 0.0);
    assert_eq!(bounds.size.width, 800.0);
    assert_eq!(bounds.size.height, 600.0);
}
