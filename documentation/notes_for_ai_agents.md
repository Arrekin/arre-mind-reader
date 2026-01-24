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
- **ECS Tab Modeling:** Each tab is an entity with components: `TabMarker`, `TabName`, `TabFontSettings`, `TabWpm`, `WordsManager`, optional `TabFilePath`.
- **Per-tab settings:** Font and WPM are stored per-tab, not globally. Highlight color is hardcoded red.
- **Stable Tab IDs:** Tabs use `TabId` (u64) in `TabMarker` for reliable identification across save/load.

## Module Structure
- `main.rs` - App entry point, plugin registration
- `state.rs` - Tab entity components (`TabMarker`, `TabName`, `TabFontSettings`, `TabWpm`, `TabFilePath`, `WordsManager`), `ActiveTab` resource, `ReadingState`, `Word`, and `constants` module
- `reader.rs` - Plugin orchestration, timing tick, reading state transitions
- `orp.rs` - ORP calculation, display entity setup, word display updates (hardcoded red highlight)
- `timing.rs` - `ReadingTimer` resource and `calc_delay()` for smart timing
- `input.rs` - Keyboard input handling (play/pause, navigation, WPM)
- `text_parser.rs` - `TextParser` trait and `TxtParser` implementation
- `fonts.rs` - `FontsStore` resource with `Vec<FontData>`, scans assets/fonts at startup
- `settings.rs` - Persistence (RON format), periodic tab saving, spawns tab entities on load
- `ui.rs` - Egui UI: tab bar, controls, new tab dialog with async file loading, `spawn_tab()` helper

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

## ORP Algorithm
```rust
match word.chars().count() {
    1 => 0,
    2..=5 => 1,
    6..=9 => 2,
    10..=13 => 3,
    _ => 4,
}
```

## Delay Multipliers (use max, not cumulative)
- Base: `60000 / WPM` ms
- Long word (>10 chars): ×1.3
- Comma/semicolon: ×2.0
- Period/question/exclamation: ×3.0
- Paragraph end: ×4.0
