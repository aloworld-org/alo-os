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
Xvfb -displayfd 3 -screen 0 1280x960x24 -nolisten tcp 3>"$scratch/display" >"$scratch/xvfb.log" 2>&1 &
display_pid=$!
for attempt in $(seq 1 40); do [[ -s "$scratch/display" ]] && break; sleep 0.1; done
export DISPLAY=":$(cat "$scratch/display")"
export GDK_BACKEND=x11
export GSK_RENDERER=cairo
export XDG_DATA_HOME="$scratch/data"
export NO_AT_BRIDGE=1
dbus-run-session -- "$build_target/debug/alo-studio" >"$scratch/app.log" 2>&1 &
app_pid=$!
sleep 3
kill -0 "$app_pid"
"$build_target/debug/examples/verify-recording" "$scratch/display.webm"
desktop-file-validate "$app_dir/data/org.aloworld.Studio.desktop"
appstream-util validate-relax --nonet "$app_dir/data/org.aloworld.Studio.metainfo.xml"
if grep -E 'panicked|CRITICAL|Theme parser error' "$scratch/app.log"; then
  echo 'The native app reported a runtime failure.' >&2
  exit 1
fi
printf 'Native verification evidence: %s\n' "$scratch"
