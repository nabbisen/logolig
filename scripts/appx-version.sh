#!/usr/bin/env bash
# appx-version.sh: the Microsoft Store manifest version is derived from the workspace version (RFC 037, phase A).
# Usage: scripts/appx-version.sh check|update [REPOSITORY_ROOT]   (default: the repository this script is in; any directory)
# The rule: the `Version` attribute of the <Identity> element of packaging/windows/AppxManifest.xml (not MinVersion or
# MaxVersionTested, which are other attributes) is `<version of [workspace.package] in Cargo.toml>.0`; the Store owns the
# fourth part. `check` exits 0 only when that holds; `update` rewrites that one attribute value, nothing else (idempotent).
# grep, sed and awk only. The optional ROOT exists so that the test can point the script at throw-away trees.
set -u
MODE=${1:-}
ROOT=$(cd "${2:-$(dirname "${BASH_SOURCE[0]}")/..}" && pwd) || exit 2
REL=packaging/windows/AppxManifest.xml
MANIFEST="$ROOT/$REL"
die() { echo "$1: $2" >&2; exit 1; }

# The three-part version of [workspace.package]; anything else is an error.
workspace_version() {
  local v
  v=$(awk '/^\[/ { in_pkg = ($0 ~ /^\[workspace\.package\][[:space:]]*$/) } in_pkg && /^version[[:space:]]*=/ { print; exit }' "$ROOT/Cargo.toml" 2>/dev/null |
    sed -n 's/^version[[:space:]]*=[[:space:]]*"\([0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*\)"[[:space:]]*\(#.*\)\{0,1\}$/\1/p')
  [ -n "$v" ] || die Cargo.toml 'no three-part numeric `version = "X.Y.Z"` in [workspace.package]'
  echo "$v"
}

# The Version attribute of the <Identity> element (the element may span lines).
manifest_version() {
  local v
  [ -f "$MANIFEST" ] || die "$REL" "the manifest does not exist"
  v=$(tr '\n' ' ' < "$MANIFEST" | grep -o '<Identity[^>]*>' | sed -n 's/.*[[:space:]]Version="\([^"]*\)".*/\1/p')
  [ -n "$v" ] || die "$REL" "the <Identity> element has no Version attribute"
  echo "$v"
}

case $MODE in
  check|update) ;;
  *) echo "usage: $0 check|update [REPOSITORY_ROOT]" >&2; exit 2 ;;
esac
want="$(workspace_version).0" || exit 1
have=$(manifest_version) || exit 1
if [ "$have" = "$want" ]; then echo "appx-version: ok ($want)"; exit 0; fi
if [ "$MODE" = check ]; then
  die "$REL" "Identity Version is $have, expected $want (run: bash scripts/appx-version.sh update)"
fi
# update: only the value of the Identity element's Version attribute (the whole file is one record for sed -z).
sed -z -i "s/\(<Identity[^>]*[[:space:]]Version=\"\)[^\"]*\"/\1$want\"/" "$MANIFEST"
echo "appx-version: updated $have -> $(manifest_version)"
