use serde::{Deserialize, Serialize};

/// Supported encoding formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncodingFormat {
    /// RFC 4648 base64.
    #[serde(rename = "base64")]
    Base64,
    /// RFC 4648 URL-safe base64.
    #[serde(rename = "base64url")]
    Base64url,
    /// Raw base64 without padding.
    #[serde(rename = "base64_raw")]
    Base64Raw,
    /// RFC 4648 base32.
    #[serde(rename = "base32")]
    Base32,
    /// RFC 4648 base32hex.
    #[serde(rename = "base32hex")]
    Base32hex,
    /// Hex (base16).
    #[serde(rename = "hex")]
    Hex,
    /// UTF-8.
    #[serde(rename = "utf-8")]
    Utf8,
    /// UTF-16LE.
    #[serde(rename = "utf-16le")]
    Utf16le,
    /// UTF-16BE.
    #[serde(rename = "utf-16be")]
    Utf16be,
    /// ISO-8859-1.
    #[serde(rename = "iso-8859-1")]
    Iso88591,
    /// Windows-1252.
    #[serde(rename = "cp1252")]
    Cp1252,
    /// 7-bit ASCII.
    #[serde(rename = "ascii")]
    Ascii,
}

impl EncodingFormat {
    /// Canonical schema value.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Base64 => "base64",
            Self::Base64url => "base64url",
            Self::Base64Raw => "base64_raw",
            Self::Base32 => "base32",
            Self::Base32hex => "base32hex",
            Self::Hex => "hex",
            Self::Utf8 => "utf-8",
            Self::Utf16le => "utf-16le",
            Self::Utf16be => "utf-16be",
            Self::Iso88591 => "iso-8859-1",
            Self::Cp1252 => "cp1252",
            Self::Ascii => "ascii",
        }
    }

    /// Parse from a canonical schema value.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "base64" => Some(Self::Base64),
            "base64url" => Some(Self::Base64url),
            "base64_raw" => Some(Self::Base64Raw),
            "base32" => Some(Self::Base32),
            "base32hex" => Some(Self::Base32hex),
            "hex" => Some(Self::Hex),
            "utf-8" => Some(Self::Utf8),
            "utf-16le" => Some(Self::Utf16le),
            "utf-16be" => Some(Self::Utf16be),
            "iso-8859-1" => Some(Self::Iso88591),
            "cp1252" => Some(Self::Cp1252),
            "ascii" => Some(Self::Ascii),
            _ => None,
        }
    }
}

/// Supported normalization profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NormalizationProfile {
    /// Unicode NFC.
    Nfc,
    /// Unicode NFD.
    Nfd,
    /// Unicode NFKC.
    Nfkc,
    /// Unicode NFKD.
    Nfkd,
    /// Fulencode text-safe profile.
    #[serde(rename = "text-safe")]
    TextSafe,
    /// Identifier-safe profile (deferred in v1.0 implementation).
    #[serde(rename = "identifier-safe")]
    IdentifierSafe,
    /// Filename-safe profile (deferred in v1.0 implementation).
    #[serde(rename = "filename-safe")]
    FilenameSafe,
    /// Search profile (deferred in v1.0 implementation).
    Search,
    /// Display profile (deferred in v1.0 implementation).
    Display,
}

impl NormalizationProfile {
    /// Canonical schema/profile value.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Nfc => "nfc",
            Self::Nfd => "nfd",
            Self::Nfkc => "nfkc",
            Self::Nfkd => "nfkd",
            Self::TextSafe => "text-safe",
            Self::IdentifierSafe => "identifier-safe",
            Self::FilenameSafe => "filename-safe",
            Self::Search => "search",
            Self::Display => "display",
        }
    }
}

/// Detection confidence level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfidenceLevel {
    /// High confidence (>= 0.9).
    High,
    /// Medium confidence (>= 0.5 and < 0.9).
    Medium,
    /// Low confidence (< 0.5).
    Low,
}

/// Options for encode().
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EncodeOptions {
    /// Enable/disable padding when supported.
    #[serde(default)]
    pub padding: Option<bool>,
    /// Hex output case.
    #[serde(default)]
    pub case: Option<String>,
    /// Optional line wrap length.
    #[serde(default)]
    pub line_length: Option<usize>,
    /// Line ending used with wrapping.
    #[serde(default)]
    pub line_ending: Option<String>,
    /// Maximum encoded size.
    #[serde(default)]
    pub max_encoded_size: Option<usize>,
    /// Checksum algorithm.
    #[serde(default)]
    pub compute_checksum: Option<String>,
    /// Embed checksum framing.
    #[serde(default)]
    pub embed_checksum: Option<bool>,
    /// Error mode.
    #[serde(default)]
    pub on_error: Option<String>,
}

/// Options for decode().
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecodeOptions {
    /// Verify checksum when present.
    #[serde(default)]
    pub verify_checksum: Option<bool>,
    /// Compute checksum algorithm.
    #[serde(default)]
    pub compute_checksum: Option<String>,
    /// Maximum decoded size.
    #[serde(default)]
    pub max_decoded_size: Option<usize>,
    /// Maximum expansion ratio.
    #[serde(default)]
    pub max_expansion_ratio: Option<f64>,
    /// Error mode.
    #[serde(default)]
    pub on_error: Option<String>,
    /// Fallback formats.
    #[serde(default)]
    pub fallback_formats: Option<Vec<String>>,
    /// Ignore ASCII whitespace in encoded input.
    #[serde(default)]
    pub ignore_whitespace: Option<bool>,
    /// Validate padding in strict mode.
    #[serde(default)]
    pub validate_padding: Option<bool>,
}

