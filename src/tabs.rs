//! Tab entity management with ECS-aligned event handling.
//!
//! Provides tab components, bundles, entity events, and observers for reactive tab management.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::fonts::{FontData, FontsStore};
use crate::persistence::ProgramState;
use crate::reader::{FONT_SIZE_DEFAULT, WPM_DEFAULT, WordChanged};
use crate::text::Word;

pub struct TabsPlugin;
impl Plugin for TabsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TabOrder>()
            .init_resource::<DefaultTabSettings>()
            .add_systems(Startup, HomepageTab::spawn)
            .add_observer(TabSelect::on_trigger)
            .add_observer(TabClose::on_trigger)
            .add_observer(TabCreateRequest::on_trigger)
            .add_observer(ApplyDefaultsToAll::on_trigger)
            .add_observer(TabOrder::on_tab_added)
            .add_observer(TabOrder::on_tab_removed)
            ;
    }
}

// ============================================================================
// Resources
// ============================================================================

/// Defaults applied to newly created tabs and used by "Apply to all tabs".
/// Serialized to disk as part of `ProgramState`. Stores `font_name` as a string
/// (not `FontData`) because font handles are runtime-only.
#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct DefaultTabSettings {
    pub font_name: String,
    pub font_size: f32,
    pub wpm: u32,
}
impl Default for DefaultTabSettings {
    fn default() -> Self {
        Self {
            font_name: String::new(),
            font_size: FONT_SIZE_DEFAULT,
            wpm: WPM_DEFAULT,
        }
    }
}

/// Maintains ordered list of tab entities for consistent UI display.
/// Automatically updated via lifecycle observers on `TabMarker`.
#[derive(Resource, Default)]
pub struct TabOrder(Vec<Entity>);
impl TabOrder {
    pub fn entities(&self) -> &[Entity] {
        &self.0
    }
    /// Returns the adjacent tab to `target`, preferring next then previous.
    /// Excludes `target` itself from the result (safe for close-then-select).
    pub fn find_adjacent(&self, target: Entity) -> Option<Entity> {
        let idx = self.0.iter().position(|&e| e == target)?;
        self.0.get(idx + 1)
            .or_else(|| idx.checked_sub(1).and_then(|i| self.0.get(i)))
            .filter(|&&e| e != target)
            .copied()
    }
    fn on_tab_added(trigger: On<Add, TabMarker>, mut order: ResMut<TabOrder>) {
        order.0.push(trigger.event_target());
    }
    fn on_tab_removed(trigger: On<Remove, TabMarker>, mut order: ResMut<TabOrder>) {
        order.0.retain(|&e| e != trigger.event_target());
    }
}

// ============================================================================
// Components
// ============================================================================

/// Marker for the single currently-active tab.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct ActiveTab;

/// Present on every tab entity. `TabOrder` tracks entities via Add/Remove
/// observers on this component.
#[derive(Component)]
pub struct TabMarker;

#[derive(Component)]
pub struct HomepageTab;
impl HomepageTab {
    fn spawn(mut commands: Commands) {
        commands.spawn((
            TabMarker,
            HomepageTab,
            Name::new("🏠"),
            ActiveTab,
        ));
    }
}

#[derive(Component)]
pub struct ReaderTab;

/// Per-tab font configuration. Inserting this component on the active tab
/// triggers the ORP font update observer in `orp.rs`.
#[derive(Component)]
pub struct TabFontSettings {
    pub font: FontData,
    pub font_size: f32,
}
impl TabFontSettings {
    pub fn from_font(font: &FontData, size: f32) -> Self {
        Self {
            font: font.clone(),
            font_size: size,
        }
    }
}

#[derive(Component)]
pub struct TabWpm(pub u32);

#[derive(Component)]
pub struct TabFilePath(pub String);

#[derive(Component, Clone)]
pub struct Content {
    pub content_cache_id: String,
    pub words: Vec<Word>,
    pub current_index: usize,
}
impl Content {
    /// Creates new content and writes the word cache to disk immediately.
    pub fn new(words: Vec<Word>) -> Self {
        let content_cache_id = ProgramState::generate_cache_id();
        ProgramState::write_word_cache(&content_cache_id, &words);
        Self { content_cache_id, words, current_index: 0 }
    }
    /// Restores content from an existing cache (skips cache write).
    pub fn new_from_loaded(content_cache_id: String, words: Vec<Word>, current_index: usize) -> Self {
        Self { content_cache_id, words, current_index }
    }
    pub fn has_words(&self) -> bool {
        !self.words.is_empty()
    }
    pub fn current_word(&self) -> Option<&Word> {
        self.words.get(self.current_index)
    }
    /// Returns (current 1-indexed, total) for UI display.
    pub fn progress(&self) -> (usize, usize) {
        (self.current_index + 1, self.words.len())
    }
    pub fn skip_forward(&mut self, amount: usize) {
        self.current_index = (self.current_index + amount)
            .min(self.words.len().saturating_sub(1));
    }
    pub fn skip_backward(&mut self, amount: usize) {
        self.current_index = self.current_index.saturating_sub(amount);
    }
    pub fn restart(&mut self) {
        self.current_index = 0;
    }
    pub fn is_at_end(&self) -> bool {
        self.current_index + 1 >= self.words.len()
    }
    /// Advances to next word. Returns true if advanced, false if at end.
    pub fn advance(&mut self) -> bool {
        if !self.is_at_end() {
            self.current_index += 1;
            true
        } else {
            false
        }
    }
}

