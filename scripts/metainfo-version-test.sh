#!/usr/bin/env bash
# metainfo-version-test.sh: shows that scripts/metainfo-version.sh can fail, rule by rule. Every case builds a tiny tree under
# mktemp -d (removed at the end; nothing is written in the repository); the last case runs the script on the real repository.
set -u
HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
GUARD="$HERE/metainfo-version.sh"
WORK=$(mktemp -d); trap 'rm -rf "$WORK"' EXIT
BAD=0
REL=packaging/linux/io.github.nabbisen.Logolig.metainfo.xml

# tree WORKSPACE_VERSION_LINE RELEASES_XML: a fresh tree; echoes its path.
tree() {
  local t; t=$(mktemp -d "$WORK/tree.XXXXXX")
  mkdir -p "$t/packaging/linux"
  printf '[workspace]\nmembers = ["a"]\n\n[workspace.package]\n%s\nedition = "2024"\n\n[workspace.dependencies]\nfoo = { version = "9.9.9" }\n' "$1" > "$t/Cargo.toml"
  printf '<?xml version="1.0"?>\n<!-- a comment that mentions <release version="9.9.9" date="2030-01-01"> must be ignored -->\n<component>\n  <releases>\n%s\n  </releases>\n</component>\n' "$2" > "$t/$REL"
  echo "$t"
}
pass() { local out; out=$("$GUARD" check "$2" 2>&1); if [ $? = 0 ]; then echo "ok   $1"; else echo "FAIL $1: $out"; BAD=$((BAD + 1)); fi; }
fail() {
  local out; out=$("$GUARD" check "$2" 2>&1)
  if [ $? != 0 ] && grep -qF -- "$3" <<<"$out"; then echo "ok   $1"; else echo "FAIL $1 (wanted '$3'): $out"; BAD=$((BAD + 1)); fi
}
V='version = "2.5.1"'
R251='<release version="2.5.1" date="2026-11-30"><description><p>x</p></description></release>'
R250='<release version="2.5.0" date="2026-11-01"/>'

t=$(tree "$V" "$R251"); pass "the newest release equals the workspace version" "$t"
t=$(tree "$V" "$R251
$R250"); pass "older releases below it are fine" "$t"
t=$(tree "$V" "<release
        version=\"2.5.1\"
        date=\"2026-11-30\"/>"); pass "attributes on several lines are read" "$t"
t=$(tree 'version   =   "2.5.1"   # the release' "$R251"); pass "spacing and a trailing comment in Cargo.toml are fine" "$t"
t=$(tree "$V" "$R250"); fail "a version bump without an entry fails and names both versions and the fix" "$t" "the newest <release> is 2.5.0, the workspace version is 2.5.1 (add a <release version=\"2.5.1\""
t=$(tree "$V" '<release version="2.6.0" date="2026-12-01"/>'); fail "an entry newer than the workspace version fails" "$t" "the newest <release> is 2.6.0, the workspace version is 2.5.1"
t=$(tree "$V" "$R250
$R251"); fail "the right entry listed second (oldest first) fails" "$t" "the newest <release> is 2.5.0"
t=$(tree "$V" "$R251
<release version=\"2.6.0\" date=\"2026-12-01\"/>"); fail "a higher version below the first fails: newest first" "$t" "a <release> with version 2.6.0 is listed below the first"
t=$(tree "$V" ""); fail "no release entry fails" "$t" "there is no <release> entry"
t=$(tree "$V" '<release date="2026-11-30"/>'); fail "a release without a version fails" "$t" "has no version attribute"
t=$(tree "$V" '<release version="2.5.1"/>'); fail "no date on the newest release fails" "$t" "has no date=\"YYYY-MM-DD\""
t=$(tree "$V" '<release version="2.5.1" date="30/11/2026"/>'); fail "a malformed date fails" "$t" "found '30/11/2026'"
t=$(tree 'version = "2.5"' "$R251"); fail "a malformed workspace version fails" "$t" "Cargo.toml: no three-part numeric"
t=$(tree '# no version' "$R251"); fail "no workspace version fails (a version in another section does not count)" "$t" "Cargo.toml: no three-part numeric"
t=$(tree "$V" "$R251"); rm "$t/$REL"; fail "a missing MetaInfo fails" "$t" "the MetaInfo does not exist"
out=$("$GUARD" frobnicate 2>&1); [ $? = 2 ] && grep -q usage <<<"$out" && echo "ok   an unknown mode is a usage error (exit 2)" || { echo "FAIL usage: $out"; BAD=$((BAD + 1)); }
if out=$("$GUARD" check 2>&1); then echo "ok   the real repository passes: $out"; else echo "FAIL the real repository: $out"; BAD=$((BAD + 1)); fi
[ "$BAD" = 0 ] && echo "ALL METAINFO-VERSION CASES BEHAVE" || { echo "$BAD case(s) FAILED"; exit 1; }
