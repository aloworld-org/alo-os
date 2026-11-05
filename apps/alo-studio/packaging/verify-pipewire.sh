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
  mkdir -p "$XDG_CONFIG_HOME/wireplumber/wireplumber.conf.d"
  printf 'wireplumber.profiles = { main = { hardware.audio = disabled hardware.bluetooth = disabled hardware.video-capture = disabled } }\n' > "$XDG_CONFIG_HOME/wireplumber/wireplumber.conf.d/89-studio-fixture.conf"
  exec dbus-run-session -- bash "$0" --session "$scratch" "${CARGO_TARGET_DIR:-$app_dir/target}"
fi
scratch="$2"
build_target="$3"
cleanup() {
  result=$?
  if [[ "$result" != 0 ]]; then
    printf 'PipeWire verification failed (%s). Evidence: %s\n' "$result" "$scratch" >&2
    for log in pipewire manager source pulse input; do
      if [[ -f "$scratch/$log.log" ]]; then tail -n 20 "$scratch/$log.log" >&2; fi
    done
  fi
  for process in ${source_pid:-} ${pulse_pid:-} ${manager_pid:-} ${pipewire_pid:-}; do
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
start_source() {
source_name="studio-pipewire-fixture-$RANDOM-$RANDOM"
publisher=(env)
if [[ -n "${ALO_STUDIO_PUBLISHER_PLUGIN_PATH:-}" ]]; then
  publisher+=("GST_PLUGIN_PATH_1_0=$ALO_STUDIO_PUBLISHER_PLUGIN_PATH" "GST_REGISTRY=$scratch/publisher-registry.bin" "ALO_CAPTURE_REQUIRE_PRIVATE_PLUGIN=1")
fi
if [[ -n "${ALO_STUDIO_CAPTURE_PROVIDER:-}" ]]; then
  # The actual compositor publisher must preserve square pixels across the
  # PipeWire bridge, including a source whose aspect ratio differs from 16:9.
  : > "$scratch/publisher.json"
  gst-launch-1.0 -q videotestsrc is-live=true pattern=ball ! video/x-raw,format=BGRx,width=1280,height=800,framerate=30/1 ! fdsink fd=1 2> "$scratch/input.log" |
    "${publisher[@]}" "$ALO_STUDIO_CAPTURE_PROVIDER" 1280 800 "$source_name" org.aloworld.Studio > "$scratch/publisher.json" 2> "$scratch/source.log" &
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
  "${publisher[@]}" gst-launch-1.0 -q videotestsrc is-live=true pattern=ball ! video/x-raw,format=RGBx,width=640,height=360,framerate=30/1 ! pipewiresink mode=provide stream-properties="props,node.name=$source_name,media.class=Video/Source" > "$scratch/source.log" 2>&1 &
  source_pid=$!
  node=
  for attempt in $(seq 1 100); do
    pw-dump > "$scratch/nodes.json"
    node=$(python3 -c 'import json,sys; print(next((n["id"] for n in json.load(open(sys.argv[1])) if n.get("info",{}).get("props",{}).get("node.name")==sys.argv[2]), ""))' "$scratch/nodes.json" "$source_name")
    [[ -z "$node" ]] || break
    kill -0 "$source_pid" 2>/dev/null || { cat "$scratch/source.log" >&2; exit 1; }
    sleep .1
  done
  [[ -n "$node" ]] || { echo 'The private publisher did not become ready.' >&2; exit 1; }
fi
}

# Each case owns a fresh publisher. Reconnecting to the distribution's
# pipewiresink after the first receiver closes can crash that external fixture
# in buffer-pool teardown, before the audio case even receives its first frame.
start_source
timeout 25s "$build_target/debug/examples/verify-recording" "$scratch/pipewire.webm" "$node" "$XDG_RUNTIME_DIR/pipewire-0"
kill "$source_pid" 2>/dev/null || true
wait "$source_pid" 2>/dev/null || true
unset source_pid
start_source
ALO_STUDIO_VERIFY_GENERATED_AUDIO=1 timeout 25s "$build_target/debug/examples/verify-recording" "$scratch/pipewire-with-audio.webm" "$node" "$XDG_RUNTIME_DIR/pipewire-0"
kill "$source_pid" 2>/dev/null || true
wait "$source_pid" 2>/dev/null || true
unset source_pid
# Exercise the microphone implementation through a private Pulse server and a
# virtual monitor. Its socket and default source never refer to host hardware.
export PULSE_SERVER="unix:$XDG_RUNTIME_DIR/pulse/native"
pipewire-pulse > "$scratch/pulse.log" 2>&1 &
pulse_pid=$!
for attempt in $(seq 1 100); do
  pactl info > "$scratch/pulse-info.txt" 2>&1 && break
  kill -0 "$pulse_pid" 2>/dev/null || { cat "$scratch/pulse.log" >&2; exit 1; }
  sleep .1
done
pactl load-module module-null-sink sink_name=studio-private-audio > "$scratch/audio-module.txt"
for attempt in $(seq 1 100); do
  pactl list short sources > "$scratch/audio-sources.txt"
  grep -q studio-private-audio.monitor "$scratch/audio-sources.txt" && break
  sleep .1
done
pactl set-default-source studio-private-audio.monitor
start_source
ALO_STUDIO_VERIFY_PULSE_AUDIO=1 timeout 35s "$build_target/debug/examples/verify-recording" "$scratch/pipewire-with-microphone.webm" "$node" "$XDG_RUNTIME_DIR/pipewire-0"
kill "$source_pid" 2>/dev/null || true
wait "$source_pid" 2>/dev/null || true
unset source_pid
start_source
ALO_STUDIO_VERIFY_LIGHTWEIGHT=1 ALO_STUDIO_VERIFY_PULSE_AUDIO=1 timeout 35s "$build_target/debug/examples/verify-recording" "$scratch/pipewire-lightweight-with-microphone.webm" "$node" "$XDG_RUNTIME_DIR/pipewire-0"
printf 'PipeWire verification evidence: %s\n' "$scratch"
