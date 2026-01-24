//! Settings persistence using RON format.
//!
//! Saves and loads tab state (open files, reading positions) to the user's config directory.
//! Uses debounced saving to avoid excessive disk writes.

use bevy::log::{debug, info, warn};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::fonts::FontsStore;
use crate::state::{
    ActiveTab, TabFilePath, TabFontSettings, TabId, TabMarker, TabName, TabWpm, Word, WordsManager,
};

const TABS_FILE: &str = "tabs.ron";
const SAVE_DEBOUNCE_SECS: f32 = 5.0;

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
struct SavedState {
    tabs: Vec<SavedTab>,
    active_id: Option<TabId>,
    next_id: TabId,
}

#[derive(Resource)]
struct TabSaveTimer {
    timer: Timer,
}
impl Default for TabSaveTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(SAVE_DEBOUNCE_SECS, TimerMode::Repeating),
        }
    }
}

fn get_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("arre-mind-reader"))
}

fn load_saved_state() -> SavedState {
    let Some(config_dir) = get_config_path() else {
        warn!("Could not determine config directory");
        return SavedState::default();
    };
    let path = config_dir.join(TABS_FILE);
    if !path.exists() {
        debug!("No saved tabs file found at {:?}", path);
        return SavedState::default();
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => match ron::from_str::<SavedState>(&content) {
            Ok(state) => {
                debug!("Loaded {} tabs from {:?}", state.tabs.len(), path);
                state
            }
            Err(e) => {
                warn!("Failed to parse tabs file: {}", e);
                SavedState::default()
            }
        },
        Err(e) => {
            warn!("Failed to read tabs file: {}", e);
            SavedState::default()
        }
    }
}

fn save_state(state: &SavedState) {
    let Some(config_dir) = get_config_path() else {
        warn!("Could not determine config directory for saving");
        return;
    };
    if let Err(e) = std::fs::create_dir_all(&config_dir) {
        warn!("Failed to create config directory: {}", e);
        return;
    }
    let path = config_dir.join(TABS_FILE);
    match ron::ser::to_string_pretty(state, ron::ser::PrettyConfig::default()) {
        Ok(content) => {
            if let Err(e) = std::fs::write(&path, content) {
                warn!("Failed to write tabs file: {}", e);
            } else {
                debug!("Saved {} tabs to {:?}", state.tabs.len(), path);
            }
        }
        Err(e) => warn!("Failed to serialize tabs: {}", e),
    }
}

fn spawn_tabs_from_saved(
    mut commands: Commands,
    mut active_tab: ResMut<ActiveTab>,
    fonts: Res<FontsStore>,
) {
    let saved = load_saved_state();
    if saved.tabs.is_empty() {
        return;
    }
    
    active_tab.set_next_id(saved.next_id);
    
    let mut active_entity = None;
    for tab in saved.tabs {
        let font_data = fonts.get_by_name(&tab.font_name).or_else(|| fonts.default_font());
        let font_name = font_data.map(|f| f.name.clone()).unwrap_or_default();
        let font_handle = font_data.map(|f| f.handle.clone()).unwrap_or_default();
        
        let mut entity_commands = commands.spawn((
            TabMarker { id: tab.id },
            TabName(tab.name),
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
        if saved.active_id == Some(tab.id) {
            active_entity = Some(entity);
        }
    }
    
    active_tab.entity = active_entity;
    info!("Restored {} tabs from saved state", saved.next_id.saturating_sub(1));
}

fn collect_saved_state(
    active_tab: &ActiveTab,
    tabs_q: &Query<(
        Entity,
        &TabMarker,
        &TabName,
        &TabFontSettings,
        &TabWpm,
        &WordsManager,
        Option<&TabFilePath>,
    )>,
) -> SavedState {
    let mut tabs = Vec::new();
    let mut active_id = None;
    let mut max_id: TabId = 0;
    
    for (entity, marker, name, font_settings, wpm, words_mgr, file_path) in tabs_q.iter() {
        max_id = max_id.max(marker.id);
        if active_tab.entity == Some(entity) {
            active_id = Some(marker.id);
        }
        tabs.push(SavedTab {
            id: marker.id,
            name: name.0.clone(),
            file_path: file_path.map(|fp| fp.0.clone()),
            font_name: font_settings.font_name.clone(),
            font_size: font_settings.font_size,
            wpm: wpm.0,
            words: words_mgr.words.clone(),
            current_index: words_mgr.current_index,
        });
    }
    
    SavedState {
        tabs,
        active_id,
        next_id: max_id + 1,
    }
}

fn persist_tabs(
    time: Res<Time>,
    mut save_timer: ResMut<TabSaveTimer>,
    active_tab: Res<ActiveTab>,
    tabs_q: Query<(
        Entity,
        &TabMarker,
        &TabName,
        &TabFontSettings,
        &TabWpm,
        &WordsManager,
        Option<&TabFilePath>,
    )>,
) {
    // Always tick timer and save periodically when tabs exist
    if tabs_q.is_empty() {
        return;
    }
    
    save_timer.timer.tick(time.delta());
    if save_timer.timer.just_finished() {
        let state = collect_saved_state(&active_tab, &tabs_q);
        save_state(&state);
    }
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TabSaveTimer>()
            .add_systems(PostStartup, spawn_tabs_from_saved)
            .add_systems(PostUpdate, persist_tabs);
    }
}
