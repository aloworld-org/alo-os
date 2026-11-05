#!/usr/bin/env bash
set -euo pipefail
app_dir="$(cd -- "$(dirname -- "$0")/.." && pwd)"
build_target="${CARGO_TARGET_DIR:-$app_dir/target}"
scratch="$(mktemp -d)"
cleanup() {
  result=$?
  if [[ "$result" != 0 ]]; then
    printf 'Native verification failed (%s). Evidence: %s\n' "$result" "$scratch" >&2
    for log in "$scratch/xvfb.log" "$scratch/app.log"; do
      if [[ -f "$log" ]]; then tail -n 35 "$log" >&2; fi
    done
  fi
  if [[ -s "$scratch/app-group" ]]; then
    app_group=$(tr -d '[:space:]' < "$scratch/app-group")
    own_group=$(ps -o pgid= -p "$$" | tr -d '[:space:]')
    if [[ "$app_group" =~ ^[0-9]+$ && "$app_group" -gt 1 && "$app_group" != "$own_group" ]]; then
      kill -TERM -- "-$app_group" 2>/dev/null || true
    fi
  elif [[ -n "${app_pid:-}" ]]; then
    kill "$app_pid" 2>/dev/null || true
  fi
  if [[ -n "${app_runner_pid:-}" ]]; then wait "$app_runner_pid" 2>/dev/null || true; fi
  if [[ -n "${display_pid:-}" ]]; then kill "$display_pid" 2>/dev/null || true; fi
}
trap cleanup EXIT
# WSLg can own /tmp/.X11-unix read-only. A loopback-only diagnostic display
# avoids altering that shared directory or an existing graphical session.
Xvfb -displayfd 3 -screen 0 1280x960x24 -nolisten unix -nolisten local -listen tcp -ac 3>"$scratch/display" >"$scratch/xvfb.log" 2>&1 &
display_pid=$!
for attempt in $(seq 1 150); do
  [[ ! -s "$scratch/display" ]] || break
  kill -0 "$display_pid" 2>/dev/null || break
  sleep .1
done
if [[ ! -s "$scratch/display" ]]; then
  echo 'The private test display did not report readiness.' >&2
  exit 1
fi
export DISPLAY="localhost:$(cat "$scratch/display")"
export GDK_BACKEND=x11
export GSK_RENDERER=cairo
export XDG_DATA_HOME="$scratch/data"
export ALO_STUDIO_VERIFY_DISPLAY="$DISPLAY"
export NO_AT_BRIDGE=1
export G_DEBUG=fatal-criticals
setsid --wait dbus-run-session -- bash -c 'printf "%s\n" "$$" > "$1/app-pid"; ps -o pgid= -p "$$" > "$1/app-group"; exec "$2"' studio-verification "$scratch" "$build_target/debug/alo-studio" >"$scratch/app.log" 2>&1 &
app_runner_pid=$!
sleep 3
if [[ ! -s "$scratch/app-pid" ]]; then head -30 "$scratch/app.log"; exit 1; fi
app_pid=$(cat "$scratch/app-pid")
if ! kill -0 "$app_pid"; then head -30 "$scratch/app.log"; exit 1; fi
ALO_STUDIO_VERIFY_PREPARE_DELAY=1 "$build_target/debug/examples/verify-recording" "$scratch/display.webm"
ALO_STUDIO_VERIFY_LIGHTWEIGHT=1 "$build_target/debug/examples/verify-recording" "$scratch/display-lightweight.webm"
ALO_STUDIO_VERIFY_SOURCE_END=1 "$build_target/debug/examples/verify-recording" "$scratch/source-ended-while-paused.webm"
bash "$app_dir/packaging/verify-ui.sh" "$scratch"
desktop-file-validate "$app_dir/data/org.aloworld.Studio.desktop"
appstream-util validate-relax --nonet "$app_dir/data/org.aloworld.Studio.metainfo.xml"
if grep -E 'panicked|CRITICAL|Theme parser error' "$scratch/app.log"; then
  echo 'The native app reported a runtime failure.' >&2
  exit 1
fi
printf 'Native verification evidence: %s\n' "$scratch"
