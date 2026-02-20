//! Fulencode: canonical encoding/decoding, detection, normalization, and BOM helpers.
#![allow(clippy::result_large_err)]

mod bom;
mod decode;
mod detect;
mod encode;
mod normalize;
mod types;

pub use types::{
    BomResult, ConfidenceLevel, DecodeOptions, DecodingResult, DetectOptions, DetectionResult,
    EncodeOptions, EncodingFormat, EncodingResult, FulencodeError, FulencodeErrorDetails,
    NormalizationProfile, NormalizationResult, NormalizeOptions, SemanticChange,
};

/// Encode raw bytes into a requested format.
pub fn encode(
    data: &[u8],
    format: EncodingFormat,
    options: Option<&EncodeOptions>,
) -> Result<EncodingResult, FulencodeError> {
    encode::encode_impl(data, format, options)
}

/// Decode text payload into raw bytes.
pub fn decode(
    data: &str,
    format: EncodingFormat,
    options: Option<&DecodeOptions>,
) -> Result<DecodingResult, FulencodeError> {
    decode::decode_impl(data, format, options)
}

/// Detect likely encoding from raw bytes.
pub fn detect(
    data: &[u8],
    options: Option<&DetectOptions>,
) -> Result<DetectionResult, FulencodeError> {
    detect::detect_impl(data, options)
}

/// Normalize Unicode text using a profile.
pub fn normalize(
    text: &str,
    profile: NormalizationProfile,
    options: Option<&NormalizeOptions>,
) -> Result<NormalizationResult, FulencodeError> {
    normalize::normalize_impl(text, profile, options)
}

/// Detect a byte-order-mark (BOM) in input.
pub fn detect_bom(data: &[u8]) -> Result<BomResult, FulencodeError> {
    Ok(bom::detect_bom_impl(data))
}

/// Remove a BOM prefix when present.
///
/// If `expected_encoding` is provided and a BOM is present, the BOM must match
/// that encoding or `BOM_MISMATCH` is returned.
pub fn remove_bom(data: &[u8], expected_encoding: Option<&str>) -> Result<Vec<u8>, FulencodeError> {
    bom::remove_bom_impl(data, expected_encoding)
}

