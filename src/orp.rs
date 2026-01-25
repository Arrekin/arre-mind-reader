//! ORP (Optical Recognition Point) display system.
//!
//! Renders the current word with the ORP letter highlighted and centered.
//! Uses three text entities (left, center, right) to keep the focus letter fixed.

use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::fonts::FontsStore;
use crate::state::constants::*;
use crate::state::{ActiveTab, TabFontSettings, WordsManager};

pub struct OrpPlugin;
impl Plugin for OrpPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_orp_display)
            .add_systems(Update, update_word_display)
            ;
    }
}

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
    let reticle_color = Color::srgba(HIGHLIGHT_COLOR.0, HIGHLIGHT_COLOR.1, HIGHLIGHT_COLOR.2, RETICLE_ALPHA);
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
        TextColor(Color::srgb(HIGHLIGHT_COLOR.0, HIGHLIGHT_COLOR.1, HIGHLIGHT_COLOR.2)),
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
    active_tab: Res<ActiveTab>,
    tabs: Query<(&TabFontSettings, &WordsManager)>,
    mut left_texts: Query<(&mut Text2d, &mut TextFont), (With<LeftTextMarker>, Without<CenterTextMarker>, Without<RightTextMarker>)>,
    mut center_texts: Query<(&mut Text2d, &mut TextFont), (With<CenterTextMarker>, Without<LeftTextMarker>, Without<RightTextMarker>)>,
    mut right_texts: Query<(&mut Text2d, &mut TextFont), (With<RightTextMarker>, Without<LeftTextMarker>, Without<CenterTextMarker>)>,
) {
    let Some(entity) = active_tab.entity else { return };
    let Ok((font_settings, words_mgr)) = tabs.get(entity) else { return };
    if words_mgr.words.is_empty() {
        return;
    }
    
    let index = words_mgr.current_index.min(words_mgr.words.len().saturating_sub(1));
    let word = &words_mgr.words[index];
    let chars: Vec<char> = word.text.chars().collect();
    let orp_index = word.orp_index();
    
    let left: String = chars[..orp_index].iter().collect();
    let center: String = chars.get(orp_index).map(|c| c.to_string()).unwrap_or_default();
    let right: String = chars.get(orp_index + 1..).map(|s| s.iter().collect()).unwrap_or_default();
    
    let font_handle = font_settings.font_handle.clone();
    let font_size = font_settings.font_size;
    
    if let Ok((mut text, mut font)) = left_texts.single_mut() {
        **text = left;
        font.font_size = font_size;
        font.font = font_handle.clone();
    }
    
    if let Ok((mut text, mut font)) = center_texts.single_mut() {
        **text = center;
        font.font_size = font_size;
        font.font = font_handle.clone();
    }
    
    if let Ok((mut text, mut font)) = right_texts.single_mut() {
        **text = right;
        font.font_size = font_size;
        font.font = font_handle;
    }
}
