# Detailed Implementation Plan for lean-md Features

## Overview
This plan outlines the implementation of three new features for the lean-md markdown viewer/editor:
1. Save functionality (button/menu item to write file to disk)
2. Toggle editor visibility (button/menu item to hide/show editor)
3. Read-only mode via `--readonly` CLI flag

## Current State Analysis

### Codebase Structure
- **Main Application**: `src/main.rs` (114 lines)
- **Framework**: Iced 0.14.0 with markdown and highlighter features
- **CLI**: Clap 4.5 with derive feature
- **Current Features**:
  - Split-pane layout: editor (left) + preview (right)
  - Real-time markdown parsing
  - File opening via CLI argument
  - TokyoNight theme

### Key Components
1. **Args struct**: Currently only accepts optional FILE argument
2. **MarkdownViewer struct**: Contains raw content and parsed markdown
3. **Message enum**: Handles Edit actions and LinkClicked events
4. **Application State**: No state tracking for UI configuration

## Feature 1: Save Functionality

### Requirements
- Add a save button or menu item
- Write current editor content to disk
- Handle file path scenarios:
  - File was opened via CLI argument → save to same path
  - No file was opened → prompt for save location

### Implementation Steps

#### Step 1.1: Add Dependencies
Add `rfd` (Rust File Dialog) crate to `Cargo.toml`:
```toml
rfd = "0.14"
```

#### Step 1.2: Track File Path in State
Modify `MarkdownViewer` struct to include current file path:
```rust
struct MarkdownViewer {
    raw: text_editor::Content,
    parsed: Vec<markdown::Item>,
    file_path: Option<String>,  // New field
}
```

#### Step 1.3: Add Save-Related Messages
Extend `Message` enum:
```rust
enum Message {
    Edit(text_editor::Action),
    LinkClicked(String),
    Save,                    // New: Trigger save
    SaveAs,                  // New: Trigger save with new path
    FileSaved(Result<(), std::io::Error>),  // New: Handle save result
}
```

#### Step 1.4: Implement Save Logic
Add save methods to `MarkdownViewer`:
```rust
impl MarkdownViewer {
    fn save(&self) -> Task<Message> {
        if let Some(path) = &self.file_path {
            Task::perform(
                std::fs::write(path, self.raw.text()),
                Message::FileSaved
            )
        } else {
            Task::perform(self.save_as_dialog(), |result| {
                match result {
                    Ok(path) => Message::FileSaved(std::fs::write(&path, self.raw.text())),
                    Err(_) => Message::FileSaved(Err(std::io::Error::new(
                        std::io::ErrorKind::Other, "No file selected"
                    ))),
                }
            })
        }
    }
    
    fn save_as_dialog(&self) -> impl Future<Output = Result<String, ()>> {
        async move {
            rfd::AsyncFileDialog::new()
                .add_filter("Markdown", &["md"])
                .set_title("Save Markdown File")
                .save_file()
                .await
                .map(|handle| handle.path().to_string_lossy().to_string())
                .ok_or(())
        }
    }
}
```

#### Step 1.5: Add Save Button to UI
Modify `view()` method to include a toolbar with save button:
```rust
use iced::widget::{button, horizontal_space};

fn view(&self) -> Element<'_, Message> {
    let toolbar = row![
        button("Save")
            .on_press(Message::Save),
        button("Save As...")
            .on_press(Message::SaveAs),
        horizontal_space(Length::Fill),
        // Future: Toggle editor button
    ]
    .spacing(10)
    .padding(10);
    
    // Rest of existing view code...
}
```

#### Step 1.6: Handle Save Results
Update `update()` method to handle save results:
```rust
fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::Edit(action) => {
            self.raw.perform(action);
            self.parsed = markdown::parse(&self.raw.text()).collect();
            Task::none()
        }
        Message::LinkClicked(_url) => Task::none(),
        Message::Save => self.save(),
        Message::SaveAs => {
            Task::perform(self.save_as_dialog(), |result| {
                match result {
                    Ok(path) => {
                        // Update file path and save
                        Message::FileSaved(std::fs::write(&path, self.raw.text()))
                    }
                    Err(_) => Message::FileSaved(Err(std::io::Error::new(
                        std::io::ErrorKind::Other, "Save cancelled"
                    ))),
                }
            })
        }
        Message::FileSaved(Ok(())) => {
            // Optionally show success indicator
            Task::none()
        }
        Message::FileSaved(Err(e)) => {
            eprintln!("Failed to save file: {}", e);
            Task::none()
        }
    }
}
```

