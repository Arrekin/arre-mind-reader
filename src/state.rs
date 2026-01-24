use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum ReadingState {
    #[default]
    Idle,
    Active,
    Paused,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Word {
    pub text: String,
    pub is_paragraph_end: bool,
}

#[derive(Resource, Default)]
pub struct ReaderState {
    pub current_index: usize,
    pub words: Vec<Word>,
}

#[derive(Resource, Default)]
pub struct FocusModeState {
    pub ui_opacity: f32,
    pub mouse_idle_timer: f32,
}

#[derive(Resource)]
pub struct AvailableFonts {
    pub fonts: Vec<String>,
}

impl Default for AvailableFonts {
    fn default() -> Self {
        Self {
            fonts: vec!["fonts/JetBrainsMono-Regular.ttf".to_string()],
        }
    }
}

#[derive(Resource, Serialize, Deserialize, Clone)]
pub struct ReaderSettings {
    pub wpm: u32,
    pub font_size: f32,
    pub highlight_color: [f32; 3],
    pub font_path: String,
}

impl Default for ReaderSettings {
    fn default() -> Self {
        Self {
            wpm: 300,
            font_size: 48.0,
            highlight_color: [1.0, 0.0, 0.0],
            font_path: "fonts/JetBrainsMono-Regular.ttf".to_string(),
        }
    }
}

impl ReaderSettings {
    pub fn highlight_bevy_color(&self) -> Color {
        Color::srgb(self.highlight_color[0], self.highlight_color[1], self.highlight_color[2])
    }
}

#[derive(Resource, Default, serde::Serialize, serde::Deserialize)]
pub struct TabManager {
    pub tabs: Vec<Tab>,
    pub active_index: Option<usize>,
    #[serde(skip)]
    pub last_synced_index: Option<usize>,
}

impl TabManager {
    pub fn active_tab(&self) -> Option<&Tab> {
        self.active_index.and_then(|i| self.tabs.get(i))
    }
    
    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.active_index.and_then(|i| self.tabs.get_mut(i))
    }
    
    pub fn add_tab(&mut self, tab: Tab) {
        self.tabs.push(tab);
        self.active_index = Some(self.tabs.len() - 1);
    }
    
    pub fn close_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.tabs.remove(index);
            if self.tabs.is_empty() {
                self.active_index = None;
            } else if let Some(active) = self.active_index {
                if active >= self.tabs.len() {
                    self.active_index = Some(self.tabs.len() - 1);
                } else if active > index {
                    self.active_index = Some(active - 1);
                }
            }
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Tab {
    pub name: String,
    pub file_path: Option<std::path::PathBuf>,
    pub words: Vec<Word>,
    pub current_index: usize,
}
