# rsfulmen Operations Guide

This document covers day-to-day development workflow, release processes, and tooling for rsfulmen.

## Development Workflow

### Primary Makefile Targets

| Target | Description |
|--------|-------------|
| `make bootstrap` | Install dependencies and external tools |
| `make sync` | Sync Crucible SSOT assets |
| `make build` | Build library |
| `make test` | Run all tests |
| `make lint` | Run clippy linter |
| `make fmt` | Format code with rustfmt |
| `make check-all` | Full quality gate (fmt-check + lint + test) |
| `make doc` | Generate rustdoc documentation |

### Daily Workflow

```bash
# Before starting work
git pull
make sync              # Update Crucible assets if needed

# During development
cargo build            # Quick build
cargo test             # Run tests
cargo clippy           # Check for issues

# Before committing
make check-all         # Full quality gate
```

### Pre-commit/Pre-push Hooks

rsfulmen uses goneat hooks for automated quality checks:

```bash
make precommit   # Fast checks (format, lint, security critical)
make prepush     # Comprehensive checks (all above + tests)
```

Hooks are auto-installed on first `make build`.

## Release Process

### Versioning Strategy

rsfulmen uses [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking API changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

Version is tracked in:
- `VERSION` file (source of truth)
- `Cargo.toml` (auto-propagated)

### Release Checklist

See [RELEASE_CHECKLIST.md](../../RELEASE_CHECKLIST.md) for the complete checklist.

**Summary:**

1. Ensure all changes are committed and pushed
2. Update CHANGELOG.md
3. Bump version: `make version-set VERSION=x.y.z`
4. Run release checks: `make release-check`
5. Create signed tag: `make release-tag`
6. Push tag: `git push origin vx.y.z`
7. Publish to crates.io: `cargo publish`

### Version Bump Commands

```bash
make version-bump-patch   # 0.1.0 -> 0.1.1
make version-bump-minor   # 0.1.0 -> 0.2.0
make version-bump-major   # 0.1.0 -> 1.0.0
make version-set VERSION=1.2.3  # Explicit version
```

## Testing Strategy

### Coverage Targets

Coverage thresholds are lifecycle-phase dependent:

| Phase | Minimum Coverage |
|-------|-----------------|
| experimental | 0% |
| alpha | 30% |
| beta | 60% |
| rc | 70% |
| ga | 75% |
| lts | 80% |

rsfulmen is currently in **alpha** phase (30% minimum).

### Running Tests

```bash
# Basic test run
make test

# With coverage report
make test-cov

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture
```

### Test Categories

- **Unit tests**: In `src/` files as `#[cfg(test)]` modules
- **Integration tests**: In `tests/` directory
- **Doc tests**: In rustdoc comments

## Tooling Reference

### Bootstrap

```bash
make bootstrap          # Install all tools
make bootstrap-force    # Force reinstall
make tools              # Verify tool availability
```

**Installed tools:**
- `sfetch` - Secure file fetcher (trust anchor)
- `goneat` - FulmenHQ development tooling
- `cargo-tarpaulin` - Code coverage
- `cargo-audit` - Security vulnerability scanner
- `cargo-deny` - License/security policy checker

### Goneat Usage

```bash
# Sync Crucible assets
goneat ssot sync --force-remote

# Run assessments
goneat assess --categories format,lint,security

# Check tool versions
goneat doctor tools --scope foundation
```

### Rust Toolchain

**MSRV**: 1.83+

```bash
# Check MSRV compatibility
make msrv-check

# Update toolchain
rustup update
```

## Security Expectations

### Dependency Scanning

```bash
# Audit for known vulnerabilities
cargo audit

# Check licenses and security policy
cargo deny check
```

### Vulnerability Reporting

Report security vulnerabilities to: security@fulmenhq.dev

Do NOT open public issues for security vulnerabilities.

## Community & Support

- **Issues**: [GitHub Issues](https://github.com/fulmenhq/rsfulmen/issues)
- **Discussions**: [GitHub Discussions](https://github.com/fulmenhq/rsfulmen/discussions)
- **Slack**: #rsfulmen (FulmenHQ workspace)

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make changes with tests
4. Ensure `make check-all` passes
5. Submit pull request

See [README.md](../../README.md#contributing) for details.
