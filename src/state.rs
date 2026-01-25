//! Core state types and resources for the reader application.
//!
//! Tab entities use ECS components: TabMarker, TabName, TabFontSettings, TabWpm, TabFilePath, WordsManager.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ============================================================================
// Constants
// ============================================================================

pub mod constants {
    pub const WPM_DEFAULT: u32 = 300;
    pub const WPM_MIN: u32 = 100;
    pub const WPM_MAX: u32 = 1000;
    pub const WPM_STEP: u32 = 50;
    pub const FONT_SIZE_DEFAULT: f32 = 48.0;
    pub const WORD_SKIP_AMOUNT: usize = 5;
    pub const RETICLE_OFFSET_Y: f32 = 40.0;
    pub const RETICLE_WIDTH: f32 = 3.0;
    pub const RETICLE_HEIGHT: f32 = 40.0;
    pub const RETICLE_ALPHA: f32 = 0.5;
    pub const CHAR_WIDTH_RATIO: f32 = 0.6;
    pub const HIGHLIGHT_COLOR: (f32, f32, f32) = (1.0, 0.0, 0.0);
}

// ============================================================================
// Reading State
// ============================================================================

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

impl Word {
    pub fn orp_index(&self) -> usize {
        match self.text.chars().count() {
            0 => 0,
            1 => 0,
            2..=5 => 1,
            6..=9 => 2,
            10..=13 => 3,
            _ => 4,
        }
    }
    
    pub fn display_duration_ms(&self, wpm: u32) -> u64 {
        let base_ms = 60_000.0 / wpm as f64;
        let mut multiplier = 1.0f64;
        
        if self.text.chars().count() > 10 {
            multiplier = multiplier.max(1.3);
        }
        if self.text.ends_with(',') || self.text.ends_with(';') {
            multiplier = multiplier.max(2.0);
        }
        if self.text.ends_with('.') || self.text.ends_with('?') || self.text.ends_with('!') {
            multiplier = multiplier.max(3.0);
        }
        if self.is_paragraph_end {
            multiplier = multiplier.max(4.0);
        }
        
        (base_ms * multiplier) as u64
    }
}

// ============================================================================
// Tab Components
// ============================================================================

pub type TabId = u64;

#[derive(Component)]
pub struct TabMarker {
    pub id: TabId,
}

#[derive(Component)]
pub struct TabName(pub String);

#[derive(Component)]
pub struct TabFontSettings {
    pub font_name: String,
    pub font_handle: Handle<Font>,
    pub font_size: f32,
}

#[derive(Component)]
pub struct TabWpm(pub u32);

#[derive(Component)]
pub struct TabFilePath(pub PathBuf);

#[derive(Component)]
pub struct WordsManager {
    pub words: Vec<Word>,
    pub current_index: usize,
}

// ============================================================================
// Resources
// ============================================================================

#[derive(Resource, Default)]
pub struct ActiveTab {
    pub entity: Option<Entity>,
    next_id: TabId,
}
impl ActiveTab {
    pub fn allocate_id(&mut self) -> TabId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    pub fn set_next_id(&mut self, id: TabId) {
        self.next_id = id;
    }
}
