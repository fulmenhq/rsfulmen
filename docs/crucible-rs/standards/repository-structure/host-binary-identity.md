---
title: "Host Binary Identity Standard"
description: "Cross-language `version --extended` contract and build-time injection recipes for identifying a compiled binary (host identity) vs the SDK/SSOT pins it was built against"
author: "entarch"
author_of_record: "Dave Thompson (https://github.com/3leapsdave)"
supervised_by: "@3leapsdave"
date: "2026-08-07"
last_updated: "2026-08-08"
status: "approved"
tags:
  [
    "standards",
    "repository-structure",
    "cli",
    "identity",
    "version",
    "build",
    "release",
  ]
related_docs:
  - "README.md"
  - "go/cli-cobra.md"
  - "../fulmen/identity/README.md"
  - "../library/modules/app-identity.md"
---

# Host Binary Identity Standard

Codename: **`host-identity`**.

## 1. Purpose and scope

Forge CLIs need a **shared `version --extended` contract** so operators can dump build
provenance, support dirty-binary triage, and compare "what am I actually running" vs
"what SDK/SSOT was it built against". Today each CLI re-implements identity ad hoc, and
the Cobra CLI guide sketches ldflags without separating **host identity** from **SDK/SSOT
pins**, dirty semantics, or multi-language injection.

This standard delivers, **cross-language** (Go, Rust, TypeScript, Python):

1. A canonical field contract for the identity of a compiled binary (the **host**).
2. A clear separation of **host identity** from **SDK/SSOT pins**.
3. Explicit **dirty** semantics.
4. **Phase A** build-time injection recipes that require **no helper library**.
5. A **Phase B** gate for preferring the `gofulmen`/`rsfulmen` resolvers once those ship.

### Relationship to other identity standards (do not conflate)

| Standard                                                       | Concern                                                                                              | Runtime/build       |
| -------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- | ------------------- |
| **[App Identity Module](../library/modules/app-identity.md)**  | _Application_ identity: `binary_name`, `vendor`, env prefix, config discovery via `.fulmen/app.yaml` | Runtime             |
| **[Application Identity Guide](../fulmen/identity/README.md)** | Broader app identity: metadata, PID, service discovery, instance correlation                         | Runtime             |
| **This standard (host-identity)**                              | **Binary build identity**: version, commit, build date, dirty, runtime, platform, plus SDK/SSOT pins | **Build / release** |

The app-identity module tells you _which_ app you are and how it finds config. This
standard tells you _what build of that app is on disk_ and what it was built against.
They are complementary; a binary should be able to report both (app identity from
`app-identity`; build identity from this standard).

## 2. Core model: Host identity vs pins

**Host identity** = facts about the **binary you are running** (its own version, the
commit it was built from, when, whether the tree was dirty, the runtime and platform).

**SDK/SSOT pins** = versions of the **ecosystem libraries/toolchain the binary was built
against** (e.g. `gofulmen`, `rsfulmen`, `tsfulmen`, `pyfulmen`, and the Crucible SSOT
itself).

Normative rule: **pins are extended-only**, are reported under their own labeled block
(`Pins:` / `Crucible:` / `SSOT:`), and **never** surface as the host `Commit:`. The host
`Commit:` is THE commit of the binary's own source tree, nothing else. Mixing the two is
the primary anti-pattern this standard kills.

## 3. Field contract (host identity)

Canonical keys are **camelCase** for structured output (aligned with the repository's
`success`/`message`/`data` envelope convention); text output uses the hyphenated labels
shown below.

| Field      | Key (JSON/YAML) | Text label  | Placeholder            | Notes                                                                                                                 |
| ---------- | --------------- | ----------- | ---------------------- | --------------------------------------------------------------------------------------------------------------------- |
| Version    | `version`       | `Version`   | `dev`                  | App/binary version (from `VERSION` file or package metadata; SemVer/CalVer per repository-versioning)                 |
| Commit     | `commit`        | `Commit`    | `unknown`              | Prefer short (7) hash for text; full 40 allowed in `--json`                                                           |
| Build date | `buildDate`     | `BuildDate` | `unknown`              | RFC3339 UTC timestamp                                                                                                 |
| Dirty      | `dirty`         | `Dirty`     | omit/null when unknown | `true` if VCS tree was dirty at build **or** injected; see §4                                                         |
| Runtime    | `runtime`       | `Runtime`   | omit when unknown      | Language-appropriate: `go1.23`, `rustc 1.70`, `node 20`, `python 3.12`                                                |
| Platform   | `platform`      | `Platform`  | omit when unknown      | Both `darwin/arm64` (Go) and `macos/aarch64` (Rust) are conformant; prefer the Go form (`OS/GOARCH`) in JSON examples |

