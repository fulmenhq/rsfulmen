# Fulencode Schemas (v1.0.0)

These schemas define the SSOT contracts for the Fulencode module.

Fulencode is an opt-in (Specialized-tier) facade that standardizes encoding/decoding, text encoding validation/detection, Unicode normalization, BOM handling, and security limits across Fulmen helper libraries.

## Files

- `fulencode-config.schema.json` - Module configuration (limits + defaults).

Method option schemas:

- `encode-options.schema.json`
- `decode-options.schema.json`
- `detect-options.schema.json`
- `normalize-options.schema.json`

Result payload schemas:

- `encoding-result.schema.json`
- `decoding-result.schema.json`
- `detection-result.schema.json`
- `normalization-result.schema.json`
- `bom-result.schema.json`

Error envelope:

- `fulencode-error.schema.json`

## Related SSOT (Taxonomies + Standard)

- Standard: `docs/standards/library/modules/fulencode.md`
- Encoding formats taxonomy: `schemas/taxonomy/library/fulencode/encoding-families/v1.0.0/families.yaml`
- Normalization profiles taxonomy: `schemas/taxonomy/library/fulencode/normalization-profiles/v1.0.0/profiles.yaml`
- Detection confidence taxonomy: `schemas/taxonomy/library/fulencode/detection-confidence/v1.0.0/levels.yaml`

These schemas validate data structures produced by helper libraries and support cross-language parity fixtures.
