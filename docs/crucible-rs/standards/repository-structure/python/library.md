---
title: "Python Library Structure"
description: "Repository structure for pure Python libraries including public API design, type hints, py.typed markers, and publishing workflows"
author: "Pipeline Architect"
author_of_record: "Dave Thompson (https://github.com/3leapsdave)"
supervised_by: "@3leapsdave"
date: "2025-10-08"
last_updated: "2025-10-08"
status: "approved"
tags:
  [
    "standards",
    "repository-structure",
    "python",
    "library",
    "packaging",
    "variant",
  ]
related_docs: ["README.md", "cli-click.md", "../../coding/python.md"]
---

# Python Library Structure

## Overview

This guide establishes the **repository structure and patterns** for pure Python libraries in the FulmenHQ ecosystem. It builds upon the [Python Repository Structure Standards](README.md) with library-specific patterns.

**Target Use Cases**:

- Reusable Python packages
- Shared utilities and frameworks
- SDK libraries
- Internal tooling libraries

**Example**: The `fulmenhq-crucible` package in `lang/python/` of this repository.

---

## Prerequisites

Read and apply **[Python Repository Structure Standards](README.md)** first. This document extends those requirements.

**Required Tooling** (from base standards):

- Python 3.12+
- uv (package management)
- ruff (linting/formatting)
- pytest (testing)

**Library-Specific Requirements**:

- Type hints on ALL public APIs
- `py.typed` marker file (PEP 561)
- Comprehensive docstrings (Google style)
- Test coverage ≥90% on public API

---

## Project Structure

```
library-name/
├── .python-version              # 3.12
├── pyproject.toml               # Project metadata
├── ruff.toml                    # Linting config
├── uv.lock                      # Locked dependencies
├── README.md
├── LICENSE
├── CHANGELOG.md                 # Version history
├── Makefile                     # Development commands
│
├── src/
│   └── package_name/
│       ├── __init__.py          # Public API exports
│       ├── py.typed             # Type marker (PEP 561)
│       ├── core/                # Core functionality
│       │   ├── __init__.py
│       │   ├── module1.py
│       │   └── module2.py
│       ├── utils/               # Utilities
│       │   ├── __init__.py
│       │   └── helpers.py
│       └── exceptions.py        # Custom exceptions
│
├── tests/
│   ├── __init__.py
│   ├── unit/
│   │   ├── test_core.py
│   │   └── test_utils.py
│   ├── integration/
│   │   └── test_api.py
│   └── fixtures/
│       └── sample_data.json
│
├── docs/                        # Documentation (optional)
│   ├── guide.md
│   └── api.md
│
└── schemas/                     # JSON/YAML schemas (optional)
    └── config.schema.json
```

---

## Key Components

### 1. Package Init (`__init__.py`)

**Purpose**: Define public API and version

```python
"""Package Name - Brief description.

This package provides [core functionality description].

Example:
    >>> from package_name import main_function
    >>> result = main_function(data)
"""

__version__ = "1.0.0"

from package_name.core.module1 import PublicClass
from package_name.core.module2 import public_function
from package_name.exceptions import PackageError

__all__ = [
    "__version__",
    "PublicClass",
    "public_function",
    "PackageError",
]
```

**Rules**:

- Export ONLY public API in `__all__`
- Use relative imports internally, absolute in `__init__.py`
- Document package purpose in module docstring

### 2. Type Marker (`py.typed`)

**File**: `src/package_name/py.typed`  
**Content**: Empty file

```bash
# Create type marker
touch src/package_name/py.typed
```

**Purpose**: Signals to type checkers that package includes type hints (PEP 561)

### 3. Core Module Example (`core/module1.py`)

```python
"""Core functionality for [specific feature].

This module provides...
"""

import logging
from pathlib import Path
from typing import Any

logger = logging.getLogger(__name__)

class PublicClass:
    """Brief description of class purpose.

    Longer description with usage context.

    Args:
        config: Configuration dictionary.
        cache_dir: Optional cache directory path.

    Example:
        >>> obj = PublicClass(config={"key": "value"})
        >>> result = obj.process(data)
    """

    def __init__(
        self,
        config: dict[str, Any],
        cache_dir: Path | None = None
    ) -> None:
        self.config = config
        self.cache_dir = cache_dir or Path.cwd() / ".cache"

    def process(self, data: str) -> dict[str, Any]:
        """Process input data.

        Args:
            data: Input data to process.

        Returns:
            Dictionary containing processed results.

        Raises:
            ProcessingError: If processing fails.
        """
        logger.debug("Processing data of length %d", len(data))
        # ... implementation
        return {"processed": True}

def public_function(value: int) -> int:
    """Brief description.

    Args:
        value: Input value.

    Returns:
        Processed value.
    """
    return value * 2
```

### 4. Custom Exceptions (`exceptions.py`)

