# AILink Schemas v0

Schemas for AI-powered backend integration.

**Status**: Unstable (v0) - breaking changes may occur without notice.

## Schemas

| Schema                        | Purpose                                     |
| ----------------------------- | ------------------------------------------- |
| `prompt.schema.json`          | AILink prompt configuration                 |
| `search-response.schema.json` | Base response structure for search/analysis |

## Usage

### Prompt Configuration

Prompts use YAML frontmatter validated against `prompt.schema.json`:

```yaml
slug: my-prompt
name: My Prompt
description: What this prompt does
version: 1.0.0
input:
  required_variables:
    - query
tools:
  - type: web_search
```

The prompt body (after `---`) contains the system template in markdown.

### Response Validation

Responses are validated against `search-response.schema.json` or a domain-specific schema that extends it.

## Schema URLs

```
https://schemas.3leaps.dev/ailink/v0/prompt.schema.json
https://schemas.3leaps.dev/ailink/v0/search-response.schema.json
```

## Related

- [namelens/namelens](https://github.com/namelens/namelens) - Reference implementation
- [FulmenHQ Crucible](https://github.com/fulmenhq/crucible) - Enterprise extensions
