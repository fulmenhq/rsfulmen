---
title: "Python Coding Standards for FulmenHQ"
description: "Python-specific coding standards including type safety, Pydantic patterns, logging, testing, and standard library choices for enterprise-grade Python development"
author: "Pipeline Architect"
author_of_record: "Dave Thompson (https://github.com/3leapsdave)"
supervised_by: "@3leapsdave"
date: "2025-10-08"
last_updated: "2025-10-09"
status: "approved"
tags: ["standards", "coding", "python", "pydantic", "type-safety", "testing"]
related_docs: ["README.md"]
---

# Python Coding Standards for FulmenHQ

## Overview

This document establishes coding standards for FulmenHQ Python projects, ensuring consistency, quality, and adherence to enterprise-grade practices. As tools designed for scale, FulmenHQ projects require rigorous standards to maintain reliability and structured output integrity.

**Core Principle**: Write idiomatic Python code that is simple, readable, and maintainable, with strict type safety and clean output.

**Foundation**: This guide builds upon **[Cross-Language Coding Standards](README.md)** which establishes patterns for:

- Output hygiene (STDERR for logs, STDOUT for data)
- RFC3339 timestamps
- Schema validation with goneat
- CLI exit codes
- Logging standards
- Security practices

Read the cross-language standards first, then apply the Python-specific patterns below.

---

## 1. Critical Rules (Zero-Tolerance)

### 1.1 Python Version

**Minimum**: Python 3.12+

**Why Critical**:

- Type system improvements (PEP 695 - Type Parameter Syntax)
- Performance improvements (~5% faster than 3.11)
- Better error messages
- Modern f-string syntax improvements

```python
# pyproject.toml
requires-python = ">=3.12"
```

### 1.2 No `any` Types - Ever

```python
# ❌ WRONG - Defeats type checking
def process_data(data: any) -> any:
    pass

config: dict[str, any] = {}

# ✅ CORRECT - Be specific or use Unknown
from typing import Any

def process_data(data: dict[str, Any]) -> str:
    pass

# Better: Define specific types
class Config(TypedDict):
    port: int
    host: str
```

### 1.3 Type Hints Required

**Rule**: All public functions and methods must have complete type hints.

```python
# ❌ WRONG - Missing type hints
def process_file(filename, options):
    return result

# ✅ CORRECT - Complete type hints
def process_file(filename: str, options: dict[str, Any]) -> ProcessResult:
    return ProcessResult(...)
```

### 1.4 Output Hygiene ⚠️ **CRITICAL**

**Rule**: Output streams must remain clean for structured output (JSON, YAML) consumed by tools and automation.

**DO**: Use logging for all diagnostic output

```python
import logging

logger = logging.getLogger(__name__)

# ✅ Correct logging
logger.debug("Processing %d files", file_count)
logger.info("Operation completed in %.2fs: %d issues found", duration, issue_count)
logger.error("Failed to process: %s", error)
logger.warning("Configuration file not found, using defaults")
```

**DO NOT**: Pollute output streams with print statements

```python
# ❌ CRITICAL ERROR: Breaks structured output
print(f"DEBUG: Processing {filename}")
print("Status:", status)
sys.stderr.write("Error message\n")
```

**Why Critical**: FulmenHQ tools produce structured output consumed by:

- CI/CD pipelines expecting clean data
- Automated tools parsing results
- Agentic systems processing structured data
- Pre-commit/pre-push hooks expecting parseable output

---

## 2. Standard Libraries and Frameworks

FulmenHQ Python projects use a curated set of libraries and frameworks to ensure consistency, quality, and maintainability across the ecosystem.

### 2.1 Data Modeling and Validation

**Pydantic** (REQUIRED for data models)

**Why Pydantic**:

- Runtime validation with type hints
- Immutable models for configuration
- JSON Schema generation
- Excellent error messages
- Industry standard for modern Python

```python
from pydantic import BaseModel, Field, ConfigDict

class Config(BaseModel):
    model_config = ConfigDict(frozen=True)  # Immutable

    host: str = Field(default="localhost")
    port: int = Field(ge=1, le=65535)
```