/// Options for detect().
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DetectOptions {
    /// Maximum bytes to sample.
    #[serde(default)]
    pub max_sample_size: Option<usize>,
    /// Minimum confidence threshold.
    #[serde(default)]
    pub min_confidence: Option<f64>,
    /// Recognize multibase prefixes.
    #[serde(default)]
    pub recognize_multibase: Option<bool>,
}

/// Options for normalize().
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NormalizeOptions {
    /// Warn on semantic change for compatibility forms.
    #[serde(default)]
    pub warn_semantic_change: Option<bool>,
    /// Reject zero-width characters.
    #[serde(default)]
    pub reject_zero_width: Option<bool>,
    /// Reject bidi controls.
    #[serde(default)]
    pub reject_bidi_controls: Option<bool>,
    /// Maximum combining marks per base character.
    #[serde(default)]
    pub max_combining_marks: Option<usize>,
    /// Strip accents/diacritics.
    #[serde(default)]
    pub strip_accents: Option<bool>,
    /// Apply Unicode case fold.
    #[serde(default)]
    pub case_fold: Option<bool>,
    /// Remove punctuation/symbol categories.
    #[serde(default)]
    pub remove_punctuation: Option<bool>,
    /// Collapse whitespace runs.
    #[serde(default)]
    pub compress_whitespace: Option<bool>,
}

/// Result of encode().
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodingResult {
    /// Encoded data.
    pub data: String,
    /// Output format.
    pub format: EncodingFormat,
    /// Input byte length.
    pub input_size: usize,
    /// Output byte length.
    pub output_size: usize,
    /// Non-fatal warnings.
    pub warnings: Vec<String>,
    /// Optional checksum.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    /// Optional checksum algorithm.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum_algorithm: Option<String>,
}

/// Result of decode().
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodingResult {
    /// Decoded raw bytes.
    pub data: Vec<u8>,
    /// Input format.
    pub format: EncodingFormat,
    /// Input byte length.
    pub input_size: usize,
    /// Output byte length.
    pub output_size: usize,
    /// Non-fatal warnings.
    pub warnings: Vec<String>,
    /// Number of corrections applied.
    pub corrections_applied: usize,
    /// Optional checksum.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    /// Optional checksum verification status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum_verified: Option<bool>,
    /// Optional checksum algorithm.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum_algorithm: Option<String>,
}

/// Result of detect().
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    /// Detected encoding or null.
    pub encoding: Option<String>,
    /// Confidence score [0, 1].
    pub confidence: f64,
    /// Confidence band.
    pub level: ConfidenceLevel,
    /// Detected multibase prefix if present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multibase_prefix: Option<String>,
    /// Non-fatal warnings.
    pub warnings: Vec<String>,
}

/// Semantic normalization delta.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticChange {
    /// Position in original codepoint sequence.
    pub position: usize,
    /// Original segment.
    pub original: String,
    /// Normalized segment.
    pub normalized: String,
    /// Reason code.
    pub reason: String,
}

/// Result of normalize().
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationResult {
    /// Normalized text.
    pub text: String,
    /// Applied profile.
    pub profile: String,
    /// Input byte length.
    pub input_length: usize,
    /// Output byte length.
    pub output_length: usize,
    /// Applied transformations.
    pub transformations_applied: Vec<String>,
    /// Semantic changes detected.
    pub semantic_changes: Vec<SemanticChange>,
    /// Non-fatal warnings.
    pub warnings: Vec<String>,
}

/// Result of BOM detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BomResult {
    /// BOM type, if detected.
    pub bom_type: Option<String>,
    /// BOM byte length (0 if none).
    pub byte_length: usize,
    /// Encoding implied by BOM.
    pub encoding_implied: Option<String>,
}

/// Additional error details.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FulencodeErrorDetails {
    /// Byte offset where failure occurred.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub byte_offset: Option<usize>,
    /// Codepoint offset where failure occurred.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codepoint_offset: Option<usize>,
    /// Detected encoding name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detected_encoding: Option<String>,
    /// Detection confidence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    /// Invalid bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invalid_bytes: Option<Vec<u8>>,
    /// Expected value/context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    /// Actual value/context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actual: Option<String>,
}

/// Canonical serializable error envelope for fulencode operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulencodeError {
    /// Canonical error code.
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// Operation name.
    pub operation: String,
    /// Input format, if relevant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_format: Option<String>,
    /// Output format, if relevant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
    /// Optional details.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<FulencodeErrorDetails>,
}

impl FulencodeError {
    /// Construct a canonical fulencode error.
    pub fn new(code: &str, message: &str, operation: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            operation: operation.to_string(),
            input_format: None,
            output_format: None,
            details: None,
        }
    }

    /// Attach format metadata to an error.
    pub fn with_formats(mut self, input_format: Option<&str>, output_format: Option<&str>) -> Self {
        self.input_format = input_format.map(str::to_string);
        self.output_format = output_format.map(str::to_string);
        self
    }

    /// Attach structured details to an error.
    pub fn with_details(mut self, details: FulencodeErrorDetails) -> Self {
        self.details = Some(details);
        self
    }
}

impl std::fmt::Display for FulencodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} ({})", self.code, self.message, self.operation)
    }
}

impl std::error::Error for FulencodeError {}
