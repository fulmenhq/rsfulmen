---
title: "Python CLI Application Structure (Click)"
description: "Repository structure and patterns for Click-based CLI applications including command organization, configuration, and testing patterns"
author: "Pipeline Architect"
author_of_record: "Dave Thompson (https://github.com/3leapsdave)"
supervised_by: "@3leapsdave"
date: "2025-10-08"
last_updated: "2025-10-08"
status: "approved"
tags: ["standards", "repository-structure", "python", "cli", "click", "variant"]
related_docs: ["README.md", "library.md", "../../coding/python.md"]
---

# Python CLI Application Structure (Click)

## Overview

This guide establishes the **repository structure and patterns** for Click-based CLI applications in the FulmenHQ ecosystem. It builds upon the [Python Repository Structure Standards](README.md) with CLI-specific patterns.

**Target Use Cases**:

- Command-line tools and utilities
- CLI-based automation scripts
- Developer tooling
- System administration utilities

---

## Prerequisites

Read and apply **[Python Repository Structure Standards](README.md)** first. This document extends those requirements.

**Required Tooling** (from base standards):

- Python 3.12+
- uv (package management)
- ruff (linting/formatting)
- pytest (testing)

**Additional CLI-Specific Dependencies**:

- Click 8.1+ (CLI framework)
- Rich (terminal formatting, optional but recommended)
- Pydantic (configuration validation)

---

## Project Structure

```
cli-app/
├── .python-version              # 3.12
├── pyproject.toml               # Project metadata
├── ruff.toml                    # Linting config
├── uv.lock                      # Locked dependencies
├── README.md
├── LICENSE
├── Makefile                     # Development commands
│
├── src/
│   └── app_name/
│       ├── __init__.py          # Package init, version
│       ├── __main__.py          # Entry point for python -m app_name
│       ├── cli.py               # Click CLI definition
│       ├── commands/            # Command implementations
│       │   ├── __init__.py
│       │   ├── init.py          # app-name init
│       │   ├── process.py       # app-name process
│       │   └── config.py        # app-name config
│       ├── core/                # Business logic (CLI-agnostic)
│       │   ├── __init__.py
│       │   ├── processor.py
│       │   └── validator.py
│       ├── config/              # Configuration handling
│       │   ├── __init__.py
│       │   ├── models.py        # Pydantic config models
│       │   └── loader.py        # Config loading logic
│       └── utils/               # Shared utilities
│           ├── __init__.py
│           ├── logging.py       # Logging setup
│           └── output.py        # Output formatting (Rich)
│
├── tests/
│   ├── __init__.py
│   ├── unit/
│   │   ├── test_commands.py
│   │   ├── test_core.py
│   │   └── test_config.py
│   ├── integration/
│   │   └── test_cli.py          # End-to-end CLI tests
│   └── fixtures/
│       ├── sample_config.yaml
│       └── test_data/
│
└── schemas/                     # JSON/YAML schemas (optional)
    └── config.schema.json
```

---

## Key Components

### 1. Entry Point (`__main__.py`)

```python
"""Entry point for running package as a module: python -m app_name"""
from app_name.cli import cli

if __name__ == "__main__":
    cli()
```

### 2. CLI Definition (`cli.py`)

```python
"""Main CLI application using Click."""
import click
from app_name import __version__
from app_name.commands import init, process, config
from app_name.utils.logging import setup_logging

@click.group()
@click.version_option(version=__version__)
@click.option(
    "--verbose", "-v",
    is_flag=True,
    help="Enable verbose output"
)
@click.option(
    "--config",
    type=click.Path(exists=True),
    help="Path to configuration file"
)
@click.pass_context
def cli(ctx: click.Context, verbose: bool, config: str | None) -> None:
    """App Name - Brief description of what the CLI does."""
    ctx.ensure_object(dict)

    # Setup logging
    log_level = "DEBUG" if verbose else "INFO"
    setup_logging(log_level)

    # Store global options in context
    ctx.obj["verbose"] = verbose
    ctx.obj["config_path"] = config

# Register commands
cli.add_command(init.init_cmd)
cli.add_command(process.process_cmd)
cli.add_command(config.config_cmd)

if __name__ == "__main__":
    cli()
```

### 3. Command Implementation (`commands/process.py`)

```python
"""Process command implementation."""
import click
import logging
from pathlib import Path

from app_name.core.processor import Processor
from app_name.config.loader import load_config
from app_name.utils.output import print_result, print_error

logger = logging.getLogger(__name__)

@click.command(name="process")
@click.argument(
    "target",
    type=click.Path(exists=True, path_type=Path)
)
@click.option(
    "--output", "-o",
    type=click.Path(path_type=Path),
    help="Output file path"
)
@click.option(
    "--format",
    type=click.Choice(["json", "yaml", "text"]),
    default="text",
    help="Output format"
)
@click.pass_context
def process_cmd(
    ctx: click.Context,
    target: Path,
    output: Path | None,
    format: str
) -> None:
    """Process the specified target.

    TARGET: Path to file or directory to process.
    """
    try:
        # Load configuration
        config = load_config(ctx.obj.get("config_path"))

        # Create processor
        processor = Processor(config=config)

        # Process target
        logger.info("Processing %s", target)
        result = processor.process(target)

        # Output results
        print_result(result, format=format, output=output)

    except Exception as e:
        logger.error("Processing failed: %s", e)
        print_error(str(e))
        raise click.Abort()
```

