//! Canonical archive operations (tar, tar.gz, zip, gzip).
//!
//! `fulpack` implements the Crucible **Fulpack Archive Module Standard** (v1.0.0,
//! Common tier) — see `docs/crucible-rs/standards/library/modules/fulpack.md`.
//! It provides typed, cross-language-consistent archive operations with a
//! security-by-default model.
//!
//! Implemented operations:
//!
//! - [`info`] — archive metadata without listing entries
//! - [`scan`] — list the table of contents without extraction
//! - [`extract`] — extract to a destination with the full security model
//!
//! `scan` is *discovery* and lists every entry as stored (traversal/absolute/
//! symlink paths included; absolute paths normalized to relative). `extract` is
//! *enforcing*: it rejects traversal/absolute paths, escaping symlinks, and
//! decompression bombs. (`create`/`verify` arrive in a later release.)
//!
//! ```no_run
//! use rsfulmen::fulpack;
//! use std::path::Path;
//!
//! let meta = fulpack::info(Path::new("release.tar.gz")).unwrap();
//! println!("{} entries, {} bytes", meta.entry_count, meta.total_size);
//!
//! for entry in fulpack::scan(Path::new("release.tar.gz"), None).unwrap() {
//!     println!("{}", entry.path);
//! }
//! ```

mod error;
mod extract;
mod format;
mod read;
mod types;

