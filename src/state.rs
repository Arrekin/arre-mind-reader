use bevy::prelude::*;

use crate::reader::TabId;

// ============================================================================
// Constants
// ============================================================================

pub mod constants {
    pub const WPM_DEFAULT: u32 = 300;
    pub const WPM_MIN: u32 = 100;
    pub const WPM_MAX: u32 = 1000;
    pub const WPM_STEP: u32 = 50;
    pub const FONT_SIZE_DEFAULT: f32 = 48.0;
    pub const WORD_SKIP_AMOUNT: usize = 5;
    pub const CHAR_WIDTH_RATIO: f32 = 0.6;
}


// ============================================================================
// Resources
// ============================================================================

#[derive(Resource, Default)]
pub struct ActiveTab {
    pub entity: Option<Entity>,
    next_id: TabId,
}
impl ActiveTab {
    pub fn allocate_id(&mut self) -> TabId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    pub fn set_next_id(&mut self, id: TabId) {
        self.next_id = id;
    }
}
