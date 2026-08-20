//! File-backed JSON Schema instance validation (on-disk catalogs, offline `$ref`).

use super::{
    keyword_from_schema_path, parse_schema_bytes, select_draft, SchemaValidationError, Severity,
    ValidationIssue, ValidationSource,
};
use jsonschema::{JSONSchema, SchemaResolver, SchemaResolverError};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use url::Url;

/// How `$id` URLs are matched against files in `ref_dirs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileSchemaResolution {
    /// Prefer `$id` equality, then path-suffix match.
    #[default]
    PreferId,
    /// Match by path suffix only (filename / trailing segments).
    PathOnly,
}

/// Options for file-backed instance validation.
#[derive(Debug, Clone, Default)]
pub struct FileSchemaOptions {
    /// Additional catalog roots searched for referenced schemas. The root schema
    /// file’s directory is always included.
    pub ref_dirs: Vec<PathBuf>,
    /// `$id` vs path-suffix resolution.
    pub resolution: FileSchemaResolution,
}

/// Offline catalog resolver for on-disk JSON Schema trees.
///
/// External `$ref` values are resolved **without network access** against the
/// canonical directory of the root schema file and each
/// [`FileSchemaOptions::ref_dirs`] entry. Traversal that leaves those roots,
/// non-local `file:` hosts, unsupported URI schemes, duplicate `$id`s, and
/// symlink path components are rejected as
/// [`SchemaValidationError::SchemaCompileFailed`].
///
/// On Unix, each component under an allowed root is opened with `openat(2)` and
/// `O_NOFOLLOW` so a swapped intermediate directory cannot escape the catalog.
/// The root schema path and each `ref_dirs` entry must be real (non-symlink)
/// files/directories. On other platforms, catalog roots are treated as trusted
/// for the duration of validation.
pub struct FileBackedResolver {
    allowed_roots: Vec<PathBuf>,
    schema_dir: PathBuf,
    resolution: FileSchemaResolution,
    id_index: HashMap<String, PathBuf>,
    files: Vec<PathBuf>,
}

impl FileBackedResolver {
    /// Build a resolver for a root schema file and catalog options.
    pub fn from_schema_file(
        schema_path: &Path,
        opts: &FileSchemaOptions,
    ) -> Result<Self, SchemaValidationError> {
        let schema_path = require_real_canonical(schema_path)?;
        let schema_dir = schema_path
            .parent()
            .ok_or_else(|| SchemaValidationError::SchemaCompileFailed {
                path: schema_path.display().to_string(),
                message: "schema path has no parent directory".to_string(),
            })?
            .to_path_buf();
        let schema_dir = require_real_canonical(&schema_dir)?;

        let mut allowed_roots = vec![schema_dir.clone()];
        for dir in &opts.ref_dirs {
            allowed_roots.push(require_real_canonical(dir)?);
        }
        allowed_roots.sort();
        allowed_roots.dedup();

        let mut files = Vec::new();
        let mut id_index = HashMap::new();
        for root in &allowed_roots {
            collect_schema_files(root, &mut files)?;
        }
        files.sort();
        files.dedup();
        for file in &files {
            if let Ok(value) = load_value(file) {
                if let Some(id) = value.get("$id").and_then(|v| v.as_str()) {
                    let id = strip_fragment(id).to_string();
                    if let Some(prev) = id_index.insert(id.clone(), file.clone()) {
                        if prev != *file {
                            return Err(SchemaValidationError::SchemaCompileFailed {
                                path: file.display().to_string(),
                                message: format!(
                                    "duplicate schema $id {id} in catalog (also {})",
                                    prev.display()
                                ),
                            });
                        }
                    }
                }
            }
        }

        Ok(Self {
            allowed_roots,
            schema_dir,
            resolution: opts.resolution,
            id_index,
            files,
        })
    }

