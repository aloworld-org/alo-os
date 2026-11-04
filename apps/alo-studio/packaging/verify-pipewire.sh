#!/usr/bin/env bash
set -euo pipefail
# A private server and generated video exercise the actual PipeWire socket.
# This is not a screen permission test, and it never opens hardware devices.
if [[ "${1:-}" != --session ]]; then
  app_dir="$(cd -- "$(dirname -- "$0")/.." && pwd)"
  scratch=$(mktemp -d)
  printf 'PipeWire verification evidence: %s\n' "$scratch"
  export XDG_RUNTIME_DIR="$scratch/run" XDG_CONFIG_HOME="$scratch/config" XDG_STATE_HOME="$scratch/state"
  mkdir -m 700 "$XDG_RUNTIME_DIR"
  mkdir -p "$XDG_CONFIG_HOME/wireplumber/main.lua.d"
  printf 'alsa_monitor.enabled = false\nv4l2_monitor.enabled = false\nlibcamera_monitor.enabled = false\n' > "$XDG_CONFIG_HOME/wireplumber/main.lua.d/89-studio-fixture.lua"
  exec dbus-run-session -- bash "$0" --session "$scratch" "${CARGO_TARGET_DIR:-$app_dir/target}"
fi
scratch="$2"
build_target="$3"
cleanup() {
  for process in ${source_pid:-} ${manager_pid:-} ${pipewire_pid:-}; do
    kill "$process" 2>/dev/null || true
  done
}
trap cleanup EXIT
pipewire > "$scratch/pipewire.log" 2>&1 &
pipewire_pid=$!
wireplumber > "$scratch/manager.log" 2>&1 &
manager_pid=$!
for attempt in $(seq 1 50); do
  [[ ! -S "$XDG_RUNTIME_DIR/pipewire-0" ]] || break
  sleep .1
done
if [[ -n "${ALO_STUDIO_CAPTURE_PROVIDER:-}" ]]; then
  # The actual compositor publisher must preserve square pixels across the
  # PipeWire bridge, including a source whose aspect ratio differs from 16:9.
  gst-launch-1.0 -q videotestsrc is-live=true pattern=ball ! video/x-raw,format=BGRx,width=1280,height=800,framerate=30/1 ! fdsink fd=1 2> "$scratch/input.log" |
    "$ALO_STUDIO_CAPTURE_PROVIDER" 1280 800 studio-pipewire-fixture org.aloworld.Studio > "$scratch/publisher.json" 2> "$scratch/source.log" &
  source_pid=$!
  for attempt in $(seq 1 350); do
    [[ ! -s "$scratch/publisher.json" ]] || break
    kill -0 "$source_pid" 2>/dev/null || break
    sleep .1
  done
  node=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["id"])' "$scratch/publisher.json")
  export ALO_STUDIO_VERIFY_NODE_SERIAL
  ALO_STUDIO_VERIFY_NODE_SERIAL=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["serial"])' "$scratch/publisher.json")
  export ALO_STUDIO_EXPECT_PREVIEW_HEIGHT=400
else
  gst-launch-1.0 -q videotestsrc is-live=true pattern=ball ! video/x-raw,format=RGBx,width=640,height=360,framerate=30/1 ! pipewiresink mode=provide stream-properties='props,node.name=studio-pipewire-fixture,media.class=Video/Source' > "$scratch/source.log" 2>&1 &
  source_pid=$!
  sleep 2
  pw-dump > "$scratch/nodes.json"
  node=$(python3 -c 'import json,sys; print(next(n["id"] for n in json.load(open(sys.argv[1])) if n.get("info",{}).get("props",{}).get("node.name")=="studio-pipewire-fixture"))' "$scratch/nodes.json")
fi
timeout 25s "$build_target/debug/examples/verify-recording" "$scratch/pipewire.webm" "$node" "$XDG_RUNTIME_DIR/pipewire-0"
printf 'PipeWire verification evidence: %s\n' "$scratch"
