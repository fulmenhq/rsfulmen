---
title: "Docscribe Module"
description: "Standalone lightweight module for processing markdown and YAML documentation from any source"
status: "stable"
module_id: "docscribe"
tier: "core"
requirement: "mandatory"
coverage_targets:
  go: 90
  python: 95
  typescript: 90
last_updated: "2025-10-20"
author: "@fulmen-ea-steward"
tags:
  [
    "module",
    "docscribe",
    "documentation",
    "frontmatter",
    "markdown",
    "yaml",
    "multi-document",
  ]
---

# Docscribe Module

## Overview

The docscribe module provides standalone, lightweight processing capabilities for markdown and YAML documentation from any source. It handles common documentation tasks like frontmatter extraction, header parsing, format detection, and multi-document splitting without coupling to specific storage or access patterns.

**Key Design Principle**: This module is intentionally **source-agnostic**. It processes content from Crucible, Cosmography, local files, or any other documentation source with equal facility.

## Purpose

Helper libraries integrate with multiple SSOT documentation sources, each with their own embedded assets and access patterns. Rather than duplicating documentation processing logic in each source-specific shim, this module provides reusable utilities that work with content from any origin.

## Core Capabilities

### Phase 1: Mandatory APIs

All implementations MUST provide these core functions:

#### 1. Frontmatter Processing

```python
# Extract frontmatter and clean content
content, metadata = parse_frontmatter(markdown_text)

# Extract only metadata
metadata = extract_metadata(markdown_text)

# Strip frontmatter, return clean content
clean_text = strip_frontmatter(markdown_text)
```

#### 2. Header Extraction

```python
# Extract all headers with hierarchy
headers = extract_headers(markdown_text)
# Returns: [{"level": 1, "text": "Title", "anchor": "title", "line_number": 1}, ...]
```

#### 3. Format Detection

```python
# Detect content format
format = detect_format(content)
# Returns: "markdown", "yaml", "json", "toml", "text", "multi-yaml", "multi-markdown"
```

#### 4. Document Inspection

```python
# Quick inspection without full parsing
info = inspect_document(content)
# Returns: {
#   "has_frontmatter": true,
#   "header_count": 12,
#   "format": "markdown",
#   "line_count": 450,
#   "estimated_sections": 5
# }
```

#### 5. Multi-Document Handling

```python
# Split multi-document files
docs = split_documents(content)  # YAML streams, concatenated markdown
# Returns: ["doc1", "doc2", "doc3"]
```

### Phase 2: Enhanced APIs

#### 1. Outline Generation

```python
# Generate hierarchical outline
outline = generate_outline(content, max_depth=3)
# Returns nested structure for TOC rendering
```

#### 2. Batch Processing

```python
# Process multiple documents
results = parse_collection({
    "path1": content1,
    "path2": content2
})
```

#### 3. Header Search

```python
# Find headers matching pattern
matching = search_headers(content, "installation")
```

#### 4. Link Extraction

```python
# Extract markdown links
links = extract_links(content)
# Returns: [{"text": "link text", "url": "https://...", "line_number": 10}, ...]
```

## Implementation Requirements

### Input Flexibility

- Accept strings, bytes, or streams as input
- Handle various encodings (UTF-8, UTF-16)
- Support both UNIX and Windows line endings

### Frontmatter Parsing

- Detect YAML frontmatter delimited by `---`
- Support empty frontmatter blocks
- Handle malformed YAML gracefully (return ParseError)
- Common keys: title, description, author, date, status, tags

### Multi-Document Support

**Critical**: The `---` separator has dual purposes:

1. Frontmatter delimiter in markdown
2. Document separator in YAML streams

The module MUST:

- Correctly distinguish between frontmatter and document separators
- Support YAML streams (Kubernetes manifests, etc.)
- Handle concatenated markdown documents
- Process each document independently after splitting

### Header Extraction

- Support all markdown header levels (1-6)
- Generate anchor slugs for navigation
- Include line numbers for source mapping
- Handle both ATX (`#`) and Setext (underline) styles

### Format Detection

- Use heuristics, not file extensions
- Check for frontmatter markers (`---`)
- Detect JSON (`{` or `[` start)
- Identify YAML structure
- Recognize multi-document formats
- Default to "text" for unknown

### Error Handling

- `ParseError`: Invalid YAML or malformed content
- `FormatError`: Content doesn't match expected format
- Provide helpful error messages with line numbers

## Usage Examples

### Basic Document Processing

```python
from pyfulmen import crucible, documentation

# Get doc from any source
doc = crucible.get_documentation('standards/observability/logging')

# Quick inspection
info = documentation.inspect_document(doc)
print(f"Has frontmatter: {info.has_frontmatter}")
print(f"Sections: {info.estimated_sections}")

# Full processing
content, metadata = documentation.parse_frontmatter(doc)
if metadata:
    print(f"Title: {metadata.get('title')}")
    print(f"Status: {metadata.get('status')}")
```

### Multi-Document Handling

