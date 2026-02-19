---
title: "Fulencode text_safe Normalization Profile"
description: "Deterministic, security-focused normalization for log-safe and UI-safe text"
author: "Schema Cartographer"
date: "2026-01-26"
status: "draft"
tags:
  ["fulencode", "normalization", "unicode", "security", "bidi", "zero-width"]
---

# Fulencode `text_safe` Normalization Profile

`text_safe` is a Fulencode normalization profile intended for _display and logging safety_.
It is not an identifier profile and it must not be used to canonicalize security-critical identifiers.

## SSOT and Drift Warning

This document is explanatory.

The authoritative contract is:

- Taxonomy + constraints: `schemas/taxonomy/library/fulencode/normalization-profiles/v1.0.0/profiles.yaml`
- Parity fixtures: `config/library/fulencode/fixtures/normalization/text-safe.yaml`

Helper library implementations must follow the SSOT definitions and tests. If this document and SSOT ever disagree, treat SSOT as correct and update this document.

## Goal

Prevent common text spoofing and hidden-content attacks in enterprise systems:

- Bidi override/isolate injection (UI/log confusion)
- Zero-width / invisible character hiding
- Control character injection (log line breaks, terminal escapes, invisible separators)
- Excessive combining marks (rendering and perf hazards)

## Deterministic Algorithm

Given input `text`:

1. NFC normalize

- Apply Unicode normalization NFC.

2. Reject disallowed characters

The profile rejects (errors) if any forbidden character is present.

- ASCII control characters (C0): U+0000..U+001F and U+007F
- C1 controls: U+0080..U+009F
- Bidi controls (minimum required set):
  - U+202A..U+202E (embedding/override)
  - U+2066..U+2069 (isolate controls)
  - U+200E, U+200F (LRM/RLM)
  - U+061C (Arabic Letter Mark)
- Zero-width characters (minimum required set):
  - U+200B (ZWSP)
  - U+200C (ZWNJ)
  - U+200D (ZWJ)
  - U+FEFF (ZWNBSP / BOM code point)

3. Printable-only constraint

Implementations must treat characters in Unicode General Category `Cc` (control) as forbidden.
If a language/runtime does not expose category queries, follow the explicit ranges above.

4. Combining mark cap

Enforce a maximum number of consecutive combining marks per base character (default 10). Exceeding the cap fails with `EXCESSIVE_COMBINING_MARKS`.

## Options (If Exposed)

If a helper library exposes profile options, the defaults must match the SSOT behavior.

Recommended optional switches:

- `allow_joiners` (default: false): if true, permit U+200C/U+200D for legitimate script shaping and emoji sequences.
- `collapse_whitespace` (default: false): if true, collapse runs of whitespace to a single ASCII space and trim leading/trailing whitespace.

## Error Mapping

Minimum expected error codes:

- Zero-width rejection: `ZERO_WIDTH_CHARACTER`
- Bidi control rejection: `BIDI_CONTROL_CHARACTER`
- Combining mark cap: `EXCESSIVE_COMBINING_MARKS`
- Control character rejection: `INVALID_ENCODING` or a dedicated control-character error if defined by Fulencode error taxonomy

## Notes

- `text_safe` is intentionally stricter than “general text normalization” to avoid ambiguous rendering and audit log corruption.
- If you need permissive display (e.g., allow newlines or tabs), use a different profile or add an explicit application-layer sanitizer.