**Use Cases**:

- Configuration models
- API request/response schemas
- Data transfer objects
- Settings management

**Do NOT use**: dataclasses for validated models, attrs, marshmallow

### 2.2 JSON Schema Standard

**JSON Schema Draft 2020-12** (REQUIRED when no other requirement)

**Why 2020-12**:

- Modern standard with `$defs` support
- Better composition with `unevaluatedProperties`
- Consistent with FulmenHQ schema standards
- Supported by modern validators

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://example.com/config.schema.json",
  "type": "object",
  "properties": {
    "host": { "type": "string" },
    "port": { "type": "integer", "minimum": 1, "maximum": 65535 }
  }
}
```

**Fallback**: Draft-07 only when required for compatibility

**Do NOT use**: Draft-04 or earlier (deprecated)

### 2.3 CLI Framework

**Click** (REQUIRED for CLI applications)

**Why Click**:

- Industry standard
- Decorator-based syntax
- Automatic help generation
- Testing support with `CliRunner`
- Rich integration support

```python
import click

@click.command()
@click.option("--verbose", "-v", is_flag=True)
@click.argument("target", type=click.Path(exists=True))
def process(verbose: bool, target: str) -> None:
    """Process the target file."""
    pass
```

**Use Cases**:

- Command-line tools
- Developer utilities
- CLI automation

**Do NOT use**: argparse (built-in, less ergonomic), typer (less mature), fire (unpredictable)

### 2.4 HTTP Client

**httpx** (RECOMMENDED for async HTTP) or **requests** (simple sync cases)

**Why httpx**:

- Async/await support
- HTTP/2 support
- Modern API similar to requests
- Better timeout handling

```python
import httpx

async with httpx.AsyncClient() as client:
    response = await client.get("https://api.example.com/data")
    return response.json()
```

**Use requests for**:

- Simple synchronous HTTP calls
- Legacy code compatibility

**Do NOT use**: urllib directly (use httpx/requests wrapper)

### 2.5 Date/Time Handling

**Pendulum** (RECOMMENDED for complex date/time) or **standard library datetime**

**Why Pendulum**:

- Timezone-aware by default
- Human-friendly intervals
- Better parsing than dateutil

```python
import pendulum

now = pendulum.now("America/New_York")
tomorrow = now.add(days=1)
interval = now.diff(tomorrow)
```

**Use standard datetime for**:

- Simple timestamps
- No timezone complexity

**Do NOT use**: arrow (deprecated), dateutil alone (verbose)

### 2.6 Configuration Files

**YAML via PyYAML** (RECOMMENDED for configs) or **TOML via tomli/tomllib**

**Why YAML for configs**:

- Human-readable
- Comments support
- Multi-line strings
- FulmenHQ standard for config files

```python
import yaml
from pathlib import Path

config_path = Path("config.yaml")
with config_path.open() as f:
    config_data = yaml.safe_load(f)
```

**Use TOML for**:

- Python project metadata (pyproject.toml)
- Simple key-value configs

**Do NOT use**: JSON for human-edited configs, INI files (limited)

### 2.7 Path Handling

**pathlib.Path** (REQUIRED over os.path)

**Why pathlib**:

- Object-oriented API
- Cross-platform
- Readable operations
- Modern Python standard

```python
from pathlib import Path

config_dir = Path.home() / ".config" / "app"
config_dir.mkdir(parents=True, exist_ok=True)

for config_file in config_dir.glob("*.yaml"):
    content = config_file.read_text()
```

**Do NOT use**: os.path string manipulation, manual path joining

### 2.8 Logging

**Standard library logging** (REQUIRED) with **structlog** (optional for structured logs)

**Why standard logging**:

- Universal compatibility
- Library standard
- Hierarchical loggers

```python
import logging

