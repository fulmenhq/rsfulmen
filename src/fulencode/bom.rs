use crate::fulencode::types::{BomResult, FulencodeError};

pub(crate) const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];
pub(crate) const UTF16LE_BOM: &[u8] = &[0xFF, 0xFE];
pub(crate) const UTF16BE_BOM: &[u8] = &[0xFE, 0xFF];
pub(crate) const UTF32LE_BOM: &[u8] = &[0xFF, 0xFE, 0x00, 0x00];
pub(crate) const UTF32BE_BOM: &[u8] = &[0x00, 0x00, 0xFE, 0xFF];

pub(crate) fn detect_bom_impl(data: &[u8]) -> BomResult {
    if data.starts_with(UTF32LE_BOM) {
        return BomResult {
            bom_type: Some("utf-32le".to_string()),
            byte_length: 4,
            encoding_implied: Some("utf-32le".to_string()),
        };
    }

    if data.starts_with(UTF32BE_BOM) {
        return BomResult {
            bom_type: Some("utf-32be".to_string()),
            byte_length: 4,
            encoding_implied: Some("utf-32be".to_string()),
        };
    }

    if data.starts_with(UTF8_BOM) {
        return BomResult {
            bom_type: Some("utf-8".to_string()),
            byte_length: 3,
            encoding_implied: Some("utf-8".to_string()),
        };
    }

    if data.starts_with(UTF16LE_BOM) {
        return BomResult {
            bom_type: Some("utf-16le".to_string()),
            byte_length: 2,
            encoding_implied: Some("utf-16le".to_string()),
        };
    }

    if data.starts_with(UTF16BE_BOM) {
        return BomResult {
            bom_type: Some("utf-16be".to_string()),
            byte_length: 2,
            encoding_implied: Some("utf-16be".to_string()),
        };
    }

    BomResult {
        bom_type: None,
        byte_length: 0,
        encoding_implied: None,
    }
}

pub(crate) fn remove_bom_impl(
    data: &[u8],
    expected_encoding: Option<&str>,
) -> Result<Vec<u8>, FulencodeError> {
    let bom = detect_bom_impl(data);

    if let Some(expected) = expected_encoding {
        if let Some(detected) = bom.encoding_implied.as_deref() {
            if detected != expected {
                return Err(FulencodeError::new(
                    "BOM_MISMATCH",
                    "detected BOM does not match expected encoding",
                    "remove_bom",
                )
                .with_details(crate::fulencode::types::FulencodeErrorDetails {
                    expected: Some(expected.to_string()),
                    actual: Some(detected.to_string()),
                    ..Default::default()
                }));
            }
        }
    }

    if bom.byte_length == 0 {
        Ok(data.to_vec())
    } else {
        Ok(data[bom.byte_length..].to_vec())
    }
}

pub(crate) fn add_bom_impl(data: &[u8], encoding: &str) -> Result<Vec<u8>, FulencodeError> {
    let bom = match encoding {
        "utf-8" => UTF8_BOM,
        "utf-16le" => UTF16LE_BOM,
        "utf-16be" => UTF16BE_BOM,
        "utf-32le" => UTF32LE_BOM,
        "utf-32be" => UTF32BE_BOM,
        _ => {
            return Err(FulencodeError::new(
                "UNSUPPORTED_FORMAT",
                "BOM is not defined for the requested encoding",
                "add_bom",
            ))
        }
    };

    let mut out = Vec::with_capacity(bom.len() + data.len());
    out.extend_from_slice(bom);
    out.extend_from_slice(data);
    Ok(out)
}
