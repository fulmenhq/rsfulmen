# Assessment Schemas

This directory contains shared schema definitions for assessment- and analysis-related
artifacts that span multiple Fulmen tools.

## Files

| Schema                                    | Purpose                                                       |
| ----------------------------------------- | ------------------------------------------------------------- |
| `v1.0.0/severity-definitions.schema.json` | Canonical severity name/level mapping for assessment tooling. |

### Severity Definitions

`severity-definitions.schema.json` provides `$defs` that map assessment severity names
(`info`, `low`, `medium`, `high`, `critical`) to sortable numeric levels (`0-4`).
Consumers should reference these definitions via `$ref`, for example:

```yaml
fail_on:
  $ref: ../../assessment/v1.0.0/severity-definitions.schema.json#/$defs/severityName
  default: high
```

When comparing severities, always convert names to `severityLevel` using the canonical
mapping (`severityMapping`) rather than relying on lexical ordering.
