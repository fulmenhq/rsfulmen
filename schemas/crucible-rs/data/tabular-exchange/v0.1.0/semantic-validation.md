---
title: "Tabular Exchange Semantic Validation v0.1.0"
description: "Rules of the Tabular Exchange Standard that its JSON Schemas cannot express, with the category, rule identifier and negative control of each"
author: "Crucible maintainers"
date: "2026-11-19"
last_updated: "2026-11-19"
status: "draft"
tags: ["schemas", "data", "tabular-exchange", "validation"]
---

# Tabular Exchange Semantic Validation (v0.1.0)

The [Tabular Exchange Standard](../../../../docs/standards/data/tabular-exchange.md)
is normative. This document lists, one by one, the rules a reader implements
in code because validating against the JSON Schemas in this directory does not
establish them. Each rule has a number that is stable within this contract
revision, the category and rule identifier a reader reports, and a negative
control.

Controls are named relative to `config/data/tabular-exchange/v0.1.0/`. A
control of the form `value-vectors.json: decimal` is every refused entry of
that list with the stated rule. `<kind>` is `schema`, `policy` or `result`.

Column **JS** says what the JSON Schema does with the control:

- **no**: the schema accepts it, or the control is not a JSON document;
- **shape**: the schema rejects it but cannot report the rule, the category or
  the order required;
- **n/a**: there is no JSON Schema for the subject.

## 1. Bytes and JSON grammar of a document

These rules apply to all three documents. The controls use the table schema.

| No.  | Rule                                                                                                     | Category        | Rule identifier         | JS    | Control                                                                                                                                                                                                                                                     |
| ---- | -------------------------------------------------------------------------------------------------------- | --------------- | ----------------------- | ----- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| D-01 | The bytes are UTF-8.                                                                                     | `invalid-input` | `<kind>-not-utf8`       | no    | `document-vectors.json`: invalid UTF-8                                                                                                                                                                                                                      |
| D-02 | There is no byte order mark.                                                                             | `invalid-input` | `<kind>-malformed`      | no    | `document-vectors.json`: byte order mark                                                                                                                                                                                                                    |
| D-03 | The text is exactly one JSON value, with only JSON whitespace around it.                                 | `invalid-input` | `<kind>-malformed`      | no    | `document-vectors.json`: empty, two values, text after the value                                                                                                                                                                                            |
| D-04 | No object repeats a key, at any depth, compared after escapes are decoded.                               | `invalid-input` | `<kind>-malformed`      | no    | `fixtures/semantic-invalid/*/key-repeated.json`; `document-vectors.json`: key repeated through an escape                                                                                                                                                    |
| D-05 | A number is `0` or a nonzero digit followed by digits.                                                   | `invalid-input` | `<kind>-malformed`      | no    | `fixtures/semantic-invalid/table-schema/number-with-fraction.json`, `number-with-exponent.json`; `document-vectors.json`                                                                                                                                    |
| D-06 | A string has no unknown escape, unpaired surrogate or unescaped control character.                       | `invalid-input` | `<kind>-malformed`      | no    | `document-vectors.json`: unknown escape, lone surrogate                                                                                                                                                                                                     |
| D-07 | Nesting is at most 3 in a schema and 6 in a policy or a result, the document's own object being depth 1. | `invalid-input` | `<kind>-malformed`      | shape | `fixtures/invalid/table-schema/nested-too-deeply.json`; `fixtures/invalid/conversion-policy/extensions-past-depth-bound.json`; `fixtures/invalid/conversion-result/extensions-past-depth-bound.json`                                                        |
| D-08 | Rules D-01 to D-07 are judged before `format`, and `format` before any other member.                     | `invalid-input` | as the rule that fails  | shape | `document-vectors.json`: repeated key and unsupported format; `fixtures/invalid/table-schema/format-other-revision.json`                                                                                                                                    |
| D-09 | A member that does not apply to the stated type, status or representation is an unknown member.          | `invalid-input` | `<kind>-member-unknown` | shape | `fixtures/invalid/table-schema/precision-on-integer.json`; `fixtures/invalid/conversion-result/ok-with-refusal.json`                                                                                                                                        |
| D-10 | An integer member outside its range is an invalid member, whatever its size. It is not malformed.        | `invalid-input` | `<kind>-member-invalid` | shape | `fixtures/invalid/table-schema/precision-39.json`; `fixtures/invalid/conversion-result/ok-rows-far-past-max.json`                                                                                                                                           |
| D-11 | No number exceeds 9007199254740991. The bound holds in a result of every status.                         | `invalid-input` | `<kind>-member-invalid` | shape | `fixtures/invalid/conversion-result/ok-rows-past-max.json`, `refused-row-past-max.json`, `failed-input-bytes-past-max.json`; accepted at the bound: `fixtures/valid/conversion-result/ok-count-max.json`, `refused-count-max.json`, `failed-count-max.json` |

