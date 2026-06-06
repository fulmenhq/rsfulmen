//! Archive extraction with the mandatory fulpack security model.
//!
//! Unlike `scan` (lenient discovery), `extract` is *enforcing*: it rejects
//! traversal/absolute entry paths, symlinks that escape the destination, and
//! archives that exceed the decompression-bomb limits — failing before the
//! offending entry is written.

use super::error::FulpackError;
use super::format::detect_format;
use super::read::{corrupt, open};
use super::types::{ArchiveFormat, ExtractOptions, ExtractResult, OverwriteMode};
use std::fs;
use std::io::{BufReader, Read};
use std::path::{Component, Path, PathBuf};

/// Decompression-bomb defaults (extract-options schema).
const DEFAULT_MAX_SIZE: u64 = 1024 * 1024 * 1024; // 1 GiB total extracted bytes
const DEFAULT_MAX_ENTRIES: u64 = 10_000;
const COPY_BUF: usize = 8192;

/// Extract `archive` into `destination` (which is created if absent).
///
/// Security (always enforced): rejects `..`/absolute entry paths
/// ([`FulpackError::PathTraversal`] / [`FulpackError::AbsolutePath`]), symlinks
/// whose target escapes `destination` ([`FulpackError::SymlinkEscape`]), and
/// archives exceeding `max_size`/`max_entries` ([`FulpackError::DecompressionBomb`]).
///
/// # Errors
/// Returns [`FulpackError`] on unrecognized format, I/O failure, or any security
/// violation (the offending entry is never written).
pub fn extract(
    archive: &Path,
    destination: &Path,
    options: Option<&ExtractOptions>,
) -> Result<ExtractResult, FulpackError> {
    let owned;
    let opts = match options {
        Some(o) => o,
        None => {
            owned = ExtractOptions::default();
            &owned
        }
    };
    let format = detect_format(archive)?;
    fs::create_dir_all(destination).map_err(|source| FulpackError::Io {
        path: destination.to_path_buf(),
        source,
    })?;

    let mut ctx = ExtractCtx::new(destination, opts);
    match format {
        ArchiveFormat::Tar => extract_tar(BufReader::new(open(archive)?), archive, &mut ctx),
        ArchiveFormat::TarGz => {
            let dec = flate2::read::GzDecoder::new(BufReader::new(open(archive)?));
            extract_tar(dec, archive, &mut ctx)
        }
        ArchiveFormat::Zip => extract_zip(open(archive)?, archive, &mut ctx),
        ArchiveFormat::Gzip => extract_gzip(archive, &mut ctx),
    }?;
    Ok(ctx.into_result())
}

struct ExtractCtx<'a> {
    dest: &'a Path,
    overwrite: OverwriteMode,
    max_size: u64,
    max_entries: u64,
    includes: Option<Vec<glob::Pattern>>,
    total_bytes: u64,
    extracted: u64,
    skipped: u64,
    warnings: Vec<String>,
}

impl<'a> ExtractCtx<'a> {
    fn new(dest: &'a Path, opts: &ExtractOptions) -> Self {
        let includes = opts.include_patterns.as_ref().map(|pats| {
            pats.iter()
                .filter_map(|p| glob::Pattern::new(p).ok())
                .collect()
        });
        Self {
            dest,
            overwrite: opts.overwrite.unwrap_or(OverwriteMode::Error),
            max_size: opts.max_size.unwrap_or(DEFAULT_MAX_SIZE),
            max_entries: opts.max_entries.unwrap_or(DEFAULT_MAX_ENTRIES),
            includes,
            total_bytes: 0,
            extracted: 0,
            skipped: 0,
            warnings: Vec::new(),
        }
    }

    fn included(&self, rel: &str) -> bool {
        match &self.includes {
            None => true,
            Some(pats) => pats.iter().any(|p| p.matches(rel)),
        }
    }

    /// Account for `entry_index` against the max-entries bomb guard.
    fn count_entry(&self, entry_index: u64) -> Result<(), FulpackError> {
        if entry_index >= self.max_entries {
            return Err(FulpackError::DecompressionBomb {
                message: format!("archive exceeds max entries ({})", self.max_entries),
            });
        }
        Ok(())
    }

    /// Account for `bytes` against the max-total-size bomb guard.
    fn add_bytes(&mut self, bytes: u64) -> Result<(), FulpackError> {
        self.total_bytes = self.total_bytes.saturating_add(bytes);
        if self.total_bytes > self.max_size {
            return Err(FulpackError::DecompressionBomb {
                message: format!("archive exceeds max size ({} bytes)", self.max_size),
            });
        }
        Ok(())
    }

