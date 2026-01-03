---
title: "Fulmen Technical Manifesto"
description: "Principles, architecture patterns, and quality bars that define the Fulmen ecosystem"
author: "Schema Cartographer"
date: "2025-10-10"
last_updated: "2025-10-10"
status: "draft"
tags: ["fulmen", "architecture", "manifesto", "2025.10.2"]
---

# Fulmen Technical Manifesto

## Why This Manifesto Exists

Fulmen’s promise: **Thrive on Scale**—start fast with systems engineered for enterprise reliability and growth. This manifesto codifies our commitments across the evolved ecosystem: from CRDL-optimized forges to Crucible SSOT, language foundations, tools like goneat/brooklyn-mcp, and AI-human collaboration (via MAINTAINERS.md roles). Refined over eight months from template frustration to a flywheel preventing repeated pains (e.g., logging, schemas), it guides building without silos.

### Lightning Strikes with Purpose

The Fulmen ecosystem enables a balance: rapid starts with built-in scalability. This means delivering complete, operational systems from the outset, rather than assembling basics over time.

Through the layer cake (Crucible SSOT → Helper Libraries → Templates → DX Tools → Apps/Services), Fulmen refines development practices via rigorous validation, supporting a cycle: refine standards → embed in libraries → scaffold projects → automate governance → deploy at scale.

The CRDL process (Clone → Degit → Refit → Launch) applies particularly to templates (Layer 2), providing production-ready starters that teams adapt efficiently.

## Core Fulmen Principles

These principles guide development across all Fulmen layers, ensuring consistent, scalable systems. They build on the ecosystem's foundation in Crucible (Layer 0) and extend to production apps (Layer 4).

1. **Start Fast, Thrive on Scale**  
   Launch quickly with systems designed for growth. Fulmen provides operational foundations—authentication, observability, scaling patterns—from the start, allowing customization without rebuilding basics.  
   _Tie to Layers_: Layer 2 templates bootstrap with Layer 0 schemas and Layer 1 libraries for immediate interoperability.  
   _Example (TypeScript)_: Import from `@fulmenhq/crucible` for pre-validated schemas in a new service.

2. **Be Persnickety About Code**  
   Enforce strict quality at every step: linting, formatting, types, and tests before commits. This maintains reliability amid high-velocity changes.  
   _Tie to Layers_: Layer 3 tools like goneat automate via Crucible standards (`docs/standards/devsecops/pre-commit-processes.md`).  
   _Example_: Pre-commit hooks run `bun test` and Biome checks on modified files.

3. **Be Ruthless About Type Safety**  
   Leverage the strongest typing available, with runtime validation as backup. This catches issues early and ensures consistency.  
   _Tie to Layers_: Layer 0 schemas generate types; Layer 1 libraries enforce them.  
   _Example (TypeScript with Zod)_:

   ```typescript
   import { z } from "zod";
   const OrderSchema = z.object({
     id: z.string().uuid(),
     total: z.number().positive(),
   });
   type Order = z.infer<typeof OrderSchema>;
   ```

4. **Support Observability from Day One**  
   Integrate logging, metrics, and tracing inherently. Measurable systems scale predictably.  
   _Tie to Layers_: Layer 0 schemas define events; Layer 1 libs wire them.  
   _Example_: Use `@fulmenhq/crucible` for structured logs in services.

5. **Develop Schemas First**  
   Define data contracts upfront for cross-language consistency and validation.  
   _Tie to Layers_: Core to Crucible (Layer 0); consumed by all above.  
   _Example (JSON Schema)_: Versioned in `schemas/` for API responses.

6. **Build in AAA and Zero-Trust**  
   Include authentication, authorization, and auditing with zero-trust defaults. Security is foundational, not additive.  
   _Tie to Layers_: Layer 0 policies; Layer 3 fulward enforces.  
   _Example_: Middleware verifies tokens using OIDC schemas from Crucible.

7. **Follow DRY with Purpose**  
   Reuse patterns judiciously—abstract after repetition (Rule of Three). Avoid premature complexity.  
   _Tie to Layers_: Layer 1 libraries capture shared modules from Layer 0 standards.  
   _Example_: Extract common error handling into a reusable hook after 3 uses.