`<kind>-malformed` covers both text that is not JSON and JSON that rules D-02,
D-04, D-05 and D-07 exclude. The two are not given separate identifiers in a
document. They are in the rows encoding: see R-05 and R-09.

D-11 is stated at 2^53 - 1 so that a reader whose numbers are binary64 still
decides it correctly: 9007199254740992 and every greater integer reads as a
value above the bound. A reader MUST NOT refuse such a number as unparseable,
and MUST NOT wrap or truncate it. The sums in C-06 and C-07 never exceed
`rows`, so they stay within the bound.

A reader MAY refuse a document above a size limit of its own with
`resource-limit`, `<kind>-too-large`. This standard sets no limit, so there is
no control.

## 2. Table schema

| No.  | Rule                                                          | Category           | Rule identifier           | JS    | Control                                                                    |
| ---- | ------------------------------------------------------------- | ------------------ | ------------------------- | ----- | -------------------------------------------------------------------------- |
| S-01 | Column identifiers are unique.                                | `invalid-input`    | `schema-columns-invalid`  | no    | `fixtures/semantic-invalid/table-schema/column-id-repeated.json`           |
| S-02 | `scale` does not exceed `precision`.                          | `invalid-input`    | `schema-member-invalid`   | no    | `fixtures/semantic-invalid/table-schema/scale-exceeds-precision.json`      |
| S-03 | A `type` outside the closed list is unsupported, not invalid. | `unsupported-type` | `schema-type-unsupported` | shape | `fixtures/invalid/table-schema/type-unknown.json`, `type-letter-case.json` |
| S-04 | There is at least one column and no identifier is empty.      | `invalid-input`    | `schema-columns-invalid`  | shape | `fixtures/invalid/table-schema/columns-empty.json`, `column-id-empty.json` |

## 3. Rows

There is no JSON Schema for the row stream. `row.schema.json` describes one
parsed row and nothing in this section.

| No.  | Rule                                                                                                                                                                                                                         | Category               | Rule identifier        | JS  | Control                                                                         |
| ---- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------- | ---------------------- | --- | ------------------------------------------------------------------------------- |
| R-01 | The input has no byte order mark.                                                                                                                                                                                            | `invalid-input`        | `byte-order-mark`      | n/a | `row-vectors.json`: byte order mark                                             |
| R-02 | Every line is UTF-8.                                                                                                                                                                                                         | `invalid-input`        | `invalid-utf8`         | n/a | `row-vectors.json`: invalid UTF-8                                               |
| R-03 | No line is empty or only spaces and tabs.                                                                                                                                                                                    | `invalid-input`        | `blank-line`           | n/a | `row-vectors.json`: four cases                                                  |
| R-04 | A line is one JSON array.                                                                                                                                                                                                    | `invalid-input`        | `not-an-array`         | n/a | `row-vectors.json`: an object, a string, a literal                              |
| R-05 | No element is a JSON number. An element that begins with `-` or a digit, and whose token up to the next space, tab, comma, `]` or end of line is a number in the grammar of RFC 8259, is well-formed JSON that is forbidden. | `invalid-input`        | `numeric-literal`      | n/a | `row-vectors.json`: cases named "number: …"                                     |
| R-06 | No element is an array or an object.                                                                                                                                                                                         | `invalid-input`        | `nested-value`         | n/a | `row-vectors.json`: two cases                                                   |
| R-07 | Strings are well formed.                                                                                                                                                                                                     | `invalid-input`        | `invalid-string`       | n/a | `row-vectors.json`: six cases                                                   |
| R-08 | Nothing but spaces and tabs follows the array.                                                                                                                                                                               | `invalid-input`        | `trailing-content`     | n/a | `row-vectors.json`: four cases                                                  |
| R-09 | The array is otherwise well formed, with only spaces and tabs between tokens. An element that begins with `-` or a digit and whose token is not a JSON number is not JSON.                                                   | `invalid-input`        | `malformed-row`        | n/a | `row-vectors.json`: cases named "not a number: …", and nine more                |
| R-10 | The array has one element per column. A short row names its first missing column.                                                                                                                                            | `schema-mismatch`      | `arity-mismatch`       | n/a | `row-vectors.json`: four cases                                                  |
| R-11 | A null appears only in a nullable column.                                                                                                                                                                                    | `schema-mismatch`      | `null-not-allowed`     | n/a | `value-vectors.json`: cells                                                     |
| R-12 | A boolean appears only in a `boolean` column and a string only in another column.                                                                                                                                            | `schema-mismatch`      | `wrong-scalar-kind`    | n/a | `value-vectors.json`: cells                                                     |
| R-13 | Lines are judged in order, arity before elements, elements in schema order; the first failure is reported.                                                                                                                   | as the rule that fails | as the rule that fails | n/a | `row-vectors.json`: arity before values, first fault in a row, first faulty row |

