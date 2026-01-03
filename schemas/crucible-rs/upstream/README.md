# Upstream Schemas

**DO NOT EDIT FILES IN THIS DIRECTORY.**

This directory contains vendored copies of schemas from upstream repositories. These files are copied verbatim for offline validation and CI reliability.

## Why Vendoring?

CI pipelines need to validate against these schemas without network dependencies. Fetching from remote URLs at build time creates fragile builds that fail when upstream is unavailable.

## Refresh Process

When upstream schemas change:

1. Pull latest from upstream repository
2. Copy updated schemas to appropriate subdirectory
3. Update `PROVENANCE.md` in the source directory
4. Run `make precommit` to validate
5. Commit with clear attribution

```bash
# Example refresh from local clone
cp -r ../3leaps/crucible/schemas/ailink schemas/upstream/3leaps/
cp -r ../3leaps/crucible/schemas/agentic schemas/upstream/3leaps/
# Update PROVENANCE.md with new commit hash
```

## Directory Structure

```
upstream/
├── README.md           # This file
└── 3leaps/             # From github.com/3leaps/crucible
    ├── PROVENANCE.md   # Source commit and date
    ├── ailink/v0/      # AILink prompt schemas
    └── agentic/v0/     # Agentic role prompt schemas
```

## Provenance

Each upstream source has a `PROVENANCE.md` documenting:

- Source repository
- Commit hash
- Sync date
- Files included

## Related

- [goneat metaschemas](https://github.com/fulmenhq/goneat) - Same pattern for JSON Schema metaschemas
