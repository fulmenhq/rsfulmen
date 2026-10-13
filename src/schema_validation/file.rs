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
/// files/directories. On Windows, containment compares paths after stripping
/// `\\?\` verbatim prefixes and folding ASCII case so `file://` URLs match
/// `fs::canonicalize` roots. Intermediate reparse points (junctions, directory
/// symlinks) under a catalog root are opened from a trusted parent handle
/// without following reparse points, then rejected. On other
/// platforms, catalog roots are treated as trusted for the duration of
/// validation.
pub struct FileBackedResolver {
    allowed_roots: Vec<PathBuf>,
    schema_dir: PathBuf,
    resolution: FileSchemaResolution,
    id_index: HashMap<String, PathBuf>,
    files: Vec<PathBuf>,
    root_schema: PathBuf,
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
            #[cfg(all(windows, test))]
            run_before_index_read_hook();
            // Containment/reparse failures must abort the index. Parse
            // errors on incidental `.json`/`.yaml` files are skipped.
            let value = match load_catalog_file(&allowed_roots, file) {
                Ok(value) => value,
                Err(err) if catalog_read_must_fail(&err) => return Err(err),
                Err(_) => continue,
            };
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

        Ok(Self {
            allowed_roots,
            schema_dir,
            resolution: opts.resolution,
            id_index,
            files,
            root_schema: schema_path,
        })
    }

    fn load_contained(&self, path: &Path) -> Result<Arc<Value>, SchemaResolverError> {
        let abs = lexical_absolute(path).map_err(|e| anyhow::anyhow!(e))?;
        let root = self
            .allowed_roots
            .iter()
            .find(|root| path_is_under_root(&abs, root))
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
    #[cfg(all(windows, test))]
    run_before_root_schema_read_hook();
    let mut schema = load_catalog_file(&resolver.allowed_roots, &resolver.root_schema)?;
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

fn catalog_read_must_fail(err: &SchemaValidationError) -> bool {
    let SchemaValidationError::SchemaCompileFailed { message, .. } = err else {
        return false;
    };
    message.contains("reparse")
        || message.contains("not contained")
        || message.contains("not found")
        || message.contains("cannot open")
        || message.contains("symlink")
        || message.contains("nofollow")
}

fn load_catalog_file(roots: &[PathBuf], file: &Path) -> Result<Value, SchemaValidationError> {
    let root = roots.iter().find(|root| path_is_under_root(file, root));
    let Some(root) = root else {
        return Err(SchemaValidationError::SchemaCompileFailed {
            path: file.display().to_string(),
            message: format!(
                "schema path not contained in catalog roots: {}",
                file.display()
            ),
        });
    };
    read_under_root(root, file).map_err(|message| SchemaValidationError::SchemaCompileFailed {
        path: file.display().to_string(),
        message,
    })
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
        // Some hosted filesystems ignore O_NOFOLLOW. Reject a trailing
        // symlink first so a canonicalize-then-swap cannot be read.
        let meta = fs::symlink_metadata(path)?;
        if meta.file_type().is_symlink() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("refusing to read symlink: {}", path.display()),
            ));
        }
        let mut opts = fs::OpenOptions::new();
        opts.read(true);
        opts.custom_flags(libc::O_NOFOLLOW);
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

fn path_is_under_root(abs: &Path, root: &Path) -> bool {
    #[cfg(windows)]
    {
        comparable_windows_path(abs).starts_with(comparable_windows_path(root))
    }
    #[cfg(not(windows))]
    {
        abs.starts_with(root)
    }
}

/// Strip `\\?\` / `\\?\UNC\` prefixes and fold ASCII case so `file://`
/// paths (`C:\...`) match `fs::canonicalize` roots (`\\?\C:\...`).
/// Compiled on every target so Windows-only type errors fail Linux CI.
#[cfg_attr(not(windows), allow(dead_code))]
fn comparable_windows_path(path: &Path) -> PathBuf {
    use std::path::{Component, Prefix};
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => match prefix.kind() {
                Prefix::VerbatimDisk(disk) | Prefix::Disk(disk) => {
                    out.push(format!("{}:", disk.to_ascii_uppercase() as char));
                }
                Prefix::VerbatimUNC(server, share) | Prefix::UNC(server, share) => {
                    out.push(r"\\");
                    out.push(server.to_string_lossy().to_ascii_lowercase());
                    out.push(share.to_string_lossy().to_ascii_lowercase());
                }
                Prefix::Verbatim(rest) | Prefix::DeviceNS(rest) => {
                    out.push(rest.to_string_lossy().to_ascii_lowercase());
                }
            },
            Component::RootDir => out.push(std::path::MAIN_SEPARATOR_STR),
            Component::CurDir => {}
            Component::ParentDir => {
                let _ = out.pop();
            }
            Component::Normal(part) => {
                out.push(part.to_string_lossy().to_ascii_lowercase());
            }
        }
    }
    out
}