A reader MAY refuse a line above a length limit of its own with
`resource-limit`, `line-too-long`. This standard sets no limit, so there is no
control.

## 4. Values

The accepted spelling of a value depends on the type and parameters of its
column, so the schemas give no pattern for values.

| No.  | Rule                                                                                                                                                                  | Category               | Rule identifier                 | JS  | Control                                                                                   |
| ---- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------- | ------------------------------- | --- | ----------------------------------------------------------------------------------------- |
| V-01 | An exact number is not empty.                                                                                                                                         | `invalid-input`        | `exact-empty`                   | n/a | `value-vectors.json`: integer, decimal, cells                                             |
| V-02 | An exact number has only digits, one leading minus sign and one point.                                                                                                | `invalid-input`        | `exact-invalid-character`       | n/a | `value-vectors.json`: integer, decimal                                                    |
| V-03 | A point has a digit on each side and occurs once.                                                                                                                     | `invalid-input`        | `exact-malformed`               | n/a | `value-vectors.json`: integer, decimal                                                    |
| V-04 | There is no redundant leading zero.                                                                                                                                   | `invalid-input`        | `exact-leading-zero`            | n/a | `value-vectors.json`: integer, decimal                                                    |
| V-05 | There are at most 38 significant digits.                                                                                                                              | `resource-limit`       | `exact-too-many-digits`         | n/a | `value-vectors.json`: integer, decimal                                                    |
| V-06 | Zero has no minus sign.                                                                                                                                               | `invalid-input`        | `exact-negative-zero`           | n/a | `value-vectors.json`: integer, decimal                                                    |
| V-07 | A point is present exactly when the column is a `decimal` of positive scale.                                                                                          | `schema-mismatch`      | `exact-point-mismatch`          | n/a | `value-vectors.json`: integer, decimal                                                    |
| V-08 | The digits after the point number exactly the scale.                                                                                                                  | `schema-mismatch`      | `exact-scale-mismatch`          | n/a | `value-vectors.json`: decimal                                                             |
| V-09 | The significant digits do not exceed the precision.                                                                                                                   | `schema-mismatch`      | `exact-precision-exceeded`      | n/a | `value-vectors.json`: decimal                                                             |
| V-10 | V-01 to V-06 are judged, in the order of the standard, before V-07 to V-09.                                                                                           | as the rule that fails | as the rule that fails          | n/a | `value-vectors.json`: entries with a note                                                 |
| V-11 | A date or timestamp has the required shape; a point after the seconds is followed by a digit.                                                                         | `invalid-input`        | `temporal-malformed`            | n/a | `value-vectors.json`: date, local-timestamp, offset-timestamp                             |
| V-12 | The date exists in the proleptic Gregorian calendar, years 0001 to 9999.                                                                                              | `invalid-input`        | `temporal-invalid-date`         | n/a | `value-vectors.json`: date, local-timestamp, offset-timestamp                             |
| V-13 | Hour, minute and second are in range; no `24:00`, no leap second.                                                                                                     | `invalid-input`        | `temporal-invalid-time`         | n/a | `value-vectors.json`: local-timestamp, offset-timestamp                                   |
| V-14 | What follows the seconds and the fraction is nothing, `Z` or a signed `hh:mm` in range, in a column of either timestamp type.                                         | `invalid-input`        | `temporal-invalid-offset`       | n/a | `value-vectors.json`: local-timestamp, offset-timestamp                                   |
| V-15 | The offset is not `-00:00`, in a column of either timestamp type.                                                                                                     | `invalid-input`        | `temporal-negative-zero-offset` | n/a | `value-vectors.json`: local-timestamp, offset-timestamp                                   |
| V-16 | The fraction has exactly the declared number of digits, and is absent when that is 0.                                                                                 | `schema-mismatch`      | `temporal-fraction-mismatch`    | n/a | `value-vectors.json`: local-timestamp, offset-timestamp                                   |
| V-17 | A local timestamp has no offset and an offset timestamp has one. A well-formed timestamp of the other type is a mismatch with the schema, not malformed input.        | `schema-mismatch`      | `temporal-offset-mismatch`      | n/a | `value-vectors.json`: local-timestamp (offset present), offset-timestamp (offset absent)  |
| V-18 | V-11 to V-15 are judged, in the order of the standard, before V-16 and V-17; V-16 before V-17. A text that is not a well-formed timestamp is never `schema-mismatch`. | as the rule that fails | as the rule that fails          | n/a | `value-vectors.json`: entries with a note; `row-vectors.json`: cases named "timestamp: …" |

