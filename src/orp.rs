//! ORP (Optical Recognition Point) display system.
//!
//! Renders the current word with the ORP letter highlighted and centered.
//! Uses three text entities (left, center, right) to keep the focus letter fixed.

use bevy::color::palettes::css::RED;
use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::reader::WordChanged;
use crate::tabs::{ActiveTab, Content, ReaderTab, TabFontChanged};

/// Approximate ratio of character width to font size for monospace-like positioning.
/// Used to offset left/right text so they abut the center ORP character.
const CHAR_WIDTH_RATIO: f32 = 0.6;

pub struct OrpPlugin;
impl Plugin for OrpPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_orp_display)
            .add_observer(on_word_changed)
            .add_observer(on_font_changed)
            .add_observer(on_active_tab_changed)
            ;
    }
}

const RETICLE_OFFSET_Y: f32 = 40.0;
const RETICLE_WIDTH: f32 = 3.0;
const RETICLE_HEIGHT: f32 = 40.0;
const RETICLE_ALPHA: f32 = 0.5;

// ============================================================================
// Components
// ============================================================================

#[derive(Component)]
pub struct ReaderDisplay;

#[derive(Component, PartialEq)]
enum OrpSegment {
    Left,
    Center,
    Right,
}

#[derive(Component)]
struct ReticleMarker;

// ============================================================================
// Systems
// ============================================================================

fn setup_orp_display(
    mut commands: Commands,
) {
    let reticle_color = RED.with_alpha(RETICLE_ALPHA);
    let reticle_size = Vec2::new(RETICLE_WIDTH, RETICLE_HEIGHT);
    
    // Top reticle
    commands.spawn((
        Sprite::from_color(reticle_color, reticle_size),
        Transform::from_xyz(0.0, RETICLE_OFFSET_Y, 0.0),
        ReticleMarker,
        ReaderDisplay,
        Visibility::Hidden,
    ));
    // Bottom reticle
    commands.spawn((
        Sprite::from_color(reticle_color, reticle_size),
        Transform::from_xyz(0.0, -RETICLE_OFFSET_Y, 0.0),
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

fn on_active_tab_changed(
    _trigger: On<Add, ActiveTab>,
    active_tab: Single<Entity, With<ActiveTab>>,
    reader_tabs: Query<(), With<ReaderTab>>,
    mut displays: Query<&mut Visibility, With<ReaderDisplay>>,
) {
    let is_reader = reader_tabs.get(active_tab.into_inner()).is_ok();
    let target_visibility = if is_reader { Visibility::Inherited } else { Visibility::Hidden };
    for mut visibility in displays.iter_mut() {
        *visibility = target_visibility;
    }
}

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

fn on_font_changed(
    trigger: On<TabFontChanged>,
    mut segments: Query<(&mut TextFont, &mut Transform, &OrpSegment)>,
) {
    let font_handle = trigger.handle.clone();
    let font_size = trigger.size;
    // half_char = half the estimated width of the center character,
    // so left/right text edges meet the center character's edges.
    let half_char = font_size * CHAR_WIDTH_RATIO * 0.5;
    
    for (mut font, mut transform, segment) in segments.iter_mut() {
        font.font_size = font_size;
        font.font = font_handle.clone();
        match segment {
            OrpSegment::Left => transform.translation.x = -half_char,
            OrpSegment::Center => {},
            OrpSegment::Right => transform.translation.x = half_char,
        }
    }
}
