//! Similarity Utilities
//!
//! Provides text similarity helpers for CLI fuzzy matching, typo correction,
//! and "did you mean?" suggestions.
//!
//! This module targets the Crucible Similarity Standard v2.0.0 and is validated
//! against the cross-language fixtures in
//! `config/crucible-rs/library/similarity/fixtures.yaml`.

use std::fmt;
use std::ops::Range;

use unicode_normalization::char::is_combining_mark;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

/// Normalization preset levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NormalizePreset {
    /// No normalization (pass-through).
    None,
    /// NFC + trim whitespace.
    Minimal,
    /// NFC + Unicode casefold (implemented as lowercase) + trim whitespace.
    #[default]
    Default,
    /// NFKD + casefold + strip accents + remove punctuation + trim.
    Aggressive,
}

/// Supported Unicode normalization forms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnicodeForm {
    /// Unicode Normalization Form C (canonical composition).
    Nfc,
    /// Unicode Normalization Form KD (compatibility decomposition).
    Nfkd,
}

/// Custom normalization options.
#[derive(Debug, Clone, Default)]
pub struct NormalizeOptions {
    /// Which Unicode normalization form to apply.
    pub unicode_form: Option<UnicodeForm>,
    /// Apply Unicode case-folding (implemented as lowercase).
    pub casefold: bool,
    /// Strip accent/diacritic marks.
    pub strip_accents: bool,
    /// Remove punctuation and symbols (keeps alphanumeric + whitespace).
    pub remove_punctuation: bool,
    /// Trim leading and trailing whitespace.
    pub trim: bool,
}

/// Normalize a string using a preset.
///
/// Presets are designed for consistent CLI matching behavior across languages.
pub fn normalize(s: &str, preset: NormalizePreset) -> String {
    match preset {
        NormalizePreset::None => s.to_string(),
        NormalizePreset::Minimal => normalize_with_options(
            s,
            &NormalizeOptions {
                unicode_form: Some(UnicodeForm::Nfc),
                casefold: false,
                strip_accents: false,
                remove_punctuation: false,
                trim: true,
            },
        ),
        NormalizePreset::Default => normalize_with_options(
            s,
            &NormalizeOptions {
                unicode_form: Some(UnicodeForm::Nfc),
                casefold: true,
                strip_accents: false,
                remove_punctuation: false,
                trim: true,
            },
        ),
        NormalizePreset::Aggressive => normalize_with_options(
            s,
            &NormalizeOptions {
                unicode_form: Some(UnicodeForm::Nfkd),
                casefold: true,
                strip_accents: true,
                remove_punctuation: true,
                trim: true,
            },
        ),
    }
}

/// Normalize a string using explicit options.
pub fn normalize_with_options(s: &str, options: &NormalizeOptions) -> String {
    let mut out = match options.unicode_form {
        None => s.to_string(),
        Some(UnicodeForm::Nfc) => s.nfc().collect(),
        Some(UnicodeForm::Nfkd) => s.nfkd().collect(),
    };

    if options.casefold {
        out = out.chars().flat_map(|c| c.to_lowercase()).collect();
    }

    if options.strip_accents {
        // Ensure we are decomposed before stripping combining marks.
        let decomposed: String = out.nfkd().collect();
        out = decomposed
            .chars()
            .filter(|c| !is_combining_mark(*c))
            .collect();
    }

    if options.remove_punctuation {
        out = out
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect();
    }

    if options.trim {
        out = out.trim().to_string();
    }

    out
}

/// Levenshtein distance between two strings.
pub fn levenshtein(a: &str, b: &str) -> usize {
    strsim::levenshtein(a, b)
}

/// Levenshtein similarity score (0.0-1.0).
pub fn levenshtein_score(a: &str, b: &str) -> f64 {
    normalized_score_from_distance(levenshtein(a, b), a, b)
}