logger = logging.getLogger(__name__)
logger.info("Operation completed", extra={"duration": 1.5, "items": 42})
```

**Add structlog for**:

- Structured JSON logs
- Context binding
- Advanced filtering

**Do NOT use**: print(), custom logging frameworks

### 2.9 Testing Fixtures and Data

**pytest-fixtures** for test data, **Faker** for generated test data

**Why Faker**:

- Realistic test data generation
- Locale support
- Repeatable with seeds

```python
import pytest
from faker import Faker

@pytest.fixture
def faker_instance():
    """Provide Faker instance with fixed seed."""
    return Faker()
    Faker.seed(12345)
    return Faker()

def test_user_creation(faker_instance):
    """Test user creation with realistic data."""
    email = faker_instance.email()
    name = faker_instance.name()
    # ... test logic
```

**Use Cases**:

- Integration test data
- Fixture generation
- Data validation testing

### 2.10 Environment Variables

**python-dotenv** (RECOMMENDED) or **Pydantic Settings**

**Why python-dotenv**:

- Simple .env file loading
- Development/production parity
- 12-factor app pattern

```python
from dotenv import load_dotenv
import os

load_dotenv()
api_key = os.environ["API_KEY"]
```

**Use Pydantic Settings for**:

- Type-safe environment config
- Validation on load
- Complex settings

```python
from pydantic_settings import BaseSettings

class Settings(BaseSettings):
    api_key: str
    debug: bool = False

    class Config:
        env_file = ".env"
```

### 2.11 JSON Handling

**Standard library json** (DEFAULT) or **orjson** (performance-critical)

**Why standard json**:

- Built-in, zero dependencies
- Sufficient for most cases

```python
import json

data = {"key": "value"}
json_str = json.dumps(data, indent=2)
```

**Use orjson for**:

- High-throughput APIs
- Large JSON payloads
- Performance-critical paths

**Do NOT use**: ujson (less maintained), simplejson (unnecessary)

### 2.12 Schema Validation

**Goneat binary** (REQUIRED for FulmenHQ schemas) - NOT a Python library

**Why goneat**:

- FulmenHQ standard validator
- Multi-schema support
- Consistent across languages

```bash
goneat schema validate-schema schemas/config.schema.json
```

**Do NOT use**: Python jsonschema library for FulmenHQ schemas

### 2.13 Async Framework

**asyncio** (standard library) with **aiofiles** for file I/O

**Why asyncio**:

- Standard library
- Native async/await
- Event loop management

```python
import asyncio
import aiofiles

async def process_file(path: Path) -> str:
    async with aiofiles.open(path, mode="r") as f:
        return await f.read()
```

**Use Cases**:

- Concurrent I/O operations
- API servers (FastAPI)
- Long-running tasks

### 2.14 Terminal Output

**Rich** (RECOMMENDED for CLI output formatting)

**Why Rich**:

- Beautiful terminal output
- Progress bars, tables, syntax highlighting
- Zero-configuration prettiness

```python
from rich.console import Console
from rich.table import Table

console = Console()
console.print("[bold green]Success![/bold green]")

