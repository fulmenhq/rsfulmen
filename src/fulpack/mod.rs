//! Canonical archive operations (tar, tar.gz, zip, gzip).
//!
//! `fulpack` implements the Crucible **Fulpack Archive Module Standard** (v1.0.0,
//! Common tier) — see `docs/crucible-rs/standards/library/modules/fulpack.md`.
//! It provides typed, cross-language-consistent archive operations with a
//! security-by-default model.
//!
//! This release implements the read-only operations:
//!
//! - [`info`] — archive metadata without listing entries
//! - [`scan`] — list the table of contents without extraction
//!
//! `scan` is *discovery* and lists every entry as stored (traversal/absolute/
//! symlink paths included; absolute paths normalized to relative). Security
//! *enforcement* lives in the extract/verify operations (added in a later release).
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
mod format;
mod read;
mod types;

pub use error::FulpackError;
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
}
