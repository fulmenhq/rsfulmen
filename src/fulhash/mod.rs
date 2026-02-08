//! Canonical hashing with integrity verification.
//!
//! This module provides a uniform API for computing and verifying content
//! hashes using either the fast xxHash3-128 algorithm (default) or
//! cryptographic SHA-256.
//!
//! Digests are formatted with a canonical `algorithm:hex` prefix so they
//! can be persisted, compared, and exchanged with other Fulmen SDK
//! implementations (e.g., gofulmen's `FormatDigest`).
//!
//! # Quick Start
//!
//! ```
//! use rsfulmen::fulhash::{hash, verify, format_digest};
//!
//! let digest = hash(b"hello world", None).unwrap();
//! assert!(verify(b"hello world", &digest));
//! println!("{}", format_digest(&digest));
//! ```
//!
//! # Algorithms
//!
//! | Name       | Variant               | Digest size | Use-case                     |
//! |------------|-----------------------|-------------|------------------------------|
//! | xxh3-128   | `Algorithm::Xxh3_128` | 16 bytes    | Fast content-addressable IDs |
//! | SHA-256    | `Algorithm::Sha256`   | 32 bytes    | Cryptographic integrity      |

use serde::{Deserialize, Serialize};
use sha2::Digest as ShaDigest;
use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Default buffer size for streaming reads (8 KiB).
const DEFAULT_BUFFER_SIZE: usize = 8192;

// ---------------------------------------------------------------------------
// Algorithm
// ---------------------------------------------------------------------------

/// Supported hash algorithms.
///
/// The default algorithm is [`Algorithm::Xxh3_128`] — a fast,
/// non-cryptographic hash suitable for content-addressable storage.
/// Use [`Algorithm::Sha256`] when a cryptographic guarantee is required.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Algorithm {
    /// xxHash3-128 — fast, non-cryptographic (default).
    Xxh3_128,
    /// SHA-256 — cryptographic.
    Sha256,
}

impl Default for Algorithm {
    fn default() -> Self {
        Self::Xxh3_128
    }
}

impl Algorithm {
    /// Returns the canonical string tag used in the digest format.
    fn tag(self) -> &'static str {
        match self {
            Self::Xxh3_128 => "xxh3-128",
            Self::Sha256 => "sha256",
        }
    }

    /// Parse an algorithm from its canonical tag.
    fn from_tag(s: &str) -> Result<Self, FulhashError> {
        match s {
            "xxh3-128" => Ok(Self::Xxh3_128),
            "sha256" => Ok(Self::Sha256),
            other => Err(FulhashError::UnsupportedAlgorithm(other.to_string())),
        }
    }

    /// Expected digest byte length for this algorithm.
    fn digest_len(self) -> usize {
        match self {
            Self::Xxh3_128 => 16,
            Self::Sha256 => 32,
        }
    }
}

// ---------------------------------------------------------------------------
// Digest
// ---------------------------------------------------------------------------

/// A computed hash digest pairing an algorithm with raw hash bytes.
///
/// Use [`format_digest`] (or the [`Display`](fmt::Display) impl) to obtain
/// the canonical `algorithm:hex` string representation, and [`parse_digest`]
/// to round-trip it back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Digest {
    /// The algorithm that produced this digest.
    pub algorithm: Algorithm,
    /// Raw hash bytes.
    pub bytes: Vec<u8>,
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format_digest(self))
    }
}

// ---------------------------------------------------------------------------
// HashOptions
// ---------------------------------------------------------------------------

/// Options for streaming hash operations.
///
/// When not supplied, sensible defaults are used:
/// - `algorithm`: [`Algorithm::Xxh3_128`]
/// - `buffer_size`: 8192
#[derive(Debug, Clone)]
pub struct HashOptions {
    /// Algorithm to use (default: [`Algorithm::Xxh3_128`]).
    pub algorithm: Algorithm,
    /// Buffer size in bytes for streaming reads (default: 8192).
    pub buffer_size: usize,
}

