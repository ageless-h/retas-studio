use serde::{Deserialize, Serialize};
use crate::{Rect, Point, LayerId, Color8};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintSettings {
    pub paper_size: PaperSize,
    pub orientation: Orientation,
    pub margins: Margins,
    pub scale: f64,
    pub fit_to_page: bool,
    pub center_on_page: bool,
    pub print_background: bool,
    pub background_color: Color8,
    pub crop_marks: bool,
    pub registration_marks: bool,
    pub frame_numbers: bool,
    pub layer_names: bool,
    pub header: Option<String>,
    pub footer: Option<String>,
}

impl Default for PrintSettings {
    fn default() -> Self {
        Self {
            paper_size: PaperSize::A4,
            orientation: Orientation::Portrait,
            margins: Margins::default(),
            scale: 100.0,
            fit_to_page: true,
            center_on_page: true,
            print_background: false,
            background_color: Color8::new(255, 255, 255, 255),
            crop_marks: false,
            registration_marks: false,
            frame_numbers: true,
            layer_names: false,
            header: None,
            footer: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PaperSize {
    A4,
    A3,
    Letter,
    Legal,
    Tabloid,
    Custom { width: f64, height: f64 },
}

impl PaperSize {
    pub fn dimensions_mm(&self) -> (f64, f64) {
        match self {
            PaperSize::A4 => (210.0, 297.0),
            PaperSize::A3 => (297.0, 420.0),
            PaperSize::Letter => (215.9, 279.4),
            PaperSize::Legal => (215.9, 355.6),
            PaperSize::Tabloid => (279.4, 431.8),
            PaperSize::Custom { width, height } => (*width, *height),
        }
    }

    pub fn dimensions_points(&self) -> (f64, f64) {
        let (w, h) = self.dimensions_mm();
        (w * 2.83465, h * 2.83465)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Orientation {
    Portrait,
    Landscape,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Margins {
    pub top: f64,
    pub bottom: f64,
    pub left: f64,
    pub right: f64,
}

impl Default for Margins {
    fn default() -> Self {
        Self {
            top: 10.0,
            bottom: 10.0,
            left: 10.0,
            right: 10.0,
        }
    }
}

impl Margins {
    pub fn new(all: f64) -> Self {
        Self {
            top: all,
            bottom: all,
            left: all,
            right: all,
        }
    }

    pub fn mm_to_points(&self) -> Margins {
        Self {
            top: self.top * 2.83465,
            bottom: self.bottom * 2.83465,
            left: self.left * 2.83465,
            right: self.right * 2.83465,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CelPrintJob {
    pub id: u64,
    pub name: String,
    pub frames: Vec<u32>,
    pub layers: Vec<LayerId>,
    pub settings: PrintSettings,
    pub layout: CelLayout,
    pub per_page: u32,
}

impl CelPrintJob {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: 0,
            name: name.into(),
            frames: Vec::new(),
            layers: Vec::new(),
            settings: PrintSettings::default(),
            layout: CelLayout::default(),
            per_page: 1,
        }
    }

    pub fn add_frame(&mut self, frame: u32) {
        if !self.frames.contains(&frame) {
            self.frames.push(frame);
        }
    }

    pub fn add_layer(&mut self, layer: LayerId) {
        if !self.layers.contains(&layer) {
            self.layers.push(layer);
        }
    }

    pub fn page_count(&self) -> u32 {
        let total = self.frames.len() * self.layers.len().max(1);
        ((total as f64) / (self.per_page as f64)).ceil() as u32
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CelLayout {
    pub columns: u32,
    pub rows: u32,
    pub spacing: f64,
    pub border: f64,
    pub show_labels: bool,
    pub label_position: LabelPosition,
}

impl Default for CelLayout {
    fn default() -> Self {
        Self {
            columns: 1,
            rows: 1,
            spacing: 5.0,
            border: 1.0,
            show_labels: true,
            label_position: LabelPosition::Bottom,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LabelPosition {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScorePrintJob {
    pub id: u64,
    pub name: String,
    pub start_frame: u32,
    pub end_frame: u32,
    pub settings: PrintSettings,
    pub column_width: f64,
    pub row_height: f64,
    pub show_layer_names: bool,
    pub show_frame_numbers: bool,
    pub highlight_current_frame: bool,
    pub current_frame: Option<u32>,
}

impl ScorePrintJob {
    pub fn new(name: impl Into<String>, start: u32, end: u32) -> Self {
        Self {
            id: 0,
            name: name.into(),
            start_frame: start,
            end_frame: end,
            settings: PrintSettings::default(),
            column_width: 20.0,
            row_height: 30.0,
            show_layer_names: true,
            show_frame_numbers: true,
            highlight_current_frame: false,
            current_frame: None,
        }
    }

    pub fn frame_count(&self) -> u32 {
        self.end_frame.saturating_sub(self.start_frame) + 1
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintPreview {
    pub page_width: f64,
    pub page_height: f64,
    pub pages: Vec<PrintPage>,
    pub current_page: u32,
}

impl PrintPreview {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            page_width: width,
            page_height: height,
            pages: Vec::new(),
            current_page: 0,
        }
    }

    pub fn add_page(&mut self, page: PrintPage) {
        self.pages.push(page);
    }

    pub fn page_count(&self) -> u32 {
        self.pages.len() as u32
    }

    pub fn current_page(&self) -> Option<&PrintPage> {
        self.pages.get(self.current_page as usize)
    }

    pub fn next_page(&mut self) -> bool {
        if (self.current_page as usize) < self.pages.len() - 1 {
            self.current_page += 1;
            true
        } else {
            false
        }
    }

    pub fn prev_page(&mut self) -> bool {
        if self.current_page > 0 {
            self.current_page -= 1;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintPage {
    pub page_number: u32,
    pub content_area: Rect,
    pub elements: Vec<PrintElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrintElement {
    Image {
        bounds: Rect,
        frame: u32,
        layer: LayerId,
    },
    Text {
        position: Point,
        content: String,
        font_size: f64,
    },
    CropMark {
        position: Point,
        length: f64,
        horizontal: bool,
    },
    RegistrationMark {
        position: Point,
        size: f64,
    },
    FrameNumber {
        position: Point,
        frame: u32,
    },
    LayerName {
        position: Point,
        name: String,
    },
}

pub struct PrintJobBuilder {
    job_type: PrintJobType,
    settings: PrintSettings,
}

#[derive(Debug, Clone)]
pub enum PrintJobType {
    Cel(CelPrintJob),
    Score(ScorePrintJob),
}

impl PrintJobBuilder {
    pub fn cel(name: impl Into<String>) -> Self {
        Self {
            job_type: PrintJobType::Cel(CelPrintJob::new(name)),
            settings: PrintSettings::default(),
        }
    }

    pub fn score(name: impl Into<String>, start: u32, end: u32) -> Self {
        Self {
            job_type: PrintJobType::Score(ScorePrintJob::new(name, start, end)),
            settings: PrintSettings::default(),
        }
    }

    pub fn paper_size(mut self, size: PaperSize) -> Self {
        self.settings.paper_size = size;
        self
    }

    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.settings.orientation = orientation;
        self
    }

    pub fn margins(mut self, margins: Margins) -> Self {
        self.settings.margins = margins;
        self
    }

    pub fn scale(mut self, scale: f64) -> Self {
        self.settings.scale = scale;
        self
    }

    pub fn fit_to_page(mut self, fit: bool) -> Self {
        self.settings.fit_to_page = fit;
        self
    }

    pub fn crop_marks(mut self, show: bool) -> Self {
        self.settings.crop_marks = show;
        self
    }

    pub fn frame_numbers(mut self, show: bool) -> Self {
        self.settings.frame_numbers = show;
        self
    }

    pub fn header(mut self, header: impl Into<String>) -> Self {
        self.settings.header = Some(header.into());
        self
    }

    pub fn footer(mut self, footer: impl Into<String>) -> Self {
        self.settings.footer = Some(footer.into());
        self
    }

    pub fn add_frame(mut self, frame: u32) -> Self {
        if let PrintJobType::Cel(ref mut job) = self.job_type {
            job.add_frame(frame);
        }
        self
    }

    pub fn add_layer(mut self, layer: LayerId) -> Self {
        if let PrintJobType::Cel(ref mut job) = self.job_type {
            job.add_layer(layer);
        }
        self
    }

    pub fn layout(mut self, columns: u32, rows: u32) -> Self {
        if let PrintJobType::Cel(ref mut job) = self.job_type {
            job.layout.columns = columns;
            job.layout.rows = rows;
            job.per_page = columns * rows;
        }
        self
    }

    pub fn build(self) -> (PrintJobType, PrintSettings) {
        (self.job_type, self.settings)
    }
}
