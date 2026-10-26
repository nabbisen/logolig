#!/usr/bin/env bash
# Render packaging/aur/logolig/PKGBUILD.in into <outdir>/PKGBUILD and <outdir>/.SRCINFO.
#
#   scripts/render-aur.sh logolig <version> <outdir> [--tarball <path>]
#
# Without --tarball the source is the GitHub tag archive, downloaded and hashed here.
# With --tarball the file is copied into <outdir> and used through a file:// URL (dry runs).
# This script never pushes and never writes a network resource.
set -euo pipefail

usage() { echo "usage: $0 logolig <version> <outdir> [--tarball <path>]" >&2; exit 2; }

[[ $# -ge 3 ]] || usage
package=$1 version=$2 outdir=$3
shift 3
[[ $package == logolig ]] || { echo "unknown package: $package" >&2; usage; }

tarball=
while [[ $# -gt 0 ]]; do
  case $1 in
    --tarball) [[ $# -ge 2 ]] || usage; tarball=$2; shift 2 ;;
    *) echo "unknown argument: $1" >&2; usage ;;
  esac
done

[[ $version =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "version must be X.Y.Z: $version" >&2; exit 2; }

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
template=$root/packaging/aur/logolig/PKGBUILD.in
[[ -f $template ]] || { echo "missing template: $template" >&2; exit 1; }

# Never write into the AUR clones or into a repository that pushes to the AUR.
mkdir -p "$outdir"
out=$(cd "$outdir" && pwd -P)
case $out in
  */.git-exclude/tmp/aur-repositories|*/.git-exclude/tmp/aur-repositories/*)
    echo "refusing to write into the AUR clones: $out" >&2; exit 1 ;;
esac
if git -C "$out" rev-parse --git-dir >/dev/null 2>&1 \
   && git -C "$out" remote -v | grep -q 'aur\.archlinux\.org'; then
  echo "refusing to write into a repository whose remote is aur.archlinux.org: $out" >&2; exit 1
fi

if [[ -n $tarball ]]; then
  [[ -f $tarball ]] || { echo "tarball not found: $tarball" >&2; exit 1; }
  name=$package-$version.tar.gz
  cp -- "$tarball" "$out/$name"
  source_url=file://$out/$name
  sha=$(sha256sum "$out/$name" | cut -d' ' -f1)
else
  source_url=https://github.com/nabbisen/logolig/archive/refs/tags/$version.tar.gz
  tmp=$(mktemp)
  trap 'rm -f "$tmp"' EXIT
  curl --fail --silent --show-error --location --output "$tmp" "$source_url"
  [[ -s $tmp ]] || { echo "downloaded archive is empty" >&2; exit 1; }
  sha=$(sha256sum "$tmp" | cut -d' ' -f1)
fi
[[ $sha =~ ^[0-9a-f]{64}$ ]] || { echo "not a SHA-256: $sha" >&2; exit 1; }

# '|' cannot occur in the substituted values (a URL path and hex).
sed -e "s|@PKGVER@|$version|g" -e "s|@SOURCE_URL@|$source_url|g" -e "s|@SHA256@|$sha|g" \
  "$template" > "$out/PKGBUILD"

if grep -n '@[A-Z0-9_]*@' "$out/PKGBUILD"; then
  echo "unreplaced placeholder in $out/PKGBUILD" >&2; exit 1
fi

(cd "$out" && makepkg --printsrcinfo > .SRCINFO)
echo "rendered $out/PKGBUILD and $out/.SRCINFO (sha256 $sha)"
