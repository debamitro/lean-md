use iced::widget::{column, container, row, scrollable, text, text_editor};
use iced::{Element, Length, Task, Theme};

pub fn main() -> iced::Result {
    iced::application(MarkdownViewer::new, MarkdownViewer::update, MarkdownViewer::view)
        .theme(Theme::TokyoNight)
        .run()
}

struct MarkdownViewer {
    raw: text_editor::Content,
}

impl MarkdownViewer {
    fn new() -> (Self, Task<Message>) {
        const INITIAL_CONTENT: &str = r#"# Markdown Viewer

Welcome to the Markdown Viewer!

## Features

- **Bold text** and *italic text*
- Lists like this one
- `Code` and code blocks

## Try it out!

Type your own markdown below.

1. First item
2. Second item
3. Third item
"#;

        (
            Self {
                raw: text_editor::Content::with_text(INITIAL_CONTENT),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Edit(action) => {
                self.raw.perform(action);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let editor = text_editor(&self.raw)
            .placeholder("Type your Markdown here...")
            .on_action(Message::Edit)
            .height(Length::Fill)
            .padding(10);

        let content = self.raw.text().to_string();
        let preview = render_markdown(&content);

        row![
            container(editor)
                .width(Length::FillPortion(1))
                .height(Length::Fill)
                .padding(10),
            container(scrollable(preview))
                .width(Length::FillPortion(1))
                .height(Length::Fill)
                .padding(10)
        ]
        .spacing(10)
        .padding(10)
        .into()
    }
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
}

fn render_markdown(content: &str) -> Element<'static, Message> {
    let mut widgets = Vec::new();

    for line in content.lines() {
        let line_owned = line.to_string();
        let widget: Element<Message> = if line.starts_with("# ") {
            text(line_owned[2..].to_string()).size(32).into()
        } else if line.starts_with("## ") {
            text(line_owned[3..].to_string()).size(24).into()
        } else if line.starts_with("### ") {
            text(line_owned[4..].to_string()).size(20).into()
        } else if line.starts_with("- ") {
            text(line_owned).into()
        } else if line.starts_with("1. ") || line.starts_with("2. ") || line.starts_with("3. ") {
            text(line_owned).into()
        } else if line.starts_with("```") {
            text(String::new()).into()
        } else if !line.is_empty() {
            text(line_owned).into()
        } else {
            text(String::new()).into()
        };
        widgets.push(widget);
    }

    column(widgets).spacing(5).into()
}
