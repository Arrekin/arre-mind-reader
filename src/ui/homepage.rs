//! Homepage tile system.
//!
//! Each tile is a Bevy entity with shared components (TilePosition, TileSize, TileVisuals)
//! and a unique marker component. Each tile type has its own system that queries only
//! what it needs.

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
#[allow(dead_code)]
const COLOR_STATS: egui::Color32 = egui::Color32::from_rgb(56, 68, 82);
const COLOR_TIPS: egui::Color32 = egui::Color32::from_rgb(72, 62, 48);
const COLOR_TILE_TEXT: egui::Color32 = egui::Color32::from_rgb(187, 197, 214);
const WEBSITE_PLACEHOLDER_URL: &str = "https://arrekin.com/?utm_source=arre-mind-reader";

// ── Shared tile components ──────────────────────────────────────────────────

#[derive(Component)]
/// Center-relative tile offset. Values are interpreted as deltas from the
/// current egui content rect center to the tile center.
pub struct TilePosition(pub Vec2);
impl TilePosition {
    fn to_absolute_top_left(&self, ctx: &egui::Context, size: &TileSize) -> egui::Pos2 {
        let center = ctx.content_rect().center();
        egui::pos2(
            center.x + self.0.x - size.0.x * 0.5,
            center.y + self.0.y - size.0.y * 0.5,
        )
    }
}

#[derive(Component)]
pub struct TileSize(pub Vec2);

#[derive(Component)]
pub struct TileVisuals {
    pub title: &'static str,
    pub color: egui::Color32,
}

#[derive(Component, Default)]
pub struct HomepageTile;
impl HomepageTile {
    /// Run condition: returns true when the homepage tab is active.
    pub fn is_active(
        query: Option<Single<(), (With<HomepageTab>, With<ActiveTab>)>>,
    ) -> bool {
        query.is_some()
    }

    pub fn spawn(mut commands: Commands) {
        // TilePosition is center-relative tile-center offset.
        // These values match the current visual layout while keeping the tile group
        // centered automatically when the window is resized.
        commands.spawn((
            AboutTile,
            TilePosition(Vec2::new(0.0, -94.0)),
            TileSize(Vec2::new(380.0, 380.0)),
            TileVisuals { title: "About", color: COLOR_ABOUT },
        ));
        commands.spawn((
            FontSettingsTile,
            TilePosition(Vec2::new(400.0, -94.0)),
            TileSize(Vec2::new(260.0, 220.0)),
            TileVisuals { title: "Default Tab Settings", color: COLOR_FONT },
        ));
        commands.spawn((
            ShortcutsTile,
            TilePosition(Vec2::new(-400.0, -200.0)),
            TileSize(Vec2::new(200.0, 120.0)),
            TileVisuals { title: "Keyboard Shortcuts", color: COLOR_SHORTCUTS },
        ));
        // commands.spawn((
        //     StatsTile,
        //     TilePosition(Vec2::new(0.0, 164.0)),
        //     TileSize(Vec2::new(220.0, 180.0)),
        //     TileVisuals { title: "Reading Stats", color: COLOR_STATS },
        // ));
        commands.spawn((
            TipsTile,
            TilePosition(Vec2::new(-400.0, 0.)),
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
#[require(HomepageTile)]
pub struct AboutTile;
impl AboutTile {
    pub fn update(
        mut contexts: EguiContexts,
        tile: Single<(&TilePosition, &TileSize, &TileVisuals), With<AboutTile>>,
    ) {
        let Ok(ctx) = contexts.ctx_mut() else { return };
        let (position, size, visuals) = tile.into_inner();
        tile_frame(ctx, "about", position, size, visuals, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(
                    egui::RichText::new("Arre Mind Reader")
                        .size(26.0)
                        .strong()
                        .color(egui::Color32::from_rgb(238, 244, 255)),
                );
            });
            ui.add_space(8.0);
            ui.label("Read faster with RSVP (Rapid Serial Visual Presentation).");
            ui.add_space(12.0);

            ui.strong("How it works?");
            ui.add_space(4.0);
            ui.label("Your eyes stay anchored to a fixed point while words flow");
            ui.label("at your chosen speed, elevating your reading experience");
            ui.label("until your inner voice quiets and you enter");
            ui.label("the realm of frictionless comprehension.");
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.label(
                        egui::RichText::new("* Training required. Results may vary.")
                            .small()
                            .italics(),
                    );
                });
            });
            ui.add_space(6.0);

