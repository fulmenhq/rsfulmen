# Fulmen Repository Naming Standard

**Standard**: Consistent naming conventions for FulmenHQ repositories and binaries to ensure discoverability, avoid namespace conflicts, and maintain ecosystem cohesion.

**Version**: 1.0.0
**Status**: Stable
**Approval Authority**: Fulmen Maintainers

---

## 🎯 Purpose

Establish clear naming patterns for FulmenHQ repositories to enable:

- **Organizational Clarity**: Group related repositories alphabetically in GitHub org
- **Namespace Hygiene**: Avoid binary name conflicts with existing tools
- **Ecosystem Cohesion**: Clear branding for Fulmen-specific tools
- **Discoverability**: Easy identification of repository category and purpose

---

## 📋 Naming Patterns by Category

### Core Ecosystem Libraries

**Pattern**: `{langprefix}fulmen`

**Examples**:

- `gofulmen` - Go helper library
- `pyfulmen` - Python helper library
- `tsfulmen` - TypeScript helper library
- `rsfulmen` - Rust helper library (future)

**Binaries**: No binaries (library-only)

**Rules**:

- No category prefix (only one per language)
- Must use `fulmen` suffix
- Language prefix must be standard: `go`, `py`, `ts`, `rs`, `cs` (C#)

---

### Single Source of Truth (SSOT)

**Pattern**: `crucible` (canonical name, no prefix)

**Rationale**: Core schema/standard repository is uniquely named

**Binaries**: No binaries (content repository)

---

### General-Purpose CLI Tools

**Pattern**: `{toolname}` (no category prefix)

**Examples**:

- `goneat` - General-purpose CLI tool for Fulmen ecosystem

**Binary**: Same as repository name

**Rules**:

- Multi-purpose tools that can grow in scope
- Not ecosystem-specific (broadly useful)
- No category prefix needed
- Subject to conflict avoidance rules (see below)

---

### Microtools

**Pattern (Fulmen-Specific)**:

- Repository: `microtool-fulmen-{purpose}`
- Binary: `fulmen-{purpose}`

**Pattern (Generic/Reusable)**:

- Repository: `microtool-{purpose}`
- Binary: `{purpose}` (or same as repo)

**Examples**:

| Repository                  | Binary            | Rationale                                                                            |
| --------------------------- | ----------------- | ------------------------------------------------------------------------------------ |
| `microtool-fulmen-fixtures` | `fulmen-fixtures` | Fulmen ecosystem fixtures manager (ships with Crucible schemas, tightly coupled)     |
| `microtool-schemacheck`     | `schemacheck`     | Generic schema validator (YAML-aware, meta-validation, broad utility outside Fulmen) |
| `microtool-configsync`      | `configsync`      | Generic config synchronization tool (not Fulmen-specific)                            |
| `microtool-fulmen-metrics`  | `fulmen-metrics`  | Fulmen-specific Prometheus metrics generator                                         |

**Rule for `fulmen-` Prefix**:

**Include `fulmen` in the repository name IF and ONLY IF the tool is specific to the Fulmen ecosystem and has limited utility outside it.**

**Decision Criteria**:

- ✅ Use `fulmen-`: Ships with Fulmen-specific content, uses Crucible schemas exclusively, tightly coupled to Fulmen patterns
- ❌ Skip `fulmen-`: Solves general problem, useful to non-Fulmen users, no hard dependency on Fulmen ecosystem

---

### Workhorses

**Pattern**:

- Repository: `workhorse-fulmen-{purpose}` (Fulmen-specific) OR `workhorse-{purpose}` (generic)
- Binary: `fulmen-{purpose}` OR `{purpose}`

**Examples**:

- `workhorse-fulmen-gateway` → `fulmen-gateway`
- `workhorse-metrics` → `metrics` (if generic)

**Rules**: Same `fulmen-` inclusion criteria as microtools

---

### Services

**Pattern**:

- Repository: `service-fulmen-{purpose}` (Fulmen-specific) OR `service-{purpose}` (generic)
- Deployment: `fulmen-{purpose}` OR `{purpose}`

**Rules**: Same `fulmen-` inclusion criteria as microtools

---

### Templates (Forge)

**Pattern**: `forge-{category}-{name}`

**Examples**:

- `forge-workhorse-groningen` (Go workhorse template)
- `forge-codex-pulsar` (documentation site template)
- `forge-microtool-grinder` (Go microtool template)

**Binary**: N/A (templates don't produce binaries directly)

**Rules**:

- No `fulmen-` prefix in template names (templates are patterns, not implementations)
- Name uses category-specific pattern (horse breeds, instruments, etc.)

---

### Codex (Documentation Sites)

**Pattern**:

- Repository: `codex-fulmen-{name}` (Fulmen-specific) OR `codex-{name}` (generic)
- Deployment: `{name}.fulmenhq.dev` OR `{name}.yourdomain.com`

**Examples**:

- `codex-fulmen-docs` → `docs.fulmenhq.dev`
- `codex-technical-writing` → `writing.example.com` (generic guide)

---

## 🚫 Conflict Avoidance Rules

### Well-Known Binary Conflicts

**PROHIBITED** - Never use these names for binaries (non-exhaustive list):

**Development Tools**:

- `code`, `git`, `vim`, `emacs`, `docker`, `kubectl`, `npm`, `yarn`, `pip`, `cargo`, `make`, `cmake`

**System Utilities**:

- `ls`, `cat`, `grep`, `find`, `curl`, `wget`, `ssh`, `rsync`, `tar`, `zip`, `python`, `node`, `java`

**Popular CLI Tools**:

- `jq`, `yq`, `gh`, `az`, `aws`, `gcloud`, `terraform`, `ansible`, `helm`, `flux`

**Infrastructure/DevOps**:

- `jenkins`, `gitlab`, `circleci`, `travis`, `prometheus`, `grafana`, `vault`, `consul`

### Context-Based Evaluation

**Acceptable Collision IF**:

1. Usage context is clearly separated (e.g., DevOps/CI-only vs developer desktop)
2. Name is sufficiently specific to avoid confusion
3. Users would not reasonably run both binaries in same environment

**Examples**:

| Name              | Acceptable? | Rationale                                                |
| ----------------- | ----------- | -------------------------------------------------------- |
| `fulmen-fixtures` | ✅ Yes      | DevOps/CI-only, scoped to Fulmen ecosystem               |
| `fixtures`        | ⚠️ Risky    | Generic name, potential desktop use, could conflict      |
| `schemacheck`     | ✅ Yes      | Specific purpose, unlikely to conflict with common tools |
| `validate`        | ❌ No       | Too generic, many tools use this                         |
| `fulmen-validate` | ✅ Yes      | Scoped to Fulmen, clear purpose                          |

### Research Requirements

**Before proposing a new repository name, maintainers MUST**:

1. **Search GitHub**: Check for repositories with similar names
2. **Search Package Managers**:
   - npm registry: `npm search {name}`
   - PyPI: `pip search {name}` (or web search)
   - Go packages: `go search {name}` (or pkg.go.dev search)
   - Homebrew: `brew search {name}`
3. **Search Common Tools**: Google "{name} cli tool" to find existing utilities
4. **Check Docker Hub**: `docker search {name}` for container conflicts
5. **Test Command Availability**: On fresh Ubuntu/macOS, run `which {name}` and `command -v {name}`

**Documentation**: Proposer must document research findings in repository creation request.

---

## ✅ Approval Process

### Maintainer Approval Required

**All new repository names MUST be approved by Fulmen maintainers BEFORE creation.**

**Approval Process**:

1. **Propose Name**: Submit proposal via issue, PR, or Mattermost (#fulmen-governance)
2. **Include Research**: Document conflict avoidance research (see above)
3. **Justify Pattern**: Explain category, purpose, and `fulmen-` prefix decision
4. **Await Approval**: Maintainer review (typically 24-48 hours)
5. **Create Repository**: Only after explicit approval

**Approvers**:

- `@3leapsdave` (Primary approval authority)
- Designated ecosystem leads (when delegated)

### Emergency Exception

**Emergency repository creation MAY skip approval IF**:

1. **Critical Production Issue**: Security vulnerability, production outage, or data loss prevention
2. **Time-Sensitive**: Issue requires immediate fix (< 4 hours)
3. **Temporary Name**: Use `emergency-YYYYMMDD-{purpose}` pattern
4. **Retroactive Approval**: Must obtain approval within 24 hours and rename if needed

**Example**:

```
emergency-20251108-security-patch
  ↓ (after approval/rename)
microtool-fulmen-security-scanner
```

---

## 📊 Naming Decision Matrix

### Quick Reference

```
┌─────────────────────────────────────────────────────────────────────┐
│ Is this specific to Fulmen ecosystem?                               │
│   YES → Include 'fulmen-' in repo name                              │
│   NO  → Skip 'fulmen-', use generic name                            │
├─────────────────────────────────────────────────────────────────────┤
│ Category prefix (all repositories):                                 │
│   - Microtool:  microtool-{name}                                    │
│   - Workhorse:  workhorse-{name}                                    │
│   - Service:    service-{name}                                      │
│   - Codex:      codex-{name}                                        │
│   - Template:   forge-{category}-{pattern}                          │
│   - Library:    {lang}fulmen (no category prefix)                   │
│   - CLI:        {name} (no category prefix)                         │
├─────────────────────────────────────────────────────────────────────┤
│ Binary name collision check:                                        │
│   1. Search GitHub, package managers, Docker Hub                    │
│   2. Check well-known tools list (above)                            │
│   3. Test on Ubuntu/macOS: which {name}                             │
│   4. Document research findings                                     │
│   5. Get maintainer approval                                        │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 🎯 Examples

### Example 1: Fulmen-Specific Microtool

**Tool**: Fixture deployment and generation tool for Fulmen ecosystem

**Decision Process**:

1. Category: Microtool (narrow, single-purpose)
2. Fulmen-specific? YES (ships with Crucible schemas, Fulmen fixtures)
3. Repository: `microtool-fulmen-fixtures`
4. Binary: `fulmen-fixtures`
5. Conflict check: No conflicts found
6. Approval: ✅ Approved by @3leapsdave

### Example 2: Generic Microtool

**Tool**: YAML-aware JSON Schema validator with meta-validation

**Decision Process**:

1. Category: Microtool (narrow, single-purpose)
2. Fulmen-specific? NO (solves general schema validation problem)
3. Repository: `microtool-schemacheck`
4. Binary: `schemacheck`
5. Conflict check:
   - npm: No exact match
   - PyPI: No exact match
   - GitHub: Similar tools exist but different names
   - `which schemacheck`: Not found
6. Approval: ✅ Approved by @3leapsdave

### Example 3: Generic CLI Tool

**Tool**: General-purpose Fulmen ecosystem tooling

**Decision Process**:

1. Category: CLI (multi-purpose, can grow)
2. Fulmen-specific? YES (ecosystem tooling) but multi-purpose
3. Repository: `goneat` (no category prefix for general CLI)
4. Binary: `goneat`
5. Conflict check: No conflicts
6. Approval: ✅ Approved by @3leapsdave

---

## 🔄 Renaming Existing Repositories

**If an existing repository violates this standard**:

1. **Assess Impact**: Determine downstream dependencies, package manager registrations
2. **Propose Rename**: Submit rename proposal with migration plan
3. **Maintain Redirects**: GitHub auto-redirects old URLs (preserve for 6+ months)
4. **Update Documentation**: All references across ecosystem
5. **Notify Consumers**: Announce via Mattermost, README deprecation notices
6. **Gradual Migration**: Allow 1-2 release cycles for transition

**Grandfather Clause**: Existing repositories may continue using current names unless conflicts arise.

---

## 📚 Related Standards

- [Repository Categories](../taxonomy/repository-categories.yaml) - Category definitions
- [Microtool Standard](../architecture/fulmen-forge-microtool-standard.md) - Microtool constraints
- [Workhorse Standard](../architecture/fulmen-forge-workhorse-standard.md) - Workhorse patterns
- [CDRL Standard](../architecture/fulmen-template-cdrl-standard.md) - Template workflows

---

## 🔒 Governance

**Authority**: This standard is maintained by Fulmen ecosystem maintainers and enforced via:

1. **Pre-Creation Review**: All new repositories require maintainer approval
2. **Naming Compliance**: CI/CD checks validate naming patterns
3. **Conflict Monitoring**: Automated scans for binary name conflicts
4. **Documentation**: Repository creation checklist enforces standard

**Standard Owner**: @3leapsdave
**Last Updated**: 2025-11-08
**Review Cadence**: Quarterly or as ecosystem evolves
