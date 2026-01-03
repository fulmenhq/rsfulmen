# Pathfinder Schemas v1.0.0

JSON Schemas for the Pathfinder file discovery API, ensuring consistent data structures across Go, TypeScript, Python, and other language implementations.

## Schemas

### find-query.schema.json

**Purpose**: Query parameters for file discovery operations

**Key Properties**:

- `root`: Root directory to search from (required)
- `include`: Glob patterns to include (array of strings)
- `exclude`: Glob patterns to exclude (array of strings)
- `maxDepth`: Maximum directory depth (integer, min 0, default 0)
- `followSymlinks`: Whether to follow symbolic links (boolean, default false)
- `includeHidden`: Whether to include hidden files/directories (boolean, default false)

### finder-config.schema.json

**Purpose**: Configuration for the Finder component

**Key Properties**:

- `maxWorkers`: Maximum concurrent workers (integer, min 1)
- `cacheEnabled`: Whether caching is enabled (boolean)
- `cacheTTL`: Cache time-to-live in seconds (integer, min 0)
- `constraint`: Path constraint configuration (PathConstraint object)
- `loaderType`: Type of loader to use (string)

### path-result.schema.json

**Purpose**: Structure of discovery results returned by Pathfinder

**Key Properties**:

- `relativePath`: Path relative to discovery root (string)
- `sourcePath`: Absolute source path (string)
- `logicalPath`: Logical mapping path (string)
- `loaderType`: Type of loader used (string)
- `metadata`: Additional file metadata (Metadata object)

### error-response.schema.json

**Purpose**: Standardized error response structure for consistent error handling

**Key Properties**:

- `code`: Error code identifier (string, required)
- `message`: Human-readable error message (string, required)
- `details`: Additional error details (object)
- `path`: Path that caused the error (string, optional)
- `timestamp`: When the error occurred (date-time string, optional)

### path-constraint.schema.json

**Purpose**: Path constraint configuration for defining safety boundaries

**Key Properties**:

- `root`: Root path for the constraint boundary (string, required)
- `type`: Type of constraint - `repository`, `workspace`, or `cloud` (required)
- `enforcementLevel`: Strictness - `strict`, `warn`, or `permissive` (required)
- `allowedPatterns`: Additional allowed path patterns (array of strings)
- `blockedPatterns`: Blocked path patterns (array of strings)

### metadata.schema.json

**Purpose**: Flexible metadata structure for path results

**Key Properties**:

- `size`: File size in bytes (integer)
- `modified`: Last modification timestamp (date-time string)
- `permissions`: File permissions (string, octal or symbolic)
- `mimeType`: MIME type of the file (string)
- `encoding`: Character encoding if applicable (string)
- `checksum`: File checksum/hash (string)
- `tags`: User-defined tags (array of strings)
- `custom`: Custom metadata fields (object)

## Usage

These schemas are used across all Fulmen implementations:

- `gofulmen/pathfinder`: Go implementation
- `tsfulmen/pathfinder`: TypeScript implementation
- `pyfulmen/pathfinder`: Python implementation (future)

**Purpose**:

- Validate API inputs and outputs
- Generate type definitions and validation code
- Ensure API consistency across languages
- Provide documentation for API consumers

## Versioning

Schemas follow semantic versioning. Breaking changes will increment the major version number.

## Naming Conventions

All property names use **camelCase** for consistency with JSON/JavaScript conventions.
