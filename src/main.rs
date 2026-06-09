use clap::Parser;
use iced::widget::{container, markdown, row, scrollable, text_editor};
use iced::{Element, Length, Task, Theme};
use std::fs;

#[derive(Parser, Debug)]
#[command(name = "lean-md")]
#[command(about = "A simple Markdown viewer", long_about = None)]
struct Args {
    /// Path to a markdown file to open
    #[arg(name = "FILE")]
    file: Option<String>,
}

pub fn main() -> iced::Result {
    let args = Args::parse();

    iced::application(
        move || MarkdownViewer::new(args.file.clone()),
        MarkdownViewer::update,
        MarkdownViewer::view,
    )
    .theme(Theme::TokyoNight)
    .run()
}

struct MarkdownViewer {
    raw: text_editor::Content,
    parsed: Vec<markdown::Item>,
}

impl MarkdownViewer {
    fn new(file_path: Option<String>) -> (Self, Task<Message>) {
        let initial_content = if let Some(path) = file_path {
            match fs::read_to_string(&path) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("Error reading file '{}': {}", path, e);
                    String::new()
                }
            }
        } else {
            r#"# Markdown Viewer

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
"#
            .to_string()
        };

        let raw = text_editor::Content::with_text(&initial_content);
        let parsed = markdown::parse(&raw.text()).collect();

        (Self { raw, parsed }, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Edit(action) => {
                self.raw.perform(action);
                // Re-parse markdown when content changes
                self.parsed = markdown::parse(&self.raw.text()).collect();
                Task::none()
            }
            Message::LinkClicked(_url) => {
                // Handle link clicks if needed
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

        let preview = markdown::view(&self.parsed, Theme::TokyoNight).map(Message::LinkClicked);

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
    LinkClicked(String),
}
