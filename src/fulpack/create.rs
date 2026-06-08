//! Archive creation with pathfinder-based discovery and fulhash checksums.

use super::error::FulpackError;
use super::read::{epoch_secs_to_rfc3339, info};
use super::types::{ArchiveFormat, ArchiveInfo, ChecksumAlgorithm, CreateOptions};
use crate::fulhash;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_COMPRESSION_LEVEL: u32 = 6;

/// Create an archive at `output` from `sources` (files and/or directories).
///
/// Directory sources are walked with glob `include_patterns`/`exclude_patterns`
/// (default: include everything); file sources are added under their base name.
/// An archive-level checksum is computed with [`crate::fulhash`] and reported in
/// the returned [`ArchiveInfo`]. `ArchiveFormat::Gzip` accepts exactly one file.
///
/// # Errors
/// Returns [`FulpackError`] on I/O failure, or [`FulpackError::InvalidFormat`]
/// misuse (e.g. gzip with multiple/directory sources).
pub fn create(
    sources: &[&Path],
    output: &Path,
    format: ArchiveFormat,
    options: Option<&CreateOptions>,
) -> Result<ArchiveInfo, FulpackError> {
    let owned;
    let opts = match options {
        Some(o) => o,
        None => {
            owned = CreateOptions::default();
            &owned
        }
    };

    let files = discover(sources, opts)?;
    let level = opts.compression_level.unwrap_or(DEFAULT_COMPRESSION_LEVEL);
    let preserve = opts.preserve_permissions.unwrap_or(true);

    match format {
        ArchiveFormat::Tar => write_tar(&files, output, None, preserve)?,
        ArchiveFormat::TarGz => write_tar(&files, output, Some(level), preserve)?,
        ArchiveFormat::Zip => write_zip(&files, output, level)?,
        ArchiveFormat::Gzip => write_gzip(&files, output, level)?,
    }

    // Re-read the written archive for canonical metadata, then attach checksum + created.
    let mut archive_info = info(output)?;
    let algo = opts.checksum_algorithm.unwrap_or(ChecksumAlgorithm::Sha256);
    let (fulhash_algo, label) = map_algorithm(algo);
    let digest = fulhash::hash_file(output, Some(fulhash_algo)).map_err(|e| FulpackError::Io {
        path: output.to_path_buf(),
        source: std::io::Error::other(e.to_string()),
    })?;
    let mut checksums = BTreeMap::new();
    checksums.insert(label.to_string(), fulhash::format_digest(&digest));

    archive_info.has_checksums = Some(true);
    archive_info.checksum_algorithm = Some(algo_from_label(label));
    archive_info.checksums = Some(checksums);
    archive_info.created = Some(now_rfc3339());
    Ok(archive_info)
}

/// A discovered entry: filesystem path + the path stored in the archive.
struct Entry {
    fs_path: PathBuf,
    archive_path: String,
    /// `Some(target)` for a symlink that must NOT be followed — archived as a
    /// symlink entry (tar/tar.gz) rather than by reading the target's bytes.
    link_target: Option<String>,
}

/// If `path` is a symlink and `follow` is false, return its raw target (the entry
/// must be archived as a symlink, never by reading the target).
fn link_target_if_unfollowed(path: &Path, follow: bool) -> Option<String> {
    if follow {
        return None;
    }
    match std::fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => std::fs::read_link(path)
            .ok()
            .map(|t| t.to_string_lossy().into_owned()),
        _ => None,
    }
}

fn discover(sources: &[&Path], opts: &CreateOptions) -> Result<Vec<Entry>, FulpackError> {
    let includes = opts
        .include_patterns
        .clone()
        .unwrap_or_else(|| vec!["**/*".to_string()]);
    let excludes = opts.exclude_patterns.clone().unwrap_or_default();
    let follow = opts.follow_symlinks.unwrap_or(false);

    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for source in sources {
        let meta = std::fs::symlink_metadata(source).map_err(|e| FulpackError::Io {
            path: source.to_path_buf(),
            source: e,
        })?;
        if meta.is_dir() {
            let base = source
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let query = crate::pathfinder::FindQuery {
                root: source.to_path_buf(),
                include: includes.clone(),
                exclude: excludes.clone(),
                max_depth: None,
                follow_symlinks: follow,
                include_hidden: true,
                checksums: false,
            };
            let results = crate::pathfinder::find_files(&query).map_err(|e| FulpackError::Io {
                path: source.to_path_buf(),
                source: std::io::Error::other(e.to_string()),
            })?;
            for file in results.files {
                let rel = file.relative_path.to_string_lossy().replace('\\', "/");
                let archive_path = if base.is_empty() {
                    rel
                } else {
                    format!("{base}/{rel}")
                };
                if seen.insert(archive_path.clone()) {
                    let link_target = link_target_if_unfollowed(&file.source_path, follow);
                    out.push(Entry {
                        fs_path: file.source_path,
                        archive_path,
                        link_target,
                    });
                }
            }
        } else {
            let name = source
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if matches_any(&includes, &name)
                && !matches_any(&excludes, &name)
                && seen.insert(name.clone())
            {
                let link_target = link_target_if_unfollowed(source, follow);
                out.push(Entry {
                    fs_path: source.to_path_buf(),
                    archive_path: name,
                    link_target,
                });
            }
        }
    }
    Ok(out)
}

fn matches_any(patterns: &[String], name: &str) -> bool {
    patterns.iter().any(|p| {
        glob::Pattern::new(p)
            .map(|pat| pat.matches(name))
            .unwrap_or(false)
    })
}

