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

# github_repo_from_url URL: print owner/name for a github.com remote URL.
github_repo_from_url() {
	printf '%s\n' "$1" | sed -nE \
		-e 's#^(https|ssh)://([^@/]+@)?github\.com(:[0-9]+)?/([^/]+/[^/]+)$#\4#p' \
		-e 's#^[^@/]+@github\.com:([^/]+/[^/]+)$#\1#p' |
		sed -E 's#\.git$##; s#/$##'
}

# resolve_release_repo REMOTE: the one GitHub repository that both the remote
# tag check and gh target. RSFULMEN_RELEASE_REPO (owner/name) wins; otherwise
# it comes from the remote URL. Any conflicting gh selector is refused.
resolve_release_repo() {
	local remote="$1" url derived repo
	url="$(git remote get-url "${remote}" 2>/dev/null)" || {
		echo "error: remote ${remote} not configured" >&2
		return 1
	}
	derived="$(github_repo_from_url "${url}")"
	repo="${RSFULMEN_RELEASE_REPO:-${derived}}"
	if [ -z "${repo}" ]; then
		echo "error: cannot derive a github.com repository from remote ${remote}; set RSFULMEN_RELEASE_REPO=owner/name" >&2
		return 1
	fi
	if ! printf '%s\n' "${repo}" | grep -Eq '^[A-Za-z0-9._-]+/[A-Za-z0-9._-]+$'; then
		echo "error: release repository must be owner/name: ${repo}" >&2
		return 1
	fi
	if [ -n "${derived}" ] && [ "${derived}" != "${repo}" ]; then
		echo "error: RSFULMEN_RELEASE_REPO=${repo} differs from remote ${remote} (${derived})" >&2
		return 1
	fi
	if [ -n "${GH_REPO:-}" ] && [ "${GH_REPO}" != "${repo}" ] && [ "${GH_REPO}" != "github.com/${repo}" ]; then
		echo "error: GH_REPO=${GH_REPO} differs from release repository ${repo}" >&2
		return 1
	fi
	if [ -n "${GH_HOST:-}" ] && [ "${GH_HOST}" != "github.com" ]; then
		echo "error: GH_HOST=${GH_HOST} is not github.com" >&2
		return 1
	fi
	printf '%s\n' "${repo}"
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

	local gpg_homedir="${RSFULMEN_GPG_HOMEDIR:-${RSFULMEN_GPG_HOME:-}}"
	if [ -n "${RSFULMEN_GPG_HOME:-}" ] && [ -z "${RSFULMEN_GPG_HOMEDIR:-}" ]; then
		echo "warning: RSFULMEN_GPG_HOME is deprecated; use RSFULMEN_GPG_HOMEDIR" >&2
	fi
	if [ -n "${gpg_homedir}" ]; then
		if [ ! -d "${gpg_homedir}" ]; then
			echo "error: RSFULMEN_GPG_HOMEDIR=${gpg_homedir} is not a directory" >&2
			exit 1
		fi
		export GNUPGHOME="${gpg_homedir}"
	fi
	require_tagger_env
	echo "→ Verifying tag signature: ${tag}"
	if ! git verify-tag "refs/tags/${tag}" >/dev/null 2>&1; then
		echo "error: tag ${tag} has no valid signature (git verify-tag failed; check RSFULMEN_GPG_HOMEDIR)" >&2
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
	if ! minisign -Vm "${payload}" -p "${RSFULMEN_MINISIGN_PUB}" >/dev/null; then
		echo "error: minisign verification failed for ${payload}" >&2
		exit 1
	fi
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
	local repo
	repo="$(resolve_release_repo "${remote}")" || exit 1
	echo "→ Release repository: ${repo} (remote ${remote})"

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

	# Bind the destination too: the tag gh will release must be the same
	# annotated tag object (and so the same commit) that was verified.
	local dest_ref
	if ! dest_ref="$(gh api "repos/${repo}/git/ref/tags/${tag}" --jq '.object.type + " " + .object.sha')"; then
		echo "error: tag ${tag} not found in GitHub repository ${repo}" >&2
		exit 1
	fi
	require_equal "GitHub ${repo} tag ${tag}" "tag ${tag_object}" "${dest_ref}"
	echo "✅ GitHub ${repo} tag ${tag} matches"

	if gh release view "${tag}" --repo "${repo}" >/dev/null 2>&1; then
		echo "error: GitHub release ${tag} already exists" >&2
		exit 1
	fi

	echo "→ Creating GitHub release: ${tag}"
	gh release create "${tag}" --repo "${repo}" --verify-tag --notes-file "${notes}" "${payload}" "${sig}"
	echo "✅ Published GitHub release: ${tag}"
}

main "$@"