### 4. Configuration Models (`config/models.py`)

```python
"""Configuration models using Pydantic."""
from pydantic import BaseModel, Field, ConfigDict
from pathlib import Path

class ProcessorConfig(BaseModel):
    """Processor configuration."""
    model_config = ConfigDict(frozen=True)

    max_workers: int = Field(default=4, ge=1, le=16)
    timeout: int = Field(default=30, ge=1)
    verbose: bool = False

class AppConfig(BaseModel):
    """Application configuration."""
    model_config = ConfigDict(frozen=True)

    version: str = "1.0.0"
    processor: ProcessorConfig = Field(default_factory=ProcessorConfig)
    output_dir: Path = Field(default=Path("./output"))

    @classmethod
    def default(cls) -> "AppConfig":
        """Create default configuration."""
        return cls()
```

### 5. Logging Setup (`utils/logging.py`)

```python
"""Logging configuration."""
import logging
import sys
from typing import Literal

LogLevel = Literal["DEBUG", "INFO", "WARNING", "ERROR", "CRITICAL"]

def setup_logging(level: LogLevel = "INFO") -> None:
    """Configure logging for the application.

    Args:
        level: Logging level to use.
    """
    logging.basicConfig(
        level=level,
        format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
        datefmt="%Y-%m-%d %H:%M:%S",
        stream=sys.stderr,  # Keep stderr for logs, stdout for output
    )
```

### 6. Output Formatting (`utils/output.py`)

```python
"""Output formatting using Rich."""
import json
from pathlib import Path
from typing import Any

import yaml
from rich.console import Console
from rich.table import Table

console = Console()

def print_result(
    result: dict[str, Any],
    format: str = "text",
    output: Path | None = None
) -> None:
    """Print result in specified format.

    Args:
        result: Result data to print.
        format: Output format (json, yaml, text).
        output: Optional output file path.
    """
    if format == "json":
        content = json.dumps(result, indent=2)
    elif format == "yaml":
        content = yaml.dump(result, default_flow_style=False)
    else:
        content = _format_text(result)

    if output:
        output.write_text(content)
        console.print(f"✓ Results written to {output}", style="green")
    else:
        print(content)  # stdout for structured output

def print_error(message: str) -> None:
    """Print error message to stderr.

    Args:
        message: Error message to display.
    """
    console.print(f"✗ Error: {message}", style="bold red", err=True)

def _format_text(data: dict[str, Any]) -> str:
    """Format data as human-readable text."""
    # Use Rich tables, formatting, etc.
    pass
```

---

## Package Metadata

### `pyproject.toml`

```toml
[project]
name = "app-name"
version = "0.1.0"
description = "Brief description of CLI tool"
authors = [
    {name = "Your Name", email = "you@example.com"}
]
requires-python = ">=3.12"
readme = "README.md"
license = {text = "MIT"}

dependencies = [
    "click>=8.1.0",
    "pydantic>=2.8.0",
    "pyyaml>=6.0.1",
    "rich>=13.0.0",  # Optional but recommended
]

[project.optional-dependencies]
dev = [
    "pytest>=8.0.0",
    "pytest-cov>=4.1.0",
    "ruff>=0.1.0",
]

# CLI entry point
[project.scripts]
app-name = "app_name.cli:cli"

[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[tool.hatch.build.targets.wheel]
packages = ["src/app_name"]
```

**Entry Point**: Defines `app-name` command that runs `cli()` function

---

## Testing

### CLI Testing with Click

```python
"""Test CLI commands."""
import pytest
from click.testing import CliRunner
from app_name.cli import cli

@pytest.fixture
def runner():
    """Provide Click CLI test runner."""
    return CliRunner()

def test_cli_version(runner):
    """Test --version flag."""
    result = runner.invoke(cli, ["--version"])
    assert result.exit_code == 0
    assert "version" in result.output.lower()

def test_process_command(runner, tmp_path):
    """Test process command with temporary file."""
    # Create test file
    test_file = tmp_path / "test.txt"
    test_file.write_text("test content")

    # Run command
    result = runner.invoke(cli, ["process", str(test_file)])

    assert result.exit_code == 0
    assert "success" in result.output.lower()

def test_process_command_missing_file(runner):
    """Test process command with missing file."""
    result = runner.invoke(cli, ["process", "/nonexistent/file.txt"])

    assert result.exit_code != 0
    assert "error" in result.output.lower()
```

### Integration Testing

```python
"""Integration tests for CLI."""
import subprocess
import pytest
from pathlib import Path

def test_cli_installed():
    """Test that CLI is installed and accessible."""
    result = subprocess.run(
        ["app-name", "--version"],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0

def test_cli_process_integration(tmp_path):
    """Test full CLI process workflow."""
    # Setup test environment
    test_file = tmp_path / "input.txt"
    test_file.write_text("test data")
    output_file = tmp_path / "output.json"

    # Run CLI
    result = subprocess.run(
        ["app-name", "process", str(test_file), "-o", str(output_file)],
        capture_output=True,
        text=True
    )

    assert result.returncode == 0
    assert output_file.exists()
```

