//! Archive format detection.
//!
//! Detection is by file extension (case-insensitive), matching the cross-language
//! standard. Magic-byte sniffing is intentionally out of scope for v1.0.0.

use super::error::FulpackError;
use super::types::ArchiveFormat;
use std::path::Path;

/// Detect the [`ArchiveFormat`] of `archive` from its file extension.
///
/// `.tar.gz` / `.tgz` → [`ArchiveFormat::TarGz`], `.tar` → [`ArchiveFormat::Tar`],
/// `.zip` → [`ArchiveFormat::Zip`], `.gz` / `.gzip` → [`ArchiveFormat::Gzip`].
///
/// # Errors
/// Returns [`FulpackError::InvalidFormat`] if the extension is not recognized.
pub fn detect_format(archive: &Path) -> Result<ArchiveFormat, FulpackError> {
    let name = archive
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    // Order matters: check the two-part `.tar.gz` before the single `.gz`.
    if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        Ok(ArchiveFormat::TarGz)
    } else if name.ends_with(".tar") {
        Ok(ArchiveFormat::Tar)
    } else if name.ends_with(".zip") {
        Ok(ArchiveFormat::Zip)
    } else if name.ends_with(".gz") || name.ends_with(".gzip") {
        Ok(ArchiveFormat::Gzip)
    } else {
        Err(FulpackError::InvalidFormat {
            path: archive.to_path_buf(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn detects_each_format() {
        assert_eq!(
            detect_format(Path::new("a/b/data.tar")).unwrap(),
            ArchiveFormat::Tar
        );
        assert_eq!(
            detect_format(Path::new("data.tar.gz")).unwrap(),
            ArchiveFormat::TarGz
        );
        assert_eq!(
            detect_format(Path::new("data.tgz")).unwrap(),
            ArchiveFormat::TarGz
        );
        assert_eq!(
            detect_format(Path::new("data.zip")).unwrap(),
            ArchiveFormat::Zip
        );
        assert_eq!(
            detect_format(Path::new("data.gz")).unwrap(),
            ArchiveFormat::Gzip
        );
        assert_eq!(
            detect_format(Path::new("data.gzip")).unwrap(),
            ArchiveFormat::Gzip
        );
    }

    #[test]
    fn detection_is_case_insensitive() {
        assert_eq!(
            detect_format(Path::new("DATA.TAR.GZ")).unwrap(),
            ArchiveFormat::TarGz
        );
        assert_eq!(
            detect_format(Path::new("Photos.ZIP")).unwrap(),
            ArchiveFormat::Zip
        );
    }

    #[test]
    fn unknown_extension_errors() {
        let err = detect_format(Path::new("notes.txt")).unwrap_err();
        assert_eq!(err.code(), "INVALID_ARCHIVE_FORMAT");
    }
}
