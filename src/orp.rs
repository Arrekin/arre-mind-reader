//! ORP (Optical Recognition Point) display system.
//!
//! Renders the current word with the ORP letter highlighted and centered.
//! Uses three text entities (left, center, right) to keep the focus letter fixed.

use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::fonts::FontCache;
use crate::state::constants::*;
use crate::state::{ReaderSettings, ReaderState, TabManager};

#[derive(Component)]
pub struct LeftTextMarker;

#[derive(Component)]
pub struct CenterTextMarker;

#[derive(Component)]
pub struct RightTextMarker;

#[derive(Component)]
pub struct ReticleMarker;

pub fn setup_orp_display(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<ReaderSettings>,
) {
    let reticle_color = Color::srgba(1.0, 0.0, 0.0, RETICLE_ALPHA);
    let reticle_size = Vec2::new(RETICLE_WIDTH, RETICLE_HEIGHT);
    let font_size = settings.font_size;
    let font: Handle<Font> = asset_server.load(&settings.font_path);
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
        TextColor(Color::srgb(1.0, 0.0, 0.0)),
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

pub fn calculate_orp_index(word: &str) -> usize {
    match word.chars().count() {
        0 => 0,
        1 => 0,
        2..=5 => 1,
        6..=9 => 2,
        10..=13 => 3,
        _ => 4,
    }
}

pub fn update_word_display(
    reader_state: Res<ReaderState>,
    tabs: Res<TabManager>,
    settings: Res<ReaderSettings>,
    font_cache: Res<FontCache>,
    mut left_q: Query<(&mut Text2d, &mut TextFont), (With<LeftTextMarker>, Without<CenterTextMarker>, Without<RightTextMarker>)>,
    mut center_q: Query<(&mut Text2d, &mut TextFont, &mut TextColor), (With<CenterTextMarker>, Without<LeftTextMarker>, Without<RightTextMarker>)>,
    mut right_q: Query<(&mut Text2d, &mut TextFont), (With<RightTextMarker>, Without<LeftTextMarker>, Without<CenterTextMarker>)>,
) {
    let Some(tab) = tabs.active_tab() else { return };
    if tab.words.is_empty() {
        return;
    }
    
    let index = reader_state.current_index.min(tab.words.len().saturating_sub(1));
    let word = &tab.words[index];
    let chars: Vec<char> = word.text.chars().collect();
    let orp_index = calculate_orp_index(&word.text);
    
    let left: String = chars[..orp_index].iter().collect();
    let center: String = chars.get(orp_index).map(|c| c.to_string()).unwrap_or_default();
    let right: String = chars.get(orp_index + 1..).map(|s| s.iter().collect()).unwrap_or_default();
    
    let font_handle = font_cache.current_handle.clone();
    
    if let Ok((mut text, mut font)) = left_q.single_mut() {
        **text = left;
        font.font_size = settings.font_size;
        font.font = font_handle.clone();
    }
    
    if let Ok((mut text, mut font, mut color)) = center_q.single_mut() {
        **text = center;
        font.font_size = settings.font_size;
        font.font = font_handle.clone();
        *color = TextColor(settings.highlight_bevy_color());
    }
    
    if let Ok((mut text, mut font)) = right_q.single_mut() {
        **text = right;
        font.font_size = settings.font_size;
        font.font = font_handle;
    }
}
