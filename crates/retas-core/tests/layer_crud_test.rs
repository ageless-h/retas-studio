#[cfg(test)]
mod tests {
    use retas_core::{
        Document, Project, RasterLayer, VectorLayer, CameraLayer, TextLayer, SoundLayer,
        Layer, LayerId, LayerType, BlendMode,
    };

    #[test]
    fn test_document_new() {
        let doc = Document::new("Test", 1920.0, 1080.0);
        assert_eq!(doc.settings.name, "Test");
        assert_eq!(doc.settings.resolution.width, 1920.0);
        assert_eq!(doc.settings.resolution.height, 1080.0);
        assert!(doc.layers.is_empty());
        assert!(doc.timeline.layer_order.is_empty());
        assert!(!doc.modified);
    }

    #[test]
    fn test_document_add_raster_layer() {
        let mut doc = Document::new("Test", 1920.0, 1080.0);
        let layer = RasterLayer::new("Background");
        let id = doc.add_layer(Layer::Raster(layer));

        assert_eq!(doc.layers.len(), 1);
        assert_eq!(doc.timeline.layer_order.len(), 1);
        assert_eq!(doc.timeline.layer_order[0], id);
        assert!(doc.modified);

        let layer_ref = doc.get_layer(id).unwrap();
        assert_eq!(layer_ref.name(), "Background");
        assert_eq!(layer_ref.layer_type(), LayerType::Raster);
    }

    #[test]
    fn test_document_add_multiple_layers() {
        let mut doc = Document::new("Test", 1920.0, 1080.0);
        let layer1 = RasterLayer::new("Layer 1");
        let id1 = doc.add_layer(Layer::Raster(layer1));
        let layer2 = VectorLayer::new("Layer 2");
        let id2 = doc.add_layer(Layer::Vector(layer2));
        let layer3 = TextLayer::new("Layer 3");
        let id3 = doc.add_layer(Layer::Text(layer3));

        assert_eq!(doc.layers.len(), 3);
        assert_eq!(doc.timeline.layer_order.len(), 3);
        assert_eq!(doc.timeline.layer_order, vec![id1, id2, id3]);
    }

    #[test]
    fn test_document_add_different_layer_types() {
        let mut doc = Document::new("Test", 1920.0, 1080.0);
        let raster = RasterLayer::new("Raster");
        let vector = VectorLayer::new("Vector");
        let camera = CameraLayer::new("Camera", 1920, 1080);
        let text = TextLayer::new("Text");
        let sound = SoundLayer::new("Sound");

        let id_r = doc.add_layer(Layer::Raster(raster));
        let id_v = doc.add_layer(Layer::Vector(vector));
        let id_c = doc.add_layer(Layer::Camera(camera));
        let id_t = doc.add_layer(Layer::Text(text));
        let id_s = doc.add_layer(Layer::Sound(sound));

        assert_eq!(doc.get_layer(id_r).unwrap().layer_type(), LayerType::Raster);
        assert_eq!(doc.get_layer(id_v).unwrap().layer_type(), LayerType::Vector);
        assert_eq!(doc.get_layer(id_c).unwrap().layer_type(), LayerType::Camera);
        assert_eq!(doc.get_layer(id_t).unwrap().layer_type(), LayerType::Text);
        assert_eq!(doc.get_layer(id_s).unwrap().layer_type(), LayerType::Sound);
    }

