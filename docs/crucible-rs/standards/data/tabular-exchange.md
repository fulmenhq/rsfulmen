---
title: "Tabular Exchange Standard"
description: "Format-neutral contract for typed tables: value grammar, table schema, rows encoding, conversion policy and conversion result"
author: "Crucible maintainers"
date: "2026-11-19"
last_updated: "2026-11-19"
status: "draft"
tags: ["standards", "data", "tabular-exchange", "schemas", "conversion"]
---

# Tabular Exchange Standard

## Scope

This standard defines how a typed table is described and carried between
programs, how a conversion of such a table is instructed, and how the outcome
of a conversion is reported. It is independent of any file format that a table
is converted to or from, and of any one implementation.

It defines:

- the logical types of a column and the exact spelling of their values;
- the table schema document;
- the rows encoding;
- the conversion policy document;
- the conversion result document;
- the categories and rule identifiers of a refusal;
- the rules for matching a result to an output and to the request a caller
  made.

Contract revision: `tabular-exchange/0.1.0`.

The JSON Schemas are in `schemas/data/tabular-exchange/v0.1.0/`. Examples,
fixtures and vectors are in `config/data/tabular-exchange/v0.1.0/`. The rules
that JSON Schema cannot express are listed in
[What the JSON Schemas do not express](#what-the-json-schemas-do-not-express)
and, one by one, in `schemas/data/tabular-exchange/v0.1.0/semantic-validation.md`.

The words MUST, MUST NOT, SHOULD and MAY are used as in RFC 2119.

## Terms

- **Conversion**: one run that reads a source, a table schema and a conversion
  policy and writes one output.
- **Producer**: the program that performs a conversion and writes its result.
- **Reader**: any program that reads a schema, a policy, rows or a result. A
  producer is a reader of its inputs.
- **Consumer**: a reader of a result, usually in another project.
- **Destination**: the format of the output.
- **Adapter profile**: the definition, owned by an implementation, of how one
  conversion maps the table model onto one destination, including its limits.
- **Exact-number column**: a column of type `integer` or `decimal`.
- **Refuse**: stop without producing an output and report a category and a
  rule. A reader never repairs, skips or normalizes what it refuses.

## Three revisions

Three revisions exist and are never inferred from one another.

| Revision                 | Where it appears                     | What it identifies                                  |
| ------------------------ | ------------------------------------ | --------------------------------------------------- |
| Wire-format revision     | the `format` member of each document | the shape of that document                          |
| Contract revision        | the `contract` member of a result    | the revision of this standard the producer followed |
| Adapter profile revision | the `profile` member of a result     | the implementation's mapping onto one destination   |

A contract revision states which wire-format revisions it defines. Revision
`tabular-exchange/0.1.0` defines `tabular-exchange.schema/0`,
`tabular-exchange.policy/0` and `tabular-exchange.result/0`.

## Table model

A table is a schema and a sequence of rows.

A schema is an ordered list of at least one column. A column has:

- an **identifier**: a non-empty string, unique within the schema.
  Identifiers are compared as exact sequences of Unicode code points, without
  normalization or case folding;
- a **label**: display text. A label may be empty and labels may repeat;
- a **type**, from the closed list below;
- a **nullable** flag.

A row has exactly one value per column, matched by position.

### Types

| Type               | Parameters                                    | A value is                                               |
| ------------------ | --------------------------------------------- | -------------------------------------------------------- |
| `string`           | none                                          | a sequence of Unicode scalar values, carried unchanged   |
| `boolean`          | none                                          | true or false                                            |
| `integer`          | none                                          | an exact integer of at most 38 digits                    |
| `decimal`          | `precision` 1 to 38, `scale` 0 to `precision` | an exact decimal: a signed coefficient times 10^-`scale` |
| `date`             | none                                          | a calendar date                                          |
| `local-timestamp`  | `fraction` 0 to 9                             | a date and time of day with no offset                    |
| `offset-timestamp` | `fraction` 0 to 9                             | a date and time of day with a fixed offset from UTC      |

The list is closed. A type outside it is refused as unsupported. Binary
floating point, bytes, time of day without a date, named time zones and nested
values are not in the list, and a reader MUST NOT flatten or stringify a value
to make it fit.

### Null, empty and missing

A null is not an empty string. Any column type may be nullable. A null in a
column that is not nullable is refused. A row with fewer or more values than
the schema has columns is refused; a missing value is never read as a null.

### Identity of values

A decimal's scale is part of the value: `1.0` and `1.00` are different values.
A timestamp is the fields as written: `Z` and `+00:00` are different offsets,
and two offset timestamps that denote the same instant at different offsets are
different values. This standard defines no ordering of timestamps across
fractional precisions or offsets. No value is converted to another offset, and
no host time zone or clock is consulted.

## Values

Every value other than a null or a boolean is carried as a string. This section
gives the only accepted spelling of each. A spelling outside it is refused; it
is never normalized into an accepted one. A string column accepts any string,
including one that looks like a number or a date.

Nothing in the reading, comparison or rounding of an exact number goes through
binary floating point.

### Exact numbers

The grammar, for every exact-number column:

```text
exact    = [ "-" ] whole [ "." 1*DIGIT ]
whole    = "0" / ( nonzero *DIGIT )
nonzero  = %x31-39
DIGIT    = %x30-39
```

Only the ASCII digits, one leading `-` and one `.` occur. There is no plus
sign, no whitespace, no exponent, no digit grouping and no redundant leading
zero. Zero is never written with a minus sign, at any scale.

The **significant digits** of a spelling are its digits with the point removed
and leading zeros removed; zero has one significant digit.

- An `integer` value has no point.
- A `decimal` value in a column of scale 0 has no point.
- A `decimal` value in a column of scale `s` greater than 0 has a point
  followed by exactly `s` digits.
- A `decimal` value has at most `precision` significant digits.
- Every exact number has at most 38 significant digits.

A reader judges an exact number in this order and reports the first rule that
fails. Rules 1 to 6 do not depend on the column's type.

| Order | Condition                                                                                          | Category          | Rule                       |
| ----- | -------------------------------------------------------------------------------------------------- | ----------------- | -------------------------- |
| 1     | nothing follows the optional minus sign                                                            | `invalid-input`   | `exact-empty`              |
| 2     | a character other than a digit or a point follows the optional minus sign                          | `invalid-input`   | `exact-invalid-character`  |
| 3     | a point with no digit before it or after it, or more than one point                                | `invalid-input`   | `exact-malformed`          |
| 4     | more than one digit before the point and the first is `0`                                          | `invalid-input`   | `exact-leading-zero`       |
| 5     | more than 38 significant digits                                                                    | `resource-limit`  | `exact-too-many-digits`    |
| 6     | a minus sign and a value of zero                                                                   | `invalid-input`   | `exact-negative-zero`      |
| 7     | a point in an `integer` or in a `decimal` of scale 0, or no point in a `decimal` of positive scale | `schema-mismatch` | `exact-point-mismatch`     |
| 8     | the number of digits after the point differs from the scale                                        | `schema-mismatch` | `exact-scale-mismatch`     |
| 9     | more significant digits than the precision                                                         | `schema-mismatch` | `exact-precision-exceeded` |

Examples: `decimal(7,4)` accepts `123.4500` and `decimal(6,4)` refuses it;
`decimal(2,2)` accepts `0.01` and refuses `1.00`; `000123` is refused in an
`integer` column and unchanged in a `string` column.

### Dates

`YYYY-MM-DD`: exactly ten characters, ASCII digits and two hyphens, in the
proleptic Gregorian calendar, years `0001` to `9999`.

| Order | Condition                                                                               | Category        | Rule                    |
| ----- | --------------------------------------------------------------------------------------- | --------------- | ----------------------- |
| 1     | not ten characters of the form digits, hyphen, digits, hyphen, digits                   | `invalid-input` | `temporal-malformed`    |
| 2     | year `0000`, month outside 01 to 12, or a day that the month does not have in that year | `invalid-input` | `temporal-invalid-date` |

### Timestamps

A `local-timestamp` is `YYYY-MM-DDThh:mm:ss` followed by the fraction. An
`offset-timestamp` is the same followed by an offset.

- The separator is an uppercase `T`. Seconds are mandatory.
- Hours are `00` to `23`; minutes and seconds are `00` to `59`. There is no
  `24:00` and no leap second.
- The fraction is a point followed by exactly `fraction` digits. When
  `fraction` is 0 there is no point.
- The offset is `Z`, or a sign followed by `hh:mm` with `hh` from `00` to `23`
  and `mm` from `00` to `59`. `-00:00` is refused in every column. `Z` and
  `+00:00` are both accepted and stay distinct.
- A `local-timestamp` has no offset. An `offset-timestamp` always has one.

A timestamp is judged in two parts. Rules 1 to 7 ask whether the text is a
well-formed timestamp of either type; they do not depend on the column, and
each is `invalid-input`. Rules 8 and 9 ask whether a well-formed timestamp is
the one the column declares; each is `schema-mismatch`. A reader reports the
first rule that fails, so a text with a malformed calendar, clock or offset is
`invalid-input` whatever else is wrong with it, and `schema-mismatch` is
reported only for a well-formed timestamp. A fraction of more than nine digits
is well formed; no column declares that width, so it fails rule 8 in every
column.

For the rules below, the **tail** is everything after the nineteenth
character. The **fraction** is present when the tail begins with a point; it is
the point and the longest run of digits that follows it. The **offset part**
is what remains of the tail after the fraction.

| Order | Condition                                                                                                                                  | Category          | Rule                            |
| ----- | ------------------------------------------------------------------------------------------------------------------------------------------ | ----------------- | ------------------------------- |
| 1     | fewer than nineteen characters, or `T`, `:` and `:` are not at their positions                                                             | `invalid-input`   | `temporal-malformed`            |
| 2     | the first ten characters are not a date                                                                                                    | as for a date     | as for a date                   |
| 3     | a character of the hour, minute or second is not a digit                                                                                   | `invalid-input`   | `temporal-malformed`            |
| 4     | hour above 23, minute above 59 or second above 59                                                                                          | `invalid-input`   | `temporal-invalid-time`         |
| 5     | the tail begins with a point that no digit follows                                                                                         | `invalid-input`   | `temporal-malformed`            |
| 6     | the offset part is not empty and is neither `Z` nor a sign and `hh:mm` in range                                                            | `invalid-input`   | `temporal-invalid-offset`       |
| 7     | the offset part is `-00:00`                                                                                                                | `invalid-input`   | `temporal-negative-zero-offset` |
| 8     | `fraction` is 0 and a fraction is present; or `fraction` is positive and the fraction is absent or does not have exactly `fraction` digits | `schema-mismatch` | `temporal-fraction-mismatch`    |
| 9     | `local-timestamp`: the offset part is not empty; `offset-timestamp`: the offset part is empty                                              | `schema-mismatch` | `temporal-offset-mismatch`      |

Examples, for a column with `fraction` 0. `2026-11-19T08:30:00Z` in a
`local-timestamp` column and `2026-11-19T08:30:00` in an `offset-timestamp`
column are `schema-mismatch`, `temporal-offset-mismatch`: each is the other
type's value. `2026-11-19T08:30:00+24:00` and `2026-11-19T08:30:00-00:00` are
`invalid-input` in both columns. `2026-11-19T08:30:00.5+24:00` is
`invalid-input`, `temporal-invalid-offset`, although its fraction also does
not fit: it is not a well-formed timestamp.

### Booleans, nulls and strings

A boolean is the JSON literal `true` or `false`. A null is the JSON literal
`null`. A string value is any JSON string. A string in a `boolean` column, and
a boolean in any other column, is refused with `schema-mismatch`,
`wrong-scalar-kind`. A null in a column that is not nullable is refused with
`schema-mismatch`, `null-not-allowed`; this is judged before the kind of the
value.

`value-vectors.json` gives accepted and refused spellings of every type with
the expected category and rule.

## Documents

The schema, the policy and the result are documents. The rules of this section
apply to all three; `<kind>` stands for `schema`, `policy` or `result` in a
rule identifier.

1. A document is one JSON value encoded as UTF-8 with no byte order mark.
2. No object repeats a key, at any depth. Keys are compared after string
   escapes are decoded.
3. A number is written `0` or as a nonzero digit followed by digits: no sign,
   no fraction, no exponent and no leading zero. Every number in a document is
   a count or a small bound, and none exceeds 9007199254740991 (2^53 - 1), so
   that every JSON implementation reads it exactly. Everything exact is a
   string.
4. Strings follow RFC 8259. An unpaired surrogate escape, an unknown escape
   and an unescaped control character are refused.
5. Arrays and objects nest at most 3 deep in a schema and at most 6 deep in a
   policy or a result. The document's own object is depth 1. The members this
   standard defines reach depth 3 in a schema and in a policy and depth 4 in a
   result; depths 5 and 6 can be reached only under `extensions`.
6. The value is an object with a `format` member naming the document's kind and
   wire-format revision. A reader checks `format` before it examines any other
   member and refuses a value it does not support, including the absence of
   the member and a `format` that is not a string.
7. A member that this standard does not define is refused, at every depth,
   including a member that is defined for another type, status or
   representation than the one the document states. The only open place is the
   `extensions` member of a policy and of a result, and it is bounded: see
   [Extensions](#extensions).
8. No member names a path, and no member carries a value from the data.

A reader judges a document in this order and reports the first rule that
fails.

| Order | Condition                                                                        | Category         | Rule                        |
| ----- | -------------------------------------------------------------------------------- | ---------------- | --------------------------- |
| 1     | the document exceeds a size limit of the reader                                  | `resource-limit` | `<kind>-too-large`          |
| 2     | the bytes are not UTF-8                                                          | `invalid-input`  | `<kind>-not-utf8`           |
| 3     | rules 1 to 5 above are broken, or the text is not exactly one JSON value         | `invalid-input`  | `<kind>-malformed`          |
| 4     | the value is not an object                                                       | `invalid-input`  | `<kind>-not-an-object`      |
| 5     | `format` is absent, not a string or not supported                                | `invalid-input`  | `<kind>-format-unsupported` |
| 6     | a required member is absent                                                      | `invalid-input`  | `<kind>-member-missing`     |
| 6     | a member is not defined, or does not apply                                       | `invalid-input`  | `<kind>-member-unknown`     |
| 6     | a member has the wrong JSON type or a value outside its range or its closed list | `invalid-input`  | `<kind>-member-invalid`     |
| 6     | a member or an array entry that must be an object is not one                     | `invalid-input`  | `<kind>-not-an-object`      |

Steps 1 to 5 are ordered. When a document breaks more than one rule at step 6,
or a rule at step 6 and a rule specific to its kind, which of them is reported
is not specified, with two exceptions that are ordered: the members of a result
are read in the order given under
[Order of reading](#order-of-reading), and the bounds on `extensions` are
judged before any namespace is looked up. A size limit is the reader's own;
this standard sets none.

`<kind>-malformed` means that the text is outside the document grammar of this
standard. It covers text that is not JSON at all and text that is JSON but is
excluded by rules 1 to 5: a byte order mark, a repeated key, a number such as
`1.0`, `1e1` or `-1`, and excess nesting. A number that is spelled as rule 3
requires and is too large for its member is not malformed; it is
`<kind>-member-invalid`, however large it is. A reader therefore MUST NOT read
a number through a type that would lose it, wrap it or refuse it as
unparseable.

## Table schema document

Wire format `tabular-exchange.schema/0`. JSON Schema:
`table-schema.schema.json`.

```json
{
  "format": "tabular-exchange.schema/0",
  "columns": [
    {
      "id": "invoice",
      "label": "Invoice",
      "type": "string",
      "nullable": false
    },
    {
      "id": "amount",
      "label": "Amount",
      "type": "decimal",
      "precision": 12,
      "scale": 4,
      "nullable": true
    },
    {
      "id": "seen",
      "label": "Seen",
      "type": "local-timestamp",
      "fraction": 3,
      "nullable": true
    }
  ]
}
```

- `columns` is required and has at least one entry.
- Every column has `id`, `label`, `type` and `nullable`. `nullable` is always
  written; there is no default.
- A `decimal` column also has `precision` and `scale`. A `local-timestamp` or
  `offset-timestamp` column also has `fraction`. These members are required
  for those types and are unknown members on any other type.

| Condition                                                                  | Category           | Rule                      |
| -------------------------------------------------------------------------- | ------------------ | ------------------------- |
| `type` is a string outside the closed list                                 | `unsupported-type` | `schema-type-unsupported` |
| no columns, an empty identifier, or an identifier that repeats             | `invalid-input`    | `schema-columns-invalid`  |
| `precision` outside 1 to 38, `scale` above `precision`, `fraction` above 9 | `invalid-input`    | `schema-member-invalid`   |

Type names are compared exactly; `String` is not `string`.

## Rows encoding

Rows travel as newline-delimited JSON. The schema is a separate document and
never a line of the row stream.

1. The input is UTF-8 with no byte order mark.
2. A line ends with LF or with CR LF. The last line may omit its terminator. A
   CR that is not directly followed by LF is part of the line.
3. An empty input has zero rows.
4. Every line is one row. A line that is empty or holds only spaces and tabs
   is refused; it is not skipped.
5. A row is one JSON array. Spaces and tabs may appear before it, after it and
   between its tokens. Nothing else may.
6. Each element is `null`, `true`, `false` or a JSON string. A JSON number is
   refused, whatever the column's type and however it is spelled. An array or
   an object is refused.
7. The array has exactly one element per schema column.
8. Each element is judged against its column as in [Values](#values).

Row indexes are zero-based and count lines of the input. There is no header
row.

| Condition                                                                                                                              | Category              | Rule                  | Located at                       |
| -------------------------------------------------------------------------------------------------------------------------------------- | --------------------- | --------------------- | -------------------------------- |
| a line exceeds a length limit of the reader                                                                                            | `resource-limit`      | `line-too-long`       | row                              |
| the input starts with a byte order mark                                                                                                | `invalid-input`       | `byte-order-mark`     | nothing                          |
| a line is not UTF-8                                                                                                                    | `invalid-input`       | `invalid-utf8`        | row                              |
| a line is empty or holds only spaces and tabs                                                                                          | `invalid-input`       | `blank-line`          | row                              |
| the first character after leading spaces and tabs is not `[`                                                                           | `invalid-input`       | `not-an-array`        | row                              |
| an element is a JSON number                                                                                                            | `invalid-input`       | `numeric-literal`     | row                              |
| an element begins with `[` or `{`                                                                                                      | `invalid-input`       | `nested-value`        | row                              |
| a string has an unknown escape, a `\u` escape without four hexadecimal digits, an unpaired surrogate or an unescaped control character | `invalid-input`       | `invalid-string`      | row                              |
| something other than spaces and tabs follows the array                                                                                 | `invalid-input`       | `trailing-content`    | row                              |
| the line is not a well-formed array for any other reason, including an element that begins like a number and is not one                | `invalid-input`       | `malformed-row`       | row                              |
| the array has fewer elements than the schema has columns                                                                               | `schema-mismatch`     | `arity-mismatch`      | row and the first missing column |
| the array has more elements than the schema has columns                                                                                | `schema-mismatch`     | `arity-mismatch`      | row                              |
| an element does not fit its column                                                                                                     | see [Values](#values) | see [Values](#values) | row and column                   |

Lines are judged in order. For one line the length limit, the byte order mark
and UTF-8 are judged in that order; the line is then judged from left to
right. The number of elements is judged before any element. Elements are then
judged in schema order. The refusal reports the first rule that fails. A
length limit is the reader's own; this standard sets none.

`numeric-literal` and `malformed-row` are told apart as follows. When an
element begins with `-` or a digit, its **token** is the run of characters from
there to the next space, tab, comma or `]`, or to the end of the line. If the
token is a number in the grammar of RFC 8259,

```text
number = [ "-" ] ( "0" / ( %x31-39 *DIGIT ) ) [ "." 1*DIGIT ] [ ( "e" / "E" ) [ "+" / "-" ] 1*DIGIT ]
```

the element is well-formed JSON that this standard forbids, and the rule is
`numeric-literal`: `1`, `-0`, `1.5` and `1e3` are examples. The line is judged
from left to right, so a number is reported even when the line is also broken
after it, for example by a missing `]`. Otherwise the line
is not JSON, and the rule is `malformed-row`: `-`, `01`, `1.`, `1e` and `1x`
are examples. An element that begins with `+` or with a point does not begin
like a number and is `malformed-row` as before, as is a string that is still
open at the end of its line. The two rules have the same
category; the distinction tells an author whether to quote a value or to
repair a serializer.

`row-vectors.json` gives inputs with the expected outcome of each.
`row.schema.json` describes one parsed row; it cannot describe the rules above.

## Conversion policy document

Wire format `tabular-exchange.policy/0`. JSON Schema:
`conversion-policy.schema.json`.

```json
{
  "format": "tabular-exchange.policy/0",
  "exact_numbers": "require-exact",
  "quantize": [{ "column": "amount", "scale": 2, "rounding": "half-even" }]
}
```

A conversion always has a policy. Nothing about the representation of a value
changes unless the policy says so.

### Representation of exact numbers

`exact_numbers` is required and has no default. It applies to every
exact-number column of the conversion. It does not apply to any other column:
`text-always` does not turn booleans, dates or strings into anything else.

An adapter profile defines which exact-number values its destination can hold
in a native numeric form from which the exact value, with its declared scale,
can be recovered. A value that qualifies is **eligible**.

| Value            | Behaviour                                                                                                                                                                                   |
| ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `require-exact`  | Every exact-number value is written in native numeric form. The first value that is not eligible refuses the run with category `unsupported-representation`, located at its row and column. |
| `text-if-needed` | A column whose non-null values are all eligible is written in native numeric form. A column with at least one value that is not eligible is written entirely as text.                       |
| `text-always`    | Every exact-number column is written as text.                                                                                                                                               |

- A column has one representation. It is fixed before the first value of the
  column is written. A column is never part native and part text.
- Text is the canonical spelling in [Values](#values), after any quantization.
- Eligibility is judged on the value after quantization.
- What a native numeric form is for a destination, and which values are
  eligible, is part of the adapter profile and not of this standard.

### Quantization

`quantize` is optional. Each entry names one `decimal` column, a target
`scale` and a `rounding` mode. A column appears in at most one entry. An empty
list means the same as an absent one.

For a column declared `decimal(p, s)` and a target scale `t`:

- `t` MUST NOT exceed `p`. The quantized column is `decimal(p, t)`: the
  precision is kept and never widened.
- When `t` is greater than `s`, the value is padded with zeros exactly.
- When `t` is less than `s`, the digits beyond `t` are removed by the rounding
  mode.
- When `t` equals `s`, nothing changes.
- A result with more than `p` significant digits refuses the run. Nothing is
  clipped.
- A negative value that becomes zero is written without a sign.
- Nulls are not quantized.

The scale of a value never changes for any other reason, and in particular
never to make a value eligible.

| Mode                  | A removed part that is not zero moves the value                              |
| --------------------- | ---------------------------------------------------------------------------- |
| `half-even`           | to the nearer neighbour; on a tie, to the neighbour whose last digit is even |
| `half-away-from-zero` | to the nearer neighbour; on a tie, to the neighbour farther from zero        |
| `toward-zero`         | to the neighbour nearer zero                                                 |
| `toward-positive`     | to the greater neighbour                                                     |
| `toward-negative`     | to the lesser neighbour                                                      |

Direction is mathematical for negative values: `-2.5` rounded to an integer is
`-2` under `toward-positive` and `-3` under `toward-negative`.

| Condition                                              | Category          | Rule                        | Located at     |
| ------------------------------------------------------ | ----------------- | --------------------------- | -------------- |
| two entries name the same column                       | `invalid-input`   | `policy-quantize-duplicate` | nothing        |
| `scale` above 38                                       | `invalid-input`   | `policy-member-invalid`     | nothing        |
| an entry names a column the schema does not have       | `schema-mismatch` | `quantize-unknown-column`   | nothing        |
| an entry names a column that is not `decimal`          | `schema-mismatch` | `quantize-not-decimal`      | nothing        |
| `scale` exceeds the declared precision of the column   | `schema-mismatch` | `quantize-scale-too-large`  | nothing        |
| a quantized value has more significant digits than `p` | `schema-mismatch` | `quantize-overflow`         | row and column |

### Policy extensions

`extensions` is optional and has the shape and the bounds given under
[Extensions](#extensions). Settings of one destination, such as a worksheet
name, and limits of one implementation belong here and nowhere else in a
policy.

A reader MUST refuse a policy that carries a namespace it does not implement,
with category `invalid-input` and rule `policy-extension-unsupported`. A
setting that is silently ignored is a conversion the author did not ask for.
The bounds are judged first: a policy whose `extensions` member breaks a bound
is refused for the bound, whether or not the reader implements the namespace.

An extension MUST NOT change the meaning of `exact_numbers` or `quantize`. A
policy from which the `extensions` member has been deleted is a complete
policy with the same meaning for those two members.

## Conversion result document

Wire format `tabular-exchange.result/0`. JSON Schema:
`conversion-result.schema.json`.

```json
{
  "format": "tabular-exchange.result/0",
  "contract": "tabular-exchange/0.1.0",
  "status": "ok",
  "run_id": "case-0001",
  "profile": { "id": "example-destination/r1", "sha256": "…" },
  "inputs": {
    "source": { "sha256": "…", "bytes": 79 },
    "schema": { "sha256": "…", "bytes": 335 },
    "policy": { "sha256": "…", "bytes": 148 }
  },
  "output": { "sha256": "…", "bytes": 6103 },
  "counts": {
    "rows": 3,
    "columns": [
      { "id": "invoice", "representation": "text", "null": 0, "empty": 1 },
      {
        "id": "amount",
        "representation": "native",
        "null": 1,
        "empty": 0,
        "numeric": 2,
        "text_fallback": 0,
        "text_forced": 0,
        "quantized": 1,
        "scale_changed": 2
      }
    ]
  }
}
```

A producer writes at most one result per run. A run that ends before a result
can be written leaves none.

### Members

| Member       | Present                        | Meaning                                         |
| ------------ | ------------------------------ | ----------------------------------------------- |
| `format`     | always                         | `tabular-exchange.result/0`                     |
| `contract`   | always                         | the contract revision the producer followed     |
| `status`     | always                         | `ok`, `refused`, `failed` or `cancelled`        |
| `run_id`     | always                         | identity of the run                             |
| `profile`    | always                         | the adapter profile the run used                |
| `inputs`     | always                         | digest and length of each input read to its end |
| `output`     | only when `status` is `ok`     | digest and length of the output                 |
| `counts`     | only when `status` is `ok`     | what the conversion did, per column             |
| `refusal`    | only when `status` is not `ok` | why the run did not complete                    |
| `extensions` | optional                       | implementation members, by namespace            |

A digest is SHA-256 over the exact bytes, written as 64 lowercase hexadecimal
digits. A length is a number of bytes. No count or length exceeds
9007199254740991, in a result of any status: the bound applies to `bytes`
under `inputs` and to `refusal.row` in a result that is not `ok` exactly as it
applies to `counts` in one that is. A count of 9007199254740992 or more is
`result-member-invalid`.

### Order of reading

A consumer reads a result in this order and reports the first rule that fails.

1. The document rules, steps 1 to 5 of [Documents](#documents): bytes,
   grammar, object, `format`.
2. `contract` is present and is a revision the consumer implements. A
   `contract` that is not a string is treated as a revision the consumer does
   not implement, as `format` is.
3. `status` is present and is one of the four values.
4. The members the status requires are present, and the members it excludes
   are absent. Status `ok` requires `output`, `counts` and all three entries
   of `inputs`, and excludes `refusal`. Every other status requires `refusal`
   and excludes `output` and `counts`. A member counts as present whatever its
   value, `null` included. For status `ok`, an `inputs` member that is absent
   is a missing member, and one that is present and is not an object is
   `result-not-an-object`. When more than one condition of this step holds,
   which is reported is not specified.
5. Every other rule, in no specified order.

| Step | Condition                                                             | Category        | Rule                          |
| ---- | --------------------------------------------------------------------- | --------------- | ----------------------------- |
| 2    | `contract` is absent                                                  | `invalid-input` | `result-member-missing`       |
| 2    | `contract` is not a string, or not a revision the consumer implements | `invalid-input` | `result-contract-unsupported` |
| 3    | `status` is absent                                                    | `invalid-input` | `result-member-missing`       |
| 3    | `status` is not one of the four values                                | `invalid-input` | `result-member-invalid`       |
| 4    | a member the status requires is absent                                | `invalid-input` | `result-member-missing`       |
| 4    | a member the status excludes is present                               | `invalid-input` | `result-member-unknown`       |

A consumer MUST NOT read the content of `output` or `counts`, or rely on an
entry of `inputs`, before step 4 has passed with status `ok`. At step 4 it
asks only whether a member is present. A result whose status is not `ok` and
which carries an `output` member is refused at step 4 without that member
being examined, so nothing placed in it can be acted on.

### Status

| Status      | Meaning                                                                                              | Categories of `refusal`                                                                                |
| ----------- | ---------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ |
| `ok`        | the output is complete and is the one `output` describes                                             | none                                                                                                   |
| `refused`   | the inputs or the policy ruled the conversion out; running it again on the same inputs refuses again | `invalid-input`, `schema-mismatch`, `unsupported-type`, `unsupported-representation`, `resource-limit` |
| `failed`    | the run could not be carried out                                                                     | `resource-limit`, `io-failure`, `internal`                                                             |
| `cancelled` | the run was asked to stop                                                                            | `cancelled`                                                                                            |

`resource-limit` is a refusal when a declared limit or a capacity of the
destination was exceeded, and a failure when the system ran out of storage or
memory. A producer need not be able to produce every status.

### Run identity

`run_id` is 1 to 128 characters from `A` to `Z`, `a` to `z`, `0` to `9`, `.`,
`_` and `-`. It is the caller's identifier when the caller supplied one, and
otherwise one the producer generates. It is not a secret and is not promised
to be unique beyond what the caller or the producer arranges.

### Profile

`profile.id` is a name followed by `/r` and the profile revision, for example
`example-destination/r1`. The name matches `[a-z][a-z0-9]*(-[a-z0-9]+)*` and
the revision is `0` or a nonzero digit followed by digits. `profile.sha256` is the digest of the profile's
definition as its owner publishes it. The definition is outside this standard.
A consumer uses the pair to tell which mapping onto the destination a result
refers to. A change to anything the definition covers is a new profile
revision.

### Inputs

`inputs` has an entry for an input only when that input was read to its end:

- `source`: the data that was converted;
- `schema`: the table schema document;
- `policy`: the conversion policy document.

An input that was absent, unreadable or read only in part has no entry. A
result with status `ok` has all three.

The `policy` entry is the digest of the policy document exactly as it was
supplied, byte for byte. It is the portable binding of a run to its policy. A
digest of a policy after defaults or settings have been resolved is defined by
an implementation and belongs under `extensions`.

### Counts

`counts.rows` is the number of rows converted. `counts.columns` has one entry
per schema column, in schema order, with the column's `id`; it has at least one
entry, and no `id` is empty.

`representation` says how every non-null value of the column was written:

| Value           | Meaning                                                                                    |
| --------------- | ------------------------------------------------------------------------------------------ |
| `native`        | in a typed form of the destination for the column's type                                   |
| `text-fallback` | an exact-number column written as text because `text-if-needed` found a value not eligible |
| `text-forced`   | an exact-number column written as text because the policy is `text-always`                 |
| `text`          | a `string` column; or a column of another type written in its canonical spelling as text   |

An exact-number column is `native`, `text-fallback` or `text-forced` and never
`text`. A `string` column is always `text`.

| Counter         | On                   | Counts                                                                          |
| --------------- | -------------------- | ------------------------------------------------------------------------------- |
| `null`          | every column         | null cells                                                                      |
| `empty`         | every column         | cells holding the empty string; zero unless the column is `string`              |
| `numeric`       | exact-number columns | non-null cells written in native numeric form                                   |
| `text_fallback` | exact-number columns | non-null cells of a `text-fallback` column, including values that were eligible |
| `text_forced`   | exact-number columns | non-null cells of a `text-forced` column                                        |
| `quantized`     | exact-number columns | cells whose mathematical value was changed by quantization                      |
| `scale_changed` | exact-number columns | cells whose scale was changed by quantization, whether or not the value changed |

The last five appear together, on exact-number columns only. An entry with
some of the five and not all, or a `text-fallback` or `text-forced` entry with
none, lacks a required member (`result-member-missing`); an entry whose
`representation` is `text` and that carries any of them has a member that does
not apply (`result-member-unknown`). For such a
column, `numeric + text_fallback + text_forced` equals `rows - null`, and only
the counter that matches `representation` is above zero. `quantized` and
`scale_changed` overlap, do not partition the cells, and are zero on a column
the policy does not quantize. On a column the policy quantizes to a scale
other than its declared scale, `scale_changed` equals `rows - null`: every
non-null cell changed scale, whether or not any value changed, and a column
whose cells are all null reports zero. On a column quantized to its declared
scale, `scale_changed` is zero. A result that is not `ok` carries no counts: a
run that did not complete claims nothing about its columns.

#### Which counts mean the same under every adapter profile

Some counts follow from the source, the schema and the policy alone. Two
conforming producers given the same three inputs report the same value for
them, whatever the destination. The others depend on which values an adapter
profile holds eligible, or on which typed forms its destination has, and are
**profile-defined**: their names and invariants are fixed by this standard,
and their values are comparable only between results that carry the same
`profile.id` and `profile.sha256`.

| Count                                                                          | Defined by      | Why                                                                                   |
| ------------------------------------------------------------------------------ | --------------- | ------------------------------------------------------------------------------------- |
| `rows`                                                                         | this standard   | the number of lines of the source                                                     |
| `null`                                                                         | this standard   | null elements of the source                                                           |
| `empty`                                                                        | this standard   | empty strings of the source                                                           |
| `quantized`                                                                    | this standard   | exact arithmetic on the source values under the policy's `quantize` entries           |
| `scale_changed`                                                                | this standard   | the same                                                                              |
| `text_forced`                                                                  | this standard   | `rows - null` on an exact-number column under `text-always`, and zero otherwise       |
| `numeric`                                                                      | adapter profile | depends on which values are eligible and, under `text-if-needed`, on the whole column |
| `text_fallback`                                                                | adapter profile | the same                                                                              |
| `representation` `text-forced`                                                 | this standard   | follows from `text-always`                                                            |
| `representation` of a `string` column                                          | this standard   | always `text`                                                                         |
| `representation` `native` or `text-fallback` on an exact-number column         | adapter profile | depends on eligibility                                                                |
| `representation` `native` or `text` on a `boolean`, `date` or timestamp column | adapter profile | depends on the typed forms of the destination                                         |

A consumer MUST NOT conclude anything about a destination from a
profile-defined count without the definition of the profile the result names.

### Refusal

`refusal` has `category` and `rule`, and `row` and `column` when the refusal
is located. `rule` has the form of a rule identifier and `column`, when
present, is not empty. See [Refusals](#refusals).

### Extensions

`extensions` is the one place where a policy or a result carries members this
standard does not define. There is one such member per document, at the top
level, and nowhere else.

Shape:

- `extensions` is an object. Each key is a **namespace** matching
  `[a-z][a-z0-9]*(-[a-z0-9]+)*`, at most 64 characters. Each value is an
  object whose content the owner of the namespace defines.
- A namespace belongs to an implementation or to an adapter profile. The
  namespace `example` and every namespace beginning `example-` are reserved
  for documentation and fixtures; no implementation defines one.

Bounds. They are part of this standard, are the same for every reader, and are
checked without knowing any namespace:

| Bound                                                                                                                 | Limit            | When exceeded           |
| --------------------------------------------------------------------------------------------------------------------- | ---------------- | ----------------------- |
| nesting of the whole document                                                                                         | 6                | `<kind>-malformed`      |
| namespaces                                                                                                            | 8                | `<kind>-member-invalid` |
| values under `extensions`: every member value and every array element, at every depth, the namespace objects included | 256              | `<kind>-member-invalid` |
| length of any string under `extensions`, member names included                                                        | 256 code points  | `<kind>-member-invalid` |
| any number under `extensions`                                                                                         | 9007199254740991 | `<kind>-member-invalid` |

A namespace key that does not match the pattern is `<kind>-member-invalid`. A
namespace value that is not an object is `<kind>-not-an-object`. Numbers under
`extensions` follow the document grammar like any other: unsigned integers
only.

The nesting bound gives a namespace four levels: the namespace object at depth
3 and three levels beneath it. That is the depth of the deepest document this
standard defines, a result, whose `counts` holds an array of objects. A
namespace can therefore carry a record, a list of records inside a record, and
nothing deeper. The fixtures `ok-extensions-at-depth-bound.json` and
`extensions-past-depth-bound.json` show a realistic member at the bound and
the first shape past it.

Rules for a result:

- A consumer applies the bounds, then MUST ignore every namespace it does not
  know. Unknown content is never a reason to refuse a result that is within
  the bounds.
- A consumer MUST be able to interpret every other member without reading any
  namespace. Deleting the `extensions` member from a result that is within
  the bounds changes no status, binding, count, refusal or matching verdict
  that this standard defines. A result whose `extensions` member breaks a
  bound is refused for that, and deleting the member removes the fault.
- An extension MUST NOT restate, qualify or contradict a member this standard
  defines. A consumer that finds such content ignores it.

Owned by an implementation, and therefore carried under `extensions` when they
are carried at all: the name and version of a tool and of a library; the name
and version of a backend; the limits that were in effect for the run; and a
digest of the policy as the implementation resolved it. Portable, and
therefore members of the result itself: the adapter profile's identifier and
definition digest, the digests and lengths under `inputs` and `output`, and
the counts.

Rules for a policy are under [Policy extensions](#policy-extensions).

## Refusals

### Categories

| Category                     | Meaning                                                                           |
| ---------------------------- | --------------------------------------------------------------------------------- |
| `invalid-input`              | malformed bytes, serialization, document or spelling of a value                   |
| `schema-mismatch`            | a well-formed value, row or policy that does not fit the schema                   |
| `unsupported-type`           | a type the reader or the destination does not carry                               |
| `unsupported-representation` | a value or a name the destination cannot represent under the policy               |
| `resource-limit`             | a declared limit, a capacity of the destination or a system resource was exceeded |
| `io-failure`                 | reading an input or writing an output failed                                      |
| `cancelled`                  | the run was asked to stop                                                         |
| `internal`                   | an unexpected failure of the producer                                             |

The list is closed and the spellings are exact.

### Rule identifiers

A rule identifier is 1 to 64 characters: lowercase ASCII letters and digits in
groups joined by single hyphens, starting with a letter. It names the one rule
that refused or failed, and it is stable: once published, an identifier keeps
its meaning.

- This standard registers the identifiers of the rules it defines. They are
  listed in `semantic-validation.md` and in `contract.json`. When a registered
  rule is the one that refuses, a producer MUST report its registered
  identifier and category.
- An implementation defines identifiers for its own rules: those of an adapter
  profile, of a destination and of the file system. It MUST NOT give a
  registered identifier another meaning.
- A consumer that does not know an identifier acts on the category.

### Location

`row` is the zero-based index of the row. `column` is the column identifier.
Each is present when the rule that refused is about one row or one column, and
absent otherwise. A refusal never carries the text of a value, of a line or of
a document.

### Which refusal is reported

A result reports one refusal: the first found in this order.

1. the schema document;
2. the policy document, including a namespace the reader does not implement;
3. the policy against the schema, and the schema and the policy against the
   adapter profile;
4. the rows, in order. Within one row: the encoding of the line; then the
   number of elements; then every element against its column, in schema
   order; then, column by column in schema order, quantization followed by
   the representation policy.

A failure of the system can occur at any point and is reported when it occurs.

## Matching a result to an output and to a request

A result does not name its output. How a consumer comes to hold a result and a
candidate output together, for example by a naming convention, is arranged
between the producer and the consumer.

A result and an output that agree with each other say nothing about which
request produced them. A result left by an earlier run, beside an output whose
bytes happen to be the same, agrees perfectly. Matching therefore has two
verdicts, and the stronger one needs facts that the caller holds independently
of the result.

### Expectations

An **expectation** is a fact about a request that the caller recorded when it
made the request:

| Expectation | Content                                                       | Compared with   |
| ----------- | ------------------------------------------------------------- | --------------- |
| `run_id`    | the run identifier the caller gave the producer               | `run_id`        |
| `source`    | length in bytes and SHA-256 of the source the caller supplied | `inputs.source` |
| `schema`    | length in bytes and SHA-256 of the schema the caller supplied | `inputs.schema` |
| `policy`    | length in bytes and SHA-256 of the policy the caller supplied | `inputs.policy` |

- An expectation MUST come from the caller's own record of the request. It
  MUST NOT be taken from the result being matched, or from anything the
  producer stored with it.
- A run identifier that the producer generated cannot be an expectation. A
  caller that needs the stronger verdict supplies its own run identifier, and
  does not reuse one for requests it needs to tell apart.
- An expectation is well formed when its `run_id` satisfies
  [Run identity](#run-identity), each length is an integer from 0 to
  9007199254740991, and each digest is 64 lowercase hexadecimal digits. A
  digest in another letter case is not well formed; it is never folded.

### Verdicts

| Verdict                   | Establishes                                                                                                                                       |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| `requested-run-completed` | the result is valid, reports `ok`, is bound to the caller's run identifier, source, schema and policy, and describes exactly the candidate output |
| `output-agreement`        | the result is valid, reports `ok`, and describes exactly the candidate output; nothing about which request it belongs to                          |
| `not-established`         | neither; a rule identifier says which step decided                                                                                                |

Which of the first two can be reached depends only on the expectations
supplied:

| Expectations supplied | Verdict when every step passes | Otherwise                                    |
| --------------------- | ------------------------------ | -------------------------------------------- |
| all four              | `requested-run-completed`      | `not-established`                            |
| none                  | `output-agreement`             | `not-established`                            |
| one, two or three     | never reached                  | `not-established`, `expectations-incomplete` |

A partial set of expectations is refused before the result is looked at. It is
not treated as none, because a caller that supplied some expectations asked
about a request, and `output-agreement` would be read as an answer to that
question. It is not checked as far as it goes, because a subset that matches
proves less than it appears to.

`output-agreement` MUST NOT be taken as evidence that a particular request
completed. A consumer that acts on behalf of a request requires
`requested-run-completed`.

### Procedure

The steps are applied in this order. The first that fails decides, the verdict
is `not-established`, and later steps are not applied.

| Step | Requirement                                                                                                                                     | Fails with                       |
| ---- | ----------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------- |
| 1    | all four expectations are supplied, or none is                                                                                                  | `expectations-incomplete`        |
| 2    | every supplied expectation is well formed                                                                                                       | `expectation-invalid`            |
| 3    | the consumer holds a result                                                                                                                     | `result-missing`                 |
| 4    | the result is accepted under every rule of this standard that applies to a result on its own, read in the [order of reading](#order-of-reading) | the rule that refuses it         |
| 5    | `status` is `ok`                                                                                                                                | `result-not-ok`                  |
| 6    | `run_id` equals the expected run identifier, code point for code point                                                                          | `binding-run-id-mismatch`        |
| 7    | `inputs.source.bytes` equals the expected length                                                                                                | `binding-source-length-mismatch` |
| 8    | `inputs.source.sha256` equals the expected digest                                                                                               | `binding-source-digest-mismatch` |
| 9    | `inputs.schema.bytes` equals the expected length                                                                                                | `binding-schema-length-mismatch` |
| 10   | `inputs.schema.sha256` equals the expected digest                                                                                               | `binding-schema-digest-mismatch` |
| 11   | `inputs.policy.bytes` equals the expected length                                                                                                | `binding-policy-length-mismatch` |
| 12   | `inputs.policy.sha256` equals the expected digest                                                                                               | `binding-policy-digest-mismatch` |
| 13   | the consumer holds a candidate output                                                                                                           | `output-missing`                 |
| 14   | the length of the candidate in bytes equals `output.bytes`                                                                                      | `output-length-mismatch`         |
| 15   | the SHA-256 of the candidate equals `output.sha256`                                                                                             | `output-digest-mismatch`         |

Steps 6 to 12 are applied only when all four expectations were supplied. When
every applicable step passes, the verdict is `requested-run-completed` if
steps 6 to 12 were applied and `output-agreement` if they were not.

- Steps 1 to 3 read nothing from a result. Step 4 reads `status` and settles
  which members are present before step 5; steps 6 to 15 read `inputs` and
  `output` only after step 5 has established that the status is `ok`. No step
  reads a member that exists only on success from a result that has not
  passed steps 4 and 5.
- An output with no result is `not-established`. The absence of a result says
  that the run did not complete; it does not say why.
- A result that is not `ok` is `not-established` with `result-not-ok`,
  whatever expectations it would satisfy. This standard defines no verdict
  that binds a refusal to a request; a consumer MAY compare `run_id` and the
  entries of `inputs` that are present for its own diagnosis.
- A producer MUST NOT make a result with status `ok` available before the
  bytes of the output it describes are final.
- The result, not the exit status of a process, is the record. A consumer that
  did not supervise the run relies on the result alone.
- Neither verdict establishes who wrote the result or the output. A party able
  to write both, and knowing the inputs, can produce a pair that matches.
- `profile` and `extensions` take no part in matching. A consumer that requires
  a particular adapter profile compares `profile` itself.

`matching-vectors.json` gives results, candidate outputs and expectations with
the expected verdict and the step that decides each.

## What the JSON Schemas do not express

Validating a document against its JSON Schema is necessary and is not
conformance. The schemas set `additionalProperties: false` on every object
this standard defines; the values under `extensions` are the only objects left
open, within their bounds.

The schemas **do** express, structurally:

- which members are required and which values are in each closed list;
- that `precision` and `scale` appear on `decimal` columns only and `fraction`
  on timestamp columns only;
- that `output` and `counts` appear only with status `ok` and `refusal` only
  with any other status, and that an `ok` result has all three inputs;
- the bounds on `extensions`, except the total number of values: the number of
  namespaces, the nesting, the length of strings and the size of numbers;
- which categories go with which status;
- that the five exact-number counters appear together, are present on
  `text-fallback` and `text-forced` columns and absent on `text` columns.

A reader still implements those rules itself, because a JSON Schema validator
reports that a document is invalid and not which rule, category and location
this standard requires.

The schemas **cannot** express:

- anything about bytes: UTF-8, the byte order mark, a repeated key, the
  spelling of a number (`12.0` and `1e1` are integers to JSON Schema), the
  nesting bound;
- the order of judgment, including that `format` is checked first and that
  the status of a result is settled before its other members are read;
- that an unknown `type` is `unsupported-type` while every other bad member is
  `invalid-input`;
- that `scale` does not exceed `precision`, in a column or in a quantize
  entry;
- that column identifiers are unique, and that quantize entries name distinct
  columns;
- anything that relates two documents: a quantize entry to its column, the
  columns of `counts` to the columns of the schema, which columns carry the
  exact-number counters, where `null` and `empty` may be above zero;
- arithmetic: that the counters sum to the non-null cells;
- that a policy namespace is one the reader implements;
- the total number of values under `extensions`;
- that a digest is the digest of anything, and anything about matching: the
  expectations are not in any document;
- the rows encoding beyond the shape of one parsed row: lines, tokens, the
  number of elements, and the typing of each element by its column;
- the value grammar. No `pattern` is given for values, because the accepted
  spelling depends on the column's type and parameters.

Each of these is a numbered rule in `semantic-validation.md` with at least one
negative fixture or vector.

## Outside this standard

The following belong to an implementation and its adapter profiles, and are
not defined here:

- the layout of any destination format, and how a destination records types,
  scale, nulls and empty strings;
- which values are eligible for a native numeric form;
- command-line interfaces and process exit codes;
- backend libraries and their versions;
- the definition of an adapter profile, and its limits;
- how files are created, named, published and made durable, and what happens
  when two runs share a destination;
- telemetry and progress reporting.

## Compatibility

- The family is below 1.0. A consumer pins an exact contract revision.
- A published revision is never changed. A correction is a new revision.
- A change to the shape of a document that an existing reader would refuse, or
  would read differently, is a new wire-format revision of that document:
  `tabular-exchange.result/1` follows `tabular-exchange.result/0`. This holds
  below 1.0 as well; the revision does not stay at 0 until then. A reader
  refuses a wire-format revision it does not support.
- A format identifier that an implementation used before it adopted this
  standard is a different format. A reader of this standard does not accept
  it as another spelling of one of the three identifiers above; an
  implementation that reads both states which it supports and how a document
  moves from one to the other.
- A change to the meaning of a document whose shape is unchanged is a new
  contract revision. A consumer refuses a result whose `contract` it does not
  implement, with `result-contract-unsupported`.
- Adding a type, a policy value, a rounding mode, a status, a category or a
  representation changes a closed list and is a new wire-format revision.
- Registering a new rule identifier is a new contract revision.
- Changing a bound, including a bound on `extensions`, is a new contract
  revision.

## Related

- `schemas/data/tabular-exchange/v0.1.0/README.md`
- `schemas/data/tabular-exchange/v0.1.0/semantic-validation.md`
- [ADR-0012: Use Absolute $id URLs for Cross-Schema $ref References](../../architecture/decisions/ADR-0012-schema-ref-ids.md)
- [Canonical URI Resolution Standard](../publishing/canonical-uri-resolution.md)
