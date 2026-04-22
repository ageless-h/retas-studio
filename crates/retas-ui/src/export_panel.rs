use iced::widget::{button, column, container, row, slider, text, Space, pick_list, text_input};
use iced::{Element, Length, Color, Fill, Alignment};
use super::Message;
use retas_core::advanced::render_queue::{RenderQueue, RenderJob, RenderFormat, RenderQuality, RenderStatus, FrameRange};
use std::path::PathBuf;

pub struct ExportPanelState {
    pub render_queue: RenderQueue,
    pub show_panel: bool,
    pub selected_format: RenderFormat,
    pub selected_quality: RenderQuality,
    pub frame_range: FrameRange,
    pub output_path: String,
    pub file_name_pattern: String,
}

impl Default for ExportPanelState {
    fn default() -> Self {
        Self {
            render_queue: RenderQueue::new(),
            show_panel: false,
            selected_format: RenderFormat::Png,
            selected_quality: RenderQuality::Standard,
            frame_range: FrameRange::All,
            output_path: String::from("./output"),
            file_name_pattern: String::from("frame_{:04d}.png"),
        }
    }
}

impl ExportPanelState {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn toggle_panel(&mut self) {
        self.show_panel = !self.show_panel;
    }
    
    pub fn queue_export(&mut self, doc_id: String) {
        let (start, end) = match &self.frame_range {
            FrameRange::All => (0, 143),
            FrameRange::Current => (0, 0),
            FrameRange::Custom(s, e) => (*s, *e),
            FrameRange::WorkArea => (0, 143),
        };
        
        self.render_queue.add_batch_export(
            format!("导出 {}", doc_id),
            doc_id,
            (start, end),
            PathBuf::from(&self.output_path),
            self.selected_format,
        );
    }
    
    pub fn cancel_job(&mut self, job_id: u64) {
        self.render_queue.cancel_job(job_id);
    }
    
    pub fn remove_job(&mut self, job_id: u64) {
        self.render_queue.remove_job(job_id);
    }
    
    pub fn clear_completed(&mut self) {
        self.render_queue.clear_completed();
    }
}

#[derive(Debug, Clone)]
pub enum ExportMessage {
    SetFormat(RenderFormat),
    SetQuality(RenderQuality),
    SetFrameRange(FrameRange),
    SetOutputPath(String),
    SetFilePattern(String),
    QueueExport(String),
    CancelJob(u64),
    RemoveJob(u64),
    ClearCompleted,
    StartRender,
}