## Feature 2: Toggle Editor Visibility

### Requirements
- Add a button or menu item to hide/show the editor
- When hidden, show only the preview pane
- Maintain editor state while hidden

### Implementation Steps

#### Step 2.1: Add Editor Visibility State
Add visibility tracking to `MarkdownViewer`:
```rust
struct MarkdownViewer {
    raw: text_editor::Content,
    parsed: Vec<markdown::Item>,
    file_path: Option<String>,
    editor_visible: bool,  // New field
}
```

#### Step 2.2: Add Toggle Message
Extend `Message` enum:
```rust
enum Message {
    Edit(text_editor::Action),
    LinkClicked(String),
    Save,
    SaveAs,
    FileSaved(Result<(), std::io::Error>),
    ToggleEditor,  // New: Toggle editor visibility
}
```

#### Step 2.3: Implement Toggle Logic
Update `update()` method:
```rust
Message::ToggleEditor => {
    self.editor_visible = !self.editor_visible;
    Task::none()
}
```

#### Step 2.4: Update UI Layout
Modify `view()` method to conditionally show editor:
```rust
fn view(&self) -> Element<'_, Message> {
    let toolbar = row![
        button("Save")
            .on_press(Message::Save),
        button("Save As...")
            .on_press(Message::SaveAs),
        horizontal_space(Length::Fill),
        button(if self.editor_visible { 
            "Hide Editor" 
        } else { 
            "Show Editor" 
        })
        .on_press(Message::ToggleEditor),
    ]
    .spacing(10)
    .padding(10);
    
    let editor = container(
        text_editor(&self.raw)
            .placeholder("Type your Markdown here...")
            .on_action(Message::Edit)
            .height(Length::Fill)
            .padding(10)
    )
    .width(if self.editor_visible {
        Length::FillPortion(1)
    } else {
        Length::Shrink
    })
    .height(Length::Fill)
    .padding(10);
    
    let preview = container(scrollable(
        markdown::view(&self.parsed, Theme::TokyoNight)
            .map(Message::LinkClicked)
    ))
    .width(if self.editor_visible {
        Length::FillPortion(1)
    } else {
        Length::Fill
    })
    .height(Length::Fill)
    .padding(10);
    
    let content = if self.editor_visible {
        row![editor, preview].spacing(10)
    } else {
        row![preview]
    };
    
    column![
        toolbar,
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
    ]
    .spacing(10)
    .into()
}
```

#### Step 2.5: Update Constructor
Initialize `editor_visible` in `new()`:
```rust
fn new(file_path: Option<String>) -> (Self, Task<Message>) {
    // ... existing code ...
    
    (
        Self { 
            raw, 
            parsed, 
            file_path,
            editor_visible: true,  // Default to visible
        }, 
        Task::none()
    )
}
```

## Feature 3: Read-Only Mode via CLI Flag

### Requirements
- Add `--readonly` CLI flag
- When flag is present, start with editor hidden
- Optionally disable editing functionality entirely

### Implementation Steps

#### Step 3.1: Add CLI Argument
Extend `Args` struct:
```rust
#[derive(Parser, Debug)]
#[command(name = "lean-md")]
#[command(about = "A simple Markdown viewer", long_about = None)]
struct Args {
    /// Path to a markdown file to open
    #[arg(name = "FILE")]
    file: Option<String>,
    
    /// Start in read-only mode (hide editor)
    #[arg(long = "readonly")]
    readonly: bool,  // New field
}
```

#### Step 3.2: Pass Readonly State to Application
Modify `main()` function:
```rust
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
```

#### Step 3.3: Update Constructor Signature
Modify `new()` to accept readonly parameter:
```rust
fn new(file_path: Option<String>, readonly: bool) -> (Self, Task<Message>) {
    // ... existing code ...
    
    (
        Self { 
            raw, 
            parsed, 
            file_path,
            editor_visible: !readonly,  // Hidden if readonly
        }, 
        Task::none()
    )
}
```

#### Step 3.4: Optional - Add Read-Only State
For more robust read-only mode, add explicit tracking:
```rust
struct MarkdownViewer {
    raw: text_editor::Content,
    parsed: Vec<markdown::Item>,
    file_path: Option<String>,
    editor_visible: bool,
    readonly: bool,  // New field
}

fn new(file_path: Option<String>, readonly: bool) -> (Self, Task<Message>) {
    // ... existing code ...
    
    (
        Self { 
            raw, 
            parsed, 
            file_path,
            editor_visible: !readonly,
            readonly,
        }, 
        Task::none()
    )
}
```

