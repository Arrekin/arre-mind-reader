//! Timing calculations for word display duration.
//!
//! Implements smart timing that adjusts based on word length and punctuation
//! to create a natural reading rhythm.

use bevy::prelude::*;
use std::time::Duration;

use crate::state::Word;

#[derive(Resource, Default)]
pub struct ReadingTimer {
    pub timer: Timer,
}

pub fn calc_delay(word: &Word, wpm: u32) -> Duration {
    let base_ms = 60_000.0 / wpm as f64;
    let mut multiplier = 1.0f64;
    
    let text = &word.text;
    if text.chars().count() > 10 {
        multiplier = multiplier.max(1.3);
    }
    if text.ends_with(',') || text.ends_with(';') {
        multiplier = multiplier.max(2.0);
    }
    if text.ends_with('.') || text.ends_with('?') || text.ends_with('!') {
        multiplier = multiplier.max(3.0);
    }
    if word.is_paragraph_end {
        multiplier = multiplier.max(4.0);
    }
    
    Duration::from_millis((base_ms * multiplier) as u64)
}
