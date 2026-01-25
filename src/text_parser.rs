//! Text parsing with support for multiple file formats.
//!
//! Provides the `TextParser` trait for extensible format support and a `TxtParser`
//! implementation for plain text files.

use std::path::Path;

use crate::state::Word;

/// Trait for parsing text content into words. Implement for new file format support.
pub trait TextParser {
    /// Returns true if this parser can handle the given file path (based on extension).
    #[allow(dead_code)]
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
        let mut words = Vec::new();
        let mut is_paragraph_end = false;
        
        for line in content.lines() {
            let trimmed = line.trim();
            
            if trimmed.is_empty() {
                is_paragraph_end = true;
                continue;
            }
            
            for word_text in trimmed.split_whitespace() {
                words.push(Word {
                    text: word_text.to_string(),
                    is_paragraph_end,
                });
                is_paragraph_end = false;
            }
        }
        
        words
    }
}

/// Returns an appropriate parser for the given file path, or None if unsupported.
#[allow(dead_code)]
pub fn get_parser_for_path(path: &Path) -> Option<Box<dyn TextParser>> {
    if TxtParser::can_parse(path) {
        return Some(Box::new(TxtParser));
    }
    None
}
