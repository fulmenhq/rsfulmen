# RSFulmen Makefile
# Repository: rsfulmen
# Bootstrapped with: Goneat
# Compliant with: FulmenHQ Makefile Standard
#
# Quick Start Commands:
#   make help           - Show all available commands
#   make bootstrap      - Install dependencies and external tools
#   make test           - Run tests
#   make build          - Build library
#   make check-all      - Full quality check (lint, test)

# Variables
VERSION := $(shell cat VERSION 2>/dev/null || echo "0.1.0")
LIFECYCLE := $(shell cat LIFECYCLE_PHASE 2>/dev/null || echo experimental)
BIN_DIR := ./bin

# External tooling (bootstrap)
BINDIR ?= $(HOME)/.local/bin
GONEAT_VERSION ?= v0.4.1
SFETCH_INSTALL_URL ?= https://github.com/3leaps/sfetch/releases/latest/download/install-sfetch.sh

# Coverage thresholds by lifecycle phase
# experimental: 0%, alpha: 30%, beta: 60%, rc: 70%, ga: 75%, lts: 80%
ifeq ($(LIFECYCLE),alpha)
    COVERAGE_MIN := 30
else ifeq ($(LIFECYCLE),beta)
    COVERAGE_MIN := 60
else ifeq ($(LIFECYCLE),rc)
    COVERAGE_MIN := 70
else ifeq ($(LIFECYCLE),ga)
    COVERAGE_MIN := 75
else ifeq ($(LIFECYCLE),lts)
    COVERAGE_MIN := 80
else
    COVERAGE_MIN := 0
endif

.PHONY: all help bootstrap bootstrap-force bootstrap-cargo-tools tools hooks-ensure sync sync-ssot lint fmt test test-cov build build-all clean
.PHONY: version version-set version-propagate version-bump-major version-bump-minor version-bump-patch version-bump-calver
.PHONY: check-all quality precommit prepush lifecycle msrv-check
.PHONY: release-check release-prepare release-build release-clean
.PHONY: release-provenance-check release-guard-tag-version release-tag release-verify-tag
.PHONY: doc

# Default target
all: check-all

# Help target
help: ## Show this help message
	@echo "RSFulmen - Rust Fulmen Helper Library"
	@echo ""
	@echo "Core Targets:"
	@echo "  help                 - Show this help message"
	@echo "  bootstrap            - Install dependencies and external tools"
	@echo "  bootstrap-force      - Force reinstall all tools (or use: make bootstrap FORCE=1)"
	@echo "  bootstrap-cargo-tools - Install Rust cargo tools only"
	@echo "  tools                - Verify external tools are available"
	@echo "  hooks-ensure         - Ensure git hooks are installed"
	@echo "  sync            - Alias for sync-ssot"
	@echo "  sync-ssot       - Sync assets from Crucible SSOT"
	@echo "  lint            - Run linting checks (clippy)"
	@echo "  fmt             - Format code (rustfmt)"
	@echo "  test            - Run all tests"
	@echo "  test-cov        - Run tests with coverage enforcement"
	@echo "  build           - Build library"
	@echo "  build-all       - Build all targets (N/A for library)"
	@echo "  clean           - Remove build artifacts"
	@echo "  doc             - Generate documentation"
	@echo "  lifecycle       - Show current lifecycle phase and requirements"
	@echo ""
	@echo "Version Targets:"
	@echo "  version         - Print current version"
	@echo "  version-set     - Update VERSION (usage: make version-set VERSION=x.y.z)"
	@echo "  version-propagate - Sync VERSION to Cargo.toml"
	@echo "  version-bump-*  - Bump version (major/minor/patch/calver)"
	@echo ""
	@echo "Quality Targets:"
	@echo "  check-all       - Run all quality checks (lint, test)"
	@echo "  quality         - Run build, lint, and tests"
	@echo "  precommit       - Pre-commit hooks (fast, critical issues)"
	@echo "  prepush         - Pre-push hooks (comprehensive, high severity)"
	@echo ""
	@echo "Release Targets:"
	@echo "  release-check   - Validate release readiness"
	@echo "  release-prepare - Prepare for release"
	@echo "  release-build   - Build release artifacts"
	@echo "  release-clean   - Remove local release artifacts"
	@echo ""
	@echo "Tag Signing Targets:"
	@echo "  release-provenance-check  - Verify Crucible SSOT provenance files"
	@echo "  release-guard-tag-version - Guard: ensure tag matches VERSION"
	@echo "  release-tag               - Create and verify a signed git tag"
	@echo "  release-verify-tag        - Verify the signed git tag"
	@echo ""

