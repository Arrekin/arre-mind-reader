//! Font management and caching.
//!
//! Scans available fonts from assets/fonts

use bevy::log::{info, warn};
use bevy::prelude::*;

pub struct FontsPlugin;
impl Plugin for FontsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<FontsStore>()
            .add_systems(Startup, FontsStore::load_fonts)
            ;
    }
}

pub struct FontData {
    pub name: String,
    pub handle: Handle<Font>,
}

#[derive(Resource, Default)]
pub struct FontsStore {
    pub fonts: Vec<FontData>,
}
impl FontsStore {
    pub fn default_font(&self) -> Option<&FontData> {
        self.fonts.first()
    }
    pub fn get_by_name(&self, name: &str) -> Option<&FontData> {
        self.fonts.iter().find(|f| f.name == name)
    }
    fn load_fonts(
        mut fonts_store: ResMut<FontsStore>,
        asset_server: Res<AssetServer>,
    ) {
        let fonts_dir = std::path::Path::new("assets/fonts");
        match std::fs::read_dir(fonts_dir) {
            Ok(entries) => {
                fonts_store.fonts.clear();
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|e| e == "ttf" || e == "otf") {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            fonts_store.fonts.push(FontData {
                                name: name.to_string(),
                                handle: asset_server.load(format!("fonts/{}", name)),
                            });
                        }
                    }
                }
                fonts_store.fonts.sort_by(|a, b| a.name.cmp(&b.name));
                info!("Loaded {} fonts in assets/fonts", fonts_store.fonts.len());
            }
            Err(e) => {
                warn!("Could not read fonts directory: {}", e);
            }
        }
    }
}
