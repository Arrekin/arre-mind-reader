//! Font management and caching.
//!
//! Scans available fonts from assets/fonts and provides a cache to avoid
//! repeated asset loading. Syncs with `ReaderSettings` for font changes.

use std::collections::HashMap;

use bevy::log::{debug, warn};
use bevy::prelude::*;

#[derive(Resource)]
pub struct FontCache {
    cache: HashMap<String, Handle<Font>>,
    pub current_handle: Handle<Font>,
    pub current_path: String,
}

impl FontCache {
    pub fn get_or_load(&mut self, path: String, asset_server: &AssetServer) -> Handle<Font> {
        if let Some(handle) = self.cache.get(&path) {
            return handle.clone();
        }
        
        let handle: Handle<Font> = asset_server.load(&path);
        self.cache.insert(path, handle.clone());
        handle
    }
    
    pub fn set_current(&mut self, path: String, asset_server: &AssetServer) {
        let handle = self.get_or_load(path.clone(), asset_server);
        self.current_handle = handle;
        self.current_path = path;
    }
}

pub struct FontsPlugin;

impl Plugin for FontsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<AvailableFonts>()
            .add_systems(Startup, (scan_fonts, initialize_font_cache).chain())
            .add_systems(Update, sync_font_from_settings);
    }
}

#[derive(Resource, Default)]
pub struct AvailableFonts {
    pub fonts: Vec<String>,
}

fn scan_fonts(mut available: ResMut<AvailableFonts>) {
    let fonts_dir = std::path::Path::new("assets/fonts");
    match std::fs::read_dir(fonts_dir) {
        Ok(entries) => {
            available.fonts.clear();
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "ttf" || e == "otf") {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        available.fonts.push(format!("fonts/{}", name));
                    }
                }
            }
            available.fonts.sort();
            debug!("Found {} fonts in assets/fonts", available.fonts.len());
        }
        Err(e) => {
            warn!("Could not read fonts directory: {}", e);
        }
    }
}

fn initialize_font_cache(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<crate::state::ReaderSettings>,
) {
    let handle: Handle<Font> = asset_server.load(&settings.font_path);
    let mut cache = HashMap::new();
    cache.insert(settings.font_path.clone(), handle.clone());
    
    commands.insert_resource(FontCache {
        cache,
        current_handle: handle,
        current_path: settings.font_path.clone(),
    });
}

fn sync_font_from_settings(
    settings: Res<crate::state::ReaderSettings>,
    mut font_cache: ResMut<FontCache>,
    asset_server: Res<AssetServer>,
) {
    if settings.is_changed() && font_cache.current_path != settings.font_path {
        font_cache.set_current(settings.font_path.clone(), &asset_server);
    }
}