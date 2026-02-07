//! ORP (Optical Recognition Point) display system.
//!
//! Renders the current word with the ORP letter highlighted and centered.
//! Uses three text entities (left, center, right) to keep the focus letter fixed.

use bevy::color::palettes::css::RED;
use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::fonts::FontsStore;
use crate::reader::FONT_SIZE_DEFAULT;
use crate::tabs::{ActiveTab, TabFontSettings, WordsManager};

/// Approximate ratio of character width to font size for monospace-like positioning.
/// Used to offset left/right text so they abut the center ORP character.
const CHAR_WIDTH_RATIO: f32 = 0.6;

pub struct OrpPlugin;
impl Plugin for OrpPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_orp_display)
            .add_systems(Update, update_word_display)
            ;
    }
}

pub const RETICLE_OFFSET_Y: f32 = 40.0;
pub const RETICLE_WIDTH: f32 = 3.0;
pub const RETICLE_HEIGHT: f32 = 40.0;
pub const RETICLE_ALPHA: f32 = 0.5;

// ============================================================================
// Components
// ============================================================================

#[derive(Component)]
struct LeftTextMarker;

#[derive(Component)]
struct CenterTextMarker;

#[derive(Component)]
struct RightTextMarker;

#[derive(Component)]
struct ReticleMarker;

// ============================================================================
// Systems
// ============================================================================

fn setup_orp_display(
    mut commands: Commands,
    fonts: Res<FontsStore>,
) {
    let reticle_color = RED.with_alpha(RETICLE_ALPHA);
    let reticle_size = Vec2::new(RETICLE_WIDTH, RETICLE_HEIGHT);
    let font_size = FONT_SIZE_DEFAULT;
    let font = fonts.default_font().map(|f| f.handle.clone()).unwrap_or_default();
    let half_char = font_size * CHAR_WIDTH_RATIO * 0.5;
    
    // Top reticle
    commands.spawn((
        Sprite::from_color(reticle_color, reticle_size),
        Transform::from_xyz(0.0, RETICLE_OFFSET_Y, 0.0),
        ReticleMarker,
    ));
    // Bottom reticle
    commands.spawn((
        Sprite::from_color(reticle_color, reticle_size),
        Transform::from_xyz(0.0, -RETICLE_OFFSET_Y, 0.0),
        ReticleMarker,
    ));
    
    // Left text - right edge touches left edge of center char
    commands.spawn((
        Text2d::new(""),
        TextFont {
            font: font.clone(),
            font_size,
            ..default()
        },
        TextColor(Color::WHITE),
        Anchor::CENTER_RIGHT,
        Transform::from_xyz(-half_char, 0.0, 0.0),
        LeftTextMarker,
    ));
    
    // Center text (ORP letter) - fixed at x=0, aligned with reticles
    commands.spawn((
        Text2d::new(""),
        TextFont {
            font: font.clone(),
            font_size,
            ..default()
        },
        TextColor(RED.into()),
        Anchor::CENTER,
        Transform::from_xyz(0.0, 0.0, 0.0),
        CenterTextMarker,
    ));
    
    // Right text - left edge touches right edge of center char
    commands.spawn((
        Text2d::new(""),
        TextFont {
            font,
            font_size,
            ..default()
        },
        TextColor(Color::WHITE),
        Anchor::CENTER_LEFT,
        Transform::from_xyz(half_char, 0.0, 0.0),
        RightTextMarker,
    ));
}

fn update_word_display(
    active_tabs: Query<(&TabFontSettings, &WordsManager), With<ActiveTab>>,
    left_texts: Single<(&mut Text2d, &mut TextFont, &mut Transform), (With<LeftTextMarker>, Without<CenterTextMarker>, Without<RightTextMarker>)>,
    center_texts: Single<(&mut Text2d, &mut TextFont), (With<CenterTextMarker>, Without<LeftTextMarker>, Without<RightTextMarker>)>,
    right_texts: Single<(&mut Text2d, &mut TextFont, &mut Transform), (With<RightTextMarker>, Without<LeftTextMarker>, Without<CenterTextMarker>)>,
) {
    let Ok((font_settings, words_mgr)) = active_tabs.single() else { return };
    let Some(word) = words_mgr.current_word() else { return };
    let chars: Vec<char> = word.text.chars().collect();
    let orp_index = word.orp_index();
    
    // Split word into three parts around the ORP letter. The center char stays at x=0,
    // left text grows rightward toward center (Anchor::CenterRight), and right text
    // grows leftward away from center (Anchor::CenterLeft).
    let left: String = chars[..orp_index].iter().collect();
    let center: String = chars.get(orp_index).map(|c| c.to_string()).unwrap_or_default();
    let right: String = chars.get(orp_index + 1..).map(|s| s.iter().collect()).unwrap_or_default();
    
    let font_handle = font_settings.font_handle.clone();
    let font_size = font_settings.font_size;
    // half_char = half the estimated width of the center character,
    // so left/right text edges meet the center character's edges.
    let half_char = font_size * CHAR_WIDTH_RATIO * 0.5;
    
    let (mut text, mut font, mut transform) = left_texts.into_inner();
    **text = left;
    font.font_size = font_size;
    font.font = font_handle.clone();
    transform.translation.x = -half_char;
    
    let (mut text, mut font) = center_texts.into_inner();
    **text = center;
    font.font_size = font_size;
    font.font = font_handle.clone();
    
    let (mut text, mut font, mut transform) = right_texts.into_inner();
    **text = right;
    font.font_size = font_size;
    font.font = font_handle;
    transform.translation.x = half_char;
}
