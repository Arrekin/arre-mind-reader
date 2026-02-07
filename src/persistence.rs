//! Persistence for tab state using RON format.
//!
//! Saves and loads tab state (open files, reading positions) to the user's config directory.

use bevy::log::{debug, info, warn};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::tabs::{
    ActiveTab, TabCreateRequest, TabFilePath, TabFontSettings, TabMarker, TabWpm, WordsManager,
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
    name: String,
    file_path: Option<String>,
    font_name: String,
    font_size: f32,
    wpm: u32,
    words: Vec<Word>,
    current_index: usize,
    is_active: bool,
}

#[derive(Serialize, Deserialize, Default)]
struct ProgramState {
    tabs: Vec<SavedTab>,
}
#[cfg(not(target_arch = "wasm32"))]
impl ProgramState {
    fn save(&self) {
        let Some(config_dir) = dirs::config_dir().map(|p| p.join("arre-mind-reader")) else {
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
        let Some(config_dir) = dirs::config_dir().map(|p| p.join("arre-mind-reader")) else {
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

#[cfg(target_arch = "wasm32")]
impl ProgramState {
    fn save(&self) {
        use gloo_storage::Storage;
        match ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()) {
            Ok(content) => {
                if let Err(e) = gloo_storage::LocalStorage::set(TABS_FILE, content) {
                    warn!("Failed to save to localStorage: {:?}", e);
                } else {
                    debug!("Saved {} tabs to localStorage", self.tabs.len());
                }
            }
            Err(e) => warn!("Failed to serialize tabs: {}", e),
        }
    }

    fn load() -> Self {
        use gloo_storage::Storage;
        match gloo_storage::LocalStorage::get::<String>(TABS_FILE) {
            Ok(content) => match ron::from_str::<ProgramState>(&content) {
                Ok(state) => {
                    debug!("Loaded {} tabs from localStorage", state.tabs.len());
                    state
                }
                Err(e) => {
                    warn!("Failed to parse tabs from localStorage: {}", e);
                    ProgramState::default()
                }
            },
            Err(_) => {
                debug!("No saved tabs found in localStorage");
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

fn spawn_tabs_from_program_state(mut commands: Commands) {
    let program_state = ProgramState::load();
    let total_tabs = program_state.tabs.len();
    
    for tab in program_state.tabs {
        let mut request = TabCreateRequest::new(tab.name, tab.words)
            .with_font(tab.font_name, tab.font_size)
            .with_wpm(tab.wpm)
            .with_current_index(tab.current_index)
            .with_active(tab.is_active);
        
        if let Some(path) = tab.file_path {
            request = request.with_file_path(path);
        }
        
        commands.trigger(request);
    }
    
    info!("Restored {} tabs from saved state", total_tabs);
}

fn persist_program_state(
    time: Res<Time>,
    mut save_timer: ResMut<TabSaveTimer>,
    app_exit_events: MessageReader<AppExit>,
    tabs: Query<(
        &Name,
        &TabFontSettings,
        &TabWpm,
        &WordsManager,
        Option<&TabFilePath>,
        Has<ActiveTab>,
    ), With<TabMarker>>,
) {
    // Save on periodic timer OR on app exit, whichever comes first
    save_timer.timer.tick(time.delta());
    if !save_timer.timer.just_finished() && app_exit_events.is_empty() { return; }

    let saved_tabs: Vec<SavedTab> = tabs.iter()
        .map(|(name, font_settings, wpm, words_mgr, file_path, is_active)| {
            SavedTab {
                name: name.to_string(),
                file_path: file_path.map(|fp| fp.0.clone()),
                font_name: font_settings.font_name.clone(),
                font_size: font_settings.font_size,
                wpm: wpm.0,
                words: words_mgr.words.clone(),
                current_index: words_mgr.current_index,
                is_active,
            }
        })
        .collect();
    
    ProgramState { tabs: saved_tabs }.save();
    info!("The program state was saved");
}