pub use error::FulpackError;
pub use extract::extract;
pub use format::detect_format;
pub use read::{info, scan};
pub use types::{
    ArchiveEntry, ArchiveFormat, ArchiveInfo, ArchiveManifest, ChecksumAlgorithm, Compression,
    CreateOptions, EntryType, ExtractOptions, ExtractResult, OverwriteMode, ScanOptions,
    ValidationResult,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    // Synced Crucible fixtures (embedded so tests don't depend on cwd).
    const BASIC_TAR: &[u8] =
        include_bytes!("../../config/crucible-rs/library/fulpack/fixtures/basic.tar");
    const BASIC_TAR_GZ: &[u8] =
        include_bytes!("../../config/crucible-rs/library/fulpack/fixtures/basic.tar.gz");
    const NESTED_ZIP: &[u8] =
        include_bytes!("../../config/crucible-rs/library/fulpack/fixtures/nested.zip");
    const PATHOLOGICAL_TAR_GZ: &[u8] =
        include_bytes!("../../config/crucible-rs/library/fulpack/fixtures/pathological.tar.gz");

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    /// RAII temp dir with a unique name (process id + nanos + atomic counter, so
    /// parallel test threads never collide).
    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time after epoch")
                .as_nanos();
            let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
            let mut path = std::env::temp_dir();
            path.push(format!(
                "rsfulmen-fulpack-{}-{}-{}",
                std::process::id(),
                nanos,
                seq
            ));
            fs::create_dir_all(&path).expect("create temp dir");
            Self { path }
        }

        /// Materialize `bytes` into a file named `name` (extension matters — the
        /// API detects format by extension) and return its path.
        fn materialize(&self, name: &str, bytes: &[u8]) -> PathBuf {
            let path = self.path.join(name);
            fs::write(&path, bytes).expect("write fixture");
            path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn paths(entries: &[ArchiveEntry]) -> Vec<&str> {
        entries.iter().map(|e| e.path.as_str()).collect()
    }

    #[test]
    fn info_basic_tar_reports_uncompressed() {
        let tmp = TestDir::new();
        let archive = tmp.materialize("basic.tar", BASIC_TAR);

        let meta = info(&archive).expect("info basic.tar");
        assert_eq!(meta.format, ArchiveFormat::Tar);
        assert_eq!(meta.compression, Some(Compression::None));
        assert_eq!(meta.compression_ratio, Some(1.0));
        assert_eq!(meta.compressed_size, BASIC_TAR.len() as u64);
        assert!(meta.entry_count >= 1);
        assert!(meta.total_size > 0);
        assert_eq!(meta.has_checksums, Some(false));
    }

    #[test]
    fn scan_basic_tar_lists_known_files() {
        let tmp = TestDir::new();
        let archive = tmp.materialize("basic.tar", BASIC_TAR);

        let entries = scan(&archive, None).expect("scan basic.tar");
        let names = paths(&entries);
        assert!(names.contains(&"README.md"), "paths: {names:?}");
        // All entries are file/directory; sizes are populated for files.
        assert!(entries
            .iter()
            .any(|e| e.entry_type == EntryType::File && e.size > 0));
        // info() entry_count agrees with scan() length.
        assert_eq!(info(&archive).unwrap().entry_count, entries.len() as u64);
    }

    #[test]
    fn info_basic_tar_gz_reports_compression() {
        let tmp = TestDir::new();
        let archive = tmp.materialize("basic.tar.gz", BASIC_TAR_GZ);

        let meta = info(&archive).expect("info basic.tar.gz");
        assert_eq!(meta.format, ArchiveFormat::TarGz);
        assert_eq!(meta.compression, Some(Compression::Gzip));
        assert!(meta.compression_ratio.unwrap() > 1.0);
        assert_eq!(meta.compressed_size, BASIC_TAR_GZ.len() as u64);
    }

    #[test]
    fn scan_nested_zip_handles_depth_filter() {
        let tmp = TestDir::new();
        let archive = tmp.materialize("nested.zip", NESTED_ZIP);

        let meta = info(&archive).expect("info nested.zip");
        assert_eq!(meta.format, ArchiveFormat::Zip);
        assert_eq!(meta.compression, Some(Compression::Deflate));

        let all = scan(&archive, None).expect("scan nested.zip");
        assert!(
            paths(&all).contains(&"root.txt"),
            "paths: {:?}",
            paths(&all)
        );
        assert!(all.iter().any(|e| e.path.contains("level3")));

        // max_depth=0 keeps only top-level entries.
        let shallow = scan(
            &archive,
            Some(&ScanOptions {
                max_depth: Some(0),
                ..Default::default()
            }),
        )
        .expect("shallow scan");
        assert!(shallow
            .iter()
            .all(|e| !e.path.trim_end_matches('/').contains('/')));
        assert!(shallow.len() < all.len());
    }

    #[test]
    fn scan_files_only_filter() {
        let tmp = TestDir::new();
        let archive = tmp.materialize("nested.zip", NESTED_ZIP);

        let files = scan(
            &archive,
            Some(&ScanOptions {
                entry_types: Some(vec![EntryType::File]),
                ..Default::default()
            }),
        )
        .expect("files-only scan");
        assert!(files.iter().all(|e| e.entry_type == EntryType::File));
        assert!(!files.is_empty());
    }

    #[test]
    fn scan_pathological_lists_without_rejecting() {
        // Read-path contract: scan MUST list a malicious archive's entries (it does
        // not enforce security — extract/verify do) and MUST normalize absolute
        // paths to relative.
        let tmp = TestDir::new();
        let archive = tmp.materialize("pathological.tar.gz", PATHOLOGICAL_TAR_GZ);

        let entries = scan(&archive, None).expect("scan must not reject malicious archives");
        assert!(!entries.is_empty());
        assert!(
            entries.iter().all(|e| !e.path.starts_with('/')),
            "absolute paths must be normalized to relative in scan"
        );
    }

    #[test]
    fn gzip_single_file_roundtrip() {
        // No `.gz` fixture is synced, so build a gzip stream to exercise that path.
        let tmp = TestDir::new();
        let payload = b"hello fulpack gzip path\n".repeat(8);
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(&payload).unwrap();
        let gz = encoder.finish().unwrap();
        let archive = tmp.materialize("payload.txt.gz", &gz);

        let meta = info(&archive).expect("info gzip");
        assert_eq!(meta.format, ArchiveFormat::Gzip);
        assert_eq!(meta.entry_count, 1);
        assert_eq!(meta.total_size, payload.len() as u64);

        let entries = scan(&archive, None).expect("scan gzip");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entry_type, EntryType::File);
        assert_eq!(entries[0].size, payload.len() as u64);
    }

    #[test]
    fn missing_archive_is_not_found() {
        let tmp = TestDir::new();
        let archive = tmp.path.join("does-not-exist.tar.gz");
        let err = info(&archive).unwrap_err();
        assert_eq!(err.code(), "ARCHIVE_NOT_FOUND");
    }

    #[test]
    fn archive_info_serializes_to_schema_field_names() {
        let tmp = TestDir::new();
        let archive = tmp.materialize("basic.tar.gz", BASIC_TAR_GZ);
        let meta = info(&archive).unwrap();
        let json = serde_json::to_value(&meta).unwrap();
        assert_eq!(json["format"], "tar.gz");
        assert!(json.get("entry_count").is_some());
        assert!(json.get("total_size").is_some());
    }

    /// Build an uncompressed tar from `(path, data)` pairs (used to craft both
    /// benign and malicious archives in tests).
    fn build_tar(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut builder = tar::Builder::new(Vec::new());
        for (path, data) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_entry_type(tar::EntryType::Regular);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append_data(&mut header, path, &data[..])
                .expect("append tar entry");
        }
        builder.into_inner().expect("finish tar")
    }

    #[test]
    fn extract_roundtrip_writes_files() {
        let tmp = TestDir::new();
        let archive = tmp.materialize("basic.tar.gz", BASIC_TAR_GZ);
        let dest = tmp.path.join("out");

        // Pick a real file entry from the archive to assert on (robust to fixture content).
        let a_file = scan(&archive, None)
            .unwrap()
            .into_iter()
            .find(|e| e.entry_type == EntryType::File)
            .expect("a file entry");

        let result = extract(&archive, &dest, None).expect("extract");
        assert!(result.extracted_count > 0);
        assert!(dest.join(&a_file.path).exists(), "missing {}", a_file.path);
    }

    #[test]
    fn extract_pathological_fixture_is_safe_by_construction() {
        // This canonical fixture simulates attack *shapes* via naming but contains
        // no real traversal/absolute/escape, so extraction succeeds.
        let tmp = TestDir::new();
        let archive = tmp.materialize("pathological.tar.gz", PATHOLOGICAL_TAR_GZ);
        let dest = tmp.path.join("out");
        let result = extract(&archive, &dest, None).expect("safe fixture extracts");
        assert!(result.extracted_count > 0);
        assert!(dest.join("legitimate.txt").exists());
    }

    /// Build a zip from `(name, data)` pairs. Unlike the tar builder, the zip
    /// writer stores names verbatim — including `../` — which lets us craft a
    /// zip-slip archive to prove extract rejects it.
    fn build_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut cursor = std::io::Cursor::new(Vec::new());
        {
            let mut writer = zip::ZipWriter::new(&mut cursor);
            let options = zip::write::SimpleFileOptions::default();
            for (name, data) in entries {
                writer.start_file(*name, options).expect("start zip entry");
                writer.write_all(data).expect("write zip entry");
            }
            writer.finish().expect("finish zip");
        }
        cursor.into_inner()
    }

    #[test]
    fn extract_rejects_path_traversal() {
        let tmp = TestDir::new();
        let bytes = build_zip(&[("../evil.txt", b"pwned")]);
        let archive = tmp.materialize("evil.zip", &bytes);
        let dest = tmp.path.join("out");

        let err = extract(&archive, &dest, None).unwrap_err();
        assert_eq!(err.code(), "PATH_TRAVERSAL");
        assert!(!tmp.path.join("evil.txt").exists(), "must not escape dest");
    }

    #[test]
    fn extract_enforces_max_entries_bomb_guard() {
        let tmp = TestDir::new();
        let bytes = build_tar(&[("a.txt", b"a"), ("b.txt", b"b"), ("c.txt", b"c")]);
        let archive = tmp.materialize("many.tar", &bytes);
        let dest = tmp.path.join("out");

        let err = extract(
            &archive,
            &dest,
            Some(&ExtractOptions {
                max_entries: Some(2),
                ..Default::default()
            }),
        )
        .unwrap_err();
        assert_eq!(err.code(), "DECOMPRESSION_BOMB");
    }

    #[test]
    fn extract_overwrite_policy() {
        let tmp = TestDir::new();
        let archive = tmp.materialize("basic.tar.gz", BASIC_TAR_GZ);
        let dest = tmp.path.join("out");

        extract(&archive, &dest, None).expect("first extract");

        // Default policy is "error" -> re-extract over existing files fails.
        assert!(extract(&archive, &dest, None).is_err());

        // "skip" policy succeeds, skipping existing files.
        let result = extract(
            &archive,
            &dest,
            Some(&ExtractOptions {
                overwrite: Some(OverwriteMode::Skip),
                ..Default::default()
            }),
        )
        .expect("skip extract");
        assert!(result.skipped_count > 0);
    }

    /// Build a tar containing a single symlink entry.
    fn build_tar_symlink(link_path: &str, target: &str) -> Vec<u8> {
        let mut builder = tar::Builder::new(Vec::new());
        let mut header = tar::Header::new_gnu();
        header.set_entry_type(tar::EntryType::Symlink);
        header.set_size(0);
        header.set_mode(0o777);
        builder
            .append_link(&mut header, link_path, target)
            .expect("append symlink");
        builder.into_inner().expect("finish tar")
    }

    #[cfg(unix)]
    #[test]
    fn extract_rejects_write_through_preexisting_symlink() {
        // P1: a lexically-safe archive path that traverses a pre-existing symlinked
        // directory under the destination must be rejected (no escape).
        let tmp = TestDir::new();
        let outside = tmp.path.join("outside");
        fs::create_dir_all(&outside).unwrap();
        let dest = tmp.path.join("dest");
        fs::create_dir_all(&dest).unwrap();
        std::os::unix::fs::symlink(&outside, dest.join("link")).unwrap();

        let bytes = build_zip(&[("link/payload.txt", b"pwned")]);
        let archive = tmp.materialize("eviltree.zip", &bytes);

        let err = extract(&archive, &dest, None).unwrap_err();
        assert_eq!(err.code(), "SYMLINK_ESCAPE");
        assert!(
            !outside.join("payload.txt").exists(),
            "must not write through the symlinked directory"
        );
    }

    #[cfg(unix)]
    #[test]
    fn extract_symlink_respects_overwrite_policy() {
        // P2a: symlink entries must honor the overwrite policy like files do.
        let tmp = TestDir::new();
        let archive = tmp.materialize("link.tar", &build_tar_symlink("mylink", "target.txt"));
        let dest = tmp.path.join("out");

        extract(&archive, &dest, None).expect("first extract");
        assert!(fs::symlink_metadata(dest.join("mylink"))
            .unwrap()
            .file_type()
            .is_symlink());

        // Default 'error' policy must not silently overwrite.
        assert!(extract(&archive, &dest, None).is_err());

        // 'skip' leaves it and reports skipped.
        let result = extract(
            &archive,
            &dest,
            Some(&ExtractOptions {
                overwrite: Some(OverwriteMode::Skip),
                ..Default::default()
            }),
        )
        .expect("skip extract");
        assert!(result.skipped_count >= 1);
    }

    #[test]
    fn extract_gzip_honors_options() {
        // P2b: gzip extraction applies max_entries and include_patterns.
        let tmp = TestDir::new();
        let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        enc.write_all(&b"data".repeat(4)).unwrap();
        let gz = enc.finish().unwrap();
        let archive = tmp.materialize("data.txt.gz", &gz);
        let dest = tmp.path.join("out");

        let err = extract(
            &archive,
            &dest,
            Some(&ExtractOptions {
                max_entries: Some(0),
                ..Default::default()
            }),
        )
        .unwrap_err();
        assert_eq!(err.code(), "DECOMPRESSION_BOMB");

        let result = extract(
            &archive,
            &dest,
            Some(&ExtractOptions {
                include_patterns: Some(vec!["*.csv".to_string()]),
                ..Default::default()
            }),
        )
        .expect("non-matching include skips");
        assert_eq!(result.extracted_count, 0);
        assert!(result.skipped_count >= 1);
    }
}