# Bootstrap targets
bootstrap: ## Install dependencies and external tools
	@echo "Bootstrapping rsfulmen development environment..."
	@mkdir -p "$(BINDIR)"
	@echo ""
	@echo "Step 1: Installing sfetch (trust anchor)..."
	@if ! command -v sfetch >/dev/null 2>&1; then \
		echo "→ Installing sfetch into $(BINDIR)..."; \
		if command -v curl >/dev/null 2>&1; then \
			curl -sSfL "$(SFETCH_INSTALL_URL)" | bash -s -- --dir "$(BINDIR)" --yes; \
		elif command -v wget >/dev/null 2>&1; then \
			wget -qO- "$(SFETCH_INSTALL_URL)" | bash -s -- --dir "$(BINDIR)" --yes; \
		else \
			echo "❌ curl or wget is required to bootstrap sfetch"; \
			exit 1; \
		fi; \
	else \
		echo "✅ sfetch already installed"; \
	fi
	@echo ""
	@echo "Step 2: Installing goneat via sfetch..."
	@SFETCH_BIN="$$(command -v sfetch 2>/dev/null || true)"; \
	if [ -z "$$SFETCH_BIN" ] && [ -x "$(BINDIR)/sfetch" ]; then SFETCH_BIN="$(BINDIR)/sfetch"; fi; \
	if [ -z "$$SFETCH_BIN" ]; then echo "❌ sfetch not found after bootstrap"; exit 1; fi; \
	if [ "$(FORCE)" = "1" ] || [ "$(FORCE)" = "true" ]; then \
		echo "→ Force installing goneat $(GONEAT_VERSION) into $(BINDIR)..."; \
		"$$SFETCH_BIN" -repo fulmenhq/goneat -tag "$(GONEAT_VERSION)" -dest-dir "$(BINDIR)"; \
	else \
		if ! command -v goneat >/dev/null 2>&1 && [ ! -x "$(BINDIR)/goneat" ]; then \
			echo "→ Installing goneat $(GONEAT_VERSION) into $(BINDIR)..."; \
			"$$SFETCH_BIN" -repo fulmenhq/goneat -tag "$(GONEAT_VERSION)" -dest-dir "$(BINDIR)"; \
		else \
			echo "✅ goneat already installed: $$(goneat version 2>&1 | head -1)"; \
		fi; \
	fi
	@echo ""
	@echo "Step 3: Verifying Rust toolchain..."
	@cargo --version >/dev/null 2>&1 || (echo "❌ Cargo not found. Please install Rust: https://rustup.rs" && exit 1)
	@echo "✅ cargo: $$(cargo --version)"
	@echo "✅ rustc: $$(rustc --version)"
	@echo ""
	@echo "Step 4: Installing Rust development tools..."
	@$(MAKE) bootstrap-cargo-tools
	@echo ""
	@echo "Step 5: Installing foundation tools via goneat..."
	@GONEAT_BIN="$$(command -v goneat 2>/dev/null || true)"; \
	if [ -z "$$GONEAT_BIN" ] && [ -x "$(BINDIR)/goneat" ]; then GONEAT_BIN="$(BINDIR)/goneat"; fi; \
	if [ -n "$$GONEAT_BIN" ]; then \
		"$$GONEAT_BIN" doctor tools --scope foundation --install --yes --no-cooling 2>/dev/null || \
		echo "⚠️  Some foundation tools may need manual installation"; \
	fi
	@echo ""
	@echo "✅ Bootstrap completed"
	@echo ""
	@echo "💡 Ensure $(BINDIR) is in your PATH:"
	@echo "   export PATH=\"$(BINDIR):\$$PATH\""

bootstrap-cargo-tools: ## Install Rust cargo tools
	@echo "Installing cargo development tools..."
	@cargo install cargo-tarpaulin 2>/dev/null || echo "⚠️  cargo-tarpaulin may already be installed or failed"
	@cargo install cargo-audit 2>/dev/null || echo "⚠️  cargo-audit may already be installed or failed"
	@cargo install cargo-deny 2>/dev/null || echo "⚠️  cargo-deny may already be installed or failed"
	@cargo install cargo-msrv 2>/dev/null || echo "⚠️  cargo-msrv may already be installed or failed"
	@echo "✅ Cargo tools installed"

bootstrap-force: ## Force reinstall dependencies and external tools
	@$(MAKE) bootstrap FORCE=1