table = Table(title="Results")
table.add_column("Name")
table.add_column("Status")
table.add_row("Task 1", "✓")
console.print(table)
```

**Use Cases**:

- CLI progress indicators
- Formatted output
- Error messages with color

**Do NOT use**: colorama alone (less powerful), custom ANSI codes

### 2.15 Regular Expressions

- Use the third-party [`regex`](https://pypi.org/project/regex/) package for **all** regular expression work in
  FulmenHQ codebases. It offers superior Unicode support (grapheme clusters, case folding) and named set
  features needed for internationalization.
- Import it using the canonical alias `import regex as re` so existing call sites continue to reference `re`:

  ```python
  import regex as re

  EMAIL_PATTERN = re.compile(r"(?i)[\p{Letter}\p{Number}._%+-]+@[\p{Letter}\p{Number}.-]+\.[\p{Letter}]{2,}")
  ```

- When modifying legacy modules that still import the standard library `re`, migrate them to `regex` at the same
  time. Mixing both modules in the same project is discouraged and new code MUST NOT use `import re` directly.

---

## 3. Code Organization and Structure

### 2.1 Project Structure

Follow FulmenHQ's established structure:

```
project/
├── src/
│   └── package_name/
│       ├── __init__.py
│       ├── cli/              # CLI commands (if applicable)
│       ├── core/             # Core business logic
│       ├── config/           # Configuration handling
│       └── utils/            # Utilities
├── tests/
│   ├── __init__.py
│   ├── unit/
│   ├── integration/
│   └── fixtures/
├── schemas/                  # JSON/YAML schemas (if applicable)
├── pyproject.toml
├── ruff.toml
└── .python-version
```

### 2.2 Naming Conventions

- **Modules/Packages**: `snake_case` (e.g., `config_loader`, `file_processor`)
- **Classes**: `PascalCase` (e.g., `ConfigLoader`, `FileProcessor`)
- **Functions/Methods**: `snake_case` (e.g., `load_config`, `process_file`)
- **Constants**: `UPPER_SNAKE_CASE` (e.g., `MAX_RETRIES`, `DEFAULT_TIMEOUT`)
- **Type Variables**: `PascalCase` with `T` prefix (e.g., `TConfig`, `TResult`)

### 2.3 Import Organization

```python
# Standard library (alphabetical)
import json
import logging
from pathlib import Path
from typing import Any

# Third-party packages (alphabetical)
import click
import yaml
from pydantic import BaseModel

# Local imports (alphabetical)
from package_name.config import Config
from package_name.core import Processor
```

**Rule**: Group imports with blank lines between groups. Use `isort` or ruff to enforce.

---

## 4. Type Safety and Patterns

### 3.1 Pydantic for Data Validation

```python
from pydantic import BaseModel, Field, ConfigDict

class UserConfig(BaseModel):
    model_config = ConfigDict(frozen=True)  # Immutable configs

    username: str = Field(min_length=1)
    port: int = Field(ge=1, le=65535)
    enabled: bool = True
```

### 3.2 Computed Fields

```python
from pydantic import BaseModel, computed_field
from typing import ClassVar, FrozenSet

class MyModel(BaseModel):
    field1: int

    @computed_field
    @property
    def computed_value(self) -> int:
        return self.field1 * 2

    # Exclude computed fields from serialization
    _computed_fields: ClassVar[FrozenSet[str]] = frozenset(["computed_value"])

    def model_dump(self, **kwargs) -> dict[str, Any]:
        return super().model_dump(exclude=self._computed_fields, **kwargs)
```

### 3.3 Protocol Usage

```python
from typing import Protocol, runtime_checkable

@runtime_checkable
class Processor(Protocol):
    def process(self, data: str) -> dict[str, Any]: ...
    def validate(self) -> bool: ...
```

**Note**: Cannot mix Pydantic models with Protocol due to metaclass conflicts. Use `runtime_checkable` or abstract base classes instead.

### 3.4 Type Guards

```python
from typing import TypeGuard

def is_error(obj: object) -> TypeGuard[Exception]:
    return isinstance(obj, Exception)

# Usage
if is_error(result):
    logger.error("Process failed: %s", result)  # result is Exception type here
```

### 3.5 Schema-Driven Configuration Hydration

- Provide a single normalization layer (for example, `normalize_logger_config(data: Mapping[str, Any]) -> LoggerConfig`) that converts schema-authored camelCase keys to snake_case attributes, applies defaults, and flattens nested dictionaries such as `middleware[].config` into typed constructor kwargs. Avoid sprinkling ad-hoc `dict` mutations across the codebase.
- Treat policy enforcement as part of the hydration pipeline: resolve policy files, merge overrides, and fail fast when `enforceStrictMode` blocks a configuration. Do not leave `_load_policy`-style stubs in production code.
- Cover the mapper with exhaustive unit tests that assert every field, enum, and optional value round-trips correctly (including zero/empty values). Tests MUST verify alpha/numeric code normalization, throttle settings, middleware ordering, and boolean flags.
- Validate hydrated configs and emitted log events against the canonical schemas (for example, `schemas/observability/logging/v1.0.0/logger-config.schema.json` and `log-event.schema.json`) as part of the test suite. Prefer using shared fixtures to guarantee parity across languages.
- Keep the mapper pure and deterministic so other languages can port identical behaviour; document its contract in module docstrings and architecture notes.

---

## 5. Error Handling

### 4.1 Custom Exceptions

```python
class PackageError(Exception):
    """Base exception for package errors."""