/// Damerau-Levenshtein (OSA) distance.
///
/// OSA (Optimal String Alignment) disallows editing the same substring twice.
pub fn damerau_osa(a: &str, b: &str) -> usize {
    strsim::osa_distance(a, b)
}

/// Damerau-Levenshtein (OSA) similarity score (0.0-1.0).
pub fn damerau_osa_score(a: &str, b: &str) -> f64 {
    normalized_score_from_distance(damerau_osa(a, b), a, b)
}

/// Damerau-Levenshtein (unrestricted) distance.
///
/// This is the "true" Damerau-Levenshtein distance that allows unrestricted
/// transpositions.
pub fn damerau_levenshtein(a: &str, b: &str) -> usize {
    strsim::damerau_levenshtein(a, b)
}

/// Damerau-Levenshtein (unrestricted) similarity score (0.0-1.0).
pub fn damerau_levenshtein_score(a: &str, b: &str) -> f64 {
    normalized_score_from_distance(damerau_levenshtein(a, b), a, b)
}

/// Jaro-Winkler similarity (0.0-1.0).
pub fn jaro_winkler(a: &str, b: &str) -> f64 {
    strsim::jaro_winkler(a, b)
}

/// Longest common substring ratio.
///
/// Defined as: `lcs_len / max(len(needle), len(haystack))`.
pub fn substring_score(needle: &str, haystack: &str) -> f64 {
    let (best_len, _) = longest_common_substring(needle, haystack);
    let denom = needle
        .graphemes(true)
        .count()
        .max(haystack.graphemes(true).count());
    if denom == 0 {
        return 1.0;
    }

    (best_len as f64) / (denom as f64)
}

/// Find the longest common substring position.
///
/// Returns a byte range into `haystack`. When multiple matches have the same
/// length, ties are broken by choosing the earliest start position.
pub fn substring_range(needle: &str, haystack: &str) -> Option<Range<usize>> {
    let (_, range) = longest_common_substring(needle, haystack);
    range
}

fn normalized_score_from_distance(distance: usize, a: &str, b: &str) -> f64 {
    let denom = a.chars().count().max(b.chars().count());
    if denom == 0 {
        return 1.0;
    }

    let ratio = distance as f64 / denom as f64;
    (1.0 - ratio).max(0.0)
}

#[derive(Debug, Clone)]
struct GraphemeSpan<'a> {
    start: usize,
    end: usize,
    text: &'a str,
}

fn grapheme_spans(s: &str) -> Vec<GraphemeSpan<'_>> {
    s.grapheme_indices(true)
        .map(|(start, text)| GraphemeSpan {
            start,
            end: start + text.len(),
            text,
        })
        .collect()
}

fn longest_common_substring(needle: &str, haystack: &str) -> (usize, Option<Range<usize>>) {
    let needle_gr = grapheme_spans(needle);
    let hay_gr = grapheme_spans(haystack);

    let n = needle_gr.len();
    let m = hay_gr.len();

    if n == 0 || m == 0 {
        return (0, None);
    }

    let mut dp = vec![0usize; m + 1];

    let mut best_len = 0usize;
    let mut best_start_h = 0usize;
    let mut best_start_n = 0usize;
    let mut best_end_h = 0usize;

    for i in 1..=n {
        let mut prev_diag = 0usize;
        for j in 1..=m {
            let saved = dp[j];
            if needle_gr[i - 1].text == hay_gr[j - 1].text {
                let val = prev_diag + 1;
                dp[j] = val;

                let start_h = j - val;
                let start_n = i - val;

                if val > best_len
                    || (val == best_len
                        && val > 0
                        && (start_h < best_start_h
                            || (start_h == best_start_h && start_n < best_start_n)))
                {
                    best_len = val;
                    best_start_h = start_h;
                    best_start_n = start_n;
                    best_end_h = j;
                }
            } else {
                dp[j] = 0;
            }

            prev_diag = saved;
        }
    }

    if best_len == 0 {
        return (0, None);
    }

    let start_byte = hay_gr[best_start_h].start;
    let end_byte = hay_gr[best_end_h - 1].end;

    (best_len, Some(start_byte..end_byte))
}

