//! ORP (Optical Recognition Point) display system.
//!
//! Renders the current word with the ORP letter highlighted and centered.
//! Uses three text entities (left, center, right) to keep the focus letter fixed.

use bevy::color::palettes::css::RED;
use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::reader::WordChanged;
use crate::tabs::{ActiveTab, Content, HomepageTab, ReaderTab, TabFontSettings};

/// Approximate ratio of character width to font size for monospace-like positioning.
/// Used to offset left/right text so they abut the center ORP character.
const CHAR_WIDTH_RATIO: f32 = 0.6;

pub struct OrpPlugin;
impl Plugin for OrpPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_orp_display)
            .add_observer(OrpSegment::on_word_changed)
            .add_observer(OrpSegment::on_font_settings_inserted)
            .add_observer(ReaderDisplay::on_reader_tab_activated)
            .add_observer(ReaderDisplay::on_homepage_tab_activated)
            .add_observer(ReticleMarker::on_font_settings_inserted)
            ;
    }
}

const RETICLE_OFFSET_Y_RATIO: f32 = 0.833;
const RETICLE_WIDTH_RATIO: f32 = 0.0625;
const RETICLE_HEIGHT_RATIO: f32 = 0.833;
const RETICLE_ALPHA: f32 = 0.5;

// ============================================================================
// Components
// ============================================================================

/// Marker on all ORP display entities (reticles and text segments).
/// Used to toggle visibility when switching between reader and homepage tabs.
#[derive(Component)]
pub struct ReaderDisplay;
impl ReaderDisplay {
    /// Shows the ORP display and re-inserts the tab's existing `TabFontSettings`
    fn on_reader_tab_activated(
        _trigger: On<Insert, ActiveTab>,
        mut commands: Commands,
        active_reader: Single<(Entity, &TabFontSettings), (With<ActiveTab>, With<ReaderTab>)>,
        mut displays: Query<&mut Visibility, With<ReaderDisplay>>,
    ) {
        let (entity, font_settings) = active_reader.into_inner();
        for mut visibility in displays.iter_mut() {
            *visibility = Visibility::Inherited;
        }
        commands.entity(entity).insert(TabFontSettings::from_font(&font_settings.font, font_settings.font_size));
        commands.trigger(WordChanged);
    }

    /// Hides the ORP display when a non-reader tab becomes active.
    fn on_homepage_tab_activated(
        _trigger: On<Insert, ActiveTab>,
        _active_homepage: Single<Entity, (With<ActiveTab>, With<HomepageTab>)>,
        mut displays: Query<&mut Visibility, With<ReaderDisplay>>,
    ) {
        for mut visibility in displays.iter_mut() {
            *visibility = Visibility::Hidden;
        }
    }
}

/// Identifies which part of the three-entity word display this entity renders.
#[derive(Component, PartialEq)]
enum OrpSegment {
    Left,
    Center,
    Right,
}
impl OrpSegment {
    /// Splits the current word at the ORP index into three strings and assigns
    /// each to its corresponding text entity.
    fn on_word_changed(
        _trigger: On<WordChanged>,
        active_tab: Single<&Content, With<ActiveTab>>,
        mut segments: Query<(&mut Text2d, &OrpSegment)>,
    ) {
        let Some(word) = active_tab.into_inner().current_word() else { return };
        
        let chars: Vec<char> = word.text.chars().collect();
        let orp_index = word.orp_index();
        
        // Split word into three parts around the ORP letter. The center char stays at x=0,
        // left text grows rightward toward center (Anchor::CenterRight), and right text
        // grows leftward away from center (Anchor::CenterLeft).
        let mut left: String = chars[..orp_index].iter().collect();
        let mut center: String = chars.get(orp_index).map(|c| c.to_string()).unwrap_or_default();
        let mut right: String = chars.get(orp_index + 1..).map(|s| s.iter().collect()).unwrap_or_default();
        
        for (mut text, segment) in segments.iter_mut() {
            **text = match segment {
                OrpSegment::Left => std::mem::take(&mut left),
                OrpSegment::Center => std::mem::take(&mut center),
                OrpSegment::Right => std::mem::take(&mut right),
            };
        }
    }