```python
"""Package-specific exceptions."""

class PackageError(Exception):
    """Base exception for package errors."""

class ConfigurationError(PackageError):
    """Raised when configuration is invalid."""

class ProcessingError(PackageError):
    """Raised when processing fails."""

class ValidationError(PackageError):
    """Raised when validation fails."""
```

---

## Package Metadata

### `pyproject.toml` for Libraries

```toml
[project]
name = "package-name"
version = "1.0.0"
description = "Brief description of library purpose"
authors = [
    {name = "FulmenHQ", email = "dev@fulmenhq.com"}
]
requires-python = ">=3.12"
readme = "README.md"
license = {text = "MIT"}
keywords = ["keyword1", "keyword2", "library"]
classifiers = [
    "Development Status :: 4 - Beta",
    "Intended Audience :: Developers",
    "License :: OSI Approved :: MIT License",
    "Programming Language :: Python :: 3",
    "Programming Language :: Python :: 3.12",
    "Programming Language :: Python :: 3.13",
    "Typing :: Typed",
]

# Minimal runtime dependencies
dependencies = [
    "pyyaml>=6.0.1",
]

[project.optional-dependencies]
dev = [
    "pytest>=8.0.0",
    "pytest-cov>=4.1.0",
    "ruff>=0.1.0",
    "mypy>=1.0.0",
]

[project.urls]
Homepage = "https://github.com/fulmenhq/package-name"
Documentation = "https://github.com/fulmenhq/package-name/blob/main/README.md"
Repository = "https://github.com/fulmenhq/package-name"
Issues = "https://github.com/fulmenhq/package-name/issues"

[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[tool.hatch.build.targets.wheel]
packages = ["src/package_name"]

# Include py.typed marker
[tool.hatch.build.targets.wheel.force-include]
"src/package_name/py.typed" = "package_name/py.typed"
```

**Key Points**:

- Keep runtime dependencies minimal
- Include `Typing :: Typed` classifier
- Force-include `py.typed` in wheel

---

## Testing

### Test Coverage Requirements

**Minimum**: 90% coverage on public API  
**Goal**: 100% coverage on public API

```bash
# Run tests with coverage
uv run pytest --cov=src/package_name --cov-report=term-missing
```

### Test Structure

```python
"""Tests for core module."""
import pytest
from package_name.core.module1 import PublicClass, public_function
from package_name.exceptions import ProcessingError

def test_public_function():
    """Test public_function with valid input."""
    result = public_function(21)
    assert result == 42, f"Expected 42, got {result}"

def test_public_class_initialization():
    """Test PublicClass can be initialized with config."""
    config = {"key": "value"}
    obj = PublicClass(config=config)
    assert obj.config == config

def test_public_class_process():
    """Test PublicClass.process handles valid data."""
    obj = PublicClass(config={})
    result = obj.process("test data")
    assert result["processed"] is True

def test_public_class_process_error():
    """Test PublicClass.process raises error on invalid data."""
    obj = PublicClass(config={})
    with pytest.raises(ProcessingError, match="invalid"):
        obj.process("invalid data")

@pytest.fixture
def sample_instance():
    """Provide sample PublicClass instance for tests."""
    return PublicClass(config={"test": True})
```

---

## Documentation

### README.md Structure

```markdown
# Package Name

Brief description (1-2 sentences).

## Installation

\`\`\`bash
pip install package-name
\`\`\`

Or with uv:

\`\`\`bash
uv add package-name
\`\`\`

## Requirements

- Python 3.12 or later

## Quick Start

\`\`\`python
from package_name import main_function

result = main_function(data)
print(result)
\`\`\`

## API Reference

### `main_function(data)`

Description of function...

**Parameters:**

- `data` (str): Input data

**Returns:**

- `dict`: Result dictionary

## Development

\`\`\`bash

# Install with dev dependencies

uv sync

# Run tests

uv run pytest

# Lint

uv run ruff check .
\`\`\`

## License

MIT License - See LICENSE file.
```

### CHANGELOG.md

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

## [1.0.0] - 2025-10-08

### Added

- Initial release
- Core functionality for X
- Support for Y

### Changed

- Improved Z performance

### Fixed

- Bug in X handling
```

---

## Best Practices

### 1. Public vs Private API

```python
# Public API (exported in __all__)
def public_function(x: int) -> int:
    """Public function with full documentation."""
    return _internal_helper(x)

# Private helper (leading underscore)
def _internal_helper(x: int) -> int:
    """Internal helper, not part of public API."""
    return x * 2
```

### 2. Lazy Imports (For Heavy Dependencies)

```python
# __init__.py
def expensive_function():
    """Function that imports heavy dependency only when called."""
    from package_name.heavy import heavy_function
    return heavy_function()
```

### 3. Versioning

Use CalVer or SemVer consistently:

```python
# CalVer (crucible style)
__version__ = "2025.10.2"

