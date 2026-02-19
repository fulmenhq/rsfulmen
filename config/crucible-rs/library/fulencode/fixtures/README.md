# Fulencode Fixtures

These fixtures provide cross-language, deterministic inputs for Fulencode parity tests.

Design goals:

- ASCII-only source files; represent bytes as hex strings.
- Small, stable fixtures; large/pathological payloads should be generated in tests.
- Fixtures are SSOT and should not be duplicated in `lang/*` directories.

Directory layout:

- `valid-encodings/` - Known-good inputs/outputs for encode/decode
- `invalid-encodings/` - Inputs that must trigger canonical errors
- `normalization/` - Inputs/outputs for normalization profiles
- `bom/` - BOM detection/removal cases
- `detection/` - Minimal detection cases (BOM, UTF-8 validity, ASCII ambiguity)
- `telemetry/` - Expected metric emissions for parity tests
