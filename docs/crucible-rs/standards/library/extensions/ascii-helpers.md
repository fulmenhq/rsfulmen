---
title: "ASCII Helpers Extension"
description: "Optional helper module for console formatting utilities"
author: "Schema Cartographer"
date: "2025-10-09"
last_updated: "2025-10-09"
status: "draft"
tags: ["standards", "library", "extensions", "console", "2025.10.2"]
---

# ASCII Helpers Extension

## Scope

Expose lightweight utilities for rendering ASCII/Unicode tables, boxes, progress bars, and highlighted text in
command-line tools. Optional but improves DX for Fulmen CLIs.

## Capabilities

- Table rendering with automatic column width detection and alignment.
- Box drawing utilities (borders, callouts) using ASCII by default with optional Unicode line art.
- Progress indicators (spinner, progress bar) respecting TTY detection.
- Color/formatting helpers that degrade gracefully when ANSI support absent (or when `NO_COLOR` set).

## Implementation Notes

- **Go**: Wrap popular packages (`github.com/jedib0t/go-pretty/table`) with Fulmen defaults. Provide interfaces
  for dependency injection.
- **Python**: Use `rich` or `textual` primitives with a thin adapter (lazy import to avoid heavy deps when unused).
- **TypeScript**: Use `cli-table3`, `kleur`, `ora` with an adapter exported from helper library.

## Testing

- Snapshot tests verifying ASCII output for core helpers.
- Ensure behavior when not attached to TTY (disable color/progress).

## Status

- Optional; adopt when CLI experience benefits justify additional dependencies.
