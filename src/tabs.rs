//! Tab entity management with ECS-aligned event handling.
//!
//! Provides tab components, bundles, entity events, and observers for reactive tab management.

use bevy::prelude::*;

use crate::fonts::FontsStore;
use crate::persistence::ProgramState;
use crate::reader::{FONT_SIZE_DEFAULT, WPM_DEFAULT, WordChanged};
use crate::text::Word;

pub struct TabsPlugin;
impl Plugin for TabsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TabOrder>()
            .add_systems(Startup, spawn_homepage_tab)
            .add_observer(TabSelect::on_trigger)
            .add_observer(TabClose::on_trigger)
            .add_observer(TabCreateRequest::on_trigger)
            .add_observer(TabFontChanged::on_trigger)
            .add_observer(TabOrder::on_tab_added)
            .add_observer(TabOrder::on_tab_removed)
            ;
    }
}

// ============================================================================
// Resources
// ============================================================================

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

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct ActiveTab;

#[derive(Component)]
pub struct TabMarker;

#[derive(Component)]
pub struct HomepageTab;

#[derive(Component)]
pub struct ReaderTab;

#[derive(Component)]
pub struct TabFontSettings {
    pub font_name: String,
    pub font_handle: Handle<Font>,
    pub font_size: f32,
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
    pub fn new(words: Vec<Word>) -> Self {
        let content_cache_id = ProgramState::generate_cache_id();
        ProgramState::write_word_cache(&content_cache_id, &words);
        Self { content_cache_id, words, current_index: 0 }
    }
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
    fn on_trigger(
        trigger: On<TabSelect>,
        mut commands: Commands,
        active_tab: Option<Single<Entity, With<ActiveTab>>>,
        reader_tabs: Query<&TabFontSettings, With<ReaderTab>>,
    ) {
        let target = trigger.entity;
        
        if let Some(current_active) = active_tab {
            commands.entity(current_active.into_inner()).remove::<ActiveTab>();
        }
        
        commands.entity(target).insert(ActiveTab);
        
        if let Ok(font_settings) = reader_tabs.get(target) {
            commands.trigger(TabFontChanged {
                entity: target,
                name: font_settings.font_name.clone(),
                handle: font_settings.font_handle.clone(),
                size: font_settings.font_size,
            });
        }
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


#[derive(EntityEvent)]
pub struct TabFontChanged {
    pub entity: Entity,
    pub name: String,
    pub handle: Handle<Font>,
    pub size: f32,
}
impl TabFontChanged {
    fn on_trigger(
        trigger: On<TabFontChanged>,
        mut tabs: Query<&mut TabFontSettings>,
    ) {
        if let Ok(mut font_settings) = tabs.get_mut(trigger.entity) {
            font_settings.font_name = trigger.name.clone();
            font_settings.font_handle = trigger.handle.clone();
            font_settings.font_size = trigger.size;
        }
    }
}

#[derive(Event)]
pub struct TabCreateRequest {
    pub name: String,
    pub content: Content,
    pub file_path: Option<String>,
    pub font_name: Option<String>,
    pub font_size: f32,
    pub wpm: u32,
    pub is_active: bool,
}
impl TabCreateRequest {
    pub fn new(name: String, content: Content) -> Self {
        Self {
            name,
            content,
            file_path: None,
            font_name: None,
            font_size: FONT_SIZE_DEFAULT,
            wpm: WPM_DEFAULT,
            is_active: true,
        }
    }
    pub fn with_file_path(mut self, name: impl Into<String>) -> Self {
        self.file_path = Some(name.into());
        self
    }
    pub fn with_font(mut self, name: String, size: f32) -> Self {
        self.font_name = Some(name);
        self.font_size = size;
        self
    }
    pub fn with_wpm(mut self, wpm: u32) -> Self {
        self.wpm = wpm;
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
    ) {
        // Resolve font: try requested name → fall back to first available font
        let font_data = trigger.font_name.as_ref()
            .and_then(|name| fonts.get_by_name(name))
            .or_else(|| fonts.default_font());
        let font_name = font_data.map(|f| f.name.clone()).unwrap_or_default();
        let font_handle = font_data.map(|f| f.handle.clone()).unwrap_or_default();
        
        let mut entity_commands = commands.spawn((
            TabMarker,
            ReaderTab,
            Name::new(trigger.name.clone()),
            TabFontSettings {
                font_name,
                font_handle,
                font_size: trigger.font_size,
            },
            TabWpm(trigger.wpm),
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

// ============================================================================
// Startup Systems
// ============================================================================

fn spawn_homepage_tab(mut commands: Commands) {
    let entity = commands.spawn((
        TabMarker,
        HomepageTab,
        Name::new("Home"),
        ActiveTab,
    )).id();
    info!("Spawned homepage tab: {:?}", entity);
}
