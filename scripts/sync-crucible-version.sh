#!/usr/bin/env bash
set -euo pipefail

# sync-crucible-version.sh
#
# Updates the CRUCIBLE_VERSION constant in src/lib.rs to match
# the version from .goneat/ssot/provenance.json after sync.

repo_root() {
	git rev-parse --show-toplevel 2>/dev/null || pwd
}

main() {
	local root
	root="$(repo_root)"

	local provenance="$root/.goneat/ssot/provenance.json"
	local lib_rs="$root/src/lib.rs"

	if [ ! -f "$provenance" ]; then
		echo "❌ Provenance file not found: $provenance" >&2
		echo "   Run 'make sync' first" >&2
		exit 1
	fi

	if [ ! -f "$lib_rs" ]; then
		echo "❌ src/lib.rs not found: $lib_rs" >&2
		exit 1
	fi

	local version
	version="$(jq -r '.sources[0].ref' "$provenance")"

	if [ -z "$version" ] || [ "$version" = "null" ]; then
		echo "❌ Could not extract version from provenance" >&2
		exit 1
	fi

	local current
	current="$(grep -o 'CRUCIBLE_VERSION: &str = "[^"]*"' "$lib_rs" | sed 's/.*"\([^"]*\)".*/\1/')"

	if [ "$current" = "$version" ]; then
		echo "✅ CRUCIBLE_VERSION already up to date ($version)"
		return 0
	fi

	sed -i.bak "s/CRUCIBLE_VERSION: \&str = \"[^\"]*\"/CRUCIBLE_VERSION: \&str = \"$version\"/" "$lib_rs"
	rm -f "$lib_rs.bak"

	echo "✅ Updated CRUCIBLE_VERSION: $current → $version"
}

main "$@"
