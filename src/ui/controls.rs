//! Playback controls UI component.
//!
//! Renders play/pause, progress, WPM slider, and font selector.
//! Emits PlaybackCommand events for state changes.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::fonts::FontsStore;
use crate::playback::PlaybackCommand;
use crate::reader::{ReadingState, WPM_MIN, WPM_MAX, WPM_STEP};
use crate::tabs::{ActiveTab, TabFontChanged, TabFontSettings, TabWpm, WordsManager};

pub fn controls_system(
    mut commands: Commands,
    mut contexts: EguiContexts,
    current_state: Res<State<ReadingState>>,
    fonts: Res<FontsStore>,
    active_tabs: Query<(Entity, &TabWpm, &TabFontSettings, &WordsManager), With<ActiveTab>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    
    egui::TopBottomPanel::bottom("controls").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if let Ok((entity, tab_wpm, font_settings, words_mgr)) = active_tabs.single() {
                let at_end = words_mgr.has_words() && words_mgr.is_at_end();
                let (btn_text, btn_cmd) = match (current_state.get(), at_end) {
                    (ReadingState::Playing, _) => ("⏸ Pause", PlaybackCommand::TogglePlayPause),
                    (ReadingState::Idle, true) => ("↺ Restart", PlaybackCommand::Restart),
                    _ => ("▶ Play", PlaybackCommand::TogglePlayPause),
                };
                if ui.button(btn_text).clicked() {
                    commands.trigger(btn_cmd);
                }
                
                // Progress
                let (current, total) = words_mgr.progress();
                ui.label(format!("{}/{}", current, total));
                
                // Progress bar
                let progress = if total > 0 { current as f32 / total as f32 } else { 0.0 };
                ui.add(egui::ProgressBar::new(progress).desired_width(200.0));
                
                ui.separator();
                
                // WPM slider (per-tab)
                ui.label("WPM:");
                let mut wpm = tab_wpm.0;
                if ui.add(egui::Slider::new(&mut wpm, WPM_MIN..=WPM_MAX).step_by(WPM_STEP as f64)).changed() {
                    commands.trigger(PlaybackCommand::AdjustWpm(wpm as i32 - tab_wpm.0 as i32));
                }
                
                ui.separator();
                
                // Font selector (per-tab)
                ui.label("Font:");
                egui::ComboBox::from_id_salt("font_selector")
                    .selected_text(&font_settings.font_name)
                    .show_ui(ui, |ui| {
                        for font_data in &fonts.fonts {
                            if ui.selectable_label(font_settings.font_name == font_data.name, &font_data.name).clicked() {
                                commands.trigger(TabFontChanged {
                                    entity,
                                    name: font_data.name.clone(),
                                    handle: font_data.handle.clone(),
                                    size: font_settings.font_size,
                                });
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
