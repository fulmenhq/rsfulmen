# Enact (Planned)

This document reserves the **Enact** module namespace for future Fulmen helper-library implementations.

Enact is currently envisioned primarily as a **workhorse CLI + plugin ecosystem** for guided, imperative, idempotent infrastructure deployments (the middle ground between ad-hoc scripts and full declarative IaC).

## Current Status

- Enact is currently **schema-first**: the schema namespace is being incubated before any helper-library API is standardized.
- No helper-library (`gofulmen`, `pyfulmen`, `tsfulmen`) module exists yet.
- There is no public, stable cross-language API contract to implement.

This is intentional: schemas can stabilize independently, while the eventual helper-library module will depend on real-world CLI/plugin usage patterns.

## When This Becomes a Real Module

This file should be expanded into a normative spec once Enact has:

- A stable schema namespace and URL contract under `schemas.fulmenhq.dev/enact/`.
- A defined runtime/config surface that helper libraries should expose (loading/validation helpers, runlog/event types, inventory helpers, etc.).

## Prototype Schemas (Current Source)

Until Enact schemas land in Crucible under `schemas/enact/`, the active prototype schema set currently lives here:

- In-repo schema source: `schemas/enact/v1.0.0/`
- Published URL (target): `https://schemas.fulmenhq.dev/enact/`

## Related (Planned)

- Architecture docs for Enact should live under `docs/architecture/` once promoted from planning drafts.
