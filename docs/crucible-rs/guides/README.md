# Crucible Guides

Practical guides for integrating Crucible into your projects, bootstrapping tools, and working with the Fulmen ecosystem.

## Available Guides

### Integration & Setup

#### [Agentic Interface Adoption Guide](agentic-interface-adoption.md)

How to adopt Crucible's v0.3.0 agentic role catalog and attribution baseline.

**Topics**:

- Role catalog migration (YAML roles)
- Commit attribution format adoption
- AGENTS.md updates
- Role selection by repository type
- Validation and CI integration

**When to use**: Migrating from inline prompts to schema-validated roles, adopting attribution baseline.

#### [Integration Guide](integration-guide.md)

Complete guide to integrating Crucible into your project.

**Topics**:

- Production integration (as a dependency)
- Development integration (cloned SSOT)
- Go module usage (`go get github.com/fulmenhq/crucible`)
- TypeScript package usage (`@fulmenhq/crucible`)
- Python package usage (`fulmenhq-crucible`)
- Sync strategies for downstream consumers

**When to use**: First-time integration or switching integration patterns.

#### [Bootstrap goneat](bootstrap-goneat.md)

Step-by-step guide to installing and configuring goneat (format/lint/assessment tool).

**Topics**:

- Installing goneat from releases or building from source
- Configuration file setup
- Hook integration (pre-commit, pre-push)
- Tool bootstrapping (external dependencies)
- Troubleshooting common setup issues

**When to use**: Setting up goneat in a new repository or fixing goneat installation issues.

### Library Development

#### [Fulmen Library Bootstrap Guide](fulmen-library-bootstrap-guide.md)

Comprehensive guide to creating new Fulmen helper libraries following ecosystem standards.

**Topics**:

- Repository structure and naming conventions
- Package configuration (Go modules, package.json, pyproject.toml)
- Crucible integration and SSOT sync setup
- Module standards implementation
- Testing and quality gates
- Publishing and version management

**When to use**: Creating a new helper library (gofulmen, pyfulmen, tsfulmen) or extending the Fulmen ecosystem.

### Testing

Testing guides serve as **compliance routing documents** - they direct you to normative standards and provide practical checklists for implementation and review.

#### [HTTP Server Patterns](testing/http-server-patterns.md)

Patterns for HTTP server and fixture implementations with compliance routing.

**Topics**:

- Compliance requirements (which standards apply)
- Go HTTP handler anti-patterns (json.Encoder, closeBody, context cancellation)
- Correlation ID propagation
- Streaming correctness and determinism
- Fixture author conformance checklist

**When to use**: Building HTTP servers, fixtures, or reviewing server-side code.

#### [HTTP Client Patterns](testing/http-client-patterns.md)

Patterns for HTTP client library and middleware testing.

**Topics**:

- Timeout testing (connect vs header vs body tiers)
- Redirect handling and loop detection
- Retry and backoff behavior
- Auth failure handling (401 vs 403)
- Using rampart/gauntlet fixtures for integration tests

**When to use**: Testing HTTP clients, middleware, or proxies.

See [Testing Guides Index](testing/README.md) for the full testing guides family.

## Quick Reference

### By Task

- **Adopting agentic interface**: Start with [Agentic Interface Adoption Guide](agentic-interface-adoption.md)
- **Integrating Crucible**: Start with [Integration Guide](integration-guide.md)
- **Setting up formatting/linting**: See [Bootstrap goneat](bootstrap-goneat.md)
- **Creating a new library**: Follow [Fulmen Library Bootstrap Guide](fulmen-library-bootstrap-guide.md)
- **Building HTTP servers/fixtures**: See [HTTP Server Patterns](testing/http-server-patterns.md)
- **Testing HTTP clients**: See [HTTP Client Patterns](testing/http-client-patterns.md)

### By Role

**Application Developer**:

1. [Integration Guide](integration-guide.md) - Add Crucible to your project
2. [Bootstrap goneat](bootstrap-goneat.md) - Set up quality tooling
3. [Agentic Interface Adoption Guide](agentic-interface-adoption.md) - Adopt role catalog and attribution

**Library Maintainer**:

1. [Fulmen Library Bootstrap Guide](fulmen-library-bootstrap-guide.md) - Create conformant libraries
2. [Integration Guide](integration-guide.md) - Sync strategies for helper libraries
3. [Agentic Interface Adoption Guide](agentic-interface-adoption.md) - Adopt role catalog and attribution

**Crucible Maintainer**:

- All guides + [../sop/README.md](../sop/README.md) for operational procedures

## Related Documentation

### Architecture

- [Fulmen Ecosystem Guide](../architecture/fulmen-ecosystem-guide.md) - Understand the layer cake
- [Sync Model](../architecture/sync-model.md) - How SSOT propagation works
- [Library Ecosystem](../architecture/library-ecosystem.md) - Multi-language support architecture

### Standards

- [Helper Library Standard](../architecture/fulmen-helper-library-standard.md) - Requirements for Fulmen libraries
- [Makefile Standard](../standards/makefile-standard.md) - Standard make targets
- [Repository Structure](../standards/repository-structure/README.md) - Standard layouts by language

### Operations

- [Repository Operations SOP](../sop/repository-operations-sop.md) - Day-to-day operations
- [CI/CD Operations](../sop/cicd-operations.md) - GitHub Actions setup
- [Release Checklist](../ops/repository/release-checklist.md) - Pre-release quality gates

## Contributing

Found an issue or have a suggestion?

- **Report issues**: https://github.com/fulmenhq/crucible/issues
- **Submit improvements**: Create a PR updating guides in `docs/guides/`
- **Ask questions**: https://github.com/fulmenhq/crucible/discussions

---

**Note**: These guides are maintained in the Crucible SSOT and automatically synced to language wrappers (`lang/*/docs/guides/`).
