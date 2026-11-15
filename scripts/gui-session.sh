#!/usr/bin/env bash
# gui-session.sh — the shared session library for real-window runs of Logolig (source it; do not run it).
#
# It exists so that every GUI evidence run starts from the same isolated, asserted setup instead of a
# copy of the last script. It enforces the project's GUI evidence rules (.git-exclude/rules/):
#   * the app is started only with a throw-away XDG_CONFIG_HOME and XDG_STATE_HOME under $D, and a
#     pinned settings.json that is written and checked (the owner's real settings are never touched);
#   * the binary keeps its real name `logolig` (the config folder is derived from the executable's
#     name; a renamed copy would read another folder) and exactly one instance runs;
#   * the window is chosen by process id (native Wayland: niri reports the app's own pid), never as
#     "the first window found"; a dialog is a window that appeared after the launch;
#   * every keystroke is preceded by `focus_app`, which checks niri's focused window is the instance;
#   * screenshots are taken through the compositor to an absolute path (never `import`);
#   * the run stops at the first anomaly (`fail`), cleans up its instance, and is not repeated.
# Window size is whatever the app opens at; never resize a running window.
#
# Required variables (set them before calling the functions):
#   BIN   the binary under test, a file named `logolig`
#   D     the run directory, an absolute path (created; holds cfg/, state/, shots/, app.log);
#         it is deleted first, so it must be a directory made for this run
#   LOGF  the run's log file (created)
# Optional: FIX (a fixture file), THEME (Light|Dark, default Light), LOCALE (en|ja, default en),
#   DBUS_ADDRESS (run the app on that private session bus), APP_PATH (the app's PATH).
# Backend: native Wayland on niri (`niri msg`, `wtype` for keys). For an X11 (XWayland) run the window
# is selected by X11 class instead and the pid check does not apply: write that run's own launch.
#
# Functions:
#   say, fail                 log a line; log and stop at the first anomaly
#   session_trap              stop the app (and run `session_extra_cleanup` if defined) when the script exits
#   pinned_settings FILE      write the pinned settings.json
#   iso_setup                 isolated dirs and the pinned, asserted settings.json
#   launch                    start one instance, find its window, set ID PID KNOWN
#   focus_app                 focus the window and check niri agrees (before every keystroke)
#   new_windows, count_new_windows, wait_new_windows N, wait_gone ID
#   capture NAME              compositor screenshot to $D/shots/NAME.png (absolute path)
#   teardown                  stop the instance and log what is left
#   jwins, win_ids, focused   niri window list, ids, "id pid" of the focused window
#
# Example (about ten lines):
#   BIN=$PWD/target/release/logolig; D=$SCRATCH/run1; LOGF=$SCRATCH/run1.log   # a binary named logolig
#   . scripts/gui-session.sh; session_trap
#   iso_setup; launch; capture start
#   focus_app; wtype -M ctrl -k o -m ctrl                                      # Ctrl+O
#   wait_new_windows 1 || fail "the chooser did not open"
#   capture after-ctrl-o
#   teardown
# Analyse captures AFTER the run, never inside it (a slow decode outlives a toast).

say() { echo "$*" | tee -a "$LOGF"; }
fail() { say "ANOMALY: $*"; say "STOPPED at the first anomaly (no repeat, no workaround)"; exit 1; }
session_trap() { trap '{ pkill -x logolig; if declare -F session_extra_cleanup >/dev/null; then session_extra_cleanup; fi; } 2>/dev/null' EXIT; }

jwins() { niri msg --json windows; }
win_ids() { jwins | python3 -c 'import json,sys; print(" ".join(str(w["id"]) for w in json.load(sys.stdin)))'; }
focused() { niri msg --json focused-window | python3 -c 'import json,sys; w=json.load(sys.stdin); print(w["id"], w["pid"])'; }

# Writes the pinned settings.json (default plan, theme and locale as asked) to $1.
pinned_settings() {
  python3 - "$1" "${THEME:-Light}" "${LOCALE:-en}" <<'PY'
import json, sys
plan = {"include_ico": True, "include_apple_touch": True, "png_sizes": [32, 192, 512], "include_html_snippet": True,
        "algorithm": "Lanczos3", "overrides": [], "ico_sizes": [16, 32, 48], "include_svg": True,
        "vectorize_on_raster": True, "vtracer_preset": "Default", "web_manifest": None, "monochrome": False,
        "keep_transparency": True, "include_microsoft_app_logos": False}
json.dump({"export_plan": plan, "theme": sys.argv[2], "locale": sys.argv[3]}, open(sys.argv[1], "w"), indent=2)
PY
}