fn create_output(output: &Path) -> Result<std::fs::File, FulpackError> {
    std::fs::File::create(output).map_err(|e| FulpackError::Io {
        path: output.to_path_buf(),
        source: e,
    })
}

fn write_tar(
    files: &[Entry],
    output: &Path,
    gzip_level: Option<u32>,
    preserve: bool,
) -> Result<(), FulpackError> {
    let file = create_output(output)?;
    match gzip_level {
        Some(level) => {
            let enc = flate2::write::GzEncoder::new(file, flate2::Compression::new(level));
            append_tar_entries(tar::Builder::new(enc), files, output, preserve)
        }
        None => append_tar_entries(tar::Builder::new(file), files, output, preserve),
    }
}

fn append_tar_entries<W: Write>(
    mut builder: tar::Builder<W>,
    files: &[Entry],
    output: &Path,
    preserve: bool,
) -> Result<(), FulpackError> {
    for entry in files {
        // Un-followed symlink: store as a symlink entry, never the target bytes.
        if let Some(target) = &entry.link_target {
            let mut header = tar::Header::new_gnu();
            header.set_entry_type(tar::EntryType::Symlink);
            header.set_size(0);
            header.set_mode(0o777);
            builder
                .append_link(&mut header, &entry.archive_path, target)
                .map_err(|e| FulpackError::Io {
                    path: output.to_path_buf(),
                    source: e,
                })?;
            continue;
        }
        let data = std::fs::read(&entry.fs_path).map_err(|e| FulpackError::Io {
            path: entry.fs_path.clone(),
            source: e,
        })?;
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_entry_type(tar::EntryType::Regular);
        header.set_mode(file_mode(&entry.fs_path, preserve));
        header.set_mtime(file_mtime(&entry.fs_path));
        header.set_cksum();
        builder
            .append_data(&mut header, &entry.archive_path, &data[..])
            .map_err(|e| FulpackError::Io {
                path: output.to_path_buf(),
                source: e,
            })?;
    }
    builder.into_inner().map_err(|e| FulpackError::Io {
        path: output.to_path_buf(),
        source: e,
    })?;
    Ok(())
}

fn write_zip(files: &[Entry], output: &Path, level: u32) -> Result<(), FulpackError> {
    let file = create_output(output)?;
    let mut writer = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(level as i64));
    for entry in files {
        // ZIP does not support symlinks (taxonomy); skip un-followed symlinks.
        if entry.link_target.is_some() {
            continue;
        }
        let data = std::fs::read(&entry.fs_path).map_err(|e| FulpackError::Io {
            path: entry.fs_path.clone(),
            source: e,
        })?;
        writer
            .start_file(&entry.archive_path, options)
            .map_err(|e| zip_err(output, e))?;
        writer.write_all(&data).map_err(|e| FulpackError::Io {
            path: output.to_path_buf(),
            source: e,
        })?;
    }
    writer.finish().map_err(|e| zip_err(output, e))?;
    Ok(())
}

fn write_gzip(files: &[Entry], output: &Path, level: u32) -> Result<(), FulpackError> {
    // gzip is single-file only, over the *discovered* (filtered) set, and cannot
    // store a symlink (no following) — reject anything else.
    if files.len() != 1 || files[0].link_target.is_some() {
        return Err(FulpackError::InvalidFormat {
            path: output.to_path_buf(),
        });
    }
    let entry = &files[0];
    let data = std::fs::read(&entry.fs_path).map_err(|e| FulpackError::Io {
        path: entry.fs_path.clone(),
        source: e,
    })?;
    let mut encoder = flate2::GzBuilder::new()
        .filename(entry.archive_path.clone().into_bytes())
        .write(create_output(output)?, flate2::Compression::new(level));
    encoder.write_all(&data).map_err(|e| FulpackError::Io {
        path: output.to_path_buf(),
        source: e,
    })?;
    encoder.finish().map_err(|e| FulpackError::Io {
        path: output.to_path_buf(),
        source: e,
    })?;
    Ok(())
}

fn zip_err(output: &Path, e: zip::result::ZipError) -> FulpackError {
    FulpackError::Io {
        path: output.to_path_buf(),
        source: std::io::Error::other(e.to_string()),
    }
}

#[cfg(unix)]
fn file_mode(path: &Path, preserve: bool) -> u32 {
    use std::os::unix::fs::MetadataExt;
    if preserve {
        std::fs::metadata(path)
            .map(|m| m.mode() & 0o7777)
            .unwrap_or(0o644)
    } else {
        0o644
    }
}

#[cfg(not(unix))]
fn file_mode(_path: &Path, _preserve: bool) -> u32 {
    0o644
}

fn file_mtime(path: &Path) -> u64 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn now_rfc3339() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    epoch_secs_to_rfc3339(secs)
}

/// Map a [`ChecksumAlgorithm`] to a [`fulhash::Algorithm`] and its canonical
/// label. Algorithms beyond the two fulhash supports fall back to SHA-256.
fn map_algorithm(algo: ChecksumAlgorithm) -> (fulhash::Algorithm, &'static str) {
    match algo {
        ChecksumAlgorithm::Xxh3_128 => (fulhash::Algorithm::Xxh3_128, "xxh3-128"),
        _ => (fulhash::Algorithm::Sha256, "sha256"),
    }
}

fn algo_from_label(label: &str) -> ChecksumAlgorithm {
    match label {
        "xxh3-128" => ChecksumAlgorithm::Xxh3_128,
        _ => ChecksumAlgorithm::Sha256,
    }
}
