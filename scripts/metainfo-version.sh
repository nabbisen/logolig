#!/usr/bin/env bash
# metainfo-version.sh: the newest <release> of the Flathub MetaInfo is the workspace version (RFC 023, job F1).
# Usage: scripts/metainfo-version.sh check [REPOSITORY_ROOT]   (default: the repository this script is in; any directory)
# The rules, for packaging/linux/io.github.nabbisen.Logolig.metainfo.xml (XML comments are ignored):
#   * the first <release> under <releases> has version="<version of [workspace.package] in Cargo.toml>" (three numeric parts)
#     and a date="YYYY-MM-DD";
#   * no other <release> has a higher version (the list is newest first, as AppStream and Flathub want).
# `check` only: an entry needs a date and a description that no script can write for you. The fix is yours: add a
# <release version="X.Y.Z" date="YYYY-MM-DD"> entry at the top of <releases>. grep, sed and sort only.
set -u
MODE=${1:-}
ROOT=$(cd "${2:-$(dirname "${BASH_SOURCE[0]}")/..}" && pwd) || exit 2
REL=packaging/linux/io.github.nabbisen.Logolig.metainfo.xml
die() { echo "$1: $2" >&2; exit 1; }

workspace_version() {
  local v
  v=$(sed -n '/^\[workspace\.package\][[:space:]]*$/,/^\[/{s/^version[[:space:]]*=[[:space:]]*"\([0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*\)"[[:space:]]*\(#.*\)\{0,1\}$/\1/p}' "$ROOT/Cargo.toml" 2>/dev/null | head -1)
  [ -n "$v" ] || die Cargo.toml 'no three-part numeric `version = "X.Y.Z"` in [workspace.package]'
  echo "$v"
}

# The opening tags of every <release>, in file order, one per line (comments removed first).
release_tags() {
  [ -f "$ROOT/$REL" ] || die "$REL" "the MetaInfo does not exist"
  sed -z 's/<!--\([^-]\|-[^-]\)*-->//g' "$ROOT/$REL" | tr '\n' ' ' | { grep -o '<release[[:space:]][^>]*>' || true; }
}

[ "$MODE" = check ] || { echo "usage: $0 check [REPOSITORY_ROOT]" >&2; exit 2; }
want=$(workspace_version) || exit 1
tags=$(release_tags) || exit 1
[ -n "$tags" ] || die "$REL" "there is no <release> entry under <releases>"
first=$(head -1 <<<"$tags")
have=$(sed -n 's/.*[[:space:]]version="\([^"]*\)".*/\1/p' <<<"$first")
date=$(sed -n 's/.*[[:space:]]date="\([^"]*\)".*/\1/p' <<<"$first")
fix="add a <release version=\"$want\" date=\"YYYY-MM-DD\"> entry at the top of <releases>"
[ -n "$have" ] || die "$REL" "the first <release> has no version attribute"
[ "$have" = "$want" ] || die "$REL" "the newest <release> is $have, the workspace version is $want ($fix)"
grep -Eq '^[0-9]{4}-[0-9]{2}-[0-9]{2}$' <<<"$date" || die "$REL" "the newest <release> has no date=\"YYYY-MM-DD\" (found '${date:-nothing}')"
top=$(sed -n 's/.*[[:space:]]version="\([^"]*\)".*/\1/p' <<<"$tags" | sort -V | tail -1)
[ "$top" = "$want" ] || die "$REL" "a <release> with version $top is listed below the first: the entries must be newest first"
echo "metainfo-version: ok ($want)"
