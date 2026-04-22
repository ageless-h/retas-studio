use std::sync::{Arc, Mutex};
use retas_core::{Document, Layer, RasterLayer, History};

use crate::ai::{AiPipeline, AiQueue, PipelineConfig};

pub struct AppState {
    pub document: Mutex<Document>,
    pub history: Mutex<History>,
    pub ai_queue: Mutex<Option<Arc<AiQueue>>>,
}

impl AppState {
    pub fn new() -> Self {
        let mut doc = Document::new("未命名", 1920.0, 1080.0);

        let layer1 = Layer::Raster(RasterLayer::new("背景"));
        let layer2 = Layer::Raster(RasterLayer::new("图层 1"));
        let layer1_id = layer1.id();
        let layer2_id = layer2.id();

        doc.layers.insert(layer1_id, layer1);
        doc.layers.insert(layer2_id, layer2);
        doc.timeline.layer_order.push(layer1_id);
        doc.timeline.layer_order.push(layer2_id);
        doc.selected_layers.push(layer2_id);

        let pipeline = Arc::new(AiPipeline::new(PipelineConfig::default()));
        let queue = Arc::new(AiQueue::new(50, pipeline));

        Self {
            document: Mutex::new(doc),
            history: Mutex::new(History::new(50)),
            ai_queue: Mutex::new(Some(queue)),
        }
    }
}