### Structured output shape

```json
{
  "name": "tool-name",
  "version": "0.4.16",
  "commit": "a1b2c3d",
  "buildDate": "2026-08-07T12:00:00Z",
  "dirty": false,
  "runtime": "go1.23.4",
  "platform": "darwin/arm64",
  "pins": {
    "gofulmen": "v0.12.0",
    "rsfulmen": "v0.8.2",
    "crucible": "v0.4.16"
  }
}
```

The `pins` block is **extended-only**. `name` comes from app identity where available,
else the binary's own known name.

## 4. Dirty semantics

`dirty` is `true` when **either**:

- the VCS working tree had uncommitted changes **at build time**, **or**
- a build explicitly injects dirty (e.g. a CI build from a `-dev` tip with local patches).

`dirty` is **omitted/null (never `false`)** when it cannot be determined, so a host that
cannot establish cleanliness must not falsely claim to be clean. **Normative recipe rule:**
when the VCS probe **succeeds**, the tool MUST inject a concrete `true` or `false` (a
clean tree is `false` — a real, useful signal, as chanvoy already prints); only when the
probe **fails / VCS is unavailable** is `dirty` omitted/`unknown`. Rules:

1. Prefer deterministic detection: `git status --porcelain` non-empty → `true`; empty →
   `false`.
2. If the probe succeeds but returns no result, emit `false` (known-clean), not omission.
3. Omit/`unknown` **only** when no VCS (or no commit resolved / probe failed). A
   `git status` command **failure** after a successful repo probe is treated as unknown,
   **not** as clean.

Reproducibility note: a release build from a clean tag is `dirty: false`. A local
`go run` / ad-hoc build is typically `dirty: true` — this is expected and is exactly the
signal support wants.

## 5. CLI UX: `version` vs `version --extended`

| Invocation                                                    | Output                                                                                        |
| ------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| `tool --version` (root flag; short `-V`/`-v` per house style) | Single short line: `tool-name 0.4.16`                                                         |
| `tool version`                                                | **Short**: name + version only (matches forge dogfood: gonimbus, fulseed, limensafe, chanvoy) |
| `tool version --extended` / `-e`                              | **Full** host identity block **plus** pins block                                              |
| `tool version --json`                                         | Structured host identity (+ pins when `--extended`)                                           |

Non-Cobra dialects (clap, Click, Fastify/Bun) **must** expose the same surface:
`--extended`/`-e`, `--json`, and a root `--version`. **Root short flag:** do **not**
mandate `-v` where it collides with `--verbose` (cobra/clap frequently do). Prefer
`--version` plus a language-conventional short (`-V` where that is house style). This
reconciles the repository-structure CLI category (`--version/-v`) carefully — the long
flag is the stable contract; the short is best-effort to avoid collision.

### Text shapes

Bare `version` (short — name + version):

```text
tool-name 0.4.16
```

`version --extended` (full block on extended only):

```text
tool-name 0.4.16
Commit:     a1b2c3d
BuildDate:  2026-08-07T12:00:00Z
Dirty:      true
Runtime:    go1.23.4
Platform:   darwin/arm64

Pins:
  gofulmen    v0.12.0
  rsfulmen    v0.8.2
  Crucible    v0.4.16
```

## 6. Environment injection contract (`FULMEN_HOST_*`)

CI and build tooling may inject identity externally for reproducibility and provenance.
Phase A recipes may read these; the eventual resolvers (Phase B) standardize on them.

