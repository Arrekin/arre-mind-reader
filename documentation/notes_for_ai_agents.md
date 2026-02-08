# Notes for AI Agents

## Project Overview
Arre Mind Reader is a speed-reading application built with Bevy 0.18 and Rust. It implements RSVP (Rapid Serial Visual Presentation) — words are shown one at a time at a configurable rate, with a fixed eye fixation point (ORP).

See `work.md` in the project root for tracked bugs, refactors, and deferred items.

## Architecture
- **Core Engine:** Bevy 0.18 (Rust)
- **UI Overlay:** bevy_egui 0.39 for egui-based panels
- **Rendering:** `Text2d` in world space for the reader display
- **Data Flow:** Tab entities own all per-tab state via components → systems read the `ActiveTab`-marked entity

## Design Decisions
- **ORP (Optical Recognition Point):** The letter the eye fixates on, positioned at screen center (0,0). Research shows slightly left-of-center is optimal.
- **Monospace fonts only.** The ORP positioning uses a fixed `CHAR_WIDTH_RATIO` (0.6) to estimate character width. Proportional fonts will misalign. This is intentional — RSVP works best with monospace.
- **Per-tab settings.** Font and WPM are stored per-tab, not globally. ORP highlight color is hardcoded red.
- **WordChanged event.** A `WordChanged` trigger (in `reader.rs`) is fired whenever the current word changes — by tick advance, skip, restart, or tab switch. Observers reset `ReadingTimer` and update ORP text content. All code that changes the current word must trigger `WordChanged`.
- **TabFontChanged event.** An `EntityEvent` carrying font name, handle, and size. Fired by: (1) UI font selector, (2) `TabSelect` cascade on tab switch. Two observers react: one applies changes to `TabFontSettings` component, one updates ORP display entities (font + positions).
- **Centralized tab creation.** All tab creation goes through `TabCreateRequest` (with builder pattern). Both persistence restore and UI dialogs trigger this event — never spawn tab entities manually.
- **Encapsulation.** `Content` and `TabOrder` expose methods for their operations. Use the API (e.g. `advance()`, `current_word()`, `find_adjacent()`) instead of accessing their fields directly.
- **Paragraph detection.** Blank lines in source text mark the *last word before the gap* as `is_paragraph_end`, not the first word after. This ensures the reading pause happens at the end of the paragraph.
- **Display duration uses max-wins multiplier** (not cumulative). A sentence-ending long word gets the sentence-end pause (×3.0), not sentence-end × long-word.
- **Restart doesn't auto-play.** Pressing R resets `current_index` to 0 but doesn't change `ReadingState`. User must press Play separately. This is intentional to preserve user choice.

## Module Structure
Each file follows: imports → Plugin definition → constants → types/components → systems

- `main.rs` - App entry, plugin registration, camera spawn
- `reader.rs` - `ReaderPlugin` owns `ReadingState` (Idle/Playing/Paused), `ReadingTimer`, and `WordChanged` event+observer. Per-word timing: each word gets a one-shot timer based on its `display_duration_ms`
- `tabs.rs` - `TabsPlugin` with tab components, `TabOrder` resource, `Content` component (words + cache ID + cursor), entity events (`TabSelect`, `TabClose`), `TabCreateRequest` event, and lifecycle observers
- `playback.rs` - `PlaybackCommand` event enum with `PlaybackCommand::on_trigger` observer
- `orp.rs` - ORP display: three `Text2d` entities (left/center/right) split around the fixation letter. Fully reactive — `on_word_changed` observer updates text content, `on_font_changed` observer updates font and positions
- `input.rs` - Keyboard → `PlaybackCommand` mapping
- `text.rs` - `TextPlugin`, `FileParsers` resource (extension→parser registry), `TextParser` trait, `TxtParser`, `EpubParser`, `words_from_text()` shared utility, `Word`/`ParseResult`/`Section` structs
- `fonts.rs` - `FontsStore` resource, loads built-in fonts via `AssetServer` on all platforms; on native also discovers additional `.ttf`/`.otf` files in `assets/fonts/`
- `persistence.rs` - `PersistencePlugin`: periodic + app-exit save of tab metadata to `tabs.ron`. `ProgramState` struct owns all platform-specific persistence: `generate_cache_id()`, `write_word_cache()`, `load_word_cache()`, `delete_word_cache()` as `pub(crate)` methods. Word content cached separately per tab in `cache/` directory (native) or localStorage keys (WASM). Called from `tabs.rs` (`Content::new()`, `TabClose`). Orphan cache cleanup on startup.
- `ui/` - egui UI: `tab_bar.rs` (tab strip), `controls.rs` (playback/WPM/font), `dialogs.rs` (new tab dialog, async file load)