## 5. Conversion policy

| No.  | Rule                                                                                                                             | Category          | Rule identifier                | JS    | Control                                                                                                |
| ---- | -------------------------------------------------------------------------------------------------------------------------------- | ----------------- | ------------------------------ | ----- | ------------------------------------------------------------------------------------------------------ |
| P-01 | Quantize entries name distinct columns.                                                                                          | `invalid-input`   | `policy-quantize-duplicate`    | no    | `fixtures/semantic-invalid/conversion-policy/quantize-column-repeated.json`                            |
| P-02 | A quantize entry names a column of the schema.                                                                                   | `schema-mismatch` | `quantize-unknown-column`      | no    | `fixtures/semantic-invalid/conversion-policy/quantize-column-unknown.json`                             |
| P-03 | That column is `decimal`.                                                                                                        | `schema-mismatch` | `quantize-not-decimal`         | no    | `fixtures/semantic-invalid/conversion-policy/quantize-column-not-decimal.json`                         |
| P-04 | The target scale does not exceed the column's declared precision.                                                                | `schema-mismatch` | `quantize-scale-too-large`     | no    | `fixtures/semantic-invalid/conversion-policy/quantize-scale-exceeds-precision.json`                    |
| P-05 | A quantized value fits the declared precision.                                                                                   | `schema-mismatch` | `quantize-overflow`            | n/a   | `value-vectors.json`: quantize                                                                         |
| P-06 | Rounding follows the mode, in exact arithmetic, with mathematical direction.                                                     | none              | none                           | n/a   | `value-vectors.json`: quantize, accepted entries                                                       |
| P-07 | Every namespace under `extensions` is one the reader implements. The reserved `example` namespaces are implemented by no reader. | `invalid-input`   | `policy-extension-unsupported` | no    | `fixtures/semantic-invalid/conversion-policy/extension-unknown.json`, `extensions-at-depth-bound.json` |
| P-08 | A destination setting written outside `extensions` is an unknown member.                                                         | `invalid-input`   | `policy-member-unknown`        | shape | `fixtures/invalid/conversion-policy/member-unknown.json`                                               |

P-02 to P-04 relate a policy to a schema. The manifest names the schema each
control is checked against.

The following obligations of a producer cannot be checked from documents
alone. A conformance suite checks them by reading an output.

| No.  | Obligation                                                                                         |
| ---- | -------------------------------------------------------------------------------------------------- |
| P-09 | A column has one representation, fixed before its first value is written.                          |
| P-10 | Under `require-exact`, the first value that is not eligible refuses the run at its row and column. |
| P-11 | Under `text-if-needed`, a column with a value that is not eligible is text in every non-null cell. |
| P-12 | The scale of a value changes only through a quantize entry.                                        |
| P-13 | `exact_numbers` changes nothing in a column that is not an exact-number column.                    |

## 6. Conversion result

Order of reading. These rules are ordered among themselves and come after
D-01 to D-08; a consumer reports the first that fails.

