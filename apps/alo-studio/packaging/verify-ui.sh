#!/usr/bin/env bash
set -euo pipefail
# The caller owns an isolated Xvfb display and a verification-feature build.
# No real screen permission or physical microphone is used in this fixture.
scratch="$1"
app_dir="$(cd -- "$(dirname -- "$0")/.." && pwd)"
probe="${CARGO_TARGET_DIR:-$app_dir/target}/debug/examples/verify-recording"
# Observe the normal semantic status as well as sending UI input: a Space key
# that does nothing must not pass merely because a recording was eventually saved.
wait_for_phase() {
  local expected="$1" snapshot="$2"
  for attempt in $(seq 1 150); do
    gdbus call --address "$(cat "$scratch/session-bus")" --timeout 2 \
      --dest org.aloworld.Studio --object-path /org/aloworld/Studio/Recorder \
      --method org.aloworld.Studio.Recorder1.GetStatus > "$snapshot"
    if python3 -I - "$snapshot" "$expected" <<'PY'
import ast, json, pathlib, sys
status = json.loads(ast.literal_eval(pathlib.Path(sys.argv[1]).read_text())[0])
sys.exit(0 if status['phase'] == sys.argv[2] else 1)
PY
    then return; fi
    sleep .1
  done
  cat "$snapshot" >&2
  echo "Studio did not reach $expected." >&2
  return 1
}
win=$(timeout 15s xdotool search --sync --onlyvisible --name '^alo Studio$' | head -1)
xdotool windowfocus "$win"
xdotool key ctrl+shift+r
sleep 2
# Use the visible 720p control's mnemonic in the software-rendered fixture.
# The production default remains 1080p and slower samples still require consent.
xdotool key alt+7
sleep .2
xdotool key ctrl+shift+r
bar=''
# The global shortcut must respect the disabled New recording button while
# checking/counting down, rather than invalidating an in-flight capture.
xdotool key ctrl+n
accepted_warning=false
for attempt in $(seq 1 150); do
  bar=$(xdotool search --onlyvisible --name 'alo Studio.*Recording' | head -1) || true
  [[ -z "$bar" ]] || break
  # Only this app exists on the isolated display. Its untitled modal receives
  # focus when the measured sample is slow. Deliberately choose Record anyway.
  focus=$(xdotool getwindowfocus)
  if [[ "$focus" != "$win" && "$focus" != 1 && "$accepted_warning" == false ]]; then
    xdotool key Tab Return
    accepted_warning=true
  fi
  sleep .1
done
if [[ -z "$bar" ]]; then
  if command -v import >/dev/null; then import -window "$win" "$scratch/ui-failure.png"; fi
  head -50 "$scratch/app.log"
  echo 'The GUI did not reach its recording controls.' >&2
  exit 1
fi
xdotool windowfocus "$bar"
wait_for_phase recording "$scratch/ui-recording-status.txt"
if xdotool search --onlyvisible --name '^alo Studio$' >/dev/null 2>&1; then
  echo 'The recording workspace stayed visible after its controls appeared.' >&2
  exit 1
fi
xdotool key space
wait_for_phase paused "$scratch/ui-pause-before.txt"
sleep 5
wait_for_phase paused "$scratch/ui-pause-after.txt"
python3 -I - "$scratch/ui-pause-before.txt" "$scratch/ui-pause-after.txt" <<'PY'
import ast, json, pathlib, sys
before, after = [json.loads(ast.literal_eval(pathlib.Path(p).read_text())[0]) for p in sys.argv[1:]]
assert before['elapsed_ms'] == after['elapsed_ms'], (before, after)
for status in [before, after]:
    assert not status['microphone_active'], status
    assert not status['can_pause'] and status['can_resume'] and status['can_finish'], status
PY
xdotool key space
wait_for_phase recording "$scratch/ui-resumed-status.txt"
sleep 1
xdotool key ctrl+shift+r
for attempt in $(seq 1 100); do
  if find "$XDG_DATA_HOME/alo-studio/recordings" -name '*.json' -print -quit | grep -q .; then break; fi
  sleep .1
done
python3 - "$XDG_DATA_HOME/alo-studio/recordings" "$probe" <<'PY'
import json, pathlib, subprocess, sys
folder = pathlib.Path(sys.argv[1])
clips = list(folder.glob('*.json'))
assert len(clips) == 1, f'Expected one completed GUI recording, found {len(clips)}'
clip = json.loads(clips[0].read_text())
assert not clip['microphone'], 'The GUI opened its microphone by default'
assert clip['duration_ms'] >= 1000, clip
assert (folder / (clip['id'] + '.webm')).stat().st_size > 1024
assert (folder / (clip['id'] + '.png')).stat().st_size > 0, 'The saved recording lost its preview image'
duration = int(subprocess.check_output([sys.argv[2], '--duration', str(folder / (clip['id'] + '.webm'))], text=True).strip())
assert clip['duration_ms'] == duration, (clip, duration)
print('PASS: GUI source selection, preflight, countdown, recording controls, finalization, and library save')
PY
for attempt in $(seq 1 100); do
  win=$(xdotool search --onlyvisible --name '^alo Studio$' | head -1) || true
  [[ -z "$win" ]] || break
  sleep .1
done
if [[ -z "$win" ]]; then
  echo 'The completed recording did not return to a visible review window.' >&2
  exit 1
fi
xdotool windowfocus "$win"
xdotool key ctrl+l
sleep .3
xdotool key ctrl+n
if command -v import >/dev/null; then import -window "$win" "$scratch/studio-home.png"; fi
