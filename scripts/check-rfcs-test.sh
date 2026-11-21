#!/usr/bin/env bash
# check-rfcs-test.sh: shows that scripts/check-rfcs.sh can fail. For each invariant there is a tiny throw-away tree
# (under mktemp -d, removed at the end; nothing is written in the repository) that passes and cases that fail with
# the expected message. Then the real script runs on the repository. Exit 0 only when every case behaves.
set -u
HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
CHECK="$HERE/check-rfcs.sh"
WORK=$(mktemp -d); trap 'rm -rf "$WORK"' EXIT
BAD=0

# A valid tree in a fresh directory $WORK/<n>; echoes the path.
fresh() {
  local t; t=$(mktemp -d "$WORK/tree.XXXXXX")   # not a counter: the caller runs this in $( ), a subshell
  mkdir -p "$t"/rfcs/{proposed,accepted,done,archive} "$t"/rfcs/handoffs/002-a "$t"/rfcs/handoffs/release-1.27.0 "$t"/docs/src
  printf '# P\n\n**Status.** Proposed (P3)\n' > "$t/rfcs/proposed/001-p.md"
  printf '# A\n\n**Status.** Accepted, phased\n' > "$t/rfcs/accepted/002-a.md"
  printf '# D\n\n**Status.** Implemented (1.0.0)\n' > "$t/rfcs/done/003-d.md"
  printf '# W\n\n**Status.** Withdrawn - no longer needed\n' > "$t/rfcs/archive/004-w.md"
  printf '[1](./proposed/001-p.md) [2](./accepted/002-a.md) [3](./done/003-d.md) [4](./archive/004-w.md)\n' > "$t/rfcs/README.md"
  echo x > "$t/rfcs/handoffs/002-a/h.md"
  printf 'see [a](/docs/src/a.md) and [b](docs/src/a.md#top) [u](https://example.org/x) [m](mailto:a@b.c) [h](#here)\n' > "$t/README.md"
  printf '[r](../../README.md)\n' > "$t/docs/src/a.md"
  echo r > "$t/ROADMAP.md"
  echo "$t"
}
# pass NAME TREE: the checker must succeed.   fail NAME TREE MESSAGE: it must fail and print MESSAGE.
pass() { local out; out=$("$CHECK" "$2" 2>&1); if [ $? = 0 ]; then echo "ok   $1"; else echo "FAIL $1: $out"; BAD=$((BAD + 1)); fi; }
fail() {
  local out; out=$("$CHECK" "$2" 2>&1)
  if [ $? != 0 ] && grep -qF -- "$3" <<<"$out"; then echo "ok   $1"; else echo "FAIL $1 (wanted '$3'): $out"; BAD=$((BAD + 1)); fi
}

t=$(fresh); pass "the base tree passes" "$t"

# 1 Status matches the folder
t=$(fresh); pass "1 pass: Accepted in accepted/, Withdrawn in archive/" "$t"
t=$(fresh); sed -i 's/Accepted/Proposed/' "$t/rfcs/accepted/002-a.md"; fail "1 fail: a wrong Status word" "$t" "002-a.md: Status starts with 'Proposed'"
t=$(fresh); sed -i 's/Withdrawn/Accepted/' "$t/rfcs/archive/004-w.md"; fail "1 fail: Accepted in archive/" "$t" "004-w.md: Status starts with 'Accepted'"
# 2 filename
t=$(fresh); pass "2 pass: NNN-slug.md names" "$t"
t=$(fresh); cp "$t/rfcs/accepted/002-a.md" "$t/rfcs/accepted/Bad_Name.md"; fail "2 fail: a bad file name" "$t" "Bad_Name.md: the file name is not NNN-slug.md"
# 3 no number in two folders
t=$(fresh); pass "3 pass: distinct numbers" "$t"
t=$(fresh); cp "$t/rfcs/accepted/002-a.md" "$t/rfcs/proposed/002-copy.md"; sed -i 's/Accepted.*/Proposed/' "$t/rfcs/proposed/002-copy.md"
fail "3 fail: RFC 002 in two folders" "$t" "RFC 002 exists more than once"
# 4 index both ways
t=$(fresh); pass "4 pass: every file linked, every link resolves" "$t"
t=$(fresh); cp "$t/rfcs/accepted/002-a.md" "$t/rfcs/accepted/005-new.md"; fail "4 fail: an RFC missing from the index" "$t" "005-new.md: is not linked from rfcs/README.md"
t=$(fresh); echo '[x](./accepted/009-gone.md)' >> "$t/rfcs/README.md"; fail "4 fail: an index link to a missing file" "$t" "does not exist"
t=$(fresh); echo '[x](./accepted/003-d.md)' >> "$t/rfcs/README.md"; fail "4 fail: an index link naming the wrong folder" "$t" "names the wrong folder; the file is in done/003-d.md"
# 5 handoffs
t=$(fresh); pass "5 pass: a numbered handoff and the allow-listed folder" "$t"
t=$(fresh); mkdir "$t/rfcs/handoffs/099-orphan"; fail "5 fail: an orphan handoff directory" "$t" "099-orphan: no RFC 099 in any folder"
t=$(fresh); mkdir "$t/rfcs/handoffs/002-a/done"; fail "5 fail: a handoff with a done/ subfolder" "$t" "has a state subfolder done/"
t=$(fresh); mkdir "$t/rfcs/handoffs/stray"; fail "5 fail: a non-numbered folder outside the allow-list" "$t" "not NNN-slug and not in the allow-list"
# 6 relative links
t=$(fresh); pass "6 pass: relative, /-rooted, anchor, URL, mailto, #-only targets" "$t"
t=$(fresh); echo '[x](nope.md)' >> "$t/docs/src/a.md"; fail "6 fail: a broken relative link" "$t" "docs/src/a.md: the link target 'nope.md' does not exist"
t=$(fresh); echo '[x](/docs/missing.md)' >> "$t/README.md"; fail "6 fail: a broken /-rooted link" "$t" "README.md: the link target '/docs/missing.md' does not exist"
t=$(fresh); echo '<img src="docs/none.png">' >> "$t/README.md"; fail "6 fail: a broken HTML image" "$t" "'docs/none.png' does not exist"
t=$(fresh); printf '```\n[x](/not/there.md)\n```\nand `[y](also/not.md)` inline\n' >> "$t/README.md"; pass "6 pass: links inside a code fence and inline code are ignored" "$t"
t=$(fresh); echo '[x](/docs/src/a.md)' >> "$t/docs/src/a.md"; pass "6 pass: a /-rooted link resolves from the root, not the file" "$t"
t=$(fresh); echo '[x](/a.md)' >> "$t/docs/src/a.md"; fail "6 fail: a /-rooted link is not relative to the file" "$t" "'/a.md' does not exist"

# The real repository.
if "$CHECK" > "$WORK/real.txt" 2>&1; then echo "ok   the real repository passes"; else echo "FAIL the real repository:"; cat "$WORK/real.txt"; BAD=$((BAD + 1)); fi
[ "$BAD" = 0 ] && echo "ALL CHECK-RFCS CASES BEHAVE" || { echo "$BAD case(s) FAILED"; exit 1; }