    fn load_contained(&self, path: &Path) -> Result<Arc<Value>, SchemaResolverError> {
        let abs = lexical_absolute(path).map_err(|e| anyhow::anyhow!(e))?;
        let root = self
            .allowed_roots
            .iter()
            .find(|root| abs.starts_with(root))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "schema path not contained in catalog roots: {}",
                    abs.display()
                )
            })?;
        let value = read_under_root(root, &abs).map_err(|e| anyhow::anyhow!(e))?;
        Ok(Arc::new(value))
    }

    fn preflight(&self, schema: &Value, base: &Url) -> Result<(), SchemaValidationError> {
        let mut visited = HashSet::new();
        self.preflight_walk(schema, base, &mut visited)
    }

    fn preflight_walk(
        &self,
        schema: &Value,
        base: &Url,
        visited: &mut HashSet<String>,
    ) -> Result<(), SchemaValidationError> {
        let mut refs = Vec::new();
        collect_refs(schema, &mut refs);
        for original in refs {
            let stripped = strip_fragment(&original);
            if stripped.is_empty() {
                continue;
            }
            let url = match Url::parse(stripped) {
                Ok(url) => url,
                Err(_) => {
                    base.join(stripped)
                        .map_err(|e| SchemaValidationError::SchemaCompileFailed {
                            path: original.clone(),
                            message: format!("invalid relative $ref {original}: {e}"),
                        })?
                }
            };
            let key = strip_fragment(url.as_str()).to_string();
            if !visited.insert(key) {
                continue;
            }
            let resolved = SchemaResolver::resolve(self, schema, &url, &original).map_err(|e| {
                SchemaValidationError::SchemaCompileFailed {
                    path: original.clone(),
                    message: e.to_string(),
                }
            })?;
            let next_base = resolved
                .get("$id")
                .and_then(|v| v.as_str())
                .and_then(|s| Url::parse(strip_fragment(s)).ok())
                .unwrap_or(url);
            self.preflight_walk(&resolved, &next_base, visited)?;
        }
        Ok(())
    }

    fn lookup_id(&self, url: &Url) -> Option<PathBuf> {
        let key = strip_fragment(url.as_str()).to_string();
        if let Some(path) = self.id_index.get(&key) {
            return Some(path.clone());
        }
        None
    }

    fn lookup_suffix(&self, url: &Url) -> Option<PathBuf> {
        let path = url.path().trim_start_matches('/');
        let file_name = Path::new(path).file_name()?.to_string_lossy();
        let mut suffix_hits: Vec<&PathBuf> = self
            .files
            .iter()
            .filter(|p| p.ends_with(path) || p.file_name().is_some_and(|n| n == file_name.as_ref()))
            .collect();
        suffix_hits.sort();
        suffix_hits.dedup();
        if suffix_hits.len() == 1 {
            Some(suffix_hits[0].clone())
        } else {
            None
        }
    }
}

impl SchemaResolver for FileBackedResolver {
    fn resolve(
        &self,
        _root_schema: &Value,
        url: &Url,
        original_reference: &str,
    ) -> Result<Arc<Value>, SchemaResolverError> {
        match url.scheme() {
            "file" => {
                if url.host_str().is_some()
                    && url.host_str() != Some("")
                    && url.host_str() != Some("localhost")
                {
                    return Err(anyhow::anyhow!(
                        "unsupported non-local file: host in schema URI: {url}"
                    ));
                }
                let path = url
                    .to_file_path()
                    .map_err(|()| anyhow::anyhow!("invalid file:// schema URI: {url}"))?;
                self.load_contained(&path)
            }
            "http" | "https" => {
                let path = match self.resolution {
                    FileSchemaResolution::PreferId => {
                        self.lookup_id(url).or_else(|| self.lookup_suffix(url))
                    }
                    FileSchemaResolution::PathOnly => self.lookup_suffix(url),
                };
                let Some(path) = path else {
                    return Err(anyhow::anyhow!(
                        "schema $id not found in catalog (offline): {url}"
                    ));
                };
                self.load_contained(&path)
            }
            "json-schema" => {
                let rel = original_reference
                    .split('#')
                    .next()
                    .unwrap_or(original_reference);
                if rel.is_empty() {
                    return Err(anyhow::anyhow!(
                        "cannot resolve empty relative schema reference"
                    ));
                }
                let candidate = self.schema_dir.join(rel);
                self.load_contained(&candidate)
            }
            other => Err(anyhow::anyhow!("unsupported schema URI scheme: {other}")),
        }
    }
}

/// Validate an in-memory instance against an in-memory schema using a file-backed resolver.
pub fn validate_instance(
    schema: &Value,
    instance: &Value,
    resolver: FileBackedResolver,
) -> Result<Vec<ValidationIssue>, SchemaValidationError> {
    compile_and_validate(schema, instance, resolver, "<schema>")
}

/// Load a schema file and validate an in-memory instance.
pub fn validate_instance_with_schema_file(
    schema_path: &Path,
    instance: &Value,
    opts: FileSchemaOptions,
) -> Result<Vec<ValidationIssue>, SchemaValidationError> {
    let resolver = FileBackedResolver::from_schema_file(schema_path, &opts)?;
    let mut schema = load_value(schema_path)?;
    ensure_file_id(&mut schema, schema_path)?;
    compile_and_validate(
        &schema,
        instance,
        resolver,
        &schema_path.display().to_string(),
    )
}

/// Load schema and instance files (JSON or YAML).
pub fn validate_instance_file(
    schema_path: &Path,
    data_path: &Path,
    opts: FileSchemaOptions,
) -> Result<Vec<ValidationIssue>, SchemaValidationError> {
    let instance = load_value(data_path).map_err(|e| match e {
        SchemaValidationError::InvalidSchemaJson { message, .. } => {
            SchemaValidationError::InvalidPayload(message)
        }
        other => other,
    })?;
    validate_instance_with_schema_file(schema_path, &instance, opts)
}

/// Load a schema file and validate JSON/YAML instance bytes.
pub fn validate_instance_bytes(
    schema_path: &Path,
    bytes: &[u8],
    opts: FileSchemaOptions,
) -> Result<Vec<ValidationIssue>, SchemaValidationError> {
    let instance = parse_schema_bytes("<instance>", bytes).map_err(|e| match e {
        SchemaValidationError::InvalidSchemaJson { message, .. } => {
            SchemaValidationError::InvalidPayload(message)
        }
        other => other,
    })?;
    validate_instance_with_schema_file(schema_path, &instance, opts)
}

