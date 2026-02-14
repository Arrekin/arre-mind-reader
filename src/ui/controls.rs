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

const MARQUEE_SPEED: f32 = 50.0;

const MARQUEE_TEXTS: &[&str] = &[
    "Fun fact: you just read this one word at a time",
    "Pro tip: blinking is optional but recommended",
    "No books were harmed in the making of this reader",
    "Have you tried turning the page off and on again?",
    "In a world full of audiobooks, be a speed reader",
    "If a tree falls in a forest and no one reads about it, did it happen?",
    "According to my calculations... you should be reading something right now",
    "I used to be a scrollbar, then I took an arrow to the knee",
    "Warning: prolonged exposure to this app may cause involuntary speed reading",
    "To read, or not to read -- that is never the question",
    "One does not simply read at 100 WPM",
    "It's not a bug, it's a reading feature",
    "Keep calm and adjust your WPM",
    "To infinity and beyond -- one word at a time",
    "Resistance to reading is futile",
    "In case of emergency, increase WPM",
    "Not all those who wander are lost. Some are just scrolling.",
    "The real book was inside you all along",
    "Sponsored by absolutely nobody",
    "Side effects may include: knowledge, vocabulary expansion, and sudden opinions about fonts",
    "You are now breathing manually. Also reading manually.",
    "Reading is just staring at a dead tree and hallucinating -- now digitally!",
    "Today's forecast: 100% chance of words",
    "Your brain is downloading content at variable bitrate",
    "Technically, reading this counts as reading",
    "Every word you read here is a word you didn't read in a book. Think about that.",
    "Remember: speed is nothing without comprehension. But speed is fun.",
    "Welcome to the bottom of the screen. Population: this text.",
    "You've been watching this for a while. Go read something.",
    "The letters are just vibing right now. Let them.",
    "Day 47: the user still hasn't noticed I'm sentient",
    "The mitochondria is the powerhouse of the cell. You're welcome.",
    "Fun fact: octopuses have three hearts but zero reading apps",
    "Aliens probably have faster reading apps. Probably.",
    "Who reads the reader? You do. Right now. Meta.",
    "Life moves pretty fast. If you don't stop and read, you might miss it.",
    "<')))><     <')))><                 <')))><",
];

#[derive(Resource)]
pub struct MarqueeSeed(pub u64);
impl MarqueeSeed {
    #[cfg(not(target_arch = "wasm32"))]
    fn startup_seed() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};

        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(42)
            .saturating_add(1)
    }

    #[cfg(target_arch = "wasm32")]
    fn startup_seed() -> u64 {
        (js_sys::Date::now() as u64).saturating_add(1)
    }
}
impl Default for MarqueeSeed {
    fn default() -> Self {
        Self(Self::startup_seed())
    }
}


fn marquee_pick(cycle: u64) -> usize {
    let mut h = cycle;
    h ^= h >> 16;
    h = h.wrapping_mul(0x45d9f3b);
    h ^= h >> 16;
    h = h.wrapping_mul(0x45d9f3b);
    h ^= h >> 16;
    (h as usize) % MARQUEE_TEXTS.len()
}

pub fn controls_system(
    mut commands: Commands,
    time: Res<Time>,
    mut contexts: EguiContexts,
    current_state: Res<State<ReadingState>>,
    fonts: Res<FontsStore>,
    marquee_seed: Res<MarqueeSeed>,
    active_reader: Query<(Entity, &TabWpm, &TabFontSettings, &Content), (With<ActiveTab>, With<ReaderTab>)>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    
    egui::TopBottomPanel::bottom("controls").show(ctx, |ui| {
        ui.horizontal(|ui| {
            let Ok((entity, tab_wpm, font_settings, content)) = active_reader.single() else {
                // We are on the homepage - show scrolling marquee
                let rect = ui.available_rect_before_wrap();
                ui.allocate_rect(rect, egui::Sense::hover());

                let elapsed = time.elapsed().as_secs_f32();
                let avg_char_width = 8.5;
                let max_text_width = MARQUEE_TEXTS.iter().map(|t| t.len()).max().unwrap_or(1) as f32 * avg_char_width;
                let panel_width = rect.width();
                let total_travel = panel_width + max_text_width;
                let cycle_duration = total_travel / MARQUEE_SPEED;

                let cycle = (elapsed / cycle_duration) as u64 + marquee_seed.0;
                let cycle_t = (elapsed % cycle_duration) / cycle_duration;

                let text = MARQUEE_TEXTS[marquee_pick(cycle)];
                let x = rect.right() - cycle_t * total_travel;
                let y = rect.center().y;

                ui.painter_at(rect).text(
                    egui::pos2(x, y),
                    egui::Align2::LEFT_CENTER,
                    text,
                    egui::FontId::monospace(14.0),
                    ui.visuals().text_color().linear_multiply(0.4),
                );
                ctx.request_repaint();
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
