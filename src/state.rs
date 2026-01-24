//! Core state types and resources for the reader application.
//!
//! Contains the main data structures: `ReaderState`, `ReaderSettings`, `TabManager`, and `Tab`.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Application-wide constants for configuration values.
pub mod constants {
    pub const WPM_MIN: u32 = 100;
    pub const WPM_MAX: u32 = 1000;
    pub const WPM_STEP: u32 = 50;
    pub const WORD_SKIP_AMOUNT: usize = 5;
    pub const RETICLE_OFFSET_Y: f32 = 40.0;
    pub const RETICLE_WIDTH: f32 = 3.0;
    pub const RETICLE_HEIGHT: f32 = 40.0;
    pub const RETICLE_ALPHA: f32 = 0.5;
    pub const CHAR_WIDTH_RATIO: f32 = 0.6;
}

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

pub type TabId = u64;

#[derive(Resource, serde::Serialize, serde::Deserialize)]
pub struct TabManager {
    tabs: Vec<Tab>,
    active_id: Option<TabId>,
    next_id: TabId,
    #[serde(skip)]
    last_synced_id: Option<TabId>,
}

impl Default for TabManager {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            active_id: None,
            next_id: 1,
            last_synced_id: None,
        }
    }
}

impl TabManager {
    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }
    
    pub fn active_id(&self) -> Option<TabId> {
        self.active_id
    }
    
    pub fn set_active(&mut self, id: TabId) {
        if self.tabs.iter().any(|t| t.id == id) {
            self.active_id = Some(id);
        }
    }
    
    pub fn active_tab(&self) -> Option<&Tab> {
        self.active_id.and_then(|id| self.tabs.iter().find(|t| t.id == id))
    }
    
    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.active_id.and_then(|id| self.tabs.iter_mut().find(|t| t.id == id))
    }
    
    pub fn add_tab(&mut self, name: String, file_path: Option<std::path::PathBuf>, words: Vec<Word>) -> TabId {
        let id = self.next_id;
        self.next_id += 1;
        self.tabs.push(Tab {
            id,
            name,
            file_path,
            words,
            current_index: 0,
        });
        self.active_id = Some(id);
        id
    }
    
    pub fn close_tab(&mut self, id: TabId) {
        let Some(pos) = self.tabs.iter().position(|t| t.id == id) else { return };
        self.tabs.remove(pos);
        
        if self.active_id == Some(id) {
            // Select adjacent tab or none
            self.active_id = if self.tabs.is_empty() {
                None
            } else {
                Some(self.tabs[pos.min(self.tabs.len() - 1)].id)
            };
        }
    }
    
    pub fn last_synced_id(&self) -> Option<TabId> {
        self.last_synced_id
    }
    
    pub fn set_last_synced(&mut self, id: Option<TabId>) {
        self.last_synced_id = id;
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Tab {
    pub id: TabId,
    pub name: String,
    pub file_path: Option<std::path::PathBuf>,
    pub words: Vec<Word>,
    pub current_index: usize,
}