fn compile_and_validate(
    schema: &Value,
    instance: &Value,
    resolver: FileBackedResolver,
    schema_label: &str,
) -> Result<Vec<ValidationIssue>, SchemaValidationError> {
    let base = schema
        .get("$id")
        .and_then(|v| v.as_str())
        .and_then(|s| Url::parse(strip_fragment(s)).ok())
        .unwrap_or_else(|| {
            Url::from_directory_path(&resolver.schema_dir)
                .unwrap_or_else(|_| Url::parse("file:///").expect("static file URL"))
        });
    resolver.preflight(schema, &base)?;

    let compiled = JSONSchema::options()
        .with_draft(select_draft(schema))
        .with_resolver(resolver)
        .compile(schema)
        .map_err(|e| SchemaValidationError::SchemaCompileFailed {
            path: schema_label.to_string(),
            message: e.to_string(),
        })?;

    let mut issues = Vec::new();
    if let Err(errors) = compiled.validate(instance) {
        for error in errors {
            let schema_path = error.schema_path.to_string();
            issues.push(ValidationIssue {
                pointer: error.instance_path.to_string(),
                message: error.to_string(),
                keyword: keyword_from_schema_path(&schema_path),
                severity: Severity::Error,
                source: ValidationSource::Native,
            });
        }
    }
    // jsonschema 0.17 reports resolver failures as instance issues. The helper
    // contract treats catalog/URI failures as compile errors.
    if let Some(issue) = issues.iter().find(|i| is_resolver_failure(&i.message)) {
        return Err(SchemaValidationError::SchemaCompileFailed {
            path: schema_label.to_string(),
            message: issue.message.clone(),
        });
    }
    Ok(issues)
}

fn is_resolver_failure(message: &str) -> bool {
    message.contains("failed to resolve")
        || message.contains("not contained in catalog roots")
        || message.contains("unsupported schema URI")
        || message.contains("unsupported non-local file:")
}

fn ensure_file_id(schema: &mut Value, schema_path: &Path) -> Result<(), SchemaValidationError> {
    if schema.get("$id").and_then(|v| v.as_str()).is_some() {
        return Ok(());
    }
    let canon = canonicalize_existing(schema_path)?;
    let url =
        Url::from_file_path(&canon).map_err(|()| SchemaValidationError::SchemaCompileFailed {
            path: canon.display().to_string(),
            message: "cannot form file:// $id for schema path".to_string(),
        })?;
    if let Value::Object(map) = schema {
        map.insert("$id".to_string(), Value::String(url.to_string()));
    }
    Ok(())
}

fn load_value(path: &Path) -> Result<Value, SchemaValidationError> {
    load_value_stable(path)
}

fn load_value_stable(path: &Path) -> Result<Value, SchemaValidationError> {
    let bytes = read_stable(path).map_err(|e| SchemaValidationError::InvalidSchemaJson {
        path: path.display().to_string(),
        message: e.to_string(),
    })?;
    parse_schema_bytes(&path.display().to_string(), &bytes)
}

fn read_stable(path: &Path) -> std::io::Result<Vec<u8>> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut opts = fs::OpenOptions::new();
        opts.read(true);
        // O_NOFOLLOW: Linux 0o400000, Darwin/BSD 0x100. Closes the
        // canonicalize-then-open symlink swap on the final path.
        #[cfg(any(target_os = "linux", target_os = "android"))]
        opts.custom_flags(0o400000);
        #[cfg(any(
            target_os = "macos",
            target_os = "ios",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        opts.custom_flags(0x0000_0100);
        let mut file = opts.open(path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Ok(buf)
    }
    #[cfg(not(unix))]
    {
        fs::read(path)
    }
}

fn collect_refs(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(r)) = map.get("$ref") {
                out.push(r.clone());
            }
            for v in map.values() {
                collect_refs(v, out);
            }
        }
        Value::Array(items) => {
            for v in items {
                collect_refs(v, out);
            }
        }
        _ => {}
    }
}

