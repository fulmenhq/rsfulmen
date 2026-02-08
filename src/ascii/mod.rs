//! Terminal utilities for Unicode-aware string handling and box drawing.
//!
//! This module provides helpers for working with Unicode strings in terminal
//! contexts where display width matters. It handles CJK characters, emoji,
//! combining marks, and other Unicode complexities that affect column alignment.
//!
//! # Key functions
//!
//! - [`string_width`] - Calculate terminal display width of a string.
//! - [`analyze`] - Inspect a string's Unicode properties.
//! - [`draw_box`] - Render text inside a Unicode box-drawing frame.
//! - [`truncate_to_width`] - Shorten a string to fit a column budget.
//! - [`pad_to_width`] - Right-pad a string with spaces to a target width.

use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Analysis of a string's properties for terminal display.
///
/// Captures multiple perspectives on string length (bytes, chars, graphemes,
/// display columns) plus Unicode classification flags. Useful for layout
/// engines, table formatters, and diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringAnalysis {
    /// Byte length of the string.
    pub byte_len: usize,
    /// Number of Unicode scalar values (chars).
    pub char_count: usize,
    /// Display width in terminal columns.
    pub display_width: usize,
    /// Number of grapheme clusters.
    pub grapheme_count: usize,
    /// Whether the string contains non-ASCII characters.
    pub has_unicode: bool,
    /// Whether the string contains emoji.
    pub has_emoji: bool,
    /// Whether the string is pure ASCII.
    pub is_ascii: bool,
}

/// Draw a box around lines of text using Unicode box-drawing characters.
///
/// Uses the light box-drawing set (`\u{250c}` `\u{2510}` `\u{2514}` `\u{2518}`
/// `\u{2500}` `\u{2502}`). Each content line receives one space of padding on
/// each side. Short lines are right-padded with spaces to match the longest
/// line's display width.
///
/// An empty slice still produces a valid (minimal) box frame.
///
/// # Examples
///
/// ```
/// # use rsfulmen::ascii::draw_box;
/// let output = draw_box(&["Hello", "World"]);
/// assert!(output.contains("Hello"));
/// ```
pub fn draw_box(lines: &[&str]) -> String {
    let max_width = lines.iter().map(|l| string_width(l)).max().unwrap_or(0);

    let border_width = max_width + 2; // 1 space padding each side

    let mut result = String::new();

    // Top border
    result.push('\u{250c}');
    for _ in 0..border_width {
        result.push('\u{2500}');
    }
    result.push('\u{2510}');
    result.push('\n');

    // Content lines
    for line in lines {
        let line_width = string_width(line);
        let padding = max_width - line_width;
        result.push('\u{2502}');
        result.push(' ');
        result.push_str(line);
        for _ in 0..padding {
            result.push(' ');
        }
        result.push(' ');
        result.push('\u{2502}');
        result.push('\n');
    }

    // Bottom border
    result.push('\u{2514}');
    for _ in 0..border_width {
        result.push('\u{2500}');
    }
    result.push('\u{2518}');
    result.push('\n');

    result
}

/// Calculate terminal display width of a string.
///
/// Uses grapheme-cluster-aware width calculation via
/// [`unicode_width::UnicodeWidthStr`]. This correctly handles ZWJ sequences
/// (e.g., family emoji), skin-tone modifiers, keycap sequences, and
/// combining marks.
///
/// # Examples
///
/// ```
/// # use rsfulmen::ascii::string_width;
/// assert_eq!(string_width("hello"), 5);
/// assert_eq!(string_width(""), 0);
/// ```
pub fn string_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

