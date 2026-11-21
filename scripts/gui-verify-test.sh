#!/usr/bin/env bash
# gui-verify-test.sh: replays SAVED captures through the verification functions of gui-verify.sh, with no window and
# no keys. Usage: scripts/gui-verify-test.sh [FIXTURE_DIR]   (default: .git-exclude/review-request; the captures are
# the ones kept by the review requests of the J10, J14a and J14b real runs). Exit 0 when every case behaves as expected.
set -u
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FIX="${1:-$ROOT/.git-exclude/review-request}"
LOGF=$(mktemp); trap 'rm -f "$LOGF"' EXIT
say() { echo "$*" >> "$LOGF"; }
fail() { say "ANOMALY: $*"; exit 1; }        # as gui-session.sh: stop at the first anomaly (here: the subshell)
. "$ROOT/scripts/gui-verify.sh"

WL="$FIX/rfc-027-j14b-pages/snapshots/real-wayland-07-settings.png"      # native Wayland, 1250 x 1373, Settings page, focused item
X1="$FIX/rfc-027-j14a-engine/snapshots/real-x11-02-tab-01.png"           # X11, first Tab
X20="$FIX/rfc-027-j14a-engine/snapshots/real-x11-02-tab-20.png"
LIGHT="$FIX/rfc-021-selection-cues-build/snapshots/real-x11-run3-5-settings.png"
DARK="$FIX/rfc-021-selection-cues-build/snapshots/real-x11-run3-6-theme-dark.png"
for f in "$WL" "$X1" "$X20" "$LIGHT" "$DARK"; do [ -s "$f" ] || { echo "SKIP: fixture missing: $f"; exit 0; }; done

failures=0
# expect NAME rc COMMAND...: run COMMAND in a subshell and compare its exit status with rc.
expect() {
  local name="$1" want="$2" got
  shift 2
  ( "$@" ) >/dev/null 2>&1; got=$?
  if [ "$got" = "$want" ]; then echo "ok   $name"; else echo "FAIL $name (exit $got, wanted $want)"; failures=$((failures + 1)); fi
}

# --- scale -------------------------------------------------------------------------------------------------
capture_scale "$WL" 1042 >/dev/null; [ "$SCALE" = 1.1996 ] && echo "ok   Wayland scale $SCALE" || { echo "FAIL Wayland scale $SCALE"; failures=$((failures+1)); }
capture_scale "$X1" 1250 >/dev/null; [ "$SCALE" = 1.0 ] && echo "ok   X11 scale $SCALE" || { echo "FAIL X11 scale $SCALE"; failures=$((failures+1)); }

# --- X11 captures (scale 1): a ring is found where it is, and a missing ring is reported ----------------------
SCALE=1
expect "a ring that exists is found (Tab 1 vs Tab 20, around the first checkbox)" 0 gv_diff_gt ring "$X1" "$X20" 134 66 246 113 120
expect "a missing ring is reported (a capture against itself)" 1 gv_diff_gt ring "$X1" "$X1" 134 66 246 113 120
expect "an unchanged area is accepted by diff_le" 0 gv_diff_le same "$X1" "$X1" 0 0 1250 1373 0
expect "a changed area is refused by diff_le" 1 gv_diff_le same "$X1" "$X20" 134 66 246 113 10
SMALL="$(mktemp --suffix=.png)"; python3 -I -c "from PIL import Image; import sys; Image.new('RGB', (10, 10)).save(sys.argv[1])" "$SMALL"
expect "captures of different sizes are refused" 1 gv_diff_gt size "$X1" "$SMALL" 0 0 10 10 0
rm -f "$SMALL"
# --- light and dark pages -----------------------------------------------------------------------------------
expect "the light page reads light" 0 gv_mean page "$LIGHT" 400 40 1200 700 light
expect "the dark page reads dark" 0 gv_mean page "$DARK" 400 40 1200 700 dark
expect "a light page is not dark" 1 gv_mean page "$LIGHT" 400 40 1200 700 dark
# --- native Wayland capture at scale 1.1996: the Settings item whose ring hides its bar -----------------------
SCALE=1.1996
expect "Settings bar: the ring-covered bar reads as damaged (49 px < 60), as the real run saw" 1 gv_accent bar "$WL" 87 186 90 265 yes
expect "Settings bar: the same box is not 'no bar' either" 1 gv_accent bar "$WL" 87 186 90 265 no
expect "Home bar region of the Settings page has no bar" 0 gv_accent bar "$WL" 87 14 90 78 no
# --- the scale matters: the same box at scale 1 reads wrongly (the stop of attempt 1) -------------------------
SCALE=1
expect "at the wrong scale the Settings bar box misses the bar's pixels" 1 gv_accent bar "$WL" 87 156 90 219 yes
SCALE=1.1996
# --- locals: a helper must not write the caller's variables (the stop of attempt 2) ----------------------------
n=7; name=keep; count=keep; want=keep; lum=keep
( gv_diff_gt x "$X1" "$X20" 134 66 246 113 120 ) >/dev/null 2>&1
( gv_accent x "$WL" 87 14 90 78 no ) >/dev/null 2>&1
gv_diff_gt x "$X1" "$X20" 134 66 246 113 120 >/dev/null 2>&1 || true
gv_mean x "$LIGHT" 400 40 1200 700 light >/dev/null 2>&1 || true
if [ "$n" = 7 ] && [ "$name" = keep ] && [ "$count" = keep ] && [ "$want" = keep ] && [ "$lum" = keep ]; then echo "ok   helpers leave the caller's variables alone"; else echo "FAIL a helper wrote a caller variable (n=$n name=$name count=$count want=$want lum=$lum)"; failures=$((failures+1)); fi

[ "$failures" = 0 ] && echo "ALL REPLAY CASES BEHAVE" || { echo "$failures replay case(s) FAILED"; exit 1; }