tools: ## Verify external tools are available
	@echo "Verifying external tools..."
	@echo ""
	@echo "Core tools:"
	@if command -v sfetch >/dev/null 2>&1; then \
		echo "✅ sfetch: $$(sfetch -version 2>&1 | head -n1)"; \
	elif [ -x "$(BINDIR)/sfetch" ]; then \
		echo "✅ sfetch: $$($(BINDIR)/sfetch -version 2>&1 | head -n1)"; \
	else \
		echo "⚠️  sfetch not found (optional for day-to-day; required for bootstrap)"; \
	fi
	@goneat version > /dev/null 2>&1 && echo "✅ goneat: $$(goneat version 2>&1 | head -1)" || (echo "❌ goneat not found. Run 'make bootstrap' first." && exit 1)
	@echo ""
	@echo "Rust toolchain:"
	@cargo --version > /dev/null && echo "✅ cargo: $$(cargo --version)" || (echo "❌ cargo not found" && exit 1)
	@rustc --version > /dev/null && echo "✅ rustc: $$(rustc --version)" || (echo "❌ rustc not found" && exit 1)
	@echo ""
	@echo "Cargo tools:"
	@cargo tarpaulin --version > /dev/null 2>&1 && echo "✅ cargo-tarpaulin: $$(cargo tarpaulin --version 2>&1 | head -1)" || echo "⚠️  cargo-tarpaulin not installed (needed for coverage)"
	@cargo audit --version > /dev/null 2>&1 && echo "✅ cargo-audit: $$(cargo audit --version 2>&1 | head -1)" || echo "⚠️  cargo-audit not installed (needed for security)"
	@cargo deny --version > /dev/null 2>&1 && echo "✅ cargo-deny: $$(cargo deny --version 2>&1 | head -1)" || echo "⚠️  cargo-deny not installed (needed for license/security)"
	@echo ""
	@echo "✅ Required tools present"

hooks-ensure: ## Ensure git hooks are installed (auto-installs if missing)
	@if [ -d .git ] && [ ! -x .git/hooks/pre-commit ]; then \
		if command -v goneat >/dev/null 2>&1; then \
			echo "🔗 Installing git hooks..."; \
			goneat hooks generate 2>/dev/null || true; \
			goneat hooks install 2>/dev/null || true; \
		fi; \
	fi

# SSOT sync targets
sync: sync-ssot ## Alias for sync-ssot

sync-ssot: ## Sync assets from Crucible SSOT
	@echo "Syncing Crucible assets..."
	@goneat ssot sync --force-remote
	@./scripts/sync-crucible-version.sh
	@echo "✅ Sync completed"

# Quality targets
lint: ## Run linting checks (clippy)
	@echo "Running clippy..."
	@cargo clippy --all-targets --all-features -- -D warnings
	@echo "✅ Linting passed"

fmt: ## Format code (rustfmt)
	@echo "Formatting code..."
	@cargo fmt
	@echo "✅ Formatting complete"

fmt-check: ## Check code formatting without changes
	@echo "Checking code format..."
	@cargo fmt -- --check
	@echo "✅ Format check passed"

test: ## Run all tests
	@echo "Running tests (lifecycle=$(LIFECYCLE), min coverage=$(COVERAGE_MIN)%)..."
	@cargo test --all-features
	@echo "✅ Tests passed"

test-cov: ## Run tests with coverage (requires cargo-tarpaulin)
	@echo "Running tests with coverage (lifecycle=$(LIFECYCLE), min=$(COVERAGE_MIN)%)..."
	@cargo tarpaulin --all-features --fail-under $(COVERAGE_MIN) --out Html --out Lcov
	@echo "✅ Coverage check passed"

lifecycle: ## Show current lifecycle phase and requirements
	@echo "Repository Lifecycle Phase: $(LIFECYCLE)"
	@echo "Required test coverage: $(COVERAGE_MIN)%"

msrv-check: ## Verify MSRV compatibility with dependencies (requires cargo-msrv)
	@echo "Checking MSRV compatibility..."
	@if command -v cargo-msrv >/dev/null 2>&1; then \
		cargo msrv verify 2>&1 || (echo "❌ MSRV check failed - dependencies require newer Rust than declared in Cargo.toml"; exit 1); \
		echo "✅ MSRV check passed"; \
	else \
		echo "⚠️  cargo-msrv not installed, skipping (run: make bootstrap-cargo-tools)"; \
	fi

check-all: fmt-check lint test ## Run all quality checks
	@echo "✅ All checks passed"

quality: build check-all ## Run build, lint, and tests
	@echo "✅ Quality checks complete"

# Build targets
build: ## Build library
	@echo "Building library..."
	@cargo build --all-features
	@$(MAKE) hooks-ensure
	@echo "✅ Build complete"

build-release: ## Build library in release mode
	@echo "Building library (release)..."
	@cargo build --release --all-features
	@echo "✅ Release build complete"

build-all: build ## Build all targets (N/A for library, delegates to build)
	@echo "✅ Build complete (library package - no platform-specific binaries)"

doc: ## Generate documentation
	@echo "Generating documentation..."
	@cargo doc --all-features --no-deps
	@echo "✅ Documentation generated in target/doc/"

# Version management
version: ## Print current version
	@echo "$(VERSION)"

