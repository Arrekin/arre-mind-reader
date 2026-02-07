//! Dialog windows for tab creation.
//!
//! Handles new tab dialog and async file loading.

use bevy::prelude::*;
use bevy::tasks::{block_on, poll_once, AsyncComputeTaskPool, Task};
use bevy_egui::{EguiContexts, egui};
use std::path::Path;

use crate::tabs::{TabCreateRequest, TabMarker};
use crate::text::{TxtParser, TextParser, Word, get_parser_for_path};

// ============================================================================
// Resources
// ============================================================================

#[derive(Resource, Default)]
pub struct NewTabDialog {
    pub open: bool,
    pub text_input: String,
}

#[derive(Resource, Default)]
pub struct PendingFileLoad {
    pub task: Option<Task<Option<FileLoadResult>>>,
}

pub struct FileLoadResult {
    pub file_name: String,
    pub tab_name: String,
    pub words: Vec<Word>,
}

// ============================================================================
// Systems
// ============================================================================

pub fn new_tab_dialog_system(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut dialog: ResMut<NewTabDialog>,
    mut pending_load: ResMut<PendingFileLoad>,
    tabs: Query<Entity, With<TabMarker>>,
) {
    if !dialog.open {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    
    let is_loading = pending_load.task.is_some();
    
    egui::Window::new("New Tab")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let btn = ui.add_enabled(!is_loading, egui::Button::new("📂 Load from File"));
                // File dialog is async to avoid blocking the main thread.
                // Task is spawned here, polled separately in poll_file_load_task.
                if btn.clicked() {
                    let task_pool = AsyncComputeTaskPool::get();
                    let task = task_pool.spawn(async move {
                        let file_handle = rfd::AsyncFileDialog::new()
                            .add_filter("Text files", &["txt"])
                            .pick_file()
                            .await?;
                        
                        let file_name = file_handle.file_name();
                        let bytes = file_handle.read().await;
                        let content = String::from_utf8(bytes).ok()?;
                        let parser = get_parser_for_path(Path::new(&file_name))?;
                        let tab_name = Path::new(&file_name)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Untitled")
                            .to_string();
                        let words = parser.parse(&content);
                        
                        Some(FileLoadResult { file_name, tab_name, words })
                    });
                    pending_load.task = Some(task);
                }
                
                if is_loading {
                    ui.spinner();
                    ui.label("Loading...");
                }
            });
            
            ui.separator();
            ui.label("Or paste text below:");
            
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    ui.add_enabled(
                        !is_loading,
                        egui::TextEdit::multiline(&mut dialog.text_input)
                            .desired_width(400.0)
                            .desired_rows(10)
                            .hint_text("Paste your text here...")
                    );
                });
            
            ui.horizontal(|ui| {
                let can_create = !dialog.text_input.trim().is_empty() && !is_loading;
                if ui.add_enabled(can_create, egui::Button::new("Create Tab")).clicked() {
                    // Pasted text has no file path, so we always use TxtParser
                    let words = TxtParser.parse(&dialog.text_input);
                    let tab_count = tabs.iter().count();
                    let name = format!("Text {}", tab_count + 1);
                    
                    commands.trigger(TabCreateRequest::new(name, words));
                    
                    dialog.open = false;
                    dialog.text_input.clear();
                }
                
                if ui.button("Cancel").clicked() {
                    pending_load.task = None;
                    dialog.open = false;
                    dialog.text_input.clear();
                }
            });
        });
}

pub fn poll_file_load_task(
    mut commands: Commands,
    mut pending_load: ResMut<PendingFileLoad>,
    mut dialog: ResMut<NewTabDialog>,
) {
    let Some(task) = &mut pending_load.task else { return };
    
    if let Some(result) = block_on(poll_once(task)) {
        if let Some(file_result) = result {
            commands.trigger(TabCreateRequest::new(file_result.tab_name, file_result.words).with_file_path(file_result.file_name));
            dialog.open = false;
        }
        pending_load.task = None;
    }
}