# SemVer
__version__ = "1.2.3"
```

### 4. Type Hints Everywhere

```python
# ✅ GOOD - Complete type hints
def process(
    data: str,
    options: dict[str, Any] | None = None
) -> dict[str, Any]:
    pass

# ❌ BAD - Missing hints
def process(data, options=None):
    pass
```

### 5. Logging Best Practices

```python
import logging

logger = logging.getLogger(__name__)

def library_function():
    """Library function using logging."""
    logger.debug("Processing started")
    # ... logic
    logger.info("Processing complete")
    # Never use print() in library code
```

---

## Publishing Workflow

### 1. Pre-release Checklist

- [ ] All tests passing
- [ ] Coverage ≥90%
- [ ] CHANGELOG.md updated
- [ ] Version bumped in `__init__.py` and `pyproject.toml`
- [ ] README.md updated with latest examples
- [ ] No `print()` statements in library code
- [ ] All public APIs have docstrings
- [ ] `py.typed` present

### 2. Build Distribution

```bash
# Build wheel and sdist
uv build

# Check dist/
ls dist/
# package_name-1.0.0-py3-none-any.whl
# package_name-1.0.0.tar.gz
```

### 3. Test Installation

```bash
# Test install from wheel
uv pip install dist/package_name-1.0.0-py3-none-any.whl

# Verify import
python -c "import package_name; print(package_name.__version__)"
```

### 4. Publish to PyPI

```bash
# Install twine
uv add --dev twine

# Upload to TestPyPI first
uv run twine upload --repository testpypi dist/*

# Verify on TestPyPI
uv pip install --index-url https://test.pypi.org/simple/ package-name

# Upload to PyPI
uv run twine upload dist/*
```

---

## Examples

### Minimal Library (Crucible Style)

```
crucible/
├── src/crucible/
│   ├── __init__.py       # Public API
│   ├── py.typed
│   ├── schemas.py        # Schema loading
│   └── terminal.py       # Terminal config
├── tests/
│   ├── test_schemas.py
│   └── test_terminal.py
└── pyproject.toml
```

### Full-Featured Library

```
library/
├── src/package_name/
│   ├── __init__.py
│   ├── py.typed
│   ├── core/
│   │   ├── processor.py
│   │   └── validator.py
│   ├── config/
│   │   └── models.py
│   ├── utils/
│   │   └── helpers.py
│   └── exceptions.py
├── tests/
├── docs/
└── schemas/
```

---

## Makefile

```makefile
.PHONY: install test test-cov lint format type-check build clean publish

install:  ## Install with dev dependencies
	uv sync

test:  ## Run tests
	uv run pytest

test-cov:  ## Run tests with coverage report
	uv run pytest --cov=src/package_name --cov-report=term-missing --cov-report=html

lint:  ## Run linting
	uv run ruff check .

format:  ## Format code
	uv run ruff format .

type-check:  ## Run type checking
	uv run mypy src/package_name

build:  ## Build distribution
	uv build

clean:  ## Clean build artifacts
	rm -rf dist/ build/ .pytest_cache/ .ruff_cache/ htmlcov/
	find . -type d -name __pycache__ -exec rm -rf {} +

publish-test:  ## Publish to TestPyPI
	uv run twine upload --repository testpypi dist/*

publish:  ## Publish to PyPI
	uv run twine upload dist/*
```

---

## Version Management

### Manual Versioning

```python
# src/package_name/__init__.py
__version__ = "1.0.0"
```

```toml
# pyproject.toml
[project]
version = "1.0.0"
```

**Update both** when bumping version.

### Script-Based Versioning (Crucible Style)

Use `scripts/update-version.ts` to sync version across files:

```typescript
function updatePython(version: string): boolean {
  const initPath = join(ROOT, "src/package_name/__init__.py");
  return replaceInFile(initPath, (contents) =>
    contents.replace(/__version__\s*=\s*"[^"]+"/, `__version__ = "${version}"`),
  );
}
```

---

## Checklist for New Library Projects

- [ ] Base Python standards applied (see README.md)
- [ ] src/ layout used
- [ ] `py.typed` marker present
- [ ] All public APIs have type hints
- [ ] All public APIs have docstrings (Google style)
- [ ] `__all__` defines public API
- [ ] Custom exceptions defined
- [ ] Test coverage ≥90%
- [ ] README.md with installation and usage
- [ ] CHANGELOG.md for version history
- [ ] No `print()` in library code (use logging)
- [ ] PyPI classifiers include `Typing :: Typed`

---

## References

- [PEP 561 - Distributing and Packaging Type Information](https://peps.python.org/pep-0561/)
- [Python Packaging User Guide](https://packaging.python.org/)
- [Python Repository Structure Standards](README.md)
- [Python Coding Standards](../../coding/python.md)

---

**Remember**: Libraries are imported, not executed. Keep APIs clean, documented, and type-safe.
