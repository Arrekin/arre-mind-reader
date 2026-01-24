use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::reader::parse_text;
use crate::state::{ReaderSettings, ReaderState, ReadingState, Tab, TabManager};

#[derive(Resource, Default)]
pub struct NewTabDialog {
    pub open: bool,
    pub text_input: String,
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NewTabDialog>()
            .add_systems(Update, sync_tab_to_reader)
            .add_systems(EguiPrimaryContextPass, (tab_bar_system, controls_system, new_tab_dialog_system));
    }
}

fn tab_bar_system(
    mut contexts: EguiContexts,
    mut tabs: ResMut<TabManager>,
    mut dialog: ResMut<NewTabDialog>,
    mut next_state: ResMut<NextState<ReadingState>>,
    mut initialized: Local<bool>,
) {
    if !*initialized {
        *initialized = true;
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    
    let mut tab_to_close: Option<usize> = None;
    let mut tab_to_select: Option<usize> = None;
    let mut open_dialog = false;
    
    // Collect tab info for display
    let tab_info: Vec<(usize, String, bool)> = tabs.tabs.iter().enumerate()
        .map(|(i, tab)| (i, tab.name.clone(), tabs.active_index == Some(i)))
        .collect();
    
    egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
        ui.horizontal(|ui| {
            for (i, name, is_active) in &tab_info {
                let label = if *is_active {
                    egui::RichText::new(name).strong()
                } else {
                    egui::RichText::new(name)
                };
                
                ui.horizontal(|ui| {
                    if ui.selectable_label(*is_active, label).clicked() {
                        tab_to_select = Some(*i);
                    }
                    if ui.small_button("×").clicked() {
                        tab_to_close = Some(*i);
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
    if let Some(i) = tab_to_select {
        tabs.active_index = Some(i);
        next_state.set(ReadingState::Idle);
    }
    if let Some(i) = tab_to_close {
        tabs.close_tab(i);
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
    mut initialized: Local<bool>,
) {
    if !*initialized {
        *initialized = true;
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    
    egui::TopBottomPanel::bottom("controls").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // Play/Pause button (only if we have a tab)
            if tabs.active_index.is_some() {
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
                let total = reader_state.words.len();
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
            let mut wpm = settings.wpm as i32;
            if ui.add(egui::Slider::new(&mut wpm, 100..=1000).step_by(50.0)).changed() {
                settings.wpm = wpm as u32;
            }
            
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
    mut initialized: Local<bool>,
) {
    if !*initialized {
        *initialized = true;
        return;
    }
    if !dialog.open {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    
    egui::Window::new("New Tab")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("📂 Load from File").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Text files", &["txt"])
                        .pick_file()
                    {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            let name = path.file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("Untitled")
                                .to_string();
                            let words = parse_text(&content);
                            tabs.add_tab(Tab {
                                name,
                                file_path: Some(path),
                                words,
                                current_index: 0,
                            });
                            dialog.open = false;
                            next_state.set(ReadingState::Idle);
                        }
                    }
                }
            });
            
            ui.separator();
            ui.label("Or paste text below:");
            
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut dialog.text_input)
                            .desired_width(400.0)
                            .desired_rows(10)
                            .hint_text("Paste your text here...")
                    );
                });
            
            ui.horizontal(|ui| {
                let can_create = !dialog.text_input.trim().is_empty();
                if ui.add_enabled(can_create, egui::Button::new("Create Tab")).clicked() {
                    let words = parse_text(&dialog.text_input);
                    let name = format!("Text {}", tabs.tabs.len() + 1);
                    tabs.add_tab(Tab {
                        name,
                        file_path: None,
                        words,
                        current_index: 0,
                    });
                    dialog.open = false;
                    dialog.text_input.clear();
                    next_state.set(ReadingState::Idle);
                }
                
                if ui.button("Cancel").clicked() {
                    dialog.open = false;
                    dialog.text_input.clear();
                }
            });
        });
}

fn sync_tab_to_reader(
    mut tabs: ResMut<TabManager>,
    mut reader_state: ResMut<ReaderState>,
    current_state: Res<State<ReadingState>>,
) {
    let tab_changed = tabs.active_index != tabs.last_synced_index;
    
    // Sync from reader back to tab when reading (only if tab hasn't changed)
    if !tab_changed && *current_state.get() != ReadingState::Idle {
        if let Some(tab) = tabs.active_tab_mut() {
            tab.current_index = reader_state.current_index;
        }
    }
    
    // Sync from tab to reader when tab changes
    if tab_changed {
        tabs.last_synced_index = tabs.active_index;
        if let Some(tab) = tabs.active_tab() {
            reader_state.words = tab.words.clone();
            reader_state.current_index = tab.current_index;
        } else {
            reader_state.words.clear();
            reader_state.current_index = 0;
        }
    }
}
