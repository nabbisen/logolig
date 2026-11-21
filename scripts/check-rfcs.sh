#!/usr/bin/env bash
# check-rfcs.sh: invariants of the RFC portfolio (RFC 000 "Optional CI invariants", RFC 008 step 5), 5-folder layout.
# Usage: scripts/check-rfcs.sh [REPOSITORY_ROOT]   (default: the repository this script is in; run from anywhere)
# Exit 0 only when every check passes; each failure is printed as "file: what is wrong"; the last line is the count.
#   1 Status matches the folder   2 filename NNN-slug.md   3 no number in two folders   4 index both ways
#   5 handoff directories match an RFC number, and have no state subfolders   6 relative links resolve
# Links (check 6): Markdown `](target)` and HTML src="..." / href="..." in rfcs/, docs/src/, README.md, ROADMAP.md.
# A leading `/` means the repository root (RFC 033); otherwise the target is relative to the file. URLs (any scheme:),
# mailto: and anchor-only targets are skipped; a trailing #anchor is ignored (anchors are NOT checked); fenced code
# blocks are ignored; reference-style links ([x]: target) are not checked. Only grep, sed, awk and find are used.
set -u
ROOT=$(cd "${1:-$(dirname "${BASH_SOURCE[0]}")/..}" && pwd)
RFCS="$ROOT/rfcs"
LOG=$(mktemp); trap 'rm -f "$LOG"' EXIT
# Every failure is printed and logged (a file, not a variable: the link loop runs in pipelines/subshells).
bad() { echo "$1: $2" | tee -a "$LOG"; }
# Handoff folders that are not tied to one RFC number.
ALLOWED_HANDOFFS="release-1.27.0 release-1.28.0"

# The word an RFC's Status must start with, by folder.
expected_status() { case "$1" in proposed) echo Proposed ;; accepted) echo Accepted ;; done) echo Implemented ;; archive) echo "Withdrawn Superseded" ;; esac; }

check_rfc_files() {
  local dir file name word ok w
  for dir in proposed accepted done archive; do
    for file in "$RFCS/$dir"/*.md; do
      [ -e "$file" ] || continue
      name=$(basename "$file")
      [[ $name =~ ^[0-9]{3}-[a-z0-9]+(-[a-z0-9]+)*\.md$ ]] || bad "rfcs/$dir/$name" "the file name is not NNN-slug.md (three digits, lowercase, hyphens)"
      word=$(sed -n 's/^\*\*Status\.\*\* *\([A-Za-z]*\).*/\1/p' "$file" | head -1)
      ok=0; for w in $(expected_status "$dir"); do [ "$word" = "$w" ] && ok=1; done
      [ "$ok" = 1 ] || bad "rfcs/$dir/$name" "Status starts with '${word:-nothing}', the folder $dir/ needs: $(expected_status "$dir")"
    done
  done
  # A number in two files (any folders).
  ls "$RFCS"/{proposed,accepted,done,archive}/*.md 2>/dev/null | sed 's|.*/\([0-9]\{3\}\)-.*|\1|' | sort | uniq -d |
    while read -r n; do bad "rfcs/" "RFC $n exists more than once: $(ls "$RFCS"/*/"$n"-*.md | tr '\n' ' ')"; done
}

check_index() {
  local idx="$RFCS/README.md" dir file link where
  for dir in proposed accepted done archive; do
    for file in "$RFCS/$dir"/*.md; do
      [ -e "$file" ] && { grep -qF "(./$dir/$(basename "$file"))" "$idx" || bad "rfcs/$dir/$(basename "$file")" "is not linked from rfcs/README.md"; }
    done
  done
  for link in $(grep -o '(\./\(proposed\|accepted\|done\|archive\)/[^)]*\.md)' "$idx" | tr -d '()' | sort -u); do
    [ -f "$RFCS/$link" ] && continue
    where=$(ls "$RFCS"/*/"$(basename "$link")" 2>/dev/null | head -1)
    if [ -n "$where" ]; then bad "rfcs/README.md" "the link $link names the wrong folder; the file is in ${where#"$RFCS"/}"
    else bad "rfcs/README.md" "the link $link points to a file that does not exist"; fi
  done
}

check_handoffs() {
  local dir name n state
  for dir in "$RFCS"/handoffs/*/; do
    [ -d "$dir" ] || continue
    name=$(basename "$dir"); n=${name:0:3}
    if [[ $name =~ ^[0-9]{3}- ]]; then
      find "$RFCS"/{proposed,accepted,done,archive} -name "$n-*.md" 2>/dev/null | grep -q . || bad "rfcs/handoffs/$name" "no RFC $n in any folder"
    else
      case " $ALLOWED_HANDOFFS " in *" $name "*) ;; *) bad "rfcs/handoffs/$name" "not NNN-slug and not in the allow-list" ;; esac
    fi
    for state in proposed accepted done archive; do [ -d "$dir$state" ] && bad "rfcs/handoffs/$name" "has a state subfolder $state/ (state lives on the RFC)"; done
  done
}

check_links() {
  local file target path
  while IFS= read -r file; do
    # Drop fenced code blocks and inline code spans, then print every link target.
    awk '/^```/ {f = !f; next} !f' "$file" | sed 's/`[^`]*`//g' |
      grep -o '\](\([^)]*\))\|\(src\|href\)="[^"]*"' | sed 's/^\](//; s/)$//; s/^[a-z]*="//; s/"$//' | sed 's/ .*//; s/#.*//' |
      sort -u | while read -r target; do
        case "$target" in ""|[a-zA-Z]*:*) continue ;; esac
        case "$target" in /*) path="$ROOT$target" ;; *) path="$(dirname "$file")/$target" ;; esac
        [ -e "$path" ] || bad "${file#"$ROOT"/}" "the link target '$target' does not exist"
      done
  done < <(find "$RFCS" "$ROOT/docs/src" -name '*.md' 2>/dev/null; ls "$ROOT/README.md" "$ROOT/ROADMAP.md" 2>/dev/null)
}

check_rfc_files; check_index; check_handoffs; check_links
FAILS=$(wc -l < "$LOG")
if [ "$FAILS" = 0 ]; then echo "check-rfcs: all checks passed"; else echo "check-rfcs: $FAILS problem(s)"; exit 1; fi