---

## Best Practices

### 1. Separate CLI from Core Logic

```python
# ✅ GOOD - CLI delegates to core
@click.command()
@click.argument("target")
def process_cmd(target: str) -> None:
    processor = Processor()
    result = processor.process(Path(target))
    print_result(result)

# ❌ BAD - Business logic in CLI
@click.command()
@click.argument("target")
def process_cmd(target: str) -> None:
    # ... complex processing logic here
```

### 2. Use Context for Global State

```python
@click.group()
@click.option("--config", type=click.Path())
@click.pass_context
def cli(ctx: click.Context, config: str | None) -> None:
    ctx.ensure_object(dict)
    ctx.obj["config"] = load_config(config)

@click.command()
@click.pass_context
def sub_command(ctx: click.Context) -> None:
    config = ctx.obj["config"]  # Access global config
```

### 3. Progress Indicators for Long Operations

```python
from rich.progress import track

def process_files(files: list[Path]) -> None:
    """Process files with progress bar."""
    for file in track(files, description="Processing..."):
        process_file(file)
```

### 4. Configuration File Support

```python
# config/loader.py
def load_config(path: Path | None = None) -> AppConfig:
    """Load configuration from file or defaults."""
    if path is None:
        return AppConfig.default()

    if not path.exists():
        raise ConfigurationError(f"Config file not found: {path}")

    with path.open() as f:
        data = yaml.safe_load(f)

    return AppConfig(**data)
```

---

## Command Patterns

### Simple Command

```python
@click.command()
@click.argument("name")
def greet(name: str) -> None:
    """Greet a user by name."""
    click.echo(f"Hello, {name}!")
```

### Command with Options

```python
@click.command()
@click.option("--count", "-n", default=1, help="Number of greetings")
@click.option("--name", prompt="Your name", help="The person to greet")
def hello(count: int, name: str) -> None:
    """Simple program that greets NAME for a total of COUNT times."""
    for _ in range(count):
        click.echo(f"Hello, {name}!")
```

### Command Group

```python
@click.group()
def config() -> None:
    """Manage configuration."""
    pass

@config.command()
def show() -> None:
    """Show current configuration."""
    pass

@config.command()
@click.argument("key")
@click.argument("value")
def set(key: str, value: str) -> None:
    """Set configuration value."""
    pass
```

---

## Makefile

```makefile
.PHONY: install test lint format run clean

install:  ## Install package in development mode
	uv sync

test:  ## Run tests
	uv run pytest

lint:  ## Run linting
	uv run ruff check .

format:  ## Format code
	uv run ruff format .

run:  ## Run CLI in development
	uv run python -m app_name

clean:  ## Clean build artifacts
	rm -rf dist/ build/ .pytest_cache/ .ruff_cache/
	find . -type d -name __pycache__ -exec rm -rf {} +
```

---

## Development Workflow

```bash
# 1. Initialize project
uv init --lib app-name
cd app-name

# 2. Add dependencies
uv add click pydantic pyyaml rich

# 3. Add dev dependencies
uv add --dev pytest pytest-cov ruff

# 4. Create structure (see above)

# 5. Install in development mode
uv sync

# 6. Run CLI during development
uv run python -m app_name --help
uv run app-name --help  # After install

# 7. Test
uv run pytest

# 8. Build distribution
uv build
```

---

## Distribution

### Install from Source

```bash
uv pip install .
```

### Install in Development Mode

```bash
uv pip install -e .
```

### Build Wheel

```bash
uv build
# Creates dist/app_name-0.1.0-py3-none-any.whl
```

---

## Example CLIs

**Simple utility**:

```
cli-app/
├── src/app_name/
│   ├── __init__.py
│   ├── __main__.py
│   └── cli.py
└── tests/
```

**Complex tool** (like goneat):

```
cli-app/
├── src/app_name/
│   ├── cli.py
│   ├── commands/
│   │   ├── assess.py
│   │   ├── format.py
│   │   └── validate.py
│   ├── core/
│   │   ├── processors/
│   │   └── validators/
│   └── config/
└── tests/
```

---

## Checklist for New CLI Projects

- [ ] Base Python standards applied (see README.md)
- [ ] Click 8.1+ installed
- [ ] Entry point configured in pyproject.toml
- [ ] CLI separated from core logic
- [ ] Configuration using Pydantic models
- [ ] Logging to stderr, output to stdout
- [ ] Click.testing.CliRunner tests
- [ ] Rich for progress/formatting (optional)
- [ ] --help text for all commands
- [ ] Error handling with meaningful messages

---

## References

- [Click Documentation](https://click.palletsprojects.com/)
- [Rich Documentation](https://rich.readthedocs.io/)
- [Python Repository Structure Standards](README.md)
- [Python Coding Standards](../../coding/python.md)

---

**Remember**: Separate CLI interface from business logic. Test both independently.
