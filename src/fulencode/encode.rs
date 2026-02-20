use base64::engine::general_purpose;
use base64::Engine;

use crate::fulencode::types::{EncodeOptions, EncodingFormat, EncodingResult, FulencodeError};

const DEFAULT_MAX_ENCODED_SIZE: usize = 500 * 1024 * 1024;

pub(crate) fn encode_impl(
    data: &[u8],
    format: EncodingFormat,
    options: Option<&EncodeOptions>,
) -> Result<EncodingResult, FulencodeError> {
    let opts = options.cloned().unwrap_or_default();

    let mut encoded = match format {
        EncodingFormat::Base64 => {
            let use_padding = opts.padding.unwrap_or(true);
            if use_padding {
                general_purpose::STANDARD.encode(data)
            } else {
                general_purpose::STANDARD_NO_PAD.encode(data)
            }
        }
        EncodingFormat::Base64url => {
            let use_padding = opts.padding.unwrap_or(false);
            if use_padding {
                general_purpose::URL_SAFE.encode(data)
            } else {
                general_purpose::URL_SAFE_NO_PAD.encode(data)
            }
        }
        EncodingFormat::Base64Raw => general_purpose::STANDARD_NO_PAD.encode(data),
        EncodingFormat::Hex => {
            if opts.case.as_deref() == Some("upper") {
                hex::encode_upper(data)
            } else {
                hex::encode(data)
            }
        }
        EncodingFormat::Utf8 => String::from_utf8(data.to_vec()).map_err(|err| {
            FulencodeError::new("INVALID_UTF8", "input bytes are not valid UTF-8", "encode")
                .with_formats(Some("binary"), Some(format.as_str()))
                .with_details(crate::fulencode::types::FulencodeErrorDetails {
                    invalid_bytes: Some(err.into_bytes()),
                    ..Default::default()
                })
        })?,
        EncodingFormat::Utf16le => decode_utf16_as_string(data, true)?,
        EncodingFormat::Utf16be => decode_utf16_as_string(data, false)?,
        EncodingFormat::Ascii => {
            if data.iter().any(|b| *b > 0x7F) {
                return Err(FulencodeError::new(
                    "INVALID_ENCODING",
                    "input contains non-ASCII bytes",
                    "encode",
                )
                .with_formats(Some("binary"), Some(format.as_str())));
            }
            String::from_utf8(data.to_vec()).map_err(|_| {
                FulencodeError::new("INVALID_ENCODING", "invalid ASCII payload", "encode")
                    .with_formats(Some("binary"), Some(format.as_str()))
            })?
        }
        EncodingFormat::Base32
        | EncodingFormat::Base32hex
        | EncodingFormat::Iso88591
        | EncodingFormat::Cp1252 => {
            return Err(FulencodeError::new(
                "UNSUPPORTED_FORMAT",
                "format is defined in schema but not implemented in this phase",
                "encode",
            )
            .with_formats(Some("binary"), Some(format.as_str())))
        }
    };

    if let Some(line_len) = opts.line_length {
        if line_len == 0 {
            return Err(FulencodeError::new(
                "INVALID_OPTIONS",
                "line_length must be > 0",
                "encode",
            ));
        }
        encoded = wrap_lines(
            &encoded,
            line_len,
            opts.line_ending.as_deref().unwrap_or("\n"),
        );
    }

    let output_size = encoded.len();
    let max_encoded_size = opts.max_encoded_size.unwrap_or(DEFAULT_MAX_ENCODED_SIZE);
    if output_size > max_encoded_size {
        return Err(FulencodeError::new(
            "BUFFER_OVERFLOW",
            "encoded payload exceeds configured maximum",
            "encode",
        )
        .with_formats(Some("binary"), Some(format.as_str())));
    }

    Ok(EncodingResult {
        data: encoded,
        format,
        input_size: data.len(),
        output_size,
        warnings: Vec::new(),
        checksum: None,
        checksum_algorithm: None,
    })
}

fn decode_utf16_as_string(data: &[u8], little_endian: bool) -> Result<String, FulencodeError> {
    if !data.len().is_multiple_of(2) {
        return Err(FulencodeError::new(
            "INVALID_UTF16",
            "odd-length UTF-16 byte sequence",
            "encode",
        ));
    }

    let mut units = Vec::with_capacity(data.len() / 2);
    for chunk in data.chunks_exact(2) {
        let unit = if little_endian {
            u16::from_le_bytes([chunk[0], chunk[1]])
        } else {
            u16::from_be_bytes([chunk[0], chunk[1]])
        };
        units.push(unit);
    }

    String::from_utf16(&units).map_err(|_| {
        FulencodeError::new(
            "INVALID_UTF16",
            "invalid UTF-16 code unit sequence",
            "encode",
        )
    })
}

fn wrap_lines(input: &str, line_length: usize, ending: &str) -> String {
    if input.len() <= line_length {
        return input.to_string();
    }

    let mut out = String::with_capacity(input.len() + (input.len() / line_length) * ending.len());
    let mut start = 0usize;
    while start < input.len() {
        let end = (start + line_length).min(input.len());
        out.push_str(&input[start..end]);
        if end < input.len() {
            out.push_str(ending);
        }
        start = end;
    }

    out
}