```python
# Kubernetes manifests (YAML stream)
with open('k8s-deployments.yaml', 'r') as f:
    yaml_stream = f.read()

# Split and process each document
docs = documentation.split_documents(yaml_stream)
for doc in docs:
    if documentation.detect_format(doc) == "yaml":
        data = yaml.safe_load(doc)
        print(f"Found {data['kind']}: {data['metadata']['name']}")

# Concatenated markdown documents
multi_md = load_bundled_docs()  # From any source
md_docs = documentation.split_documents(multi_md)
for doc in md_docs:
    content, meta = documentation.parse_frontmatter(doc)
    process_document(content, meta)
```

### Document Discovery

```python
def index_documentation(sources):
    """Build searchable index from multiple sources."""
    index = []

    for source, paths in sources.items():
        for path in paths:
            content = source.get_documentation(path)

            # Inspect without full parsing
            info = documentation.inspect_document(content)

            # Extract title
            if info.has_frontmatter:
                _, meta = documentation.parse_frontmatter(content)
                title = meta.get('title', path)
            else:
                headers = documentation.extract_headers(content)
                title = headers[0].text if headers else path

            index.append({
                'source': source.__name__,
                'path': path,
                'title': title,
                'format': info.format,
                'sections': info.estimated_sections
            })

    return index
```

### Navigation Generation

```python
def build_toc(content):
    """Generate table of contents."""
    outline = documentation.generate_outline(content, max_depth=2)
    return format_toc(outline)  # Custom formatting
```

## Language Idioms

### Go

```go
package documentation

// Document inspection result
type DocumentInfo struct {
    HasFrontmatter     bool   `json:"has_frontmatter"`
    HeaderCount        int    `json:"header_count"`
    Format            string `json:"format"`
    LineCount         int    `json:"line_count"`
    EstimatedSections int    `json:"estimated_sections"`
}

// Split multi-document content
func SplitDocuments(content []byte) ([]string, error)

// Inspect without parsing
func InspectDocument(content []byte) (*DocumentInfo, error)
```

### Python

```python
from typing import Dict, List, Optional, Tuple, Any
from dataclasses import dataclass

@dataclass
class DocumentInfo:
    has_frontmatter: bool
    header_count: int
    format: str
    line_count: int
    estimated_sections: int

def split_documents(content: str | bytes) -> List[str]:
    """Split multi-document content."""

def inspect_document(content: str | bytes) -> DocumentInfo:
    """Quick inspection without full parsing."""
```

### TypeScript

```typescript
interface DocumentInfo {
  hasFrontmatter: boolean;
  headerCount: number;
  format: string;
  lineCount: number;
  estimatedSections: number;
}

function splitDocuments(content: string): string[];
function inspectDocument(content: string): DocumentInfo;
```

## Testing Requirements

### Core Test Cases

1. **Multi-Document Scenarios**
   - YAML streams with multiple documents
   - Concatenated markdown with `---` separators
   - Mixed: markdown with frontmatter + YAML stream
   - Edge: single document with frontmatter
   - Edge: `---` in code blocks (should not split)

2. **Document Inspection**
   - Various document types and structures
   - Performance: <1ms for typical documents
   - Accuracy of section estimation

3. **Frontmatter Variations**
   - Valid YAML frontmatter
   - Empty frontmatter block
   - No frontmatter
   - Malformed YAML
   - Frontmatter with `---` in values

4. **Format Detection**
   - Single vs multi-document detection
   - Markdown with/without frontmatter
   - Pure YAML files vs YAML streams
   - JSON documents
   - Plain text

### Performance Benchmarks

- Inspect 100KB document: <1ms
- Parse frontmatter: <5ms
- Split 10-document stream: <10ms
- Extract headers from 1MB doc: <50ms
- Generate outline: <20ms

## Design Rationale

### Why Standalone?

1. **Multiple Sources**: Helper libraries work with Crucible, Cosmography, and other SSOT documentation
2. **Reusability**: Documentation processing is universal, not source-specific
3. **Separation of Concerns**: Processing logic is orthogonal to storage/access
4. **Composability**: Can be used by any module needing doc processing
5. **Future-Proof**: Easily extended for new formats without affecting consumers

### Why Multi-Document Support?

The `---` separator serves dual purposes that must be handled correctly:

- Frontmatter delimiter in markdown
- Document separator in YAML streams

This is critical for:

- Kubernetes/Docker manifests
- CI/CD pipeline outputs
- Bundled documentation sets
- YAML configuration streams

### Why Not Full Markdown Parsing?

- **Size**: Full parsers add 100KB+ bloat
- **Complexity**: AST generation overkill for most needs
- **Performance**: Lightweight extraction is 10x faster
- **User Choice**: Users can pipe to full parsers if needed

## Dependencies

- **Required**: Lightweight YAML parser only
- **Optional**: Regex for advanced patterns
- **Avoid**: Heavy markdown parsers, HTML processors, DOM libraries

## Related Standards

- [Frontmatter Standard](../../frontmatter-standard.md) - YAML header format
- [Crucible Shim Module](./crucible-shim.md) - Uses this for doc access
- [Error Handling Module](./error-handling-propagation.md) - Error propagation

---

**Module Status**: Stable
**Implementation Priority**: High (blocking multi-source access)
**Target Coverage**: 90% (Go), 95% (Python), 90% (TypeScript)
