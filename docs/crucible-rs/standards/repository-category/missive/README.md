---
title: "Missive Category Standards"
description: "Standards for missive repositories - single-page promotional/CTA sites"
author: "Fulmen Enterprise Architect (@fulmen-ea-steward)"
date: "2025-12-20"
last_updated: "2025-12-20"
status: "active"
tags: ["standards", "repository-category", "missive", "static-site", "v0.2.26"]
---

# Missive Category Standards

## Purpose

This document defines the requirements and standards for `missive` category repositories. Missive repositories are single-page promotional or call-to-action (CTA) sites with minimal dependencies, designed for quick creation, visual appeal, and low maintenance.

## Category Definition

**Key**: `missive`

**Summary**: Single-page promotional/CTA sites with minimal dependencies.

**Philosophy**: "Missive" evokes a focused, one-way message — simple, impactful, and often ephemeral. These sites prioritize quick creation over complex features.

## Differentiation from Codex

| Aspect        | missive                   | Codex                      |
| ------------- | ------------------------- | -------------------------- |
| Pages         | Single (or very few)      | Multi-page collections     |
| Content model | Static, no blog/search    | Rich content, versioning   |
| Build         | None (vanilla) or minimal | Full SSG (Astro/Starlight) |
| Lifecycle     | Often ephemeral           | Long-lived                 |
| Dependencies  | Zero to minimal           | Framework-dependent        |

**Escalation rule**: If a site grows beyond one page, needs blog/events calendar, search, or rich interactivity → migrate to Codex.

## Requirements

### MUST

1. **Be single page (or very few pages)**: The defining characteristic of missive is focus. Multi-page sites belong in Codex.

2. **NOT include blog, search, or content collections**: These features indicate Codex scope.

### SHOULD

1. **Use vanilla HTML/CSS with zero build**: The default approach is plain files — no build step, no dependencies, instant deployment.

2. **Be accessible**: Follow WCAG 2.1 AA minimum (semantic HTML, alt text, sufficient contrast).

3. **Be responsive**: Mobile-first design using modern CSS (Flexbox/Grid).

4. **Load fast**: Optimize images, minimize assets, avoid unnecessary JavaScript.

### MAY

1. **Use light CSS framework via CDN**: When consistent styling is desired without custom CSS:
   - **Pico.css** or **Water.css** — Classless semantic styling
   - **Bootstrap 5** — When more components needed
   - Link via CDN in `<head>`; no build required.

2. **Use minimal SSG**: Only when partials/reuse are justified:
   - **Astro** (single-file components)
   - **Eleventy** (nunjucks templates)
   - Build to static HTML.
   - Use when multiple similar missives share headers/footers.

3. **Include basic analytics**: Privacy-respecting options (Plausible, Umami).

### MUST NOT

1. **Use React/Vue/SPAs**: Overkill for single-page CTA.
2. **Use heavy frameworks**: Next.js, Gatsby, etc. are inappropriate.

## Repository Structure

### Tier 1: Pure Vanilla (Default)

```
missive-repo/
├── index.html
├── assets/
│   ├── images/
│   │   └── hero.webp
│   └── styles.css
├── .github/
│   └── workflows/
│       └── deploy.yml
└── README.md
```

### Tier 2: Light Framework

```
missive-repo/
├── index.html          # Links Pico.css via CDN
├── assets/
│   └── images/
├── .github/
│   └── workflows/
│       └── deploy.yml
└── README.md
```

### Tier 3: Minimal SSG (Only If Justified)

```
missive-repo/
├── src/
│   ├── pages/
│   │   └── index.astro
│   └── components/
│       └── Header.astro
├── public/
│   └── images/
├── astro.config.mjs
├── package.json
├── .github/
│   └── workflows/
│       └── deploy.yml
└── README.md
```

## Example: Vanilla Skeleton

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Annual Charity Fundraiser 2025</title>
    <link
      rel="stylesheet"
      href="https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.min.css"
    />
    <style>
      /* Minimal custom styles */
      .hero {
        text-align: center;
        padding: 4rem 1rem;
      }
    </style>
  </head>
  <body>
    <main class="container">
      <section class="hero">
        <h1>Annual Holiday Fundraiser</h1>
        <p>Join us December 25th for an evening of giving and community.</p>
        <a href="#donate" role="button">Donate Now</a>
      </section>
    </main>
  </body>
</html>
```

## Deployment

Recommended platforms (all support free tier):

- **Netlify Drop** — Drag-and-drop deployment
- **Cloudflare Pages** — Git integration or folder upload
- **GitHub Pages** — Direct from repository
- **Vercel** — Git integration

For forms, use third-party services to avoid backend:

- Formspree
- Basin
- Google Forms embed

## Template Naming

Forge templates for missive follow the pattern:

```
forge-missive-{name}
```

Where `{name}` could use message/communication metaphors (naming pattern TBD).

## Use Cases

- Event announcements (concerts, conferences, meetups)
- Charity fundraisers
- Product launches
- Coming soon pages
- Quick promotional campaigns

## When to Escalate to Codex

Migrate to Codex category when:

- Multiple pages are needed
- Blog or news feed is required
- Search functionality is needed
- Content collections (events calendar, team directory) are required
- Rich interactivity beyond simple forms is needed
- Long-term content maintenance is expected

## Related Documentation

- [Repository Category Standards](../README.md) — Overview of all categories
- [Codex Category Standards](../codex/config-standard.md) — For when missive scope is exceeded
- [Fulmen Forge Codex Standard](../../../architecture/fulmen-forge-codex-standard.md) — Full Codex architecture

---

**Status**: Active (v0.2.26+)

**Maintainers**: Crucible Team
