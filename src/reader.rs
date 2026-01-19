use bevy::prelude::*;
use bevy::sprite::Anchor;
use std::time::Duration;

use crate::state::{FocusModeState, ReaderSettings, ReaderState, ReadingState, Word};

pub struct ReaderPlugin;

impl Plugin for ReaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ReadingState>()
            .init_resource::<ReaderState>()
            .init_resource::<ReaderSettings>()
            .init_resource::<FocusModeState>()
            .init_resource::<ReadingTimer>()
            .add_systems(Startup, (setup_orp_display, load_test_content))
            .add_systems(Update, (
                handle_input,
                tick_reader.run_if(in_state(ReadingState::Active)),
                update_word_display,
            ))
            .add_systems(OnEnter(ReadingState::Active), start_reading);
    }
}

#[derive(Resource, Default)]
pub struct ReadingTimer {
    pub timer: Timer,
}

#[derive(Component)]
pub struct LeftTextMarker;

#[derive(Component)]
pub struct CenterTextMarker;

#[derive(Component)]
pub struct RightTextMarker;

#[derive(Component)]
pub struct ReticleMarker;

// Approximate character width ratio for monospace fonts
const CHAR_WIDTH_RATIO: f32 = 0.6;

fn setup_orp_display(mut commands: Commands) {
    let reticle_color = Color::srgba(1.0, 0.0, 0.0, 0.5);
    let reticle_size = Vec2::new(3.0, 40.0);
    let font_size = 48.0;
    // Half character width for positioning adjacent to center
    let half_char = font_size * CHAR_WIDTH_RATIO * 0.5;
    
    // Top reticle
    commands.spawn((
        Sprite::from_color(reticle_color, reticle_size),
        Transform::from_xyz(0.0, 40.0, 0.0),
        ReticleMarker,
    ));
    // Bottom reticle
    commands.spawn((
        Sprite::from_color(reticle_color, reticle_size),
        Transform::from_xyz(0.0, -40.0, 0.0),
        ReticleMarker,
    ));
    
    // Left text - right edge touches left edge of center char
    commands.spawn((
        Text2d::new(""),
        TextFont {
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
            font_size,
            ..default()
        },
        TextColor(Color::WHITE),
        Anchor::CENTER_LEFT,
        Transform::from_xyz(half_char, 0.0, 0.0),
        RightTextMarker,
    ));
}

fn load_test_content(mut reader_state: ResMut<ReaderState>) {
    let test_text = "The quick brown fox jumps over the lazy dog. \
        This is a test of the speed reading system. \
        It should handle punctuation, like commas, and periods. \
        Can it handle questions? Yes! It can also handle exclamations! \
        \n\nThis is a new paragraph after a double newline. \
        The system should pause longer here. \
        Let's see how it handles longer words like extraordinary or unbelievable.";
    
    reader_state.words = parse_text(test_text);
    reader_state.current_index = 0;
}

