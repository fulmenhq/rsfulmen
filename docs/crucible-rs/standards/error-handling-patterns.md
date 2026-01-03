---
title: "Error Handling Patterns"
description: "Common error handling patterns for cross-language consistency in Fulmen libraries"
author: "@gofulmen-team"
date: "2025-10-02"
last_updated: "2025-10-02"
status: "approved"
tags: ["standards", "errors", "cross-language"]
---

# Error Handling Patterns

This document defines common error handling patterns used across Fulmen language implementations to ensure consistency and predictable behavior.

## Error Types

### PathTraversalError

- **Code**: `PATH_TRAVERSAL`
- **Description**: Attempted access to paths outside allowed boundaries
- **Context**: Include the attempted path and boundary root
- **Recovery**: Validate and sanitize path inputs

### InvalidPathError

- **Code**: `INVALID_PATH`
- **Description**: Path is malformed or unsafe
- **Context**: Include the invalid path
- **Recovery**: Check path format and constraints

### SchemaValidationError

- **Code**: `SCHEMA_VALIDATION`
- **Description**: Data does not conform to expected schema
- **Context**: Include validation errors and schema reference
- **Recovery**: Validate data before processing

### FileAccessError

- **Code**: `FILE_ACCESS`
- **Description**: Unable to read/write file due to permissions or existence
- **Context**: Include file path and operation attempted
- **Recovery**: Check file permissions and existence

## Error Response Structure

All errors should conform to the `ErrorResponse` schema (see `schemas/pathfinder/v1.0.0/error-response.schema.json`):

```json
{
  "code": "PATH_TRAVERSAL",
  "message": "Attempted access outside allowed boundary",
  "details": {
    "attemptedPath": "/etc/passwd",
    "boundaryRoot": "/home/user/project"
  },
  "path": "/etc/passwd",
  "timestamp": "2025-10-02T12:00:00Z"
}
```

**Required fields:**

- `code`: Machine-readable error identifier
- `message`: Human-readable description

**Optional fields:**

- `details`: Additional context (object)
- `path`: Related path if applicable (string)
- `timestamp`: When error occurred (ISO 8601 string)

## Error Handling Strategies

### Validation Errors

- Validate inputs at API boundaries
- Return detailed validation errors
- Allow partial success where appropriate
- Use schema validation for structured data

### File System Errors

- Check permissions before operations
- Provide clear error messages
- Allow callers to handle specific error types
- Include path information for debugging

### Schema Validation Errors

- Validate against JSON schemas at boundaries
- Return all validation errors, not just first
- Include schema reference in error details
- Provide actionable error messages

## Language-Specific Implementation

### Go

```go
type PathTraversalError struct {
    AttemptedPath string
    BoundaryRoot  string
}

func (e *PathTraversalError) Error() string {
    return fmt.Sprintf("path traversal: %s outside boundary %s",
        e.AttemptedPath, e.BoundaryRoot)
}

func (e *PathTraversalError) Code() string {
    return "PATH_TRAVERSAL"
}
```

### TypeScript

```typescript
class PathTraversalError extends Error {
  code = "PATH_TRAVERSAL";
  constructor(
    public attemptedPath: string,
    public boundaryRoot: string,
  ) {
    super(`path traversal: ${attemptedPath} outside boundary ${boundaryRoot}`);
  }

  toResponse(): ErrorResponse {
    return {
      code: this.code,
      message: this.message,
      details: {
        attemptedPath: this.attemptedPath,
        boundaryRoot: this.boundaryRoot,
      },
      timestamp: new Date().toISOString(),
    };
  }
}
```

### Python (Future)

```python
class PathTraversalError(Exception):
    code = "PATH_TRAVERSAL"

    def __init__(self, attempted_path: str, boundary_root: str):
        self.attempted_path = attempted_path
        self.boundary_root = boundary_root
        super().__init__(
            f"path traversal: {attempted_path} outside boundary {boundary_root}"
        )

    def to_response(self) -> dict:
        return {
            "code": self.code,
            "message": str(self),
            "details": {
                "attemptedPath": self.attempted_path,
                "boundaryRoot": self.boundary_root
            },
            "timestamp": datetime.utcnow().isoformat() + "Z"
        }
```

## Testing Error Handling

### Unit Tests

- Test all error conditions
- Verify error codes and messages
- Validate error response structure against schema

### Integration Tests

- Test error propagation across layers
- Verify error logging and monitoring
- Test error recovery paths

## Related Standards

- [ErrorResponse Schema](../../schemas/pathfinder/v1.0.0/error-response.schema.json)
- [Cross-Language API Standards](./configuration-standardization.md)

---

**Status**: Approved  
**Last Updated**: 2025-10-02  
**Original Author**: @gofulmen-team  
**Integrated By**: @3leapsdave
