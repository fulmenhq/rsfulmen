//! Error type for the `fulpack` module.
//!
//! Variants map to the canonical fulpack error codes from the Crucible standard
//! and convert into the shared [`crate::error_handling::ErrorResponse`] envelope.

use crate::error_handling::ErrorResponse;
use std::path::PathBuf;

/// Errors returned by fulpack operations.
#[derive(Debug, thiserror::Error)]
pub enum FulpackError {
    /// Archive format could not be determined from the file extension.
    #[error("unsupported or unrecognized archive format: {path}")]
    InvalidFormat {
        /// The archive path whose extension was not recognized.
        path: PathBuf,
    },

    /// Archive file does not exist.
    #[error("archive not found: {path}")]
    NotFound {
        /// The missing archive path.
        path: PathBuf,
    },

    /// Archive structure is invalid or could not be parsed.
    #[error("corrupt archive {path}: {message}")]
    Corrupt {
        /// The archive path.
        path: PathBuf,
        /// Detail of the parse failure.
        message: String,
    },

    /// Entry path attempts directory traversal (`../`). (extract/verify)
    #[error("path traversal detected: {path}")]
    PathTraversal {
        /// The offending entry path.
        path: String,
    },

    /// Entry path is absolute. (extract/verify)
    #[error("absolute path rejected: {path}")]
    AbsolutePath {
        /// The offending entry path.
        path: String,
    },

    /// Symlink target escapes the destination bounds. (extract/verify)
    #[error("symlink escape: {path} -> {target}")]
    SymlinkEscape {
        /// The symlink entry path.
        path: String,
        /// The symlink target.
        target: String,
    },

    /// Archive exceeds configured size/entry limits.
    #[error("decompression bomb guard tripped: {message}")]
    DecompressionBomb {
        /// Detail of the limit exceeded.
        message: String,
    },

    /// Entry checksum verification failed. (extract/verify)
    #[error("checksum mismatch for {path}")]
    ChecksumMismatch {
        /// The entry path whose checksum failed.
        path: String,
    },

    /// Entry count exceeded the configured safety cap.
    #[error("archive exceeds max entries ({limit})")]
    TooManyEntries {
        /// The configured limit.
        limit: u64,
    },

    /// Underlying I/O failure.
    #[error("I/O error for {path}: {source}")]
    Io {
        /// The archive path involved.
        path: PathBuf,
        /// The underlying error.
        source: std::io::Error,
    },
}

impl FulpackError {
    /// Canonical error code for this error (matches the cross-language standard).
    pub fn code(&self) -> &'static str {
        match self {
            FulpackError::InvalidFormat { .. } => "INVALID_ARCHIVE_FORMAT",
            FulpackError::NotFound { .. } => "ARCHIVE_NOT_FOUND",
            FulpackError::Corrupt { .. } => "ARCHIVE_CORRUPT",
            FulpackError::PathTraversal { .. } => "PATH_TRAVERSAL",
            FulpackError::AbsolutePath { .. } => "ABSOLUTE_PATH",
            FulpackError::SymlinkEscape { .. } => "SYMLINK_ESCAPE",
            FulpackError::DecompressionBomb { .. } => "DECOMPRESSION_BOMB",
            FulpackError::ChecksumMismatch { .. } => "CHECKSUM_MISMATCH",
            FulpackError::TooManyEntries { .. } => "MAX_ENTRIES_EXCEEDED",
            FulpackError::Io { .. } => "IO_ERROR",
        }
    }

    /// Convert into the shared canonical error envelope, tagging the operation
    /// and (optionally) the archive/entry path in the context map.
    pub fn to_error_response(
        &self,
        operation: &str,
        archive: Option<&str>,
        path: Option<&str>,
    ) -> ErrorResponse {
        let mut response = ErrorResponse::new(self.code(), self.to_string());
        response.insert_context("operation", operation.to_string());
        if let Some(archive) = archive {
            response.insert_context("archive", archive.to_string());
        }
        if let Some(path) = path {
            response.insert_context("path", path.to_string());
        }
        response
    }
}
