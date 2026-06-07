//! Read-only operations: [`info`] and [`scan`].
//!
//! Both read the archive table of contents without extracting. Per the Crucible
//! standard, `scan` is *discovery* and is deliberately lenient — it lists every
//! entry (including traversal/absolute/symlink paths) with absolute paths
//! normalized to relative and invalid UTF-8 replaced with U+FFFD. Security
//! *enforcement* (rejecting those paths) belongs to extract/verify.

use super::error::FulpackError;
use super::format::detect_format;
use super::types::{ArchiveEntry, ArchiveFormat, ArchiveInfo, Compression, EntryType, ScanOptions};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Get archive metadata without listing entries to the caller.
///
/// # Errors
/// Returns [`FulpackError`] if the format is unrecognized or the archive cannot be read.
pub fn info(archive: &Path) -> Result<ArchiveInfo, FulpackError> {
    let format = detect_format(archive)?;
    let compressed_size = file_len(archive)?;
    let entries = read_entries(archive, format, &ScanOptions::default())?;

    let total_size: u64 = entries.iter().map(|e| e.size).sum();
    let entry_count = entries.len() as u64;

    let (compression, compression_ratio) = match format {
        // Uncompressed tar: framing overhead is not compression, ratio is 1.0.
        ArchiveFormat::Tar => (Compression::None, Some(1.0)),
        ArchiveFormat::Zip => (Compression::Deflate, ratio(total_size, compressed_size)),
        ArchiveFormat::TarGz | ArchiveFormat::Gzip => {
            (Compression::Gzip, ratio(total_size, compressed_size))
        }
    };

    Ok(ArchiveInfo {
        format,
        compression: Some(compression),
        entry_count,
        total_size,
        compressed_size,
        compression_ratio,
        has_checksums: Some(false),
        checksum_algorithm: None,
        created: None,
        checksums: None,
    })
}

/// List archive entries (table of contents) without extracting.
///
/// # Errors
/// Returns [`FulpackError`] if the format is unrecognized, the archive cannot be
/// read, or the raw entry count exceeds [`ScanOptions::max_entries`].
pub fn scan(
    archive: &Path,
    options: Option<&ScanOptions>,
) -> Result<Vec<ArchiveEntry>, FulpackError> {
    let default = ScanOptions::default();
    let opts = options.unwrap_or(&default);
    let format = detect_format(archive)?;

    let mut entries = read_entries(archive, format, opts)?;

    if let Some(types) = &opts.entry_types {
        entries.retain(|e| types.contains(&e.entry_type));
    }
    if let Some(max_depth) = opts.max_depth {
        entries.retain(|e| path_depth(&e.path) <= max_depth);
    }
    Ok(entries)
}

// ---------------------------------------------------------------------------
// Internal: per-format TOC readers
// ---------------------------------------------------------------------------

fn read_entries(
    archive: &Path,
    format: ArchiveFormat,
    opts: &ScanOptions,
) -> Result<Vec<ArchiveEntry>, FulpackError> {
    let file = open(archive)?;
    match format {
        ArchiveFormat::Tar => read_tar(BufReader::new(file), archive, opts),
        ArchiveFormat::TarGz => {
            let decoder = flate2::read::GzDecoder::new(BufReader::new(file));
            read_tar(decoder, archive, opts)
        }
        ArchiveFormat::Gzip => read_gzip(file, archive),
        ArchiveFormat::Zip => read_zip(file, archive, opts),
    }
}

fn read_tar<R: Read>(
    reader: R,
    archive: &Path,
    opts: &ScanOptions,
) -> Result<Vec<ArchiveEntry>, FulpackError> {
    let mut ar = tar::Archive::new(reader);
    let mut out = Vec::new();
    let iter = ar.entries().map_err(|e| corrupt(archive, e))?;
    for entry in iter {
        let entry = entry.map_err(|e| corrupt(archive, e))?;
        check_cap(out.len() as u64, opts.max_entries)?;

        let header = entry.header();
        let kind = header.entry_type();
        let entry_type = if kind.is_dir() {
            EntryType::Directory
        } else if kind.is_symlink() {
            EntryType::Symlink
        } else {
            EntryType::File
        };

        let path = normalize_scan_path(&lossy(&entry.path_bytes()));
        let size = header.size().unwrap_or(0);
        let (modified, mode) = if opts.include_metadata {
            let modified = header.mtime().ok().map(epoch_secs_to_rfc3339);
            let mode = header.mode().ok().map(format_mode);
            (modified, mode)
        } else {
            (None, None)
        };
        let symlink_target = if entry_type == EntryType::Symlink {
            entry.link_name_bytes().map(|b| lossy(&b))
        } else {
            None
        };

        out.push(ArchiveEntry {
            path,
            entry_type,
            size,
            compressed_size: None,
            modified,
            checksum: None,
            mode,
            symlink_target,
        });
    }
    Ok(out)
}

