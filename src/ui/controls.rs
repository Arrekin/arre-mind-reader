//! Playback controls UI component.
//!
//! Renders play/pause, progress, WPM slider, and font selector.
//! Emits PlaybackCommand events for state changes.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::fonts::FontsStore;
use crate::playback::PlaybackCommand;
use crate::reader::{ReadingState, WPM_MIN, WPM_MAX, WPM_STEP};
use crate::tabs::{ActiveTab, TabFontSettings, TabWpm, WordsManager};

pub fn controls_system(
    mut contexts: EguiContexts,
    mut playback_cmds: MessageWriter<PlaybackCommand>,
    current_state: Res<State<ReadingState>>,
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
                    playback_cmds.write(PlaybackCommand::TogglePlayPause);
                }
                
                // Progress
                let (current, total) = words_mgr.progress();
                ui.label(format!("{}/{}", current, total));
                
                // Progress bar
                let progress = if total > 0 { current as f32 / total as f32 } else { 0.0 };
                ui.add(egui::ProgressBar::new(progress).desired_width(200.0));
                
                ui.separator();
                
                // WPM slider (per-tab) - direct mutation is fine for continuous sliders
                ui.label("WPM:");
                let mut wpm = tab_wpm.0;
                if ui.add(egui::Slider::new(&mut wpm, WPM_MIN..=WPM_MAX).step_by(WPM_STEP as f64)).changed() {
                    tab_wpm.0 = wpm;
                }
                
                ui.separator();
                
                // Font selector (per-tab) - direct mutation for immediate feedback
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
