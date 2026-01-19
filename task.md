# Project Architecture: "Arre Mind Reader"

- **Core Engine:** Bevy 0.18 (Rust)
- **UI Overlay:** bevy_egui (for settings panel) + Bevy UI (for tab sidebar)
- **Rendering Strategy:** `Text2d` (World Space) for the Speed Reader; `Node` (UI Space) for the Sidebars.
- **Data Model:** Resource-based state management. Timing logic writes to `ReaderState` resource; visual systems read from it. This decouples data from presentation, enabling future multi-view modes.

---

## Epic 1: The Engine Room (Core Rendering)

**Goal:** Get words flashing on screen with perfect timing and alignment.

### Ticket 1.1: The "Perfect Center" Rendering System

**Description:** Implement the split-text rendering logic to ensure the Optical Recognition Point (ORP) is mathematically fixed at screen center.

**Tech Spec:**
- Spawn a root entity at `(0, 0, 0)` with child entities for reticles and text segments.
- Use 3 `Text2d` children: `LeftText`, `CenterText` (highlighted), `RightText`.
- **Crucial:** Use `Anchor::CenterRight` for LeftText and `Anchor::CenterLeft` for RightText. This ensures they "grow outwards" from the center letter without jittering.

**Acceptance Criteria:**
- The highlighted letter never moves by a single pixel when words change.
- Kerning between Left/Center/Right looks natural (might need manual offset tweaking).

### Ticket 1.2: The "Smart Ticker" Algorithm

**Description:** Implement the timing loop that respects punctuation.

**Tech Spec:**
- Implement a `calc_delay(word: &Word, wpm: u32) -> Duration` function.
- Logic (use **max** of applicable multipliers, not cumulative):
  - Base delay = `60000 / WPM` ms.
  - Comma/Semicolon: Base * 2.0.
  - Period/Question/Exclamation: Base * 3.0.
  - Paragraph End (`word.is_paragraph_end`): Base * 4.0.
  - Word > 10 chars: Base * 1.3.

```rust
pub fn calc_delay(word: &Word, wpm: u32) -> Duration {
    let base_ms = 60_000.0 / wpm as f64;
    let mut multiplier = 1.0f64;
    
    let text = &word.text;
    if text.len() > 10 { multiplier = multiplier.max(1.3); }
    if text.ends_with([',', ';']) { multiplier = multiplier.max(2.0); }
    if text.ends_with(['.', '?', '!']) { multiplier = multiplier.max(3.0); }
    if word.is_paragraph_end { multiplier = multiplier.max(4.0); }
    
    Duration::from_millis((base_ms * multiplier) as u64)
}
```

**Acceptance Criteria:**
- Reading feels "rhythmic" and not robotic.

### Ticket 1.3: Input Parsing Service

**Description:** A background thread (using `bevy::tasks`) that takes a file path and returns a `Vec<Word>`.

**Tech Spec:**
- Support `.txt` initially.
- Sanitize inputs: Replace single newlines with spaces.
- Detect double-newlines as paragraph breaks.
- Use a `Word` struct with metadata:
  ```rust
  pub struct Word {
      pub text: String,
      pub is_paragraph_end: bool,
  }
  ```
- The `is_paragraph_end` flag triggers the extended pause in the ticker.

---

## Epic 2: The Workspace (UI & Tabs)

**Goal:** A modern, non-distracting management interface.

### Ticket 2.1: The Vertical Tab System

**Description:** A left-hand sidebar for managing open reading sessions.

**Tech Spec:**
- Use Bevy UI (`Node` component).
- Structure:
  - Sidebar: ~200px width, dark background.
  - TabList: `FlexDirection::Column`, scrollable if many tabs.
  - Tab: `Button` showing filename. Active tab highlighted.
  - **"+" Button:** At bottom of tab list, opens file picker to add new reading session.
- Behavior:
  - Clicking a tab switches `CurrentBook` resource.
  - Each tab preserves its own `current_index` (reading position).
  - Close button (X) on each tab to remove it.

### Ticket 2.2: File Loading

**Description:** Allow users to open text files for reading.

**Tech Spec:**
- "+" button in sidebar opens native file dialog (via `rfd` crate).
- On file select: parse file (Ticket 1.3), create new `BookState`, add to `OpenBooks`, switch to it.
- Supported: `.txt` initially.
- Show loading indicator while parsing large files.

### Ticket 2.3: The Reading Space

**Description:** The main content area where reading happens.

**Tech Spec:**
- **No book selected:** Show a welcome/configurator view:
  - "Open a file to start reading" message.
  - Quick settings: WPM slider, font size preview.
  - Drag-drop zone for files.
- **Book selected, idle:** Show current word statically with playback controls visible.
- **Book selected, playing:** Full-screen reading mode (Focus Mode).

### Ticket 2.4: Focus Mode

**Description:** Distraction-free reading with auto-hide UI.