/// Similarity metric for suggestions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SimilarityMetric {
    /// Levenshtein distance-based score.
    #[default]
    Levenshtein,
    /// Damerau-Levenshtein (OSA) distance-based score.
    DamerauOsa,
    /// Damerau-Levenshtein (unrestricted) distance-based score.
    DamerauLevenshtein,
    /// Jaro-Winkler similarity.
    JaroWinkler,
    /// Longest common substring ratio.
    Substring,
}

impl fmt::Display for SimilarityMetric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SimilarityMetric::Levenshtein => "levenshtein",
            SimilarityMetric::DamerauOsa => "damerau_osa",
            SimilarityMetric::DamerauLevenshtein => "damerau_unrestricted",
            SimilarityMetric::JaroWinkler => "jaro_winkler",
            SimilarityMetric::Substring => "substring",
        };
        f.write_str(s)
    }
}

/// Errors that can occur when computing similarity.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum SimilarityError {
    /// The requested metric does not support a distance API.
    #[error("distance not supported for metric: {0}")]
    DistanceUnsupported(SimilarityMetric),
}

/// Compute a distance for metrics that define one.
///
/// Per spec, this errors for `JaroWinkler` and `Substring`.
pub fn distance(metric: SimilarityMetric, a: &str, b: &str) -> Result<usize, SimilarityError> {
    match metric {
        SimilarityMetric::Levenshtein => Ok(levenshtein(a, b)),
        SimilarityMetric::DamerauOsa => Ok(damerau_osa(a, b)),
        SimilarityMetric::DamerauLevenshtein => Ok(damerau_levenshtein(a, b)),
        SimilarityMetric::JaroWinkler | SimilarityMetric::Substring => {
            Err(SimilarityError::DistanceUnsupported(metric))
        }
    }
}

/// Compute a similarity score (0.0-1.0) for any supported metric.
pub fn score(metric: SimilarityMetric, a: &str, b: &str) -> f64 {
    match metric {
        SimilarityMetric::Levenshtein => levenshtein_score(a, b),
        SimilarityMetric::DamerauOsa => damerau_osa_score(a, b),
        SimilarityMetric::DamerauLevenshtein => damerau_levenshtein_score(a, b),
        SimilarityMetric::JaroWinkler => jaro_winkler(a, b),
        SimilarityMetric::Substring => substring_score(a, b),
    }
}

/// Options for suggestion generation.
#[derive(Debug, Clone)]
pub struct SuggestOptions {
    /// Minimum similarity score threshold.
    pub min_score: f64,
    /// Maximum suggestions to return.
    pub max_suggestions: usize,
    /// Similarity metric to use.
    pub metric: SimilarityMetric,
    /// Normalization preset.
    pub normalize_preset: NormalizePreset,
    /// Prefer prefix matches (Jaro-Winkler sorting tie-break).
    pub prefer_prefix: bool,
}

impl Default for SuggestOptions {
    fn default() -> Self {
        Self {
            min_score: 0.6,
            max_suggestions: 3,
            metric: SimilarityMetric::Levenshtein,
            normalize_preset: NormalizePreset::Default,
            prefer_prefix: false,
        }
    }
}

/// A suggestion with its similarity score.
#[derive(Debug, Clone, PartialEq)]
pub struct Suggestion {
    /// Suggested candidate value.
    pub value: String,
    /// Similarity score for the suggestion.
    pub score: f64,
}

#[derive(Debug)]
struct ScoredSuggestion {
    suggestion: Suggestion,
    prefix_match: bool,
}

