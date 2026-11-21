#!/usr/bin/env bash
# gui-verify.sh: the verification functions of the real-window runs (source it after gui-session.sh).
#
# The rules (RFC 027 J14c; the shared fix for four script-side stops of earlier runs):
#   * every function takes FILE PATHS, so a run feeds live captures and a test feeds saved ones
#     (scripts/gui-verify-test.sh replays saved PNGs through the same functions);
#   * `set -u`, and `local` for EVERY helper variable (an earlier run died because a helper wrote the loop's `n`);
#   * boxes are LOGICAL pixels and are scaled by $SCALE, which capture_scale sets and prints in the log
#     (1 on X11; 1.2 or so on a scaled native-Wayland output). Never hard-code a scale.
#   * a check that fails calls `fail` (the run stops at the first anomaly).
#
# Needs: say and fail (gui-session.sh), python3 with Pillow.
set -u
GV_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCALE=1
gv() { python3 -I "$GV_DIR/gui-verify.py" --scale "$SCALE" "$@"; }

# capture_scale PNG LOGICAL_WIDTH: sets SCALE = capture width / the window's logical width, and says so.
# On Wayland the logical width comes from `niri msg -j windows` (layout.window_size[0]); on X11 pass the
# X11 window width (the capture is then at scale 1).
capture_scale() {
  local png="$1" logical="$2"
  SCALE=$(python3 -I "$GV_DIR/gui-verify.py" scale "$png" "$logical")
  say "capture scale: $(python3 -I -c "from PIL import Image; import sys; print(Image.open(sys.argv[1]).width)" "$png") px / $logical logical = $SCALE"
}

# The window's logical width by niri, for the window id $ID.
niri_logical_width() {
  niri msg --json windows | python3 -I -c "import json,sys; i=int(sys.argv[1]); print([w for w in json.load(sys.stdin) if w['id']==i][0]['layout']['window_size'][0])" "$ID"
}

# gv_diff_gt NAME A.png B.png X0 Y0 X1 Y1 MIN: the crops must differ by more than MIN pixels.
gv_diff_gt() {
  local name="$1" a="$2" b="$3" x0="$4" y0="$5" x1="$6" y1="$7" min="$8" count
  count=$(gv diff "$a" "$b" "$x0" "$y0" "$x1" "$y1")
  say "signal $name: $count differing px"
  [ "$count" != 99999999 ] || fail "$name: the two captures differ in size"
  [ "$count" -gt "$min" ] || fail "$name: expected a change (> $min px), got $count"
}

# gv_diff_le NAME A.png B.png X0 Y0 X1 Y1 MAX: the crops must differ by at most MAX pixels.
gv_diff_le() {
  local name="$1" a="$2" b="$3" x0="$4" y0="$5" x1="$6" y1="$7" max="$8" count
  count=$(gv diff "$a" "$b" "$x0" "$y0" "$x1" "$y1")
  say "signal $name: $count differing px"
  [ "$count" != 99999999 ] || fail "$name: the two captures differ in size"
  [ "$count" -le "$max" ] || fail "$name: expected at most $max px, got $count"
}

# gv_accent NAME IMG.png X0 Y0 X1 Y1 yes|no: the accent-bar colour (88,101,242) is (yes) or is not (no) in the box.
gv_accent() {
  local name="$1" img="$2" x0="$3" y0="$4" x1="$5" y1="$6" want="$7" count
  count=$(gv accent "$img" "$x0" "$y0" "$x1" "$y1" 88 101 242)
  say "signal $name: $count accent px"
  if [ "$want" = yes ]; then
    [ "$count" -gt 60 ] || fail "$name: expected the accent bar, got $count"
  else
    [ "$count" -lt 6 ] || fail "$name: expected no accent bar, got $count"
  fi
}

# gv_mean NAME IMG.png X0 Y0 X1 Y1 light|dark: the page in the box is light (> 160) or dark (< 90).
gv_mean() {
  local name="$1" img="$2" x0="$3" y0="$4" x1="$5" y1="$6" want="$7" lum
  lum=$(gv mean "$img" "$x0" "$y0" "$x1" "$y1")
  say "signal $name: mean luminance $lum"
  if [ "$want" = dark ]; then
    [ "$lum" -lt 90 ] || fail "$name: expected a dark page, got $lum"
  else
    [ "$lum" -gt 160 ] || fail "$name: expected a light page, got $lum"
  fi
}