| No.  | Rule                                                                                                                                                         | Category        | Rule identifier                                        | JS    | Control                                                                                                                                              |
| ---- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------- | ------------------------------------------------------ | ----- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| O-01 | `contract` is present, and is a revision the consumer implements.                                                                                            | `invalid-input` | `result-member-missing`, `result-contract-unsupported` | shape | `fixtures/invalid/conversion-result/contract-missing.json`, `contract-other-revision.json`; before O-02: `contract-other-and-status-unknown.json`    |
| O-02 | `status` is present, and is one of the four values.                                                                                                          | `invalid-input` | `result-member-missing`, `result-member-invalid`       | shape | `fixtures/invalid/conversion-result/status-missing.json`, `status-unknown.json`                                                                      |
| O-03 | The members the status requires are present and the members it excludes are absent. Only presence is examined.                                               | `invalid-input` | `result-member-missing`, `result-member-unknown`       | shape | `fixtures/invalid/conversion-result/ok-without-output.json`, `refused-with-output.json`; before any other rule: `refused-with-output-malformed.json` |
| O-04 | `output`, `counts` and the entries of `inputs` are read only after O-03 has passed with status `ok`; `refusal` only after it has passed with another status. | none            | none                                                   | no    | `matching-vectors.json`: cases whose `decided_by` is M-04 or M-05                                                                                    |

O-03 is rule C-02 and C-04 below, placed in the order. O-04 is an obligation
on how a consumer is written. It is observable: a consumer that reads
`output` first reports a different rule for `refused-with-output-malformed.json`,
or acts on a digest that a refused result has no right to carry.

Rules within one result:

| No.  | Rule                                                                                                                                                        | Category        | Rule identifier                                  | JS    | Control                                                                                               |
| ---- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------- | ------------------------------------------------ | ----- | ----------------------------------------------------------------------------------------------------- |
| C-01 | `contract` is a revision the consumer implements.                                                                                                           | `invalid-input` | `result-contract-unsupported`                    | shape | `fixtures/invalid/conversion-result/contract-other-revision.json`                                     |
| C-02 | `output` and `counts` are present exactly when `status` is `ok`; `refusal` exactly when it is not.                                                          | `invalid-input` | `result-member-missing`, `result-member-unknown` | shape | `fixtures/invalid/conversion-result/ok-without-output.json`, `refused-with-counts.json` and four more |
| C-03 | The category of `refusal` is one the status allows.                                                                                                         | `invalid-input` | `result-member-invalid`                          | shape | `fixtures/invalid/conversion-result/refused-category-io-failure.json` and two more                    |
| C-04 | An `ok` result has `inputs.source`, `inputs.schema` and `inputs.policy`.                                                                                    | `invalid-input` | `result-member-missing`                          | shape | `fixtures/invalid/conversion-result/ok-without-policy-input.json`                                     |
| C-05 | Column identifiers in `counts.columns` are unique.                                                                                                          | `invalid-input` | `result-counts-inconsistent`                     | no    | `fixtures/semantic-invalid/conversion-result/column-id-repeated.json`                                 |
| C-06 | `null` does not exceed `rows`; `empty` does not exceed `rows - null`.                                                                                       | `invalid-input` | `result-counts-inconsistent`                     | no    | `null-exceeds-rows.json`, `empty-exceeds-non-null.json`                                               |
| C-07 | On a column with the five counters, `numeric + text_fallback + text_forced = rows - null`.                                                                  | `invalid-input` | `result-counts-inconsistent`                     | no    | `counters-do-not-sum.json`                                                                            |
| C-08 | Only the counter that matches `representation` is above zero: `numeric` for `native`, `text_fallback` for `text-fallback`, `text_forced` for `text-forced`. | `invalid-input` | `result-counts-inconsistent`                     | no    | `native-with-text-forced.json`, `text-fallback-with-numeric.json`                                     |
| C-09 | `scale_changed` is 0 or `rows - null`.                                                                                                                      | `invalid-input` | `result-counts-inconsistent`                     | no    | `scale-changed-partial.json`                                                                          |
| C-10 | `quantized` does not exceed `scale_changed`.                                                                                                                | `invalid-input` | `result-counts-inconsistent`                     | no    | `quantized-exceeds-non-null.json`, `quantized-exceeds-scale-changed.json`                             |

Controls without a directory are in
`fixtures/semantic-invalid/conversion-result/`.

