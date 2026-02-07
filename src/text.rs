//! Text parsing with support for multiple file formats.
//!
//! Provides the `TextParser` trait for extensible format support and a `TxtParser`
//! implementation for plain text files.

use std::path::Path;

use serde::{Deserialize, Serialize};


#[derive(Clone, Serialize, Deserialize)]
pub struct Word {
    pub text: String,
    pub is_paragraph_end: bool,
}

impl Word {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into(), is_paragraph_end: false }
    }

    /// Returns the character index the eye should fixate on (slightly left-of-center).
    /// Based on RSVP research: longer words need the fixation point further in.
    pub fn orp_index(&self) -> usize {
        match self.text.chars().count() {
            0 => 0,
            1 => 0,
            2..=5 => 1,
            6..=9 => 2,
            10..=13 => 3,
            _ => 4,
        }
    }
    
    /// Uses max-wins strategy for multipliers (not cumulative), so a sentence-ending
    /// long word gets the sentence-end pause, not sentence-end × long-word.
    pub fn display_duration_ms(&self, wpm: u32) -> u64 {
        let base_ms = 60_000.0 / wpm as f64;
        let mut multiplier = 1.0f64;
        
        if self.text.chars().count() > 10 {
            multiplier = multiplier.max(1.3);
        }
        if self.text.ends_with(',') || self.text.ends_with(';') {
            multiplier = multiplier.max(2.0);
        }
        if self.text.ends_with('.') || self.text.ends_with('?') || self.text.ends_with('!') {
            multiplier = multiplier.max(3.0);
        }
        if self.is_paragraph_end {
            multiplier = multiplier.max(4.0);
        }
        
        (base_ms * multiplier) as u64
    }
}

/// Trait for parsing text content into words. Implement for new file format support.
pub trait TextParser {
    /// Returns true if this parser can handle the given file path (based on extension).
    fn can_parse(path: &Path) -> bool where Self: Sized;
    
    /// Parses the content string into a vector of Words with paragraph detection.
    fn parse(&self, content: &str) -> Vec<Word>;
}

pub struct TxtParser;
impl TextParser for TxtParser {
    fn can_parse(path: &Path) -> bool {
        path.extension()
            .map(|ext| ext.eq_ignore_ascii_case("txt"))
            .unwrap_or(false)
    }
    
    fn parse(&self, content: &str) -> Vec<Word> {
        let mut words: Vec<Word> = Vec::new();
        
        for line in content.lines() {
            let trimmed_line = line.trim();
            
            // Blank line = paragraph break. Mark the *last* word before the gap
            // so the reading pause happens at the end of the paragraph, not the start of the next.
            if trimmed_line.is_empty() {
                if let Some(last) = words.last_mut() {
                    last.is_paragraph_end = true;
                }
                continue;
            }
            
            words.extend(trimmed_line.split_whitespace().map(Word::new));
        }
        
        words
    }
}

/// Returns an appropriate parser for the given file path, or None if unsupported.
pub fn get_parser_for_path(path: &Path) -> Option<Box<dyn TextParser>> {
    if TxtParser::can_parse(path) {
        return Some(Box::new(TxtParser));
    }
    None
}
