//! Tab entity management with ECS-aligned event handling.
//!
//! Provides tab components, bundles, entity events, and observers for reactive tab management.

use bevy::prelude::*;
use std::path::PathBuf;

use crate::fonts::FontsStore;
use crate::reader::{FONT_SIZE_DEFAULT, WPM_DEFAULT};
use crate::text::Word;

pub struct TabsPlugin;
impl Plugin for TabsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(TabSelect::on_trigger)
            .add_observer(TabClose::on_trigger)
            .add_observer(TabCreate::on_trigger)
            ;
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
        
        // Remove ActiveTab from current active tab
        if let Some(current_active) = active_tab {
            commands.entity(current_active.into_inner()).remove::<ActiveTab>();
        }
        
        // Add ActiveTab to selected
        commands.entity(target).insert(ActiveTab);
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
        tabs: Query<(Entity, Has<ActiveTab>), With<TabMarker>>,
    ) {
        let target = trigger.entity;
        let was_active = tabs.iter().any(|(e, is_active)| e == target && is_active);
        
        commands.entity(target).despawn();
        
        // If closed tab was active, select another
        if was_active {
            if let Some((other_entity, _)) = tabs.iter().find(|(e, _)| *e != target) {
                commands.trigger(TabSelect { entity: other_entity });
            }
        }
    }
}
impl From<Entity> for TabClose {
    fn from(entity: Entity) -> Self {
        Self { entity }
    }
}

/// Create a new tab with content. Not an EntityEvent since no entity exists yet.
#[derive(Event)]
pub struct TabCreate {
    pub name: String,
    pub file_path: Option<PathBuf>,
    pub words: Vec<Word>,
}
impl TabCreate {
    fn on_trigger(
        trigger: On<TabCreate>,
        mut commands: Commands,
        fonts: Res<FontsStore>,
        active_tab: Option<Single<Entity, With<ActiveTab>>>,
    ) {
        // Deactivate current active tab
        if let Some(current_active) = active_tab {
            commands.entity(current_active.into_inner()).remove::<ActiveTab>();
        }
        
        // Spawn new tab
        let default_font = fonts.default_font();
        let font_name = default_font.map(|f| f.name.clone()).unwrap_or_default();
        let font_handle = default_font.map(|f| f.handle.clone()).unwrap_or_default();
        
        let mut entity_commands = commands.spawn((
            TabMarker,
            ActiveTab,
            Name::new(trigger.name.clone()),
            TabFontSettings {
                font_name,
                font_handle,
                font_size: FONT_SIZE_DEFAULT,
            },
            TabWpm(WPM_DEFAULT),
            WordsManager {
                words: trigger.words.clone(),
                current_index: 0,
            },
        ));
        
        if let Some(path) = &trigger.file_path {
            entity_commands.insert(TabFilePath(path.clone()));
        }
    }
}
