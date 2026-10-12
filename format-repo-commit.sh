#!/bin/bash
set -euo pipefail

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

format="${1:-}"

# repo forall provides the manifest revision's local ref, regardless of the
# LineageOS version or a project's branch override.
commit=$(git rev-parse --verify --short=12 "${REPO_LREV:?Run this script with repo forall}^{commit}")

if [[ "$format" == "simple" ]]; then
    printf '%s: %s\n' "$REPO_PATH" "$commit"
else
    printf -- '- `%s`: `%s` ' "$REPO_PATH" "$commit"
    git status --porcelain=v1 | "$SCRIPT_DIR/git-dirty-status.rb"
fi
