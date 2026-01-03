---
title: "Python Repository Structure Standards"
description: "Mandatory tooling (uv, ruff, pytest) and broad requirements for all FulmenHQ Python projects regardless of application type"
author: "Pipeline Architect"
author_of_record: "Dave Thompson (https://github.com/3leapsdave)"
supervised_by: "@3leapsdave"
date: "2025-10-08"
last_updated: "2025-10-08"
status: "approved"
tags: ["standards", "repository-structure", "python", "uv", "ruff", "tooling"]
related_docs: ["cli-click.md", "library.md", "../../coding/python.md"]
---

# Python Repository Structure Standards

## Overview

This document establishes **mandatory tooling and broad requirements** for all FulmenHQ Python projects. These are non-negotiable standards that apply regardless of application category.

Before selecting a variant guide, review the cross-language taxonomy in
[`docs/standards/repository-structure/README.md`](../README.md) to confirm the canonical
category definitions (e.g., `cli`, `workhorse`, `codex`).

For **application-specific patterns** (CLI, FastAPI workhorse, library, etc.), see the variant-specific guides in this directory after reading the taxonomy.

---

## Mandatory Tooling

### Package Management: uv (REQUIRED)

**Why uv**:

- Extremely fast (Rust-based, 10-100x faster than pip)
- Single tool for all Python workflows (deps, venv, version management)
- Lock file support (`uv.lock`)
- Built-in Python version management
- Industry momentum (same team as ruff)

**Installation**:

```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
```

**Basic Commands**:

```bash
# Initialize new project
uv init --lib my-package

# Add dependencies
uv add requests pydantic

# Add dev dependencies
uv add --dev pytest ruff mypy

# Install dependencies
uv sync

# Run commands in virtual environment
uv run pytest
uv run python my_script.py
```

**Do NOT use**: pip, pip-tools, pipenv, poetry (unless legacy migration in progress)

---

### Linting and Formatting: ruff (REQUIRED)

**Why ruff**:

- Extremely fast (Rust-based)
- Replaces multiple tools (flake8, black, isort, pydocstyle, etc.)
- Built by Astral (same team as uv)
- Comprehensive rule coverage

**Configuration**: Use separate `ruff.toml` file

**Minimum `ruff.toml`**:

```toml
line-length = 120
indent-width = 4
target-version = "py312"

[format]
quote-style = "double"
indent-style = "space"
skip-magic-trailing-comma = false

[lint]
select = ["E4", "E7", "E9", "F", "B"]
ignore = []
fixable = ["ALL"]
unfixable = []

[lint.per-file-ignores]
"__init__.py" = ["E402"]
"tests/*" = ["E402"]
```

**Do NOT use**: black, flake8, pylint, autopep8 as primary tools

---

### Testing: pytest (REQUIRED)

**Why pytest**:

- Industry standard for Python testing
- Rich plugin ecosystem
- Excellent fixture system
- Clear, readable test syntax

**Configuration**: In `pyproject.toml`

```toml
[tool.pytest.ini_options]
testpaths = ["tests"]
python_files = ["test_*.py"]
addopts = "--cov=src --cov-report=term-missing"
```

**Coverage**: Use `pytest-cov` for coverage reporting

**Do NOT use**: unittest as primary framework (OK for specific needs)

---

### Python Version: 3.12+ (REQUIRED)

**Why Python 3.12**:

- Type system improvements (PEP 695 - Type Parameter Syntax)
- Performance improvements (~5% faster than 3.11)
- Better error messages
- Modern f-string syntax improvements

**Minimum Version**: 3.12  
**Recommended**: 3.12.x (latest patch)  
**Not Yet**: 3.13 (too new for enterprise adoption)

**Version Pinning**:

```bash
# .python-version
3.12
```

```toml
# pyproject.toml
[project]
requires-python = ">=3.12"
```

**Python Version Management**: Use `uv python install 3.12`

---

## Project Structure Standards

### Required Files

Every Python project MUST have:

