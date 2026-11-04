#!/usr/bin/env bash
set -euo pipefail
app_dir="$(cd -- "$(dirname -- "$0")/.." && pwd)"
build_target="${CARGO_TARGET_DIR:-$app_dir/target}"
scratch="$(mktemp -d)"
cleanup() {
  if [[ -n "${app_pid:-}" ]]; then kill "$app_pid" 2>/dev/null || true; fi
  if [[ -n "${display_pid:-}" ]]; then kill "$display_pid" 2>/dev/null || true; fi
}
trap cleanup EXIT
# WSLg can own /tmp/.X11-unix read-only. A loopback-only diagnostic display
# avoids altering that shared directory or an existing graphical session.
Xvfb -displayfd 3 -screen 0 1280x960x24 -nolisten unix -nolisten local -listen tcp -ac 3>"$scratch/display" >"$scratch/xvfb.log" 2>&1 &
display_pid=$!
for attempt in $(seq 1 40); do [[ -s "$scratch/display" ]] && break; sleep 0.1; done
if [[ ! -s "$scratch/display" ]]; then head -20 "$scratch/xvfb.log"; exit 1; fi
export DISPLAY="localhost:$(cat "$scratch/display")"
export GDK_BACKEND=x11
export GSK_RENDERER=cairo
export XDG_DATA_HOME="$scratch/data"
export NO_AT_BRIDGE=1
dbus-run-session -- "$build_target/debug/alo-studio" >"$scratch/app.log" 2>&1 &
app_pid=$!
sleep 3
if ! kill -0 "$app_pid"; then head -30 "$scratch/app.log"; exit 1; fi
"$build_target/debug/examples/verify-recording" "$scratch/display.webm"
desktop-file-validate "$app_dir/data/org.aloworld.Studio.desktop"
appstream-util validate-relax --nonet "$app_dir/data/org.aloworld.Studio.metainfo.xml"
if grep -E 'panicked|CRITICAL|Theme parser error' "$scratch/app.log"; then
  echo 'The native app reported a runtime failure.' >&2
  exit 1
fi
printf 'Native verification evidence: %s\n' "$scratch"
