use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum ReadingState {
    #[default]
    Idle,
    Active,
    Paused,
}

#[derive(Clone)]
pub struct Word {
    pub text: String,
    pub is_paragraph_end: bool,
}

#[derive(Resource, Default)]
pub struct ReaderState {
    pub current_index: usize,
    pub words: Vec<Word>,
}

#[derive(Resource, Serialize, Deserialize, Clone)]
pub struct ReaderSettings {
    pub wpm: u32,
    pub font_size: f32,
    pub highlight_color: [f32; 3],
}

impl Default for ReaderSettings {
    fn default() -> Self {
        Self {
            wpm: 300,
            font_size: 48.0,
            highlight_color: [1.0, 0.0, 0.0],
        }
    }
}

impl ReaderSettings {
    pub fn highlight_bevy_color(&self) -> Color {
        Color::srgb(self.highlight_color[0], self.highlight_color[1], self.highlight_color[2])
    }
}

#[derive(Resource, Default, Serialize, Deserialize)]
pub struct OpenBooks {
    pub books: Vec<BookState>,
    pub active_index: Option<usize>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BookState {
    pub path: String,
    pub name: String,
    pub current_index: usize,
}

#[derive(Resource, Default)]
pub struct FocusModeState {
    pub ui_opacity: f32,
    pub mouse_idle_timer: f32,
}