fn read_under_root(root: &Path, abs: &Path) -> Result<Value, String> {
    if !path_is_under_root(abs, root) {
        return Err(format!(
            "schema path not contained in catalog roots: {}",
            abs.display()
        ));
    }
    #[cfg(unix)]
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
    #[cfg(windows)]
    {
        let mut file = open_nofollow_under_root(root, abs)?;
        #[cfg(test)]
        run_windows_after_open_hook();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).map_err(|e| e.to_string())?;
        parse_schema_bytes(&abs.display().to_string(), &buf).map_err(|e| e.to_string())
    }
    #[cfg(all(not(unix), not(windows)))]
    {
        let bytes = fs::read(abs).map_err(|e| e.to_string())?;
        parse_schema_bytes(&abs.display().to_string(), &bytes).map_err(|e| e.to_string())
    }
}

#[cfg(windows)]
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;

#[cfg(all(windows, test))]
thread_local! {
    static AFTER_NOFOLLOW_OPEN: std::cell::RefCell<Option<Box<dyn Fn()>>> =
        std::cell::RefCell::new(None);
    static BEFORE_INDEX_READ: std::cell::RefCell<Option<Box<dyn Fn()>>> =
        std::cell::RefCell::new(None);
    static BEFORE_ROOT_SCHEMA_READ: std::cell::RefCell<Option<Box<dyn Fn()>>> =
        std::cell::RefCell::new(None);
}

#[cfg(all(windows, test))]
fn run_windows_after_open_hook() {
    AFTER_NOFOLLOW_OPEN.with(|hook| {
        if let Some(run) = hook.borrow().as_ref() {
            run();
        }
    });
}

#[cfg(all(windows, test))]
fn run_before_index_read_hook() {
    BEFORE_INDEX_READ.with(|hook| {
        if let Some(run) = hook.borrow().as_ref() {
            run();
        }
    });
}

#[cfg(all(windows, test))]
fn run_before_root_schema_read_hook() {
    BEFORE_ROOT_SCHEMA_READ.with(|hook| {
        if let Some(run) = hook.borrow().as_ref() {
            run();
        }
    });
}

/// Open `abs` by walking from `root` with handle-relative, no-reparse
/// `NtCreateFile` so a junction swap between metadata and read cannot be
/// followed. Intermediate `CreateFileW` flags only apply to the last
/// component; the parent handle is the trust root for each hop.
#[cfg(windows)]
fn open_nofollow_under_root(root: &Path, abs: &Path) -> Result<fs::File, String> {
    use std::os::windows::io::{FromRawHandle, IntoRawHandle};

    let rel = windows_rel_components(root, abs)?;
    if rel.is_empty() {
        return Err("schema path resolved to a catalog root directory".to_string());
    }
    let mut parent = open_win_directory(root)
        .map_err(|e| format!("cannot open catalog root {}: {e}", root.display()))?;
    if handle_is_reparse(&parent)? {
        return Err(format!(
            "catalog path is a reparse point: {}",
            root.display()
        ));
    }
    for (i, name) in rel.iter().enumerate() {
        let last = i + 1 == rel.len();
        let child = open_relative_nofollow(&parent, name, !last)?;
        if handle_is_reparse(&child)? {
            return Err(format!(
                "catalog path is a reparse point: {}",
                name.to_string_lossy()
            ));
        }
        parent = child;
    }
    Ok(unsafe { fs::File::from_raw_handle(parent.into_raw_handle()) })
}

