//! Dialog windows for tab creation.
//!
//! Handles new tab dialog and async file loading.

use bevy::prelude::*;
use bevy::tasks::{block_on, poll_once, AsyncComputeTaskPool, Task};
use bevy_egui::{EguiContexts, egui};
use std::path::Path;

use crate::tabs::{Content, TabCreateRequest, TabMarker};
use crate::text::FileParsers;

// ============================================================================
// Resources
// ============================================================================

#[derive(Resource, Default)]
pub struct NewTabDialog {
    pub open: bool,
    pub text_input: String,
}
impl NewTabDialog {
    pub fn is_open(dialog: Res<NewTabDialog>) -> bool {
        dialog.open
    }

    pub fn update(
        mut commands: Commands,
        mut contexts: EguiContexts,
        mut dialog: ResMut<NewTabDialog>,
        mut pending_load: ResMut<PendingFileLoad>,
        file_parsers: Res<FileParsers>,
        tabs: Query<Entity, With<TabMarker>>,
    ) {
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
                    // Task is spawned here, polled separately in PendingFileLoad::poll.
                    if btn.clicked() {
                        let extensions = file_parsers.supported_extensions();
                        let task_pool = AsyncComputeTaskPool::get();
                        let task = task_pool.spawn(async move {
                            let ext_refs: Vec<&str> = extensions.iter().map(|s| s.as_str()).collect();
                            let file_handle = rfd::AsyncFileDialog::new()
                                .add_filter("Supported files", &ext_refs)
                                .pick_file()
                                .await?;
                            
                            let file_name = file_handle.file_name();
                            let bytes = file_handle.read().await;
                            
                            Some(RawFileLoad { file_name, bytes })
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
                        let parser = file_parsers.get_for_extension("txt").unwrap();
                        let parsed = parser.parse(dialog.text_input.as_bytes()).unwrap();
                        let tab_count = tabs.iter().count();
                        let name = format!("Text {}", tab_count + 1);
                        
                        commands.trigger(TabCreateRequest::new(name, Content::new(parsed.words)));
                        
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
}

/// Holds the async file-pick task spawned by the new tab dialog.
#[derive(Resource, Default)]
pub struct PendingFileLoad {
    pub task: Option<Task<Option<RawFileLoad>>>,
}
impl PendingFileLoad {
    /// Polls the async file-pick task each frame. On completion, parses the file
    /// and triggers `TabCreateRequest`.
    pub fn poll(
        mut commands: Commands,
        mut pending_load: ResMut<PendingFileLoad>,
        mut dialog: ResMut<NewTabDialog>,
        file_parsers: Res<FileParsers>,
    ) {
        let Some(task) = &mut pending_load.task else { return };
        
        if let Some(result) = block_on(poll_once(task)) {
            if let Some(raw) = result {
                let path = Path::new(&raw.file_name);
                let tab_name = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Untitled")
                    .to_string();
                
                if let Some(parser) = file_parsers.get_for_path(path) {
                    match parser.parse(&raw.bytes) {
                        Ok(parsed) => {
                            commands.trigger(
                                TabCreateRequest::new(tab_name, Content::new(parsed.words))
                                    .with_file_path(raw.file_name)
                            );
                            dialog.open = false;
                        }
                        Err(e) => {
                            warn!("Failed to parse '{}': {}", raw.file_name, e);
                        }
                    }
                } else {
                    warn!("No parser found for '{}'", raw.file_name);
                }
            }
            pending_load.task = None;
        }
    }
}

/// Raw bytes returned by the async file dialog, before parsing.
pub struct RawFileLoad {
    pub file_name: String,
    pub bytes: Vec<u8>,
}

