#!/usr/bin/env bash
# Decide whether GPU and browser qualification must run for this event.
#
# Pull requests whose every changed path is documentation-only may skip those
# jobs; pushes, manual runs and every other change always qualify. Paths read
# by tooling (crate/npm READMEs, docs/*.json, fixtures) never count as
# documentation. Prints `run=true|false` for $GITHUB_OUTPUT; reasons go to stderr.
#
# Usage: qualification-scope.sh <event-name> [path-list-file]
# Without a path list, pull requests diff the checked-out merge commit against
# its first parent (the base tip), which requires `fetch-depth: 2`.
set -euo pipefail

documentation_only() {
  local path count=0
  while IFS= read -r path; do
    [ -n "$path" ] || continue
    count=$((count + 1))
    case "$path" in
      docs/evidence/* | docs/reviews/*) ;;
      crates/* | packages/* | fixtures/* | examples/* | xtask/* | scripts/*)
        echo "qualification required by $path" >&2
        return 1
        ;;
      *.md) ;;
      *)
        echo "qualification required by $path" >&2
        return 1
        ;;
    esac
  done
  if [ "$count" -eq 0 ]; then
    echo "qualification required: no changed paths were found" >&2
    return 1
  fi
  echo "documentation-only change ($count paths); GPU and browser qualification skipped" >&2
}

event=${1:?usage: qualification-scope.sh <event-name> [path-list-file]}
if [ "$event" != pull_request ]; then
  echo "qualification required for $event events" >&2
  echo run=true
  exit 0
fi
if [ $# -ge 2 ]; then
  paths=$(cat "$2")
else
  # Renames are listed as deletion plus addition so moved code is never hidden.
  paths=$(git diff --no-renames --name-only HEAD^1 HEAD)
fi
if documentation_only <<<"$paths"; then
  echo run=false
else
  echo run=true
fi
