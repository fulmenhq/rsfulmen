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

setup_gpg_tty() {
	# When using passphrase-protected keys, gpg will invoke pinentry.
	# Ensure it has a real TTY to talk to, otherwise signing fails with:
	# "Inappropriate ioctl for device".
	if [ ! -t 0 ] || [ ! -t 1 ]; then
		echo "error: no TTY available for interactive gpg signing" >&2
		echo "hint: run make release-tag in an interactive terminal" >&2
		echo "hint: export GPG_TTY=\"$(tty)\" && gpg-connect-agent updatestartuptty /bye" >&2
		exit 1
	fi

	if command -v tty >/dev/null 2>&1; then
		local tty_path
		tty_path="$(tty 2>/dev/null || true)"
		if [ -n "${tty_path}" ] && [ "${tty_path}" != "not a tty" ]; then
			export GPG_TTY="${tty_path}"
			gpg-connect-agent updatestartuptty /bye >/dev/null 2>&1 || true
		fi
	fi
}

ensure_gpg_signing_ready() {
	if ! command -v gpg >/dev/null 2>&1; then
		echo "error: gpg not found in PATH (required for signed tags)" >&2
		echo "hint: see RELEASE_CHECKLIST.md (Tagging section)" >&2
		exit 1
	fi

	local key_id="${RSFULMEN_PGP_KEY_ID:-}"
	local listing

	if [ -n "${key_id}" ]; then
		listing="$(gpg --list-secret-keys --with-colons --keyid-format=long "${key_id}" 2>/dev/null || true)"
		if ! echo "${listing}" | grep -q '^sec'; then
			echo "error: no usable GPG secret key found for RSFULMEN_PGP_KEY_ID=${key_id}" >&2
			echo "hint: ensure your release env vars are loaded (shell restart is common)" >&2
			if [ -n "${GNUPGHOME:-}" ]; then
				echo "hint: GNUPGHOME=${GNUPGHOME}" >&2
			else
				echo "hint: set RSFULMEN_GPG_HOMEDIR to your signing keyring directory" >&2
			fi
			echo "hint: run: gpg --list-secret-keys --keyid-format=long" >&2
			echo "hint: see RELEASE_CHECKLIST.md (Tagging section)" >&2
			exit 1
		fi
		return 0
	fi

	listing="$(gpg --list-secret-keys --with-colons --keyid-format=long 2>/dev/null || true)"
	if ! echo "${listing}" | grep -q '^sec'; then
		echo "error: no usable GPG secret key found for signed tag creation" >&2
		echo "hint: ensure your release env vars are loaded (RSFULMEN_GPG_HOMEDIR/RSFULMEN_PGP_KEY_ID)" >&2
		if [ -n "${GNUPGHOME:-}" ]; then
			echo "hint: GNUPGHOME=${GNUPGHOME}" >&2
		else
			echo "hint: set RSFULMEN_GPG_HOMEDIR to your signing keyring directory" >&2
		fi
		echo "hint: run: gpg --list-secret-keys --keyid-format=long" >&2
		echo "hint: see RELEASE_CHECKLIST.md (Tagging section)" >&2
		exit 1
	fi
}
main() {
	local root
	root="$(repo_root)"
	cd "$root"

	local version
	version="$(read_version)"

	local tag="${RSFULMEN_RELEASE_TAG:-v${version}}"

	if ! [[ "$tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
		echo "error: invalid release tag '$tag' (expected vMAJOR.MINOR.PATCH)" >&2
		exit 1
	fi

	if [ -n "$(git status --porcelain)" ]; then
		echo "error: working tree is not clean (commit or stash changes before tagging)" >&2
		git status --porcelain >&2
		exit 1
	fi

	local branch
	branch="$(git branch --show-current 2>/dev/null || true)"
	if [ "$branch" != "main" ] && [ "${RSFULMEN_ALLOW_NON_MAIN:-}" != "1" ]; then
		echo "error: refusing to tag from branch '$branch' (set RSFULMEN_ALLOW_NON_MAIN=1 to override)" >&2
		exit 1
	fi

	if git rev-parse -q --verify "refs/tags/$tag" >/dev/null; then
		echo "error: tag $tag already exists" >&2
		exit 1
	fi

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

	if [ -n "${RSFULMEN_PGP_KEY_ID:-}" ] && [ -z "${gpg_homedir}" ]; then
		echo "error: RSFULMEN_PGP_KEY_ID is set but RSFULMEN_GPG_HOMEDIR is not; set a dedicated signing homedir" >&2
		exit 1
	fi

	setup_gpg_tty
	ensure_gpg_signing_ready

	echo "→ Creating signed tag: $tag"

	local tag_err
	tag_err="$(mktemp)"

	if [ -n "${RSFULMEN_PGP_KEY_ID:-}" ]; then
		if ! git tag -s -a "$tag" -u "${RSFULMEN_PGP_KEY_ID}" -m "Release $tag" 2>"${tag_err}"; then
			cat "${tag_err}" >&2
			if grep -qi "no secret key" "${tag_err}"; then
				echo "hint: no secret key available for signing (check RSFULMEN_GPG_HOMEDIR/RSFULMEN_PGP_KEY_ID)" >&2
				echo "hint: see RELEASE_CHECKLIST.md (Tagging section)" >&2
			fi
			rm -f "${tag_err}"
			exit 1
		fi
	else
		if ! git tag -s -a "$tag" -m "Release $tag" 2>"${tag_err}"; then
			cat "${tag_err}" >&2
			if grep -qi "no secret key" "${tag_err}"; then
				echo "hint: no secret key available for signing (check RSFULMEN_GPG_HOMEDIR/RSFULMEN_PGP_KEY_ID)" >&2
				echo "hint: see RELEASE_CHECKLIST.md (Tagging section)" >&2
			fi
			rm -f "${tag_err}"
			exit 1
		fi
	fi

	rm -f "${tag_err}"

	echo "→ Verifying tag signature: $tag"
	git verify-tag "$tag" >/dev/null

	echo "✅ Created and verified signed tag: $tag"

	# Optional: produce a minisign signature for a deterministic tag attestation.
	# This does NOT modify the git tag object; it creates a sidecar signature
	# that can be uploaded as a release asset.
	if [ -n "${RSFULMEN_MINISIGN_KEY:-}" ] || [ -n "${RSFULMEN_MINISIGN_PUB:-}" ]; then
		if ! command -v minisign >/dev/null 2>&1; then
			echo "error: minisign requested but not found in PATH" >&2
			exit 1
		fi
		if [ -z "${RSFULMEN_MINISIGN_KEY:-}" ] || [ -z "${RSFULMEN_MINISIGN_PUB:-}" ]; then
			echo "error: minisign requires both RSFULMEN_MINISIGN_KEY and RSFULMEN_MINISIGN_PUB" >&2
			exit 1
		fi

		local out_dir="dist/release"
		mkdir -p "${out_dir}"
		local payload="${out_dir}/${tag}.tag.txt"

		local tag_object
		tag_object="$(git rev-parse "${tag}^{tag}")"
		local tag_target
		tag_target="$(git rev-parse "${tag}^{}")"

		cat >"${payload}" <<EOF
	tag: ${tag}
	tag_object: ${tag_object}
	tag_target: ${tag_target}
EOF

		echo "→ Minisign tag attestation: ${payload}"
		minisign -Sm "${payload}" -s "${RSFULMEN_MINISIGN_KEY}"
		minisign -Vm "${payload}" -p "${RSFULMEN_MINISIGN_PUB}" >/dev/null
		echo "✅ Minisign signature verified: ${payload}.minisig"
	fi

	echo "Next:"
	echo "  git push origin main"
	echo "  git push origin $tag"
	if [ -f "dist/release/${tag}.tag.txt.minisig" ]; then
		echo "  # Optional: upload dist/release/${tag}.tag.txt* as release assets"
	fi
}

main "$@"
