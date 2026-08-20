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
    /// Repeatable catalog roots. The root schema file’s directory is always an allowed root.
    pub ref_dirs: Vec<PathBuf>,
    /// `$id` vs path-suffix resolution.
    pub resolution: FileSchemaResolution,
}

/// Offline catalog resolver. Allowed roots are the canonical schema-file directory and each `ref_dirs` entry.
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
        let schema_path = canonicalize_existing(schema_path)?;
        let schema_dir = schema_path
            .parent()
            .ok_or_else(|| SchemaValidationError::SchemaCompileFailed {
                path: schema_path.display().to_string(),
                message: "schema path has no parent directory".to_string(),
            })?
            .to_path_buf();

        let mut allowed_roots = vec![schema_dir.clone()];
        for dir in &opts.ref_dirs {
            allowed_roots.push(canonicalize_existing(dir)?);
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
        let contained =
            contained_canonical(path, &self.allowed_roots).map_err(|e| anyhow::anyhow!(e))?;
        let value = load_value_stable(&contained).map_err(|e| anyhow::anyhow!(e.to_string()))?;
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

fn contained_canonical(path: &Path, roots: &[PathBuf]) -> Result<PathBuf, String> {
    let canon = fs::canonicalize(path).map_err(|e| {
        format!(
            "schema path not contained in catalog roots (canonicalize failed): {}: {e}",
            path.display()
        )
    })?;
    if roots.iter().any(|root| canon.starts_with(root)) {
        Ok(canon)
    } else {
        Err(format!(
            "schema path not contained in catalog roots: {}",
            canon.display()
        ))
    }
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
}