fn read_zip<R: Read + std::io::Seek>(
    reader: R,
    archive: &Path,
    opts: &ScanOptions,
) -> Result<Vec<ArchiveEntry>, FulpackError> {
    let mut zip = zip::ZipArchive::new(reader).map_err(|e| corrupt(archive, e))?;
    let mut out = Vec::with_capacity(zip.len());
    for i in 0..zip.len() {
        check_cap(out.len() as u64, opts.max_entries)?;
        let file = zip.by_index(i).map_err(|e| corrupt(archive, e))?;

        let unix_mode = file.unix_mode();
        // The fulpack archive-formats taxonomy declares `zip: supports_symlinks: false`,
        // so ZIP entries are classified as file/directory only (cross-language parity).
        let entry_type = if file.is_dir() {
            EntryType::Directory
        } else {
            EntryType::File
        };

        let path = normalize_scan_path(file.name());
        let size = file.size();
        let compressed_size = Some(file.compressed_size());
        let (modified, mode) = if opts.include_metadata {
            (
                file.last_modified().and_then(zip_datetime_rfc3339),
                unix_mode.map(format_mode),
            )
        } else {
            (None, None)
        };

        out.push(ArchiveEntry {
            path,
            entry_type,
            size,
            compressed_size,
            modified,
            checksum: None,
            mode,
            symlink_target: None,
        });
    }
    Ok(out)
}

fn read_gzip(file: File, archive: &Path) -> Result<Vec<ArchiveEntry>, FulpackError> {
    let mut decoder = flate2::read::GzDecoder::new(BufReader::new(file));

    // gzip stores an optional original filename in its header; fall back to the
    // archive name with the `.gz`/`.gzip` suffix stripped.
    let path = decoder
        .header()
        .and_then(|h| h.filename())
        .map(|b| normalize_scan_path(&String::from_utf8_lossy(b)))
        .unwrap_or_else(|| strip_gzip_suffix(archive));

    // A single-file gzip has no TOC; the uncompressed size requires inflating.
    // Stream-count with a fixed buffer (bounded memory) and a hard cap so a
    // read-only `info`/`scan` can't be turned into a decompression bomb.
    let size = count_inflated(&mut decoder, archive, GZIP_MAX_INFLATE_BYTES)?;

    Ok(vec![ArchiveEntry {
        path,
        entry_type: EntryType::File,
        size,
        compressed_size: None,
        modified: None,
        checksum: None,
        mode: None,
        symlink_target: None,
    }])
}

/// Read-path inflation guard for single-file gzip: 10 GiB matches the standard's
/// max-total-size decompression-bomb limit. Inspecting metadata must not exceed it.
const GZIP_MAX_INFLATE_BYTES: u64 = 10 * 1024 * 1024 * 1024;

/// Count the bytes produced by `reader` using a fixed-size buffer (constant
/// memory), failing with [`FulpackError::DecompressionBomb`] if the running total
/// exceeds `cap`. Bytes are discarded, not retained.
fn count_inflated<R: Read>(reader: &mut R, archive: &Path, cap: u64) -> Result<u64, FulpackError> {
    let mut buf = [0u8; 8192];
    let mut total: u64 = 0;
    loop {
        let n = reader.read(&mut buf).map_err(|e| corrupt(archive, e))?;
        if n == 0 {
            break;
        }
        total += n as u64;
        if total > cap {
            return Err(FulpackError::DecompressionBomb {
                message: format!("gzip inflates beyond the read-path limit of {cap} bytes"),
            });
        }
    }
    Ok(total)
}

// ---------------------------------------------------------------------------
// Internal: helpers
// ---------------------------------------------------------------------------

pub(super) fn open(archive: &Path) -> Result<File, FulpackError> {
    File::open(archive).map_err(|source| {
        if source.kind() == std::io::ErrorKind::NotFound {
            FulpackError::NotFound {
                path: archive.to_path_buf(),
            }
        } else {
            FulpackError::Io {
                path: archive.to_path_buf(),
                source,
            }
        }
    })
}