8. **Embrace Simplicity**  
   Opt for clear, maintainable solutions over elaborate designs. Simplicity scales.  
   _Tie to Layers_: Reflected in all standards; enforced by Layer 3 tools.  
   _Example_: Direct functions over strategy patterns unless needed.

## Architecture Tenets

These tenets operationalize the principles across layers:

| Layer                 | Tenet                        | Implementation                                                                                                              |
| --------------------- | ---------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| **Crucible (0)**      | SSOT over synchronization    | Author schemas/standards once; sync to wrappers via `bun run sync:to-lang`; consumers pull tagged releases for consistency. |
| **Libraries (1)**     | Idiomatic by default         | `*fulmen` packages wrap Layer 0 assets in native APIs, with coverage targets from `config/library/module-manifest.yaml`.    |
| **Templates (2)**     | Production from commit zero  | Fulmens include AAA, CI/CD, telemetry (Layers 0-1), and docs; CRDL enables quick adaptation.                                |
| **DX Tools (3)**      | Automated governance         | goneat/fulward enforce principles (e.g., type safety, observability) via hooks and policies.                                |
| **Apps/Services (4)** | Observability and resilience | Integrate Layer 0 schemas with brooklyn-mcp for workflows; sumpter for data ELT.                                            |

## Technical Pillars in Practice

### Schemas & Contracts

- CalVer releases ensure compatibility (e.g., `2025.10.x`); additive changes first, with deprecation for removals.

- `scripts/validate-schemas.ts` verifies integrity pre-build/test, aligning with "Schemas First" principle.

### Code Quality & Testing

- Coverage targets per `config/library/module-manifest.yaml` (e.g., Go ≥95%, TS ≥85%); supports "Persnickety About Code."

- Embed unit/integration/E2E tests in templates (Layer 2); reuse CI via Layer 3 tools.

- Fulward/goneat block commits on failures, enforcing quality gates.

### Security & Compliance

- Zero-trust defaults: Layer 0 patterns for config, secrets, RBAC/audits; AAA scaffolding in templates.

- Aligns with "Build in AAA" principle; fulward (Layer 3) classifies risks per `docs/standards/repository-safety-framework.md`.

### Observability & Operations

- Layer 0 schemas (`config/observability/`) consumed by Layers 1-4; supports "Observability from Day One."

- Releases generate notes, verify sync; Layer 4 (brooklyn-mcp) monitors via shared metrics.

## How to Apply the Manifesto

1. **Start with Standards**: Reference Crucible (Layer 0) and principles like "Schemas First" before coding; propose gaps upstream.

2. **Automate Everything**: Use Layer 3 tools (goneat, fulward) for compliance; script repetitive tasks.

3. **Publish the Blueprint**: Update changelogs, notes, and standards for new features, per "Persnickety About Code."

4. **Refit Responsibly**: Customize templates (Layer 2) while preserving AAA/telemetry; document divergences.

5. **Partner with AI Agents**: Supply context (.plans/, docs) for agents like Schema Cartographer; review outputs rigorously.

6. **Embrace Simplicity & DRY**: Abstract only after patterns repeat; prioritize maintainable code across layers.

7. **Measure from the Start**: Wire observability early; use Layer 0 schemas for consistent telemetry.

8. **Secure by Default**: Implement zero-trust/AAA from init; validate with Layer 3 policies.

## The Road Ahead

- **Layer 0 (Crucible)**: Integrate principles into more standards; extend to Cosmography for data topology.

- **Layers 1-2**: Achieve language parity (Rust/C#); modularize gymnasiums for template reuse.

- **Layer 3 (Tools)**: Enhance goneat/fulward for principle enforcement; add auto-notes and sync automation.

- **Layer 4 (Apps)**: Scale production forges; deepen AI integration via brooklyn-mcp.

- **Ecosystem**: Publish consolidated docs site; refine human-AI protocols for sustained collaboration.

Fulmen is a living system centered at [FulmenHQ](https://github.com/fulmenhq). This manifesto keeps everyone—from humans to AI agents—aligned
on the bar we set for ourselves and the promises we make to teams adopting the ecosystem.

> **Trademark Notice**: “Fulmen” and “FulmenHQ” are trademarks of 3 Leaps LLC. Refer to
> `README.md` and `LICENSE` for usage guidelines when creating derivatives or new
> distributions.
