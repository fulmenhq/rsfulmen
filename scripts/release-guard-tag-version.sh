#!/usr/bin/env bash
set -euo pipefail

repo_root() {
	git rev-parse --show-toplevel
}

read_version() {
	if [ ! -f VERSION ]; then
		echo "error: VERSION file not found" >&2
		exit 1
	fi
	tr -d ' \t\r\n' <VERSION
}

detect_tag() {
	if [ -n "${RSFULMEN_RELEASE_TAG:-}" ]; then
		printf '%s' "${RSFULMEN_RELEASE_TAG}"
		return 0
	fi
	git describe --tags --exact-match 2>/dev/null || true
}

main() {
	local root
	root="$(repo_root)"
	cd "$root"

	local version
	version="$(read_version)"

	local expected="v${version}"
	local tag
	tag="$(detect_tag)"

	if [ -z "$tag" ]; then
		if [ "${RSFULMEN_ALLOW_UNTAGGED:-}" = "1" ]; then
			echo "→ release guard: no tag detected (RSFULMEN_ALLOW_UNTAGGED=1, skipping)"
			exit 0
		fi
		echo "error: no exact tag found for HEAD and no RSFULMEN_RELEASE_TAG provided" >&2
		echo "hint: set RSFULMEN_ALLOW_UNTAGGED=1 for local dry-runs" >&2
		exit 1
	fi

	if [ "$tag" != "$expected" ]; then
		echo "error: release tag/version mismatch" >&2
		echo "  tag:     $tag" >&2
		echo "  VERSION: $version (expected tag: $expected)" >&2
		exit 1
	fi

	echo "✅ release guard: tag matches VERSION ($tag)"
}

main "$@"
