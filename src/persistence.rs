//! Persistence for tab state using RON format.
//!
//! Saves and loads tab state (open files, reading positions) to the user's config directory.

use bevy::log::{debug, info, warn};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::fonts::FontsStore;
use crate::state::ActiveTab;
use crate::reader::{
    TabFilePath, TabFontSettings, TabId, TabMarker, TabWpm, WordsManager,
};
use crate::text::Word;

pub struct PersistencePlugin;
impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TabSaveTimer>()
            .add_systems(PostStartup, spawn_tabs_from_program_state)
            .add_systems(Last, persist_program_state)
            ;
    }
}

const TABS_FILE: &str = "tabs.ron";
const SAVE_INTERVAL_SECS: f32 = 5.0;

// ============================================================================
// Persistence-only Data Structures
// ============================================================================

#[derive(Serialize, Deserialize)]
struct SavedTab {
    id: TabId,
    name: String,
    file_path: Option<PathBuf>,
    font_name: String,
    font_size: f32,
    wpm: u32,
    words: Vec<Word>,
    current_index: usize,
}

#[derive(Serialize, Deserialize, Default)]
struct ProgramState {
    tabs: Vec<SavedTab>,
    active_id: Option<TabId>,
    next_id: TabId,
}
impl ProgramState {
    fn get_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("arre-mind-reader"))
    }

    fn save(&self) {
        let Some(config_dir) = Self::get_config_path() else {
            warn!("Could not determine config directory for saving");
            return;
        };
        if let Err(e) = std::fs::create_dir_all(&config_dir) {
            warn!("Failed to create config directory: {}", e);
            return;
        }
        let path = config_dir.join(TABS_FILE);
        match ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()) {
            Ok(content) => {
                if let Err(e) = std::fs::write(&path, content) {
                    warn!("Failed to write tabs file: {}", e);
                } else {
                    debug!("Saved {} tabs to {:?}", self.tabs.len(), path);
                }
            }
            Err(e) => warn!("Failed to serialize tabs: {}", e),
        }
    }

    fn load() -> Self {
        let Some(config_dir) = Self::get_config_path() else {
            warn!("Could not determine config directory");
            return ProgramState::default();
        };
        let path = config_dir.join(TABS_FILE);
        if !path.exists() {
            debug!("No saved tabs file found at {:?}", path);
            return ProgramState::default();
        }
        match std::fs::read_to_string(&path) {
            Ok(content) => match ron::from_str::<ProgramState>(&content) {
                Ok(state) => {
                    debug!("Loaded {} tabs from {:?}", state.tabs.len(), path);
                    state
                }
                Err(e) => {
                    warn!("Failed to parse tabs file: {}", e);
                    ProgramState::default()
                }
            },
            Err(e) => {
                warn!("Failed to read tabs file: {}", e);
                ProgramState::default()
            }
        }
    }


}

#[derive(Resource)]
struct TabSaveTimer {
    timer: Timer,
}

impl Default for TabSaveTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(SAVE_INTERVAL_SECS, TimerMode::Repeating),
        }
    }
}

// ============================================================================
// Systems
// ============================================================================

fn spawn_tabs_from_program_state(
    mut commands: Commands,
    mut active_tab: ResMut<ActiveTab>,
    fonts: Res<FontsStore>,
) {
    let program_state = ProgramState::load();

    active_tab.set_next_id(program_state.next_id);
    
    let mut active_entity = None;
    let total_tabs = program_state.tabs.len();
    for tab in program_state.tabs {
        let font_data = fonts.get_by_name(&tab.font_name).or_else(|| fonts.default_font());
        let font_name = font_data.map(|f| f.name.clone()).unwrap_or_default();
        let font_handle = font_data.map(|f| f.handle.clone()).unwrap_or_default();
        
        let mut entity_commands = commands.spawn((
            TabMarker { id: tab.id },
            Name::new(tab.name),
            TabFontSettings {
                font_name,
                font_handle,
                font_size: tab.font_size,
            },
            TabWpm(tab.wpm),
            WordsManager {
                words: tab.words,
                current_index: tab.current_index,
            },
        ));
        
        if let Some(path) = tab.file_path {
            entity_commands.insert(TabFilePath(path));
        }
        
        let entity = entity_commands.id();
        if program_state.active_id == Some(tab.id) {
            active_entity = Some(entity);
        }
    }
    
    active_tab.entity = active_entity;
    info!("Restored {} tabs from saved state", total_tabs);
}

fn persist_program_state(
    time: Res<Time>,
    mut save_timer: ResMut<TabSaveTimer>,
    active_tab: Res<ActiveTab>,
    app_exit_events: MessageReader<AppExit>,
    tabs: Query<(
        Entity,
        &TabMarker,
        &Name,
        &TabFontSettings,
        &TabWpm,
        &WordsManager,
        Option<&TabFilePath>,
    )>,
) {
    save_timer.timer.tick(time.delta());
    if !save_timer.timer.just_finished() && app_exit_events.is_empty() { return; }

    // Collect data for the save
    let mut saved_tabs = Vec::new();
    let mut active_id = None;
    let mut max_id: TabId = 0;
    
    for (entity, marker, name, font_settings, wpm, words_mgr, file_path) in tabs.iter() {
        max_id = max_id.max(marker.id);
        if active_tab.entity == Some(entity) {
            active_id = Some(marker.id);
        }
        saved_tabs.push(SavedTab {
            id: marker.id,
            name: name.to_string(),
            file_path: file_path.map(|fp| fp.0.clone()),
            font_name: font_settings.font_name.clone(),
            font_size: font_settings.font_size,
            wpm: wpm.0,
            words: words_mgr.words.clone(),
            current_index: words_mgr.current_index,
        });
    }
    
    ProgramState {
        tabs: saved_tabs,
        active_id,
        next_id: max_id + 1,
    }.save();
    info!("The program state was saved");
}
