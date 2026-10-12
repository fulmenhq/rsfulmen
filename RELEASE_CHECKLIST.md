# Release Checklist (rsfulmen)

rsfulmen is a pure library crate: releases are primarily **signed git tags** (`vX.Y.Z`) consumed via
crates.io. We do not ship binaries.

This checklist follows the shared ecosystem pattern (gofulmen/pyfulmen/tsfulmen/rsfulmen) codified in Crucible.

## Variables (Quick Reference)

- `RSFULMEN_RELEASE_TAG`: optional override tag (recommended for manual release commands; e.g. `v0.1.30`)
  - If unset, scripts default to `v$(cat VERSION)`.
  - `make release-publish` accepts it only when `make release-guard-tag-version` passes (the tag must match `VERSION`).
- `RSFULMEN_RELEASE_DIST`: optional directory for attestation assets (default `dist/release`)
- `RSFULMEN_TAGGER_NAME`: **required** annotated-tag tagger name for `make release-tag` and `make release-verify-tag`
- `RSFULMEN_TAGGER_EMAIL`: **required** annotated-tag tagger email for those targets; must be a verified email on the GitHub account that holds the signing public key
- `RSFULMEN_GPG_HOMEDIR`: recommended dedicated signing keyring directory (separate from personal `~/.gnupg`)
- `RSFULMEN_PGP_KEY_ID`: optional key id/email/fingerprint for signing
- `RSFULMEN_MINISIGN_KEY`: optional minisign secret key path (creates a sidecar signature for the tag attestation)
- `RSFULMEN_MINISIGN_PUB`: minisign public key path. Optional for `make release-tag` / `make release-verify-tag`. Required by `make release-publish` so the attestation is verified before upload.
- `RSFULMEN_ALLOW_NON_MAIN=1`: optional override to tag from a non-`main` branch (not recommended)
- `RSFULMEN_ALLOW_UNTAGGED=1`: skip tag requirement in guard check (for local dry-runs only). `make release-publish` rejects this override.

Note: `RSFULMEN_RELEASE_TAG` is not a secret and typically isn't stored in encrypted env bundles.

## Pre-Release

- [ ] Optional: start from a clean slate (removes `dist/release`):
  ```bash
  make release-clean
  ```
- [ ] `git status` is clean
- [ ] `make sync` completed and provenance reviewed:
  - [ ] `.goneat/ssot/provenance.json` is present/current
  - [ ] `.crucible/metadata/metadata.yaml` is present/current
  - [ ] Run `make release-provenance-check`
- [ ] Quality gates pass: `make check-all`
- [ ] `CHANGELOG.md` updated (Unreleased → new section)
- [ ] Create/update `docs/releases/vX.Y.Z.md` release document
- [ ] Update `RELEASE_NOTES.md` (add new release, rotate oldest per 3-release policy)
- [ ] Set version and propagate to `Cargo.toml`:
  ```bash
  make version-set VERSION=X.Y.Z
  ```
- [ ] Verify version sync:
  ```bash
  make version
  grep '^version = ' Cargo.toml | head -1
  ```
- [ ] Guard: ensure tag/version match:
  ```bash
  make release-guard-tag-version
  ```

## Tagging (Signed Tag Required)

- [ ] Export the required tagger identity (scripts refuse to tag or verify without both):
  ```bash
  export RSFULMEN_TAGGER_NAME
  export RSFULMEN_TAGGER_EMAIL
  ```
- [ ] Ensure interactive GPG signing can prompt for passphrase (recommended):
  ```bash
  export GPG_TTY="$(tty)"
  gpg-connect-agent updatestartuptty /bye
  ```
- [ ] Sanity checks (CI-friendly):
  ```bash
  make release-guard-tag-version
  make release-provenance-check
  ```
- [ ] Create the signed tag (this is _release process_ surface, not app code surface):
  - Signs an annotated git tag for the crate version.
  - Does not upload release assets by default.
  - Does not publish public keys (GitHub verifies signatures using public keys attached to the signer account).
  ```bash
  make release-tag
  ```
- [ ] Verify the signed tag locally:
  ```bash
  make release-verify-tag
  # or:
  git tag -v v$(cat VERSION)
  ```
- [ ] Push:
  ```bash
  git push origin main
  git push origin v$(cat VERSION)
  ```

## Post-Release

- [ ] Publish to crates.io (after tag is pushed):
  ```bash
  cargo publish
  ```
  **Note:** Requires authentication. For local publishing, run `cargo login` and paste your
  token from https://crates.io/settings/tokens. For CI/CD workflows, set the
  `CARGO_REGISTRY_TOKEN` secret/environment variable.
- [ ] Spot-check downstream consumption:
  ```bash
  cargo search rsfulmen
  ```
- [ ] Optional: remove local release artifacts:
  ```bash
  make release-clean
  ```
- [ ] Optional: show consumers how to verify the tag signature:
  - [ ] **Local git** (most reliable):
    ```bash
    git fetch --tags origin
    git tag -v v$(cat VERSION)
    ```
  - [ ] **GitHub API (CI-friendly)**:
    ```bash
    TAG_SHA=$(gh api repos/fulmenhq/rsfulmen/git/ref/tags/v$(cat VERSION) --jq .object.sha)
    gh api repos/fulmenhq/rsfulmen/git/tags/$TAG_SHA --jq .verification
    ```
  - [ ] **GitHub Web UI (note)**: a green "Verified" badge only appears if the signing public key is uploaded to the GitHub account and `RSFULMEN_TAGGER_EMAIL` matches a verified email on that account. Otherwise GitHub may show "Unverified" even though `git tag -v` succeeds.
- [ ] Publish the GitHub release with the minisign attestation:
  - `make release-tag` writes `dist/release/vX.Y.Z.tag.txt` and `.minisig` when `RSFULMEN_MINISIGN_KEY` and `RSFULMEN_MINISIGN_PUB` are set.
  - `make release-publish` runs `make release-guard-tag-version`, verifies the attestation with `RSFULMEN_MINISIGN_PUB`, and creates the release. It fails before `gh` if any required asset is missing or empty:
    - `CHANGELOG.md` (release notes)
    - `dist/release/<tag>.tag.txt`
    - `dist/release/<tag>.tag.txt.minisig`
    ```bash
    make release-publish
    ```
    Equivalent upload, once those checks have passed:
    ```bash
    gh release create v$(cat VERSION) --notes-file CHANGELOG.md dist/release/v$(cat VERSION).tag.txt dist/release/v$(cat VERSION).tag.txt.minisig
    ```
- [ ] Announce / coordinate downstream upgrades as needed.
