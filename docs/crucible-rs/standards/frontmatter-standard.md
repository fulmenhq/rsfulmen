---
title: "FulmenHQ Document Frontmatter Standard"
description: "Required frontmatter format for documentation files across FulmenHQ repositories"
author: "@3leapsdave"
date: "2025-10-02"
last_updated: "2025-10-07"
status: "approved"
tags: ["standards", "documentation", "metadata", "ai-attribution"]
---

# Document Frontmatter Standard

## Overview

This standard defines the required frontmatter format for all documentation files across FulmenHQ repositories. Frontmatter provides structured metadata that enables automated processing, search, and organization of documentation.

## Required Frontmatter Fields

All documentation files MUST include frontmatter using YAML format with the following required fields:

```yaml
---
title: "Document Title"
description: "Brief description of the document's purpose and scope"
author: "@3leapsdave"
date: "2025-10-02"
last_updated: "2025-10-02"
status: "draft"
tags: ["tag1", "tag2", "tag3"]
---
```

## Field Definitions

### Required Fields

- **title** (string): The document title, used for navigation and indexing
- **description** (string): Brief description (1-2 sentences) explaining the document's purpose
- **author** (string): Primary author name or AI agent identity (e.g., "Schema Cartographer" or "@3leapsdave")
- **date** (string): Creation date in ISO 8601 format (YYYY-MM-DD)
- **last_updated** (string): Last modification date in ISO 8601 format (YYYY-MM-DD)
- **status** (enum): Document status - one of: `draft`, `review`, `approved`, `deprecated`
- **tags** (array): Array of relevant tags for categorization and search

### Human Attribution Fields

When AI agents author or significantly contribute to documentation, include human oversight attribution:

- **author_of_record** (string): Human maintainer responsible for document accuracy and maintenance
  - Format: `Name <email>` or `Name (https://github.com/username)`
  - Examples:
    - `Dave Thompson <dave.thompson@3leaps.net>`
    - `Dave Thompson (https://github.com/3leapsdave)`
  - Required when `author` is an AI agent identity
  - Provides accountability and contact point for questions

### Optional Fields

- **reviewers** (array): List of reviewers for collaborative documents
- **related_docs** (array): Links to related documentation
- **version** (string): Document version for versioned content
- **category** (string): Document category (e.g., "standards", "schemas", "architecture")
- **revision** (integer): Revision number for tracking changes (useful for schemas)
- **supervised_by** (string): Human supervisor for AI-generated content (when `author` is AI agent)
  - Format: GitHub handle (e.g., "@3leapsdave")
  - Complements `author_of_record` for accountability chain

## Examples

### Standard Document

```yaml
---
title: "Repository Versioning Standard"
description: "Standards for version management across FulmenHQ repositories"
author: "@3leapsdave"
date: "2025-10-02"
last_updated: "2025-10-02"
status: "approved"
tags: ["standards", "versioning", "semver", "calver"]
---
```

### Schema Document

```yaml
---
title: "Terminal Schema v1.0.0"
description: "JSON Schema for terminal configuration overrides"
author: "@3leapsdave"
date: "2025-10-02"
last_updated: "2025-10-02"
status: "approved"
version: "v1.0.0"
revision: 1
tags: ["schemas", "terminal", "validation"]
---
```

### Collaborative Document

```yaml
---
title: "API Design Guidelines"
description: "Best practices for designing REST and GraphQL APIs in FulmenHQ"
author: "@3leapsdave"
date: "2025-10-02"
last_updated: "2025-10-02"
status: "review"
reviewers: ["@forge-warden", "@code-scout"]
tags: ["standards", "api", "rest", "graphql"]
---
```

### AI-Authored Document with Human Oversight

```yaml
---
title: "Fulmen Library Bootstrap Guide"
description: "Bootstrap a new Fulmen ecosystem library with minimal configuration"
author: "Schema Cartographer"
author_of_record: "Dave Thompson (https://github.com/3leapsdave)"
supervised_by: "@3leapsdave"
date: "2025-10-07"
last_updated: "2025-10-07"
status: "draft"
tags: ["bootstrap", "fuldx", "setup", "new-repository"]
---
```

This example shows proper attribution when an AI agent (Schema Cartographer) creates documentation under human supervision, with a clear accountability chain.

## Implementation Guidelines

### File Naming

- Use kebab-case for filenames: `document-title.md`
- Include version in filename if needed: `schema-reference-v1.0.0.md`
- Store in appropriate directory: `docs/standards/`, `docs/architecture/`, etc.

### Validation

- Frontmatter must be valid YAML
- All required fields must be present
- Dates must follow ISO 8601 format (YYYY-MM-DD)
- Status must be one of: `draft`, `review`, `approved`, `deprecated`
- Tags should be lowercase with hyphens

### Document Lifecycle

**Status progression:**

1. `draft` - Initial creation, work in progress
2. `review` - Ready for review, reviewers assigned
3. `approved` - Reviewed and approved for use
4. `deprecated` - No longer current, replaced by newer version

### Revision Tracking

For schemas and frequently updated documents:

- Increment `revision` for content changes
- Update `last_updated` date
- Keep `version` stable unless breaking changes occur
- Document changes in adjacent CHANGELOG or revision history

### Tooling Integration

Frontmatter enables:

- Automated documentation generation
- Search and indexing systems
- Status tracking dashboards
- Cross-repository documentation discovery
- Revision history tracking

## Repository-Specific Extensions

Individual repositories MAY add optional fields specific to their needs:

```yaml
---
# Required fields...
schema_version: "v1.0.0"
schema_type: "json-schema-draft-2020-12"
related_schemas: ["terminal-catalog.yaml"]
---
```

Extended fields should be documented in the repository's `CONTRIBUTING.md`.

## Related Standards

- [Repository Versioning Standard](repository-versioning.md) - Version management approaches
- [Agentic Attribution Standard](agentic-attribution.md) - AI agent contribution standards
- [Repository Version Adoption SOP](../sop/repository-version-adoption.md) - Mandatory version strategy

## Relationship to Agentic Attribution Standard

This frontmatter standard complements the [Agentic Attribution Standard](agentic-attribution.md):

- **Frontmatter**: Document-level metadata for discovery, organization, and accountability
- **Agentic Attribution**: Commit-level attribution for code changes and contributions

**Key differences:**

| Aspect       | Frontmatter                   | Agentic Attribution                     |
| ------------ | ----------------------------- | --------------------------------------- |
| **Scope**    | Document metadata             | Commit messages                         |
| **Purpose**  | Organization & accountability | Version control attribution             |
| **Location** | Top of document               | Git commit message                      |
| **Fields**   | `author`, `author_of_record`  | `Co-Authored-By`, `Committer-of-Record` |

**Best practice**: Use both standards together for complete attribution chain from document creation through version control.

---

**Status**: Approved  
**Last Updated**: 2025-10-07  
**Author**: @3leapsdave
