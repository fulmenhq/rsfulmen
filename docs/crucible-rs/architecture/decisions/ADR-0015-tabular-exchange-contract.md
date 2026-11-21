---
id: "ADR-0015"
title: "Tabular Exchange Contract in a New Data Domain"
status: "experimental"
date: "2026-11-19"
last_updated: "2026-11-19"
deciders:
  - "@3leapsdave"
scope: "Ecosystem"
tags:
  - "data"
  - "tabular-exchange"
  - "schemas"
  - "contracts"
related_adrs:
  - "ADR-0012"
adoption:
  gofulmen: "not-applicable"
  pyfulmen: "not-applicable"
  tsfulmen: "not-applicable"
  rsfulmen: "not-applicable"
  csfulmen: "not-applicable"
---

# ADR-0015: Tabular Exchange Contract in a New Data Domain

## Status

**Current Status**: Experimental

The contract design and its vectors are reviewed. No implementation is
qualified against this revision, and no consumer independent of the contract's
authors has executed the result and matching vectors yet.

## Context

Programs that convert typed tables between file formats need a shared
description of a table, of the values in its cells, of the instructions for a
conversion and of the report a conversion produces. Without one, each
converter defines these for itself, and a caller cannot tell from a report
whether an output answers the request it made.

Crucible holds the shared contracts of the ecosystem. It has no domain for
format-neutral data contracts: `library/` holds helper-library modules and
`content/` covers embedding and document provenance.

## Decision

1. Crucible adds a `data` domain and publishes the **Tabular Exchange
   Standard** as the family `tabular-exchange`, contract revision
   `tabular-exchange/0.1.0`:
   - standard: `docs/standards/data/tabular-exchange.md`;
   - JSON Schemas (draft 2020-12, local `$defs` only):
     `schemas/data/tabular-exchange/v0.1.0/`, with `$id`
     `https://schemas.fulmenhq.dev/crucible/data/tabular-exchange/v0.1.0/<file>`;
   - rules JSON Schema cannot express:
     `schemas/data/tabular-exchange/v0.1.0/semantic-validation.md`;
   - contract manifest, value, row, document and matching vectors, and
     fixtures in `valid/`, `invalid/` and `semantic-invalid/`:
     `config/data/tabular-exchange/v0.1.0/`.
2. The three wire formats are `tabular-exchange.schema/0`,
   `tabular-exchange.policy/0` and `tabular-exchange.result/0`. An
   incompatible change to one document's shape is a new wire-format revision,
   also below 1.0. Format identifiers an implementation used before adopting
   the standard are different formats, not aliases.
3. The family starts at contract revision `tabular-exchange/0.1.0`. Below
   1.0, consumers pin an exact contract revision; a published revision is never
   changed. A Crucible release distributes contract revisions and is versioned
   separately: a consumer obtains the pinned revision from a Crucible release
   that contains it.
4. Implementation detail travels in one bounded, namespaced `extensions`
   object. A result consumer ignores unknown namespaces; a policy reader
   refuses them. Every mandatory interpretation works without extension
   content.
5. A result is matched to an output **and** to the request through
   caller-supplied expectations, with separate verdicts for output agreement
   and for completion of the requested run.
6. Crucible's gates enforce the family:
   - `make validate-schema-contracts` compiles the schemas, validates the
     `valid/` and schema-valid `semantic-invalid/` fixtures against them, and
     locks every intentionally invalid fixture by SHA-256 in the inventory;
   - `make validate-tabular-exchange` runs a standard-library reference
     checker over every fixture and vector, fails on any disagreement or
     missing expectation, runs regression controls for both, and is part of
     `validate-schemas` and the `Schema Contracts` workflow.

## Relationship to the data artifact contract

Tabular exchange defines typed conversion and request-bound completion. A
data artifact descriptor (`data-artifact/v0`, published in 3leaps/crucible as
the [data artifact contract](https://github.com/3leaps/crucible/blob/3636a79f7ea507bccfa6534655764d7b36c2ea05/docs/standards/data-artifact-contract.md),
v0.2.12) can describe and link the produced file for discovery, provenance and
protection. Either contract can be used without the other; conversion
completion and artifact completeness stay independent.

## Consequences

- Converters and consumers have one published contract and one set of
  vectors to test against.
- The reference checker comes from the contract's authors. Its agreement with
  the vectors shows the vectors are consistent with one reading of the
  standard; it does not qualify an implementation.
- Qualification of this revision waits for a consumer written independently
  of the contract's authors that executes the result and matching vectors.
- Existing implementations that predate the standard differ from it in places
  and must change to conform; their earlier format identifiers remain
  separate formats.
- Every change to the family, including a bound, is a new contract revision
  and a new Crucible release.

## Related

- [Tabular Exchange Standard](../../standards/data/tabular-exchange.md)
- [ADR-0012: Use Absolute $id URLs for Cross-Schema $ref References](ADR-0012-schema-ref-ids.md)
