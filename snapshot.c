#!/bin/bash
//$(which true);SCRIPT_DIR="$(cd "$(dirname "$0")"; pwd -P)";
//$(which true);INPUT_FILE="$SCRIPT_DIR/$(basename "$0")";
//$(which true);EXE_FILE="$SCRIPT_DIR/$(basename "$0" .c).exe";
//$(which true);test "$INPUT_FILE" -ot "$EXE_FILE" || tail -n +2 "$0" | gcc $CFLAGS -x c -DSCRIPT_DIR="$SCRIPT_DIR" - -o "$EXE_FILE" || exit $?
//$(which true);exec -a "$0" "$EXE_FILE" "$@"
#include <stdio.h>
#include <unistd.h>

#define xstr(s) str(s)
#define str(s) #s

char *argv[5] = {"repo", "forall", "-c",
                xstr(SCRIPT_DIR) "/format-repo-commit.sh", "simple", NULL};

int main() {
  execvp("repo", argv);
  fprintf(stderr, "Failed to execute repo.\n");
  return 1;
}