Rules that relate a result to the schema it names under `inputs.schema`. A
consumer applies them when it holds that schema. In the fixtures the digests
are illustrative: `with_schema` and `with_policy` in the manifest stand for the
documents the result names.

| No.  | Rule                                                                                                                    | Category        | Rule identifier          | JS  | Control                                                       |
| ---- | ----------------------------------------------------------------------------------------------------------------------- | --------------- | ------------------------ | --- | ------------------------------------------------------------- |
| C-11 | `counts.columns` has the schema's columns, by identifier, in schema order.                                              | `invalid-input` | `result-schema-mismatch` | no  | `column-missing.json`, `columns-out-of-order.json`            |
| C-12 | The five counters are present exactly on `integer` and `decimal` columns.                                               | `invalid-input` | `result-schema-mismatch` | no  | `counters-on-boolean.json`, `counters-absent-on-integer.json` |
| C-13 | A `string` column is `text`. An exact-number column is not `text`. No other column is `text-fallback` or `text-forced`. | `invalid-input` | `result-schema-mismatch` | no  | `string-as-native.json`                                       |
| C-14 | `null` is 0 on a column that is not nullable.                                                                           | `invalid-input` | `result-schema-mismatch` | no  | `null-in-non-nullable.json`                                   |
| C-15 | `empty` is 0 on a column that is not `string`.                                                                          | `invalid-input` | `result-schema-mismatch` | no  | `empty-in-non-string.json`                                    |
| C-16 | `quantized` and `scale_changed` are 0 on a column that is not `decimal`.                                                | `invalid-input` | `result-schema-mismatch` | no  | `quantized-on-integer.json`                                   |

Rules that relate a result to the policy it names under `inputs.policy`. A
consumer applies them when it holds that policy.

| No.  | Rule                                                                                                                                                                                         | Category        | Rule identifier          | JS  | Control                                                                                                           |
| ---- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------- | ------------------------ | --- | ----------------------------------------------------------------------------------------------------------------- |
| C-17 | Under `text-always` every exact-number column is `text-forced`.                                                                                                                              | `invalid-input` | `result-policy-mismatch` | no  | `policy-text-always-with-native.json`                                                                             |
| C-18 | Under `require-exact` every exact-number column is `native`.                                                                                                                                 | `invalid-input` | `result-policy-mismatch` | no  | `policy-require-exact-with-text-fallback.json`                                                                    |
| C-19 | Under `text-if-needed` every exact-number column is `native` or `text-fallback`.                                                                                                             | `invalid-input` | `result-policy-mismatch` | no  | `policy-text-if-needed-with-text-forced.json`                                                                     |
| C-20 | `quantized` and `scale_changed` are 0 on a column the policy does not quantize, and `scale_changed` is 0 when the target scale equals the declared scale.                                    | `invalid-input` | `result-policy-mismatch` | no  | `policy-without-quantize.json`                                                                                    |
| C-21 | `scale_changed` equals `rows - null` on a column the policy quantizes to a target scale other than the declared scale, even when no value changes. It is therefore 0 when every row is null. | `invalid-input` | `result-policy-mismatch` | no  | `policy-quantize-scale-not-counted.json`; accepted: `fixtures/valid/conversion-result/ok-quantized-all-null.json` |

C-21 is the converse of the last clause of C-20 and, like that clause, needs
the schema as well as the policy, for the declared scale. It is a rule of its
own so that C-20 keeps the meaning it had and each has its own control. C-09
alone does not establish it: C-09 allows 0, and
`scale-changed-partial.json`, which reports 1 of 2 non-null cells, is refused
by C-09 with `result-counts-inconsistent` before any policy is consulted.

The identifiers in this section name checks made by a consumer. A consumer
does not write a result of its own, so they are diagnostics and never appear
in a `refusal`.

## 7. Extensions

`<kind>` is `policy` or `result`. X-01 to X-06 are checked without knowing any
namespace, and before P-07.