```
project/
├── .python-version          # Python version pin (3.12)
├── pyproject.toml           # Project metadata, dependencies, build config
├── ruff.toml                # Linting/formatting config (separate file)
├── uv.lock                  # Locked dependencies
├── README.md                # Project documentation
├── LICENSE                  # License file
├── .gitignore               # Git ignore patterns
├── src/
│   └── package_name/
│       └── __init__.py
└── tests/
    └── __init__.py
```

### Optional Files (Recommended)

```
├── CHANGELOG.md             # Version history
├── CONTRIBUTING.md          # Contribution guidelines
├── Makefile                 # Common development commands
└── .github/
    └── workflows/
        └── test.yml         # CI/CD workflow
```

---

## Package Metadata (`pyproject.toml`)

### Minimum Required Metadata

```toml
[project]
name = "my-package"
version = "0.1.0"
description = "Brief description"
authors = [
    {name = "Author Name", email = "author@example.com"}
]
requires-python = ">=3.12"
readme = "README.md"
license = {text = "MIT"}

dependencies = [
    # Runtime dependencies
]

[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[tool.hatch.build.targets.wheel]
packages = ["src/my_package"]
```

### Development Dependencies

```toml
[project.optional-dependencies]
dev = [
    "pytest>=8.0.0",
    "pytest-cov>=4.1.0",
    "ruff>=0.1.0",
]
```

Or with uv:

```toml
[tool.uv]
dev-dependencies = [
    "pytest>=8.0.0",
    "pytest-cov>=4.1.0",
    "ruff>=0.1.0",
]
```

---

## Type Checking (STRONGLY RECOMMENDED)

### mypy Configuration

While not mandatory, type checking with mypy is **strongly recommended**:

```toml
[tool.mypy]
python_version = "3.12"
strict = true
warn_return_any = true
warn_unused_configs = true
disallow_untyped_defs = true
```

**Type Stubs**: Include `py.typed` marker for libraries

```
src/package_name/
├── __init__.py
└── py.typed          # Empty file, signals typed package
```

---

## Source Layout

### src/ Layout (REQUIRED for Libraries)

```
src/
└── package_name/
    ├── __init__.py
    ├── core/
    │   ├── __init__.py
    │   └── logic.py
    ├── cli/             # Optional: CLI commands
    ├── api/             # Optional: API routes
    └── utils/
        ├── __init__.py
        └── helpers.py
```

**Why src/ layout**:

- Prevents accidental imports of uninstalled code
- Cleaner import paths
- Better for packaging and distribution

### Tests Layout

```
tests/
├── __init__.py
├── unit/
│   ├── test_core.py
│   └── test_utils.py
├── integration/
│   └── test_api.py
└── fixtures/
    ├── sample_data.json
    └── test_config.yaml
```

---

## Virtual Environment Management

### uv-Managed Virtual Environments

**Location**: `.venv/` in project root (auto-created by uv)

```bash
# Create/sync virtual environment
uv sync

# Activate virtual environment (optional, uv run handles this)
source .venv/bin/activate  # Linux/Mac
.venv\Scripts\activate     # Windows

# Run commands without activation
uv run pytest
uv run python -m my_package
```

**Gitignore**: Always ignore `.venv/`

```gitignore
.venv/
__pycache__/
*.py[cod]
.pytest_cache/
.ruff_cache/
```

---

## Build System

### Build Backend: hatchling (RECOMMENDED)

```toml
[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[tool.hatch.build.targets.wheel]
packages = ["src/package_name"]
```

**Alternative**: setuptools (legacy projects only)

**Building**:

```bash
uv build
```

---

## Common Makefile Targets

```makefile
.PHONY: install test lint format clean

install:  ## Install dependencies
	uv sync

test:  ## Run tests
	uv run pytest

lint:  ## Run linting
	uv run ruff check .

format:  ## Format code
	uv run ruff format .

clean:  ## Clean build artifacts
	rm -rf dist/ build/ .pytest_cache/ .ruff_cache/
	find . -type d -name __pycache__ -exec rm -rf {} +
```

---

## Dependency Management

### Dependency Declaration