class ConfigurationError(PackageError):
    """Raised when configuration is invalid."""

class ProcessingError(PackageError):
    """Raised when processing fails."""
```

### 4.2 Structured Error Handling

```python
from pathlib import Path

def load_config(path: Path) -> Config:
    try:
        with path.open() as f:
            data = yaml.safe_load(f)
    except FileNotFoundError as e:
        raise ConfigurationError(f"Config file not found: {path}") from e
    except yaml.YAMLError as e:
        raise ConfigurationError(f"Invalid YAML in {path}: {e}") from e

    try:
        return Config(**data)
    except ValidationError as e:
        raise ConfigurationError(f"Invalid config schema: {e}") from e
```

### 4.3 Error Context

```python
# ✅ CORRECT - Include context
raise ProcessingError(
    f"Failed to process file {filename} at line {line_num}: {error}"
)

# ❌ WRONG - No context
raise ProcessingError("Processing failed")
```

---

## 6. Testing Standards

### 5.1 Test Organization

```
tests/
├── __init__.py
├── unit/
│   ├── test_config.py
│   ├── test_processor.py
│   └── test_utils.py
├── integration/
│   └── test_cli.py
└── fixtures/
    ├── valid_config.yaml
    └── invalid_config.yaml
```

### 5.2 pytest Conventions

```python
import pytest
from pathlib import Path

def test_load_valid_config():
    """Test loading a valid configuration file."""
    config_path = Path("tests/fixtures/valid_config.yaml")
    config = load_config(config_path)

    assert config.port == 8080, f"Expected port 8080, got {config.port}"
    assert config.host == "localhost", f"Expected host localhost, got {config.host}"

def test_load_missing_config():
    """Test that missing config raises appropriate error."""
    with pytest.raises(ConfigurationError, match="Config file not found"):
        load_config(Path("nonexistent.yaml"))

@pytest.fixture
def sample_config():
    """Provide a sample configuration for testing."""
    return Config(port=8080, host="localhost")
```

### 5.3 Test Docstrings

**Rule**: Every test function must have a docstring explaining the scenario.

```python
def test_processor_handles_empty_input():
    """Test that processor correctly handles empty input without errors."""
    result = process([])
    assert result.success is True
```

### 5.4 Schema Contract Fixtures & Golden Events

- Maintain canonical fixtures under `tests/fixtures/logging/` (or equivalent) that represent every supported profile, sink, and middleware combination. These fixtures MUST conform to the Crucible schemas and be versioned alongside the library.
- Require CI to load each fixture through the normalization layer, emit sample log events, and validate both the hydrated configuration and output JSON against the schemas. Use snapshot tests to detect behavioural drift.
- Add table-driven tests that cover policy enforcement scenarios (`allow`, `deny`, strict mode) so regressions surface immediately when contracts change.
- Share fixtures and helpers across language foundations whenever possible so cross-language parity stays visible during audits.

---

## 7. Logging and Output

### 6.1 Logger Initialization

```python
import logging

# Module-level logger (lazy initialization safe)
logger = logging.getLogger(__name__)

def process_data(data: dict[str, Any]) -> None:
    logger.debug("Processing data with %d keys", len(data))
    # ... processing logic
    logger.info("Processing complete")
```

### 6.2 Log Levels

- `DEBUG`: Detailed diagnostic information
- `INFO`: General operational messages
- `WARNING`: Warning conditions (non-fatal)
- `ERROR`: Error conditions (may cause failures)
- `CRITICAL`: Critical failures requiring immediate attention

### 6.3 Structured Logging

```python
logger.info(
    "Operation completed",
    extra={
        "duration_ms": duration * 1000,
        "files_processed": file_count,
        "issues_found": len(issues),
    }
)
```

---

## 8. Code Style and Formatting

### 7.1 Ruff Configuration

Use separate `ruff.toml`:

```toml
line-length = 120
indent-width = 4
target-version = "py312"

