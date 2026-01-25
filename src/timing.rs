//! Timing resources for reading.

use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct ReadingTimer {
    pub timer: Timer,
}
