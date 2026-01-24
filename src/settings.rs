use bevy::prelude::*;
use std::path::PathBuf;

use crate::state::TabManager;

const TABS_FILE: &str = "tabs.ron";
const SAVE_DEBOUNCE_SECS: f32 = 5.0;

#[derive(Resource)]
struct TabSaveTimer {
    timer: Timer,
    pending: bool,
}

impl Default for TabSaveTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(SAVE_DEBOUNCE_SECS, TimerMode::Once),
            pending: false,
        }
    }
}

fn get_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("arre-mind-reader"))
}

fn load_tabs() -> TabManager {
    let Some(config_dir) = get_config_path() else {
        return TabManager::default();
    };
    let path = config_dir.join(TABS_FILE);
    if !path.exists() {
        return TabManager::default();
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => ron::from_str(&content).unwrap_or_default(),
        Err(_) => TabManager::default(),
    }
}

fn save_tabs(tabs: &TabManager) {
    let Some(config_dir) = get_config_path() else { return };
    let _ = std::fs::create_dir_all(&config_dir);
    let path = config_dir.join(TABS_FILE);
    if let Ok(content) = ron::ser::to_string_pretty(tabs, ron::ser::PrettyConfig::default()) {
        let _ = std::fs::write(path, content);
    }
}

fn persist_tabs(
    tabs: Res<TabManager>,
    time: Res<Time>,
    mut save_timer: ResMut<TabSaveTimer>,
) {
    if tabs.is_changed() {
        save_timer.timer.reset();
        save_timer.pending = true;
    }
    
    if save_timer.pending {
        save_timer.timer.tick(time.delta());
        if save_timer.timer.just_finished() {
            save_tabs(&tabs);
            save_timer.pending = false;
        }
    }
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(load_tabs())
            .init_resource::<TabSaveTimer>()
            .add_systems(PostUpdate, persist_tabs);
    }
}