fn file_len(archive: &Path) -> Result<u64, FulpackError> {
    let meta = std::fs::metadata(archive).map_err(|source| {
        if source.kind() == std::io::ErrorKind::NotFound {
            FulpackError::NotFound {
                path: archive.to_path_buf(),
            }
        } else {
            FulpackError::Io {
                path: archive.to_path_buf(),
                source,
            }
        }
    })?;
    Ok(meta.len())
}

pub(super) fn corrupt<E: std::fmt::Display>(archive: &Path, err: E) -> FulpackError {
    FulpackError::Corrupt {
        path: archive.to_path_buf(),
        message: err.to_string(),
    }
}

fn check_cap(count: u64, max_entries: u64) -> Result<(), FulpackError> {
    if count >= max_entries {
        Err(FulpackError::TooManyEntries { limit: max_entries })
    } else {
        Ok(())
    }
}

fn ratio(total: u64, compressed: u64) -> Option<f64> {
    if compressed == 0 {
        None
    } else {
        Some(total as f64 / compressed as f64)
    }
}

fn lossy(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// Normalize a TOC entry path for `scan` discovery: convert backslashes, strip a
/// leading drive letter, drop `.` and empty (leading/trailing/duplicate-slash)
/// segments so absolute paths become relative. Traversal (`..`) segments are
/// intentionally preserved — `scan` is discovery; extract/verify enforce security.
fn normalize_scan_path(raw: &str) -> String {
    let mut s = raw.replace('\\', "/");
    if s.len() >= 2 && s.as_bytes()[1] == b':' && s.as_bytes()[0].is_ascii_alphabetic() {
        s = s[2..].to_string();
    }
    let cleaned: Vec<&str> = s
        .split('/')
        .filter(|seg| !seg.is_empty() && *seg != ".")
        .collect();
    let joined = cleaned.join("/");
    if joined.is_empty() {
        // Schema requires a non-empty path; the archive root collapses to ".".
        ".".to_string()
    } else {
        joined
    }
}

fn strip_gzip_suffix(archive: &Path) -> String {
    let name = archive
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    let lower = name.to_ascii_lowercase();
    let stem = if lower.ends_with(".gz") {
        &name[..name.len() - 3]
    } else if lower.ends_with(".gzip") {
        &name[..name.len() - 5]
    } else {
        name
    };
    stem.to_string()
}

fn path_depth(path: &str) -> u32 {
    path.trim_end_matches('/').matches('/').count() as u32
}

fn format_mode(mode: u32) -> String {
    format!("{:04o}", mode & 0o7777)
}

/// Convert a `zip::DateTime` to an RFC 3339 string (assumed UTC).
fn zip_datetime_rfc3339(dt: zip::DateTime) -> Option<String> {
    Some(format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        dt.year(),
        dt.month(),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second(),
    ))
}

/// Convert seconds since the Unix epoch to an RFC 3339 (UTC) string.
///
/// Civil-calendar algorithm adapted from Howard Hinnant's `civil_from_days`
/// (same approach used by `pathfinder`), avoiding a datetime crate dependency.
pub(super) fn epoch_secs_to_rfc3339(secs: u64) -> String {
    let days = (secs / 86400) as i64;
    let time_of_day = secs % 86400;

    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    let hour = time_of_day / 3600;
    let minute = (time_of_day % 3600) / 60;
    let second = time_of_day % 60;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, m, d, hour, minute, second
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn count_inflated_trips_on_cap() {
        let err =
            count_inflated(&mut Cursor::new(vec![0u8; 5000]), Path::new("x.gz"), 1000).unwrap_err();
        assert_eq!(err.code(), "DECOMPRESSION_BOMB");
    }

    #[test]
    fn count_inflated_counts_under_cap() {
        let n = count_inflated(&mut Cursor::new(vec![7u8; 500]), Path::new("x.gz"), 1000).unwrap();
        assert_eq!(n, 500);
    }

    #[test]
    fn normalize_scan_path_strips_absolute_keeps_traversal() {
        assert_eq!(normalize_scan_path("/etc/passwd"), "etc/passwd");
        assert_eq!(normalize_scan_path("./a/./b"), "a/b");
        assert_eq!(normalize_scan_path("../../etc/passwd"), "../../etc/passwd");
        assert_eq!(normalize_scan_path("C:\\Users\\x"), "Users/x");
        assert_eq!(normalize_scan_path("/"), ".");
    }
}