[lint]
select = ["E4", "E7", "E9", "F", "B"]
```

### 7.2 Docstring Style

Use Google-style docstrings:

```python
def process_file(path: Path, options: dict[str, Any]) -> ProcessResult:
    """Process a file according to provided options.

    Args:
        path: Path to the file to process.
        options: Processing options as key-value pairs.

    Returns:
        ProcessResult containing processing outcome and any issues found.

    Raises:
        ProcessingError: If file processing fails.
        FileNotFoundError: If path does not exist.

    Example:
        >>> result = process_file(Path("data.json"), {"validate": True})
        >>> print(result.success)
        True
    """
```

### 7.3 f-strings vs String Literals

```python
# ❌ WRONG - Unnecessary f-string
message = f"Hello world"
url = f"https://api.example.com"

# ✅ CORRECT - Plain strings for literals
message = "Hello world"
url = "https://api.example.com"

# ✅ CORRECT - f-strings only for interpolation
greeting = f"Hello, {name}!"
api_url = f"https://api.example.com/users/{user_id}"
```

---

## 9. Class Design Best Practices

### 8.1 Static vs Class vs Instance Methods

```python
class DataProcessor:
    _registry: ClassVar[dict[str, type]] = {}

    def __init__(self, config: Config):
        self.config = config

    # Instance method - needs instance state
    def process(self, data: str) -> ProcessResult:
        return ProcessResult(data=data, config=self.config)

    # Class method - factory pattern
    @classmethod
    def from_file(cls, path: Path) -> "DataProcessor":
        config = load_config(path)
        return cls(config)

    # Static method - no state needed
    @staticmethod
    def validate_format(data: str) -> bool:
        return data.startswith("v1:")
```

**Decision Tree**:

- Need instance attributes? → Instance method
- Creating instances? → Class method
- Pure function, no state? → Static method

### 8.2 Pydantic Model Design

```python
from pydantic import BaseModel, ConfigDict
from typing import ClassVar

class ImmutableConfig(BaseModel):
    model_config = ConfigDict(frozen=True)

    port: int
    host: str

    # Class variables use ClassVar
    _default_port: ClassVar[int] = 8080

    @classmethod
    def default(cls) -> "ImmutableConfig":
        return cls(port=cls._default_port, host="localhost")
```

---

## 10. Performance and Best Practices

### 9.1 Path Handling

```python
from pathlib import Path

# ✅ CORRECT - Use Path objects
def load_file(path: Path) -> str:
    return path.read_text()

# ❌ WRONG - String path manipulation
def load_file(path: str) -> str:
    with open(path) as f:
        return f.read()
```

### 9.2 Resource Management

```python
from contextlib import contextmanager

@contextmanager
def managed_resource(resource_id: str):
    """Manage resource lifecycle with proper cleanup."""
    resource = acquire_resource(resource_id)
    try:
        yield resource
    finally:
        release_resource(resource)
```

### 9.3 Iteration Patterns

```python
# ✅ CORRECT - Use comprehensions for transformation
processed = [process(item) for item in items]

# ✅ CORRECT - Generator for large datasets
def process_large_dataset(items: list[str]) -> Iterator[ProcessResult]:
    for item in items:
        yield process(item)
```

---

## 11. Security and Validation

### 10.1 Input Validation

```python
from pathlib import Path

def validate_path(path: Path) -> Path:
    """Validate path is safe and accessible."""
    # Resolve to absolute path
    abs_path = path.resolve()

    # Check for path traversal
    if ".." in abs_path.parts:
        raise ValueError(f"Path traversal detected: {path}")

    # Verify path exists
    if not abs_path.exists():
        raise FileNotFoundError(f"Path does not exist: {abs_path}")

    return abs_path
```

### 10.2 Secrets Management

```python
import os

# ✅ CORRECT - Environment variables for secrets
api_key = os.environ.get("API_KEY")
if not api_key:
    raise ConfigurationError("API_KEY environment variable required")

