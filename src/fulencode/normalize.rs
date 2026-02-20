use unicode_normalization::char::is_combining_mark;
use unicode_normalization::UnicodeNormalization;

use crate::fulencode::types::{
    FulencodeError, FulencodeErrorDetails, NormalizationProfile, NormalizationResult,
    NormalizeOptions, SemanticChange,
};

const DEFAULT_MAX_COMBINING_MARKS: usize = 10;

pub(crate) fn normalize_impl(
    text: &str,
    profile: NormalizationProfile,
    options: Option<&NormalizeOptions>,
) -> Result<NormalizationResult, FulencodeError> {
    let opts = options.cloned().unwrap_or_default();

    let mut out = match profile {
        NormalizationProfile::Nfc => text.nfc().collect::<String>(),
        NormalizationProfile::Nfd => text.nfd().collect::<String>(),
        NormalizationProfile::Nfkc => text.nfkc().collect::<String>(),
        NormalizationProfile::Nfkd => text.nfkd().collect::<String>(),
        NormalizationProfile::TextSafe => text.nfc().collect::<String>(),
        NormalizationProfile::IdentifierSafe
        | NormalizationProfile::FilenameSafe
        | NormalizationProfile::Search
        | NormalizationProfile::Display => {
            return Err(FulencodeError::new(
                "UNSUPPORTED_FORMAT",
                "normalization profile is defined but not implemented in this phase",
                "normalize",
            )
            .with_formats(Some(profile.as_str()), Some(profile.as_str())))
        }
    };

    let mut transformations_applied = vec![profile.as_str().to_string()];
    let mut warnings = Vec::new();
    let mut semantic_changes = Vec::new();

    if matches!(
        profile,
        NormalizationProfile::Nfkc | NormalizationProfile::Nfkd
    ) && opts.warn_semantic_change.unwrap_or(true)
        && out != text
    {
        semantic_changes.push(SemanticChange {
            position: 0,
            original: text.to_string(),
            normalized: out.clone(),
            reason: "compatibility-normalization".to_string(),
        });
        warnings.push("compatibility normalization may be semantic-changing".to_string());
    }

    if matches!(profile, NormalizationProfile::TextSafe) {
        enforce_text_safe_constraints(&out, &opts)?;
    }

    if opts.case_fold.unwrap_or(false) {
        out = out.chars().flat_map(char::to_lowercase).collect();
        transformations_applied.push("case_fold".to_string());
    }

    if opts.strip_accents.unwrap_or(false) {
        let decomposed: String = out.nfkd().collect();
        out = decomposed
            .chars()
            .filter(|c| !is_combining_mark(*c))
            .collect();
        transformations_applied.push("strip_accents".to_string());
    }

    if opts.remove_punctuation.unwrap_or(false) {
        out = out
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect();
        transformations_applied.push("remove_punctuation".to_string());
    }

    if opts.compress_whitespace.unwrap_or(false) {
        out = out.split_whitespace().collect::<Vec<_>>().join(" ");
        transformations_applied.push("compress_whitespace".to_string());
    }

    Ok(NormalizationResult {
        text: out.clone(),
        profile: profile.as_str().to_string(),
        input_length: text.chars().count(),
        output_length: out.chars().count(),
        transformations_applied,
        semantic_changes,
        warnings,
    })
}

fn enforce_text_safe_constraints(
    text: &str,
    opts: &NormalizeOptions,
) -> Result<(), FulencodeError> {
    let reject_zero_width = opts.reject_zero_width.unwrap_or(true);
    let reject_bidi_controls = opts.reject_bidi_controls.unwrap_or(true);
    let max_combining_marks = opts
        .max_combining_marks
        .unwrap_or(DEFAULT_MAX_COMBINING_MARKS);

    let mut combining_run = 0usize;

    for (idx, ch) in text.char_indices() {
        if is_forbidden_control(ch) {
            return Err(FulencodeError::new(
                "INVALID_ENCODING",
                "control character is not allowed in text-safe profile",
                "normalize",
            )
            .with_details(FulencodeErrorDetails {
                codepoint_offset: Some(text[..idx].chars().count()),
                actual: Some(format!("U+{:04X}", ch as u32)),
                ..Default::default()
            }));
        }

        if reject_bidi_controls && is_bidi_control(ch) {
            return Err(FulencodeError::new(
                "BIDI_CONTROL_CHARACTER",
                "bidi control character is not allowed in text-safe profile",
                "normalize",
            )
            .with_details(FulencodeErrorDetails {
                codepoint_offset: Some(text[..idx].chars().count()),
                actual: Some(format!("U+{:04X}", ch as u32)),
                ..Default::default()
            }));
        }

        if reject_zero_width && is_zero_width(ch) {
            return Err(FulencodeError::new(
                "ZERO_WIDTH_CHARACTER",
                "zero-width character is not allowed in text-safe profile",
                "normalize",
            )
            .with_details(FulencodeErrorDetails {
                codepoint_offset: Some(text[..idx].chars().count()),
                actual: Some(format!("U+{:04X}", ch as u32)),
                ..Default::default()
            }));
        }

        if is_combining_mark(ch) {
            combining_run += 1;
            if combining_run > max_combining_marks {
                return Err(FulencodeError::new(
                    "EXCESSIVE_COMBINING_MARKS",
                    "combining mark limit exceeded",
                    "normalize",
                )
                .with_details(FulencodeErrorDetails {
                    codepoint_offset: Some(text[..idx].chars().count()),
                    expected: Some(max_combining_marks.to_string()),
                    actual: Some(combining_run.to_string()),
                    ..Default::default()
                }));
            }
        } else {
            combining_run = 0;
        }
    }

    Ok(())
}

fn is_forbidden_control(c: char) -> bool {
    let cp = c as u32;
    (cp <= 0x1F) || cp == 0x7F || (0x80..=0x9F).contains(&cp)
}

fn is_bidi_control(c: char) -> bool {
    let cp = c as u32;
    (0x202A..=0x202E).contains(&cp)
        || (0x2066..=0x2069).contains(&cp)
        || cp == 0x200E
        || cp == 0x200F
        || cp == 0x061C
}

fn is_zero_width(c: char) -> bool {
    matches!(c as u32, 0x200B | 0x200C | 0x200D | 0xFEFF)
}
