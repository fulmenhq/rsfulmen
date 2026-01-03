#!/usr/bin/env bash
set -euo pipefail

repo_root() {
	git rev-parse --show-toplevel
}

main() {
	local root
	root="$(repo_root)"
	cd "$root"

	local ssot_prov=".goneat/ssot/provenance.json"
	local crucible_meta=".crucible/metadata/metadata.yaml"

	if [ ! -f "$ssot_prov" ]; then
		echo "error: missing $ssot_prov (run 'make sync' first)" >&2
		exit 1
	fi

	if [ ! -f "$crucible_meta" ]; then
		echo "error: missing $crucible_meta (run 'make sync' first)" >&2
		exit 1
	fi

	echo "→ SSOT provenance: $ssot_prov"
	if command -v jq >/dev/null 2>&1; then
		jq -e . "$ssot_prov" >/dev/null
		jq -r '.sources[] | "\(.repo_url)@\(.ref) (\(.commit[0:7]))"' "$ssot_prov" | sed 's/^/  - /'
	else
		echo "  (jq not found; skipping JSON validation)"
	fi

	echo "→ Crucible metadata: $crucible_meta"
	if command -v yq >/dev/null 2>&1; then
		yq -e '.sources' "$crucible_meta" >/dev/null
		yq -r '.sources[] | (.name + ": " + (.version // "") + " " + (.commit // ""))' "$crucible_meta" | sed 's/^/  - /'
	else
		echo "  (yq not found; skipping YAML validation)"
	fi

	echo "✅ Provenance files present"
}

main "$@"
