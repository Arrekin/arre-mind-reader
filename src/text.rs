//! Text parsing with support for multiple file formats.
//!
//! `FileParsers` resource maps file extensions to `TextParser` implementations.

use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;

use bevy::prelude::*;
use quick_xml::events::Event;
use quick_xml::reader::Reader as XmlReader;
use rbook::Epub;
use rbook::ebook::Ebook;
use rbook::reader::{Reader as EbookReader, ReaderContent};
use serde::{Deserialize, Serialize};

pub struct TextPlugin;
impl Plugin for TextPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(FileParsers::new())
            ;
    }
}

/// Maps file extensions to their `TextParser` implementations.
#[derive(Resource)]
pub struct FileParsers {
    parsers: HashMap<String, Arc<dyn TextParser>>,
}
impl FileParsers {
    fn new() -> Self {
        let mut parsers: HashMap<String, Arc<dyn TextParser>> = HashMap::new();

        let txt = Arc::new(TxtParser) as Arc<dyn TextParser>;
        parsers.insert("txt".into(), txt);

        let epub = Arc::new(EpubParser) as Arc<dyn TextParser>;
        parsers.insert("epub".into(), epub);

        Self { parsers }
    }

    pub fn get_for_extension(&self, ext: &str) -> Option<&dyn TextParser> {
        self.parsers.get(&ext.to_ascii_lowercase()).map(|p| p.as_ref())
    }

    pub fn get_for_path(&self, path: &Path) -> Option<&dyn TextParser> {
        let ext = path.extension()?.to_str()?;
        self.get_for_extension(ext)
    }

    pub fn supported_extensions(&self) -> Vec<String> {
        self.parsers.keys().cloned().collect()
    }
}
/// Single display unit for the reader. Each word is shown for a duration
/// based on WPM and punctuation/length multipliers.
#[derive(Clone, Serialize, Deserialize)]
pub struct Word {
    pub text: String,
    /// When true, an extra pause is applied after this word (set on the
    /// last word before a blank line, not the first word after).
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

/// Chapter/section bookmark for future navigation UI.
#[allow(dead_code)]
pub struct Section {
    pub title: String,
    pub start_index: usize,
}

pub struct ParseResult {
    pub words: Vec<Word>,
    #[allow(dead_code)]
    pub sections: Vec<Section>,
}
impl ParseResult {
    pub fn words_only(words: Vec<Word>) -> Self {
        Self { words, sections: Vec::new() }
    }
}

/// Trait for parsing file content into words.
pub trait TextParser: Send + Sync {
    /// Parse raw file bytes into words with optional section metadata.
    fn parse(&self, data: &[u8]) -> Result<ParseResult, String>;
}

/// Splits plain text into words with paragraph detection.
/// Blank lines mark the last word before the gap as `is_paragraph_end`.
fn words_from_text(text: &str) -> Vec<Word> {
    let mut words: Vec<Word> = Vec::new();
    
    for line in text.lines() {
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

pub struct TxtParser;
impl TextParser for TxtParser {
    fn parse(&self, data: &[u8]) -> Result<ParseResult, String> {
        let content = String::from_utf8_lossy(data);
        Ok(ParseResult::words_only(words_from_text(&content)))
    }
}

pub struct EpubParser;
impl EpubParser {
    /// Extracts plain text from XHTML content.
    /// Block elements (`<p>`, `<div>`, `<br>`, headings) produce paragraph breaks.
    /// Inline elements are ignored; their text content is captured.
    fn extract_text_from_xhtml(xhtml: &str) -> String {
        let mut reader = XmlReader::from_str(xhtml);
        let mut text = String::new();
        let mut skip_depth: usize = 0;

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) => {
                    let tag = e.name();
                    let tag_bytes = tag.as_ref();
                    if skip_depth > 0 {
                        skip_depth += 1;
                        continue;
                    }
                    match tag_bytes {
                        b"style" | b"script" => { skip_depth = 1; }
                        b"p" | b"div" | b"br" | b"h1" | b"h2" | b"h3"
                        | b"h4" | b"h5" | b"h6" | b"li" | b"blockquote" | b"tr" => {
                            text.push_str("\n\n");
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    if skip_depth > 0 {
                        skip_depth -= 1;
                        continue;
                    }
                    let tag_name = e.name();
                    let tag_bytes = tag_name.as_ref();
                    match tag_bytes {
                        b"p" | b"div" | b"h1" | b"h2" | b"h3"
                        | b"h4" | b"h5" | b"h6" | b"li" | b"blockquote" | b"tr" => {
                            text.push_str("\n\n");
                        }
                        _ => {}
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    if skip_depth > 0 { continue; }
                    if e.name().as_ref() == b"br" {
                        text.push_str("\n\n");
                    }
                }
                Ok(Event::Text(e)) => {
                    if skip_depth > 0 { continue; }
                    if let Ok(decoded) = e.decode() {
                        text.push_str(&decoded);
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
        }

        text
    }
}
impl TextParser for EpubParser {
    fn parse(&self, data: &[u8]) -> Result<ParseResult, String> {
        let cursor = Cursor::new(data.to_vec());
        let epub = Epub::options()
            .strict(false)
            .read(cursor)
            .map_err(|e| format!("Failed to open EPUB: {}", e))?;

        let mut full_text = String::new();
        let mut reader = epub.reader();

        while let Some(result) = reader.read_next() {
            match result {
                Ok(content) => {
                    let chapter_text = Self::extract_text_from_xhtml(content.content());
                    if !chapter_text.trim().is_empty() {
                        full_text.push_str(&chapter_text);
                        full_text.push_str("\n\n");
                    }
                }
                Err(e) => {
                    bevy::log::warn!("Skipping malformed EPUB chapter: {}", e);
                }
            }
        }

        let words = words_from_text(&full_text);
        if words.is_empty() {
            return Err("No readable text found in EPUB".to_string());
        }

        Ok(ParseResult::words_only(words))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_duration_uses_max_wins_precedence() {
        let wpm = 600;

        assert_eq!(Word::new("abcdefghijk").display_duration_ms(wpm), 130);
        assert_eq!(Word::new("abcdefghijk,").display_duration_ms(wpm), 200);
        assert_eq!(Word::new("abcdefghijk.").display_duration_ms(wpm), 300);

        let mut paragraph_end_word = Word::new("abcdefghijk.");
        paragraph_end_word.is_paragraph_end = true;
        assert_eq!(paragraph_end_word.display_duration_ms(wpm), 400);
    }

    #[test]
    fn words_from_text_marks_last_word_before_blank_line() {
        let words = words_from_text("alpha beta\n\n gamma\n\n\n delta");

        let texts: Vec<&str> = words.iter().map(|word| word.text.as_str()).collect();
        let paragraph_end_flags: Vec<bool> = words.iter().map(|word| word.is_paragraph_end).collect();

        assert_eq!(texts, vec!["alpha", "beta", "gamma", "delta"]);
        assert_eq!(paragraph_end_flags, vec![false, true, true, false]);
    }

    #[test]
    fn words_from_text_handles_leading_and_trailing_blank_lines() {
        let words = words_from_text("\n\nalpha\n\n");

        assert_eq!(words.len(), 1);
        assert_eq!(words[0].text, "alpha");
        assert!(words[0].is_paragraph_end);
    }

    #[test]
    fn file_parsers_lookup_is_case_insensitive() {
        let parsers = FileParsers::new();

        assert!(parsers.get_for_extension("TXT").is_some());
        assert!(parsers.get_for_path(Path::new("book.EPUB")).is_some());
        assert!(parsers.get_for_extension("pdf").is_none());
    }
}
