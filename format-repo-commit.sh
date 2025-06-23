#!/bin/bash

echo -n "- \`$REPO_PATH\`: \`"
git rev-parse --short=12 m/lineage-22.2 | tr -d $'\n'
echo -n '`'