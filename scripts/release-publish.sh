#!/usr/bin/env bash
# Create a GitHub release for the signed tag and attach the minisign
# attestation. Refuses to call `gh` unless release-guard-tag-version passes
# and the checklist assets are present.
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

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

require_asset() {
	local path="$1"
	if [ ! -s "${path}" ]; then
		echo "error: required release asset missing or empty: ${path}" >&2
		return 1
	fi
}

main() {
	local root
	root="$(repo_root)"
	cd "${root}"

	if [ "${RSFULMEN_ALLOW_UNTAGGED:-}" = "1" ]; then
		echo "error: RSFULMEN_ALLOW_UNTAGGED=1 is not valid for release-publish" >&2
		exit 1
	fi

	# RSFULMEN_RELEASE_TAG, when set, must match VERSION. Otherwise HEAD must
	# already be the VERSION tag. Same contract as make release-guard-tag-version.
	"${script_dir}/release-guard-tag-version.sh"

	local version tag
	version="$(read_version)"
	tag="${RSFULMEN_RELEASE_TAG:-v${version}}"

	local dist notes payload sig
	dist="${RSFULMEN_RELEASE_DIST:-dist/release}"
	notes="CHANGELOG.md"
	payload="${dist}/${tag}.tag.txt"
	sig="${payload}.minisig"

	local missing=0
	require_asset "${notes}" || missing=1
	require_asset "${payload}" || missing=1
	require_asset "${sig}" || missing=1
	if [ "${missing}" -ne 0 ]; then
		echo "hint: make release-tag writes ${dist}/<tag>.tag.txt and .minisig when minisign env is set" >&2
		exit 1
	fi

	if ! git rev-parse -q --verify "refs/tags/${tag}" >/dev/null; then
		echo "error: tag ${tag} does not exist locally" >&2
		echo "hint: create it with make release-tag before release-publish" >&2
		exit 1
	fi

	if [ -z "${RSFULMEN_MINISIGN_PUB:-}" ]; then
		echo "error: RSFULMEN_MINISIGN_PUB is required to verify the attestation before upload" >&2
		exit 1
	fi
	if [ ! -f "${RSFULMEN_MINISIGN_PUB}" ]; then
		echo "error: RSFULMEN_MINISIGN_PUB is not a file: ${RSFULMEN_MINISIGN_PUB}" >&2
		exit 1
	fi
	if ! command -v minisign >/dev/null 2>&1; then
		echo "error: minisign not found in PATH" >&2
		exit 1
	fi

	echo "→ Verifying minisign attestation: ${sig}"
	minisign -Vm "${payload}" -p "${RSFULMEN_MINISIGN_PUB}" >/dev/null
	echo "✅ Minisign attestation verified: ${sig}"

	if ! command -v gh >/dev/null 2>&1; then
		echo "error: gh not found in PATH" >&2
		exit 1
	fi

	if gh release view "${tag}" >/dev/null 2>&1; then
		echo "error: GitHub release ${tag} already exists" >&2
		exit 1
	fi

	echo "→ Creating GitHub release: ${tag}"
	gh release create "${tag}" --notes-file "${notes}" "${payload}" "${sig}"
	echo "✅ Published GitHub release: ${tag}"
}

main "$@"
