//! Canonical hashing and checksums.
//!
//! This module provides a uniform API for computing and verifying content
//! hashes using xxHash3-128 (default), SHA-256, CRC-32, or CRC-32C.
//!
//! Digests are formatted with a canonical `algorithm:hex` prefix so they
//! can be persisted, compared, and exchanged with other Fulmen SDK
//! implementations (e.g., gofulmen's `FormatDigest`).
//!
//! **Authenticity:** CRC-32, CRC-32C, and xxHash3-128 detect accidental
//! corruption and are not collision-resistant. They must not be used as
//! proof that an attacker did not alter the bytes. For adversarial inputs
//! use SHA-256 together with an authenticated expected digest, a signature,
//! or a MAC.
//!
//! ```
//! use rsfulmen::fulhash::{hash, verify, format_digest, Algorithm};
//!
//! let digest = hash(b"hello world", Some(Algorithm::Sha256)).unwrap();
//! assert!(verify(b"hello world", &digest));
//! assert!(format_digest(&digest).starts_with("sha256:"));
//! ```
//!
//! # Algorithms
//!
//! | Name       | Variant               | Digest size | Use-case                     |
//! |------------|-----------------------|-------------|------------------------------|
//! | xxh3-128   | `Algorithm::Xxh3_128` | 16 bytes    | Fast content-addressable IDs |
//! | SHA-256    | `Algorithm::Sha256`   | 32 bytes    | Cryptographic integrity      |
//! | crc32      | `Algorithm::Crc32`    | 4 bytes     | ZIP/GZIP/PNG interoperability |
//! | crc32c     | `Algorithm::Crc32c`   | 4 bytes     | Castagnoli / cloud storage    |

use crc::{Crc, CRC_32_ISCSI, CRC_32_ISO_HDLC};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::Digest as ShaDigest;
use std::collections::BTreeMap;
use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// IEEE CRC-32 (ISO-HDLC / ZIP / GZIP / PNG).
const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);
/// Castagnoli CRC-32C (iSCSI / GCS / SSE4.2).
const CRC32C: Crc<u32> = Crc::<u32>::new(&CRC_32_ISCSI);

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
///
/// This enum is `#[non_exhaustive]`: the FulHash catalog can grow. Downstream
/// `match` arms must include a wildcard. Serde **emits** the canonical
/// catalog ids (`xxh3-128`, `sha256`, `crc32`, `crc32c`) and **accepts**
/// the v0.2.1 derived tags (`Xxh3_128`, `Sha256`) as deserialize aliases.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[non_exhaustive]
pub enum Algorithm {
    /// xxHash3-128 — fast, non-cryptographic (default).
    #[default]
    #[serde(rename = "xxh3-128", alias = "Xxh3_128")]
    Xxh3_128,
    /// SHA-256 — cryptographic.
    #[serde(rename = "sha256", alias = "Sha256")]
    Sha256,
    /// CRC-32 (IEEE / ISO-HDLC) — archive interoperability only.
    #[serde(rename = "crc32")]
    Crc32,
    /// CRC-32C (Castagnoli) — cloud / hardware-accelerated checksums.
    #[serde(rename = "crc32c")]
    Crc32c,
}