pub fn parse_text(text: &str) -> Vec<Word> {
    let mut words = Vec::new();
    let normalized = text.replace("\n\n", " \n\n ").replace("\n", " ");
    let mut is_paragraph_end = false;
    
    for token in normalized.split_whitespace() {
        if token == "\n\n" {
            is_paragraph_end = true;
            continue;
        }
        words.push(Word {
            text: token.to_string(),
            is_paragraph_end,
        });
        is_paragraph_end = false;
    }
    words
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

pub fn calc_delay(word: &Word, wpm: u32) -> Duration {
    let base_ms = 60_000.0 / wpm as f64;
    let mut multiplier = 1.0f64;
    
    let text = &word.text;
    if text.chars().count() > 10 {
        multiplier = multiplier.max(1.3);
    }
    if text.ends_with(',') || text.ends_with(';') {
        multiplier = multiplier.max(2.0);
    }
    if text.ends_with('.') || text.ends_with('?') || text.ends_with('!') {
        multiplier = multiplier.max(3.0);
    }
    if word.is_paragraph_end {
        multiplier = multiplier.max(4.0);
    }
    
    Duration::from_millis((base_ms * multiplier) as u64)
}

fn start_reading(
    mut timer: ResMut<ReadingTimer>,
    reader_state: Res<ReaderState>,
    settings: Res<ReaderSettings>,
) {
    if !reader_state.words.is_empty() {
        let word = &reader_state.words[reader_state.current_index];
        let delay = calc_delay(word, settings.wpm);
        timer.timer = Timer::new(delay, TimerMode::Once);
    }
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<ReadingState>>,
    mut next_state: ResMut<NextState<ReadingState>>,
    mut reader_state: ResMut<ReaderState>,
    mut settings: ResMut<ReaderSettings>,
) {
    // Space: toggle play/pause
    if keyboard.just_pressed(KeyCode::Space) {
        match current_state.get() {
            ReadingState::Idle | ReadingState::Paused => {
                next_state.set(ReadingState::Active);
            }
            ReadingState::Active => {
                next_state.set(ReadingState::Paused);
            }
        }
    }
    
    // Escape: stop
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(ReadingState::Idle);
    }
    
    // R: restart
    if keyboard.just_pressed(KeyCode::KeyR) {
        reader_state.current_index = 0;
    }
    
    // Arrow keys: navigation and WPM
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        reader_state.current_index = reader_state.current_index.saturating_sub(5);
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        reader_state.current_index = (reader_state.current_index + 5)
            .min(reader_state.words.len().saturating_sub(1));
    }
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        settings.wpm = (settings.wpm + 50).min(1000);
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        settings.wpm = settings.wpm.saturating_sub(50).max(100);
    }
}

fn tick_reader(
    time: Res<Time>,
    mut timer: ResMut<ReadingTimer>,
    mut reader_state: ResMut<ReaderState>,
    settings: Res<ReaderSettings>,
    mut next_state: ResMut<NextState<ReadingState>>,
) {
    timer.timer.tick(time.delta());
    
    if timer.timer.just_finished() {
        if reader_state.current_index + 1 < reader_state.words.len() {
            reader_state.current_index += 1;
            let word = &reader_state.words[reader_state.current_index];
            let delay = calc_delay(word, settings.wpm);
            timer.timer = Timer::new(delay, TimerMode::Once);
        } else {
            next_state.set(ReadingState::Idle);
        }
    }
}

fn update_word_display(
    reader_state: Res<ReaderState>,
    settings: Res<ReaderSettings>,
    mut left_q: Query<(&mut Text2d, &mut TextFont), (With<LeftTextMarker>, Without<CenterTextMarker>, Without<RightTextMarker>)>,
    mut center_q: Query<(&mut Text2d, &mut TextFont, &mut TextColor), (With<CenterTextMarker>, Without<LeftTextMarker>, Without<RightTextMarker>)>,
    mut right_q: Query<(&mut Text2d, &mut TextFont), (With<RightTextMarker>, Without<LeftTextMarker>, Without<CenterTextMarker>)>,
) {
    if reader_state.words.is_empty() {
        return;
    }
    
    let word = &reader_state.words[reader_state.current_index];
    let chars: Vec<char> = word.text.chars().collect();
    let orp_index = calculate_orp_index(&word.text);
    
    let left: String = chars[..orp_index].iter().collect();
    let center: String = chars.get(orp_index).map(|c| c.to_string()).unwrap_or_default();
    let right: String = chars.get(orp_index + 1..).map(|s| s.iter().collect()).unwrap_or_default();
    
    if let Ok((mut text, mut font)) = left_q.single_mut() {
        **text = left;
        font.font_size = settings.font_size;
    }
    
    if let Ok((mut text, mut font, mut color)) = center_q.single_mut() {
        **text = center;
        font.font_size = settings.font_size;
        *color = TextColor(settings.highlight_bevy_color());
    }
    
    if let Ok((mut text, mut font)) = right_q.single_mut() {
        **text = right;
        font.font_size = settings.font_size;
    }
}
