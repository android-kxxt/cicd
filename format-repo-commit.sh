#!/bin/bash
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

format="$1"

if [[ "$format" == "simple" ]]; then
    echo -n "$REPO_PATH: "
else
    echo -n "- \`$REPO_PATH\`: \`"
fi

git rev-parse --short=12 m/lineage-23.0 | tr -d $'\n'

if [[ "$format" != "simple" ]]; then
    echo -n '` '
    git status --porcelain=v1 | "$SCRIPT_DIR/git-dirty-status.rb"
fi