| No.  | Rule                                                                                                                                         | Category        | Rule identifier         | JS    | Control                                                                                                                                                     |
| ---- | -------------------------------------------------------------------------------------------------------------------------------------------- | --------------- | ----------------------- | ----- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| X-01 | A namespace matches `[a-z][a-z0-9]*(-[a-z0-9]+)*` and has at most 64 characters.                                                             | `invalid-input` | `<kind>-member-invalid` | shape | `fixtures/invalid/conversion-policy/extension-namespace-malformed.json`; `fixtures/invalid/conversion-result/extension-namespace-malformed.json`            |
| X-02 | A namespace value is an object.                                                                                                              | `invalid-input` | `<kind>-not-an-object`  | shape | `fixtures/invalid/conversion-policy/extension-not-an-object.json`; `fixtures/invalid/conversion-result/extension-not-an-object.json`                        |
| X-03 | There are at most 8 namespaces.                                                                                                              | `invalid-input` | `<kind>-member-invalid` | shape | `fixtures/invalid/*/extensions-too-many-namespaces.json`                                                                                                    |
| X-04 | There are at most 256 values under `extensions`: every member value and every array element, at every depth, the namespace objects included. | `invalid-input` | `<kind>-member-invalid` | no    | `fixtures/semantic-invalid/*/extensions-too-many-values.json`                                                                                               |
| X-05 | No string under `extensions`, member names included, exceeds 256 code points.                                                                | `invalid-input` | `<kind>-member-invalid` | shape | `fixtures/invalid/*/extension-string-too-long.json`                                                                                                         |
| X-06 | No number under `extensions` exceeds 9007199254740991.                                                                                       | `invalid-input` | `<kind>-member-invalid` | shape | `fixtures/invalid/*/extension-number-past-max.json`                                                                                                         |
| X-07 | The nesting bound of D-07 applies under `extensions`: a namespace object is at depth 3 and its content reaches depth 6 at most.              | `invalid-input` | `<kind>-malformed`      | shape | `fixtures/invalid/*/extensions-past-depth-bound.json`; accepted at the bound: `fixtures/valid/conversion-result/ok-extensions-at-depth-bound.json`          |
| X-08 | A result consumer ignores a namespace it does not know, once X-01 to X-07 hold.                                                              | none            | none                    | no    | `fixtures/valid/conversion-result/ok-extensions.json`, `ok-extensions-at-depth-bound.json`, `ok-extensions-at-count-bounds.json`, `refused-extensions.json` |
| X-09 | Extension content that restates or contradicts a defined member is ignored like any other.                                                   | none            | none                    | no    | `fixtures/valid/conversion-result/ok-extensions-restating.json`; `matching-vectors.json`: extension restates the status                                     |
| X-10 | Every interpretation this standard requires is unchanged when an `extensions` member that satisfies X-01 to X-07 is deleted.                 | none            | none                    | no    | see below                                                                                                                                                   |

`*` in a control stands for `conversion-policy` and `conversion-result`.

The control for X-10 is a procedure, not a file. For every result under
`fixtures/valid/` that has an `extensions` member, and for every case of
`matching-vectors.json` whose result has one that satisfies X-01 to X-07,
delete the member and repeat the check: the document is still accepted, its status, run identifier, bindings,
counts and refusal are the same values, and the matching verdict and deciding
rule are the same. For every policy under `fixtures/semantic-invalid/` that is
refused with `policy-extension-unsupported`, delete the member: the policy is
then accepted with the same `exact_numbers` and `quantize`. The JSON Schemas
do not list `extensions` as required in either document.

The value count of X-04, stated exactly: `extensions` itself is not counted;
each of its members counts one; inside any object every member counts one and
inside any array every element counts one, whatever the kind of the value.
`{"example-tool": {"a": [1, 2], "b": {}}}` has five values: the namespace
object, `a`, its two elements, and `b`.

## 8. Matching a result to an output and to a request

The matcher takes a result or its absence, a candidate output or its absence,
and the caller's expectations: `run_id`, `source`, `schema` and `policy`, each
supplied or not. The verdict is `requested-run-completed`, `output-agreement`
or `not-established`. The steps are applied in the order of their numbers; the
first that fails decides, with verdict `not-established`.