| Env var                  | Field                              |
| ------------------------ | ---------------------------------- |
| `FULMEN_HOST_VERSION`    | `version`                          |
| `FULMEN_HOST_COMMIT`     | `commit`                           |
| `FULMEN_HOST_BUILD_DATE` | `buildDate`                        |
| `FULMEN_HOST_DIRTY`      | `dirty`                            |
| `FULMEN_HOST_RUNTIME`    | `runtime` (rare; usually derived)  |
| `FULMEN_HOST_PLATFORM`   | `platform` (rare; usually derived) |

Injected values take precedence over build-time defaults but are **still part of host
identity** (they describe the binary you are running). They are distinct from
`FULMEN_APP_IDENTITY_PATH`, which selects the app-identity file — do not reuse that
variable for host fields.

**CI/build note:** release pipelines often have **no usable `.git`** (shallow/archived
checkouts). CI **may inject** `FULMEN_HOST_*` from the pipeline instead — do **not**
force a runtime `git` probe inside released binaries. When injecting, CI supplies the
full `version`/`commit`/`buildDate`/`dirty` set at build time.

### Trust boundary

Host identity is **informational, not attestation**. It describes what a binary reports
about itself at build time; it is NOT cryptographically bound to the artifact. Host
identity fields **MUST NOT** be consumed for authentication, authorization, integrity,
or other security decisions. Treat them as diagnostics/support/provenance hints only.
(For machine-verifiable provenance, use the artifact's signed tag/SBOM — see release
process.)

## 7. Phase A recipes (no helper library required)

Phase A is deliberately self-contained: copy-paste and it works, with **no gofulmen /
rsfulmen / tsfulmen / pyfulmen dependency**. Do **not** add imports to libraries whose
resolver APIs do not exist yet (see §8).

### 7.1 Go — Makefile ldflags (extends the Cobra CLI guide)

`makefile`:

```makefile
BINARY_NAME := tool-name
VERSION     := $(shell cat VERSION 2>/dev/null || echo dev)
COMMIT      := $(shell git rev-parse --short HEAD 2>/dev/null || echo unknown)
BUILD_DATE  := $(shell date -u +"%Y-%m-%dT%H:%M:%SZ")

# dirty: true/false when the VCS probe succeeds; empty (=> omit/null) when VCS is
# unavailable OR the status probe itself fails (a failed probe is unknown, never "clean").
DIRTY := $(shell \
  if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then \
    STATUS=$$(git status --porcelain 2>/dev/null) && { [ -z "$$STATUS" ] && echo false || echo true; }; \
  fi)

LDFLAGS := -X main.version=$(VERSION) \
           -X main.commit=$(COMMIT) \
           -X main.buildDate=$(BUILD_DATE) \
           -X main.dirty=$(DIRTY)

build:
	go build -ldflags="$(LDFLAGS)" -o bin/$(BINARY_NAME) ./cmd/$(BINARY_NAME)
```

`cmd/tool-name/main.go` (extend the Cobra guide entry point):

```go
package main

import "tool-name/internal/cmd"

// Identity, injected at build time via ldflags (see Host Binary Identity Standard).
var (
	version   = "dev"
	commit    = "unknown"
	buildDate = "unknown"
	dirty     = ""
	// runtime / platform are typically derived: runtime.Version(), runtime.GOOS/GOARCH
)

func main() {
	cmd.SetHostIdentity(identity{
		Version:   version,
		Commit:    commit,
		BuildDate: buildDate,
		Dirty:     dirty,
	})
	// ...
}
```

The `version` command renders bare or extended as in §5; `--json` marshals the §3 shape.
`dirty` is populated only when the injected value is non-empty: `""`/omit → `null`;
`"true"` → `true`; `"false"` → `false`.

### 7.2 Rust — `build.rs` + `FULMEN_HOST_*` (env! injection)

A dependency-free `build.rs` reads injected env (or derives from git) and stamps
`cargo:rustc-env=...`:

