---
title: "Tabular Exchange Schemas v0.1.0"
description: "JSON Schemas for the table schema, conversion policy and conversion result documents of the Tabular Exchange Standard"
author: "Crucible maintainers"
date: "2026-11-19"
last_updated: "2026-11-19"
status: "draft"
tags: ["schemas", "data", "tabular-exchange"]
---

# Tabular Exchange Schemas (v0.1.0)

These schemas describe the documents of the
[Tabular Exchange Standard](../../../../docs/standards/data/tabular-exchange.md)
at contract revision `tabular-exchange/0.1.0`. The standard is normative. The
schemas express the part of it that JSON Schema can.

## Files

| File                            | Describes                                | Wire format                 |
| ------------------------------- | ---------------------------------------- | --------------------------- |
| `table-schema.schema.json`      | a table schema document                  | `tabular-exchange.schema/0` |
| `conversion-policy.schema.json` | a conversion policy document             | `tabular-exchange.policy/0` |
| `conversion-result.schema.json` | a conversion result document             | `tabular-exchange.result/0` |
| `row.schema.json`               | one row after its line is parsed as JSON | none                        |
| `semantic-validation.md`        | the rules the schemas cannot express     |                             |

All schemas use JSON Schema draft 2020-12. Each `$id` is
`https://schemas.fulmenhq.dev/crucible/data/tabular-exchange/v0.1.0/<file>`.
No schema refers to another file; every `$ref` is to a `$defs` entry of the
same file. Every object the standard defines sets `additionalProperties:
false`. The values under `extensions` in a policy and in a result are the only
objects left open, and the schemas bound them: at most 8 namespaces, strings of
at most 256 code points, and no content deeper than depth 6 of the document.

There is no schema for a batch of rows. Rows are newline-delimited JSON, which
is not one JSON document, and the typing of an element depends on the table
schema. `row.schema.json` checks only that a parsed line is an array of
`null`, booleans and strings.

## Three layers of checking

1. **Bytes and JSON grammar.** UTF-8, no byte order mark, no repeated key, the
   restricted spelling of numbers. A general JSON parser does not check these.
2. **Structure.** The schemas in this directory.
3. **Semantics.** The rules in `semantic-validation.md`, some of which relate
   two documents.

A document that passes layer 2 alone is not conformant.

## Data

Under `config/data/tabular-exchange/v0.1.0/`:

| File                         | Content                                                                                                   |
| ---------------------------- | --------------------------------------------------------------------------------------------------------- |
| `contract.json`              | the closed lists, bounds and registered rule identifiers                                                  |
| `value-vectors.json`         | accepted and refused spellings of values; quantization results                                            |
| `row-vectors.json`           | inputs in the rows encoding and their outcomes                                                            |
| `document-vectors.json`      | documents as exact bytes, for the rules of layer 1                                                        |
| `matching-vectors.json`      | results, candidate outputs and caller expectations with the matching verdict and the step that decides it |
| `fixtures/manifest.json`     | every fixture, the layer that decides it and the expected category and rule                               |
| `fixtures/valid/`            | documents that are accepted                                                                               |
| `fixtures/invalid/`          | documents that fail their JSON Schema                                                                     |
| `fixtures/semantic-invalid/` | documents that satisfy their JSON Schema and are refused by a semantic rule                               |

Three files under `fixtures/semantic-invalid/` repeat a key on purpose and
cannot be loaded by a parser that refuses repeated keys. The manifest marks
them `strict_parse: refuse`.

Every namespace under `extensions` in a fixture or a vector is a reserved
example namespace. No reader implements one, so the expected outcome of each
fixture is the same for every reader: a policy that carries one is refused,
and a result that carries one is read as if the namespace were absent.

## Checking a consumer

A consumer of results is checked by executing the result fixtures and
`matching-vectors.json`, including the cases that mutate one digit, one length
or one member, and comparing the category and rule, or the verdict and rule,
with the expected ones.

The reference checker `scripts/tabular-exchange-reference-check.py` (Python 3,
standard library only; `make validate-tabular-exchange`) accompanies this
revision. It reads the three documents strictly, applies the rules of
`semantic-validation.md` that can be applied to documents alone, runs the
matching procedure with caller expectations, and compares its outcomes with
the manifest and the vectors. It also repeats every check on a result with its
`extensions` member deleted.

The reference checker and the vectors come from the same authors. Agreement
between them shows that the vectors are consistent with one reading of the
standard; it is weaker evidence than a consumer written by a third party, and
it does not qualify an implementation. A claim that a consumer conforms rests
on that consumer executing the fixtures and the vectors itself.

## Compatibility

The family is below 1.0 and a consumer pins this exact directory. A published
revision is not changed; a correction is a new revision directory. See the
Compatibility section of the standard.