fn collect_schema_files(root: &Path, out: &mut Vec<PathBuf>) -> Result<(), SchemaValidationError> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries =
            fs::read_dir(&dir).map_err(|e| SchemaValidationError::SchemaCompileFailed {
                path: dir.display().to_string(),
                message: e.to_string(),
            })?;
        for entry in entries {
            let entry = entry.map_err(|e| SchemaValidationError::SchemaCompileFailed {
                path: dir.display().to_string(),
                message: e.to_string(),
            })?;
            let path = entry.path();
            let file_type =
                entry
                    .file_type()
                    .map_err(|e| SchemaValidationError::SchemaCompileFailed {
                        path: path.display().to_string(),
                        message: e.to_string(),
                    })?;
            if file_type.is_dir() {
                stack.push(path);
            } else if file_type.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(".json") || name.ends_with(".yaml") || name.ends_with(".yml")
                    {
                        if let Ok(canon) = canonicalize_existing(&path) {
                            out.push(canon);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn canonicalize_existing(path: &Path) -> Result<PathBuf, SchemaValidationError> {
    fs::canonicalize(path).map_err(|e| SchemaValidationError::SchemaCompileFailed {
        path: path.display().to_string(),
        message: e.to_string(),
    })
}

fn require_real_canonical(path: &Path) -> Result<PathBuf, SchemaValidationError> {
    let meta =
        fs::symlink_metadata(path).map_err(|e| SchemaValidationError::SchemaCompileFailed {
            path: path.display().to_string(),
            message: e.to_string(),
        })?;
    if meta.file_type().is_symlink() {
        return Err(SchemaValidationError::SchemaCompileFailed {
            path: path.display().to_string(),
            message: format!("catalog path is a symlink: {}", path.display()),
        });
    }
    canonicalize_existing(path)
}

fn lexical_absolute(path: &Path) -> Result<PathBuf, String> {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(path)
    };
    Ok(normalize_lexical(&abs))
}

fn normalize_lexical(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            std::path::Component::Prefix(p) => out.push(p.as_os_str()),
            std::path::Component::RootDir => out.push(std::path::MAIN_SEPARATOR_STR),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                let _ = out.pop();
            }
            std::path::Component::Normal(s) => out.push(s),
        }
    }
    out
}

fn read_under_root(root: &Path, abs: &Path) -> Result<Value, String> {
    let rel = abs.strip_prefix(root).map_err(|_| {
        format!(
            "schema path not contained in catalog roots: {}",
            abs.display()
        )
    })?;
    #[cfg(unix)]
    {
        let mut file = openat_nofollow(root, rel)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).map_err(|e| e.to_string())?;
        parse_schema_bytes(&abs.display().to_string(), &buf).map_err(|e| e.to_string())
    }
    #[cfg(not(unix))]
    {
        let bytes = fs::read(abs).map_err(|e| e.to_string())?;
        parse_schema_bytes(&abs.display().to_string(), &bytes).map_err(|e| e.to_string())
    }
}