## ECS Event Patterns
- **Tab lifecycle:** `EntityEvent` structs (`TabSelect`, `TabClose`) with observers. `TabOrder` auto-updates via `Add`/`Remove` observers on `TabMarker`.
- **Playback:** `Event` trigger (`PlaybackCommand`) with observer
- **Word lifecycle:** `WordChanged` trigger (global `Event`) fired after any word navigation. Observer in `reader.rs` resets the reading timer. Observer in `orp.rs` updates display text.
- **Font lifecycle:** `TabFontChanged` `EntityEvent` targeting a tab entity. Observer in `tabs.rs` applies to `TabFontSettings`. Observer in `orp.rs` updates display font/positions. `TabSelect` cascades this on tab switch.
- **UI → state:** UI emits events/triggers, observers react. No direct component mutation in UI systems.
- **Bevy 0.18 naming:** `Message`/`MessageWriter`/`MessageReader` for buffered events (not `Event`/`EventWriter`/`EventReader`). `Event` + `commands.trigger()` for immediate observer-based dispatch.

## Targets
The app supports two build targets: **native** (Linux/macOS/Windows) and **WASM** (browser). See `documentation/wasm_build.md` for WASM build instructions.

- `#[cfg(target_arch = "wasm32")]` / `#[cfg(not(target_arch = "wasm32"))]` guards platform-specific code
- `Cargo.toml` uses `[target.'cfg(...)'.dependencies]` for platform-specific deps
- Default features include `native` (enables `bevy/dynamic_linking`); WASM builds use `--no-default-features`
- **All changes must compile for both targets.** Verify with `cargo check` (native) and `cargo check --target wasm32-unknown-unknown --no-default-features` (WASM)

## Dependencies
- `bevy` 0.18 - Game engine
- `bevy_egui` 0.39 - Egui integration
- `serde` + `ron` - Persistence
- `rfd` 0.15 - File dialogs (native + WASM with confirm() fallback)
- `rbook` 0.6 - EPUB parsing (WASM-compatible)
- `quick-xml` 0.37 - XHTML text extraction for EPUB content
- `dirs` - Platform config directory (native only)
- `gloo-storage` - localStorage wrapper (WASM only)
- `web-sys` - Web API bindings (WASM only)

## Bevy 0.18 Patterns
- `Camera2d` component (not bundle)
- `Text2d::new()` + `TextFont` + `TextColor` + `Anchor` for 2D text
- `Sprite::from_color()` for simple rectangles
- `children![]` macro for hierarchies
- `EguiPlugin::default()` (has struct fields now)
- `EventReader/Writer` are now `MessageReader/Writer` (new Events are now for triggers, Messages for old buffered Events)

## Keyboard Shortcuts
- `Space` - Play/Pause
- `Escape` - Stop
- `←/→` - Skip 5 words back/forward
- `↑/↓` - Adjust WPM ±50
- `R` - Restart (reset to beginning, does not auto-play)

## Code Style
- Query variables use plural form (e.g., `tabs`, `left_texts`), not `_q` suffix
- Encapsulate component internals behind methods. Use the API, don't reach into fields.
- Each module with a plugin defines it near the top after imports
- Don't put newlines between struct and its impl blocks
- **Use `pub`, not `pub(crate)`.** Single-crate project — `pub(crate)` adds noise with no benefit.
- **Comments must be timeless.** Never leave comments that reference the current conversation, refactoring session, or rationale like "we moved this here because X was duplicated." Comments should make sense to a reader who has no context of how the code evolved. If the code is self-explanatory, no comment is needed.

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
No newline between struct and impl. Each app builder call on its own line. Trailing semicolon on its own line.

### System code style
Define system elements in tihs specific order(whichever are needed)
```rust
fn my_bevy_system(
    trigger: Trigger<T>,
    mut commands: Commands,
    <resources>
    <queries>
    <locals>
)
```

### Coupling code style
If a component/event represents a logical whole and owns specific systems, keep its systems within its impl block:
```rust
#[derive(Component)]
pub struct MyComponent;
impl MyComponent {
    fn system() {
        // Keep private — only referenced by the local plugin
    }
}
```

## Agent Guidelines
- **Think before implementing.** When asked to fix a bug or add a feature, first consider whether the change reveals a deeper architectural issue. Prefer fixing the root cause over patching symptoms. If a small fix would fragment logic across multiple places, propose a broader refactor instead.
- **Avoid tunnel vision.** Don't just implement the literal request — evaluate whether it fits the existing patterns. If it doesn't, flag it and suggest an approach that does.
- **Check for event patterns.** Many cross-cutting concerns (timer reset, display update) are handled via events (`WordChanged`, `PlaybackCommand`, `TabSelect`). If you're adding logic that reacts to state changes, check if there's already an event you should hook into rather than duplicating logic.