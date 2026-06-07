//! Archive verification: structural + security reporting (never rejects).
//!
//! `verify` inspects the raw table of contents and returns a [`ValidationResult`]
//! describing any path-traversal/absolute/symlink-escape entries and
//! decompression-bomb characteristics. Unlike `extract`, it does not error on
//! findings — it reports `valid = false`. Per the standard, actual checksum
//! verification is performed by `extract`, not here.

use super::error::FulpackError;
use super::format::detect_format;
use super::read::{corrupt, open};
use super::types::{ArchiveFormat, ValidationResult};
use std::io::{BufReader, Read};
use std::path::{Component, Path};

/// Decompression ratio above which an archive is flagged as a possible bomb.
const BOMB_RATIO_WARN: f64 = 100.0;

struct RawEntry {
    path: String,
    is_symlink: bool,
    link_target: Option<String>,
    size: u64,
}

/// Verify archive integrity and security properties.
///
/// # Errors
/// Returns [`FulpackError`] if the format is unrecognized or the archive cannot
/// be opened. A corrupt-but-openable archive is reported as `valid = false`.
pub fn verify(archive: &Path) -> Result<ValidationResult, FulpackError> {
    let format = detect_format(archive)?;
    let compressed_size =
        std::fs::metadata(archive)
            .map(|m| m.len())
            .map_err(|e| FulpackError::Io {
                path: archive.to_path_buf(),
                source: e,
            })?;

    let entries = match collect_raw(archive, format) {
        Ok(entries) => entries,
        Err(_) => {
            return Ok(ValidationResult {
                valid: false,
                errors: vec!["archive structure is invalid or corrupt".to_string()],
                warnings: Vec::new(),
                entry_count: 0,
                checksums_verified: Some(0),
                checks_performed: Some(vec!["structure_valid".to_string()]),
            });
        }
    };

    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut total_size: u64 = 0;

    for entry in &entries {
        total_size = total_size.saturating_add(entry.size);
        match classify(&entry.path) {
            PathClass::Absolute => {
                errors.push(format!("absolute path: {}", entry.path));
            }
            PathClass::Traversal => {
                errors.push(format!("path traversal: {}", entry.path));
            }
            PathClass::Safe => {}
        }
        if entry.is_symlink {
            if let Some(target) = &entry.link_target {
                if Path::new(target).is_absolute() || has_parent_component(target) {
                    errors.push(format!("symlink escape: {} -> {}", entry.path, target));
                }
            }
        }
    }

    if compressed_size > 0 {
        let ratio = total_size as f64 / compressed_size as f64;
        if ratio > BOMB_RATIO_WARN {
            warnings.push(format!(
                "high compression ratio ({ratio:.0}x) — possible decompression bomb"
            ));
        }
    }
    warnings.push("no embedded checksums to verify (use extract to verify content)".to_string());

    Ok(ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
        entry_count: entries.len() as u64,
        checksums_verified: Some(0),
        checks_performed: Some(vec![
            "structure_valid".to_string(),
            "no_path_traversal".to_string(),
            "no_decompression_bomb".to_string(),
            "symlinks_safe".to_string(),
        ]),
    })
}

enum PathClass {
    Safe,
    Absolute,
    Traversal,
}

fn classify(raw: &str) -> PathClass {
    for component in Path::new(raw).components() {
        match component {
            Component::RootDir | Component::Prefix(_) => return PathClass::Absolute,
            Component::ParentDir => return PathClass::Traversal,
            _ => {}
        }
    }
    PathClass::Safe
}

fn has_parent_component(raw: &str) -> bool {
    Path::new(raw)
        .components()
        .any(|c| matches!(c, Component::ParentDir))
}

fn collect_raw(archive: &Path, format: ArchiveFormat) -> Result<Vec<RawEntry>, FulpackError> {
    match format {
        ArchiveFormat::Tar => collect_tar(BufReader::new(open(archive)?), archive),
        ArchiveFormat::TarGz => {
            let dec = flate2::read::GzDecoder::new(BufReader::new(open(archive)?));
            collect_tar(dec, archive)
        }
        ArchiveFormat::Zip => collect_zip(open(archive)?, archive),
        ArchiveFormat::Gzip => collect_gzip(archive),
    }
}

fn collect_tar<R: Read>(reader: R, archive: &Path) -> Result<Vec<RawEntry>, FulpackError> {
    let mut ar = tar::Archive::new(reader);
    let mut out = Vec::new();
    for entry in ar.entries().map_err(|e| corrupt(archive, e))? {
        let entry = entry.map_err(|e| corrupt(archive, e))?;
        let header = entry.header();
        let is_symlink = header.entry_type().is_symlink();
        let link_target = if is_symlink {
            entry
                .link_name_bytes()
                .map(|b| String::from_utf8_lossy(&b).into_owned())
        } else {
            None
        };
        out.push(RawEntry {
            path: String::from_utf8_lossy(&entry.path_bytes()).into_owned(),
            is_symlink,
            link_target,
            size: header.size().unwrap_or(0),
        });
    }
    Ok(out)
}

fn collect_zip<R: Read + std::io::Seek>(
    reader: R,
    archive: &Path,
) -> Result<Vec<RawEntry>, FulpackError> {
    let mut zip = zip::ZipArchive::new(reader).map_err(|e| corrupt(archive, e))?;
    let mut out = Vec::with_capacity(zip.len());
    for i in 0..zip.len() {
        let file = zip.by_index(i).map_err(|e| corrupt(archive, e))?;
        out.push(RawEntry {
            path: file.name().to_string(),
            is_symlink: false,
            link_target: None,
            size: file.size(),
        });
    }
    Ok(out)
}

fn collect_gzip(archive: &Path) -> Result<Vec<RawEntry>, FulpackError> {
    let mut decoder = flate2::read::GzDecoder::new(BufReader::new(open(archive)?));
    let path = decoder
        .header()
        .and_then(|h| h.filename())
        .map(|b| String::from_utf8_lossy(b).into_owned())
        .unwrap_or_else(|| "output".to_string());
    let mut size: u64 = 0;
    let mut buf = [0u8; 8192];
    loop {
        let n = decoder.read(&mut buf).map_err(|e| corrupt(archive, e))?;
        if n == 0 {
            break;
        }
        size += n as u64;
    }
    Ok(vec![RawEntry {
        path,
        is_symlink: false,
        link_target: None,
        size,
    }])
}
