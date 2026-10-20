#!/usr/bin/env bash
# Create a GitHub release for the signed tag and attach the minisign
# attestation. Before calling `gh` it requires, in order:
#   - release-guard-tag-version (tag name matches VERSION)
#   - the tag exists, is annotated, and its target is HEAD
#   - a good tag signature (git verify-tag) from RSFULMEN_TAGGER_NAME/EMAIL
#   - the checklist assets, a valid minisign attestation, and attestation
#     fields tag / tag_object / tag_target equal to the live tag
#   - the remote tag (RSFULMEN_RELEASE_REMOTE, default origin) resolves to the
#     same tag object and commit
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/release-tagger.sh
. "${script_dir}/lib/release-tagger.sh"

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

# Print the value of `key:` from an attestation payload; fail unless the key
# appears exactly once.
payload_field() {
	local payload="$1" key="$2" count
	count="$(grep -cE "^[[:space:]]*${key}:" "${payload}" || true)"
	if [ "${count}" != "1" ]; then
		echo "error: attestation ${payload} must contain exactly one ${key}: line" >&2
		return 1
	fi
	sed -nE "s/^[[:space:]]*${key}:[[:space:]]*([^[:space:]]*)[[:space:]]*$/\1/p" "${payload}"
}

require_equal() {
	local what="$1" expected="$2" actual="$3"
	if [ -z "${actual}" ] || [ "${expected}" != "${actual}" ]; then
		echo "error: ${what} mismatch" >&2
		echo "  expected: ${expected}" >&2
		echo "  actual:   ${actual:-<missing>}" >&2
		exit 1
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
	if [ "$(git cat-file -t "refs/tags/${tag}")" != "tag" ]; then
		echo "error: tag ${tag} is not an annotated tag" >&2
		exit 1
	fi

	local tag_object tag_target head
	tag_object="$(git rev-parse "refs/tags/${tag}")"
	tag_target="$(git rev-parse "refs/tags/${tag}^{commit}")"
	head="$(git rev-parse HEAD)"
	require_equal "tag ${tag} target vs HEAD" "${head}" "${tag_target}"

	local gpg_homedir="${RSFULMEN_GPG_HOMEDIR:-}"
	if [ -n "${gpg_homedir}" ]; then
		if [ ! -d "${gpg_homedir}" ]; then
			echo "error: RSFULMEN_GPG_HOMEDIR=${gpg_homedir} is not a directory" >&2
			exit 1
		fi
		export GNUPGHOME="${gpg_homedir}"
	fi
	require_tagger_env
	echo "→ Verifying tag signature: ${tag}"
	if ! git verify-tag "${tag}" >/dev/null 2>&1; then
		echo "error: tag ${tag} has no valid signature (git verify-tag failed)" >&2
		exit 1
	fi
	verify_tag_identity "${tag}"
	echo "✅ Tag signature and tagger verified: ${tag}"

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

	local att_tag att_object att_target
	att_tag="$(payload_field "${payload}" tag)" || exit 1
	att_object="$(payload_field "${payload}" tag_object)" || exit 1
	att_target="$(payload_field "${payload}" tag_target)" || exit 1
	require_equal "attestation tag" "${tag}" "${att_tag}"
	require_equal "attestation tag_object" "${tag_object}" "${att_object}"
	require_equal "attestation tag_target" "${tag_target}" "${att_target}"
	echo "✅ Attestation matches tag ${tag} (${tag_object} -> ${tag_target})"

	local remote="${RSFULMEN_RELEASE_REMOTE:-origin}"
	local remote_refs remote_object remote_target
	if ! remote_refs="$(git ls-remote --tags "${remote}" "refs/tags/${tag}" "refs/tags/${tag}^{}")"; then
		echo "error: could not list tags on remote ${remote}" >&2
		exit 1
	fi
	remote_object="$(printf '%s\n' "${remote_refs}" | awk -v r="refs/tags/${tag}" '$2 == r {print $1}')"
	remote_target="$(printf '%s\n' "${remote_refs}" | awk -v r="refs/tags/${tag}^{}" '$2 == r {print $1}')"
	require_equal "remote ${remote} tag ${tag} object" "${tag_object}" "${remote_object}"
	require_equal "remote ${remote} tag ${tag} target" "${tag_target}" "${remote_target}"
	echo "✅ Remote ${remote} tag ${tag} matches"

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