impl Algorithm {
    /// Returns the canonical string tag used in the digest format.
    fn tag(self) -> &'static str {
        match self {
            Self::Xxh3_128 => "xxh3-128",
            Self::Sha256 => "sha256",
            Self::Crc32 => "crc32",
            Self::Crc32c => "crc32c",
        }
    }

    /// Parse an algorithm from its canonical tag.
    fn from_tag(s: &str) -> Result<Self, FulhashError> {
        match s {
            "xxh3-128" => Ok(Self::Xxh3_128),
            "sha256" => Ok(Self::Sha256),
            "crc32" => Ok(Self::Crc32),
            "crc32c" => Ok(Self::Crc32c),
            other => Err(FulhashError::UnsupportedAlgorithm(other.to_string())),
        }
    }

    /// Expected digest byte length for this algorithm.
    fn digest_len(self) -> usize {
        match self {
            Self::Xxh3_128 => 16,
            Self::Sha256 => 32,
            Self::Crc32 | Self::Crc32c => 4,
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
/// to round-trip it back. Construct with [`Self::new`] or the hash APIs;
/// serde rejects a byte length that does not match the algorithm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Digest {
    algorithm: Algorithm,
    bytes: Vec<u8>,
}

impl Digest {
    /// Build a digest after checking the algorithm's expected length.
    pub fn new(algorithm: Algorithm, bytes: Vec<u8>) -> Result<Self, FulhashError> {
        let expected = algorithm.digest_len();
        if bytes.len() != expected {
            return Err(FulhashError::InvalidDigestFormat(format!(
                "expected {} bytes for {}, got {}",
                expected,
                algorithm.tag(),
                bytes.len()
            )));
        }
        Ok(Self { algorithm, bytes })
    }

    fn from_computed(algorithm: Algorithm, bytes: Vec<u8>) -> Self {
        debug_assert_eq!(bytes.len(), algorithm.digest_len());
        Self { algorithm, bytes }
    }

    /// Algorithm that produced these bytes.
    pub fn algorithm(&self) -> Algorithm {
        self.algorithm
    }

    /// Raw digest bytes. Length is algorithm-specific.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl Serialize for Digest {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Wire<'a> {
            algorithm: Algorithm,
            bytes: &'a [u8],
        }
        Wire {
            algorithm: self.algorithm,
            bytes: &self.bytes,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Digest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Wire {
            algorithm: Algorithm,
            bytes: Vec<u8>,
        }
        let wire = Wire::deserialize(deserializer)?;
        Digest::new(wire.algorithm, wire.bytes).map_err(serde::de::Error::custom)
    }
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
    #[error("unsupported algorithm: {0} (supported algorithms: xxh3-128, sha256, crc32, crc32c)")]
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

fn read_retrying<R: Read>(reader: &mut R, buf: &mut [u8]) -> std::io::Result<usize> {
    loop {
        match reader.read(buf) {
            Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
            other => return other,
        }
    }
}

/// Streaming hash using xxHash3-128.
fn hash_xxh3_128_reader<R: Read>(reader: R, buffer_size: usize) -> Result<Vec<u8>, std::io::Error> {
    use xxhash_rust::xxh3::Xxh3;

    let mut hasher = Xxh3::new();
    let mut reader = reader;
    let mut buf = vec![0u8; buffer_size];
    loop {
        let n = read_retrying(&mut reader, &mut buf)?;
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
        let n = read_retrying(&mut reader, &mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize().to_vec())
}

fn hash_crc32(data: &[u8]) -> Vec<u8> {
    CRC32.checksum(data).to_be_bytes().to_vec()
}

fn hash_crc32c(data: &[u8]) -> Vec<u8> {
    CRC32C.checksum(data).to_be_bytes().to_vec()
}

fn hash_crc32_reader<R: Read>(reader: R, buffer_size: usize) -> Result<Vec<u8>, std::io::Error> {
    crc32_reader(reader, buffer_size, &CRC32)
}

fn hash_crc32c_reader<R: Read>(reader: R, buffer_size: usize) -> Result<Vec<u8>, std::io::Error> {
    crc32_reader(reader, buffer_size, &CRC32C)
}

fn crc32_reader<R: Read>(
    mut reader: R,
    buffer_size: usize,
    algo: &Crc<u32>,
) -> Result<Vec<u8>, std::io::Error> {
    let mut digest = algo.digest();
    let mut buf = vec![0u8; buffer_size];
    loop {
        let n = read_retrying(&mut reader, &mut buf)?;
        if n == 0 {
            break;
        }
        digest.update(&buf[..n]);
    }
    Ok(digest.finalize().to_be_bytes().to_vec())
}

/// Incremental hasher used by [`multi_hash_reader`].
enum RunningHasher {
    Xxh3(Box<xxhash_rust::xxh3::Xxh3>),
    Sha256(sha2::Sha256),
    Crc32(crc::Digest<'static, u32>),
    Crc32c(crc::Digest<'static, u32>),
}

impl RunningHasher {
    fn new(algorithm: Algorithm) -> Self {
        match algorithm {
            Algorithm::Xxh3_128 => Self::Xxh3(Box::new(xxhash_rust::xxh3::Xxh3::new())),
            Algorithm::Sha256 => Self::Sha256(sha2::Sha256::new()),
            Algorithm::Crc32 => Self::Crc32(CRC32.digest()),
            Algorithm::Crc32c => Self::Crc32c(CRC32C.digest()),
        }
    }

    fn update(&mut self, data: &[u8]) {
        match self {
            Self::Xxh3(h) => h.update(data),
            Self::Sha256(h) => h.update(data),
            Self::Crc32(h) | Self::Crc32c(h) => h.update(data),
        }
    }

    fn finalize(self) -> Vec<u8> {
        match self {
            Self::Xxh3(h) => h.digest128().to_be_bytes().to_vec(),
            Self::Sha256(h) => h.finalize().to_vec(),
            Self::Crc32(h) | Self::Crc32c(h) => h.finalize().to_be_bytes().to_vec(),
        }
    }
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
/// assert_eq!(digest.algorithm(), Algorithm::Xxh3_128);
///
/// let sha = hash(b"hello", Some(Algorithm::Sha256)).unwrap();
/// assert_eq!(sha.bytes().len(), 32);
/// ```
pub fn hash(data: &[u8], algorithm: Option<Algorithm>) -> Result<Digest, FulhashError> {
    let algo = algorithm.unwrap_or_default();
    let bytes = match algo {
        Algorithm::Xxh3_128 => hash_xxh3_128(data),
        Algorithm::Sha256 => hash_sha256(data),
        Algorithm::Crc32 => hash_crc32(data),
        Algorithm::Crc32c => hash_crc32c(data),
    };
    Ok(Digest::from_computed(algo, bytes))
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
        Algorithm::Crc32 => hash_crc32_reader(reader, buffer_size)?,
        Algorithm::Crc32c => hash_crc32c_reader(reader, buffer_size)?,
    };
    Ok(Digest::from_computed(opts.algorithm, bytes))
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
/// assert_eq!(digest.algorithm(), Algorithm::Sha256);
/// ```
pub fn parse_digest(s: &str) -> Result<Digest, FulhashError> {
    let Some((algo_str, hex_str)) = s.split_once(':') else {
        return Err(FulhashError::InvalidDigestFormat(format!(
            "missing ':' separator in '{s}' (expected format algorithm:hex)"
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

    Digest::new(algorithm, bytes)
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

/// Hash `data` with each requested algorithm in a single pass.
///
/// Algorithms are deduplicated. An empty slice returns an empty map.
/// Duplicate algorithms collapse to one entry.
///
/// # Examples
///
/// ```
/// use rsfulmen::fulhash::{multi_hash, Algorithm};
///
/// let digests = multi_hash(b"content", &[Algorithm::Xxh3_128, Algorithm::Sha256]).unwrap();
/// assert_eq!(digests.len(), 2);
/// ```
pub fn multi_hash(
    data: &[u8],
    algorithms: &[Algorithm],
) -> Result<BTreeMap<Algorithm, Digest>, FulhashError> {
    multi_hash_reader(std::io::Cursor::new(data), algorithms)
}

/// Hash a UTF-8 string with each requested algorithm in a single pass.
pub fn multi_hash_string(
    s: &str,
    algorithms: &[Algorithm],
) -> Result<BTreeMap<Algorithm, Digest>, FulhashError> {
    multi_hash(s.as_bytes(), algorithms)
}

/// Hash a reader with each requested algorithm in a single pass.
///
/// The reader is consumed once. Empty `algorithms` returns an empty map
/// without reading.
pub fn multi_hash_reader<R: Read>(
    mut reader: R,
    algorithms: &[Algorithm],
) -> Result<BTreeMap<Algorithm, Digest>, FulhashError> {
    if algorithms.is_empty() {
        return Ok(BTreeMap::new());
    }

    let mut unique = BTreeMap::new();
    for algorithm in algorithms {
        unique.insert(*algorithm, RunningHasher::new(*algorithm));
    }

    let mut buf = vec![0u8; DEFAULT_BUFFER_SIZE];
    loop {
        let n = read_retrying(&mut reader, &mut buf)?;
        if n == 0 {
            break;
        }
        for hasher in unique.values_mut() {
            hasher.update(&buf[..n]);
        }
    }

    let mut out = BTreeMap::new();
    for (algorithm, hasher) in unique {
        out.insert(
            algorithm,
            Digest::from_computed(algorithm, hasher.finalize()),
        );
    }
    Ok(out)
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

    fn crc_hex(digest: &Digest) -> String {
        hex::encode(&digest.bytes)
    }

    #[test]
    fn test_crc32_known_vectors() {
        // ISO-HDLC / IEEE check value for "123456789" is 0xCBF43926.
        let empty = hash(b"", Some(Algorithm::Crc32)).unwrap();
        assert_eq!(empty.bytes.len(), 4);
        assert_eq!(crc_hex(&empty), "00000000");

        let check = hash(b"123456789", Some(Algorithm::Crc32)).unwrap();
        assert_eq!(crc_hex(&check), "cbf43926");
        assert_eq!(format_digest(&check), "crc32:cbf43926");
    }

    #[test]
    fn test_crc32c_known_vectors() {
        // Castagnoli check value for "123456789" is 0xE3069283.
        let empty = hash(b"", Some(Algorithm::Crc32c)).unwrap();
        assert_eq!(crc_hex(&empty), "00000000");

        let check = hash(b"123456789", Some(Algorithm::Crc32c)).unwrap();
        assert_eq!(crc_hex(&check), "e3069283");
        assert_eq!(format_digest(&check), "crc32c:e3069283");
    }

    #[test]
    fn test_crc_streaming_matches_block() {
        let data = b"Hello, World!";
        for algo in [Algorithm::Crc32, Algorithm::Crc32c] {
            let block = hash(data, Some(algo)).unwrap();
            let opts = HashOptions {
                algorithm: algo,
                buffer_size: 3,
            };
            let streamed = hash_reader(Cursor::new(data), Some(opts)).unwrap();
            assert_eq!(block, streamed, "{algo:?} streaming must match block");
        }
    }

    #[test]
    fn test_crc_parse_roundtrip_and_length() {
        let digest = hash(b"payload", Some(Algorithm::Crc32)).unwrap();
        let parsed = parse_digest(&format_digest(&digest)).unwrap();
        assert_eq!(parsed, digest);

        let err = parse_digest("crc32:cbf439").expect_err("wrong length");
        assert!(matches!(err, FulhashError::InvalidDigestFormat(_)));
    }

    #[test]
    fn test_multi_hash_matches_individual_and_dedups() {
        let data = b"multi-hash payload";
        let algos = [
            Algorithm::Xxh3_128,
            Algorithm::Sha256,
            Algorithm::Crc32,
            Algorithm::Crc32c,
            Algorithm::Sha256,
        ];
        let many = multi_hash(data, &algos).unwrap();
        assert_eq!(many.len(), 4, "duplicates must collapse");
        for algo in [
            Algorithm::Xxh3_128,
            Algorithm::Sha256,
            Algorithm::Crc32,
            Algorithm::Crc32c,
        ] {
            assert_eq!(many.get(&algo).unwrap(), &hash(data, Some(algo)).unwrap());
        }
        assert!(multi_hash(data, &[]).unwrap().is_empty());
        assert_eq!(
            multi_hash_string("multi-hash payload", &[Algorithm::Crc32]).unwrap()
                [&Algorithm::Crc32],
            hash(data, Some(Algorithm::Crc32)).unwrap()
        );
    }

    #[test]
    fn test_multi_hash_reader_is_single_pass() {
        let data = b"single-pass reader body";
        struct CountRead<'a> {
            data: &'a [u8],
            pos: usize,
            bytes: usize,
        }
        impl Read for CountRead<'_> {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                let n = (self.data.len() - self.pos).min(buf.len());
                buf[..n].copy_from_slice(&self.data[self.pos..self.pos + n]);
                self.pos += n;
                self.bytes += n;
                Ok(n)
            }
        }

        let mut reader = CountRead {
            data,
            pos: 0,
            bytes: 0,
        };
        let out = multi_hash_reader(
            &mut reader,
            &[
                Algorithm::Xxh3_128,
                Algorithm::Sha256,
                Algorithm::Crc32,
                Algorithm::Crc32c,
            ],
        )
        .unwrap();
        assert_eq!(reader.bytes, data.len(), "reader must be consumed once");
        assert_eq!(
            out[&Algorithm::Sha256],
            hash(data, Some(Algorithm::Sha256)).unwrap()
        );
        assert_eq!(
            out[&Algorithm::Crc32c],
            hash(data, Some(Algorithm::Crc32c)).unwrap()
        );
    }

    #[test]
    fn test_v0_2_1_algorithm_tags_deserialize_and_emit_canonical() {
        let legacy = "algorithm: Sha256\nbytes: [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32]\n";
        let digest: Digest = serde_yaml::from_str(legacy).expect("v0.2.1 Sha256 tag");
        assert_eq!(digest.algorithm, Algorithm::Sha256);
        let emitted = serde_yaml::to_string(&digest).unwrap();
        assert!(emitted.contains("sha256"), "{emitted}");
        assert!(!emitted.contains("Sha256"), "{emitted}");

        let xxh: Algorithm = serde_yaml::from_str("Xxh3_128\n").unwrap();
        assert_eq!(xxh, Algorithm::Xxh3_128);
        assert_eq!(serde_yaml::to_string(&xxh).unwrap().trim(), "xxh3-128");
    }

    #[test]
    fn test_digest_serde_rejects_wrong_length() {
        let oversized = "algorithm: crc32\nbytes: [0,1,2,3,4]\n";
        let err = serde_yaml::from_str::<Digest>(oversized).unwrap_err();
        assert!(err.to_string().contains("expected 4 bytes"), "{err}");
        let huge = vec![0u8; 1024];
        let err = Digest::new(Algorithm::Crc32, huge).unwrap_err();
        assert!(matches!(err, FulhashError::InvalidDigestFormat(_)));
    }

    #[test]
    fn test_read_retries_interrupted() {
        struct OnceInterrupted {
            interrupted: bool,
            yielded: bool,
        }
        impl Read for OnceInterrupted {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                if !self.interrupted {
                    self.interrupted = true;
                    return Err(std::io::Error::from(std::io::ErrorKind::Interrupted));
                }
                if self.yielded {
                    return Ok(0);
                }
                self.yielded = true;
                buf[..1].copy_from_slice(b"a");
                Ok(1)
            }
        }
        let digest = hash_reader(
            &mut OnceInterrupted {
                interrupted: false,
                yielded: false,
            },
            None,
        )
        .unwrap();
        assert_eq!(digest, hash(b"a", None).unwrap());
    }

    #[test]
    fn test_algorithm_serde_uses_canonical_tags() {
        let cases = [
            (Algorithm::Xxh3_128, "xxh3-128"),
            (Algorithm::Sha256, "sha256"),
            (Algorithm::Crc32, "crc32"),
            (Algorithm::Crc32c, "crc32c"),
        ];
        for (algo, tag) in cases {
            let encoded = serde_yaml::to_string(&algo).unwrap();
            assert!(
                encoded.contains(tag),
                "serde must emit catalog tag {tag}, got {encoded}"
            );
            let back: Algorithm = serde_yaml::from_str(&encoded).unwrap();
            assert_eq!(back, algo);
            let rust_name = format!("{algo:?}");
            assert!(
                !encoded.contains(&rust_name),
                "serde must not emit the Rust variant name {rust_name}"
            );
        }
    }

    #[test]
    fn test_multi_hash_reader_short_reads_empty_and_partial_error() {
        struct ShortRead<'a> {
            data: &'a [u8],
            pos: usize,
        }
        impl Read for ShortRead<'_> {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                if self.pos >= self.data.len() {
                    return Ok(0);
                }
                let n = 1.min(buf.len());
                buf[..n].copy_from_slice(&self.data[self.pos..self.pos + n]);
                self.pos += n;
                Ok(n)
            }
        }

        struct PartialThenError {
            remaining: usize,
        }
        impl Read for PartialThenError {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                if self.remaining == 0 {
                    return Err(std::io::Error::other("injected mid-stream failure"));
                }
                let n = 3.min(self.remaining).min(buf.len());
                buf[..n].fill(b'x');
                self.remaining -= n;
                Ok(n)
            }
        }

        let data = b"short-read body";
        let algos = [
            Algorithm::Xxh3_128,
            Algorithm::Sha256,
            Algorithm::Crc32,
            Algorithm::Crc32c,
        ];
        let short = multi_hash_reader(&mut ShortRead { data, pos: 0 }, &algos).unwrap();
        let expected = multi_hash(data, &algos).unwrap();
        assert_eq!(short, expected);

        let empty = multi_hash_reader(std::io::empty(), &algos).unwrap();
        assert_eq!(empty, multi_hash(b"", &algos).unwrap());

        let err = multi_hash_reader(&mut PartialThenError { remaining: 8 }, &algos)
            .expect_err("partial digest must not be returned as success");
        assert!(matches!(err, FulhashError::IoError { .. }), "{err:?}");
    }

    #[test]
    fn test_multi_hash_reader_large_generated_stream() {
        let chunk = vec![0xABu8; 8192];
        let mut data = Vec::new();
        for _ in 0..12 {
            data.extend_from_slice(&chunk);
        }
        assert!(data.len() > DEFAULT_BUFFER_SIZE);

        struct SliceRead<'a> {
            data: &'a [u8],
            pos: usize,
        }
        impl Read for SliceRead<'_> {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                let n = (self.data.len() - self.pos).min(1024).min(buf.len());
                buf[..n].copy_from_slice(&self.data[self.pos..self.pos + n]);
                self.pos += n;
                Ok(n)
            }
        }

        let algos = [
            Algorithm::Xxh3_128,
            Algorithm::Sha256,
            Algorithm::Crc32,
            Algorithm::Crc32c,
        ];
        let streamed = multi_hash_reader(
            &mut SliceRead {
                data: &data,
                pos: 0,
            },
            &algos,
        )
        .unwrap();
        assert_eq!(streamed, multi_hash(&data, &algos).unwrap());
    }

    #[derive(serde::Deserialize)]
    #[allow(dead_code)]
    struct FulhashFixtureFile {
        fixtures: Vec<ScalarFixture>,
        streaming_fixtures: Vec<StreamFixture>,
        error_fixtures: Vec<ErrorFixture>,
        format_fixtures: Vec<FormatFixture>,
        #[serde(default)]
        #[allow(dead_code)]
        version: Option<String>,
        #[serde(default)]
        #[allow(dead_code)]
        description: Option<String>,
        #[serde(default)]
        #[allow(dead_code)]
        notes: Option<String>,
        #[serde(rename = "$schema", default)]
        #[allow(dead_code)]
        schema: Option<String>,
    }

    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[allow(dead_code)]
    struct ScalarFixture {
        name: String,
        #[serde(default)]
        #[allow(dead_code)]
        description: Option<String>,
        #[serde(default)]
        input: Option<String>,
        #[serde(default)]
        #[allow(dead_code)]
        encoding: Option<String>,
        #[serde(default)]
        input_bytes: Option<Vec<u8>>,
        #[serde(default)]
        xxh3_128: Option<String>,
        #[serde(default)]
        sha256: Option<String>,
        #[serde(default)]
        crc32: Option<String>,
        #[serde(default)]
        crc32c: Option<String>,
        #[serde(default)]
        #[allow(dead_code)]
        notes: Option<String>,
    }

    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[allow(dead_code)]
    struct StreamFixture {
        name: String,
        #[serde(default)]
        #[allow(dead_code)]
        description: Option<String>,
        chunks: Vec<StreamChunk>,
        #[serde(default)]
        expected_xxh3_128: Option<String>,
        #[serde(default)]
        expected_sha256: Option<String>,
        #[serde(default)]
        #[allow(dead_code)]
        notes: Option<String>,
    }

    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[allow(dead_code)]
    struct StreamChunk {
        #[serde(default)]
        value: Option<String>,
        #[serde(default)]
        #[allow(dead_code)]
        encoding: Option<String>,
        #[serde(default)]
        size: Option<usize>,
        #[serde(default)]
        pattern: Option<String>,
    }

    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[allow(dead_code)]
    struct ErrorFixture {
        name: String,
        #[serde(default)]
        #[allow(dead_code)]
        input: Option<String>,
        #[serde(default)]
        algorithm: Option<String>,
        #[serde(default)]
        checksum: Option<String>,
        #[serde(default)]
        expected_error: Option<String>,
        #[serde(default)]
        error_message_contains: Vec<String>,
    }

    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct FormatFixture {
        name: String,
        #[serde(default)]
        algorithm: Option<String>,
        #[serde(default)]
        hex: Option<String>,
        #[serde(default)]
        expected_formatted: Option<String>,
        #[serde(default)]
        formatted: Option<String>,
        #[serde(default)]
        expected_algorithm: Option<String>,
        #[serde(default)]
        expected_hex: Option<String>,
    }

    fn load_fulhash_fixtures() -> FulhashFixtureFile {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("config/crucible-rs/library/fulhash/fixtures.yaml");
        let raw = std::fs::read_to_string(&path).expect("read fixtures.yaml");
        serde_yaml::from_str(&raw).expect("parse fixtures.yaml")
    }

    fn fixture_bytes(fx: &ScalarFixture) -> Vec<u8> {
        if let Some(bytes) = &fx.input_bytes {
            bytes.clone()
        } else {
            fx.input.as_deref().unwrap_or("").as_bytes().to_vec()
        }
    }

    fn stream_chunk_bytes(chunks: &[StreamChunk]) -> Vec<Vec<u8>> {
        chunks
            .iter()
            .map(|chunk| {
                if let Some(value) = &chunk.value {
                    return value.as_bytes().to_vec();
                }
                let size = chunk.size.expect("generated chunk needs size");
                let fill = match chunk.pattern.as_deref() {
                    Some("repeating-A") => b'A',
                    Some("repeating-B") => b'B',
                    Some("repeating-C") => b'C',
                    other => panic!("unsupported stream pattern {other:?}"),
                };
                vec![fill; size]
            })
            .collect()
    }

    fn expand_stream_chunks(chunks: &[StreamChunk]) -> Vec<u8> {
        stream_chunk_bytes(chunks).into_iter().flatten().collect()
    }

    struct ExactChunks {
        chunks: Vec<Vec<u8>>,
        index: usize,
    }

    impl Read for ExactChunks {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            if self.index >= self.chunks.len() {
                return Ok(0);
            }
            let chunk = &self.chunks[self.index];
            let n = chunk.len().min(buf.len());
            buf[..n].copy_from_slice(&chunk[..n]);
            if n == chunk.len() {
                self.index += 1;
            } else {
                self.chunks[self.index] = chunk[n..].to_vec();
            }
            Ok(n)
        }
    }

    fn assert_expected_error(err: &FulhashError, expected: &str, name: &str) {
        match expected {
            "UnsupportedAlgorithmError" => assert!(
                matches!(err, FulhashError::UnsupportedAlgorithm(_)),
                "{name}: expected UnsupportedAlgorithm, got {err:?}"
            ),
            "InvalidChecksumFormatError" => assert!(
                matches!(err, FulhashError::InvalidDigestFormat(_)),
                "{name}: expected InvalidDigestFormat, got {err:?}"
            ),
            other => panic!("{name}: unknown expected_error {other}"),
        }
    }

    #[test]
    fn test_crucible_fixtures_all_scalar_streaming_error_and_format() {
        let file = load_fulhash_fixtures();
        assert!(!file.fixtures.is_empty());
        assert!(!file.streaming_fixtures.is_empty());

        for fx in &file.fixtures {
            let data = fixture_bytes(fx);
            let pairs = [
                (fx.xxh3_128.as_deref(), Algorithm::Xxh3_128),
                (fx.sha256.as_deref(), Algorithm::Sha256),
                (fx.crc32.as_deref(), Algorithm::Crc32),
                (fx.crc32c.as_deref(), Algorithm::Crc32c),
            ];
            let present: Vec<_> = pairs.iter().filter(|(v, _)| v.is_some()).collect();
            assert!(
                !present.is_empty(),
                "fixture {} has no algorithm results",
                fx.name
            );
            for (expected, algo) in pairs {
                if let Some(expected) = expected {
                    assert_eq!(
                        format_digest(&hash(&data, Some(algo)).unwrap()),
                        expected,
                        "scalar fixture {} {:?}",
                        fx.name,
                        algo
                    );
                }
            }
        }

        for fx in &file.streaming_fixtures {
            let data = expand_stream_chunks(&fx.chunks);
            let chunk_list = stream_chunk_bytes(&fx.chunks);
            let pairs = [
                (fx.expected_xxh3_128.as_deref(), Algorithm::Xxh3_128),
                (fx.expected_sha256.as_deref(), Algorithm::Sha256),
            ];
            for (expected, algo) in pairs {
                let Some(expected) = expected else {
                    continue;
                };
                let block = format_digest(&hash(&data, Some(algo)).unwrap());
                assert_eq!(
                    block, expected,
                    "stream fixture {} block {:?}",
                    fx.name, algo
                );

                let streamed = hash_reader(
                    ExactChunks {
                        chunks: chunk_list.clone(),
                        index: 0,
                    },
                    Some(HashOptions {
                        algorithm: algo,
                        buffer_size: 64,
                    }),
                )
                .unwrap();
                assert_eq!(
                    format_digest(&streamed),
                    expected,
                    "stream fixture {} hash_reader {:?}",
                    fx.name,
                    algo
                );
            }

            let mut reader = ExactChunks {
                chunks: chunk_list,
                index: 0,
            };
            let multi =
                multi_hash_reader(&mut reader, &[Algorithm::Xxh3_128, Algorithm::Sha256]).unwrap();
            if let Some(expected) = &fx.expected_xxh3_128 {
                assert_eq!(
                    format_digest(&multi[&Algorithm::Xxh3_128]),
                    expected.as_str(),
                    "stream fixture {} multi_hash_reader xxh3",
                    fx.name
                );
            }
            if let Some(expected) = &fx.expected_sha256 {
                assert_eq!(
                    format_digest(&multi[&Algorithm::Sha256]),
                    expected.as_str(),
                    "stream fixture {} multi_hash_reader sha256",
                    fx.name
                );
            }
        }

        for fx in &file.error_fixtures {
            if let Some(algo) = &fx.algorithm {
                let err = parse_digest(&format!("{algo}:00")).expect_err(&fx.name);
                if let Some(expected) = &fx.expected_error {
                    assert_expected_error(&err, expected, &fx.name);
                }
                let msg = err.to_string();
                for needle in &fx.error_message_contains {
                    assert!(
                        msg.contains(needle),
                        "{} missing {needle:?} in {msg}",
                        fx.name
                    );
                }
            }
            if let Some(checksum) = &fx.checksum {
                let err = parse_digest(checksum).expect_err(&fx.name);
                if let Some(expected) = &fx.expected_error {
                    assert_expected_error(&err, expected, &fx.name);
                }
                let msg = err.to_string();
                for needle in &fx.error_message_contains {
                    assert!(
                        msg.contains(needle),
                        "{} missing {needle:?} in {msg}",
                        fx.name
                    );
                }
            }
        }

        for fx in &file.format_fixtures {
            if let (Some(algo), Some(hex), Some(formatted)) =
                (&fx.algorithm, &fx.hex, &fx.expected_formatted)
            {
                let digest = parse_digest(&format!("{algo}:{hex}")).expect(&fx.name);
                assert_eq!(format_digest(&digest), formatted.as_str(), "{}", fx.name);
            }
            if let (Some(formatted), Some(algo), Some(hex)) =
                (&fx.formatted, &fx.expected_algorithm, &fx.expected_hex)
            {
                let digest = parse_digest(formatted).expect(&fx.name);
                assert_eq!(digest.algorithm.tag(), algo.as_str(), "{}", fx.name);
                assert_eq!(&format_digest(&digest)[algo.len() + 1..], hex.as_str());
            }
        }
    }

    #[test]
    fn test_unsupported_algorithm_lists_supported() {
        let err = parse_digest("md5:abcdef").expect_err("md5 is unsupported");
        let msg = err.to_string();
        assert!(msg.contains("md5"), "{msg}");
        assert!(msg.contains("supported algorithms"), "{msg}");
        assert!(msg.contains("crc32"), "{msg}");
        assert!(msg.contains("crc32c"), "{msg}");
    }
}
