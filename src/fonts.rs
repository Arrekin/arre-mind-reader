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

/// A loaded font: its filename (used as display name) paired with the Bevy asset handle.
#[derive(Clone)]
pub struct FontData {
    pub name: String,
    pub handle: Handle<Font>,
}

/// Central registry of available fonts. Guaranteed non-empty after startup.
/// The first font in the sorted list serves as the default.
#[derive(Resource, Default)]
pub struct FontsStore {
    fonts: Vec<FontData>,
}
impl FontsStore {
    pub fn default_font(&self) -> &FontData {
        self.fonts.first().expect("FontsStore is guaranteed non-empty")
    }
    pub fn get_by_name(&self, name: &str) -> Option<&FontData> {
        self.fonts.iter().find(|f| f.name == name)
    }
    /// Returns the font matching `name`, falling back to the first loaded font.
    pub fn resolve(&self, name: &str) -> &FontData {
        self.get_by_name(name).unwrap_or_else(|| self.default_font())
    }
    pub fn iter(&self) -> impl Iterator<Item = &FontData> {
        self.fonts.iter()
    }
    /// Loads built-in fonts and (on native) discovers additional .ttf/.otf files
    /// dropped into assets/fonts. Fonts are sorted alphabetically by filename.
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

        assert!(!fonts_store.fonts.is_empty(), "No fonts found in assets/fonts");
        info!("Loaded {} fonts", fonts_store.fonts.len());
    }
}
