# Testing Guides

Practical guides for testing patterns used across the Fulmen ecosystem. These guides serve as **compliance routing documents** - they direct you to normative standards and provide practical examples, checklists, and anti-patterns.

## Purpose

Guides in this family help developers (human and AI) discover which standards apply to their current work. Each guide includes:

- **Compliance Requirements** - explicit list of standards to read before implementation
- **Pre-Implementation Checklist** - items to verify before writing code
- **Pre-Review Checklist** - items for reviewers (devrev) to validate
- **Anti-Patterns** - common mistakes with solutions
- **Code Examples** - practical implementations across languages

## Available Guides

| Guide                                           | Scope                          | Key Standards Referenced                         |
| ----------------------------------------------- | ------------------------------ | ------------------------------------------------ |
| [HTTP Server Patterns](http-server-patterns.md) | Server/fixture implementations | http-rest-standards, coding/go, fixture-standard |
| [HTTP Client Patterns](http-client-patterns.md) | Client library testing         | http-rest-standards, coding/\*                   |

## Relationship to Standards

```
┌─────────────────────────────────────────────────────────────────┐
│  Guide (Routing + Practical)                                    │
│  "When building HTTP servers, comply with these standards..."   │
│                                                                 │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ Standard A      │  │ Standard B      │  │ Standard C      │ │
│  │ (MUST/SHOULD)   │  │ (MUST/SHOULD)   │  │ (MUST/SHOULD)   │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

**Guides do not replace standards** - they route to them. Normative requirements (MUST, SHOULD) live in standards documents. Guides explain WHEN and WHERE to apply those standards.

## When to Use These Guides

| If you're...                       | Start with...                                                                          |
| ---------------------------------- | -------------------------------------------------------------------------------------- |
| Building an HTTP server or fixture | [HTTP Server Patterns](http-server-patterns.md)                                        |
| Writing HTTP client tests          | [HTTP Client Patterns](http-client-patterns.md)                                        |
| Implementing CLI commands          | [Language Testing Patterns](../../../standards/testing/language-testing-patterns.md)   |
| Setting up test infrastructure     | [Portable Testing Practices](../../../standards/testing/portable-testing-practices.md) |

## Future Guides

This family will expand to cover additional testing domains:

| Planned Guide              | Scope                          | Status  |
| -------------------------- | ------------------------------ | ------- |
| Auth Testing Patterns      | OAuth, OIDC, API key testing   | Planned |
| Datastore Testing Patterns | DB, cache, queue fixture usage | Planned |
| Chaos Testing Patterns     | Failure injection, resilience  | Planned |

## Related Documentation

- [Portable Testing Practices](../../standards/testing/portable-testing-practices.md) - Cross-language testing standards
- [Language Testing Patterns](../../standards/testing/language-testing-patterns.md) - CLI testing patterns (Cobra, Typer, Commander)
- [Fixture Standard](../../architecture/fulmen-fixture-standard.md) - Fixture repository requirements
- [HTTP REST Standard](../../standards/protocol/http-rest-standards.md) - HTTP protocol requirements
