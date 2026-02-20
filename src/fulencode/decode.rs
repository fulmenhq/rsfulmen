use base64::engine::general_purpose;
use base64::Engine;

use crate::fulencode::types::{DecodeOptions, DecodingResult, EncodingFormat, FulencodeError};

const DEFAULT_MAX_DECODED_SIZE: usize = 100 * 1024 * 1024;
const DEFAULT_MAX_EXPANSION_RATIO: f64 = 10.0;

pub(crate) fn decode_impl(
    data: &str,
    format: EncodingFormat,
    options: Option<&DecodeOptions>,
) -> Result<DecodingResult, FulencodeError> {
    let opts = options.cloned().unwrap_or_default();

    let ignore_whitespace = opts
        .ignore_whitespace
        .unwrap_or(default_ignore_whitespace(format));

    let normalized = if ignore_whitespace {
        data.chars().filter(|c| !c.is_ascii_whitespace()).collect()
    } else {
        data.to_string()
    };

    let decoded = match format {
        EncodingFormat::Base64 => decode_base64(&normalized, true, &opts)?,
        EncodingFormat::Base64url => decode_base64(&normalized, false, &opts)?,
        EncodingFormat::Base64Raw => decode_base64_raw(&normalized)?,
        EncodingFormat::Hex => hex::decode(normalized.as_bytes()).map_err(|_| {
            FulencodeError::new("INVALID_ENCODING", "invalid hex input", "decode")
                .with_formats(Some(format.as_str()), Some("binary"))
        })?,
        EncodingFormat::Utf8 => normalized.as_bytes().to_vec(),
        EncodingFormat::Utf16le => encode_utf16_bytes(&normalized, true),
        EncodingFormat::Utf16be => encode_utf16_bytes(&normalized, false),
        EncodingFormat::Ascii => {
            if !normalized.is_ascii() {
                return Err(FulencodeError::new(
                    "INVALID_ENCODING",
                    "non-ASCII character in input",
                    "decode",
                )
                .with_formats(Some(format.as_str()), Some("binary")));
            }
            normalized.as_bytes().to_vec()
        }
        EncodingFormat::Base32
        | EncodingFormat::Base32hex
        | EncodingFormat::Iso88591
        | EncodingFormat::Cp1252 => {
            return Err(FulencodeError::new(
                "UNSUPPORTED_FORMAT",
                "format is defined in schema but not implemented in this phase",
                "decode",
            )
            .with_formats(Some(format.as_str()), Some("binary")))
        }
    };

    let output_size = decoded.len();
    let max_decoded_size = opts.max_decoded_size.unwrap_or(DEFAULT_MAX_DECODED_SIZE);
    if output_size > max_decoded_size {
        return Err(FulencodeError::new(
            "BUFFER_OVERFLOW",
            "decoded payload exceeds configured maximum",
            "decode",
        )
        .with_formats(Some(format.as_str()), Some("binary")));
    }

    if !normalized.is_empty() {
        let ratio = output_size as f64 / normalized.len() as f64;
        let max_ratio = opts
            .max_expansion_ratio
            .unwrap_or(DEFAULT_MAX_EXPANSION_RATIO);
        if ratio > max_ratio {
            return Err(FulencodeError::new(
                "ENCODING_BOMB",
                "decode expansion ratio exceeds configured maximum",
                "decode",
            )
            .with_formats(Some(format.as_str()), Some("binary")));
        }
    }

    Ok(DecodingResult {
        data: decoded,
        format,
        input_size: normalized.len(),
        output_size,
        warnings: Vec::new(),
        corrections_applied: 0,
        checksum: None,
        checksum_verified: None,
        checksum_algorithm: None,
    })
}

fn decode_base64(
    input: &str,
    standard: bool,
    opts: &DecodeOptions,
) -> Result<Vec<u8>, FulencodeError> {
    if opts.validate_padding.unwrap_or(true) && !has_valid_padding(input) {
        return Err(FulencodeError::new(
            "INVALID_ENCODING",
            "invalid base64 padding",
            "decode",
        ));
    }

    // Strict is the canonical default. Non-strict modes may attempt lenient
    // recovery (e.g., missing padding) for compatibility.
    let strict = !matches!(
        opts.on_error.as_deref(),
        Some("replace") | Some("ignore") | Some("fallback")
    );
    let decoded = if standard {
        if strict {
            general_purpose::STANDARD.decode(input.as_bytes())
        } else {
            match general_purpose::STANDARD.decode(input.as_bytes()) {
                Ok(bytes) => Ok(bytes),
                Err(_) => decode_base64_lenient(input, true),
            }
        }
    } else if strict {
        if input.contains('=') {
            general_purpose::URL_SAFE.decode(input.as_bytes())
        } else {
            general_purpose::URL_SAFE_NO_PAD.decode(input.as_bytes())
        }
    } else {
        let primary = if input.contains('=') {
            general_purpose::URL_SAFE.decode(input.as_bytes())
        } else {
            general_purpose::URL_SAFE_NO_PAD.decode(input.as_bytes())
        };
        match primary {
            Ok(bytes) => Ok(bytes),
            Err(_) => decode_base64_lenient(input, false),
        }
    };

    decoded.map_err(|_| {
        FulencodeError::new("INVALID_ENCODING", "invalid base64 input", "decode").with_formats(
            Some(if standard { "base64" } else { "base64url" }),
            Some("binary"),
        )
    })
}

fn decode_base64_raw(input: &str) -> Result<Vec<u8>, FulencodeError> {
    let remainder = input.len() % 4;
    let padded = if remainder == 0 {
        input.to_string()
    } else {
        let mut out = input.to_string();
        out.extend(std::iter::repeat_n('=', 4 - remainder));
        out
    };

    general_purpose::STANDARD
        .decode(padded.as_bytes())
        .map_err(|_| FulencodeError::new("INVALID_ENCODING", "invalid raw base64 input", "decode"))
}

fn decode_base64_lenient(input: &str, standard: bool) -> Result<Vec<u8>, base64::DecodeError> {
    let remainder = input.len() % 4;
    let padded = if remainder == 0 {
        input.to_string()
    } else {
        let mut out = input.to_string();
        out.extend(std::iter::repeat_n('=', 4 - remainder));
        out
    };

    if standard {
        general_purpose::STANDARD.decode(padded.as_bytes())
    } else {
        general_purpose::URL_SAFE.decode(padded.as_bytes())
    }
}

// Strict padding rules: '=' only at end and allow 1 or 2 '=' when present.
fn has_valid_padding(input: &str) -> bool {
    let Some(first_eq) = input.find('=') else {
        return true;
    };

    if input[first_eq..].bytes().any(|b| b != b'=') {
        return false;
    }

    let pad_count = input.len() - first_eq;
    if !(pad_count == 1 || pad_count == 2) {
        return false;
    }

    input.len().is_multiple_of(4)
}

fn encode_utf16_bytes(input: &str, little_endian: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len() * 2);
    for unit in input.encode_utf16() {
        let bytes = if little_endian {
            unit.to_le_bytes()
        } else {
            unit.to_be_bytes()
        };
        out.extend_from_slice(&bytes);
    }
    out
}

fn default_ignore_whitespace(format: EncodingFormat) -> bool {
    matches!(
        format,
        EncodingFormat::Base64
            | EncodingFormat::Base64url
            | EncodingFormat::Base64Raw
            | EncodingFormat::Hex
    )
}