/// Generate suggestions for input from candidates.
pub fn suggest(input: &str, candidates: &[&str], options: &SuggestOptions) -> Vec<Suggestion> {
    if candidates.is_empty() {
        return Vec::new();
    }

    let normalized_input = normalize(input, options.normalize_preset);

    let mut scored: Vec<ScoredSuggestion> = candidates
        .iter()
        .filter_map(|&candidate| {
            let normalized_candidate = normalize(candidate, options.normalize_preset);
            let score = score(options.metric, &normalized_input, &normalized_candidate);

            if score < options.min_score {
                return None;
            }

            let prefix_match = options.prefer_prefix
                && options.metric == SimilarityMetric::JaroWinkler
                && normalized_candidate.starts_with(&normalized_input);

            Some(ScoredSuggestion {
                suggestion: Suggestion {
                    value: candidate.to_string(),
                    score,
                },
                prefix_match,
            })
        })
        .collect();

    scored.sort_by(|a, b| {
        b.suggestion
            .score
            .partial_cmp(&a.suggestion.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.prefix_match.cmp(&a.prefix_match))
            .then_with(|| a.suggestion.value.cmp(&b.suggestion.value))
    });

    scored
        .into_iter()
        .take(options.max_suggestions)
        .map(|s| s.suggestion)
        .collect()
}

/// Simple suggest with defaults (min_score=0.6, max=3).
pub fn suggest_simple(input: &str, candidates: &[&str]) -> Vec<Suggestion> {
    suggest(input, candidates, &SuggestOptions::default())
}

/// Format a "Did you mean?" message for CLI output.
pub fn format_did_you_mean(suggestions: &[Suggestion]) -> Option<String> {
    if suggestions.is_empty() {
        return None;
    }

    let values: Vec<String> = suggestions
        .iter()
        .map(|s| format!("'{}'", s.value))
        .collect();

    let msg = match values.len() {
        1 => format!("Did you mean {}?", values[0]),
        2 => format!("Did you mean {} or {}?", values[0], values[1]),
        _ => {
            let mut out = String::new();
            out.push_str("Did you mean ");
            for (i, v) in values.iter().enumerate() {
                if i == values.len() - 1 {
                    out.push_str("or ");
                    out.push_str(v);
                } else {
                    out.push_str(v);
                    out.push_str(", ");
                }
            }
            out.push('?');
            out
        }
    };

    Some(msg)
}