#[cfg(windows)]
fn windows_rel_components(root: &Path, abs: &Path) -> Result<Vec<std::ffi::OsString>, String> {
    use std::path::Component;
    if let Ok(rel) = abs.strip_prefix(root) {
        return rel
            .components()
            .map(|c| match c {
                Component::Normal(name) => Ok(name.to_os_string()),
                _ => Err(format!(
                    "schema path not contained in catalog roots: {}",
                    abs.display()
                )),
            })
            .collect();
    }
    let root_cmp = comparable_windows_path(root);
    let abs_cmp = comparable_windows_path(abs);
    let rel_cmp = abs_cmp.strip_prefix(&root_cmp).map_err(|_| {
        format!(
            "schema path not contained in catalog roots: {}",
            abs.display()
        )
    })?;
    let rel_n = rel_cmp.components().count();
    let names: Vec<_> = abs
        .components()
        .filter_map(|c| match c {
            Component::Normal(name) => Some(name.to_os_string()),
            _ => None,
        })
        .collect();
    if names.len() < rel_n {
        return Err(format!(
            "schema path not contained in catalog roots: {}",
            abs.display()
        ));
    }
    Ok(names[names.len() - rel_n..].to_vec())
}

#[cfg(windows)]
fn open_win_directory(path: &Path) -> std::io::Result<std::os::windows::io::OwnedHandle> {
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::io::{FromRawHandle, OwnedHandle};

    let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    wide.push(0);
    const GENERIC_READ: u32 = 0x8000_0000;
    const FILE_SHARE_READ: u32 = 0x1;
    const FILE_SHARE_WRITE: u32 = 0x2;
    const FILE_SHARE_DELETE: u32 = 0x4;
    const OPEN_EXISTING: u32 = 3;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            GENERIC_READ,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            std::ptr::null_mut(),
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
            std::ptr::null_mut(),
        )
    };
    if is_invalid_handle(handle) {
        return Err(std::io::Error::last_os_error());
    }
    Ok(unsafe { OwnedHandle::from_raw_handle(handle) })
}

#[cfg(windows)]
fn open_relative_nofollow(
    parent: &std::os::windows::io::OwnedHandle,
    name: &std::ffi::OsStr,
    directory: bool,
) -> Result<std::os::windows::io::OwnedHandle, String> {
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle, RawHandle};

    let mut wide: Vec<u16> = name.encode_wide().collect();
    let byte_len = wide.len().saturating_mul(2);
    if byte_len > u16::MAX as usize {
        return Err("catalog path component is too long".to_string());
    }
    wide.push(0);
    let mut unicode = UnicodeString {
        length: byte_len as u16,
        maximum_length: (byte_len + 2) as u16,
        buffer: wide.as_mut_ptr(),
    };
    let mut attrs = ObjectAttributes {
        length: std::mem::size_of::<ObjectAttributes>() as u32,
        root_directory: parent.as_raw_handle(),
        object_name: &mut unicode,
        attributes: OBJ_CASE_INSENSITIVE | OBJ_DONT_REPARSE,
        security_descriptor: std::ptr::null_mut(),
        security_qos: std::ptr::null_mut(),
    };
    let mut iosb = IoStatusBlock {
        status: 0,
        information: 0,
    };
    let mut handle: RawHandle = std::ptr::null_mut();
    const FILE_READ_DATA: u32 = 0x0001;
    const FILE_READ_ATTRIBUTES: u32 = 0x0080;
    const SYNCHRONIZE: u32 = 0x0010_0000;
    const FILE_SHARE_READ: u32 = 0x1;
    const FILE_SHARE_WRITE: u32 = 0x2;
    const FILE_SHARE_DELETE: u32 = 0x4;
    const FILE_OPEN: u32 = 1;
    const FILE_DIRECTORY_FILE: u32 = 0x1;
    const FILE_NON_DIRECTORY_FILE: u32 = 0x40;
    const FILE_SYNCHRONOUS_IO_NONALERT: u32 = 0x20;
    const FILE_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    const FILE_ATTRIBUTE_NORMAL: u32 = 0x80;
    let mut options = FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_REPARSE_POINT;
    options |= if directory {
        FILE_DIRECTORY_FILE
    } else {
        FILE_NON_DIRECTORY_FILE
    };
    let status = unsafe {
        NtCreateFile(
            &mut handle,
            FILE_READ_DATA | FILE_READ_ATTRIBUTES | SYNCHRONIZE,
            &mut attrs,
            &mut iosb,
            std::ptr::null_mut(),
            FILE_ATTRIBUTE_NORMAL,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            FILE_OPEN,
            options,
            std::ptr::null_mut(),
            0,
        )
    };
    if status < 0 {
        return Err(nt_open_error(status, name));
    }
    if is_invalid_handle(handle) {
        return Err(format!(
            "cannot open catalog path component {}",
            name.to_string_lossy()
        ));
    }
    Ok(unsafe { OwnedHandle::from_raw_handle(handle) })
}