```rust
// build.rs — self-contained, no external crates required
use std::env;
use std::process::Command;

fn git_short_commit() -> String {
    Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn git_dirty() -> String {
    // Probe first: only report true/false when the VCS is usable.
    let ok = Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !ok {
        return String::new(); // VCS unavailable -> unknown (omit)
    }
    match Command::new("git").args(["status", "--porcelain"]).output() {
        Ok(o) if o.status.success() => {
            if o.stdout.is_empty() { "false".into() } else { "true".into() } // known-clean -> false
        }
        _ => String::new(), // status probe FAILED -> unknown, never a misleading "false"
    }
}

fn rfc3339_utc() -> Option<String> {
    Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
}

fn main() {
    // Rerun the build script if any injected var changes, so stale stamps never cache.
    for var in [
        "FULMEN_HOST_VERSION",
        "FULMEN_HOST_COMMIT",
        "FULMEN_HOST_BUILD_DATE",
        "FULMEN_HOST_DIRTY",
    ] {
        println!("cargo:rerun-if-env-changed={var}");
    }

    // FULMEN_HOST_* take precedence; fall back to derivation.
    let version = env::var("FULMEN_HOST_VERSION")
        .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string());
    let commit = env::var("FULMEN_HOST_COMMIT")
        .unwrap_or_else(|_| git_short_commit());
    let build_date = env::var("FULMEN_HOST_BUILD_DATE")
        .ok()
        .or_else(rfc3339_utc) // RFC3339 UTC, not epoch-seconds
        .unwrap_or_else(|| "unknown".to_string());
    let dirty = env::var("FULMEN_HOST_DIRTY").unwrap_or_else(|_| git_dirty());

    println!("cargo:rustc-env=FULMEN_HOST_VERSION={version}");
    println!("cargo:rustc-env=FULMEN_HOST_COMMIT={commit}");
    println!("cargo:rustc-env=FULMEN_HOST_BUILD_DATE={build_date}");
    println!("cargo:rustc-env=FULMEN_HOST_DIRTY={dirty}");
}
```

Note: `cargo:rustc-env=` cannot emit an _unset_ variable — omit/`unknown` is represented
by the `unknown`/empty-string markers that `src/main.rs` maps to `null`, never as a false
value. `cargo:rerun-if-env-changed` is required so a changed `FULMEN_HOST_*` rebuilds
rather than serving stale stamps.

In `src/main.rs`:

```rust
fn host_identity() -> serde_json::Value {
    serde_json::json!({
        "name": env!("CARGO_PKG_NAME"),
        "version": env!("FULMEN_HOST_VERSION"),
        "commit": env!("FULMEN_HOST_COMMIT"),
        "buildDate": env!("FULMEN_HOST_BUILD_DATE"),
        "dirty": match env!("FULMEN_HOST_DIRTY") {
            "true"  => serde_json::Value::Bool(true),
            "false" => serde_json::Value::Bool(false),
            _       => serde_json::Value::Null, // empty or invalid -> unknown, never a misleading "false"
        },
        "runtime": format!("rustc {}", rustc_version()),
        "platform": format!("{}/{}", std::env::consts::OS, std::env::consts::ARCH),
    })
}
```

The `--extended`/`-e` flag (clap, or a hand-rolled parser) switches the `version`
subcommand between §5 text shapes; `--json` emits the §3 shape. Recipes using the
`vergen` crate are acceptable when a project already uses it, but must emit the same
fields.

### 7.3 TypeScript — build-time stamp primary, runtime env fallback (shorter appendix)

**Primary (preferred):** stamp host identity **at build time** via the bundler's `define`
(Vite `import.meta.env` / Bun macro / esbuild define). This cannot be forged by the
runtime environment.

```ts
// Build-time defines (Vite/Bun/esbuild define):
//   define: { __APP_VERSION__: JSON.stringify(version), __COMMIT_HASH__: ..., ... }
const host = {
  version:   __APP_VERSION__,
  commit:    __COMMIT_HASH__,
  buildDate: __BUILD_DATE__,
  dirty:     /* static bool or undefined */,
  runtime:   `node ${process.version}`,
  platform:  `${process.platform}/${process.arch}`,
};
```

**Fallback only:** reading `FULMEN_HOST_*` at runtime is permitted **only** as a fallback
when a build-time stamp is unavailable, and MUST be documented as **informational /
caller-influenceable** — the parent process or a wrapper can set these vars, so a
runtime-read `commit`/`dirty` is not trustworthy for triage decisioning.