#[cfg(unix)]
fn openat_nofollow(root: &Path, rel: &Path) -> Result<fs::File, String> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::io::{AsRawFd, FromRawFd, OwnedFd};

    if rel.as_os_str().is_empty() {
        return Err("schema path resolved to a catalog root directory".to_string());
    }

    let root_c = CString::new(root.as_os_str().as_bytes())
        .map_err(|_| "catalog root path contains interior NUL".to_string())?;
    let root_fd = unsafe {
        libc::open(
            root_c.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if root_fd < 0 {
        return Err(format!(
            "cannot open catalog root {}: {}",
            root.display(),
            std::io::Error::last_os_error()
        ));
    }
    let mut current = unsafe { OwnedFd::from_raw_fd(root_fd) };

    let comps: Vec<_> = rel.components().collect();
    for (i, component) in comps.iter().enumerate() {
        let std::path::Component::Normal(name) = component else {
            return Err(format!(
                "schema path not contained in catalog roots: {}",
                rel.display()
            ));
        };
        let name_c = CString::new(name.as_bytes())
            .map_err(|_| "schema path component contains interior NUL".to_string())?;
        let last = i + 1 == comps.len();
        let mut flags = libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC;
        if !last {
            flags |= libc::O_DIRECTORY;
        }
        let next = unsafe { libc::openat(current.as_raw_fd(), name_c.as_ptr(), flags) };
        if next < 0 {
            return Err(format!(
                "schema path not contained in catalog roots (nofollow): {}/{}: {}",
                root.display(),
                rel.display(),
                std::io::Error::last_os_error()
            ));
        }
        current = unsafe { OwnedFd::from_raw_fd(next) };
    }
    Ok(fs::File::from(current))
}

fn strip_fragment(s: &str) -> &str {
    s.split('#').next().unwrap_or(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_SEQ: AtomicU64 = AtomicU64::new(0);

    fn scratch_dir() -> PathBuf {
        let n = TEST_SEQ.fetch_add(1, Ordering::SeqCst);
        let dir =
            std::env::temp_dir().join(format!("rsfulmen-file-schema-{}-{}", std::process::id(), n));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_catalog(dir: &Path) {
        fs::write(
            dir.join("root.schema.json"),
            r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://schemas.example.test/catalog/root.schema.json",
  "type": "object",
  "additionalProperties": false,
  "required": ["name", "widget"],
  "properties": {
    "name": { "type": "string" },
    "widget": { "$ref": "widget.schema.json" }
  }
}
"#,
        )
        .unwrap();
        fs::write(
            dir.join("widget.schema.json"),
            r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://schemas.example.test/catalog/widget.schema.json",
  "type": "object",
  "additionalProperties": false,
  "required": ["kind"],
  "properties": {
    "kind": { "const": "ok" }
  }
}
"#,
        )
        .unwrap();
    }

    fn opts(dir: &Path) -> FileSchemaOptions {
        FileSchemaOptions {
            ref_dirs: vec![dir.to_path_buf()],
            resolution: FileSchemaResolution::PreferId,
        }
    }

    fn good_instance() -> Value {
        json!({"name": "n", "widget": {"kind": "ok"}})
    }

    #[test]
    fn conforming_instance_has_no_issues() {
        let dir = scratch_dir();
        write_catalog(&dir);
        let issues = validate_instance_with_schema_file(
            &dir.join("root.schema.json"),
            &good_instance(),
            opts(&dir),
        )
        .unwrap();
        assert!(issues.is_empty(), "{issues:?}");
    }

    #[test]
    fn extra_property_reports_additional_properties() {
        let dir = scratch_dir();
        write_catalog(&dir);
        let instance = json!({"name": "n", "widget": {"kind": "ok"}, "nope": true});
        let issues = validate_instance_with_schema_file(
            &dir.join("root.schema.json"),
            &instance,
            opts(&dir),
        )
        .unwrap();
        assert!(!issues.is_empty());
        assert!(
            issues
                .iter()
                .any(|i| i.keyword.as_deref() == Some("additionalProperties")),
            "{issues:?}"
        );
    }

    #[test]
    fn missing_required_reports_issues() {
        let dir = scratch_dir();
        write_catalog(&dir);
        let instance = json!({"name": "n"});
        let issues = validate_instance_with_schema_file(
            &dir.join("root.schema.json"),
            &instance,
            opts(&dir),
        )
        .unwrap();
        assert!(!issues.is_empty(), "{issues:?}");
    }

    #[test]
    fn sibling_ref_via_https_id() {
        let dir = scratch_dir();
        write_catalog(&dir);
        let issues = validate_instance_with_schema_file(
            &dir.join("root.schema.json"),
            &json!({"name": "n", "widget": {"kind": "nope"}}),
            opts(&dir),
        )
        .unwrap();
        assert!(!issues.is_empty(), "{issues:?}");
    }

    #[test]
    fn traversal_escape_is_compile_failure() {
        let dir = scratch_dir();
        write_catalog(&dir);
        let outside_name = format!("rsfulmen-outside-{}.schema.json", std::process::id());
        fs::write(
            dir.join("evil.schema.json"),
            format!(
                r#"{{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {{
    "x": {{ "$ref": "../{outside_name}" }}
  }}
}}
"#
            ),
        )
        .unwrap();
        let outside = dir.parent().unwrap().join(&outside_name);
        fs::write(&outside, r#"{"type":"string"}"#).unwrap();
        let err = validate_instance_with_schema_file(
            &dir.join("evil.schema.json"),
            &json!({"x": "a"}),
            opts(&dir),
        )
        .unwrap_err();
        let _ = fs::remove_file(&outside);
        match err {
            SchemaValidationError::SchemaCompileFailed { message, .. } => {
                assert!(
                    message.contains("not contained")
                        || message.contains("unsupported")
                        || message.contains("canonicalize"),
                    "{message}"
                );
            }
            other => panic!("expected compile failure, got {other:?}"),
        }
    }

    #[test]
    fn unsupported_scheme_is_compile_failure() {
        let dir = scratch_dir();
        fs::write(
            dir.join("ftp.schema.json"),
            r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "x": { "$ref": "ftp://example.test/x.schema.json" }
  }
}
"#,
        )
        .unwrap();
        let err = validate_instance_with_schema_file(
            &dir.join("ftp.schema.json"),
            &json!({"x": {}}),
            opts(&dir),
        )
        .unwrap_err();
        match err {
            SchemaValidationError::SchemaCompileFailed { message, .. } => {
                assert!(
                    message.contains("unsupported schema URI scheme") || message.contains("ftp"),
                    "{message}"
                );
            }
            other => panic!("expected compile failure, got {other:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn symlink_escape_is_compile_failure() {
        let dir = scratch_dir();
        write_catalog(&dir);
        let outside = dir.parent().unwrap().join(format!(
            "rsfulmen-symlink-target-{}.schema.json",
            std::process::id()
        ));
        fs::write(&outside, r#"{"type":"string"}"#).unwrap();
        let link = dir.join("escape.schema.json");
        std::os::unix::fs::symlink(&outside, &link).unwrap();
        fs::write(
            dir.join("uses-link.schema.json"),
            r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "x": { "$ref": "escape.schema.json" }
  }
}
"#,
        )
        .unwrap();
        let err = validate_instance_with_schema_file(
            &dir.join("uses-link.schema.json"),
            &json!({"x": "a"}),
            opts(&dir),
        )
        .unwrap_err();
        let _ = fs::remove_file(&outside);
        match err {
            SchemaValidationError::SchemaCompileFailed { message, .. } => {
                assert!(
                    message.contains("not contained") || message.contains("canonicalize"),
                    "{message}"
                );
            }
            other => panic!("expected compile failure, got {other:?}"),
        }
    }

    #[test]
    fn omitted_optional_traversal_ref_is_compile_failure() {
        let dir = scratch_dir();
        let outside_name = format!("rsfulmen-opt-outside-{}.schema.json", std::process::id());
        fs::write(
            dir.join("optional-escape.schema.json"),
            format!(
                r#"{{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {{
    "x": {{ "$ref": "../{outside_name}" }}
  }}
}}
"#
            ),
        )
        .unwrap();
        let outside = dir.parent().unwrap().join(&outside_name);
        fs::write(&outside, r#"{"type":"string"}"#).unwrap();
        let err = validate_instance_with_schema_file(
            &dir.join("optional-escape.schema.json"),
            &json!({}),
            opts(&dir),
        )
        .unwrap_err();
        let _ = fs::remove_file(&outside);
        match err {
            SchemaValidationError::SchemaCompileFailed { message, .. } => {
                assert!(
                    message.contains("not contained") || message.contains("canonicalize"),
                    "{message}"
                );
            }
            other => panic!("expected compile failure, got {other:?}"),
        }
    }

    #[test]
    fn omitted_optional_bad_ref_is_compile_failure() {
        let dir = scratch_dir();
        fs::write(
            dir.join("optional-ftp.schema.json"),
            r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "x": { "$ref": "ftp://example.test/x.schema.json" }
  }
}
"#,
        )
        .unwrap();
        let err = validate_instance_with_schema_file(
            &dir.join("optional-ftp.schema.json"),
            &json!({}),
            opts(&dir),
        )
        .unwrap_err();
        match err {
            SchemaValidationError::SchemaCompileFailed { message, .. } => {
                assert!(
                    message.contains("unsupported schema URI scheme") || message.contains("ftp"),
                    "{message}"
                );
            }
            other => panic!("expected compile failure, got {other:?}"),
        }
    }

    #[test]
    fn local_file_ref_succeeds() {
        let dir = scratch_dir();
        write_catalog(&dir);
        let widget = fs::canonicalize(dir.join("widget.schema.json")).unwrap();
        let file_url = Url::from_file_path(&widget).unwrap();
        fs::write(
            dir.join("file-ref.schema.json"),
            format!(
                r#"{{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "additionalProperties": false,
  "required": ["widget"],
  "properties": {{
    "widget": {{ "$ref": "{file_url}" }}
  }}
}}
"#
            ),
        )
        .unwrap();
        let issues = validate_instance_with_schema_file(
            &dir.join("file-ref.schema.json"),
            &json!({"widget": {"kind": "ok"}}),
            opts(&dir),
        )
        .unwrap();
        assert!(issues.is_empty(), "{issues:?}");
    }

    #[test]
    fn non_local_file_host_is_compile_failure() {
        let dir = scratch_dir();
        fs::write(
            dir.join("remote-file.schema.json"),
            r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "x": { "$ref": "file://example.test/tmp/x.schema.json" }
  }
}
"#,
        )
        .unwrap();
        let err = validate_instance_with_schema_file(
            &dir.join("remote-file.schema.json"),
            &json!({}),
            opts(&dir),
        )
        .unwrap_err();
        match err {
            SchemaValidationError::SchemaCompileFailed { message, .. } => {
                assert!(
                    message.contains("non-local file") || message.contains("unsupported"),
                    "{message}"
                );
            }
            other => panic!("expected compile failure, got {other:?}"),
        }
    }

    #[test]
    fn path_only_resolves_by_suffix() {
        let dir = scratch_dir();
        write_catalog(&dir);
        let issues = validate_instance_with_schema_file(
            &dir.join("root.schema.json"),
            &good_instance(),
            FileSchemaOptions {
                ref_dirs: vec![dir.to_path_buf()],
                resolution: FileSchemaResolution::PathOnly,
            },
        )
        .unwrap();
        assert!(issues.is_empty(), "{issues:?}");
    }

    #[test]
    fn duplicate_id_is_compile_failure() {
        let dir = scratch_dir();
        write_catalog(&dir);
        fs::write(
            dir.join("dup.schema.json"),
            r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://schemas.example.test/catalog/widget.schema.json",
  "type": "string"
}
"#,
        )
        .unwrap();
        let err = validate_instance_with_schema_file(
            &dir.join("root.schema.json"),
            &good_instance(),
            opts(&dir),
        )
        .unwrap_err();
        match err {
            SchemaValidationError::SchemaCompileFailed { message, .. } => {
                assert!(message.contains("duplicate schema $id"), "{message}");
            }
            other => panic!("expected compile failure, got {other:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn catalog_root_symlink_swap_is_compile_failure() {
        let dir = scratch_dir();
        write_catalog(&dir);
        let schema_path = dir.join("root.schema.json");
        let resolver = FileBackedResolver::from_schema_file(&schema_path, &opts(&dir)).unwrap();
        let schema: Value = serde_json::from_slice(&fs::read(&schema_path).unwrap()).unwrap();
        let canon = fs::canonicalize(&dir).unwrap();
        let backup = canon.with_file_name(format!(
            "{}-bak",
            canon.file_name().unwrap().to_string_lossy()
        ));
        let outside = dir
            .parent()
            .unwrap()
            .join(format!("rsfulmen-root-swap-{}-out", std::process::id()));
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("widget.schema.json"), r#"{"type":"number"}"#).unwrap();
        fs::rename(&canon, &backup).unwrap();
        std::os::unix::fs::symlink(&outside, &canon).unwrap();
        let err = validate_instance(&schema, &json!({}), resolver).unwrap_err();
        let _ = fs::remove_file(&canon);
        let _ = fs::rename(&backup, &canon);
        let _ = fs::remove_dir_all(&outside);
        match err {
            SchemaValidationError::SchemaCompileFailed { message, .. } => {
                assert!(
                    message.contains("nofollow")
                        || message.contains("not contained")
                        || message.contains("cannot open catalog root"),
                    "{message}"
                );
            }
            other => panic!("expected compile failure, got {other:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn parent_directory_symlink_swap_is_compile_failure() {
        let dir = scratch_dir();
        let sub = dir.join("sub");
        fs::create_dir(&sub).unwrap();
        fs::write(
            sub.join("leaf.schema.json"),
            r#"{"$schema":"https://json-schema.org/draft/2020-12/schema","type":"string"}"#,
        )
        .unwrap();
        fs::write(
            dir.join("root.schema.json"),
            r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "x": { "$ref": "sub/leaf.schema.json" }
  }
}
"#,
        )
        .unwrap();
        let outside_dir = dir
            .parent()
            .unwrap()
            .join(format!("rsfulmen-parent-swap-{}-out", std::process::id()));
        fs::create_dir_all(outside_dir.join("sub")).unwrap();
        fs::write(
            outside_dir.join("sub/leaf.schema.json"),
            r#"{"type":"number"}"#,
        )
        .unwrap();
        fs::remove_dir_all(&sub).unwrap();
        std::os::unix::fs::symlink(outside_dir.join("sub"), &sub).unwrap();
        let err = validate_instance_with_schema_file(
            &dir.join("root.schema.json"),
            &json!({}),
            opts(&dir),
        )
        .unwrap_err();
        let _ = fs::remove_dir_all(&outside_dir);
        match err {
            SchemaValidationError::SchemaCompileFailed { message, .. } => {
                assert!(
                    message.contains("nofollow") || message.contains("not contained"),
                    "{message}"
                );
            }
            other => panic!("expected compile failure, got {other:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn nofollow_read_rejects_symlink_swap() {
        let dir = scratch_dir();
        let inside = dir.join("inside.schema.json");
        fs::write(&inside, r#"{"type":"string"}"#).unwrap();
        let canon = fs::canonicalize(&inside).unwrap();
        let outside = dir.parent().unwrap().join(format!(
            "rsfulmen-nofollow-outside-{}.schema.json",
            std::process::id()
        ));
        fs::write(&outside, r#"{"type":"number"}"#).unwrap();
        fs::remove_file(&inside).unwrap();
        std::os::unix::fs::symlink(&outside, &inside).unwrap();
        let err = read_stable(&canon);
        let _ = fs::remove_file(&outside);
        assert!(
            err.is_err(),
            "expected O_NOFOLLOW to reject swapped symlink"
        );
    }

    #[cfg(unix)]
    #[test]
    fn schema_file_symlink_to_outside_is_compile_failure() {
        let dir = scratch_dir();
        write_catalog(&dir);
        let outside = dir.parent().unwrap().join(format!(
            "rsfulmen-schema-link-{}.schema.json",
            std::process::id()
        ));
        fs::write(
            &outside,
            r#"{"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object"}"#,
        )
        .unwrap();
        let link = dir.join("via-link.schema.json");
        std::os::unix::fs::symlink(&outside, &link).unwrap();
        let err = validate_instance_with_schema_file(&link, &json!({}), opts(&dir)).unwrap_err();
        let _ = fs::remove_file(&outside);
        match err {
            SchemaValidationError::SchemaCompileFailed { message, .. } => {
                assert!(
                    message.contains("symlink") || message.contains("nofollow"),
                    "{message}"
                );
            }
            other => panic!("expected compile failure, got {other:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn ref_dir_symlink_to_outside_is_compile_failure() {
        let dir = scratch_dir();
        write_catalog(&dir);
        let outside = dir
            .parent()
            .unwrap()
            .join(format!("rsfulmen-refdir-{}", std::process::id()));
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("extra.schema.json"), r#"{"type":"string"}"#).unwrap();
        let link = dir.join("ref-link");
        std::os::unix::fs::symlink(&outside, &link).unwrap();
        let err = validate_instance_with_schema_file(
            &dir.join("root.schema.json"),
            &good_instance(),
            FileSchemaOptions {
                ref_dirs: vec![link],
                resolution: FileSchemaResolution::PreferId,
            },
        )
        .unwrap_err();
        let _ = fs::remove_dir_all(&outside);
        match err {
            SchemaValidationError::SchemaCompileFailed { message, .. } => {
                assert!(message.contains("symlink"), "{message}");
            }
            other => panic!("expected compile failure, got {other:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn nested_ancestor_directory_symlink_swap_is_compile_failure() {
        let dir = scratch_dir();
        let nested = dir.join("a").join("b");
        fs::create_dir_all(&nested).unwrap();
        fs::write(
            nested.join("leaf.schema.json"),
            r#"{"$schema":"https://json-schema.org/draft/2020-12/schema","type":"string"}"#,
        )
        .unwrap();
        fs::write(
            dir.join("root.schema.json"),
            r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "x": { "$ref": "a/b/leaf.schema.json" }
  }
}
"#,
        )
        .unwrap();
        let outside = dir
            .parent()
            .unwrap()
            .join(format!("rsfulmen-nested-swap-{}", std::process::id()));
        fs::create_dir_all(outside.join("a").join("b")).unwrap();
        fs::write(outside.join("a/b/leaf.schema.json"), r#"{"type":"number"}"#).unwrap();
        fs::remove_dir_all(dir.join("a")).unwrap();
        std::os::unix::fs::symlink(outside.join("a"), dir.join("a")).unwrap();
        let err = validate_instance_with_schema_file(
            &dir.join("root.schema.json"),
            &json!({}),
            opts(&dir),
        )
        .unwrap_err();
        let _ = fs::remove_dir_all(&outside);
        match err {
            SchemaValidationError::SchemaCompileFailed { message, .. } => {
                assert!(
                    message.contains("nofollow") || message.contains("not contained"),
                    "{message}"
                );
            }
            other => panic!("expected compile failure, got {other:?}"),
        }
    }

    #[test]
    fn file_url_dotdot_staying_inside_succeeds() {
        let dir = scratch_dir();
        write_catalog(&dir);
        let widget = fs::canonicalize(dir.join("widget.schema.json")).unwrap();
        let nested = widget.parent().unwrap().join("sub");
        let via = Url::from_file_path(nested.join("..").join("widget.schema.json")).unwrap();
        fs::write(
            dir.join("dotdot-in.schema.json"),
            format!(
                r#"{{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "additionalProperties": false,
  "required": ["widget"],
  "properties": {{
    "widget": {{ "$ref": "{via}" }}
  }}
}}
"#
            ),
        )
        .unwrap();
        let issues = validate_instance_with_schema_file(
            &dir.join("dotdot-in.schema.json"),
            &json!({"widget": {"kind": "ok"}}),
            opts(&dir),
        )
        .unwrap();
        assert!(issues.is_empty(), "{issues:?}");
    }

    #[test]
    fn file_url_dotdot_escape_is_compile_failure() {
        let dir = scratch_dir();
        write_catalog(&dir);
        let outside = dir.parent().unwrap().join(format!(
            "rsfulmen-fileurl-escape-{}.schema.json",
            std::process::id()
        ));
        fs::write(&outside, r#"{"type":"string"}"#).unwrap();
        let via = Url::from_file_path(&outside).unwrap();
        fs::write(
            dir.join("dotdot-out.schema.json"),
            format!(
                r#"{{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {{
    "x": {{ "$ref": "{via}" }}
  }}
}}
"#
            ),
        )
        .unwrap();
        let err = validate_instance_with_schema_file(
            &dir.join("dotdot-out.schema.json"),
            &json!({}),
            opts(&dir),
        )
        .unwrap_err();
        let _ = fs::remove_file(&outside);
        match err {
            SchemaValidationError::SchemaCompileFailed { message, .. } => {
                assert!(
                    message.contains("not contained") || message.contains("nofollow"),
                    "{message}"
                );
            }
            other => panic!("expected compile failure, got {other:?}"),
        }
    }

    #[test]
    fn cyclic_refs_do_not_hang() {
        let dir = scratch_dir();
        fs::write(
            dir.join("a.schema.json"),
            r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://schemas.example.test/cycle/a.schema.json",
  "type": "object",
  "properties": { "b": { "$ref": "b.schema.json" } }
}
"#,
        )
        .unwrap();
        fs::write(
            dir.join("b.schema.json"),
            r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://schemas.example.test/cycle/b.schema.json",
  "type": "object",
  "properties": { "a": { "$ref": "a.schema.json" } }
}
"#,
        )
        .unwrap();
        let issues =
            validate_instance_with_schema_file(&dir.join("a.schema.json"), &json!({}), opts(&dir))
                .unwrap();
        assert!(issues.is_empty(), "{issues:?}");
    }

    #[cfg(unix)]
    #[test]
    fn ref_to_in_catalog_symlink_is_compile_failure() {
        let dir = scratch_dir();
        write_catalog(&dir);
        let link = dir.join("alias.schema.json");
        std::os::unix::fs::symlink(dir.join("widget.schema.json"), &link).unwrap();
        fs::write(
            dir.join("uses-alias.schema.json"),
            r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "x": { "$ref": "alias.schema.json" }
  }
}
"#,
        )
        .unwrap();
        let err = validate_instance_with_schema_file(
            &dir.join("uses-alias.schema.json"),
            &json!({}),
            opts(&dir),
        )
        .unwrap_err();
        match err {
            SchemaValidationError::SchemaCompileFailed { message, .. } => {
                assert!(
                    message.contains("nofollow") || message.contains("symlink"),
                    "{message}"
                );
            }
            other => panic!("expected compile failure, got {other:?}"),
        }
    }
}