**Runtime Dependencies** (required for package to work):

```toml
dependencies = [
    "pydantic>=2.0.0",
    "pyyaml>=6.0.1",
]
```

**Development Dependencies** (testing, linting, etc.):

```toml
[project.optional-dependencies]
dev = [
    "pytest>=8.0.0",
    "ruff>=0.1.0",
]
```

### Version Constraints

```toml
# ✅ GOOD - Allows patch updates
"pydantic>=2.8.0,<3.0.0"
"requests>=2.31.0"

# ❌ BAD - Too loose
"pydantic"  # No version constraint

# ❌ BAD - Too strict
"pydantic==2.8.2"  # Exact pin (only for lock file)
```

### Lock Files

`uv.lock` contains exact versions for reproducible builds:

```bash
# Update lock file
uv lock

# Install from lock file
uv sync
```

**Commit**: Always commit `uv.lock` to version control

---

## Continuous Integration

### GitHub Actions Example

```yaml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        python-version: ["3.12", "3.13"]

    steps:
      - uses: actions/checkout@v4

      - name: Install uv
        uses: astral-sh/setup-uv@v3

      - name: Set up Python
        run: uv python install ${{ matrix.python-version }}

      - name: Install dependencies
        run: uv sync

      - name: Lint
        run: uv run ruff check .

      - name: Test
        run: uv run pytest
```

---

## Repository Variants

This document covers **broad requirements** for all Python projects. For **application-specific patterns**, see:

- **[CLI Applications (Click)](cli-click.md)** - Click-based command-line tools
- **[Web APIs (FastAPI)](api-fastapi.md)** - FastAPI web services (planned)
- **[Libraries](library.md)** - Pure Python libraries
- **[Data Pipelines](data-pipeline.md)** - ETL/data processing apps (planned)

Each variant builds on these standards with additional patterns specific to that use case.

---

## Migration Guide

### From pip/pip-tools to uv

```bash
# 1. Install uv
curl -LsSf https://astral.sh/uv/install.sh | sh

# 2. Create pyproject.toml from requirements.txt
uv init --lib my-package

# 3. Add dependencies
uv add $(cat requirements.txt)

# 4. Add dev dependencies
uv add --dev $(cat requirements-dev.txt)

# 5. Remove old files
rm requirements.txt requirements-dev.txt setup.py setup.cfg
```

### From black/flake8 to ruff

```bash
# 1. Remove old tools
uv remove black flake8 isort

# 2. Add ruff
uv add --dev ruff

# 3. Create ruff.toml (see example above)

# 4. Run initial format
uv run ruff format .

# 5. Fix linting issues
uv run ruff check --fix .
```

---

## Checklist for New Projects

- [ ] Python 3.12+ via `.python-version`
- [ ] `uv` for package management
- [ ] `ruff.toml` for linting/formatting
- [ ] `pytest` for testing with coverage
- [ ] `pyproject.toml` with complete metadata
- [ ] src/ layout for libraries
- [ ] Type hints on all public functions
- [ ] `py.typed` marker (libraries only)
- [ ] `uv.lock` committed to git
- [ ] README.md with usage examples
- [ ] LICENSE file
- [ ] CI workflow (.github/workflows/test.yml)
- [ ] Variant-specific structure (see variant guides)

---

## Exceptions and Approvals

Deviations from these standards require approval from @3leapsdave or repository maintainer. Document exceptions in project README.md with rationale.

**Example exceptions**:

- Legacy projects migrating to uv (temporary pip usage OK)
- Projects requiring Python 3.11 for specific library compatibility (document in README)
- Monorepos with existing build systems (document structure)

---

## References

- [uv documentation](https://docs.astral.sh/uv/)
- [ruff documentation](https://docs.astral.sh/ruff/)
- [pytest documentation](https://docs.pytest.org/)
- [Python Packaging User Guide](https://packaging.python.org/)
- [PEP 621 - Project Metadata](https://peps.python.org/pep-0621/)

---

**Remember**: These are the foundations. Build your application structure on top of these standards using the variant guides.
