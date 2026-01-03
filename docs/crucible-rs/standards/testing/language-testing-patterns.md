# Language-Specific Testing Patterns

This guide complements the cross-language [Portable Testing Practices](portable-testing-practices.md) by describing patterns frequently used in Fulmen helper libraries and forges. Each section captures idioms for a specific language so that test suites remain isolated, deterministic, and CI-friendly. When adding new language guidance, follow the structure below so teams can compare patterns easily.

---

## Go – Cobra Command Isolation

**Use case:** CLI applications built with [spf13/cobra](https://github.com/spf13/cobra) (e.g., goneat, workhorse forges) where tests were flaking because `rootCmd` singletons retain state between runs.

**Pattern overview** (from `goneat/.plans/memos/crucible/cobra-test-isolation-pattern.md`):

1. **Factory Function** – Replace package-level `var rootCmd = &cobra.Command{}` with:
   ```go
   func newRootCommand() *cobra.Command { ... }
   var rootCmd = newRootCommand()
   func registerSubcommands(cmd *cobra.Command) { ... }
   func init() { registerSubcommands(rootCmd) }
   ```
   Tests call the factory to get a fresh command tree.
2. **Flag Reset Helpers** – Reset package-level flag variables (`flagDoctorInstall`, `flagDoctorScope`, etc.) before each test to prevent polluted defaults.
3. **Test Helper** – Centralize execution helper:
   ```go
   func execCmd(t *testing.T, args []string) (string, error) {
       resetDoctorFlags()
       cmd := newRootCommand()
       registerSubcommands(cmd)
       cmd.SetArgs(args)
       return captureOutput(cmd.Execute)
   }
   ```
4. **Shared Cleanup** – Use `t.Cleanup` to restore global logger output (JSON vs text) and remove temp dirs.

**Anti-patterns to avoid:**

- Reusing the global `rootCmd` in tests.
- Allowing subcommands to self-register in their `init()` (breaks the factory approach).
- Failing to reset package-level flag variables between tests.

Adopt this pattern in any Cobra-based Fulmen CLI to ensure `go test ./...` behaves the same regardless of test order.

---

## Python – Typer/Click CLI Isolation

**Use case:** Python CLIs built with Typer or Click (e.g., microtool forges) where context/state carries across tests.

**Pattern:**

1. **Application Factory** – Export `def create_app() -> typer.Typer` that builds the CLI. Production code instantiates once; tests call the factory per case.
2. **Contextvars Reset** – Use fixtures to reset global context vars or caches (`contextvars.ContextVar`, `functools.lru_cache`) between tests.
3. **Runner Helpers** – Use `typer.testing.CliRunner` with helper functions that inject environment variables, temp directories, and mock config paths, ensuring tests never touch user directories.

---

## TypeScript – Commander / oclif Isolation

**Use case:** Node/Bun CLIs (tsfulmen-based tools, codex forges).

**Pattern:**

1. **Factory** – Export `function buildProgram(): Command` returning a fresh Commander instance or oclif command class.
2. **Mock process state** – Use helper to capture `process.stdout`/`stderr` and reset `process.argv`, `env`, and timer mocks in `afterEach`.
3. **Portability** – Use `get-port` for any server tests; guard real network calls with feature flags.

---

## Rust – Clap/Tokio CLIs

**Use case:** Upcoming `rsfulmen` modules and Rust-based microtools.

**Pattern:**

1. **Builder Function** – `fn build_cli() -> Command` returns a new clap command each time; tests call `build_cli().try_get_matches_from(...)`.
2. **Async Isolation** – Use `tokio::test(flavor = "multi_thread", worker_threads = 1)` for deterministic scheduler behavior; provide helper macros that build runtimes per test if frameworks require it.
3. **Temp Directories** – Use `tempfile::TempDir` and drop at the end of each test to ensure cleanup in windows + sandboxed environments.

---

## C# – System.CommandLine / Spectre.Console.Cli

**Use case:** Planned csfulmen modules and future .NET microtools.

**Pattern:**

1. **CommandFactory** – Provide `Command CreateRootCommand()` and register subcommands centrally so NUnit/xUnit tests can build new command graphs.
2. **Dependency Injection** – Avoid static singletons; wire services through DI containers per test scope (`ServiceCollection`, `ServiceProvider`).
3. **Output Capture** – Use `System.IO.StringWriter` and dependency-injected console abstractions to capture CLI output; reset `Console.SetOut`/`SetError` in `Dispose`.

---

## Contributing New Patterns

When teams discover language-specific testing patterns (e.g., pytest fixtures for cloud emulators, Vitest helpers for worker threads), add a new subsection here and reference any canonical memos or design docs so others can reuse the approach.
