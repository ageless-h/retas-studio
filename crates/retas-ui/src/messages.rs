use retas_core::Color8;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tool {
    Brush,
    Eraser,
    Fill,
    Select,
    Move,
    Zoom,
    Hand,
    Pen,
    Text,
}

impl Tool {
    pub fn display_name(&self) -> &'static str {
        match self {
            Tool::Brush => "画笔",
            Tool::Eraser => "橡皮",
            Tool::Fill => "填充",
            Tool::Select => "选择",
            Tool::Move => "移动",
            Tool::Zoom => "缩放",
            Tool::Hand => "抓手",
            Tool::Pen => "钢笔",
            Tool::Text => "文字",
        }
    }
}

impl Default for Tool {
    fn default() -> Self {
        Tool::Brush
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    NewDocument,
    OpenDocument,
    SaveDocument,
    ExportPng,
    Undo,
    Redo,
    ToolSelected(ToolMessage),
    LayerSelected(LayerMessage),
    VectorLayerSelected(VectorLayerMessage),
    FillToolChanged(crate::fill_tool::FillToolMessage),
    EffectsChanged(crate::effects_panel::EffectsMessage),
    ExportChanged(crate::export_panel::ExportMessage),
    TimelineChanged(TimelineMessage),
    CanvasEvent(CanvasMessage),
    BrushSizeChanged(f64),
    ColorChanged(ColorMessage),
    ClearCanvas,
}

#[derive(Debug, Clone)]
pub enum ColorMessage {
    PresetSelected(Color8),
    HueChanged(f32),
    SaturationChanged(f32),
    ValueChanged(f32),
    RedChanged(u8),
    GreenChanged(u8),
    BlueChanged(u8),
}

#[derive(Debug, Clone)]
pub enum ToolMessage {
    Select,
    Move,
    Brush,
    Eraser,
    Fill,
    Eyedropper,
    Zoom,
    Hand,
    Pen,
    Text,
}

impl From<ToolMessage> for Tool {
    fn from(msg: ToolMessage) -> Self {
        match msg {
            ToolMessage::Select => Tool::Select,
            ToolMessage::Move => Tool::Move,
            ToolMessage::Brush => Tool::Brush,
            ToolMessage::Eraser => Tool::Eraser,
            ToolMessage::Fill => Tool::Fill,
            ToolMessage::Eyedropper => Tool::Select,
            ToolMessage::Zoom => Tool::Zoom,
            ToolMessage::Hand => Tool::Hand,
            ToolMessage::Pen => Tool::Pen,
            ToolMessage::Text => Tool::Text,
        }
    }
}

#[derive(Debug, Clone)]
pub enum LayerMessage {
    Add,
    Delete,
    Duplicate,
    MoveUp,
    MoveDown,
    Select(usize),
    ToggleVisibility(usize),
    ToggleLock(usize),
}

#[derive(Debug, Clone)]
pub enum VectorLayerMessage {
    Add,
    Delete(usize),
    Select(usize),
    ToggleVisibility(usize),
    MoveUp(usize),
    MoveDown(usize),
    SetPenMode(crate::vector_layer::PenToolMode),
}

#[derive(Debug, Clone)]
pub enum TimelineMessage {
    FrameChanged(u32),
    Play,
    Pause,
    Stop,
    AddFrame,
    DeleteFrame,
}

#[derive(Debug, Clone)]
pub enum CanvasMessage {
    MouseDown(f32, f32),
    MouseUp(f32, f32),
    MouseMoved(f32, f32),
    MouseWheel(f32),
    KeyPress(String),
    KeyRelease(String),
    Pan(f32, f32),
    PenFinish,
}