# ❌ WRONG - Hardcoded secrets
api_key = "sk_live_12345..."  # NEVER DO THIS
```

---

## 12. Common Anti-Patterns to Avoid

### 11.1 Mutable Default Arguments

```python
# ❌ WRONG - Mutable default
def process_items(items: list[str] = []):
    items.append("default")
    return items

# ✅ CORRECT - None with conditional assignment
def process_items(items: list[str] | None = None) -> list[str]:
    if items is None:
        items = []
    items.append("default")
    return items
```

### 11.2 Bare Except Clauses

```python
# ❌ WRONG - Catches everything including KeyboardInterrupt
try:
    risky_operation()
except:
    handle_error()

# ✅ CORRECT - Specific exception handling
try:
    risky_operation()
except (ValueError, TypeError) as e:
    logger.error("Operation failed: %s", e)
    raise
```

### 11.3 Type Ignore Overuse

```python
# ❌ WRONG - Hiding type issues
result = unsafe_operation()  # type: ignore

# ✅ CORRECT - Fix the type issue
result: ProcessResult = unsafe_operation()  # Proper annotation
```

**Rule**: Use `# type: ignore` only when deliberately bypassing type checks for valid reasons. Document why.

---

## 13. Code Review Checklist

Before submitting code, verify:

- [ ] Python 3.12+ compatibility
- [ ] Complete type hints on all public functions
- [ ] No `print()` statements in core logic (use logging)
- [ ] All errors properly handled with context
- [ ] Tests cover happy path and error conditions
- [ ] Docstrings on all public functions and classes
- [ ] Ruff linting passes with zero issues
- [ ] No `any` types or excessive `# type: ignore`
- [ ] Pydantic models use `frozen=True` for configs
- [ ] No mutable default arguments

---

## 14. Tools and Enforcement

### 13.1 Required Tools

- **uv**: Package management and virtual environments
- **ruff**: Linting and formatting
- **pytest**: Testing framework
- **mypy** (optional): Additional type checking

### 13.2 Pre-commit Hooks

Configure via `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: https://github.com/astral-sh/ruff-pre-commit
    rev: v0.1.0
    hooks:
      - id: ruff
        args: [--fix]
```

### 13.3 CI Integration

```yaml
# .github/workflows/test.yml
- name: Lint with ruff
  run: uv run ruff check .

- name: Run tests
  run: uv run pytest
```

---

## 14. Portable Testing

Fulmen Python projects must keep `pytest` suites deterministic across laptops, CI, and sandboxed environments. Follow the cross-language [Portable Testing Practices](../testing/portable-testing-practices.md) and apply these Python patterns:

- Use `tmp_path`/`tmp_path_factory` for temporary files and directories; never write to repo roots or `/tmp` without cleanup.
- Provide shared helpers (e.g., `require_network`, `require_dns`) that call `pytest.skip` with explicit reasons when capabilities are unavailable.
- Seed randomness via `random.seed(SEED)` or `secrets.randbelow` mocks to keep fixtures reproducible.
- Prefer in-memory dependencies (SQLite `:memory:`, mocked HTTP servers) for unit tests; guard real integrations with markers (`@pytest.mark.integration`) and capability checks.
- Use fixtures' `yield`/`finalizer` patterns to ensure sockets, threads, and background tasks are torn down regardless of test outcome.

**CLI Testing**: For Typer/Click-based CLIs (e.g., microtool forges), follow the application factory pattern in [Language-Specific Testing Patterns](../testing/language-testing-patterns.md#python--typerclick-cli-isolation) to isolate context between tests.

---

## Conclusion

These standards ensure FulmenHQ Python projects maintain reliability as production-grade tools. The emphasis on type safety, output hygiene, and structured error handling is critical for maintaining code quality and seamless automation integration.

**Remember**: Type hints prevent bugs. Clean output enables automation. Follow these patterns consistently.

_Adherence to these standards ensures enterprise-grade reliability and seamless integration across development workflows._
