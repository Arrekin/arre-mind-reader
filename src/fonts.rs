use bevy::prelude::*;

pub struct FontsPlugin;
impl Plugin for FontsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<AvailableFonts>()
            .add_systems(Startup, AvailableFonts::scan_fonts)
            ;
    }
}

#[derive(Resource, Default)]
pub struct AvailableFonts {
    pub fonts: Vec<String>,
}
impl AvailableFonts {
    fn scan_fonts(
        mut available: ResMut<AvailableFonts>
    ) {
        let fonts_dir = std::path::Path::new("assets/fonts");
        if let Ok(entries) = std::fs::read_dir(fonts_dir) {
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
        }
    }
}