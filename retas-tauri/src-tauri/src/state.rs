use std::sync::Arc;
use retas_core::{Document, Layer, RasterLayer, RasterFrame};
use retas_core::advanced::undo::UndoManager;
use retas_core::advanced::render_queue::RenderQueue;
use retas_core::advanced::light_table::LightTableManager;
use retas_core::advanced::motion_check::MotionCheckManager;
use retas_core::advanced::batch::BatchQueue;
use retas_core::advanced::cut_system::CutManager;
use retas_core::advanced::guides::GuideLayer;

/// Unified editor state behind a single Mutex.
/// Eliminates the 3-Mutex deadlock class (Issue #17).
pub struct EditorState {
    pub document: Document,
    pub undo_manager: UndoManager,
    pub render_queue: RenderQueue,
    pub light_table: LightTableManager,
    pub motion_check: MotionCheckManager,
    pub batch_queue: BatchQueue,
    pub cut_manager: CutManager,
    pub guide_layer: GuideLayer,
}

impl EditorState {
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
            document: doc,
            undo_manager: UndoManager::new(),
            render_queue: RenderQueue::new(),
            light_table: LightTableManager::new(),
            motion_check: MotionCheckManager::new(),
            batch_queue: BatchQueue::new(),
            cut_manager: CutManager::new(),
            guide_layer: GuideLayer::new("默认参考线"),
        }
    }
}

pub struct AppState {
    pub editor: std::sync::Mutex<EditorState>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            editor: std::sync::Mutex::new(EditorState::new()),
        }
    }
}