impl Default for HashOptions {
    fn default() -> Self {
        Self {
            algorithm: Algorithm::default(),
            buffer_size: DEFAULT_BUFFER_SIZE,
        }
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors that can occur during hashing or digest parsing.
#[derive(Debug, thiserror::Error)]
pub enum FulhashError {
    /// The requested algorithm name is not recognised.
    #[error("unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),

    /// A digest string could not be parsed.
    #[error("invalid digest format: {0}")]
    InvalidDigestFormat(String),

    /// Hex-encoded bytes could not be decoded.
    #[error("hex decode error: {source}")]
    HexDecode {
        /// Underlying hex crate error.
        source: hex::FromHexError,
    },

    /// An I/O error occurred while reading data.
    #[error("I/O error: {source}")]
    IoError {
        /// Underlying I/O error.
        #[from]
        source: std::io::Error,
    },
}

// ---------------------------------------------------------------------------
// Core hashing helpers (algorithm dispatch)
// ---------------------------------------------------------------------------

/// Hash a complete byte slice using xxHash3-128.
fn hash_xxh3_128(data: &[u8]) -> Vec<u8> {
    let h = xxhash_rust::xxh3::xxh3_128(data);
    h.to_be_bytes().to_vec()
}

/// Hash a complete byte slice using SHA-256.
fn hash_sha256(data: &[u8]) -> Vec<u8> {
    let mut hasher = sha2::Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Streaming hash using xxHash3-128.
fn hash_xxh3_128_reader<R: Read>(reader: R, buffer_size: usize) -> Result<Vec<u8>, std::io::Error> {
    use xxhash_rust::xxh3::Xxh3;

    let mut hasher = Xxh3::new();
    let mut reader = reader;
    let mut buf = vec![0u8; buffer_size];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let h = hasher.digest128();
    Ok(h.to_be_bytes().to_vec())
}

/// Streaming hash using SHA-256.
fn hash_sha256_reader<R: Read>(reader: R, buffer_size: usize) -> Result<Vec<u8>, std::io::Error> {
    let mut hasher = sha2::Sha256::new();
    let mut reader = reader;
    let mut buf = vec![0u8; buffer_size];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize().to_vec())
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Hash a byte slice using the given algorithm (default: xxHash3-128).
///
/// # Examples
///
/// ```
/// use rsfulmen::fulhash::{hash, Algorithm};
///
/// let digest = hash(b"hello", None).unwrap();
/// assert_eq!(digest.algorithm, Algorithm::Xxh3_128);
///
/// let sha = hash(b"hello", Some(Algorithm::Sha256)).unwrap();
/// assert_eq!(sha.bytes.len(), 32);
/// ```
pub fn hash(data: &[u8], algorithm: Option<Algorithm>) -> Result<Digest, FulhashError> {
    let algo = algorithm.unwrap_or_default();
    let bytes = match algo {
        Algorithm::Xxh3_128 => hash_xxh3_128(data),
        Algorithm::Sha256 => hash_sha256(data),
    };
    Ok(Digest {
        algorithm: algo,
        bytes,
    })
}

/// Hash a UTF-8 string using the given algorithm (default: xxHash3-128).
///
/// This is a convenience wrapper around [`hash`] that takes a `&str`.
///
/// # Examples
///
/// ```
/// use rsfulmen::fulhash::hash_string;
///
/// let digest = hash_string("hello world", None).unwrap();
/// ```
pub fn hash_string(s: &str, algorithm: Option<Algorithm>) -> Result<Digest, FulhashError> {
    hash(s.as_bytes(), algorithm)
}

/// Hash data from an arbitrary [`Read`] source using streaming buffered reads.
///
/// If `options` is `None`, defaults are used (xxHash3-128, 8 KiB buffer).
///
/// # Examples
///
/// ```
/// use rsfulmen::fulhash::hash_reader;
///
/// let data = b"stream me";
/// let digest = hash_reader(std::io::Cursor::new(data), None).unwrap();
/// ```
pub fn hash_reader<R: Read>(
    reader: R,
    options: Option<HashOptions>,
) -> Result<Digest, FulhashError> {
    let opts = options.unwrap_or_default();
    let buffer_size = if opts.buffer_size == 0 {
        DEFAULT_BUFFER_SIZE
    } else {
        opts.buffer_size
    };
    let bytes = match opts.algorithm {
        Algorithm::Xxh3_128 => hash_xxh3_128_reader(reader, buffer_size)?,
        Algorithm::Sha256 => hash_sha256_reader(reader, buffer_size)?,
    };
    Ok(Digest {
        algorithm: opts.algorithm,
        bytes,
    })
}

/// Hash the contents of a file at `path` using the given algorithm.
///
/// The file is read in streaming fashion via [`hash_reader`].
///
/// # Examples
///
/// ```no_run
/// use rsfulmen::fulhash::hash_file;
/// use std::path::Path;
///
/// let digest = hash_file(Path::new("Cargo.toml"), None).unwrap();
/// ```
pub fn hash_file(path: &Path, algorithm: Option<Algorithm>) -> Result<Digest, FulhashError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let options = HashOptions {
        algorithm: algorithm.unwrap_or_default(),
        ..HashOptions::default()
    };
    hash_reader(reader, Some(options))
}

/// Format a [`Digest`] as a canonical `algorithm:hex` string.
///
/// The format matches gofulmen's `FormatDigest` output:
/// - `xxh3-128:<lowercase-hex>` for xxHash3-128
/// - `sha256:<lowercase-hex>` for SHA-256
///
/// # Examples
///
/// ```
/// use rsfulmen::fulhash::{hash, format_digest};
///
/// let digest = hash(b"data", None).unwrap();
/// let s = format_digest(&digest);
/// assert!(s.starts_with("xxh3-128:"));
/// ```
pub fn format_digest(digest: &Digest) -> String {
    format!("{}:{}", digest.algorithm.tag(), hex::encode(&digest.bytes))
}

/// Parse a canonical `algorithm:hex` string back into a [`Digest`].
///
/// # Errors
///
/// Returns [`FulhashError::InvalidDigestFormat`] if the string has no colon
/// separator, [`FulhashError::UnsupportedAlgorithm`] if the algorithm tag is
/// unknown, or [`FulhashError::HexDecode`] if the hex portion is invalid.
///
/// # Examples
///
/// ```
/// use rsfulmen::fulhash::{parse_digest, Algorithm};
///
/// let digest = parse_digest("sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824").unwrap();
/// assert_eq!(digest.algorithm, Algorithm::Sha256);
/// ```
pub fn parse_digest(s: &str) -> Result<Digest, FulhashError> {
    let Some((algo_str, hex_str)) = s.split_once(':') else {
        return Err(FulhashError::InvalidDigestFormat(format!(
            "missing ':' separator in '{s}'"
        )));
    };

    let algorithm = Algorithm::from_tag(algo_str)?;

    // Reject empty hex portion.
    if hex_str.is_empty() {
        return Err(FulhashError::InvalidDigestFormat(
            "hex portion is empty".to_string(),
        ));
    }

    // Require canonical lowercase hex.
    if hex_str.chars().any(|c| c.is_ascii_uppercase()) {
        return Err(FulhashError::InvalidDigestFormat(
            "hex must be lowercase".to_string(),
        ));
    }

    let bytes = hex::decode(hex_str).map_err(|source| FulhashError::HexDecode { source })?;

    // Validate algorithm-specific digest length.
    let expected = algorithm.digest_len();
    if bytes.len() != expected {
        return Err(FulhashError::InvalidDigestFormat(format!(
            "expected {} bytes for {}, got {}",
            expected,
            algo_str,
            bytes.len()
        )));
    }

    Ok(Digest { algorithm, bytes })
}

/// Verify that `data` matches an expected [`Digest`].
///
/// Returns `true` if the computed hash of `data` equals `expected`, `false`
/// otherwise. A mismatch is a normal outcome, not an error.
///
/// # Examples
///
/// ```
/// use rsfulmen::fulhash::{hash, verify};
///
/// let digest = hash(b"hello", None).unwrap();
/// assert!(verify(b"hello", &digest));
/// assert!(!verify(b"world", &digest));
/// ```
pub fn verify(data: &[u8], expected: &Digest) -> bool {
    match hash(data, Some(expected.algorithm)) {
        Ok(actual) => actual.bytes == expected.bytes,
        Err(_) => false,
    }
}

/// Verify that a file's contents match an expected [`Digest`].
///
/// # Errors
///
/// Returns an I/O error if the file cannot be read.
///
/// # Examples
///
/// ```no_run
/// use rsfulmen::fulhash::{hash_file, verify_file};
/// use std::path::Path;
///
/// let digest = hash_file(Path::new("Cargo.toml"), None).unwrap();
/// assert!(verify_file(Path::new("Cargo.toml"), &digest).unwrap());
/// ```
pub fn verify_file(path: &Path, expected: &Digest) -> Result<bool, FulhashError> {
    let actual = hash_file(path, Some(expected.algorithm))?;
    Ok(actual.bytes == expected.bytes)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Create a temporary file with the given contents and return its path.
    fn temp_file(name: &str, contents: &[u8]) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should be after epoch")
            .as_nanos();
        path.push(format!(
            "rsfulmen-fulhash-{}-{}-{}",
            std::process::id(),
            nanos,
            name,
        ));
        std::fs::write(&path, contents).expect("failed to write temp file");
        path
    }

    #[test]
    fn test_hash_xxh3_default() {
        let digest = hash(b"hello world", None).expect("hash should succeed");
        assert_eq!(digest.algorithm, Algorithm::Xxh3_128);
        assert!(!digest.bytes.is_empty(), "digest bytes should not be empty");
        assert_eq!(digest.bytes.len(), 16, "xxh3-128 produces 16-byte digest");
    }

    #[test]
    fn test_hash_sha256() {
        let digest = hash(b"hello world", Some(Algorithm::Sha256)).expect("hash should succeed");
        assert_eq!(digest.algorithm, Algorithm::Sha256);
        assert_eq!(digest.bytes.len(), 32, "SHA-256 produces 32-byte digest");
    }

    #[test]
    fn test_hash_deterministic() {
        let d1 = hash(b"deterministic", None).unwrap();
        let d2 = hash(b"deterministic", None).unwrap();
        assert_eq!(d1, d2, "same input must produce identical digests");

        let d3 = hash(b"deterministic", Some(Algorithm::Sha256)).unwrap();
        let d4 = hash(b"deterministic", Some(Algorithm::Sha256)).unwrap();
        assert_eq!(d3, d4, "same input must produce identical SHA-256 digests");
    }

    #[test]
    fn test_hash_different_inputs_differ() {
        let d1 = hash(b"alpha", None).unwrap();
        let d2 = hash(b"beta", None).unwrap();
        assert_ne!(
            d1.bytes, d2.bytes,
            "different inputs should produce different digests"
        );
    }

    #[test]
    fn test_hash_empty_input() {
        let d_xxh3 = hash(b"", None).unwrap();
        assert_eq!(d_xxh3.algorithm, Algorithm::Xxh3_128);
        assert_eq!(
            d_xxh3.bytes.len(),
            16,
            "empty input still produces 16-byte xxh3 digest"
        );

        let d_sha = hash(b"", Some(Algorithm::Sha256)).unwrap();
        assert_eq!(
            d_sha.bytes.len(),
            32,
            "empty input still produces 32-byte SHA-256 digest"
        );
    }

    #[test]
    fn test_hash_string() {
        let from_str = hash_string("hello world", None).unwrap();
        let from_bytes = hash(b"hello world", None).unwrap();
        assert_eq!(
            from_str, from_bytes,
            "hash_string should match hash of same bytes"
        );

        let from_str_sha = hash_string("hello world", Some(Algorithm::Sha256)).unwrap();
        let from_bytes_sha = hash(b"hello world", Some(Algorithm::Sha256)).unwrap();
        assert_eq!(from_str_sha, from_bytes_sha);
    }

    #[test]
    fn test_hash_reader_matches_block() {
        let data = b"streaming data test";

        let block_digest = hash(data, None).unwrap();
        let reader_digest = hash_reader(Cursor::new(data), None).unwrap();
        assert_eq!(
            block_digest, reader_digest,
            "streaming hash should match block hash"
        );

        let block_sha = hash(data, Some(Algorithm::Sha256)).unwrap();
        let opts = HashOptions {
            algorithm: Algorithm::Sha256,
            ..HashOptions::default()
        };
        let reader_sha = hash_reader(Cursor::new(data), Some(opts)).unwrap();
        assert_eq!(
            block_sha, reader_sha,
            "streaming SHA-256 should match block SHA-256"
        );
    }

    #[test]
    fn test_hash_file() {
        let content = b"file content for hashing";
        let path = temp_file("hash_file_test", content);

        let file_digest = hash_file(&path, None).unwrap();
        let mem_digest = hash(content, None).unwrap();
        assert_eq!(
            file_digest, mem_digest,
            "file hash should match in-memory hash"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_format_digest_xxh3() {
        let digest = hash(b"test", None).unwrap();
        let formatted = format_digest(&digest);
        assert!(
            formatted.starts_with("xxh3-128:"),
            "xxh3 format should start with 'xxh3-128:', got '{formatted}'"
        );
        let hex_part = &formatted["xxh3-128:".len()..];
        assert_eq!(
            hex_part.len(),
            32,
            "xxh3-128 hex should be 32 chars (16 bytes)"
        );
        assert!(
            hex_part
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
            "hex should be lowercase"
        );
    }

    #[test]
    fn test_format_digest_sha256() {
        let digest = hash(b"test", Some(Algorithm::Sha256)).unwrap();
        let formatted = format_digest(&digest);
        assert!(
            formatted.starts_with("sha256:"),
            "sha256 format should start with 'sha256:', got '{formatted}'"
        );
        let hex_part = &formatted["sha256:".len()..];
        assert_eq!(
            hex_part.len(),
            64,
            "sha256 hex should be 64 chars (32 bytes)"
        );
        assert!(
            hex_part
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
            "hex should be lowercase"
        );
    }

    #[test]
    fn test_parse_digest_roundtrip() {
        let original = hash(b"roundtrip test", None).unwrap();
        let formatted = format_digest(&original);
        let parsed = parse_digest(&formatted).expect("parse should succeed");
        assert_eq!(
            parsed, original,
            "parse(format(digest)) should equal original"
        );
    }

    #[test]
    fn test_parse_digest_sha256_roundtrip() {
        let original = hash(b"sha256 roundtrip", Some(Algorithm::Sha256)).unwrap();
        let formatted = format_digest(&original);
        let parsed = parse_digest(&formatted).expect("parse should succeed");
        assert_eq!(
            parsed, original,
            "parse(format(digest)) should equal original for SHA-256"
        );
    }

    #[test]
    fn test_parse_digest_invalid_format() {
        let err = parse_digest("noseparator").expect_err("should fail without colon");
        assert!(
            matches!(err, FulhashError::InvalidDigestFormat(_)),
            "expected InvalidDigestFormat, got {err:?}"
        );

        let err = parse_digest("md5:abcdef").expect_err("should fail with unknown algo");
        assert!(
            matches!(err, FulhashError::UnsupportedAlgorithm(_)),
            "expected UnsupportedAlgorithm, got {err:?}"
        );

        let err = parse_digest("sha256:zzzz").expect_err("should fail with bad hex");
        assert!(
            matches!(err, FulhashError::HexDecode { .. }),
            "expected HexDecode, got {err:?}"
        );
    }

    #[test]
    fn test_parse_digest_rejects_uppercase_hex() {
        // Canonical format requires lowercase hex
        let err =
            parse_digest("sha256:2CF24DBA5FB0A30E26E83B2AC5B9E29E1B161E5C1FA7425E73043362938B9824")
                .expect_err("should reject uppercase hex");
        assert!(
            matches!(err, FulhashError::InvalidDigestFormat(_)),
            "expected InvalidDigestFormat, got {err:?}"
        );
        let msg = err.to_string();
        assert!(
            msg.contains("lowercase"),
            "error should mention lowercase, got: {msg}"
        );
    }

    #[test]
    fn test_parse_digest_rejects_mixed_case_hex() {
        let err =
            parse_digest("sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938B9824")
                .expect_err("should reject mixed-case hex");
        assert!(matches!(err, FulhashError::InvalidDigestFormat(_)));
    }

    #[test]
    fn test_parse_digest_rejects_empty_hex() {
        let err = parse_digest("sha256:").expect_err("should reject empty hex");
        assert!(
            matches!(err, FulhashError::InvalidDigestFormat(_)),
            "expected InvalidDigestFormat, got {err:?}"
        );
        let msg = err.to_string();
        assert!(
            msg.contains("empty"),
            "error should mention empty, got: {msg}"
        );
    }

    #[test]
    fn test_parse_digest_rejects_wrong_length_sha256() {
        // sha256 expects 32 bytes (64 hex chars); provide only 4 bytes (8 hex chars)
        let err = parse_digest("sha256:abcdef01").expect_err("should reject wrong-length sha256");
        assert!(
            matches!(err, FulhashError::InvalidDigestFormat(_)),
            "expected InvalidDigestFormat, got {err:?}"
        );
        let msg = err.to_string();
        assert!(
            msg.contains("32") && msg.contains("4"),
            "error should mention expected vs actual length, got: {msg}"
        );
    }

    #[test]
    fn test_parse_digest_rejects_wrong_length_xxh3() {
        // xxh3-128 expects 16 bytes (32 hex chars); provide 32 bytes (64 hex chars)
        let err = parse_digest(
            "xxh3-128:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824",
        )
        .expect_err("should reject wrong-length xxh3-128");
        assert!(
            matches!(err, FulhashError::InvalidDigestFormat(_)),
            "expected InvalidDigestFormat, got {err:?}"
        );
        let msg = err.to_string();
        assert!(
            msg.contains("16") && msg.contains("32"),
            "error should mention expected vs actual length, got: {msg}"
        );
    }

    #[test]
    fn test_parse_digest_accepts_canonical_xxh3() {
        // Valid xxh3-128: 16 bytes = 32 lowercase hex chars
        let digest = hash(b"test", None).unwrap();
        let formatted = format_digest(&digest);
        let parsed = parse_digest(&formatted).expect("canonical xxh3-128 should parse");
        assert_eq!(parsed, digest);
    }

    #[test]
    fn test_parse_digest_accepts_canonical_sha256() {
        // Valid sha256: 32 bytes = 64 lowercase hex chars
        let digest = hash(b"test", Some(Algorithm::Sha256)).unwrap();
        let formatted = format_digest(&digest);
        let parsed = parse_digest(&formatted).expect("canonical sha256 should parse");
        assert_eq!(parsed, digest);
    }

    #[test]
    fn test_verify_matching() {
        let data = b"verify me";
        let digest = hash(data, None).unwrap();
        assert!(
            verify(data, &digest),
            "verify should return true for matching data"
        );

        let digest_sha = hash(data, Some(Algorithm::Sha256)).unwrap();
        assert!(
            verify(data, &digest_sha),
            "verify should return true for matching SHA-256"
        );
    }

    #[test]
    fn test_verify_mismatched() {
        let digest = hash(b"original", None).unwrap();
        assert!(
            !verify(b"tampered", &digest),
            "verify should return false for mismatched data"
        );
    }

    #[test]
    fn test_verify_file() {
        let content = b"file verification content";
        let path = temp_file("verify_file_test", content);

        let digest = hash(content, None).unwrap();
        let result = verify_file(&path, &digest).expect("verify_file should not error");
        assert!(
            result,
            "verify_file should return true for matching content"
        );

        let wrong_digest = hash(b"wrong content", None).unwrap();
        let result = verify_file(&path, &wrong_digest).expect("verify_file should not error");
        assert!(
            !result,
            "verify_file should return false for mismatched content"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_different_algorithms_different_digests() {
        let data = b"algorithm comparison";
        let xxh3 = hash(data, Some(Algorithm::Xxh3_128)).unwrap();
        let sha = hash(data, Some(Algorithm::Sha256)).unwrap();
        assert_ne!(xxh3.algorithm, sha.algorithm);
        assert_ne!(
            xxh3.bytes, sha.bytes,
            "different algorithms should produce different bytes"
        );
    }

    #[test]
    fn test_display_impl() {
        let digest = hash(b"display test", None).unwrap();
        let display_output = digest.to_string();
        let format_output = format_digest(&digest);
        assert_eq!(
            display_output, format_output,
            "Display impl should match format_digest"
        );

        let sha_digest = hash(b"display test", Some(Algorithm::Sha256)).unwrap();
        assert_eq!(sha_digest.to_string(), format_digest(&sha_digest));
    }
}
