# Fulmen Web Styling – Implementation Guide

**Version:** v1.0.0
**Date:** 2025-11-03
**Status:** Active Standard
**Related Standards:** [Foundry Library](../standards/library/foundry/README.md), [Forge Codex Standard](https://github.com/fulmenhq/forge-codex-pulsar)

---

## Introduction

This guide establishes standards for web styling in Fulmen templates (e.g., Forge Codex Pulsar), emphasizing declarative definitions via schemas, rigorous validation, and flexible CSS patterns. Fulmen prioritizes tooling and templates over prescriptive design systems—users (CRDL instances) customize freely, but implementations must adhere to core principles: simplicity, accessibility (WCAG 2.1 AA), and OSS compliance.

**Scope**: Applies to web forges (Astro/Starlight preferred); covers branding/styling schemas, CSS organization, asset licensing, and enforcement. Not a visual design spec (no fixed palettes)—focus on _how_ to define/implement for consistency across Fulmen apps.

**Principles** (from Technical Manifesto):

- **Embrace Simplicity**: Declarative configs over complex overrides
- **Persnickety About Code**: Validate at build (tsfulmen/ajv, a11y checks)
- **Build in AAA**: Enforce contrast/line-height via schemas/tools
- **DRY**: Reuse schemas/CSS vars; OSS assets to avoid licensing pitfalls

**Non-Goals**: Mandate specific colors/fonts (user-defined); enforce frameworks beyond recommendations.

---

## Schema Standards

Define styling via Layer 0 schemas in Crucible (retrieved via tsfulmen), validated at build-time. Two core schemas establish the SSOT for web template customization:

### Branding Schema (`web/branding/v1.0.0`)

**Schema Location:** `schemas/web/branding/v1.0.0/site-branding.schema.json`
**Default Config:** `config/web/branding/site-branding.yaml`

**Purpose**: Site identity (name, logo, icons) with presentation variants.

**Key Fields**:

- `name` (string, 1-50 chars) – Site or application name
- `tagline` (string, ≤120 chars) – Short descriptive tagline
- `logo` (object) – Primary logo configuration:
  - `src` (uri-reference) – Path/URL to logo image (SVG/PNG preferred, repo-local in public/)
  - `alt` (string, required) – Accessible alt text for the logo
  - `width`/`height` (number, 32-256px) – Preferred display dimensions
- `favicon` (uri-reference) – Path to favicon (ICO/SVG, 16x16+ pixels)
- `appIcon` (uri-reference) – Path to application icon for PWA/manifest (PNG/SVG, 192x192+)
- `logoPresentation` (enum, required) – Layout variant:
  - `logo_only` – Logo includes text (no separate name/tagline)
  - `logo_left_text_right` – Horizontal alignment with logo on left
  - `logo_top_text_bottom` – Vertical stack with logo on top
  - `text_only` – No logo, just name/tagline
  - `gridbox_logo_text` – Flex/grid centered layout

**Validation**:

- tsfulmen/ajv in Makefile (`make validate:branding`)
- Required `alt` for accessibility
- uri-reference format for paths (repo-local preferred)

**Example Usage**:

```yaml
name: Pulsar Codex
tagline: Production-grade documentation template
logo:
  src: /logo.svg
  alt: Pulsar Codex Logo
  width: 120
  height: 40
favicon: /favicon.svg
logoPresentation: logo_left_text_right
```

### Styling Schema (`web/styling/v1.0.0`)

**Schema Location:** `schemas/web/styling/v1.0.0/site-styling.schema.json`
**Default Config:** `config/web/styling/site-styling.yaml`

**Purpose**: Palettes, typography, icons with accessibility constraints.

**Key Fields**:

1. **themes** (required):
   - `light`/`dark` palettes with hex colors:
     - **Required core colors** (7): `background`, `foreground`, `accent`, `success`, `failure`, `warning`, `critical`
     - **Optional semantic colors** (3): `info`, `note`, `caution` (for callout boxes)
     - **Optional categorical array**: 5-12 hex colors for charts/data visualization (accessible sequences)

2. **typography** (required):
   - `fonts` (object):
     - `body`/`code`/`heading` (required for body/code):
       - `family` – CSS font-family string (e.g., 'Inter, sans-serif')
       - `src` – Path/URL to font file (WOFF2 in public/fonts/, or external URL)
       - `weights` – Array of supported weights (e.g., [400, 700])
       - `size.base` – Base font size (px/rem/em)
   - `embed` (boolean, default: true) – Embed fonts at build-time (true: WOFF2 from src; false: URL/CDN)
   - `lineHeight` (number, 1.2-2.0, default: 1.5) – Base line-height

3. **icons** (required):
   - `registry` (enum: mdi/heroicons/lucide/tabler) – Icon set registry
   - `prefix` – CSS class prefix
   - `size` (16-48px) – Default icon size
   - `custom` (array) – Custom icons with name/src/size
   - `palette` – Icon fill colors for light/dark modes (hex)

4. **a11y** (optional, recommended):
   - `minContrast` (object):
     - `normal` (default: 4.5) – Min ratio for normal text (AA: 4.5:1)
     - `large` (default: 3.0) – Min ratio for large text (AA: 3:1)
     - `graphics` (default: 3.0) – Min ratio for UI components/graphics
   - `level` (enum: AA/AAA, default: AA) – WCAG conformance level

**Validation**:

- tsfulmen/ajv in Makefile (`make validate:styling`)
- Hex pattern validation for all colors
- Contrast checks (foreground/background ≥ minContrast.normal) via analyzer
- Font src paths must exist when `embed: true`
- Unique colors in categorical arrays

**Example Usage**:

```yaml
themes:
  light:
    background: "#ffffff"
    foreground: "#111827"
    accent: "#3b82f6"
    success: "#10b981"
    failure: "#ef4444"
    warning: "#f59e0b"
    critical: "#8b5cf6"
typography:
  fonts:
    body:
      family: "Inter, sans-serif"
      src: /fonts/inter.woff2
      weights: [400, 700]
      size:
        base: 16px
  embed: true
icons:
  registry: mdi
  size: 24
a11y:
  minContrast:
    normal: 4.5
    large: 3.0
  level: AA
```

**Sync/Usage**: Templates stage schemas in `config/schemas/` (synced from Crucible via helper library build scripts); extend via `codex.config.json`. Defaults provided in tsfulmen Layer 0 (minimal OSS).

---

## Validation Expectations

Enforce via build pipeline (Makefile/goneat):

### Build-Time Validation

- **Schema Compliance**: `make validate:styling` – ajv validates against schemas
- **Contrast Checks**: Compute color contrast ratios (e.g., accent/foreground ≥4.5:1); fail PRs if violations
- **A11y Integration**: Pair with a11y analyzer – pre-validate palette pairs; runtime checks for components
- **Asset Checks**: Verify OSS licenses (script scans public/fonts/icons/ for OFL/MIT); warn on proprietary
- **CRDL Hooks**: Refit validation (`make refit:check`) ensures custom configs pass schemas

### CI/CD Integration

- GitHub Actions job (`validate-styling`)
- Generate artifacts for reports (violations, contrast ratios)
- Block merges on validation failures

### Enforcement Standards

- **100% schema compliance** – All configs must validate
- **≥4.5:1 AA contrast ratios** – Normal text against background
- **OSS-only assets** – Fonts and icons must use approved licenses
- **Actionable feedback** – Tools provide fixes (e.g., "Update --color-accent to #007bff for 4.6:1 contrast")

---

## CSS Patterns

Fulmen templates recommend but do not mandate Tailwind CSS (per Forge Codex Standard technology criteria)—preferred for utility-first, responsive design. Alternatives allowed if meeting performance and accessibility requirements (e.g., vanilla CSS vars, Sass modules).

### Recommended Pattern: Tailwind + CSS Variables

**Tailwind CSS (Preferred)**: v3.4+ with Fulmen preset

- **Configuration**: `tailwind.config.fulmen.cjs` extends theme from tsfulmen config
  - Colors from `styling.themes`
  - Font families from `typography`
- **Pros**:
  - Utility classes for rapid prototyping
  - Built-in dark mode (class strategy)
  - A11y plugins (tailwindcss-contrast)
- **Usage**:
  - Arbitrary values for custom colors: `bg-[var(--color-note)]`
  - Responsive/dark variants: `dark:bg-[var(--color-bg-dark)]`
- **Performance**: Purge unused classes (Tailwind JIT); target <50KB CSS gzipped

### CSS Variables Mandate

**Always use CSS variables for themes** – Enables runtime theme switching and centralizes color management:

```css
:root {
  --color-bg: #ffffff;
  --color-fg: #111827;
  --color-accent: #3b82f6;
  /* ... */
}

[data-theme="dark"] {
  --color-bg: #111827;
  --color-fg: #f9fafb;
  --color-accent: #60a5fa;
  /* ... */
}
```

**Validation**: Ensure CSS vars match schema keys (build script checks `--color-*` against `styling.themes` properties)

### File Structure

```
styles/
├── global.css          # CSS vars, resets, base styles
├── components.css      # Scoped component styles
└── themes/
    ├── light.css       # Light theme overrides (auto-generated)
    └── dark.css        # Dark theme overrides (auto-generated)
```

### Alternative Patterns

- **Vanilla CSS**: Manual vars; no utilities (higher maintenance)
- **Sass/Less**: Modules for nesting; compile to CSS vars
- **Validation Requirement**: All alternatives must generate CSS vars matching schema structure

### Note Boxes & Semantic Controls

**Convention**: Use semantic classes (`.info`, `.note`, `.caution`, `.warning`) mapping to palette:

```css
.note {
  background: var(--color-note);
  border-left: 4px solid var(--color-accent);
  color: var(--color-fg);
}
```

**Mapping Guidance**:

- **Preferred**: Direct mapping to dedicated palette colors (`note` → `--color-note`)
- **Fallback**: Use core colors if semantic colors undefined (`note` → `--color-accent`)
  - Document in templates: "Prefer dedicated semantic colors for clarity"
- **Controls**: Buttons/alerts inherit semantic colors (`.btn-success { bg: var(--color-success); }`)

**A11y Requirements**:

- Ensure ≥4.5:1 contrast ratio for text/background
- Provide focus states: `outline: 2px solid var(--color-accent)`
- Schema enforces contrast validation at build time

---

## OSS Assets Mandate

For Fulmen implementations (template clones), **only OSS-licensed assets are permitted**:

### Fonts

**Licenses**: OFL (Open Font License), MIT, Apache 2.0
**Examples**: Inter (OFL), JetBrains Mono (OFL), Roboto (Apache), Source Sans (OFL)

**Requirements**:

- Embed WOFF2 at build time (subset for performance)
- No proprietary fonts (e.g., Proxima Nova, Helvetica Neue)
- Include LICENSE files in `public/fonts/`

**Validation**:

- Build script scans `public/fonts/` for license files
- Fail build if non-OSS or missing license
- Log warning for external CDN fonts (recommend embedding)

**Sourcing**:

- Prefer local WOFF2 files in `public/fonts/`
- Google Fonts URLs allowed if `embed: true` (downloads at build time)

### Icons

**Licenses**: MIT, Apache 2.0, ISC
**Examples**: MDI (MIT), Heroicons (MIT), Lucide (ISC), Tabler Icons (MIT)

**Requirements**:

- SVGs in `public/icons/`
- Enum registries in schema enforce approved icon sets
- Custom icons must include license metadata

**Validation**:

- Check custom icon sources for MIT/Apache/ISC licenses
- Sprite bundling for performance
- Document license in README or LICENSE file

**Avoid**: Licensed icon packs (e.g., Font Awesome Pro) – too complex for CRDL (cloners must replace)

### Rationale

- **Prevents licensing violations** in forks and CRDL instances
- **Ensures static/offline capability** (no external CDNs for core assets)
- **Simplifies CRDL refit** – cloners can extend without legal concerns

**Documentation**: Template README must state: "Replace assets with OSS alternatives during CRDL refit. See LICENSE files in public/fonts/ and public/icons/ for approved licenses."

---

## Enforcement & Customization

### Build Tooling

**goneat Pre-commit Hooks**:

- Validate schemas (`make validate:branding`, `make validate:styling`)
- Check asset licenses (`make validate:assets`)
- Compute contrast ratios (`make validate:a11y`)

**Makefile Targets**:

- `make validate:styling` – Schema + contrast validation
- `make build:css` – Generate CSS vars from config
- `make sync:schemas` – Pull latest schemas from Crucible

### CRDL Refit Workflow

1. **Copy Example Configs**: `config.examples/` → project root
2. **Update Config**: Edit `codex.config.json` with custom branding/styling
3. **Add Assets**: Place logo/favicon/fonts in `public/`
4. **Override Tailwind**: Extend `tailwind.config.fulmen.cjs` if needed
5. **Validate**: Run `make refit:check` to ensure schema compliance
6. **Build**: `make build` generates CSS vars and validates a11y

### Committee Exceptions

**When to request exception**:

- Alternative CSS framework (e.g., Sass if performance justified)
- Non-standard icon registry (must be OSS)
- Custom validation requirements

**Process**:

- Submit ADR (Architecture Decision Record)
- Justify with performance/accessibility data
- Document in `docs/decisions/`

### Guidance Documentation

**Required in Template**:

- `docs/standards/web-styling.md` – Implementation examples
- Tailwind config generation from schema
- Troubleshooting guide (low contrast fixes, asset validation)
- CRDL refit checklist

---

## Schema References

| Schema        | Location                                                | Config                                   |
| ------------- | ------------------------------------------------------- | ---------------------------------------- |
| Site Branding | `schemas/web/branding/v1.0.0/site-branding.schema.json` | `config/web/branding/site-branding.yaml` |
| Site Styling  | `schemas/web/styling/v1.0.0/site-styling.schema.json`   | `config/web/styling/site-styling.yaml`   |

**Helper Library Support**:

- **tsfulmen**: Schema validation via ajv (`tsfulmen.schemas.siteBranding.validate(config)`)
- **pyfulmen**: YAML loading + goneat harness integration
- **gofulmen**: Future support for Go-based web templates

---

## Related Standards

- [Foundry Library README](../standards/library/foundry/README.md) – Core library patterns
- [Forge Codex Standard](https://github.com/fulmenhq/forge-codex-pulsar) – Web template requirements
- [Technical Manifesto](../architecture/technical-manifesto.md) – Core principles

---

**Status**: Active Standard
**Next Review**: 2025-12-01
**Maintainer**: Fulmen Architecture Team
