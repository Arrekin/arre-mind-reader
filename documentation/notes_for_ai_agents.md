# Notes for AI Agents

## Project Overview
Arre Mind Reader is a speed-reading application built with Bevy 0.18 and Rust. It implements RSVP (Rapid Serial Visual Presentation) reading method.

## Architecture
- **Core Engine:** Bevy 0.18 (Rust)
- **UI Overlay:** bevy_egui 0.39 for egui-based panels
- **Rendering:** `Text2d` in world space for the reader display
- **Data Flow:** Tab entities own all state via components → ORP display reads active tab's components

## Key Concepts
- **ORP (Optical Recognition Point):** The letter the eye fixates on, positioned at screen center (0,0). Research shows slightly left-of-center is optimal.
- **Split-text rendering:** Words split into left/center/right using `Anchor::CenterRight` and `Anchor::CenterLeft` to grow outward from center.
- **ECS Tab Modeling:** Each tab is an entity with components: `TabMarker`, `Name`, `TabFontSettings`, `TabWpm`, `WordsManager`, optional `TabFilePath`, and `ActiveTab` marker for the currently active tab.
- **Per-tab settings:** Font and WPM are stored per-tab, not globally. Highlight color is hardcoded red.

## Module Structure
Each file follows: imports → Plugin definition → constants → types/components → systems

- `main.rs` - App entry point, plugin registration
- `reader.rs` - `ReaderPlugin` orchestrates sub-plugins, manages `ReadingState` transitions and timing tick. Contains WPM/font constants
- `tabs.rs` - `TabsPlugin` with tab components (`TabMarker`, `TabFontSettings`, `TabWpm`, `TabFilePath`, `WordsManager`, `ActiveTab`), `TabOrder` resource (encapsulated, exposes `entities()` and `find_adjacent()`), `WordsManager` with encapsulated API (`has_words()`, `current_word()`, `progress()`, `advance()`, `skip_forward/backward()`, `restart()`), entity events (`TabSelect`, `TabClose`), `TabCreateRequest` event with builder pattern, and observers for reactive tab management
- `playback.rs` - `PlaybackPlugin` with `PlaybackCommand` message enum (Play/Pause/Stop/etc.) and `PlaybackCommand::process` system
- `orp.rs` - `OrpPlugin` with display entity setup, word display updates with reactive font size positioning (hardcoded red highlight)
- `input.rs` - `InputPlugin` emits `PlaybackCommand` messages from keyboard input
- `text.rs` - `TextParser` trait, `TxtParser` implementation, `get_parser_for_path()` registry function, `Word` struct with `new()` constructor and ORP/duration methods
- `fonts.rs` - `FontsPlugin` with `FontsStore` resource, scans assets/fonts at startup
- `persistence.rs` - `PersistencePlugin` with RON format save/load, triggers `TabCreateRequest` events on load
- `ui/` - UI module directory:
  - `mod.rs` - `UiPlugin` registration
  - `tab_bar.rs` - Tab strip rendering, emits `TabSelect`/`TabClose` events
  - `controls.rs` - Playback controls, progress, WPM slider, font selector
  - `dialogs.rs` - New tab dialog, async file loading with `get_parser_for_path()`, emits `TabCreateRequest` events

## ECS Event Patterns
- **Tab events** use `EntityEvent` pattern with separate structs (`TabSelect`, `TabClose`) for entity-targeted events
- **Playback** uses `Message` enum (`PlaybackCommand`) for buffered events processed by central system
- **UI emits events/commands** rather than directly mutating state - observers and systems react to events
- Bevy 0.18 uses `Message`/`MessageWriter`/`MessageReader` for buffered events (not `Event`/`EventWriter`/`EventReader`)

## Dependencies
- `bevy` 0.18 - Game engine
- `bevy_egui` 0.39 - Egui integration for settings panel
- `serde` + `ron` - Persistence
- `rfd` 0.15 - Native file dialogs

## Bevy 0.18 Patterns
- `Camera2d` component (not bundle)
- `Text2d::new()` + `TextFont` + `TextColor` + `Anchor` for 2D text
- `Sprite::from_color()` for simple rectangles
- `children![]` macro for hierarchies
- `EguiPlugin::default()` (has struct fields now)
- States use `init_state::<T>()` pattern

## Keyboard Shortcuts
- `Space` - Play/Pause
- `Escape` - Stop, show UI
- `←/→` - Skip 5 words back/forward
- `↑/↓` - Adjust WPM ±50
- `R` - Restart

## Word Methods
ORP index calculation (`Word::orp_index()`):
```rust
match self.text.chars().count() {
    0 | 1 => 0,
    2..=5 => 1,
    6..=9 => 2,
    10..=13 => 3,
    _ => 4,
}
```

Display duration (`Word::display_duration_ms(wpm)`) - uses max multiplier, not cumulative:
- Base: `60000 / WPM` ms
- Long word (>10 chars): ×1.3
- Comma/semicolon: ×2.0
- Period/question/exclamation: ×3.0
- Paragraph end: ×4.0

## Code Style
- Query variables use plural form (e.g., `tabs`, `left_texts`), not `_q` suffix
- Behavior lives with data: `Word` has `new()`, `orp_index()` and `display_duration_ms()` methods; `WordsManager` encapsulates word navigation; `TabOrder` encapsulates entity ordering
- Each module with a plugin defines it near the top after imports
- Don't put newlines between struct and its impl blocks

### Plugin code style
```rust
pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, handle_input)
            ;
    }
}
```
Note: no new line between struct and impl, each app usage on its own line and finishing semicolon also on its own line.

### Coupling code style
If given component represents logical whole and owns specific systems, put its system within its impl block, f.e
```rust
#[derive(Component)]
pub struct MyComponent;
impl MyComponent {
    fn system() {
        // System code, keep it private be default as it should only be used in local plugin
    }
}
```