pub fn view(state: &ExportPanelState) -> Element<Message> {
    if !state.show_panel {
        return container(Space::new().width(Length::Shrink).height(Length::Shrink))
            .into();
    }
    
    let header = row![
        text("导出/渲染").size(14),
        Space::new().width(Fill),
    ]
    .spacing(8)
    .align_y(Alignment::Center);
    
    let format_selector = {
        let formats = vec![
            RenderFormat::Png,
            RenderFormat::Jpeg,
            RenderFormat::Gif,
            RenderFormat::WebM,
            RenderFormat::Mp4,
        ];
        
        let format_labels: Vec<String> = formats.iter()
            .map(|f| match f {
                RenderFormat::Png => "PNG",
                RenderFormat::Jpeg => "JPEG",
                RenderFormat::Gif => "GIF",
                RenderFormat::WebM => "WebM",
                RenderFormat::Mp4 => "MP4",
                RenderFormat::APNG => "APNG",
            }.to_string())
            .collect();

        column![
            text("格式").size(11),
            pick_list(
                format_labels.clone(),
                Some(match state.selected_format {
                    RenderFormat::Png => "PNG",
                    RenderFormat::Jpeg => "JPEG",
                    RenderFormat::Gif => "GIF",
                    RenderFormat::WebM => "WebM",
                    RenderFormat::Mp4 => "MP4",
                    RenderFormat::APNG => "APNG",
                }.to_string()),
                move |s| {
                    if let Some(fmt) = formats.iter().find(|f| {
                        let label = match f {
                            RenderFormat::Png => "PNG",
                            RenderFormat::Jpeg => "JPEG",
                            RenderFormat::Gif => "GIF",
                            RenderFormat::WebM => "WebM",
                            RenderFormat::Mp4 => "MP4",
                            RenderFormat::APNG => "APNG",
                        };
                        label == s
                    }) {
                        Message::ExportChanged(ExportMessage::SetFormat(*fmt))
                    } else {
                        Message::ExportChanged(ExportMessage::SetFormat(RenderFormat::Png))
                    }
                }
            )
            .width(Fill),
        ]
        .spacing(4)
    };
    
    let quality_selector = {
        let qualities = vec![
            (RenderQuality::Draft, "草稿"),
            (RenderQuality::Standard, "标准"),
            (RenderQuality::High, "高"),
        ];
        
        let quality_buttons: Vec<Element<Message>> = qualities.iter().map(|(q, label)| {
            let is_selected = *q == state.selected_quality;
            let label_text = if is_selected {
                format!("[{}]", label)
            } else {
                label.to_string()
            };
            button(text(label_text).size(10))
                .on_press(Message::ExportChanged(ExportMessage::SetQuality(*q)))
                .into()
        }).collect();
        
        column![
            text("质量").size(11),
            row(quality_buttons).spacing(4),
        ]
        .spacing(4)
    };
    
    let frame_range_selector = {
        column![
            text("帧范围").size(11),
            row![
                button(text("全部").size(10))
                    .on_press(Message::ExportChanged(ExportMessage::SetFrameRange(FrameRange::All))),
                button(text("当前帧").size(10))
                    .on_press(Message::ExportChanged(ExportMessage::SetFrameRange(FrameRange::Current))),
            ]
            .spacing(4),
        ]
        .spacing(4)
    };
    
    let export_btn = button(text("导出当前项目").size(12))
        .on_press(Message::ExportChanged(ExportMessage::QueueExport("current".to_string())))
        .width(Fill);
    
    let render_queue = view_render_queue(state);
    
    let content = column![
        header,
        format_selector,
        quality_selector,
        frame_range_selector,
        export_btn,
        render_queue,
    ]
    .spacing(12)
    .padding(8);
    
    container(content)
        .width(Fill)
        .style(|_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::from_rgb8(40, 40, 45))),
            border: iced::Border {
                color: Color::from_rgb8(60, 60, 65),
                width: 1.0,
                radius: iced::border::Radius::new(4.0),
            },
            ..Default::default()
        })
        .into()
}

fn view_render_queue(state: &ExportPanelState) -> Element<Message> {
    let queue_header = row![
        text("渲染队列").size(12),
        Space::new().width(Fill),
        button(text("清空").size(10))
            .on_press(Message::ExportChanged(ExportMessage::ClearCompleted)),
    ]
    .spacing(8)
    .align_y(Alignment::Center);
    
    let job_items: Vec<Element<Message>> = state
        .render_queue
        .jobs
        .iter()
        .map(|job| {
            let status_text = match job.status {
                RenderStatus::Queued => "排队中".to_string(),
                RenderStatus::Rendering => format!("渲染中 {:.0}%", job.progress),
                RenderStatus::Completed => "完成".to_string(),
                RenderStatus::Failed(ref msg) => format!("失败: {}", msg),
                RenderStatus::Cancelled => "已取消".to_string(),
            };
            
            let remove_btn: Element<Message> = if job.status == RenderStatus::Queued {
                button(text("×").size(10))
                    .on_press(Message::ExportChanged(ExportMessage::RemoveJob(job.id)))
                    .width(Length::Fixed(24.0))
                    .into()
            } else {
                Space::new().width(Length::Fixed(24.0)).into()
            };
            
            let job_row = row![
                text(format!("{}: {}", job.id, job.name)).size(10),
                Space::new().width(Fill),
                text(status_text).size(10),
                remove_btn,
            ]
            .spacing(4)
            .align_y(Alignment::Center);
            
            container(job_row)
                .width(Fill)
                .padding(4)
                .style(|_theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color::from_rgb8(50, 50, 55))),
                    ..Default::default()
                })
                .into()
        })
        .collect();
    
    column![
        queue_header,
        column(job_items).spacing(2),
    ]
    .spacing(8)
    .into()
}
