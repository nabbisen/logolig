#!/usr/bin/env bash
# appx-version-test.sh: shows that scripts/appx-version.sh can fail, and that `update` rewrites one attribute value and nothing else.
# Every case builds a tiny tree under mktemp -d (removed at the end; nothing is written in the repository), then the script runs on the
# real repository. Exit 0 only when every case behaves.
set -u
HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
APPX="$HERE/appx-version.sh"
WORK=$(mktemp -d); trap 'rm -rf "$WORK"' EXIT
BAD=0
REL=packaging/windows/AppxManifest.xml

# A manifest as the real one is laid out (Identity over several lines; MinVersion and MaxVersionTested must never match).
manifest() {
  cat <<XML
<?xml version="1.0" encoding="utf-8"?>
<Package xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10">
  <Identity Name="nabbisen.logolig"
            Publisher="CN=C4BA37E8-8670-4C82-8365-5ECB57373921"
            $1 />
  <Properties><DisplayName>Logolig</DisplayName><Logo>Assets\\StoreLogo.png</Logo></Properties>
  <Dependencies>
    <TargetDeviceFamily Name="Windows.Desktop" MinVersion="10.0.17763.0" MaxVersionTested="10.0.19041.0" />
  </Dependencies>
</Package>
XML
}
# tree WORKSPACE_VERSION_LINE IDENTITY_ATTRIBUTE: a fresh tree; echoes its path.
tree() {
  local t; t=$(mktemp -d "$WORK/tree.XXXXXX")
  mkdir -p "$t/packaging/windows"
  printf '[workspace]\nmembers = ["a"]\n\n[workspace.package]\n%s\nedition = "2024"\n\n[workspace.dependencies]\nfoo = { version = "9.9.9" }\n' "$1" > "$t/Cargo.toml"
  manifest "$2" > "$t/$REL"
  echo "$t"
}
# pass NAME TREE [MODE]: exit 0.   fail NAME TREE MESSAGE [MODE]: non-zero and the message in the output.
pass() { local out; out=$("$APPX" "${3:-check}" "$2" 2>&1); if [ $? = 0 ]; then echo "ok   $1"; else echo "FAIL $1: $out"; BAD=$((BAD + 1)); fi; }
fail() {
  local out; out=$("$APPX" "${4:-check}" "$2" 2>&1)
  if [ $? != 0 ] && grep -qF -- "$3" <<<"$out"; then echo "ok   $1"; else echo "FAIL $1 (wanted '$3'): $out"; BAD=$((BAD + 1)); fi
}
same() { if cmp -s "$2" "$3"; then echo "ok   $1"; else echo "FAIL $1: files differ"; diff "$2" "$3" | head -5; BAD=$((BAD + 1)); fi; }

V='version = "2.5.1"'
t=$(tree "$V" 'Version="2.5.1.0"'); pass "in sync passes (2.5.1 and 2.5.1.0)" "$t"
t=$(tree "$V" 'Version="1.0.0.0"'); fail "a stale manifest fails and names both values and the fix" "$t" "Identity Version is 1.0.0.0, expected 2.5.1.0 (run: bash scripts/appx-version.sh update)"
t=$(tree "$V" 'Version="2.5.1.7"'); fail "a fourth part other than 0 fails" "$t" "Identity Version is 2.5.1.7, expected 2.5.1.0"
t=$(tree "$V" 'Version="2.5.1"'); fail "a three-part manifest version fails" "$t" "Identity Version is 2.5.1, expected 2.5.1.0"
t=$(tree "$V" 'Foo="bar"'); fail "no Version on the Identity fails, even though MinVersion and MaxVersionTested exist" "$t" "the <Identity> element has no Version attribute"
t=$(tree 'version = "2.5"' 'Version="2.5.0.0"'); fail "a two-part workspace version fails" "$t" "Cargo.toml: no three-part numeric"
t=$(tree 'version = "2.5.1-beta"' 'Version="2.5.1.0"'); fail "a pre-release workspace version fails" "$t" "Cargo.toml: no three-part numeric"
t=$(tree '# no version here' 'Version="2.5.1.0"'); fail "no workspace version fails (a version in another section does not count)" "$t" "Cargo.toml: no three-part numeric"
t=$(tree 'version   =   "2.5.1"   # the release' 'Version="2.5.1.0"'); pass "spacing and a trailing comment in Cargo.toml are fine" "$t"

# update: fixes a stale manifest, then check passes, and every other byte is untouched
good=$(tree "$V" 'Version="2.5.1.0"')
t=$(tree "$V" 'Version="1.0.0.0"')
out=$("$APPX" update "$t" 2>&1); echo "     update said: $out"
pass "after update the check passes" "$t"
same "update changed only the Version value (byte for byte equal to the in-sync manifest)" "$good/$REL" "$t/$REL"
cp "$t/$REL" "$WORK/after-first"
out2=$("$APPX" update "$t" 2>&1); echo "     second update said: $out2"
same "a second update changes nothing" "$WORK/after-first" "$t/$REL"
[ "$(diff "$good/$REL" <(manifest 'Version="1.0.0.0"') | grep -c '^[<>]')" = 2 ] && echo "ok   the stale and the good manifest differ in exactly one line (the Identity's Version line)" || { echo "FAIL the fixture pair differs in more than one line"; BAD=$((BAD + 1)); }
# update never adds or guesses
t=$(tree "$V" 'Foo="bar"'); cp "$t/$REL" "$WORK/before"
fail "update with no Version attribute fails" "$t" "has no Version attribute" update
same "and leaves the manifest untouched" "$WORK/before" "$t/$REL"
t=$(tree 'version = "x"' 'Version="1.0.0.0"'); cp "$t/$REL" "$WORK/before"
fail "update with a malformed workspace version fails" "$t" "Cargo.toml: no three-part numeric" update
same "and leaves the manifest untouched" "$WORK/before" "$t/$REL"
# a usage error
out=$("$APPX" frobnicate 2>&1); [ $? = 2 ] && grep -q usage <<<"$out" && echo "ok   an unknown mode is a usage error (exit 2)" || { echo "FAIL usage: $out"; BAD=$((BAD + 1)); }

# The real repository.
if out=$("$APPX" check 2>&1); then echo "ok   the real repository passes: $out"; else echo "FAIL the real repository: $out"; BAD=$((BAD + 1)); fi
[ "$BAD" = 0 ] && echo "ALL APPX-VERSION CASES BEHAVE" || { echo "$BAD case(s) FAILED"; exit 1; }