#### Step 3.5: Update UI for Read-Only Mode
Modify view to disable toggle button in readonly mode:
```rust
let toggle_button = if self.readonly {
    button("Show Editor")  // Readonly can show editor but not hide
        .on_press(Message::ToggleEditor)
} else {
    button(if self.editor_visible { 
        "Hide Editor" 
    } else { 
        "Show Editor" 
    })
    .on_press(Message::ToggleEditor)
};
```

Alternatively, completely disable editing:
```rust
let editor = if self.readonly {
    container(text("Read-only mode"))
        .width(Length::FillPortion(1))
        .height(Length::Fill)
        .padding(10)
} else {
    container(
        text_editor(&self.raw)
            .placeholder("Type your Markdown here...")
            .on_action(Message::Edit)
            .height(Length::Fill)
            .padding(10)
    )
    .width(Length::FillPortion(1))
    .height(Length::Fill)
    .padding(10)
};
```

## Implementation Order

### Phase 1: Foundation (Prerequisites)
1. Add `rfd` dependency to `Cargo.toml`
2. Add basic state fields to `MarkdownViewer` struct
3. Update constructor signature

### Phase 2: Save Functionality
4. Implement save messages
5. Add save logic methods
6. Create toolbar UI with save buttons
7. Handle save results in update method

### Phase 3: Editor Toggle
8. Add editor visibility state
9. Implement toggle message handling
10. Update view method for conditional layout

### Phase 4: Read-Only Mode
11. Add CLI argument
12. Update main function to pass readonly flag
13. Update constructor to handle readonly initialization
14. (Optional) Add readonly-specific UI behavior

### Phase 5: Testing & Refinement
15. Test each feature independently
16. Test feature combinations
17. Error handling improvements
18. UI polish and responsive design

## Testing Strategy

### Unit Tests
- File save functionality
- State transitions
- CLI argument parsing

### Integration Tests
- Open file → modify → save workflow
- Toggle editor visibility
- Read-only mode startup

### Manual Testing
```bash
# Test normal mode
cargo run -- file.md

# Test read-only mode
cargo run -- --readonly file.md

# Test no file (should use default content)
cargo run

# Test read-only with no file
cargo run -- --readonly
```

## Potential Enhancements (Future Considerations)

1. **Keyboard Shortcuts**
   - Ctrl+S for save
   - Ctrl+Shift+S for save as
   - Ctrl+E for toggle editor

2. **Status Bar**
   - Show current file path
   - Show save status
   - Show mode (editor/viewer)

3. **Auto-Save**
   - Optional auto-save on changes
   - Configurable interval

4. **Recent Files**
   - Track recently opened files
   - Quick access menu

5. **File Watching**
   - Reload file if changed externally
   - Prompt on external changes

## Error Handling Considerations

1. **File I/O Errors**
   - Permission denied
   - Disk full
   - Invalid path

2. **Dialog Errors**
   - Dialog cancelled
   - Platform-specific issues

3. **State Errors**
   - Inconsistent state
   - Missing file path

## Dependencies Summary

### Current Dependencies
- `iced = { version = "0.14.0", features = ["markdown", "highlighter"] }`
- `clap = { version = "4.5", features = ["derive"] }`

### New Dependencies Required
- `rfd = "0.14"` (for file dialogs)

### Optional Future Dependencies
- `notify = "6.1"` (for file watching)
- `serde = { version = "1.0", features = ["derive"] }` (for config persistence)

## Code Quality Notes

1. **Maintain Existing Patterns**
   - Follow Iced's functional architecture
   - Use Task for async operations
   - Keep Message enum comprehensive

2. **Error Handling**
   - Use Result types for fallible operations
   - Provide user feedback for errors
   - Log errors appropriately

3. **State Management**
   - Keep state minimal and focused
   - Ensure consistent state transitions
   - Document state invariants

## Estimated Implementation Time

- **Phase 1 (Foundation)**: 30 minutes
- **Phase 2 (Save Functionality)**: 1-2 hours
- **Phase 3 (Editor Toggle)**: 30-45 minutes
- **Phase 4 (Read-Only Mode)**: 30 minutes
- **Phase 5 (Testing & Refinement)**: 1 hour

**Total Estimated Time**: 3-5 hours

## Conclusion

This plan provides a comprehensive roadmap for implementing all three requested features while maintaining code quality and following Rust/Iced best practices. The implementation is structured to be incremental, allowing each feature to be tested independently before moving to the next.