| No.  | Step                                                                                                                                                                  | Applies           | Rule identifier                  | Control (`matching-vectors.json`)             |
| ---- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------- | -------------------------------- | --------------------------------------------- |
| M-01 | All four expectations are supplied, or none is.                                                                                                                       | always            | `expectations-incomplete`        | cases named "missing expectation: …"          |
| M-02 | Every supplied expectation is well formed: `run_id` of 1 to 128 permitted characters; lengths from 0 to 9007199254740991; digests of 64 lowercase hexadecimal digits. | always            | `expectation-invalid`            | cases named "expectation: …"                  |
| M-03 | A result is held.                                                                                                                                                     | always            | `result-missing`                 | "missing result", in both modes               |
| M-04 | The result is accepted under sections 1, 6 and 7, read in the order O-01 to O-03.                                                                                     | always            | as the rule that fails           | cases named "result: …"                       |
| M-05 | `status` is `ok`.                                                                                                                                                     | always            | `result-not-ok`                  | cases named "status: …"                       |
| M-06 | `run_id` equals the expected run identifier, compared code point for code point.                                                                                      | all four supplied | `binding-run-id-mismatch`        | "stale run", "run id differs in letter case"  |
| M-07 | `inputs.source.bytes` equals the expected length.                                                                                                                     | all four supplied | `binding-source-length-mismatch` | "different source of another length"          |
| M-08 | `inputs.source.sha256` equals the expected digest.                                                                                                                    | all four supplied | `binding-source-digest-mismatch` | "different source, identical output"          |
| M-09 | `inputs.schema.bytes` equals the expected length.                                                                                                                     | all four supplied | `binding-schema-length-mismatch` | "schema length differs"                       |
| M-10 | `inputs.schema.sha256` equals the expected digest.                                                                                                                    | all four supplied | `binding-schema-digest-mismatch` | "different schema, identical output"          |
| M-11 | `inputs.policy.bytes` equals the expected length.                                                                                                                     | all four supplied | `binding-policy-length-mismatch` | "policy length differs"                       |
| M-12 | `inputs.policy.sha256` equals the expected digest.                                                                                                                    | all four supplied | `binding-policy-digest-mismatch` | "different policy, identical output"          |
| M-13 | A candidate output is held.                                                                                                                                           | always            | `output-missing`                 | "missing output"                              |
| M-14 | The candidate's length in bytes equals `output.bytes`.                                                                                                                | always            | `output-length-mismatch`         | "output length differs", "output truncated"   |
| M-15 | The candidate's SHA-256 equals `output.sha256`.                                                                                                                       | always            | `output-digest-mismatch`         | "output digest differs"                       |
| M-16 | Every applicable step passed: the verdict is `requested-run-completed` when all four expectations were supplied and `output-agreement` when none was.                 | always            | none                             | "requested run completed", "output agreement" |

Further obligations, each with a control:

| No.  | Obligation                                                                                                                                                         | Control (`matching-vectors.json`)                                                               |
| ---- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------- |
| M-17 | With no expectations, a stale result beside the same output bytes is `output-agreement`. That verdict is never evidence that a particular request completed.       | "stale run, no expectations"                                                                    |
| M-18 | A partial set of expectations is never downgraded to `output-agreement` and is never checked as far as it goes: M-01 decides before a result is read.              | "missing expectation: …", including the case with no result                                     |
| M-19 | M-06 to M-15 read `inputs` and `output` only from a result that passed M-04 and M-05. A result that is not `ok` and carries an `output` member is refused at O-03. | "result: refused with an output member", "result: refused with a malformed output member"       |
| M-20 | Digests are compared as written. No letter case is folded, in the result or in an expectation.                                                                     | "expectation: digest in upper case"; `fixtures/invalid/conversion-result/digest-uppercase.json` |
| M-21 | `profile` and `extensions` take no part in the verdict.                                                                                                            | "extension restates the status"                                                                 |

Expectations are not a document of this standard and have no wire format. In
`matching-vectors.json` they are written as an object with the members
supplied; an absent member is an expectation that was not supplied.

## Coverage

Every registered rule identifier in `contract.json` has at least one control
above, except these, which depend on a limit of the reader: `schema-too-large`,
`policy-too-large`, `result-too-large` and `line-too-long`; and
`policy-not-utf8` and `result-not-utf8`, which are the same rule as
`schema-not-utf8`, which has a control. Every matching rule identifier has at
least one case in `matching-vectors.json`.

## Executing the controls

`fixtures/manifest.json` lists every fixture with its expected outcome. The
vectors state theirs case by case. A reader is checked by running them and
comparing category, rule and location; a consumer is checked by running the
result fixtures and the matching vectors, mutations included. A checker
written by the author of the vectors shares the author's reading of the
standard, so agreement between the two is a check of consistency. Conformance
of the contract itself is shown by a consumer written independently.
