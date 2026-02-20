use crate::fulencode::bom::detect_bom_impl;
use crate::fulencode::types::{ConfidenceLevel, DetectOptions, DetectionResult, FulencodeError};

const DEFAULT_MAX_SAMPLE_SIZE: usize = 8192;

pub(crate) fn detect_impl(
    data: &[u8],
    options: Option<&DetectOptions>,
) -> Result<DetectionResult, FulencodeError> {
    let opts = options.cloned().unwrap_or_default();
    let sample_limit = opts
        .max_sample_size
        .unwrap_or(DEFAULT_MAX_SAMPLE_SIZE)
        .max(1);
    let sample = &data[..data.len().min(sample_limit)];

    let bom = detect_bom_impl(sample);
    let mut result = if let Some(encoding) = bom.encoding_implied {
        DetectionResult {
            encoding: Some(encoding),
            confidence: 1.0,
            level: ConfidenceLevel::High,
            multibase_prefix: None,
            warnings: Vec::new(),
        }
    } else if sample.is_empty() {
        DetectionResult {
            encoding: Some("utf-8".to_string()),
            confidence: 0.4,
            level: ConfidenceLevel::Low,
            multibase_prefix: None,
            warnings: vec!["empty input; defaulting to utf-8".to_string()],
        }
    } else if sample.iter().all(u8::is_ascii) {
        DetectionResult {
            encoding: Some("utf-8".to_string()),
            confidence: 0.4,
            level: ConfidenceLevel::Low,
            multibase_prefix: None,
            warnings: vec!["ASCII-only payload is ambiguous across UTF-8 supersets".to_string()],
        }
    } else if std::str::from_utf8(sample).is_ok() {
        DetectionResult {
            encoding: Some("utf-8".to_string()),
            confidence: 0.85,
            level: ConfidenceLevel::Medium,
            multibase_prefix: None,
            warnings: vec!["valid UTF-8 with no BOM".to_string()],
        }
    } else {
        DetectionResult {
            encoding: None,
            confidence: 0.0,
            level: ConfidenceLevel::Low,
            multibase_prefix: None,
            warnings: vec!["unable to infer encoding from sample".to_string()],
        }
    };

    if let Some(min_confidence) = opts.min_confidence {
        if result.confidence < min_confidence {
            return Err(FulencodeError::new(
                "DETECTION_FAILED",
                "detected confidence below minimum threshold",
                "detect",
            ));
        }
    }

    result.level = confidence_level(result.confidence);
    Ok(result)
}

fn confidence_level(score: f64) -> ConfidenceLevel {
    if score >= 0.9 {
        ConfidenceLevel::High
    } else if score >= 0.5 {
        ConfidenceLevel::Medium
    } else {
        ConfidenceLevel::Low
    }
}
