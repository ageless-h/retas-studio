use iced::widget::{button, container, row, text, column, slider};
use iced::{Element, Length, Color, Fill};
use super::{Message, TimelineMessage};

pub struct TimelineState {
    pub current_frame: u32,
    pub total_frames: u32,
    pub frame_rate: f32,
    pub is_playing: bool,
    pub start_frame: u32,
    pub end_frame: u32,
    pub onion_skin: bool,
}

impl Default for TimelineState {
    fn default() -> Self {
        Self {
            current_frame: 0,
            total_frames: 100,
            frame_rate: 24.0,
            is_playing: false,
            start_frame: 0,
            end_frame: 99,
            onion_skin: false,
        }
    }
}

impl TimelineState {
    pub fn next_frame(&mut self) {
        if self.current_frame < self.end_frame {
            self.current_frame += 1;
        } else {
            self.current_frame = self.start_frame;
        }
    }
    
    pub fn prev_frame(&mut self) {
        if self.current_frame > self.start_frame {
            self.current_frame -= 1;
        } else {
            self.current_frame = self.end_frame;
        }
    }
    
    pub fn go_to_frame(&mut self, frame: u32) {
        self.current_frame = frame.clamp(self.start_frame, self.end_frame);
    }
    
    pub fn toggle_play(&mut self) {
        self.is_playing = !self.is_playing;
    }
    
    pub fn stop(&mut self) {
        self.is_playing = false;
        self.current_frame = self.start_frame;
    }
    
    pub fn frame_time(&self) -> f32 {
        1.0 / self.frame_rate
    }
}

pub fn view(state: &TimelineState) -> Element<'static, Message> {
    let play_text = if state.is_playing { "暂停" } else { "播放" };
    
    let controls = row![
        button(text("起始")).on_press(Message::TimelineChanged(TimelineMessage::Stop)),
        button(text("上一帧")).on_press(Message::TimelineChanged(TimelineMessage::FrameChanged(state.current_frame.saturating_sub(1)))),
        button(text(play_text)).on_press(Message::TimelineChanged(TimelineMessage::Play)),
        button(text("下一帧")).on_press(Message::TimelineChanged(TimelineMessage::FrameChanged((state.current_frame + 1).min(state.end_frame)))),
        button(text("增加帧")).on_press(Message::TimelineChanged(TimelineMessage::AddFrame)),
        button(text("删除帧")).on_press(Message::TimelineChanged(TimelineMessage::DeleteFrame)),
    ]
    .spacing(2);

    let frame_display = text(format!(
        "帧: {} / {}  |  {} 帧/秒",
        state.current_frame + 1,
        state.total_frames,
        state.frame_rate
    ))
    .size(12);

    let frame_slider = slider(
        0.0..=state.end_frame as f32,
        state.current_frame as f32,
        |v| Message::TimelineChanged(TimelineMessage::FrameChanged(v as u32))
    );

    let timeline_track = container(
        column![
            frame_slider,
            text(format!("范围: {} - {}", state.start_frame, state.end_frame)).size(11),
        ]
        .spacing(4)
    )
    .width(Fill)
    .height(Length::Fixed(50.0));

    let layout = row![
        controls,
        frame_display,
        timeline_track,
    ]
    .spacing(12)
    .align_y(iced::Alignment::Center);

    container(layout)
        .width(Fill)
        .height(Length::Fixed(80.0))
        .padding(8)
        .style(timeline_style)
        .into()
}

fn timeline_style(_: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(iced::Background::Color(Color::from_rgb8(30, 30, 30))),
        border: iced::Border {
            color: Color::from_rgb8(60, 60, 60),
            width: 1.0,
            radius: iced::border::Radius::default(),
        },
        ..Default::default()
    }
}