#[cfg(windows)]
fn handle_is_reparse(handle: &std::os::windows::io::OwnedHandle) -> Result<bool, String> {
    use std::os::windows::io::AsRawHandle;
    let mut info = ByHandleFileInformation::default();
    let ok = unsafe { GetFileInformationByHandle(handle.as_raw_handle(), &mut info) };
    if ok == 0 {
        return Err(format!(
            "cannot query catalog handle: {}",
            std::io::Error::last_os_error()
        ));
    }
    Ok((info.file_attributes & FILE_ATTRIBUTE_REPARSE_POINT) != 0)
}

#[cfg(windows)]
const STATUS_OBJECT_NAME_NOT_FOUND: u32 = 0xC000_0034;
#[cfg(windows)]
const STATUS_OBJECT_PATH_NOT_FOUND: u32 = 0xC000_003A;
#[cfg(windows)]
const STATUS_REPARSE_POINT_ENCOUNTERED: u32 = 0xC000_050B;
#[cfg(windows)]
const STATUS_STOPPED_ON_SYMLINK: u32 = 0x8000_002D;

#[cfg(windows)]
fn nt_open_error(status: i32, name: &std::ffi::OsStr) -> String {
    let code = status as u32;
    let shown = format!("0x{code:08X}");
    let display = name.to_string_lossy();
    match code {
        STATUS_REPARSE_POINT_ENCOUNTERED | STATUS_STOPPED_ON_SYMLINK => {
            format!("catalog path is a reparse point: {display} (ntstatus {shown})")
        }
        STATUS_OBJECT_NAME_NOT_FOUND | STATUS_OBJECT_PATH_NOT_FOUND => {
            format!("catalog path not found: {display} (ntstatus {shown})")
        }
        _ => format!("cannot open catalog path component {display}: ntstatus {shown}"),
    }
}

#[cfg(windows)]
fn is_invalid_handle(handle: std::os::windows::io::RawHandle) -> bool {
    handle.is_null() || handle == (-1isize as std::os::windows::io::RawHandle)
}

#[cfg(windows)]
const OBJ_CASE_INSENSITIVE: u32 = 0x0000_0040;
#[cfg(windows)]
const OBJ_DONT_REPARSE: u32 = 0x0000_1000;

#[cfg(windows)]
#[repr(C)]
struct UnicodeString {
    length: u16,
    maximum_length: u16,
    buffer: *mut u16,
}

#[cfg(windows)]
#[repr(C)]
struct ObjectAttributes {
    length: u32,
    root_directory: std::os::windows::io::RawHandle,
    object_name: *mut UnicodeString,
    attributes: u32,
    security_descriptor: *mut std::ffi::c_void,
    security_qos: *mut std::ffi::c_void,
}

#[cfg(windows)]
#[repr(C)]
struct IoStatusBlock {
    status: isize,
    information: usize,
}

#[cfg(windows)]
#[repr(C)]
#[derive(Default)]
struct FileTime {
    dw_low_date_time: u32,
    dw_high_date_time: u32,
}

#[cfg(windows)]
#[repr(C)]
#[derive(Default)]
struct ByHandleFileInformation {
    file_attributes: u32,
    creation_time: FileTime,
    last_access_time: FileTime,
    last_write_time: FileTime,
    volume_serial_number: u32,
    file_size_high: u32,
    file_size_low: u32,
    number_of_links: u32,
    file_index_high: u32,
    file_index_low: u32,
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn CreateFileW(
        file_name: *const u16,
        desired_access: u32,
        share_mode: u32,
        security_attributes: *mut std::ffi::c_void,
        creation_disposition: u32,
        flags_and_attributes: u32,
        template_file: std::os::windows::io::RawHandle,
    ) -> std::os::windows::io::RawHandle;
    fn GetFileInformationByHandle(
        handle: std::os::windows::io::RawHandle,
        file_information: *mut ByHandleFileInformation,
    ) -> i32;
}

