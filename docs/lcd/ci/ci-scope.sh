#!/usr/bin/env bash
# ci-scope.sh — the scope of a CI run (D-071), written to the step's outputs. Both jobs in ci.yml
# call it, so they cannot disagree about what a run covers.
#
# A pull request is scoped to what it changes against its base branch, a push to what it added to
# the branch, and anything else (a manual "Run workflow") runs everything. Needs the full history:
# check out with `fetch-depth: 0`.
#
# Reads GITHUB_EVENT_NAME, GITHUB_BASE_REF and GITHUB_OUTPUT, which the runner sets, and BEFORE,
# which the step passes from `github.event.before`.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"

case "${GITHUB_EVENT_NAME:-}" in
  pull_request)
    scope="$("$here/scope.sh" "origin/$GITHUB_BASE_REF")"
    ;;
  push)
    # A first push has no commit before it, and a forced one may name a commit that is gone.
    if git cat-file -e "${BEFORE:-}^{commit}" 2>/dev/null; then
      scope="$("$here/scope.sh" "$BEFORE")"
    else
      scope="$("$here/scope.sh" --all)"
    fi
    ;;
  *)
    scope="$("$here/scope.sh" --all)"
    ;;
esac

echo "$scope"
echo "$scope" >>"$GITHUB_OUTPUT"
