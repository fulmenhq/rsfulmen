# Classifiers Catalog

**Canonical URL**: `https://crucible.3leaps.dev/catalog/classifiers`

Index of Crucible classifier dimensions and their canonical sources.

## How To Use This Catalog

- Use the narrative standards to decide what a value means.
- Use the dimension definition JSON for machine validation, UI dropdowns, and automation.
- Vendor `config/classifiers/` alongside `schemas/classifiers/` when you need offline or pinned behavior.

Missing classification is a policy error. Use explicit `unknown` until classification is complete.

## Dimensions

| Dimension                   | Key                   | Standard                                               | Definition                                                         |
| --------------------------- | --------------------- | ------------------------------------------------------ | ------------------------------------------------------------------ |
| Data Sensitivity            | `sensitivity`         | `docs/standards/data-sensitivity-classification.md`    | `config/classifiers/dimensions/sensitivity.dimension.json`         |
| Volatility & Update Cadence | `volatility`          | `docs/standards/volatility-classification.md`          | `config/classifiers/dimensions/volatility.dimension.json`          |
| Access Tier                 | `access-tier`         | `docs/standards/access-tier-classification.md`         | `config/classifiers/dimensions/access-tier.dimension.json`         |
| Retention & Lifecycle       | `retention-lifecycle` | `docs/standards/retention-lifecycle-classification.md` | `config/classifiers/dimensions/retention-lifecycle.dimension.json` |
| Schema Stability            | `schema-stability`    | `docs/standards/schema-stability-classification.md`    | `config/classifiers/dimensions/schema-stability.dimension.json`    |
| Volume Tier                 | `volume-tier`         | `docs/standards/volume-tier-classification.md`         | `config/classifiers/dimensions/volume-tier.dimension.json`         |
| Velocity Mode               | `velocity-mode`       | `docs/standards/velocity-mode-classification.md`       | `config/classifiers/dimensions/velocity-mode.dimension.json`       |

## Framework Overview

See `docs/standards/classifiers-framework.md` for how the pieces fit together.
