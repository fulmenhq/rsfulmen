# Provenance: 3leaps/crucible

**DO NOT EDIT THESE FILES.** They are vendored copies from upstream.

## Source

| Field      | Value                                      |
| ---------- | ------------------------------------------ |
| Repository | https://github.com/3leaps/crucible         |
| Tag        | v0.1.12                                  |
| Commit     | 96a17853ee48446f7740d61a518fcfcba20ae444 |
| Date       | 2026-02-18                               |
| Synced By  | devlead (Claude Opus 4.5 via Claude Code)  |

## Structure

```
schemas/upstream/3leaps/crucible/
 ├── schemas/
 │   ├── classifiers/v0/   # Dimension meta-schemas
 │   ├── foundation/v0/    # Type primitives, error response
 │   ├── ailink/v0/        # Prompt/response schemas
 │   └── agentic/v0/       # Role prompt schemas
 ├── config/
 │   └── classifiers/      # Dimension definitions
 ├── docs/
 │   ├── standards/        # Classification standards
 │   └── catalog/          # Classifier catalog
 └── PROVENANCE.md
```

## Files Synced

28 files from 3leaps/crucible

## Canonical URLs

These schemas are canonically hosted at:

- `https://schemas.3leaps.dev/classifiers/v0/*.json`
- `https://schemas.3leaps.dev/foundation/v0/*.json`
- `https://schemas.3leaps.dev/ailink/v0/*.json`
- `https://schemas.3leaps.dev/agentic/v0/*.json`

## Excluded (Manual Sync)

The following are NOT synced by this script:

- `config/agentic/` - Role definitions

These are synced manually as they require review for fulmenhq-specific customization.

## Refresh

To refresh from upstream:

```bash
bun run scripts/3leaps-crucible-upstream-pull.ts
```

Or with dry-run to preview:

```bash
bun run scripts/3leaps-crucible-upstream-pull.ts --dry-run
```