/// Check if input matches any candidate exactly (case-insensitive via default normalization).
pub fn exact_match<'a>(input: &str, candidates: &'a [&'a str]) -> Option<&'a str> {
    let normalized_input = normalize(input, NormalizePreset::Default);

    candidates
        .iter()
        .find(|&&candidate| normalize(candidate, NormalizePreset::Default) == normalized_input)
        .copied()
}

/// Get closest single match above threshold.
pub fn closest_match(input: &str, candidates: &[&str], min_score: f64) -> Option<Suggestion> {
    let options = SuggestOptions {
        min_score,
        max_suggestions: 1,
        ..SuggestOptions::default()
    };

    suggest(input, candidates, &options).into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURES_YAML: &str =
        include_str!("../../config/crucible-rs/library/similarity/fixtures.yaml");

    #[derive(Debug, serde::Deserialize)]
    struct Fixtures {
        #[allow(dead_code)]
        version: String,
        test_cases: Vec<TestCaseGroup>,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(tag = "category")]
    enum TestCaseGroup {
        #[serde(rename = "levenshtein")]
        Levenshtein { cases: Vec<DistanceCase> },
        #[serde(rename = "damerau_osa")]
        DamerauOsa { cases: Vec<DistanceCase> },
        #[serde(rename = "damerau_unrestricted")]
        DamerauUnrestricted { cases: Vec<DistanceCase> },
        #[serde(rename = "jaro_winkler")]
        JaroWinkler { cases: Vec<JaroWinklerCase> },
        #[serde(rename = "substring")]
        Substring { cases: Vec<SubstringCase> },
        #[serde(rename = "normalization_presets")]
        NormalizationPresets { cases: Vec<NormalizationCase> },
        #[serde(rename = "suggestions")]
        Suggestions { cases: Vec<SuggestionCase> },
    }

    #[derive(Debug, serde::Deserialize)]
    struct DistanceCase {
        input_a: String,
        input_b: String,
        expected_distance: usize,
        expected_score: f64,
    }

    #[derive(Debug, serde::Deserialize)]
    struct JaroWinklerCase {
        input_a: String,
        input_b: String,
        expected_score: f64,
    }

    #[derive(Debug, serde::Deserialize)]
    struct ExpectedRange {
        start: usize,
        end: usize,
    }

    #[derive(Debug, serde::Deserialize)]
    struct SubstringCase {
        needle: String,
        haystack: String,
        expected_score: f64,
        #[serde(default)]
        expected_range: Option<ExpectedRange>,
    }

    #[derive(Debug, Clone, Copy, serde::Deserialize)]
    #[serde(rename_all = "lowercase")]
    enum FixturePreset {
        None,
        Minimal,
        Default,
        Aggressive,
    }

    impl From<FixturePreset> for NormalizePreset {
        fn from(value: FixturePreset) -> Self {
            match value {
                FixturePreset::None => NormalizePreset::None,
                FixturePreset::Minimal => NormalizePreset::Minimal,
                FixturePreset::Default => NormalizePreset::Default,
                FixturePreset::Aggressive => NormalizePreset::Aggressive,
            }
        }
    }

    #[derive(Debug, serde::Deserialize)]
    struct NormalizationCase {
        input: String,
        preset: FixturePreset,
        expected: String,
    }

    #[derive(Debug, Clone, Copy, serde::Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum FixtureMetric {
        Levenshtein,
        DamerauOsa,
        DamerauUnrestricted,
        JaroWinkler,
        Substring,
    }

    impl From<FixtureMetric> for SimilarityMetric {
        fn from(value: FixtureMetric) -> Self {
            match value {
                FixtureMetric::Levenshtein => SimilarityMetric::Levenshtein,
                FixtureMetric::DamerauOsa => SimilarityMetric::DamerauOsa,
                FixtureMetric::DamerauUnrestricted => SimilarityMetric::DamerauLevenshtein,
                FixtureMetric::JaroWinkler => SimilarityMetric::JaroWinkler,
                FixtureMetric::Substring => SimilarityMetric::Substring,
            }
        }
    }

    #[derive(Debug, serde::Deserialize)]
    struct SuggestFixtureOptions {
        min_score: f64,
        max_suggestions: usize,
        metric: FixtureMetric,
        normalize_preset: FixturePreset,
        #[serde(default)]
        prefer_prefix: bool,
    }

    #[derive(Debug, serde::Deserialize)]
    struct SuggestionFixture {
        value: String,
        score: f64,
    }

    #[derive(Debug, serde::Deserialize)]
    struct SuggestionCase {
        input: String,
        candidates: Vec<String>,
        options: SuggestFixtureOptions,
        expected: Vec<SuggestionFixture>,
    }

    fn load_fixtures() -> Fixtures {
        serde_yaml::from_str(FIXTURES_YAML).expect("fixtures should parse")
    }

    fn assert_f64_eq(actual: f64, expected: f64, context: &str) {
        let diff = (actual - expected).abs();
        assert!(
            diff < 1e-12,
            "{context}: expected {expected}, got {actual} (diff {diff})"
        );
    }

    #[test]
    fn test_similarity_fixtures() {
        let fixtures = load_fixtures();

        for group in fixtures.test_cases {
            match group {
                TestCaseGroup::Levenshtein { cases } => {
                    for case in cases {
                        let dist = levenshtein(&case.input_a, &case.input_b);
                        assert_eq!(dist, case.expected_distance);
                        let score = levenshtein_score(&case.input_a, &case.input_b);
                        assert_f64_eq(score, case.expected_score, "levenshtein_score");
                    }
                }
                TestCaseGroup::DamerauOsa { cases } => {
                    for case in cases {
                        let dist = damerau_osa(&case.input_a, &case.input_b);
                        assert_eq!(dist, case.expected_distance);
                        let score = damerau_osa_score(&case.input_a, &case.input_b);
                        assert_f64_eq(score, case.expected_score, "damerau_osa_score");
                    }
                }
                TestCaseGroup::DamerauUnrestricted { cases } => {
                    for case in cases {
                        let dist = damerau_levenshtein(&case.input_a, &case.input_b);
                        assert_eq!(dist, case.expected_distance);
                        let score = damerau_levenshtein_score(&case.input_a, &case.input_b);
                        assert_f64_eq(score, case.expected_score, "damerau_unrestricted_score");
                    }
                }
                TestCaseGroup::JaroWinkler { cases } => {
                    for case in cases {
                        let score = jaro_winkler(&case.input_a, &case.input_b);
                        assert_f64_eq(score, case.expected_score, "jaro_winkler");
                    }
                }
                TestCaseGroup::Substring { cases } => {
                    for case in cases {
                        let score = substring_score(&case.needle, &case.haystack);
                        assert_f64_eq(score, case.expected_score, "substring_score");

                        let range = substring_range(&case.needle, &case.haystack);
                        match (range, case.expected_range) {
                            (None, None) => {}
                            (Some(actual), Some(expected)) => {
                                assert_eq!(actual.start, expected.start);
                                assert_eq!(actual.end, expected.end);
                            }
                            (actual, expected) => panic!(
                                "substring_range mismatch: actual={actual:?} expected={expected:?}"
                            ),
                        }
                    }
                }
                TestCaseGroup::NormalizationPresets { cases } => {
                    for case in cases {
                        let preset: NormalizePreset = case.preset.into();
                        let actual = normalize(&case.input, preset);
                        assert_eq!(actual, case.expected);
                    }
                }
                TestCaseGroup::Suggestions { cases } => {
                    for case in cases {
                        let candidates: Vec<&str> =
                            case.candidates.iter().map(|s| s.as_str()).collect();

                        let options = SuggestOptions {
                            min_score: case.options.min_score,
                            max_suggestions: case.options.max_suggestions,
                            metric: case.options.metric.into(),
                            normalize_preset: case.options.normalize_preset.into(),
                            prefer_prefix: case.options.prefer_prefix,
                        };

                        let actual = suggest(&case.input, &candidates, &options);
                        assert_eq!(actual.len(), case.expected.len());

                        for (actual_s, expected_s) in actual.iter().zip(case.expected.iter()) {
                            assert_eq!(&actual_s.value, &expected_s.value);
                            assert_f64_eq(actual_s.score, expected_s.score, "suggest score");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_distance_errors_for_non_distance_metrics() {
        assert!(matches!(
            distance(SimilarityMetric::JaroWinkler, "a", "b"),
            Err(SimilarityError::DistanceUnsupported(_))
        ));

        assert!(matches!(
            distance(SimilarityMetric::Substring, "a", "b"),
            Err(SimilarityError::DistanceUnsupported(_))
        ));
    }

    #[test]
    fn test_format_did_you_mean() {
        let s = vec![Suggestion {
            value: "build".to_string(),
            score: 0.9,
        }];
        assert_eq!(
            format_did_you_mean(&s),
            Some("Did you mean 'build'?".to_string())
        );

        let s = vec![
            Suggestion {
                value: "build".to_string(),
                score: 0.9,
            },
            Suggestion {
                value: "test".to_string(),
                score: 0.8,
            },
        ];
        assert_eq!(
            format_did_you_mean(&s),
            Some("Did you mean 'build' or 'test'?".to_string())
        );
    }
}