**Tech Spec:**
- Triggered when `ReadingState::Active`.
- **Fade out:** Sidebar and controls fade to 0% opacity over 300ms.
- **Mouse movement:** Any mouse move fades UI back in, starts idle timer.
- **Idle timer:** After 2 seconds of no mouse movement, fade out again (if still playing).
- **Pause/Stop:** Immediately shows UI.

**Acceptance Criteria:**
- No UI clutter visible while reading.
- Moving mouse reveals controls without stopping playback.

### Ticket 2.5: Playback Controls

**Description:** Controls for reading playback.

**Tech Spec:**
- Visual controls (shown when UI visible):
  - Play/Pause button
  - Progress bar (clickable to seek)
  - Current word / total words counter
- **Keyboard shortcuts:**
  - `Space`: Play/Pause toggle
  - `Escape`: Stop (return to Idle, show UI)
  - `Left Arrow`: Go back 5 words
  - `Right Arrow`: Skip forward 5 words
  - `Up Arrow`: Increase WPM by 50
  - `Down Arrow`: Decrease WPM by 50
  - `R`: Restart from beginning

**Acceptance Criteria:**
- Controls respond even during Focus Mode (keyboard always active).

---

## Epic 3: Settings & Customization

**Goal:** Allow the user to tweak the experience without touching code.

### Ticket 3.1: The Egui Settings Panel

**Description:** A floating window (toggleable with ESC) containing sliders.

**Fields:**
- WPM: Slider (100 - 1000), default 300.
- Font Size: Slider (20px - 120px), default 48px.
- ORP Highlight Color: Color Picker (Default: Red).

### Ticket 3.2: Persistence

**Description:** Save open tabs and current reading position to disk.

**Tech Spec:**
- Use `serde` + `ron` (Rusty Object Notation).
- On `App::exit`, write `session.ron`.
- On Startup, load `session.ron` if it exists.

---

## Detailed Spec: Efficient Rendering (Ticket 1.1 Deep Dive)

To achieve the "zero eye movement" goal, the rendering must follow the Spritz methodology closely.

### 1. The "Reticle" Visual

You need a visual anchor. Even though the red letter is the anchor, drawing two subtle vertical lines above and below the center point helps guide the eye before the word appears.

```
       |
   uniVerse
       |
```

### 2. The Mathematical Center (ORP Calculation)

Do not just pick the middle letter. Research shows the eye prefers a fixation point slightly to the left of center for optimal word recognition.

**Algorithm:**
- Word Length 1: Index 0
- Word Length 2-5: Index 1
- Word Length 6-9: Index 2
- Word Length 10-13: Index 3
- Word Length 14+: Index 4

```rust
pub fn calculate_orp_index(word: &str) -> usize {
    match word.chars().count() {
        1 => 0,
        2..=5 => 1,
        6..=9 => 2,
        10..=13 => 3,
        _ => 4,
    }
}
```

### 3. The Bevy 0.18 Entity Hierarchy

```rust
// Spawn once at startup
commands.spawn((
    Name::new("WordRoot"),
    Transform::from_xyz(0.0, 0.0, 0.0),
    Visibility::default(),
    children![
        // Top reticle - simple rectangle
        (
            Sprite::from_color(Color::srgba(1.0, 0.0, 0.0, 0.6), Vec2::new(2.0, 20.0)),
            Transform::from_xyz(0.0, 40.0, 0.0),
            ReticleMarker,
        ),
        // Bottom reticle
        (
            Sprite::from_color(Color::srgba(1.0, 0.0, 0.0, 0.6), Vec2::new(2.0, 20.0)),
            Transform::from_xyz(0.0, -40.0, 0.0),
            ReticleMarker,
        ),
        // Left text - grows leftward from center
        (
            Text2d::new(""),
            TextFont { font_size: 48.0, ..default() },
            TextColor(Color::WHITE),
            Anchor::CenterRight,
            LeftTextMarker,
        ),
        // Center text - the ORP letter, highlighted
        (
            Text2d::new(""),
            TextFont { font_size: 48.0, ..default() },
            TextColor(Color::srgb(1.0, 0.0, 0.0)),
            Anchor::Center,
            CenterTextMarker,
        ),
        // Right text - grows rightward from center
        (
            Text2d::new(""),
            TextFont { font_size: 48.0, ..default() },
            TextColor(Color::WHITE),
            Anchor::CenterLeft,
            RightTextMarker,
        ),
    ],
));
```

### 4. Update Logic (The "Hot Path")

The system that updates the text runs in `Update` schedule.

1. Timer fires, advance `ReaderState.current_index`.
2. Get next word from `ReaderState.words[current_index]`.
3. Calculate ORP index (e.g., for "Processing", ORP is 'c' at index 3).
4. Split string: `left="Pro"`, `center="c"`, `right="essing"`.
5. Query for `LeftTextMarker`, `CenterTextMarker`, `RightTextMarker` and mutate their `Text2d` values.

**Optimization:** Entities are spawned once at startup. Only the `Text2d` string content is mutated each tick - no spawn/despawn overhead.