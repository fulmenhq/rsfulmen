---
title: "Crucible Pseudo-Monorepo Playbook"
description: "Why Crucible mirrors assets into language wrappers and how maintainers keep the SSOT healthy"
author: "Codex Assistant"
date: "2025-10-02"
last_updated: "2025-10-02"
status: "draft"
tags: ["architecture", "monorepo", "distribution", "maintenance"]
---

# Crucible Pseudo-Monorepo Playbook

## Why This Structure Exists

Crucible acts as a standards forge for **every** Fulmen repo. We need the ergonomics of a monorepo (single CalVer, shared CI, one review surface) without forcing downstream tools to depend on a giant checkout. The compromise:

- Root directories (`schemas/`, `docs/`, `templates/`) are **the only authoritative copies**.
- Language wrappers under `lang/` are **facades** that embed the same assets for Go/TypeScript distribution.
- Scripts in `scripts/` keep the facades fresh so we can ship packages as if they lived in independent repos.

## Distribution Paths at a Glance

| Consumer need                      | Recommendation                                                                                      | Notes                                                             |
| ---------------------------------- | --------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------- |
| Runtime access inside Go/Bun tools | Install the published package (`go get github.com/fulmenhq/crucible`, `bun add @fulmenhq/crucible`) | Gives canonical schemas + standards without vendoring             |
| Vendored/uncommitted clones        | Copy `scripts/pull/crucible-pull.ts` into the repo and pin `.crucible-sync.json`                    | Ideal for templates, infra repos, or when editing schemas locally |
| Language port or custom wrapper    | Start in the downstream repo, but source assets by syncing from Crucible                            | Ensures shared versioning + schema normalization contract         |

## Maintainer Responsibilities

1. **Author in the Root** – Never edit files under `lang/` by hand. Update the root asset, then run:
   ```bash
   bun run scripts/sync-to-lang.ts
   bun run scripts/update-version.ts
   ```
2. **Validate Packages** – After syncing, run language tests:
   ```bash
   bun run test:go
   bun run test:ts
   ```
3. **Release with Confidence** – CalVer lives in `VERSION`. Bump once, run `bun run version:update`, commit, and tag.
4. **Document Changes** – Each new schema or standard should add a README or doc entry so downstream consumers know what changed.

## Integration Expectations for Downstream Teams

- **Version Pinning** – Teams should commit `.crucible-version` when using the pull script, or rely on dependency managers when using packages.
- **Schema Semantics** – Use `normalizeSchema` / `compareSchemas` helpers (Go + TypeScript) to assert nothing drifted when vendoring.
- **Sync in CI** – Downstream pipelines should run pull scripts in validate mode (`bun run scripts/crucible-pull.ts --validate`) to avoid stale assets.

## FAQ

**Why not a single workspace monorepo?**  
We ship assets to different registries (Go Proxy, npm) and need to keep repo contributions lightweight. Mirroring into `lang/` lets us publish without requiring every consumer to adopt workspaces.

**Can we add more languages?**  
Yes. New `{lang}fulmen` repos should follow the same pattern: author assets in Crucible, sync via scripts, and expose the shared normalization + comparison API.

**How do we spot unsynced changes?**  
Run `bun run sync:to-lang` locally or wire the command into CI. Any drift shows up as a diff in `lang/` directories.

## Related Documents

- [Crucible Sync Model Architecture](./sync-model.md)
- [Crucible Sync Strategy Guide](../guides/sync-strategy.md)
- [Schema Normalization Standard](../standards/schema-normalization.md)
