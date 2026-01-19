use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use std::path::PathBuf;

use crate::reader::parse_text;
use crate::state::{ReaderSettings, ReaderState, ReadingState};

#[derive(Resource, Default)]
pub struct FileLoadRequest(Option<PathBuf>);

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FileLoadRequest>()
            .add_systems(Update, process_file_load)
            .add_systems(EguiPrimaryContextPass, ui_system);
    }
}

fn ui_system(
    mut contexts: EguiContexts,
    reader_state: Res<ReaderState>,
    mut settings: ResMut<ReaderSettings>,
    current_state: Res<State<ReadingState>>,
    mut next_state: ResMut<NextState<ReadingState>>,
    mut file_req: ResMut<FileLoadRequest>,
    mut initialized: Local<bool>,
) {
    // Skip first frame - egui context not ready yet
    if !*initialized {
        *initialized = true;
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    
    egui::TopBottomPanel::bottom("controls").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // Load file button
            if ui.button("📂 Open").clicked() {
                open_file_dialog(&mut file_req);
            }
            
            ui.separator();
            
            // Play/Pause button
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

fn process_file_load(
    mut file_req: ResMut<FileLoadRequest>,
    mut reader_state: ResMut<ReaderState>,
    mut next_state: ResMut<NextState<ReadingState>>,
) {
    if let Some(path) = file_req.0.take() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            reader_state.words = parse_text(&content);
            reader_state.current_index = 0;
            next_state.set(ReadingState::Idle);
            info!("Loaded file: {:?} ({} words)", path, reader_state.words.len());
        }
    }
}

fn open_file_dialog(file_req: &mut FileLoadRequest) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("Text files", &["txt"])
        .pick_file()
    {
        file_req.0 = Some(path);
    }
}