/// Analyze a string's Unicode properties for terminal display.
///
/// Performs a single pass over the string's characters to collect byte length,
/// char count, display width, emoji presence, and ASCII classification.
/// Grapheme count is derived via [`unicode_segmentation`].
///
/// # Examples
///
/// ```
/// # use rsfulmen::ascii::analyze;
/// let a = analyze("hello");
/// assert!(a.is_ascii);
/// assert_eq!(a.char_count, 5);
/// ```
pub fn analyze(s: &str) -> StringAnalysis {
    let byte_len = s.len();
    let is_ascii = s.is_ascii();
    let has_unicode = !is_ascii;
    let char_count = s.chars().count();
    let has_emoji = s.chars().any(is_emoji_char);
    let display_width = string_width(s);
    let grapheme_count = UnicodeSegmentation::graphemes(s, true).count();

    StringAnalysis {
        byte_len,
        char_count,
        display_width,
        grapheme_count,
        has_unicode,
        has_emoji,
        is_ascii,
    }
}

/// Truncate a string to fit within a given display width.
///
/// If the string already fits, it is returned unchanged. When truncation is
/// required, grapheme cluster boundaries are respected and `"..."` is appended.
///
/// If `max_width` is 3 or less, the result is `"..."` truncated to
/// `max_width` characters (i.e., `"."`, `".."`, or `"..."`).
///
/// # Examples
///
/// ```
/// # use rsfulmen::ascii::{truncate_to_width, string_width};
/// assert_eq!(truncate_to_width("hello", 10), "hello");
/// let t = truncate_to_width("hello world", 8);
/// assert!(string_width(&t) <= 8);
/// assert!(t.ends_with("..."));
/// ```
pub fn truncate_to_width(s: &str, max_width: usize) -> String {
    if string_width(s) <= max_width {
        return s.to_string();
    }

    if max_width == 0 {
        return String::new();
    }

    let ellipsis = "...";
    if max_width <= 3 {
        return ellipsis[..max_width].to_string();
    }

    let budget = max_width - 3; // reserve space for "..."
    let mut result = String::new();
    let mut current_width: usize = 0;

    for grapheme in UnicodeSegmentation::graphemes(s, true) {
        let gw = grapheme_width(grapheme);
        if current_width + gw > budget {
            break;
        }
        result.push_str(grapheme);
        current_width += gw;
    }

    result.push_str(ellipsis);
    result
}