    #[test]
    fn test_document_remove_layer() {
        let mut doc = Document::new("Test", 1920.0, 1080.0);
        let layer = RasterLayer::new("ToRemove");
        let id = doc.add_layer(Layer::Raster(layer));

        let removed = doc.remove_layer(id);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name(), "ToRemove");
        assert!(doc.layers.is_empty());
        assert!(doc.timeline.layer_order.is_empty());
        assert!(doc.modified);
    }

    #[test]
    fn test_document_remove_nonexistent_layer() {
        let mut doc = Document::new("Test", 1920.0, 1080.0);
        let fake_id = LayerId::new();
        let removed = doc.remove_layer(fake_id);
        assert!(removed.is_none());
    }

    #[test]
    fn test_document_remove_layer_updates_selected() {
        let mut doc = Document::new("Test", 1920.0, 1080.0);
        let layer = RasterLayer::new("Selected");
        let id = doc.add_layer(Layer::Raster(layer));
        doc.selected_layers.push(id);

        doc.remove_layer(id);
        assert!(doc.selected_layers.is_empty());
    }

    #[test]
    fn test_document_get_layer_mut() {
        let mut doc = Document::new("Test", 1920.0, 1080.0);
        let layer = RasterLayer::new("Mutable");
        let id = doc.add_layer(Layer::Raster(layer));

        {
            let layer_mut = doc.get_layer_mut(id).unwrap();
            layer_mut.base_mut().name = "Renamed".to_string();
            layer_mut.base_mut().opacity = 0.5;
        }

        let layer_ref = doc.get_layer(id).unwrap();
        assert_eq!(layer_ref.name(), "Renamed");
        assert_eq!(layer_ref.base().opacity, 0.5);
        assert!(doc.modified);
    }

    #[test]
    fn test_document_move_layer_to_top() {
        let mut doc = Document::new("Test", 1920.0, 1080.0);
        let id1 = doc.add_layer(Layer::Raster(RasterLayer::new("A")));
        let id2 = doc.add_layer(Layer::Raster(RasterLayer::new("B")));
        let id3 = doc.add_layer(Layer::Raster(RasterLayer::new("C")));

        doc.move_layer(id1, 2);
        assert_eq!(doc.timeline.layer_order, vec![id2, id3, id1]);
    }

    #[test]
    fn test_document_move_layer_to_bottom() {
        let mut doc = Document::new("Test", 1920.0, 1080.0);
        let id1 = doc.add_layer(Layer::Raster(RasterLayer::new("A")));
        let id2 = doc.add_layer(Layer::Raster(RasterLayer::new("B")));
        let id3 = doc.add_layer(Layer::Raster(RasterLayer::new("C")));

        doc.move_layer(id3, 0);
        assert_eq!(doc.timeline.layer_order, vec![id3, id1, id2]);
    }

    #[test]
    fn test_document_move_layer_to_middle() {
        let mut doc = Document::new("Test", 1920.0, 1080.0);
        let id1 = doc.add_layer(Layer::Raster(RasterLayer::new("A")));
        let id2 = doc.add_layer(Layer::Raster(RasterLayer::new("B")));
        let id3 = doc.add_layer(Layer::Raster(RasterLayer::new("C")));

        doc.move_layer(id3, 1);
        assert_eq!(doc.timeline.layer_order, vec![id1, id3, id2]);
    }

    #[test]
    fn test_document_move_layer_clamps_index() {
        let mut doc = Document::new("Test", 1920.0, 1080.0);
        let id1 = doc.add_layer(Layer::Raster(RasterLayer::new("A")));
        let id2 = doc.add_layer(Layer::Raster(RasterLayer::new("B")));

        doc.move_layer(id1, 100);
        assert_eq!(doc.timeline.layer_order, vec![id2, id1]);
    }

    #[test]
    fn test_document_move_nonexistent_layer() {
        let mut doc = Document::new("Test", 1920.0, 1080.0);
        let id = doc.add_layer(Layer::Raster(RasterLayer::new("A")));
        let fake_id = LayerId::new();

        doc.move_layer(fake_id, 0);
        assert_eq!(doc.timeline.layer_order, vec![id]);
    }

    #[test]
    fn test_document_layer_properties() {
        let mut doc = Document::new("Test", 1920.0, 1080.0);
        let mut layer = RasterLayer::new("Props");
        layer.base.opacity = 0.75;
        layer.base.blend_mode = BlendMode::Multiply;
        layer.base.visible = false;
        layer.base.locked = true;

        let id = doc.add_layer(Layer::Raster(layer));
        let layer_ref = doc.get_layer(id).unwrap();

        assert_eq!(layer_ref.base().opacity, 0.75);
        assert_eq!(layer_ref.base().blend_mode, BlendMode::Multiply);
        assert!(!layer_ref.base().visible);
        assert!(layer_ref.base().locked);
    }

    #[test]
    fn test_document_bounds() {
        let doc = Document::new("Test", 1920.0, 1080.0);
        let bounds = doc.bounds();
        assert_eq!(bounds.left(), 0.0);
        assert_eq!(bounds.top(), 0.0);
        assert_eq!(bounds.right(), 1920.0);
        assert_eq!(bounds.bottom(), 1080.0);
    }

    #[test]
    fn test_timeline_duration() {
        let timeline = retas_core::Timeline::new();
        assert_eq!(timeline.duration_seconds(), 100.0 / 24.0);
    }

    #[test]
    fn test_timeline_frame_time_conversion() {
        let timeline = retas_core::Timeline::new();
        assert_eq!(timeline.frame_to_time(24), 1.0);
        assert_eq!(timeline.time_to_frame(1.0), 24);
    }

    #[test]
    fn test_project_add_document() {
        let mut project = Project::new();
        let doc1 = Document::new("Doc1", 1920.0, 1080.0);
        let doc2 = Document::new("Doc2", 1280.0, 720.0);

        let idx1 = project.add_document(doc1);
        let idx2 = project.add_document(doc2);

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(project.documents.len(), 2);
        assert_eq!(project.active_document, Some(1));
    }

    #[test]
    fn test_project_active_document() {
        let mut project = Project::new();
        assert!(project.active_document().is_none());

        let doc = Document::new("Doc", 1920.0, 1080.0);
        project.add_document(doc);

        assert!(project.active_document().is_some());
        assert_eq!(project.active_document().unwrap().settings.name, "Doc");
    }

    #[test]
    fn test_document_layer_order_preserved_after_removal() {
        let mut doc = Document::new("Test", 1920.0, 1080.0);
        let id1 = doc.add_layer(Layer::Raster(RasterLayer::new("A")));
        let id2 = doc.add_layer(Layer::Raster(RasterLayer::new("B")));
        let id3 = doc.add_layer(Layer::Raster(RasterLayer::new("C")));
        let id4 = doc.add_layer(Layer::Raster(RasterLayer::new("D")));

        doc.remove_layer(id2);
        assert_eq!(doc.timeline.layer_order, vec![id1, id3, id4]);

        doc.remove_layer(id4);
        assert_eq!(doc.timeline.layer_order, vec![id1, id3]);
    }

    #[test]
    fn test_document_modified_flag() {
        let mut doc = Document::new("Test", 1920.0, 1080.0);
        assert!(!doc.modified);

        doc.add_layer(Layer::Raster(RasterLayer::new("A")));
        assert!(doc.modified);

        doc.modified = false;
        doc.remove_layer(doc.timeline.layer_order[0]);
        assert!(doc.modified);

        doc.modified = false;
        let id = doc.add_layer(Layer::Raster(RasterLayer::new("B")));
        doc.get_layer_mut(id);
        assert!(doc.modified);
    }
}
