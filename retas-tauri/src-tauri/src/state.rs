use std::sync::{Mutex, Arc};
use retas_core::{Document, Layer, RasterLayer, RasterFrame, History};

pub struct AppState {
    pub document: Mutex<Document>,
    pub history: Mutex<History>,
}

impl AppState {
    pub fn new() -> Self {
        let mut doc = Document::new("未命名", 1920.0, 1080.0);

        let mut layer1 = RasterLayer::new("背景");
        let mut layer2 = RasterLayer::new("图层 1");

        let width = doc.settings.resolution.width as u32;
        let height = doc.settings.resolution.height as u32;

        layer1.frames.insert(0, RasterFrame {
            frame_number: 0,
            image_data: Arc::new(vec![255u8; (width * height * 4) as usize]),
            width,
            height,
            bounds: None,
        });

        layer2.frames.insert(0, RasterFrame {
            frame_number: 0,
            image_data: Arc::new(vec![0u8; (width * height * 4) as usize]),
            width,
            height,
            bounds: None,
        });

        let layer1_id = layer1.base.id;
        let layer2_id = layer2.base.id;

        doc.layers.insert(layer1_id, Layer::Raster(layer1));
        doc.layers.insert(layer2_id, Layer::Raster(layer2));
        doc.timeline.layer_order.push(layer1_id);
        doc.timeline.layer_order.push(layer2_id);
        doc.selected_layers.push(layer2_id);

        Self {
            document: Mutex::new(doc),
            history: Mutex::new(History::new(50)),
        }
    }
}
