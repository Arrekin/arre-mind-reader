//! Playback controls UI component.
//!
//! Renders play/pause, progress, WPM slider, and font selector.
//! Emits PlaybackCommand events for state changes.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::fonts::FontsStore;
use crate::playback::PlaybackCommand;
use crate::reader::{ContentNavigate, ReadingState, FONT_SIZE_MIN, FONT_SIZE_MAX, WPM_MIN, WPM_MAX, WPM_STEP};
use crate::tabs::{ActiveTab, Content, ReaderTab, TabFontSettings, TabWpm};

pub fn controls_system(
    mut commands: Commands,
    mut contexts: EguiContexts,
    current_state: Res<State<ReadingState>>,
    fonts: Res<FontsStore>,
    active_reader: Query<(Entity, &TabWpm, &TabFontSettings, &Content), (With<ActiveTab>, With<ReaderTab>)>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    
    egui::TopBottomPanel::bottom("controls").show(ctx, |ui| {
        ui.horizontal(|ui| {
            let Ok((entity, tab_wpm, font_settings, content)) = active_reader.single() else {
                ui.label(""); // Keeps the height of the panel constant
                return;
            };
            let at_end = content.has_words() && content.is_at_end();
            let (btn_text, btn_cmd) = match (current_state.get(), at_end) {
                (_, true) => ("↺ Restart", PlaybackCommand::Restart),
                (ReadingState::Playing, _) => ("⏸ Pause", PlaybackCommand::TogglePlayPause),
                _ => ("▶ Play", PlaybackCommand::TogglePlayPause),
            };
            let btn = egui::Button::new(btn_text);
            // Size the button manually to ensure constant width over the text(otherwise it jumps when seeking the content)
            if ui.add_sized(egui::vec2(80.0, ui.spacing().interact_size.y), btn).clicked() {
                commands.trigger(btn_cmd);
            }
            
            // Seekable progress
            let (current, total) = content.progress();
            let mut seek_index = current;
            // ilog10() + 1 = digit count of total; pad current to match so label width stays constant
            let width = total.max(1).ilog10() as usize + 1;
            ui.label(egui::RichText::new(format!("{:>width$}/{}", current + 1, total)).monospace());
            let max_index = total.saturating_sub(1);
            if max_index > 0 {
                let slider = egui::Slider::new(&mut seek_index, 0..=max_index)
                    .show_value(false);
                if ui.add_sized(egui::vec2(200.0, ui.spacing().interact_size.y), slider).changed() {
                    commands.trigger(ContentNavigate::Seek(seek_index));
                }
            }
            
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
                .selected_text(&font_settings.font.name)
                .show_ui(ui, |ui| {
                    for font_data in fonts.iter() {
                        if ui.selectable_label(font_settings.font.name == font_data.name, &font_data.name).clicked() {
                            commands.entity(entity).insert(TabFontSettings::from_font(font_data, font_settings.font_size));
                        }
                    }
                });
            
            // Font size (per-tab)
            let mut font_size = font_settings.font_size;
            let drag = egui::DragValue::new(&mut font_size)
                .range(FONT_SIZE_MIN..=FONT_SIZE_MAX)
                .speed(0.5)
                .suffix(" px");
            if ui.add(drag).changed() {
                commands.entity(entity).insert(TabFontSettings::from_font(&font_settings.font, font_size));
            }
            
            ui.separator();
            
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