    fn into_result(self) -> ExtractResult {
        ExtractResult {
            extracted_count: self.extracted,
            skipped_count: self.skipped,
            error_count: 0,
            errors: None,
            warnings: if self.warnings.is_empty() {
                None
            } else {
                Some(self.warnings)
            },
            checksums_verified: None,
            total_bytes: Some(self.total_bytes),
        }
    }
}

// ---------------------------------------------------------------------------
// Security: resolve an entry path safely under the destination.
// ---------------------------------------------------------------------------

/// Resolve `entry_path` (as stored in the archive) to a path under `dest`,
/// rejecting absolute paths and `..` traversal. Returns the joined path.
fn safe_join(dest: &Path, entry_path: &str) -> Result<PathBuf, FulpackError> {
    let candidate = Path::new(entry_path);
    let mut out = dest.to_path_buf();
    for component in candidate.components() {
        match component {
            Component::Normal(seg) => out.push(seg),
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(FulpackError::PathTraversal {
                    path: entry_path.to_string(),
                })
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(FulpackError::AbsolutePath {
                    path: entry_path.to_string(),
                })
            }
        }
    }
    Ok(out)
}

/// True if a symlink stored at `entry_rel` pointing at `target` would resolve
/// outside the destination root (absolute target, or `..` escaping the root).
fn symlink_escapes(entry_rel: &str, target: &str) -> bool {
    if Path::new(target).is_absolute() {
        return true;
    }
    // Lexically resolve `<dir of entry>/<target>` and ensure depth never drops
    // below the destination root.
    let mut depth: i64 = 0;
    let parent = Path::new(entry_rel).parent().unwrap_or(Path::new(""));
    for component in parent.components().chain(Path::new(target).components()) {
        match component {
            Component::Normal(_) => depth += 1,
            Component::ParentDir => {
                depth -= 1;
                if depth < 0 {
                    return true;
                }
            }
            Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) => return true,
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Per-format extraction
// ---------------------------------------------------------------------------

fn extract_tar<R: Read>(
    reader: R,
    archive: &Path,
    ctx: &mut ExtractCtx<'_>,
) -> Result<(), FulpackError> {
    let mut ar = tar::Archive::new(reader);
    let entries = ar.entries().map_err(|e| corrupt(archive, e))?;
    for (index, entry) in entries.enumerate() {
        ctx.count_entry(index as u64)?;
        let mut entry = entry.map_err(|e| corrupt(archive, e))?;

        let raw = String::from_utf8_lossy(&entry.path_bytes()).into_owned();
        let target = safe_join(ctx.dest, &raw)?;
        let header = entry.header();
        let kind = header.entry_type();

        if !ctx.included(raw.trim_start_matches("./")) {
            ctx.skipped += 1;
            continue;
        }

        if kind.is_dir() {
            fs::create_dir_all(&target).map_err(|e| io_at(&target, e))?;
            ctx.extracted += 1;
        } else if kind.is_symlink() {
            let link = entry
                .link_name_bytes()
                .map(|b| String::from_utf8_lossy(&b).into_owned())
                .unwrap_or_default();
            if symlink_escapes(raw.trim_start_matches("./"), &link) {
                return Err(FulpackError::SymlinkEscape {
                    path: raw,
                    target: link,
                });
            }
            create_symlink(&link, &target, ctx)?;
        } else {
            let size = header.size().unwrap_or(0);
            ctx.add_bytes(size)?;
            if !prepare_file(&target, ctx)? {
                ctx.skipped += 1;
                continue;
            }
            let mut out = fs::File::create(&target).map_err(|e| io_at(&target, e))?;
            std::io::copy(&mut entry, &mut out).map_err(|e| io_at(&target, e))?;
            ctx.extracted += 1;
        }
    }
    Ok(())
}

fn extract_zip<R: Read + std::io::Seek>(
    reader: R,
    archive: &Path,
    ctx: &mut ExtractCtx<'_>,
) -> Result<(), FulpackError> {
    let mut zip = zip::ZipArchive::new(reader).map_err(|e| corrupt(archive, e))?;
    for i in 0..zip.len() {
        ctx.count_entry(i as u64)?;
        let mut file = zip.by_index(i).map_err(|e| corrupt(archive, e))?;
        let raw = file.name().to_string();
        let target = safe_join(ctx.dest, &raw)?;

        if file.is_dir() {
            if ctx.included(raw.trim_end_matches('/')) {
                fs::create_dir_all(&target).map_err(|e| io_at(&target, e))?;
                ctx.extracted += 1;
            }
            continue;
        }
        if !ctx.included(&raw) {
            ctx.skipped += 1;
            continue;
        }
        ctx.add_bytes(file.size())?;
        if !prepare_file(&target, ctx)? {
            ctx.skipped += 1;
            continue;
        }
        let mut out = fs::File::create(&target).map_err(|e| io_at(&target, e))?;
        std::io::copy(&mut file, &mut out).map_err(|e| io_at(&target, e))?;
        ctx.extracted += 1;
    }
    Ok(())
}

fn extract_gzip(archive: &Path, ctx: &mut ExtractCtx<'_>) -> Result<(), FulpackError> {
    let mut decoder = flate2::read::GzDecoder::new(BufReader::new(open(archive)?));
    let name = decoder
        .header()
        .and_then(|h| h.filename())
        .map(|b| String::from_utf8_lossy(b).into_owned())
        .unwrap_or_else(|| {
            archive
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output")
                .to_string()
        });
    let target = safe_join(ctx.dest, &name)?;
    if !prepare_file(&target, ctx)? {
        ctx.skipped += 1;
        return Ok(());
    }

    // Stream-copy with the running size cap so a gzip bomb is caught mid-extract.
    let mut out = fs::File::create(&target).map_err(|e| io_at(&target, e))?;
    let mut buf = [0u8; COPY_BUF];
    loop {
        let n = decoder.read(&mut buf).map_err(|e| corrupt(archive, e))?;
        if n == 0 {
            break;
        }
        ctx.add_bytes(n as u64)?;
        std::io::Write::write_all(&mut out, &buf[..n]).map_err(|e| io_at(&target, e))?;
    }
    ctx.extracted += 1;
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Ensure the parent dir exists and apply the overwrite policy. Returns `false`
/// if the entry should be skipped (exists + `skip` policy).
fn prepare_file(target: &Path, ctx: &mut ExtractCtx<'_>) -> Result<bool, FulpackError> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| io_at(parent, e))?;
    }
    if target.exists() {
        match ctx.overwrite {
            OverwriteMode::Error => {
                return Err(FulpackError::Io {
                    path: target.to_path_buf(),
                    source: std::io::Error::new(
                        std::io::ErrorKind::AlreadyExists,
                        "destination exists (overwrite policy is 'error')",
                    ),
                })
            }
            OverwriteMode::Skip => return Ok(false),
            OverwriteMode::Overwrite => {}
        }
    }
    Ok(true)
}

#[cfg(unix)]
fn create_symlink(link: &str, target: &Path, ctx: &mut ExtractCtx<'_>) -> Result<(), FulpackError> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| io_at(parent, e))?;
    }
    if target.exists() {
        let _ = fs::remove_file(target);
    }
    std::os::unix::fs::symlink(link, target).map_err(|e| io_at(target, e))?;
    ctx.extracted += 1;
    Ok(())
}

#[cfg(not(unix))]
fn create_symlink(
    _link: &str,
    target: &Path,
    ctx: &mut ExtractCtx<'_>,
) -> Result<(), FulpackError> {
    ctx.skipped += 1;
    ctx.warnings.push(format!(
        "symlink not created (unsupported platform): {}",
        target.display()
    ));
    Ok(())
}

fn io_at(path: &Path, source: std::io::Error) -> FulpackError {
    FulpackError::Io {
        path: path.to_path_buf(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_join_accepts_relative() {
        let dest = Path::new("/tmp/x");
        assert_eq!(
            safe_join(dest, "a/b/c.txt").unwrap(),
            Path::new("/tmp/x/a/b/c.txt")
        );
        assert_eq!(safe_join(dest, "./a/./b").unwrap(), Path::new("/tmp/x/a/b"));
    }

    #[test]
    fn safe_join_rejects_traversal_and_absolute() {
        let dest = Path::new("/tmp/x");
        assert_eq!(
            safe_join(dest, "../../etc/passwd").unwrap_err().code(),
            "PATH_TRAVERSAL"
        );
        assert_eq!(
            safe_join(dest, "a/../../b").unwrap_err().code(),
            "PATH_TRAVERSAL"
        );
        assert_eq!(
            safe_join(dest, "/etc/passwd").unwrap_err().code(),
            "ABSOLUTE_PATH"
        );
    }

    #[test]
    fn symlink_escape_detection() {
        // Safe: target stays within the tree.
        assert!(!symlink_escapes("dir/link", "target.txt"));
        assert!(!symlink_escapes("dir/link", "../sibling.txt")); // back to root level, still inside
                                                                 // Escapes: absolute, or climbs above the root.
        assert!(symlink_escapes("dir/link", "/etc/passwd"));
        assert!(symlink_escapes("link", "../../etc/passwd"));
        assert!(symlink_escapes("dir/link", "../../etc/passwd"));
    }
}
