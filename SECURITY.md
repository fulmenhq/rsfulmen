# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

**Do not report security vulnerabilities through public GitHub issues.**

Instead, please report them via one of these methods:

1. **Email**: Send details to `security@3leaps.net`
2. **GitHub Security Advisories**: Use the "Report a vulnerability" button in the Security tab

### What to Include

- Description of the vulnerability
- Steps to reproduce
- Affected versions
- Potential impact
- Any suggested fixes (optional)

### Response Timeline

- **Initial response**: Within 48 hours
- **Status update**: Within 7 days
- **Fix timeline**: Depends on severity (critical: ASAP, high: 14 days, medium: 30 days)

## Security Practices

### Dependency Management

rsfulmen follows these practices for supply chain security:

- **Minimal dependencies**: Feature flags allow consumers to include only needed functionality
- **Auditable**: Run `cargo tree` to inspect the full dependency graph
- **SBOM generation**: Compatible with `cargo sbom` and `cargo cyclonedx`
- **License compliance**: All dependencies are MIT, Apache-2.0, or compatible licenses
- **Vulnerability scanning**: Dependencies checked via `cargo audit`

### Embedded Data

All Crucible catalog data (country codes, exit codes, HTTP statuses, etc.) is:

- Embedded at compile time from Crucible SSOT
- Versioned and traceable via `.crucible/metadata/metadata.yaml`
- No runtime network calls required

### Code Security

- No `unsafe` code in core modules
- All inputs validated before use
- Pattern matching uses bounded execution (no ReDoS)

## Dependency Audit

To audit rsfulmen's dependencies:

```bash
# View dependency tree
cargo tree

# Check for known vulnerabilities
cargo audit

# Generate SBOM (requires cargo-sbom)
cargo sbom > sbom.json

# Generate CycloneDX SBOM (requires cargo-cyclonedx)
cargo cyclonedx
```

## Related Policies

- [FulmenHQ Security Policies](https://github.com/fulmenhq/crucible/blob/main/SECURITY.md)
- [3 Leaps OSS Policies](https://github.com/3leaps/oss-policies)
