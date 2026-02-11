//! Homepage tile system.
//!
//! Each tile is a Bevy entity with shared components (TilePosition, TileSize, TileVisuals)
//! and a unique marker component. Each tile type has its own system that queries only
//! what it needs. See task.md for full architecture rationale.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::fonts::FontsStore;
use crate::reader::{FONT_SIZE_MIN, FONT_SIZE_MAX, WPM_MIN, WPM_MAX, WPM_STEP};
use crate::tabs::{ActiveTab, ApplyDefaultsToAll, DefaultTabSettings, HomepageTab};

const TILE_ROUNDING: u8 = 6;
const TILE_INNER_MARGIN: i8 = 12;
const COLOR_ABOUT: egui::Color32 = egui::Color32::from_rgb(45, 55, 72);
const COLOR_FONT: egui::Color32 = egui::Color32::from_rgb(56, 78, 56);
const COLOR_SHORTCUTS: egui::Color32 = egui::Color32::from_rgb(78, 56, 72);
const COLOR_STATS: egui::Color32 = egui::Color32::from_rgb(56, 68, 82);
const COLOR_TIPS: egui::Color32 = egui::Color32::from_rgb(72, 62, 48);

// ── Shared tile components ──────────────────────────────────────────────────

#[derive(Component)]
pub struct TilePosition(pub Vec2);

#[derive(Component)]
pub struct TileSize(pub Vec2);

#[derive(Component)]
pub struct TileVisuals {
    pub title: &'static str,
    pub color: egui::Color32,
}

#[derive(Component)]
pub struct HomepageTile;
impl HomepageTile {
    /// Run condition: returns true when the homepage tab is active.
    pub fn is_active(
        query: Option<Single<Entity, (With<HomepageTab>, With<ActiveTab>)>>,
    ) -> bool {
        query.is_some()
    }

    pub fn spawn(mut commands: Commands) {
        // Row 1: About (360×200) + Default Tab Settings (260×320)
        // Row 1 height = 320, About vertically centered: y_offset = (320-200)/2 = 60
        // Row 1 total width: 360 + 8 + 260 = 628, centered in 1280: x_start = 326
        commands.spawn((HomepageTile, AboutTile,
            TilePosition(Vec2::new(326.0, 166.0)),
            TileSize(Vec2::new(360.0, 200.0)),
            TileVisuals { title: "About", color: COLOR_ABOUT },
        ));
        commands.spawn((HomepageTile, FontSettingsTile,
            TilePosition(Vec2::new(694.0, 106.0)),
            TileSize(Vec2::new(260.0, 320.0)),
            TileVisuals { title: "Default Tab Settings", color: COLOR_FONT },
        ));
        // Row 2: Shortcuts (260×180) + Stats (220×180) + Tips (300×180)
        // Row 2 starts at 106 + 320 + 8 = 434
        // Row 2 total width: 260 + 8 + 220 + 8 + 300 = 796, centered in 1280: x_start = 242
        commands.spawn((HomepageTile, ShortcutsTile,
            TilePosition(Vec2::new(242.0, 434.0)),
            TileSize(Vec2::new(260.0, 180.0)),
            TileVisuals { title: "Keyboard Shortcuts", color: COLOR_SHORTCUTS },
        ));
        commands.spawn((HomepageTile, StatsTile,
            TilePosition(Vec2::new(510.0, 434.0)),
            TileSize(Vec2::new(220.0, 180.0)),
            TileVisuals { title: "Reading Stats", color: COLOR_STATS },
        ));
        commands.spawn((HomepageTile, TipsTile,
            TilePosition(Vec2::new(738.0, 434.0)),
            TileSize(Vec2::new(300.0, 180.0)),
            TileVisuals { title: "Tips", color: COLOR_TIPS },
        ));
    }

    /// Draws an empty `CentralPanel` so egui captures background clicks
    /// and the window has a consistent fill behind the tiles.
    pub fn background(mut contexts: EguiContexts) {
        let Ok(ctx) = contexts.ctx_mut() else { return };
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |_ui| {});
    }
}

// ── Per-tile types ──────────────────────────────────────────────────────────

#[derive(Component)]
pub struct AboutTile;
impl AboutTile {
    pub fn update(
        mut contexts: EguiContexts,
        tile: Single<(&TilePosition, &TileSize, &TileVisuals), With<AboutTile>>,
    ) {
        let Ok(ctx) = contexts.ctx_mut() else { return };
        let (position, size, visuals) = tile.into_inner();
        tile_frame(ctx, "about", position, size, visuals, |ui| {
            ui.heading("Arre Mind Reader");
            ui.add_space(8.0);
            ui.label("A speed-reading app using RSVP (Rapid Serial Visual Presentation).");
            ui.add_space(4.0);
            ui.label("Words are displayed one at a time at your chosen speed, \
                with a fixed eye fixation point for optimal reading flow.");
            ui.add_space(12.0);
            ui.label(egui::RichText::new("Open a file or paste text using the '+ New' button above.")
                .italics()
                .color(egui::Color32::from_rgb(160, 170, 180)));
        });
    }
}

