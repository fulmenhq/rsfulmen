# Schema Validation Schemas v1.0.0

JSON Schemas for configuring and managing JSON Schema validation across Fulmen libraries.

## Schemas

### validator-config.schema.json

**Purpose**: Configuration for JSON Schema validators

**Key Properties**:

- `schema`: Schema content as JSON string (string)
- `schemaFile`: Path to schema file (string)
- `baseURI`: Base URI for schema resolution (URI string)
- `formatValidators`: Custom format validators (object)
- `metaSchemas`: Additional meta-schemas to load (array of strings)

**Use Cases**:

- Configuring schema validation in libraries
- Runtime schema loading
- Custom format validation
- Meta-schema management

### schema-registry.schema.json

**Purpose**: Configuration for schema registry management

**Key Properties**:

- `baseDir`: Base directory for schema storage (string, required)
- `cache`: Cached schema content map (object, name/version → content)
- `defaultRegistry`: Whether this is the default registry (boolean)
- `autoReload`: Whether to auto-reload changed schemas (boolean)

**Use Cases**:

- Managing collections of schemas
- Schema caching and reloading
- Multi-version schema support
- Registry configuration

## Usage

These schemas are used by the schema validation packages in:

- `gofulmen/schema`: Go schema validation
- `tsfulmen/schema`: TypeScript schema validation
- `pyfulmen/schema`: Python schema validation (future)

**Purpose**:

- Validate schema validation configuration
- Ensure consistent schema registry APIs
- Type generation for validation utilities
- Cross-language consistency

## Versioning

Schemas follow semantic versioning. Breaking changes will increment the major version number.

## Naming Conventions

All property names use **camelCase** for consistency with JSON/JavaScript conventions.