/// Pad a string to a target display width with trailing spaces.
///
/// If the string's display width already meets or exceeds `target_width`, it
/// is returned unchanged (no truncation is performed).
///
/// # Examples
///
/// ```
/// # use rsfulmen::ascii::{pad_to_width, string_width};
/// let padded = pad_to_width("hi", 5);
/// assert_eq!(string_width(&padded), 5);
/// assert_eq!(pad_to_width("hello world", 5), "hello world");
/// ```
pub fn pad_to_width(s: &str, target_width: usize) -> String {
    let current = string_width(s);
    if current >= target_width {
        return s.to_string();
    }

    let mut result = String::with_capacity(s.len() + (target_width - current));
    result.push_str(s);
    for _ in 0..(target_width - current) {
        result.push(' ');
    }
    result
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Check whether a character falls within common emoji ranges.
///
/// This is intentionally a simplified heuristic covering the most frequently
/// encountered emoji blocks. It is not an exhaustive emoji classifier.
fn is_emoji_char(c: char) -> bool {
    matches!(c,
        '\u{1F600}'..='\u{1F64F}'   // Emoticons
        | '\u{1F300}'..='\u{1F5FF}' // Misc Symbols and Pictographs
        | '\u{1F680}'..='\u{1F6FF}' // Transport and Map Symbols
        | '\u{1F1E0}'..='\u{1F1FF}' // Regional Indicator Symbols (flags)
        | '\u{2600}'..='\u{27BF}'   // Misc Symbols / Dingbats
        | '\u{FE00}'..='\u{FE0F}'   // Variation Selectors
        | '\u{1F900}'..='\u{1FAFF}' // Supplemental Symbols and Pictographs
        | '\u{200D}'                 // Zero Width Joiner (used in emoji sequences)
    )
}

/// Calculate the display width of a grapheme cluster.
fn grapheme_width(grapheme: &str) -> usize {
    UnicodeWidthStr::width(grapheme)
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // string_width tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_width_ascii() {
        assert_eq!(string_width("hello"), 5);
        assert_eq!(string_width(""), 0);
        assert_eq!(string_width(" "), 1);
    }

    #[test]
    fn test_width_cjk() {
        assert_eq!(string_width("\u{65e5}\u{672c}\u{8a9e}"), 6); // 日本語
        assert_eq!(string_width("\u{4e2d}\u{6587}"), 4); // 中文
    }

    #[test]
    fn test_width_emoji() {
        assert_eq!(string_width("\u{1f44b}"), 2); // waving hand
        assert_eq!(string_width("hello \u{1f30d}"), 8); // hello + globe
    }

    #[test]
    fn test_width_mixed() {
        assert_eq!(string_width("A\u{65e5}B"), 4); // A + CJK(2) + B
    }

    #[test]
    fn test_width_combining_mark() {
        // e + combining acute accent = single grapheme, width 1
        assert_eq!(string_width("e\u{301}"), 1);
    }

    #[test]
    fn test_width_skin_tone_emoji() {
        // Thumbs up + skin tone modifier = single grapheme, width 2
        assert_eq!(string_width("\u{1F44D}\u{1F3FD}"), 2);
    }

    #[test]
    fn test_width_zwj_family_emoji() {
        // Family ZWJ sequence = single grapheme, width 2
        assert_eq!(
            string_width("\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}"),
            2
        );
    }

    #[test]
    fn test_width_keycap_emoji() {
        // Keycap "1" emoji = single grapheme, width 2
        assert_eq!(string_width("1\u{FE0F}\u{20E3}"), 2);
    }

    // -----------------------------------------------------------------------
    // analyze tests (complex Unicode)
    // -----------------------------------------------------------------------

    #[test]
    fn test_analyze_combining_mark() {
        let a = analyze("e\u{301}");
        assert_eq!(a.display_width, 1);
        assert_eq!(a.grapheme_count, 1);
        assert!(a.has_unicode);
        assert!(!a.has_emoji);
    }

    #[test]
    fn test_analyze_skin_tone_emoji() {
        let a = analyze("\u{1F44D}\u{1F3FD}");
        assert_eq!(a.display_width, 2);
        assert_eq!(a.grapheme_count, 1);
        assert!(a.has_unicode);
        assert!(a.has_emoji);
    }

    #[test]
    fn test_analyze_zwj_family_emoji() {
        let a = analyze("\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}");
        assert_eq!(a.display_width, 2);
        assert_eq!(a.grapheme_count, 1);
        assert!(a.has_unicode);
        assert!(a.has_emoji);
    }

    #[test]
    fn test_analyze_keycap_emoji() {
        let a = analyze("1\u{FE0F}\u{20E3}");
        assert_eq!(a.display_width, 2);
        assert_eq!(a.grapheme_count, 1);
        assert!(a.has_unicode);
        assert!(a.has_emoji);
    }

    // -----------------------------------------------------------------------
    // truncation regression tests (complex emoji)
    // -----------------------------------------------------------------------

    #[test]
    fn test_truncate_zwj_emoji_fits() {
        // ZWJ family emoji is width 2 — should fit in max_width 5 without truncation
        let emoji = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}";
        let result = truncate_to_width(emoji, 5);
        assert_eq!(result, emoji, "emoji should not be truncated when it fits");
        assert!(string_width(&result) <= 5);
    }

    #[test]
    fn test_truncate_preserves_grapheme_boundary() {
        // Two ZWJ emojis back to back — each width 2, total 4
        let two = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}";
        let result = truncate_to_width(two, 5);
        assert!(
            string_width(&result) <= 5,
            "result width should not exceed max"
        );
    }

    // -----------------------------------------------------------------------
    // analyze tests (basic)
    // -----------------------------------------------------------------------

    #[test]
    fn test_analyze_ascii() {
        let a = analyze("hello");
        assert!(a.is_ascii);
        assert!(!a.has_unicode);
        assert_eq!(a.display_width, 5);
        assert_eq!(a.byte_len, 5);
        assert_eq!(a.char_count, 5);
        assert_eq!(a.grapheme_count, 5);
        assert!(!a.has_emoji);
    }

    #[test]
    fn test_analyze_unicode() {
        let a = analyze("caf\u{00e9}");
        assert!(!a.is_ascii);
        assert!(a.has_unicode);
        assert_eq!(a.char_count, 4);
        assert_eq!(a.display_width, 4);
    }

    #[test]
    fn test_analyze_empty() {
        let a = analyze("");
        assert!(a.is_ascii);
        assert_eq!(a.byte_len, 0);
        assert_eq!(a.char_count, 0);
        assert_eq!(a.display_width, 0);
        assert_eq!(a.grapheme_count, 0);
        assert!(!a.has_unicode);
        assert!(!a.has_emoji);
    }

    #[test]
    fn test_analyze_cjk() {
        let a = analyze("\u{65e5}\u{672c}\u{8a9e}");
        assert_eq!(a.char_count, 3);
        assert_eq!(a.display_width, 6);
        assert_eq!(a.grapheme_count, 3);
        assert!(a.has_unicode);
        assert!(!a.is_ascii);
    }

    // -----------------------------------------------------------------------
    // draw_box tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_draw_box_single_line() {
        let output = draw_box(&["Hello"]);
        assert!(
            output.contains('\u{250c}'),
            "should contain top-left corner"
        );
        assert!(
            output.contains('\u{2518}'),
            "should contain bottom-right corner"
        );
        assert!(output.contains("Hello"));
    }

    #[test]
    fn test_draw_box_multiple_lines() {
        let output = draw_box(&["Hello", "World"]);
        assert!(output.contains("Hello"));
        assert!(output.contains("World"));
    }

    #[test]
    fn test_draw_box_varying_width() {
        let output = draw_box(&["Hi", "Hello"]);
        for line in output.lines() {
            if line.contains("Hi") && !line.contains("Hello") {
                assert!(
                    line.ends_with('\u{2502}'),
                    "content lines should end with box edge"
                );
            }
        }
    }

    #[test]
    fn test_draw_box_empty() {
        let output = draw_box(&[]);
        assert!(
            output.contains('\u{250c}'),
            "empty box should still have top-left corner"
        );
        assert!(
            output.contains('\u{2518}'),
            "empty box should still have bottom-right corner"
        );
    }

    #[test]
    fn test_draw_box_unicode_content() {
        let output = draw_box(&["\u{65e5}\u{672c}\u{8a9e}"]);
        let top_line = output.lines().next().expect("should have a top line");
        let dash_count = top_line.chars().filter(|&c| c == '\u{2500}').count();
        assert_eq!(dash_count, 8, "border should account for CJK double-width");
        assert!(output.contains("\u{65e5}\u{672c}\u{8a9e}"));
    }

    // -----------------------------------------------------------------------
    // truncate_to_width tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_truncate_no_truncation_needed() {
        assert_eq!(truncate_to_width("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_at_boundary() {
        let result = truncate_to_width("hello world", 8);
        assert!(
            string_width(&result) <= 8,
            "truncated result should fit in 8 columns, got width {}",
            string_width(&result)
        );
        assert!(
            result.ends_with("..."),
            "truncated result should end with ellipsis"
        );
    }

    #[test]
    fn test_truncate_very_short() {
        assert_eq!(truncate_to_width("hello", 3), "...");
        assert_eq!(truncate_to_width("hello", 2), "..");
        assert_eq!(truncate_to_width("hello", 1), ".");
        assert_eq!(truncate_to_width("hello", 0), "");
    }

    // -----------------------------------------------------------------------
    // pad_to_width tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_pad_shorter_string() {
        let padded = pad_to_width("hi", 5);
        assert_eq!(string_width(&padded), 5);
        assert!(padded.starts_with("hi"));
    }

    #[test]
    fn test_pad_already_at_width() {
        let padded = pad_to_width("hello", 5);
        assert_eq!(padded, "hello");
    }

    #[test]
    fn test_pad_longer_string_unchanged() {
        let padded = pad_to_width("hello world", 5);
        assert_eq!(padded, "hello world");
    }
}
