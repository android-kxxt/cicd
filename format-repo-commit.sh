#!/bin/bash
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

echo -n "- \`$REPO_PATH\`: \`"
git rev-parse --short=12 m/lineage-22.2 | tr -d $'\n'
echo -n '` '
git status --porcelain=v1 | "$SCRIPT_DIR/git-dirty-status.rb"