#[cfg(windows)]
#[link(name = "ntdll")]
extern "system" {
    fn NtCreateFile(
        file_handle: *mut std::os::windows::io::RawHandle,
        desired_access: u32,
        object_attributes: *mut ObjectAttributes,
        io_status_block: *mut IoStatusBlock,
        allocation_size: *mut i64,
        file_attributes: u32,
        share_access: u32,
        create_disposition: u32,
        create_options: u32,
        ea_buffer: *mut std::ffi::c_void,
        ea_length: u32,
    ) -> i32;
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

    #[cfg(windows)]
    #[test]
    fn file_url_path_matches_verbatim_canonical_root() {
        // Url::to_file_path yields `C:\...`; fs::canonicalize yields `\\?\C:\...`.
        let root = PathBuf::from(r"\\?\C:\Users\runner\catalog");
        let via_url = PathBuf::from(r"C:\Users\runner\catalog\widget.schema.json");
        assert!(
            path_is_under_root(&via_url, &root),
            "file:// paths must match canonicalize() roots on Windows"
        );
        assert!(path_is_under_root(
            &PathBuf::from(r"c:\users\runner\catalog\widget.schema.json"),
            &root
        ));
        assert!(
            !path_is_under_root(
                &PathBuf::from(r"C:\Users\runner\catalog-other\widget.schema.json"),
                &root
            ),
            "sibling prefix must not count as contained"
        );
    }

    #[cfg(windows)]
    fn create_junction(link: &Path, target: &Path) -> std::io::Result<()> {
        if std::os::windows::fs::symlink_dir(target, link).is_ok() {
            return Ok(());
        }
        let status = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(link)
            .arg(target)
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(std::io::Error::other("mklink /J failed"))
        }
    }

    #[cfg(windows)]
    type SwapHookSlot = std::thread::LocalKey<std::cell::RefCell<Option<Box<dyn Fn()>>>>;

    #[cfg(windows)]
    fn with_swap_hook(
        slot: &'static SwapHookSlot,
        from: PathBuf,
        to: PathBuf,
    ) -> std::sync::Arc<std::sync::atomic::AtomicBool> {
        use std::sync::atomic::{AtomicBool, Ordering};
        let done = std::sync::Arc::new(AtomicBool::new(false));
        let flag = done.clone();
        slot.with(|hook| {
            *hook.borrow_mut() = Some(Box::new(move || {
                if flag.swap(true, Ordering::SeqCst) {
                    return;
                }
                fs::remove_file(&from).expect("remove_file for reparse swap");
                std::os::windows::fs::symlink_file(&to, &from)
                    .expect("symlink_file for reparse swap");
                assert!(
                    fs::symlink_metadata(&from)
                        .expect("swapped path metadata")
                        .file_type()
                        .is_symlink(),
                    "swap must leave a symlink"
                );
            }));
        });
        done
    }

    #[cfg(windows)]
    fn clear_hook(slot: &'static SwapHookSlot) {
        slot.with(|hook| *hook.borrow_mut() = None);
    }

    #[cfg(windows)]
    #[test]
    fn swapped_reparse_after_open_is_not_followed() {
        use std::sync::atomic::Ordering;
        let dir = scratch_dir();
        write_catalog(&dir);
        let target = dir.join("widget.schema.json");
        let outside = dir.parent().unwrap().join(format!(
            "rsfulmen-win-swap-{}-{}",
            std::process::id(),
            TEST_SEQ.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("evil.schema.json"), r#"{"type":"number"}"#).unwrap();
        let done = with_swap_hook(
            &AFTER_NOFOLLOW_OPEN,
            target.clone(),
            outside.join("evil.schema.json"),
        );
        let got = read_under_root(&dir, &target);
        clear_hook(&AFTER_NOFOLLOW_OPEN);
        assert!(done.load(Ordering::SeqCst), "swap hook must run");
        let _ = fs::remove_file(&target);
        let _ = fs::remove_dir_all(&outside);
        let value = got.expect("nofollow handle must keep the bytes opened before the swap");
        assert_eq!(
            value.get("type").and_then(|v| v.as_str()),
            Some("object"),
            "check-then-follow would load the swapped number schema"
        );
    }

    #[cfg(windows)]
    #[test]
    fn root_schema_swap_after_canonical_is_compile_failure() {
        use std::sync::atomic::Ordering;
        let dir = scratch_dir();
        write_catalog(&dir);
        let target = dir.join("root.schema.json");
        let outside = dir.parent().unwrap().join(format!(
            "rsfulmen-win-root-swap-{}-{}",
            std::process::id(),
            TEST_SEQ.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("evil.schema.json"), r#"{"type":"number"}"#).unwrap();
        let done = with_swap_hook(
            &BEFORE_ROOT_SCHEMA_READ,
            target.clone(),
            outside.join("evil.schema.json"),
        );
        let err = validate_instance_with_schema_file(&target, &json!({}), opts(&dir)).unwrap_err();
        clear_hook(&BEFORE_ROOT_SCHEMA_READ);
        assert!(
            done.load(Ordering::SeqCst),
            "root-schema swap hook must run"
        );
        let _ = fs::remove_file(&target);
        let _ = fs::remove_dir_all(&outside);
        match err {
            SchemaValidationError::SchemaCompileFailed { message, .. } => {
                assert!(
                    message.contains("reparse"),
                    "root schema swap must report reparse, got {message}"
                );
            }
            other => panic!("expected compile failure, got {other:?}"),
        }
    }

    #[cfg(windows)]
    #[test]
    fn catalog_index_swap_is_compile_failure() {
        use std::sync::atomic::Ordering;
        let dir = scratch_dir();
        write_catalog(&dir);
        let target = dir.join("widget.schema.json");
        let outside = dir.parent().unwrap().join(format!(
            "rsfulmen-win-idx-swap-{}-{}",
            std::process::id(),
            TEST_SEQ.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("evil.schema.json"), r#"{"type":"number"}"#).unwrap();
        let done = with_swap_hook(
            &BEFORE_INDEX_READ,
            target.clone(),
            outside.join("evil.schema.json"),
        );
        let err = match FileBackedResolver::from_schema_file(
            &dir.join("root.schema.json"),
            &opts(&dir),
        ) {
            Err(err) => err,
            Ok(_) => panic!("index swap must fail resolver construction"),
        };
        clear_hook(&BEFORE_INDEX_READ);
        assert!(done.load(Ordering::SeqCst), "index swap hook must run");
        let _ = fs::remove_file(&target);
        let _ = fs::remove_dir_all(&outside);
        match err {
            SchemaValidationError::SchemaCompileFailed { message, .. } => {
                assert!(
                    message.contains("reparse"),
                    "index swap must report reparse, got {message}"
                );
            }
            other => panic!("expected compile failure, got {other:?}"),
        }
    }

    #[cfg(windows)]
    #[test]
    fn missing_in_root_ref_is_not_reported_as_reparse() {
        let dir = scratch_dir();
        write_catalog(&dir);
        fs::write(
            dir.join("uses-missing.schema.json"),
            r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "x": { "$ref": "no-such.schema.json" }
  }
}
"#,
        )
        .unwrap();
        let err = validate_instance_with_schema_file(
            &dir.join("uses-missing.schema.json"),
            &json!({}),
            opts(&dir),
        )
        .unwrap_err();
        match err {
            SchemaValidationError::SchemaCompileFailed { message, .. } => {
                assert!(
                    !message.contains("reparse"),
                    "missing file must not be diagnosed as reparse: {message}"
                );
                assert!(
                    message.contains("not found") || message.contains("cannot open"),
                    "missing file should be not-found, got {message}"
                );
            }
            other => panic!("expected compile failure, got {other:?}"),
        }
    }

    #[cfg(windows)]
    #[test]
    fn in_root_junction_to_outside_is_compile_failure() {
        let dir = scratch_dir();
        write_catalog(&dir);
        let outside = dir.parent().unwrap().join(format!(
            "rsfulmen-win-junc-{}-{}",
            std::process::id(),
            TEST_SEQ.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("evil.schema.json"), r#"{"type":"number"}"#).unwrap();
        let link = dir.join("junc");
        create_junction(&link, &outside).expect("create junction or directory symlink");
        fs::write(
            dir.join("uses-junc.schema.json"),
            r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "x": { "$ref": "junc/evil.schema.json" }
  }
}
"#,
        )
        .unwrap();
        let err = validate_instance_with_schema_file(
            &dir.join("uses-junc.schema.json"),
            &json!({}),
            opts(&dir),
        )
        .unwrap_err();
        let _ = fs::remove_dir(&link);
        let _ = fs::remove_dir_all(&outside);
        match err {
            SchemaValidationError::SchemaCompileFailed { message, .. } => {
                assert!(
                    message.contains("reparse"),
                    "junction must be diagnosed as a reparse point, got {message}"
                );
            }
            other => panic!("expected compile failure, got {other:?}"),
        }
    }
}
