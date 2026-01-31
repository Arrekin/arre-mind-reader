//! UI systems using bevy_egui.
//!
//! Provides tab bar, playback controls, settings panel, and the new tab dialog.
//! Uses async file loading to prevent UI freezes.

use bevy::prelude::*;
use bevy::tasks::{block_on, poll_once, AsyncComputeTaskPool, Task};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use std::path::PathBuf;

use crate::fonts::FontsStore;
use crate::reader::{
    ActiveTab, ReadingState, TabFilePath, TabFontSettings, TabMarker, TabWpm, WordsManager,
    WPM_DEFAULT, WPM_MIN, WPM_MAX, WPM_STEP, FONT_SIZE_DEFAULT,
};
use crate::text::{TextParser, TxtParser, Word};

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NewTabDialog>()
            .init_resource::<PendingFileLoad>()
            .add_systems(Update, poll_file_load_task)
            .add_systems(EguiPrimaryContextPass, (tab_bar_system, controls_system, new_tab_dialog_system))
            ;
    }
}

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

fn tab_bar_system(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut dialog: ResMut<NewTabDialog>,
    mut next_state: ResMut<NextState<ReadingState>>,
    tabs: Query<(Entity, &Name, Has<ActiveTab>), With<TabMarker>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    
    let mut tab_to_close: Option<Entity> = None;
    let mut tab_to_select: Option<Entity> = None;
    let mut open_dialog = false;
    
    let tab_info: Vec<(Entity, Name, bool)> = tabs.iter()
        .map(|(e, name, is_active)| (e, name.clone(), is_active))
        .collect();
    
    egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
        ui.horizontal(|ui| {
            for (entity, name, is_active) in &tab_info {
                let label = if *is_active {
                    egui::RichText::new(name.as_str()).strong()
                } else {
                    egui::RichText::new(name.as_str())
                };
                
                ui.horizontal(|ui| {
                    if ui.selectable_label(*is_active, label).clicked() {
                        tab_to_select = Some(*entity);
                    }
                    if ui.small_button("×").clicked() {
                        tab_to_close = Some(*entity);
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
    if let Some(entity) = tab_to_select {
        // Remove ActiveTab from all tabs, add to selected
        for (e, _, was_active) in &tab_info {
            if *was_active {
                commands.entity(*e).remove::<ActiveTab>();
            }
        }
        commands.entity(entity).insert(ActiveTab);
        next_state.set(ReadingState::Idle);
    }
    if let Some(entity) = tab_to_close {
        let was_active = tab_info.iter().find(|(e, _, _)| *e == entity).is_some_and(|(_, _, a)| *a);
        commands.entity(entity).despawn();
        if was_active {
            // Select another tab
            if let Some((other_entity, _, _)) = tab_info.iter().find(|(e, _, _)| *e != entity) {
                commands.entity(*other_entity).insert(ActiveTab);
            }
        }
        next_state.set(ReadingState::Idle);
    }
    if open_dialog {
        dialog.open = true;
        dialog.text_input.clear();
    }
}

fn controls_system(
    mut contexts: EguiContexts,
    current_state: Res<State<ReadingState>>,
    mut next_state: ResMut<NextState<ReadingState>>,
    fonts: Res<FontsStore>,
    mut active_tabs: Query<(&mut TabWpm, &mut TabFontSettings, &WordsManager), With<ActiveTab>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    
    egui::TopBottomPanel::bottom("controls").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if let Ok((mut tab_wpm, mut font_settings, words_mgr)) = active_tabs.single_mut() {
                    let btn_text = match current_state.get() {
                        ReadingState::Playing => "⏸ Pause",
                        _ => "▶ Play",
                    };
                    if ui.button(btn_text).clicked() {
                        match current_state.get() {
                            ReadingState::Playing => next_state.set(ReadingState::Paused),
                            _ => next_state.set(ReadingState::Playing),
                        }
                    }
                    
                    // Progress
                    let total = words_mgr.words.len();
                    let current = words_mgr.current_index + 1;
                    ui.label(format!("{}/{}", current, total));
                    
                    // Progress bar
                    let progress = if total > 0 { current as f32 / total as f32 } else { 0.0 };
                    ui.add(egui::ProgressBar::new(progress).desired_width(200.0));
                    
                    ui.separator();
                    
                    // WPM slider (per-tab)
                    ui.label("WPM:");
                    let mut wpm = tab_wpm.0;
                    if ui.add(egui::Slider::new(&mut wpm, WPM_MIN..=WPM_MAX).step_by(WPM_STEP as f64)).changed() {
                        tab_wpm.0 = wpm;
                    }
                    
                    ui.separator();
                    
                    // Font selector (per-tab)
                    ui.label("Font:");
                    egui::ComboBox::from_id_salt("font_selector")
                        .selected_text(&font_settings.font_name)
                        .show_ui(ui, |ui| {
                            for font_data in &fonts.fonts {
                                if ui.selectable_label(font_settings.font_name == font_data.name, &font_data.name).clicked() {
                                    font_settings.font_name = font_data.name.clone();
                                    font_settings.font_handle = font_data.handle.clone();
                                }
                            }
                        });
                    
                    ui.separator();
            } else {
                ui.label("No tab open. Click '+ New' to add text.");
            }
            
            // State indicator
            let state_text = match current_state.get() {
                ReadingState::Idle => "Idle",
                ReadingState::Playing => "Reading",
                ReadingState::Paused => "Paused",
            };
            ui.label(format!("[{}]", state_text));
        });
    });
}

fn new_tab_dialog_system(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut dialog: ResMut<NewTabDialog>,
    mut next_state: ResMut<NextState<ReadingState>>,
    mut pending_load: ResMut<PendingFileLoad>,
    fonts: Res<FontsStore>,
    tabs: Query<Entity, With<TabMarker>>,
    active_tabs: Query<Entity, With<ActiveTab>>,
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
                        let words = crate::text::TxtParser.parse(&content);
                        
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
                    // Remove ActiveTab from current
                    if let Ok(old_active) = active_tabs.single() {
                        commands.entity(old_active).remove::<ActiveTab>();
                    }
                    spawn_tab(&mut commands, &fonts, name, None, words);
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
    mut commands: Commands,
    mut pending_load: ResMut<PendingFileLoad>,
    mut dialog: ResMut<NewTabDialog>,
    mut next_state: ResMut<NextState<ReadingState>>,
    fonts: Res<FontsStore>,
    active_tabs: Query<Entity, With<ActiveTab>>,
) {
    let Some(task) = &mut pending_load.task else { return };
    
    if let Some(result) = block_on(poll_once(task)) {
        if let Some(file_result) = result {
            // Remove ActiveTab from current
            if let Ok(old_active) = active_tabs.single() {
                commands.entity(old_active).remove::<ActiveTab>();
            }
            spawn_tab(
                &mut commands,
                &fonts,
                file_result.name,
                Some(file_result.path),
                file_result.words,
            );
            dialog.open = false;
            next_state.set(ReadingState::Idle);
        }
        pending_load.task = None;
    }
}

pub fn spawn_tab(
    commands: &mut Commands,
    fonts: &FontsStore,
    name: String,
    file_path: Option<PathBuf>,
    words: Vec<Word>,
) -> Entity {
    let default_font = fonts.default_font();
    let font_name = default_font.map(|f| f.name.clone()).unwrap_or_default();
    let font_handle = default_font.map(|f| f.handle.clone()).unwrap_or_default();
    
    let mut entity_commands = commands.spawn((
        TabMarker,
        ActiveTab,
        Name::new(name),
        TabFontSettings {
            font_name,
            font_handle,
            font_size: FONT_SIZE_DEFAULT,
        },
        TabWpm(WPM_DEFAULT),
        WordsManager {
            words,
            current_index: 0,
        },
    ));
    
    if let Some(path) = file_path {
        entity_commands.insert(TabFilePath(path));
    }
    
    entity_commands.id()
}