```ts
// Fallback — informational only; see §6 trust boundary.
const host = {
  version: process.env.FULMEN_HOST_VERSION ?? "dev",
  commit: process.env.FULMEN_HOST_COMMIT ?? "unknown",
  buildDate: process.env.FULMEN_HOST_BUILD_DATE ?? "unknown",
  dirty: (() => {
    const d = process.env.FULMEN_HOST_DIRTY;
    return d === "true" || d === "false" ? d === "true" : undefined;
  })(),
  runtime: `node ${process.version}`,
  platform: `${process.platform}/${process.arch}`,
};
```

- A Bun/Node CLI exposes `--version`, `version`, `version --extended`/`-e`, `--json`
  with consistent shapes.

### 7.4 Python — build-time/meta primary, env fallback (shorter appendix)

**Primary:** `version` from distribution metadata (`importlib.metadata.version`) — this is
build-time bound to the installed dist. Stamp `commit`/`buildDate`/`dirty` at **build
time** via a generated module or `setuptools-scm` (LocalVersion/piece-version) where
possible.

**Fallback only:** runtime `FULMEN_HOST_*` reads are permitted as a fallback and MUST be
documented as **informational / caller-influenceable** (see §6 trust boundary).

```python
import os
import platform
from importlib.metadata import PackageNotFoundError, version

def _host():
    try:
        ver = version("my_dist")
    except PackageNotFoundError:
        ver = "dev"  # documented placeholder; JSON-serializable
    dirty_raw = os.environ.get("FULMEN_HOST_DIRTY")
    dirty = (dirty_raw == "true") if dirty_raw in ("true", "false") else None  # True/False/None
    return {
        "version":   ver,
        "commit":    os.environ.get("FULMEN_HOST_COMMIT", "unknown"),
        "buildDate": os.environ.get("FULMEN_HOST_BUILD_DATE", "unknown"),
        "dirty":     dirty,
        "runtime":   f"python {platform.python_version()}",
        "platform":  f"{platform.system().lower()}/{platform.machine()}",
    }
```

- A Click CLI adds a `version` command honoring `--extended`/`-e` and `--json`, keeping
  the §5 and §3 shapes.

## 8. Phase B gate (library resolvers)

When `gofulmen` / `rsfulmen` (and later `tsfulmen` / `pyfulmen`) ship **host-identity
resolvers** (`Resolve`), prefer them over hand-rolled injection. That is **blocked** on
the gofulmen / rsfulmen (and later pyfulmen / tsfulmen) resolver tracks shipping.

Until those ship:

- Do **not** invent import paths or APIs that do not exist.
- Keep Phase A recipes in this standard authoritative.
- When resolvers ship, they MUST emit the same field contract (§3) and honor
  `FULMEN_HOST_*` precedence, so no consumer-visible shape change occurs.

**chanvoy** (lanytehq) already proved Phase A externally with local `FULMEN_HOST_*`
injection and `version --extended` — a reference implementation of a
helper-library-free recipe.

## 9. SSOT and link wiring

This standard is the **hub** for host build identity. Inbound references:

- The Cobra CLI guide's "Version Command" + "Makefile" sections link here for the
  extended `--extended`/dirty/pins contract.
- The repository-structure CLI category links here for `version --extended`.
- The Application Identity guide and App Identity module carry a one-line cross-reference
  distinguishing runtime app identity from build host identity.

## 10. Conformance checklist

- [ ] CLI exposes root `--version` (short per house style) and `version` (short, name+version).
- [ ] `version --extended` / `-e` emits full host identity + pins block.
- [ ] `version --json` emits the §3 camelCase shape; pins appear only when extended.
- [ ] `dirty` is `true`/`false`/`unknown`; never a misleading `false` when undetermined.
- [ ] Pins are labeled (`Pins:` / `Crucible:` / `SSOT:`); Crucible/pins never shown as host `Commit:`.
- [ ] Host `Commit:` is the binary's own source commit only.
- [ ] Phase A recipes require no helper library; no invented import paths (§8).
- [ ] Empty placeholders: `dev` / `unknown` per §3 table.

## 11. Out of scope

- Implementing helper-library host-identity resolvers (Phase B, separate tracks).
- Migrating every existing CLI (chanvoy is the external Phase A dogfood).
- `appidentity` runtime discovery changes (separate module).
- Release packaging, tagging, signing, and publishing.