/// Prepend a BOM for an encoding that defines one.
pub fn add_bom(data: &[u8], encoding: &str) -> Result<Vec<u8>, FulencodeError> {
    bom::add_bom_impl(data, encoding)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde::Deserialize;

    use super::*;

    #[derive(Debug, Deserialize)]
    struct FixtureCases<T> {
        cases: Vec<T>,
    }

    #[derive(Debug, Deserialize)]
    struct ValidEncodingCase {
        format: String,
        input_hex: String,
        encoded: String,
    }

    #[derive(Debug, Deserialize)]
    struct InvalidEncodingCase {
        name: String,
        format: String,
        encoded: String,
        expected_error_code: String,
    }

    #[derive(Debug, Deserialize)]
    struct DetectionCase {
        input_hex: String,
        expected: DetectionExpected,
    }

    #[derive(Debug, Deserialize)]
    struct DetectionExpected {
        encoding: String,
        level: String,
    }

    #[derive(Debug, Deserialize)]
    struct BomCase {
        input_hex: String,
        expected: BomExpected,
    }

    #[derive(Debug, Deserialize)]
    struct BomExpected {
        bom_type: Option<String>,
        byte_length: usize,
        encoding_implied: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    struct TextSafeCase {
        input: String,
        expected_error_code: String,
    }

    fn fixture_path(parts: &[&str]) -> PathBuf {
        let mut base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        base.push("config");
        base.push("crucible-rs");
        base.push("library");
        base.push("fulencode");
        base.push("fixtures");
        for part in parts {
            base.push(part);
        }
        base
    }

    #[test]
    fn valid_encoding_fixture_cases() {
        let file = fixture_path(&["valid-encodings", "base64.yaml"]);
        let fixture: FixtureCases<ValidEncodingCase> =
            serde_yaml::from_str(&std::fs::read_to_string(file).expect("fixture should exist"))
                .expect("valid fixture yaml");

        for case in fixture.cases {
            let format = EncodingFormat::parse(&case.format).expect("supported test format");
            let input = hex::decode(case.input_hex).expect("valid hex test input");
            let encoded = encode(&input, format, None).expect("encode should succeed");
            assert_eq!(encoded.data, case.encoded, "format={}", case.format);
        }
    }

    #[test]
    fn invalid_encoding_fixture_cases() {
        let file = fixture_path(&["invalid-encodings", "base64.yaml"]);
        let fixture: FixtureCases<InvalidEncodingCase> =
            serde_yaml::from_str(&std::fs::read_to_string(file).expect("fixture should exist"))
                .expect("valid fixture yaml");

        for case in fixture.cases {
            let format = EncodingFormat::parse(&case.format).expect("supported test format");
            match decode(&case.encoded, format, None) {
                Err(err) => {
                    assert_eq!(err.code, case.expected_error_code, "input={}", case.encoded)
                }
                Ok(_) => {
                    // Known fixture drift: a single '=' pad is canonical for some Base64
                    // payload lengths. Keep this explicit until SSOT fixtures are updated.
                    assert_eq!(
                        case.name, "invalid-base64-padding",
                        "unexpected success in invalid fixture case"
                    );
                    assert_eq!(case.encoded, "SGVsbG8=");
                }
            }
        }
    }

    #[test]
    fn detection_fixture_cases() {
        let file = fixture_path(&["detection", "detection.yaml"]);
        let fixture: FixtureCases<DetectionCase> =
            serde_yaml::from_str(&std::fs::read_to_string(file).expect("fixture should exist"))
                .expect("valid fixture yaml");

        for case in fixture.cases {
            let input = hex::decode(case.input_hex).expect("valid hex test input");
            let detected = detect(&input, None).expect("detect should succeed");
            assert_eq!(detected.encoding, Some(case.expected.encoding));
            let level = match detected.level {
                ConfidenceLevel::High => "high",
                ConfidenceLevel::Medium => "medium",
                ConfidenceLevel::Low => "low",
            };
            assert_eq!(level, case.expected.level);
        }
    }

    #[test]
    fn bom_fixture_cases() {
        let file = fixture_path(&["bom", "bom.yaml"]);
        let fixture: FixtureCases<BomCase> =
            serde_yaml::from_str(&std::fs::read_to_string(file).expect("fixture should exist"))
                .expect("valid fixture yaml");

        for case in fixture.cases {
            let input = hex::decode(case.input_hex).expect("valid hex test input");
            let bom = detect_bom(&input).expect("BOM detection should succeed");
            assert_eq!(bom.bom_type, case.expected.bom_type);
            assert_eq!(bom.byte_length, case.expected.byte_length);
            assert_eq!(bom.encoding_implied, case.expected.encoding_implied);
        }
    }

    #[test]
    fn text_safe_fixture_cases() {
        let file = fixture_path(&["normalization", "text-safe.yaml"]);
        let fixture: FixtureCases<TextSafeCase> =
            serde_yaml::from_str(&std::fs::read_to_string(file).expect("fixture should exist"))
                .expect("valid fixture yaml");

        for case in fixture.cases {
            let err = normalize(&case.input, NormalizationProfile::TextSafe, None)
                .expect_err("text-safe case should fail");
            assert_eq!(err.code, case.expected_error_code);
        }
    }

    #[test]
    fn round_trip_base64() {
        let input = b"hello, world";
        let enc = encode(input, EncodingFormat::Base64, None).expect("encode should succeed");
        let dec = decode(&enc.data, EncodingFormat::Base64, None).expect("decode should succeed");
        assert_eq!(dec.data, input);
    }

    #[test]
    fn round_trip_base64url_unpadded() {
        let input = b"hello, world";
        let enc = encode(input, EncodingFormat::Base64url, None).expect("encode should succeed");
        let dec =
            decode(&enc.data, EncodingFormat::Base64url, None).expect("decode should succeed");
        assert_eq!(dec.data, input);
    }

    #[test]
    fn decode_base64_default_strict_rejects_missing_padding() {
        let err = decode("SGVsbG8", EncodingFormat::Base64, None)
            .expect_err("strict default must reject missing padding");
        assert_eq!(err.code, "INVALID_ENCODING");
    }

    #[test]
    fn decode_base64_allows_single_padding_character() {
        let out = decode("SGVsbG8=", EncodingFormat::Base64, None)
            .expect("single '=' padding is canonical for 2-byte remainder");
        assert_eq!(out.data, b"Hello");
    }

    #[test]
    fn decode_base64_can_recover_in_non_strict_mode() {
        let opts = DecodeOptions {
            on_error: Some("fallback".to_string()),
            ..Default::default()
        };
        let out = decode("SGVsbG8", EncodingFormat::Base64, Some(&opts))
            .expect("non-strict mode may repair missing padding");
        assert_eq!(out.data, b"Hello");
    }

    #[test]
    fn decode_base64_default_ignores_whitespace() {
        let out = decode("SGVs bG8s\nIFdvcmxkIQ==", EncodingFormat::Base64, None)
            .expect("base64 should ignore whitespace by default");
        assert_eq!(out.data, b"Hello, World!");
    }

    #[test]
    fn decode_hex_default_ignores_whitespace() {
        let out = decode("48 65\n6c\t6c 6f", EncodingFormat::Hex, None)
            .expect("hex should ignore whitespace by default");
        assert_eq!(out.data, b"Hello");
    }

    #[test]
    fn decode_base64_whitespace_can_be_forced_strict() {
        let opts = DecodeOptions {
            ignore_whitespace: Some(false),
            ..Default::default()
        };
        let err = decode("SGVs bG8=", EncodingFormat::Base64, Some(&opts))
            .expect_err("explicit strict mode should reject whitespace");
        assert_eq!(err.code, "INVALID_ENCODING");
    }

    #[test]
    fn round_trip_utf16le() {
        let text = "Hello 🌍";
        let raw = decode(text, EncodingFormat::Utf16le, None)
            .expect("utf16 encode should succeed")
            .data;
        let recoded =
            encode(&raw, EncodingFormat::Utf16le, None).expect("utf16 decode should succeed");
        assert_eq!(recoded.data, text);
    }

    #[test]
    fn bom_round_trip_utf8() {
        let data = b"hello";
        let with = add_bom(data, "utf-8").expect("should add bom");
        let stripped = remove_bom(&with, Some("utf-8")).expect("should remove bom");
        assert_eq!(stripped, data);
    }

    #[test]
    fn remove_bom_detects_mismatch_when_expected_encoding_is_set() {
        let with = add_bom(b"hello", "utf-8").expect("should add bom");
        let err = remove_bom(&with, Some("utf-16le")).expect_err("must reject mismatched BOM");
        assert_eq!(err.code, "BOM_MISMATCH");
    }

    #[test]
    fn detect_rejects_low_confidence_when_threshold_set() {
        let opts = DetectOptions {
            min_confidence: Some(0.9),
            ..Default::default()
        };
        let err = detect(b"Hello", Some(&opts)).expect_err("should reject low confidence");
        assert_eq!(err.code, "DETECTION_FAILED");
    }

    #[test]
    fn decode_rejects_expansion_ratio() {
        let input = "SGVsbG8sIFdvcmxkIQ==";
        let opts = DecodeOptions {
            max_expansion_ratio: Some(0.5),
            ..Default::default()
        };
        let err = decode(input, EncodingFormat::Base64, Some(&opts)).expect_err("must reject bomb");
        assert_eq!(err.code, "ENCODING_BOMB");
    }

    #[test]
    fn normalize_rejects_excessive_combining_marks() {
        let input = format!("e{}", "\u{0301}".repeat(12));
        let err = normalize(&input, NormalizationProfile::TextSafe, None)
            .expect_err("must reject excessive combining marks");
        assert_eq!(err.code, "EXCESSIVE_COMBINING_MARKS");
    }

    #[test]
    fn normalize_lengths_are_codepoint_counts() {
        let result =
            normalize("é", NormalizationProfile::Nfd, None).expect("normalize should succeed");
        assert_eq!(result.input_length, 1);
        assert_eq!(result.output_length, 2);
    }

    #[test]
    fn normalize_warn_semantic_change_defaults_true_for_nfkc() {
        let result =
            normalize("ﬁ", NormalizationProfile::Nfkc, None).expect("normalize should succeed");
        assert!(
            !result.warnings.is_empty(),
            "default should warn about semantic change"
        );
        assert!(
            !result.semantic_changes.is_empty(),
            "default should track semantic change"
        );
    }
}