    /// Single source of truth for applying font to the ORP display.
    /// Updates font handle, size, and repositions Left/Right segments
    /// based on estimated character width.
    fn on_font_settings_inserted(
        _trigger: On<Insert, TabFontSettings>,
        font_settings: Single<&TabFontSettings, With<ActiveTab>>,
        mut segments: Query<(&mut TextFont, &mut Transform, &OrpSegment)>,
    ) {
        // half_char = half the estimated width of the center character,
        // so left/right text edges meet the center character's edges.
        let half_char = font_settings.font_size * CHAR_WIDTH_RATIO * 0.5;

        for (mut font, mut transform, segment) in segments.iter_mut() {
            font.font_size = font_settings.font_size;
            font.font = font_settings.font.handle.clone();
            match segment {
                OrpSegment::Left => transform.translation.x = -half_char,
                OrpSegment::Center => {},
                OrpSegment::Right => transform.translation.x = half_char,
            }
        }
    }
}

/// Visual alignment guides (thin red bars) above and below the ORP letter.
#[derive(Component)]
struct ReticleMarker;
impl ReticleMarker {
    fn on_font_settings_inserted(
        _trigger: On<Insert, TabFontSettings>,
        font_settings: Single<&TabFontSettings, With<ActiveTab>>,
        mut reticles: Query<(&mut Sprite, &mut Transform), With<ReticleMarker>>,
    ) {
        let size = font_settings.font_size;
        let offset_y = size * RETICLE_OFFSET_Y_RATIO;
        let reticle_size = Vec2::new(size * RETICLE_WIDTH_RATIO, size * RETICLE_HEIGHT_RATIO);

        for (mut sprite, mut transform) in reticles.iter_mut() {
            sprite.custom_size = Some(reticle_size);
            let sign = transform.translation.y.signum();
            transform.translation.y = sign * offset_y;
        }
    }
}

// ============================================================================
// Systems
// ============================================================================

fn setup_orp_display(
    mut commands: Commands,
) {
    let default_size = crate::reader::FONT_SIZE_DEFAULT;
    let reticle_color = RED.with_alpha(RETICLE_ALPHA);
    let reticle_size = Vec2::new(default_size * RETICLE_WIDTH_RATIO, default_size * RETICLE_HEIGHT_RATIO);
    let offset_y = default_size * RETICLE_OFFSET_Y_RATIO;
    
    // Top reticle
    commands.spawn((
        Sprite::from_color(reticle_color, reticle_size),
        Transform::from_xyz(0.0, offset_y, 0.0),
        ReticleMarker,
        ReaderDisplay,
        Visibility::Hidden,
    ));
    // Bottom reticle
    commands.spawn((
        Sprite::from_color(reticle_color, reticle_size),
        Transform::from_xyz(0.0, -offset_y, 0.0),
        ReticleMarker,
        ReaderDisplay,
        Visibility::Hidden,
    ));
    
    // Left text - right edge touches left edge of center char
    commands.spawn((
        Text2d::new(""),
        TextColor(Color::WHITE),
        Anchor::CENTER_RIGHT,
        OrpSegment::Left,
        ReaderDisplay,
        Visibility::Hidden,
    ));
    
    // Center text (ORP letter) - fixed at x=0, aligned with reticles
    commands.spawn((
        Text2d::new(""),
        TextColor(RED.into()),
        Anchor::CENTER,
        OrpSegment::Center,
        ReaderDisplay,
        Visibility::Hidden,
    ));
    
    // Right text - left edge touches right edge of center char
    commands.spawn((
        Text2d::new(""),
        TextColor(Color::WHITE),
        Anchor::CENTER_LEFT,
        OrpSegment::Right,
        ReaderDisplay,
        Visibility::Hidden,
    ));
}

