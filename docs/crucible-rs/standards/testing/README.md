# Fulmen Testing Standards

This folder contains testing guidance used across Crucible, helper libraries, and forge templates.

| Document                                                       | Scope                                                                                                                |
| -------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| [portable-testing-practices.md](portable-testing-practices.md) | Cross-language practices for deterministic, sandbox-friendly test suites.                                            |
| [language-testing-patterns.md](language-testing-patterns.md)   | Language-specific patterns (Go/Python/TypeScript/Rust/C#) with CLI-focused guidance such as Cobra command isolation. |

When adding new testing guidance, update this README with a short description so downstream teams can discover the right document quickly.

## Related: Testing Guides

For practical compliance routing and implementation patterns, see the [Testing Guides](../../guides/testing/README.md) family:

| Guide                                                                | Scope                                                     |
| -------------------------------------------------------------------- | --------------------------------------------------------- |
| [HTTP Server Patterns](../../guides/testing/http-server-patterns.md) | Server/fixture implementations with compliance checklists |
| [HTTP Client Patterns](../../guides/testing/http-client-patterns.md) | Client library testing with fixture usage patterns        |

**Standards vs Guides**: This folder contains normative standards (MUST/SHOULD). The guides folder contains compliance routing documents that direct readers to applicable standards.