iso_setup() {
  [ -n "${D:-}" ] && [ -n "${LOGF:-}" ] || { echo "gui-session: set D and LOGF first" >&2; exit 2; }
  # D is deleted: it must be an absolute path that is not the root, the home folder or one of their parents.
  case "$D" in
    /*) ;;
    *) echo "gui-session: D must be an absolute path (got '$D')" >&2; exit 2 ;;
  esac
  case "$D/" in
    "/"|"$HOME/"|"$PWD/") echo "gui-session: refusing to delete '$D'" >&2; exit 2 ;;
  esac
  [ "$(printf '%s' "$D" | tr -cd '/' | wc -c)" -ge 3 ] || { echo "gui-session: '$D' is too shallow to delete" >&2; exit 2; }
  rm -rf -- "$D"; mkdir -p "$D/shots" "$D/cfg/logolig" "$D/state"; : > "$LOGF"
  pinned_settings "$D/cfg/logolig/settings.json"
  [ -s "$D/cfg/logolig/settings.json" ] || fail "the pinned settings.json was not written"
}

launch() {
  [ -x "${BIN:-}" ] || fail "no binary (BIN=${BIN:-unset})"
  [ "$(basename "$BIN")" = logolig ] || fail "the binary must be named logolig (its config folder follows the name)"
  [ "$(pgrep -c -x logolig)" = 0 ] || fail "logolig already running"
  BEFORE=$(win_ids)
  (env ${DBUS_ADDRESS:+DBUS_SESSION_BUS_ADDRESS=$DBUS_ADDRESS} ${APP_PATH:+PATH=$APP_PATH} \
     XDG_CONFIG_HOME="$D/cfg" XDG_STATE_HOME="$D/state" "$BIN" > "$D/app.log" 2>&1 &); sleep 6
  [ "$(pgrep -c -x logolig)" = 1 ] || fail "not exactly one instance"
  PID=$(pgrep -x logolig | head -1)
  # native Wayland: niri reports the app's own pid; the window is the one with that pid
  ID=$(jwins | python3 -c "import json,sys; n=[w['id'] for w in json.load(sys.stdin) if w['pid']==$PID]; print(n[0] if len(n)==1 else '')")
  [ -n "$ID" ] || fail "no single window with pid $PID"
  say "window: niri id $ID, pid $PID, $(jwins | python3 -c "import json,sys; w=[w for w in json.load(sys.stdin) if w['id']==$ID][0]; print('app_id', w['app_id'], 'size', w['layout'].get('window_size'), '(not resized by me)')")"
  KNOWN="$BEFORE $ID"
}

# Focus the instance and check that niri's focused window is it. Call before every keystroke.
focus_app() { niri msg action focus-window --id "$ID"; sleep 0.3; set -- $(focused); [ "$1" = "$ID" ] && [ "$2" = "$PID" ] || fail "focus not on the instance (have $1/$2, want $ID/$PID)"; }

new_windows() { for w in $(win_ids); do echo " $KNOWN " | grep -q " $w " || echo -n "$w "; done; }
count_new_windows() { new_windows | wc -w; }
# Wait up to 4 s until the number of new windows equals $1 (a state poll, not a frame loop).
wait_new_windows() { for i in $(seq 1 20); do [ "$(count_new_windows)" = "$1" ] && return 0; sleep 0.2; done; return 1; }
wait_gone() { for i in $(seq 1 20); do echo " $(win_ids) " | grep -q " $1 " || return 0; sleep 0.2; done; return 1; }

capture() {
  OUT="$D/shots/$1.png"; rm -f "$OUT"
  niri msg action screenshot-window --id "$ID" --path "$OUT"
  for i in $(seq 1 25); do [ -s "$OUT" ] && break; sleep 0.1; done
  [ -s "$OUT" ] || fail "no screenshot $1 (a locked session writes none)"
  say "captured $1 at $(date +%H:%M:%S.%N | cut -c1-12)"
}

teardown() { pkill -x logolig; sleep 1; say "instances left: $(pgrep -c -x logolig); stderr bytes: $(wc -c < "$D/app.log")"; }
