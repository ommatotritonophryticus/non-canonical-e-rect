#!/usr/bin/env bash
#
# Prints one release's section of CHANGELOG.md - what the release workflow
# publishes as that release's notes - or, with --title, the release's name.
#
#   tools/release-notes.sh v0.3            the section headed "## 0.3 ..."
#   tools/release-notes.sh --title v0.3    "E-Rect 0.3 - Head Start"
#
# The section's heading is the version, then optionally a name for the
# release: "## 0.3 - Head Start". The title is "E-Rect <version>", plus " - "
# and that name when there is one.
#
# Run it before pushing a tag to see exactly what the release page will say. It
# fails, and says so, when there is no section for the version: the workflow
# then falls back to GitHub's own notes rather than publishing an empty page.

set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
title=0
if [ "${1:-}" = --title ]; then
    title=1
    shift
fi
version=${1:?usage: tools/release-notes.sh [--title] <tag or version>}
version=${version#v}

heading=$(awk -v want="$version" '/^## / && $2 == want { print; exit }' "$ROOT/CHANGELOG.md")
if [ -z "$heading" ]; then
    echo "CHANGELOG.md has no section for $version" >&2
    exit 1
fi

if [ "$title" = 1 ]; then
    # "## 0.3 - Head Start" -> "Head Start"; "## 0.3" -> nothing.
    name=$(printf '%s\n' "$heading" | sed -E 's/^## +[^ ]+ *//; s/^- *//')
    if [ -n "$name" ]; then
        printf 'E-Rect %s - %s\n' "$version" "$name"
    else
        printf 'E-Rect %s\n' "$version"
    fi
    exit 0
fi

notes=$(awk -v want="$version" '
    /^## / { if (found) exit; found = ($2 == want); next }
    found { print }
' "$ROOT/CHANGELOG.md")

# Blank lines at either end are layout, not content.
notes=$(printf '%s\n' "$notes" | sed -e '/./,$!d')
if [ -z "$notes" ]; then
    echo "CHANGELOG.md has no section for $version" >&2
    exit 1
fi
printf '%s\n' "$notes"