version-set: ## Update VERSION (usage: make version-set VERSION=x.y.z)
	@test -n "$(VERSION)" || (echo "❌ VERSION not set. Use: make version-set VERSION=x.y.z" && exit 1)
	@goneat version set $(VERSION)
	@$(MAKE) version-propagate
	@echo "✅ Version set to $(VERSION) and propagated"

version-propagate: ## Sync VERSION to Cargo.toml
	@echo "Propagating version to Cargo.toml..."
	@if [ -f Cargo.toml ]; then \
		sed -i.bak 's/^version = ".*"/version = "$(VERSION)"/' Cargo.toml && rm -f Cargo.toml.bak; \
		echo "✅ Version propagated to Cargo.toml"; \
	else \
		echo "⚠️  Cargo.toml not found, skipping propagation"; \
	fi

version-bump-major: ## Bump major version
	@goneat version bump major
	@$(MAKE) version-propagate
	@echo "✅ Version bumped (major) and propagated"

version-bump-minor: ## Bump minor version
	@goneat version bump minor
	@$(MAKE) version-propagate
	@echo "✅ Version bumped (minor) and propagated"

version-bump-patch: ## Bump patch version
	@goneat version bump patch
	@$(MAKE) version-propagate
	@echo "✅ Version bumped (patch) and propagated"

version-bump-calver: ## Bump to CalVer (YYYY.0M.MICRO)
	@goneat version bump calver
	@$(MAKE) version-propagate
	@echo "✅ Version bumped (calver) and propagated"

# Release targets
release-check: check-all ## Validate release readiness
	@echo "Running release validation..."
	@test -f VERSION || (echo "❌ VERSION file missing" && exit 1)
	@test -f Cargo.toml || (echo "❌ Cargo.toml file missing" && exit 1)
	@if [ -n "$$(git status --porcelain 2>/dev/null)" ]; then echo "⚠️  Working tree has uncommitted changes"; fi
	@cargo publish --dry-run 2>/dev/null || echo "⚠️  cargo publish --dry-run failed (may be expected before first publish)"
	@echo "✅ Release checks passed"

release-prepare: sync-ssot release-check ## Prepare for release
	@echo "✅ Release prepared"

release-build: build-release ## Build release artifacts
	@echo "Building release artifacts..."
	@cargo package --allow-dirty
	@echo "✅ Release artifacts ready in target/package/"

release-clean: ## Remove local release artifacts (dist/release)
	@echo "Cleaning release artifacts..."
	@rm -rf dist/release
	@echo "✅ Release artifacts removed"

release-provenance-check: ## Verify Crucible SSOT provenance files exist
	@./scripts/release-provenance-check.sh

release-guard-tag-version: ## Guard: ensure tag matches VERSION (CI-friendly)
	@./scripts/release-guard-tag-version.sh

release-tag: ## Create and verify a signed git tag for VERSION
	@./scripts/release-tag.sh

release-verify-tag: ## Verify the signed git tag for VERSION
	@./scripts/release-verify-tag.sh

# Hook targets
precommit: ## Run pre-commit hooks (fast, critical issues only)
	@echo "Running pre-commit checks..."
	@echo ""
	@echo "Step 1: Rust formatting check..."
	@cargo fmt -- --check || (echo "❌ Run 'cargo fmt' to fix formatting" && exit 1)
	@echo "✅ Rust formatting OK"
	@echo ""
	@echo "Step 2: Rust clippy (quick lint)..."
	@cargo clippy --all-targets --all-features -- -D warnings
	@echo "✅ Clippy OK"
	@echo ""
	@echo "Step 3: Goneat assess (format, lint, security - fail on critical)..."
	@goneat assess --categories format,lint,security --fail-on critical --staged-only 2>/dev/null || \
		goneat assess --categories format,lint,security --fail-on critical
	@echo ""
	@echo "✅ Pre-commit checks passed"

prepush: ## Run pre-push hooks (comprehensive, fail on high severity)
	@echo "Running pre-push checks..."
	@echo ""
	@echo "Step 1: Rust formatting check..."
	@cargo fmt -- --check || (echo "❌ Run 'cargo fmt' to fix formatting" && exit 1)
	@echo "✅ Rust formatting OK"
	@echo ""
	@echo "Step 2: Rust clippy..."
	@cargo clippy --all-targets --all-features -- -D warnings
	@echo "✅ Clippy OK"
	@echo ""
	@echo "Step 3: Rust tests..."
	@cargo test --all-features
	@echo "✅ Tests OK"
	@echo ""
	@echo "Step 4: Goneat assess (format, lint, security - fail on high)..."
	@goneat assess --categories format,lint,security --fail-on high
	@echo ""
	@echo "✅ Pre-push checks passed"

# Clean targets
clean: ## Remove build artifacts
	@echo "Cleaning build artifacts..."
	@cargo clean
	@rm -rf target/
	@echo "✅ Clean complete"
