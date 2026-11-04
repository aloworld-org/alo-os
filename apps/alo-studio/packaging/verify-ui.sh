#!/usr/bin/env bash
set -euo pipefail
# The caller owns an isolated Xvfb display and a verification-feature build.
# No real screen permission or physical microphone is used in this fixture.
scratch="$1"
win=$(timeout 15s xdotool search --sync --name '^alo Studio$' | head -1)
xdotool windowfocus "$win"
xdotool key ctrl+shift+r
sleep 2
# Choose 720p explicitly in this software-rendered test environment. Production
# still offers the person's choice and warns about a slow sample.
xdotool mousemove --window "$win" 300 780 click 1
xdotool key Down Return
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
sleep 1
xdotool key space
sleep 1
xdotool key space
sleep 1
xdotool key ctrl+shift+r
for attempt in $(seq 1 100); do
  if find "$XDG_DATA_HOME/alo-studio/recordings" -name '*.json' -print -quit | grep -q .; then break; fi
  sleep .1
done
python3 - "$XDG_DATA_HOME/alo-studio/recordings" <<'PY'
import json, pathlib, sys
folder = pathlib.Path(sys.argv[1])
clips = list(folder.glob('*.json'))
assert len(clips) == 1, f'Expected one completed GUI recording, found {len(clips)}'
clip = json.loads(clips[0].read_text())
assert not clip['microphone'], 'The GUI opened its microphone by default'
assert clip['duration_ms'] >= 1000, clip
assert (folder / (clip['id'] + '.webm')).stat().st_size > 1024
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
