# Notes for AI Agents

## Project Overview
Arre Mind Reader is a speed-reading application built with Bevy 0.18 and Rust. It implements RSVP (Rapid Serial Visual Presentation) reading method.

## Architecture
- **Core Engine:** Bevy 0.18 (Rust)
- **UI Overlay:** bevy_egui 0.39 for egui-based panels
- **Rendering:** `Text2d` in world space for the reader display
- **Data Flow:** `Tab` owns word data → `ReaderState` tracks current index → ORP display reads both

## Key Concepts
- **ORP (Optical Recognition Point):** The letter the eye fixates on, positioned at screen center (0,0). Research shows slightly left-of-center is optimal.
- **Split-text rendering:** Words split into left/center/right using `Anchor::CenterRight` and `Anchor::CenterLeft` to grow outward from center.
- **Tab-owned state:** `Tab` is the single source of truth for words and reading position. `ReaderState` is a lightweight runtime cache for the current index.
- **Stable Tab IDs:** Tabs use `TabId` (u64) instead of indices for reliable identification across operations.

## Module Structure
- `main.rs` - App entry point, plugin registration
- `state.rs` - Resources (`ReaderState`, `ReaderSettings`, `TabManager`, `Tab`, `Word`), States (`ReadingState`), and `constants` module
- `reader.rs` - Plugin orchestration, timing tick, reading state transitions
- `orp.rs` - ORP calculation, display entity setup, word display updates
- `timing.rs` - `ReadingTimer` resource and `calc_delay()` for smart timing
- `input.rs` - Keyboard input handling (play/pause, navigation, WPM)
- `text_parser.rs` - `TextParser` trait and `TxtParser` implementation
- `fonts.rs` - `FontCache` for caching font handles, `AvailableFonts` scanning
- `settings.rs` - Persistence (RON format), debounced tab saving
- `ui.rs` - Egui UI: tab bar, controls, new tab dialog with async file loading

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
