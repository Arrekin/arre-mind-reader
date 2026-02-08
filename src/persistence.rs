//! Persistence for tab state using RON format.
//!
//! Tab metadata saved periodically to tabs.ron. Word content cached separately
//! per tab, written once on creation.

use std::collections::HashSet;

use bevy::log::{debug, info, warn};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::tabs::{
    ActiveTab, Content, TabCreateRequest, TabFilePath, TabFontSettings,
    TabMarker, TabWpm,
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
    content_cache_id: String,
    current_index: usize,
    is_active: bool,
}

#[derive(Serialize, Deserialize, Default)]
pub struct ProgramState {
    tabs: Vec<SavedTab>,
}
impl ProgramState {
    pub fn generate_cache_id() -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::time::{SystemTime, UNIX_EPOCH};

        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let count = COUNTER.fetch_add(1, Ordering::Relaxed);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        format!("{:x}_{}", timestamp, count)
    }
}
#[cfg(not(target_arch = "wasm32"))]
impl ProgramState {
    fn config_dir() -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|p| p.join("arre-mind-reader"))
    }
    fn cache_dir() -> Option<std::path::PathBuf> {
        Self::config_dir().map(|p| p.join("cache"))
    }
    pub fn write_word_cache(cache_id: &str, words: &[Word]) {
        let Some(dir) = Self::cache_dir() else {
            warn!("Could not determine cache directory");
            return;
        };
        if let Err(e) = std::fs::create_dir_all(&dir) {
            warn!("Failed to create cache directory: {}", e);
            return;
        }
        let path = dir.join(format!("{}.ron", cache_id));
        match ron::ser::to_string(words) {
            Ok(content) => {
                if let Err(e) = std::fs::write(&path, content) {
                    warn!("Failed to write word cache: {}", e);
                }
            }
            Err(e) => warn!("Failed to serialize word cache: {}", e),
        }
    }
    pub fn load_word_cache(cache_id: &str) -> Option<Vec<Word>> {
        let path = Self::cache_dir()?.join(format!("{}.ron", cache_id));
        let content = std::fs::read_to_string(&path).ok()?;
        ron::from_str(&content).ok()
    }
    pub fn delete_word_cache(cache_id: &str) {
        if let Some(path) = Self::cache_dir().map(|d| d.join(format!("{}.ron", cache_id))) {
            let _ = std::fs::remove_file(path);
        }
    }
    fn cleanup_orphan_caches(valid_ids: &HashSet<String>) {
        let Some(dir) = Self::cache_dir() else { return };
        let Ok(entries) = std::fs::read_dir(&dir) else { return };
        for entry in entries.flatten() {
            if let Some(stem) = entry.path().file_stem().and_then(|s| s.to_str()) {
                if !valid_ids.contains(stem) {
                    debug!("Removing orphan cache: {:?}", entry.path());
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }
    }
    fn save(&self) {
        let Some(dir) = Self::config_dir() else {
            warn!("Could not determine config directory for saving");
            return;
        };
        if let Err(e) = std::fs::create_dir_all(&dir) {
            warn!("Failed to create config directory: {}", e);
            return;
        }
        let path = dir.join(TABS_FILE);
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
        let Some(dir) = Self::config_dir() else {
            warn!("Could not determine config directory");
            return ProgramState::default();
        };
        let path = dir.join(TABS_FILE);
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
                    warn!("Failed to parse tabs file, starting fresh: {}", e);
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
    fn cache_key(cache_id: &str) -> String {
        format!("word_cache_{}", cache_id)
    }
    pub fn write_word_cache(cache_id: &str, words: &[Word]) {
        use gloo_storage::Storage;
        match ron::ser::to_string(words) {
            Ok(content) => {
                if let Err(e) = gloo_storage::LocalStorage::set(&Self::cache_key(cache_id), content) {
                    warn!("Failed to write word cache to localStorage: {:?}", e);
                }
            }
            Err(e) => warn!("Failed to serialize word cache: {}", e),
        }
    }
    pub fn load_word_cache(cache_id: &str) -> Option<Vec<Word>> {
        use gloo_storage::Storage;
        let content: String = gloo_storage::LocalStorage::get(&Self::cache_key(cache_id)).ok()?;
        ron::from_str(&content).ok()
    }
    pub fn delete_word_cache(cache_id: &str) {
        use gloo_storage::Storage;
        gloo_storage::LocalStorage::delete(&Self::cache_key(cache_id));
    }
    fn cleanup_orphan_caches(_valid_ids: &HashSet<String>) {
        // localStorage iteration not available without extra web-sys features.
    }
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
                    warn!("Failed to parse tabs from localStorage, starting fresh: {}", e);
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

    let valid_ids: HashSet<String> = program_state.tabs.iter()
        .map(|t| t.content_cache_id.clone())
        .collect();

    let mut restored = 0;
    for tab in program_state.tabs {
        let Some(words) = ProgramState::load_word_cache(&tab.content_cache_id) else {
            warn!("Cache miss for tab '{}' ({}), skipping", tab.name, tab.content_cache_id);
            continue;
        };

        let content = Content::new_from_loaded(tab.content_cache_id, words, tab.current_index);
        let mut request = TabCreateRequest::new(tab.name, content)
            .with_font(tab.font_name, tab.font_size)
            .with_wpm(tab.wpm)
            .with_active(tab.is_active);

        if let Some(path) = tab.file_path {
            request = request.with_file_path(path);
        }

        commands.trigger(request);
        restored += 1;
    }

    ProgramState::cleanup_orphan_caches(&valid_ids);
    info!("Restored {}/{} tabs from saved state", restored, total_tabs);
}

fn persist_program_state(
    time: Res<Time>,
    mut save_timer: ResMut<TabSaveTimer>,
    app_exit_events: MessageReader<AppExit>,
    tabs: Query<(
        &Name,
        &TabFontSettings,
        &TabWpm,
        &Content,
        Option<&TabFilePath>,
        Has<ActiveTab>,
    ), With<TabMarker>>,
) {
    save_timer.timer.tick(time.delta());
    if !save_timer.timer.just_finished() && app_exit_events.is_empty() { return; }

    let saved_tabs: Vec<SavedTab> = tabs.iter()
        .map(|(name, font_settings, wpm, content, file_path, is_active)| {
            SavedTab {
                name: name.to_string(),
                file_path: file_path.map(|fp| fp.0.clone()),
                font_name: font_settings.font_name.clone(),
                font_size: font_settings.font_size,
                wpm: wpm.0,
                content_cache_id: content.content_cache_id.clone(),
                current_index: content.current_index,
                is_active,
            }
        })
        .collect();

    ProgramState { tabs: saved_tabs }.save();
    info!("The program state was saved");
}
