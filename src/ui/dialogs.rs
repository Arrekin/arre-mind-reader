//! Dialog windows for tab creation.
//!
//! Handles new tab dialog and async file loading.

use bevy::prelude::*;
use bevy::tasks::{block_on, poll_once, AsyncComputeTaskPool, Task};
use bevy_egui::{EguiContexts, egui};
use std::path::PathBuf;

use crate::tabs::{TabCreate, TabMarker};
use crate::text::{TxtParser, TextParser, Word};

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
    pub path: PathBuf,
    pub name: String,
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
                if btn.clicked() {
                    let task_pool = AsyncComputeTaskPool::get();
                    let task = task_pool.spawn(async move {
                        let file_handle = rfd::AsyncFileDialog::new()
                            .add_filter("Text files", &["txt"])
                            .pick_file()
                            .await?;
                        
                        let path = file_handle.path().to_path_buf();
                        let content = std::fs::read_to_string(&path).ok()?;
                        let name = path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Untitled")
                            .to_string();
                        let words = TxtParser.parse(&content);
                        
                        Some(FileLoadResult { path, name, words })
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
                    let words = TxtParser.parse(&dialog.text_input);
                    let tab_count = tabs.iter().count();
                    let name = format!("Text {}", tab_count + 1);
                    
                    commands.trigger(TabCreate {
                        name,
                        file_path: None,
                        words,
                    });
                    
                    dialog.open = false;
                    dialog.text_input.clear();
                }
                
                if ui.add_enabled(!is_loading, egui::Button::new("Cancel")).clicked() {
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
            commands.trigger(TabCreate {
                name: file_result.name,
                file_path: Some(file_result.path),
                words: file_result.words,
            });
            dialog.open = false;
        }
        pending_load.task = None;
    }
}