#[derive(Component)]
pub struct FontSettingsTile;
impl FontSettingsTile {
    pub fn update(
        mut commands: Commands,
        mut contexts: EguiContexts,
        fonts: Res<FontsStore>,
        mut defaults: ResMut<DefaultTabSettings>,
        tile: Single<(&TilePosition, &TileSize, &TileVisuals), With<FontSettingsTile>>,
    ) {
        let Ok(ctx) = contexts.ctx_mut() else { return };
        let (position, size, visuals) = tile.into_inner();

        let effective_font_name = defaults.font_name.clone();

        tile_frame(ctx, "font_settings", position, size, visuals, |ui| {
            ui.label("Font:");
            ui.add_space(4.0);
            egui::ComboBox::from_id_salt("default_font")
                .selected_text(&effective_font_name)
                .width(ui.available_width() - 16.0)
                .show_ui(ui, |ui| {
                    for font_data in fonts.iter() {
                        if ui.selectable_label(
                            effective_font_name == font_data.name,
                            &font_data.name,
                        ).clicked() {
                            defaults.font_name = font_data.name.clone();
                        }
                    }
                });

            ui.add_space(8.0);
            ui.label("Font Size:");
            ui.add(egui::Slider::new(&mut defaults.font_size, FONT_SIZE_MIN..=FONT_SIZE_MAX)
                .suffix(" px"));

            ui.add_space(8.0);
            ui.label("WPM:");
            ui.add(egui::Slider::new(&mut defaults.wpm, WPM_MIN..=WPM_MAX)
                .step_by(WPM_STEP as f64)
                .suffix(" wpm"));

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);
            if ui.button("Apply to all tabs").clicked() {
                commands.trigger(ApplyDefaultsToAll);
            }
        });
    }
}

#[derive(Component)]
pub struct ShortcutsTile;
impl ShortcutsTile {
    pub fn update(
        mut contexts: EguiContexts,
        tile: Single<(&TilePosition, &TileSize, &TileVisuals), With<ShortcutsTile>>,
    ) {
        let Ok(ctx) = contexts.ctx_mut() else { return };
        let (position, size, visuals) = tile.into_inner();
        tile_frame(ctx, "shortcuts", position, size, visuals, |ui| {
            Self::shortcut_row(ui, "Space", "Play / Pause");
            Self::shortcut_row(ui, "Escape", "Stop");
            Self::shortcut_row(ui, "← / →", "Skip 5 words");
            Self::shortcut_row(ui, "↑ / ↓", "Adjust WPM ±50");
            Self::shortcut_row(ui, "R", "Restart");
        });
    }

    fn shortcut_row(ui: &mut egui::Ui, key: &str, description: &str) {
        ui.horizontal(|ui| {
            ui.monospace(egui::RichText::new(format!("{:>9}", key))
                .color(egui::Color32::from_rgb(200, 200, 140)));
            ui.label(description);
        });
    }
}

#[derive(Component)]
pub struct StatsTile;
impl StatsTile {
    pub fn update(
        mut contexts: EguiContexts,
        tile: Single<(&TilePosition, &TileSize, &TileVisuals), With<StatsTile>>,
    ) {
        let Ok(ctx) = contexts.ctx_mut() else { return };
        let (position, size, visuals) = tile.into_inner();
        tile_frame(ctx, "stats", position, size, visuals, |ui| {
            Self::stat_row(ui, "Total words read", "12,847");
            Self::stat_row(ui, "Sessions", "23");
            Self::stat_row(ui, "Avg WPM", "342");
            Self::stat_row(ui, "Books finished", "2");
        });
    }

    fn stat_row(ui: &mut egui::Ui, label: &str, value: &str) {
        ui.horizontal(|ui| {
            ui.label(label);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.strong(egui::RichText::new(value)
                    .color(egui::Color32::from_rgb(140, 200, 200)));
            });
        });
    }

}

#[derive(Component)]
pub struct TipsTile;
impl TipsTile {
    pub fn update(
        mut contexts: EguiContexts,
        tile: Single<(&TilePosition, &TileSize, &TileVisuals), With<TipsTile>>,
    ) {
        let Ok(ctx) = contexts.ctx_mut() else { return };
        let (position, size, visuals) = tile.into_inner();
        tile_frame(ctx, "tips", position, size, visuals, |ui| {
            ui.label("💡 Start with a lower WPM and gradually increase as you get comfortable.");
            ui.add_space(8.0);
            ui.label("💡 Monospace fonts work best — the fixation point stays aligned.");
            ui.add_space(8.0);
            ui.label("💡 Take breaks every 15–20 minutes for better retention.");
        });
    }
}

// ── Shared frame helper ─────────────────────────────────────────────────────

/// Renders the shared chrome for a homepage tile: positioned `egui::Area` with
/// colored background, rounded corners, title heading, and separator.
fn tile_frame(
    ctx: &egui::Context,
    id: &str,
    position: &TilePosition,
    size: &TileSize,
    visuals: &TileVisuals,
    content: impl FnOnce(&mut egui::Ui),
) {
    egui::Area::new(egui::Id::new(id))
        .fixed_pos(egui::pos2(position.0.x, position.0.y))
        .show(ctx, |ui| {
            egui::Frame::NONE
                .fill(visuals.color)
                .corner_radius(egui::CornerRadius::same(TILE_ROUNDING))
                .inner_margin(egui::Margin::same(TILE_INNER_MARGIN))
                .show(ui, |ui| {
                    ui.set_min_size(egui::vec2(size.0.x, size.0.y));
                    ui.set_max_size(egui::vec2(size.0.x, size.0.y));
                    ui.heading(egui::RichText::new(visuals.title)
                        .color(egui::Color32::WHITE).strong());
                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(6.0);
                    content(ui);
                });
        });
}
