#!/usr/bin/env bash
#
# Prints one release's section of CHANGELOG.md - what the release workflow
# publishes as that release's notes.
#
#   tools/release-notes.sh v0.3      the section headed "## 0.3"
#
# Run it before pushing a tag to see exactly what the release page will say. It
# fails, and says so, when there is no section for the version: the workflow
# then falls back to GitHub's own notes rather than publishing an empty page.

set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
version=${1:?usage: tools/release-notes.sh <tag or version>}
version=${version#v}

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
