//! Font management and caching.
//!
//! Loads built-in fonts from assets/fonts on all platforms.
//! On native, also discovers additional font files dropped into the assets/fonts directory.

use bevy::log::info;
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

const BUILT_IN_FONTS: &[&str] = &[
    "JetBrainsMono-Regular.ttf",
    "UbuntuMono-Regular.ttf",
];

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
        let mut names: Vec<String> = BUILT_IN_FONTS.iter().map(|&s| s.to_string()).collect();

        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(entries) = std::fs::read_dir("assets/fonts") {
                for entry in entries.flatten() {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if file_name.ends_with(".ttf") || file_name.ends_with(".otf") {
                        if !names.contains(&file_name) {
                            names.push(file_name);
                        }
                    }
                }
            }
        }

        names.sort();

        fonts_store.fonts = names.into_iter().map(|name| {
            let handle = asset_server.load(format!("fonts/{}", name));
            FontData { name, handle }
        }).collect();
        info!("Loaded {} fonts", fonts_store.fonts.len());
    }
}
