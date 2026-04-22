use iced::widget::{button, container, row, text};
use iced::{Element, Fill};
use super::{Message, ToolMessage};

pub fn view() -> Element<'static, Message> {
    let file_menu = row![
        button(text("File")).on_press(Message::NewDocument),
        button(text("Edit")).on_press(Message::Undo),
        button(text("View")).on_press(Message::Redo),
    ]
    .spacing(4);

    let tools = row![
        tool_button("Select", ToolMessage::Select),
        tool_button("Move", ToolMessage::Move),
        tool_button("Brush", ToolMessage::Brush),
        tool_button("Eraser", ToolMessage::Eraser),
        tool_button("Fill", ToolMessage::Fill),
        tool_button("Zoom", ToolMessage::Zoom),
        tool_button("Hand", ToolMessage::Hand),
        tool_button("Pen", ToolMessage::Pen),
        tool_button("Text", ToolMessage::Text),
    ]
    .spacing(2);

    let toolbar = row![
        file_menu,
        tools,
    ]
    .spacing(16);

    container(toolbar)
        .width(Fill)
        .padding(4)
        .into()
}

fn tool_button(label: &str, tool: ToolMessage) -> Element<'static, Message> {
    button(text(label.to_string()).size(12))
        .padding(iced::Padding::from([4, 8]))
        .on_press(Message::ToolSelected(tool))
        .into()
}
