//! Tab entity management with ECS-aligned event handling.
//!
//! Provides tab components, bundles, entity events, and observers for reactive tab management.

use bevy::prelude::*;
use std::path::PathBuf;

use crate::fonts::FontsStore;
use crate::reader::{FONT_SIZE_DEFAULT, WPM_DEFAULT, WordChanged};
use crate::text::Word;

pub struct TabsPlugin;
impl Plugin for TabsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TabOrder>()
            .add_observer(TabSelect::on_trigger)
            .add_observer(TabClose::on_trigger)
            .add_observer(TabCreateRequest::on_trigger)
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
pub struct TabFontSettings {
    pub font_name: String,
    pub font_handle: Handle<Font>,
    pub font_size: f32,
}

#[derive(Component)]
pub struct TabWpm(pub u32);

#[derive(Component)]
pub struct TabFilePath(pub PathBuf);

#[derive(Component)]
pub struct WordsManager {
    pub words: Vec<Word>,
    pub current_index: usize,
}
impl WordsManager {
    pub fn has_words(&self) -> bool {
        !self.words.is_empty()
    }
    pub fn current_word(&self) -> Option<&Word> {
        self.words.get(self.current_index)
    }
    /// Returns (current_1indexed, total) for UI display.
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
    /// Advances to next word. Returns true if advanced, false if at end.
    pub fn advance(&mut self) -> bool {
        if self.current_index + 1 < self.words.len() {
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
    fn on_trigger(
        trigger: On<TabClose>,
        mut commands: Commands,
        tab_order: Res<TabOrder>,
        tabs: Query<Has<ActiveTab>, With<TabMarker>>,
    ) {
        let target = trigger.entity;
        let was_active = tabs.get(target).is_ok_and(|is_active| is_active);
        
        commands.entity(target).despawn();
        
        // If closed tab was active, select adjacent tab from ordered list
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


#[derive(Event)]
pub struct TabCreateRequest {
    pub name: String,
    pub words: Vec<Word>,
    pub file_path: Option<PathBuf>,
    pub font_name: Option<String>,
    pub font_size: f32,
    pub wpm: u32,
    pub current_index: usize,
    pub is_active: bool,
}
impl TabCreateRequest {
    pub fn new(name: String, words: Vec<Word>) -> Self {
        Self {
            name,
            words,
            file_path: None,
            font_name: None,
            font_size: FONT_SIZE_DEFAULT,
            wpm: WPM_DEFAULT,
            current_index: 0,
            is_active: true,
        }
    }
    pub fn with_file_path(mut self, path: PathBuf) -> Self {
        self.file_path = Some(path);
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
    pub fn with_current_index(mut self, index: usize) -> Self {
        self.current_index = index;
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
            Name::new(trigger.name.clone()),
            TabFontSettings {
                font_name,
                font_handle,
                font_size: trigger.font_size,
            },
            TabWpm(trigger.wpm),
            WordsManager {
                words: trigger.words.clone(),
                current_index: trigger.current_index,
            },
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
