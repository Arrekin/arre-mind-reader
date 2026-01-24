//! UI systems using bevy_egui.
//!
//! Provides tab bar, playback controls, settings panel, and the new tab dialog.
//! Uses async file loading to prevent UI freezes.

use bevy::prelude::*;
use bevy::tasks::{block_on, poll_once, AsyncComputeTaskPool, Task};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use std::path::PathBuf;

use crate::fonts::AvailableFonts;
use crate::text_parser::parse_text;
use crate::state::constants::{WPM_MAX, WPM_MIN, WPM_STEP};
use crate::state::{ReaderSettings, ReaderState, ReadingState, TabId, TabManager, Word};

#[derive(Resource, Default)]
pub struct NewTabDialog {
    pub open: bool,
    pub text_input: String,
}

pub struct FileLoadResult {
    pub path: PathBuf,
    pub name: String,
    pub words: Vec<Word>,
}

#[derive(Resource, Default)]
pub struct PendingFileLoad {
    pub task: Option<Task<Option<FileLoadResult>>>,
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NewTabDialog>()
            .init_resource::<PendingFileLoad>()
            .add_systems(Update, (sync_tab_to_reader, poll_file_load_task))
            .add_systems(EguiPrimaryContextPass, (tab_bar_system, controls_system, new_tab_dialog_system));
    }
}

fn tab_bar_system(
    mut contexts: EguiContexts,
    mut tabs: ResMut<TabManager>,
    mut dialog: ResMut<NewTabDialog>,
    mut next_state: ResMut<NextState<ReadingState>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    
    let mut tab_to_close: Option<TabId> = None;
    let mut tab_to_select: Option<TabId> = None;
    let mut open_dialog = false;
    
    let active_id = tabs.active_id();
    let tab_info: Vec<(TabId, String, bool)> = tabs.tabs().iter()
        .map(|tab| (tab.id, tab.name.clone(), active_id == Some(tab.id)))
        .collect();
    
    egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
        ui.horizontal(|ui| {
            for (id, name, is_active) in &tab_info {
                let label = if *is_active {
                    egui::RichText::new(name).strong()
                } else {
                    egui::RichText::new(name)
                };
                
                ui.horizontal(|ui| {
                    if ui.selectable_label(*is_active, label).clicked() {
                        tab_to_select = Some(*id);
                    }
                    if ui.small_button("×").clicked() {
                        tab_to_close = Some(*id);
                    }
                });
                ui.separator();
            }
            
            if ui.button("+ New").clicked() {
                open_dialog = true;
            }
        });
    });
    
    // Apply mutations after UI
    if let Some(id) = tab_to_select {
        tabs.set_active(id);
        next_state.set(ReadingState::Idle);
    }
    if let Some(id) = tab_to_close {
        tabs.close_tab(id);
        next_state.set(ReadingState::Idle);
    }
    if open_dialog {
        dialog.open = true;
        dialog.text_input.clear();
    }
}

fn controls_system(
    mut contexts: EguiContexts,
    reader_state: Res<ReaderState>,
    mut settings: ResMut<ReaderSettings>,
    current_state: Res<State<ReadingState>>,
    mut next_state: ResMut<NextState<ReadingState>>,
    tabs: Res<TabManager>,
    fonts: Res<AvailableFonts>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    
    egui::TopBottomPanel::bottom("controls").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // Play/Pause button (only if we have a tab)
            if let Some(tab) = tabs.active_tab() {
                let btn_text = match current_state.get() {
                    ReadingState::Active => "⏸ Pause",
                    _ => "▶ Play",
                };
                if ui.button(btn_text).clicked() {
                    match current_state.get() {
                        ReadingState::Active => next_state.set(ReadingState::Paused),
                        _ => next_state.set(ReadingState::Active),
                    }
                }
                
                // Progress
                let total = tab.words.len();
                let current = reader_state.current_index + 1;
                ui.label(format!("{}/{}", current, total));
                
                // Progress bar
                let progress = if total > 0 { current as f32 / total as f32 } else { 0.0 };
                ui.add(egui::ProgressBar::new(progress).desired_width(200.0));
                
                ui.separator();
            } else {
                ui.label("No tab open. Click '+ New' to add text.");
            }
            
            // WPM slider
            ui.label("WPM:");
            let mut wpm = settings.wpm;
            if ui.add(egui::Slider::new(&mut wpm, WPM_MIN..=WPM_MAX).step_by(WPM_STEP as f64)).changed() {
                settings.wpm = wpm;
            }
            
            ui.separator();
            
            // Font selector
            ui.label("Font:");
            let current_font = settings.font_path.split('/').last().unwrap_or(&settings.font_path);
            egui::ComboBox::from_id_salt("font_selector")
                .selected_text(current_font)
                .show_ui(ui, |ui| {
                    for font_path in &fonts.fonts {
                        let display_name = font_path.split('/').last().unwrap_or(font_path);
                        if ui.selectable_label(settings.font_path == *font_path, display_name).clicked() {
                            settings.font_path = font_path.clone();
                        }
                    }
                });
            
            ui.separator();
            
            // State indicator
            let state_text = match current_state.get() {
                ReadingState::Idle => "Idle",
                ReadingState::Active => "Reading",
                ReadingState::Paused => "Paused",
            };
            ui.label(format!("[{}]", state_text));
        });
    });
}

fn new_tab_dialog_system(
    mut contexts: EguiContexts,
    mut dialog: ResMut<NewTabDialog>,
    mut tabs: ResMut<TabManager>,
    mut next_state: ResMut<NextState<ReadingState>>,
    mut pending_load: ResMut<PendingFileLoad>,
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
                        let words = crate::text_parser::parse_text(&content);
                        
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
                    let words = parse_text(&dialog.text_input);
                    let name = format!("Text {}", tabs.tabs().len() + 1);
                    tabs.add_tab(name, None, words);
                    dialog.open = false;
                    dialog.text_input.clear();
                    next_state.set(ReadingState::Idle);
                }
                
                if ui.add_enabled(!is_loading, egui::Button::new("Cancel")).clicked() {
                    dialog.open = false;
                    dialog.text_input.clear();
                }
            });
        });
}

fn poll_file_load_task(
    mut pending_load: ResMut<PendingFileLoad>,
    mut tabs: ResMut<TabManager>,
    mut dialog: ResMut<NewTabDialog>,
    mut next_state: ResMut<NextState<ReadingState>>,
) {
    let Some(task) = &mut pending_load.task else { return };
    
    if let Some(result) = block_on(poll_once(task)) {
        if let Some(file_result) = result {
            tabs.add_tab(file_result.name, Some(file_result.path), file_result.words);
            dialog.open = false;
            next_state.set(ReadingState::Idle);
        }
        pending_load.task = None;
    }
}

fn sync_tab_to_reader(
    mut tabs: ResMut<TabManager>,
    mut reader_state: ResMut<ReaderState>,
    current_state: Res<State<ReadingState>>,
) {
    let tab_changed = tabs.active_id() != tabs.last_synced_id();
    
    // Sync current_index from reader back to tab when reading (only if tab hasn't changed)
    if !tab_changed && *current_state.get() != ReadingState::Idle {
        if let Some(tab) = tabs.active_tab_mut() {
            tab.current_index = reader_state.current_index;
        }
    }
    
    // Sync current_index from tab to reader when tab changes
    if tab_changed {
        let active_id = tabs.active_id();
        tabs.set_last_synced(active_id);
        if let Some(tab) = tabs.active_tab() {
            reader_state.current_index = tab.current_index;
        } else {
            reader_state.current_index = 0;
        }
    }
}