// ============================================================================
// Entity Events
// ============================================================================

/// Select an existing tab by entity.
#[derive(EntityEvent)]
pub struct TabSelect {
    pub entity: Entity,
}
impl TabSelect {
    /// Moves `ActiveTab` to the target entity and fires `WordChanged`
    /// so the ORP display and reading timer sync to the new tab's state.
    fn on_trigger(
        trigger: On<TabSelect>,
        mut commands: Commands,
        active_tab: Option<Single<Entity, With<ActiveTab>>>,
    ) {
        let target = trigger.entity;
        
        if let Some(current_active) = active_tab {
            commands.entity(current_active.into_inner()).remove::<ActiveTab>();
        }
        
        commands.entity(target).insert(ActiveTab);
        commands.trigger(WordChanged);
    }
}
impl From<Entity> for TabSelect {
    fn from(entity: Entity) -> Self {
        Self { entity }
    }
}

/// Close (despawn) a tab by entity.
#[derive(EntityEvent)]
pub struct TabClose {
    pub entity: Entity,
}
impl TabClose {
    /// Despawns the tab, cleans up its word cache, and auto-selects
    /// an adjacent tab if the closed tab was active.
    fn on_trigger(
        trigger: On<TabClose>,
        mut commands: Commands,
        tab_order: Res<TabOrder>,
        tabs: Query<(Has<ActiveTab>, &Content), (With<TabMarker>, With<ReaderTab>)>,
    ) {
        let target = trigger.entity;
        let Ok((was_active, content)) = tabs.get(target) else { return; };

        ProgramState::delete_word_cache(&content.content_cache_id);
        commands.entity(target).despawn();
        
        if was_active {
            if let Some(entity) = tab_order.find_adjacent(target) {
                commands.trigger(TabSelect { entity });
            }
        }
    }
}
impl From<Entity> for TabClose {
    fn from(entity: Entity) -> Self {
        Self { entity }
    }
}

/// Builder-pattern event for creating reader tabs. Optional fields fall back
/// to `DefaultTabSettings`. The observer spawns the entity and optionally
/// triggers `TabSelect` to make it active.
#[derive(Event)]
pub struct TabCreateRequest {
    pub name: String,
    pub content: Content,
    pub file_path: Option<String>,
    pub font_name: Option<String>,
    pub font_size: Option<f32>,
    pub wpm: Option<u32>,
    pub is_active: bool,
}
impl TabCreateRequest {
    pub fn new(name: String, content: Content) -> Self {
        Self {
            name,
            content,
            file_path: None,
            font_name: None,
            font_size: None,
            wpm: None,
            is_active: true,
        }
    }
    pub fn with_file_path(mut self, name: impl Into<String>) -> Self {
        self.file_path = Some(name.into());
        self
    }
    pub fn with_font(mut self, name: String, size: f32) -> Self {
        self.font_name = Some(name);
        self.font_size = Some(size);
        self
    }
    pub fn with_wpm(mut self, wpm: u32) -> Self {
        self.wpm = Some(wpm);
        self
    }
    pub fn with_active(mut self, active: bool) -> Self {
        self.is_active = active;
        self
    }
    fn on_trigger(
        trigger: On<TabCreateRequest>,
        mut commands: Commands,
        fonts: Res<FontsStore>,
        defaults: Res<DefaultTabSettings>,
    ) {
        let font = fonts.resolve(trigger.font_name.as_deref().unwrap_or(&defaults.font_name));
        let font_size = trigger.font_size.unwrap_or(defaults.font_size);
        let wpm = trigger.wpm.unwrap_or(defaults.wpm);
        
        let mut entity_commands = commands.spawn((
            TabMarker,
            ReaderTab,
            Name::new(trigger.name.clone()),
            TabFontSettings::from_font(font, font_size),
            TabWpm(wpm),
            trigger.content.clone(),
        ));
        
        if let Some(path) = &trigger.file_path {
            entity_commands.insert(TabFilePath(path.clone()));
        }
        
        if trigger.is_active {
            let entity = entity_commands.id();
            commands.trigger(TabSelect { entity });
        }
    }
}

/// Overwrites font and WPM on every reader tab with current `DefaultTabSettings`.
#[derive(Event)]
pub struct ApplyDefaultsToAll;
impl ApplyDefaultsToAll {
    fn on_trigger(
        _trigger: On<ApplyDefaultsToAll>,
        mut commands: Commands,
        defaults: Res<DefaultTabSettings>,
        fonts: Res<FontsStore>,
        reader_tabs: Query<Entity, With<ReaderTab>>,
    ) {
        let font = fonts.resolve(&defaults.font_name);

        for entity in reader_tabs.iter() {
            commands.entity(entity).insert((
                TabFontSettings::from_font(font, defaults.font_size),
                TabWpm(defaults.wpm),
            ));
        }
    }
}

