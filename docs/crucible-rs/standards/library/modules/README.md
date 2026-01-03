# Fulmen Helper Library Module Standards

This directory contains standards for modules within Fulmen helper libraries (gofulmen, pyfulmen, tsfulmen, and future languages).

## Module Tier System

Modules are classified into three tiers:

- **Core**: Always present in all languages, zero optional dependencies
- **Common**: Default install, lightweight (may vary by language via tier_override)
- **Specialized**: Opt-in via extras/optional installs

**Definitive Registry**: `config/taxonomy/library/modules/v1.0.0/modules.yaml`

This registry is the canonical SSOT for all helper library modules. It is manually maintained but automatically validated against Crucible schemas, configs, and helper repo packages.

**Current Module Count** (v0.2.10):

- Core: 7 modules
- Common: 9 modules
- Specialized: 0 modules (fulencoding, fulpack planned for v0.2.11)

## For Helper Library Maintainers

Read the module registry to determine:

1. Which modules to implement
2. What tier each module belongs to
3. Whether your language has tier overrides
4. What dependencies are required
5. When each module was added

See the [Helper Library Standard](../../../architecture/fulmen-helper-library-standard.md) requirement #10 for compliance details.

## For Schema Cartographer

The module registry is validated by `scripts/validate-module-registry.ts` with 7 checks:

1. Orphan detection (schemas/configs without registry entry)
2. Dead entry detection (registry entries without evidence)
3. Evidence pointer validation (claimed paths exist)
4. Cross-language status validation (package fields present when status=available)
5. Schema compliance (structure and required fields)
6. Tier override rules (Core universality, Specialized graduation metrics)
7. Cross-reference validation (required_modules exist)

Run validation:

```bash
make validate-schemas  # Includes module registry validation
bun run scripts/validate-module-registry.ts  # Direct invocation
```

## Full Documentation

**Full tier standard documentation**: Planned for v0.2.11

The complete Extension Tier Standard will include:

- Decision tree for tier assignment
- Language-specific packaging patterns (Python extras, TypeScript optional deps, Go imports)
- Module gate checklist for adding new modules
- Graduation criteria for Specialized modules
- Sunset trigger policy

## Related

- **Registry**: `config/taxonomy/library/modules/v1.0.0/modules.yaml` (THE canonical source)
- **Schema**: `schemas/taxonomy/library/modules/v1.0.0/module-entry.schema.json`
- **Validation**: `scripts/validate-module-registry.ts`
- **Helper Library Standard**: `docs/architecture/fulmen-helper-library-standard.md`

---

_Module tier system created v0.2.10 by Schema Cartographer under supervision of @3leapsdave_
