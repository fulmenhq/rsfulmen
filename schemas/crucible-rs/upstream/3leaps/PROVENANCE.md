# Provenance: 3leaps/crucible

**DO NOT EDIT THESE FILES.** They are vendored copies from upstream.

## Source

| Field      | Value                                     |
| ---------- | ----------------------------------------- |
| Repository | https://github.com/3leaps/crucible        |
| Tag        | v0.1.3                                    |
| Commit     | 92a37c41c0b66cc595e963a990082e388c2c9abc  |
| Date       | 2026-01-02                                |
| Synced By  | devlead (Claude Opus 4.5 via Claude Code) |

## Files Included

```
3leaps/
├── ailink/v0/
│   ├── prompt.schema.json
│   └── search-response.schema.json
└── agentic/v0/
    └── role-prompt.schema.json
```

## Canonical URLs

These schemas are canonically hosted at:

- `https://schemas.3leaps.dev/ailink/v0/prompt.schema.json`
- `https://schemas.3leaps.dev/ailink/v0/search-response.schema.json`
- `https://schemas.3leaps.dev/agentic/v0/role-prompt.schema.json`

## Refresh

To refresh from upstream:

```bash
cd /path/to/3leaps/crucible && git pull
cp schemas/ailink/v0/*.json /path/to/fulmenhq/crucible/schemas/upstream/3leaps/ailink/v0/
cp schemas/agentic/v0/*.json /path/to/fulmenhq/crucible/schemas/upstream/3leaps/agentic/v0/
# Update this PROVENANCE.md with new commit hash and date
```
