# Design System Schemas

Schema definitions for Fulmen design systems. Provides a layered architecture separating semantic vocabulary (core) from implementation-specific patterns (TUI, web).

## Architecture

```
schemas/design/
├── core/v1.0.0/           # Semantic vocabulary (shared)
│   ├── semantic-colors.schema.json
│   ├── spacing-scale.schema.json
│   ├── typography-roles.schema.json
│   └── component-states.schema.json
├── tui/v1.0.0/            # Terminal UI implementation
│   ├── theme.schema.json          # Root composition
│   ├── color-palette.schema.json
│   ├── typography.schema.json
│   ├── layout.schema.json
│   └── component.schema.json
└── web/v1.0.0/            # (Future) Web implementation
```

## Core Schemas

Core schemas define **semantic vocabulary** - the language of design - without implementation details. Both TUI and web implementations reference these definitions.

| Schema             | Purpose                                                        |
| ------------------ | -------------------------------------------------------------- |
| `semantic-colors`  | Color roles: primary, secondary, success, warning, error, etc. |
| `spacing-scale`    | Spacing tokens: none, xs, sm, md, lg, xl, etc.                 |
| `typography-roles` | Text roles: display, headline, body, caption, code, etc.       |
| `component-states` | UI states: default, hover, focus, active, disabled, etc.       |

## TUI Schemas

Terminal UI schemas implement the core vocabulary for terminal environments with:

- **Color fallback chains**: truecolor → 256-color → 16-color → basic
- **Character set degradation**: unicode-full → unicode-basic → extended-ascii → ascii
- **Cell-based layouts**: dimensions in columns/rows
- **Box drawing**: configurable character sets (rounded, single, double, ascii)

| Schema          | Purpose                                         |
| --------------- | ----------------------------------------------- |
| `theme`         | Root composition - aggregates all TUI schemas   |
| `color-palette` | Terminal colors with capability-based fallbacks |
| `typography`    | Charset support, glyphs, CJK width handling     |
| `layout`        | Cell-based dimensions, responsive breakpoints   |
| `component`     | Widget patterns with state styling              |

## Usage

### Theme Files

Create theme YAML files that reference the TUI theme schema:

```yaml
$schema: https://schemas.fulmenhq.dev/crucible/design/tui/v1.0.0/theme.schema.json
version: "1.0.0"
schema_version: "1.0.0"

metadata:
  name: "Fulmen Dark"
  id: "fulmen-dark"
  description: "Default dark theme for Fulmen TUI applications"
  dark_mode: true

requirements:
  min_color_capability: "256"
  min_charset: "unicode-basic"

colors:
  version: "1.0.0"
  name: "fulmen-dark"
  semantic_colors:
    primary:
      role: "primary"
      foreground:
        truecolor: "#7AA2F7"
        ansi256: 111
        ansi16: "bright-blue"
  # ...
```

### Accessing via Helper Libraries

```go
// Go (gofulmen)
import "github.com/fulmenhq/crucible"

theme, _ := crucible.DesignRegistry.TUI().Theme("fulmen-dark")
primaryColor := theme.Colors.SemanticColors.Primary.Foreground.Truecolor
```

```typescript
// TypeScript (tsfulmen)
import { design } from "@fulmenhq/crucible";

const theme = design.tui().theme("fulmen-dark");
const primaryColor = theme.colors.semanticColors.primary.foreground.truecolor;
```

## Terminal Capability Detection

TUI themes support graceful degradation based on terminal capabilities:

| Capability    | Detection                     | Fallback Chain                     |
| ------------- | ----------------------------- | ---------------------------------- |
| **Color**     | `$COLORTERM`, `$TERM`         | truecolor → 256 → 16 → basic       |
| **Charset**   | `$LANG`, terminal query       | unicode-full → basic-latin → ascii |
| **Nerd Font** | User config, detection script | nerd-font → unicode → ascii        |

## WCAG Accessibility

Themes can declare WCAG conformance:

```yaml
accessibility:
  wcag_level: "AA"
  min_contrast_ratio: 4.5
  focus_visible_required: true
  color_blind_safe: true
```

The `wcag_contrast_ratio` field on semantic colors enables automated accessibility checking.

## Examples

Reference themes are available in `examples/design/tui/v1.0.0/themes/`:

- `dark.yaml` - Default dark theme with full capability (WCAG AA)
- `light.yaml` - Light mode for daytime use (WCAG AA)
- `high-contrast.yaml` - WCAG AAA compliant high contrast theme

## Roadmap

### v1.0.0 (Current)

- Core semantic vocabulary
- TUI implementation schemas
- Reference dark and high-contrast themes

### v1.1.0 (Planned)

- Animation/transition tokens
- Sound/haptic feedback schemas
- Theme inheritance/composition

### Future

- Web implementation schemas (`design/web/`)
- Design token export (Figma, Style Dictionary)
- Theme marketplace/registry

## Related Documentation

- [Fulmen Ecosystem Guide](../../docs/architecture/fulmen-ecosystem-guide.md)
- [Schema Normalization Standard](../../docs/standards/schema-normalization.md)
- [Terminal Catalog](../terminal/v1.0.0/)
