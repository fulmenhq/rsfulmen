#!/usr/bin/env bash
# Shared signed-tag identity helpers. Sourced by release-tag.sh and
# release-verify-tag.sh. Identity comes from required env; nothing is
# hard-coded here.

require_tagger_env() {
	if [ -z "${RSFULMEN_TAGGER_NAME:-}" ]; then
		echo "error: RSFULMEN_TAGGER_NAME is required" >&2
		echo "hint: see RELEASE_CHECKLIST.md (Variables)" >&2
		exit 1
	fi
	if [ -z "${RSFULMEN_TAGGER_EMAIL:-}" ]; then
		echo "error: RSFULMEN_TAGGER_EMAIL is required" >&2
		echo "hint: see RELEASE_CHECKLIST.md (Variables)" >&2
		exit 1
	fi
}

verify_tag_identity() {
	local tag="$1"
	local tagger_name tagger_email

	tagger_name="$(git for-each-ref --format='%(taggername)' "refs/tags/${tag}")"
	tagger_email="$(git for-each-ref --format='%(taggeremail)' "refs/tags/${tag}")"
	tagger_email="${tagger_email#<}"
	tagger_email="${tagger_email%>}"

	if [ "${tagger_name}" != "${RSFULMEN_TAGGER_NAME}" ] || [ "${tagger_email}" != "${RSFULMEN_TAGGER_EMAIL}" ]; then
		echo "error: tag object tagger does not match RSFULMEN_TAGGER_NAME / RSFULMEN_TAGGER_EMAIL" >&2
		exit 1
	fi
}
