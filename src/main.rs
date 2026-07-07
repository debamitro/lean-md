use clap::Parser;
use iced::widget::{button, column, container, markdown, row, scrollable, space, text, text_editor};
use iced::{Element, Length, Task, Theme};
use std::fs;

#[derive(Parser, Debug)]
#[command(name = "lean-md")]
#[command(about = "A simple Markdown viewer", long_about = None)]
struct Args {
    /// Path to a markdown file to open
    #[arg(name = "FILE")]
    file: Option<String>,
    
    /// Start in read-only mode (hide editor)
    #[arg(long = "readonly", short = 'R')]
    readonly: bool,
}

pub fn main() -> iced::Result {
    let args = Args::parse();

    iced::application(
        move || MarkdownViewer::new(args.file.clone(), args.readonly),
        MarkdownViewer::update,
        MarkdownViewer::view,
    )
    .theme(Theme::TokyoNight)
    .run()
}

struct MarkdownViewer {
    raw: text_editor::Content,
    parsed: Vec<markdown::Item>,
    file_path: Option<String>,
    editor_visible: bool,
    readonly: bool,
}

impl MarkdownViewer {
    fn new(file_path: Option<String>, readonly: bool) -> (Self, Task<Message>) {
        let initial_content = if let Some(path) = &file_path {
            match fs::read_to_string(path) {
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

        (
            Self {
                raw,
                parsed,
                file_path,
                editor_visible: !readonly,
                readonly,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Edit(action) => {
                self.raw.perform(action);
                self.parsed = markdown::parse(&self.raw.text()).collect();
                Task::none()
            }
            Message::LinkClicked(_url) => Task::none(),
            Message::ToggleEditor => {
                self.editor_visible = !self.editor_visible;
                Task::none()
            }
            Message::Open => {
                Task::perform(open_file_dialog(), Message::OpenPath)
            }
            Message::OpenPath(Ok(path)) => {
                match fs::read_to_string(&path) {
                    Ok(content) => {
                        self.raw = text_editor::Content::with_text(&content);
                        self.parsed = markdown::parse(&self.raw.text()).collect();
                        self.file_path = Some(path);
                    }
                    Err(e) => {
                        eprintln!("Error reading file '{}': {}", path, e);
                    }
                }
                Task::none()
            }
            Message::OpenPath(Err(_)) => Task::none(),
            Message::Save => self.save(),
            Message::SaveAs => {
                if let Some(path) = &self.file_path {
                    Task::perform(
                        save_file_dialog(Some(path.clone())),
                        Message::SaveAsPath,
                    )
                } else {
                    Task::perform(save_file_dialog(None), Message::SaveAsPath)
                }
            }
            Message::SaveAsPath(Ok(path)) => {
                self.file_path = Some(path.clone());
                let content = self.raw.text();
                Task::perform(
                    async move { std::fs::write(&path, content).map_err(|e| e.to_string()) },
                    Message::FileSaved,
                )
            }
            Message::SaveAsPath(Err(_)) => Task::none(),
            Message::FileSaved(Ok(())) => Task::none(),
            Message::FileSaved(Err(e)) => {
                eprintln!("Failed to save file: {}", e);
                Task::none()
            }
        }
    }

    fn save(&self) -> Task<Message> {
        if let Some(path) = self.file_path.clone() {
            let content = self.raw.text();
            Task::perform(
                async move { std::fs::write(&path, content).map_err(|e| e.to_string()) },
                Message::FileSaved,
            )
        } else {
            Task::perform(save_file_dialog(None), Message::SaveAsPath)
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let toolbar = if self.readonly {
            row![
                button("Open")
                    .on_press(Message::Open),
                space().width(Length::Fill)
            ]
                .spacing(10)
                .padding(10)
        } else {
            row![
                button("Open")
                    .on_press(Message::Open),
                button("Save")
                    .on_press(Message::Save),
                button("Save As...")
                    .on_press(Message::SaveAs),
                space().width(Length::Fill),
                button(if self.editor_visible {
                    "Hide Editor"
                } else {
                    "Show Editor"
                })
                .on_press(Message::ToggleEditor),
            ]
            .spacing(10)
            .padding(10)
        };

        let editor = if self.readonly {
            container(
                text("Read-only mode - editing disabled")
                    .size(14)
            )
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .padding(10)
            
        } else {
            container(
                text_editor(&self.raw)
                    .placeholder("Type your Markdown here...")
                    .on_action(Message::Edit)
                    .height(Length::Fill)
                    .padding(10),
            )
            .width(if self.editor_visible {
                Length::FillPortion(1)
            } else {
                Length::Shrink
            })
            .height(if self.editor_visible {
                Length::Fill
            } else {
                Length::Shrink
            })
            .padding(10)
        };

        let preview = container(scrollable(
            markdown::view(&self.parsed, Theme::TokyoNight).map(Message::LinkClicked),
        ))
        .width(if self.editor_visible {
            Length::FillPortion(1)
        } else {
            Length::Fill
        })
        .height(Length::Fill)
        .padding(10);

        let content = if self.readonly {
            row![preview]
        } else if self.editor_visible {
            row![editor, preview].spacing(10)
        } else {
            row![preview]
        };

        column![
            toolbar,
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(10),
        ]
        .spacing(10)
        .into()
    }
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    LinkClicked(String),
    ToggleEditor,
    Open,
    OpenPath(Result<String, ()>),
    Save,
    SaveAs,
    SaveAsPath(Result<String, ()>),
    FileSaved(Result<(), String>),
}

async fn save_file_dialog(suggested_name: Option<String>) -> Result<String, ()> {
    let mut dialog = rfd::AsyncFileDialog::new()
        .add_filter("Markdown", &["md"])
        .set_title("Save Markdown File");

    if let Some(name) = suggested_name {
        dialog = dialog.set_file_name(name);
    }

    dialog
        .save_file()
        .await
        .map(|handle| handle.path().to_string_lossy().to_string())
        .ok_or(())
}

async fn open_file_dialog() -> Result<String, ()> {
    rfd::AsyncFileDialog::new()
        .add_filter("Markdown", &["md"])
        .set_title("Open Markdown File")
        .pick_file()
        .await
        .map(|handle| handle.path().to_string_lossy().to_string())
        .ok_or(())
}