            ui.strong("Our Motto");
            ui.add_space(4.0);
            ui.label("Read. Increase the WPM. Repeat.");
            ui.add_space(10.0);

            ui.strong("How do I start?");
            ui.add_space(4.0);
            ui.label("1. Click + New and open a text");
            ui.label("2. Start around 250-350 WPM");
            ui.label("3. Increase by +50 WPM when comprehension stays solid");
            ui.add_space(10.0);
            ui.label(
                egui::RichText::new("\"Telepathy was hard, so I built RSVP. It's close enough.\" ~ Arrekin")
                    .italics()
                    .strong()
                    .color(egui::Color32::from_rgb(223, 223, 105)),
            );
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(6.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.hyperlink_to("Arrekin.com", WEBSITE_PLACEHOLDER_URL);
                ui.label(
                    egui::RichText::new(format!("| v{}", env!("CARGO_PKG_VERSION")))
                        .color(egui::Color32::from_rgb(170, 182, 198)),
                );
            });
        });
    }
}

#[derive(Component)]
#[require(HomepageTile)]
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
#[require(HomepageTile)]
pub struct ShortcutsTile;
impl ShortcutsTile {
    pub fn update(
        mut contexts: EguiContexts,
        tile: Single<(&TilePosition, &TileSize, &TileVisuals), With<ShortcutsTile>>,
    ) {
        let Ok(ctx) = contexts.ctx_mut() else { return };
        let (position, size, visuals) = tile.into_inner();
        let wpm_adjust_description = format!("Adjust WPM ±{}", WPM_STEP);
        tile_frame(ctx, "shortcuts", position, size, visuals, |ui| {
            Self::shortcut_row(ui, "Space", "Play / Pause");
            Self::shortcut_row(ui, "← / →", "Skip 5 words");
            Self::shortcut_row(ui, "↑ / ↓", &wpm_adjust_description);
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
#[require(HomepageTile)]
#[allow(dead_code)]
pub struct StatsTile;
#[allow(dead_code)]
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
#[require(HomepageTile)]
pub struct TipsTile;
impl TipsTile {
    pub fn update(
        mut contexts: EguiContexts,
        tile: Single<(&TilePosition, &TileSize, &TileVisuals), With<TipsTile>>,
    ) {
        let Ok(ctx) = contexts.ctx_mut() else { return };
        let (position, size, visuals) = tile.into_inner();
        tile_frame(ctx, "tips", position, size, visuals, |ui| {
            ui.label("💡 Start around 250-350 WPM. Increase only when comprehension stays easy.");
            ui.add_space(8.0);
            ui.label("💡 If focus slips, drop WPM by 50 and continue.");
            ui.add_space(8.0);
            ui.label("💡 Take short breaks every 15-20 minutes to reduce eye strain.");
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                ui.label("💡 Lost thread? Use");
                ui.label(egui::RichText::new("←/→").monospace());
                ui.label("to recover context.");
            });
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
        .fixed_pos(position.to_absolute_top_left(ctx, size))
        .show(ctx, |ui| {
            egui::Frame::NONE
                .fill(visuals.color)
                .corner_radius(egui::CornerRadius::same(TILE_ROUNDING))
                .inner_margin(egui::Margin::same(TILE_INNER_MARGIN))
                .show(ui, |ui| {
                    ui.visuals_mut().override_text_color = Some(COLOR_TILE_TEXT);
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
