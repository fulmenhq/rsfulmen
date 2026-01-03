# Fulmen Portable Testing Practices

**Audience:** All FulmenHQ repositories (Crucible, helper libraries, forges, customer templates)  
**Goal:** Keep `go test ./...`, `pytest`, `npm test`, etc. deterministic across laptops, CI, and sandboxed environments without hiding real regressions.

---

## Core Principles

1. **Deterministic Execution** – Avoid hard-coded ports, unseeded randomness, or undeclared timeouts. Seed PRNGs explicitly, derive configs from helpers, and surface timeouts via flags/env vars.
2. **Capability Detection** – Probe for network, filesystem, or kernel features (loopback sockets, IPv6, epoll) before relying on them. Skip with descriptive messages when missing rather than crashing.
3. **In-Memory First** – Prefer in-memory emitters/fakes (gofulmen telemetry memory emitter, SQLite `:memory:`, mock servers) for unit tests. Reserve real sockets/filesystems for integration tests guarded by capability checks.
4. **Context Propagation** – Pass `context.Context`, correlation IDs, and trace IDs through helper APIs so logs remain actionable under test.
5. **Isolated Cleanup** – Register cleanup handlers (`t.Cleanup`, `pytest` fixtures, `afterEach`) to tear down listeners, goroutines, and temp dirs. Tests must leave the environment as they found it.

---

## Recommended Structure

- **Test Modes** – Support `-short`, `FOO_TEST_FAST=1`, or similar flags to disable heavy integrations during tight loops.
- **Shared Skip Helpers** – Provide language-specific helpers that detect missing capabilities and skip with consistent reasons (e.g., `RequireNetwork(t *testing.T)` in Go, `require_network()` fixture in Python).
- **Resource Guards** – Wrap expensive operations (Prometheus exporters, DB containers) behind helpers that accept the test handle and perform capability checks.
- **Documentation Hooks** – Link this guide from language coding standards so contributors can find it easily.

---

## Language Guidance

### Go

- Bind listeners via `net.Listen("127.0.0.1:0")` (or `tcp4`) and inject them into `httptest.Server` to avoid IPv6-only binds.
- Use gofulmen’s in-memory telemetry emitter for unit tests; guard real exporters with skip helpers.
- Ensure error/logging helpers accept `context.Context` so request IDs flow through tests.

See: `docs/standards/coding/go.md#portable-testing`

### Python

- Use `pytest` fixtures (`tmp_path`, `monkeypatch`) for temp dirs and environment overrides.
- Call `pytest.skip` via shared helpers when sockets or DNS aren’t available; make skips explicit (`"network capability unavailable"`).

See: `docs/standards/coding/python.md#portable-testing`

### TypeScript / Node

- Avoid privileged ports; use utilities such as `get-port` to obtain ephemeral ports in Jest/Vitest.
- Clean up mocked timers and HTTP servers in `afterEach` to prevent cross-test leaks.

See: `docs/standards/coding/typescript.md#portable-testing`

_(Extend with additional languages as needed.)_

---

## Integration Points

- **Forge Standards** – Workhorse/codex/microtool standards should reference this guide in their testing sections.
- **Helper Library Standards** – `docs/architecture/fulmen-helper-library-standard.md` should point to these practices so Specialized modules follow the same rules.
- **CI Templates** – Reference the shared skip helpers and `-short` patterns to keep pipelines consistent.

---

## Next Steps

1. Update language coding standards (`docs/standards/coding/*.md`) with a “Portable Testing” section that links back here and captures language-specific helpers.
2. Cross-link from forge/helper architecture docs so template authors adopt these practices.
3. Encourage each repo to add shared skip helpers and reference them in CONTRIBUTING/README files.

Maintaining portable tests keeps AI agents, sandboxed developers, and CI pipelines aligned, reducing false negatives while preserving real regression detection.
