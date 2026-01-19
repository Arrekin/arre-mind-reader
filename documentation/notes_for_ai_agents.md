# Notes for AI Agents

## Project Overview
Arre Mind Reader is a speed-reading application built with Bevy 0.18 and Rust. It implements RSVP (Rapid Serial Visual Presentation) reading method.

## Architecture
- **Core Engine:** Bevy 0.18 (Rust)
- **UI Overlay:** bevy_egui 0.39 for settings, Bevy UI for tab sidebar
- **Rendering:** `Text2d` in world space for the reader, `Node` for sidebars
- **Data Flow:** Timing system → `ReaderState` resource → Visual sync systems

## Key Concepts
- **ORP (Optical Recognition Point):** The letter the eye fixates on, positioned at screen center (0,0). Research shows slightly left-of-center is optimal.
- **Split-text rendering:** Words split into left/center/right using `Anchor::CenterRight` and `Anchor::CenterLeft` to grow outward from center.
- **Resource-based state:** All state in Bevy Resources. Timing logic writes to `ReaderState`; visuals read from it.
- **Focus Mode:** UI fades out during reading, fades back on mouse movement.

## Module Structure
- `main.rs` - App entry point, plugin registration
- `state.rs` - Resources (`ReaderState`, `ReaderSettings`, `OpenBooks`, `Word`) and States (`ReadingState`)
- `reader.rs` - Core reading logic, timing, ORP calculation
- `settings.rs` - Persistence (ron), book management
- `ui.rs` - UI components (sidebar, tabs, focus mode, playback controls)

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
