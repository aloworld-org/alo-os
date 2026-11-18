"""Reject unhealthy live observations before grading a Studio recording.

This reads draw callbacks and actual Pulse monitor samples. It generates no
frames, changes no timestamps and fills no missing audio. Run as a command with
the reference JSON path, or call inspect from the installed walk's final audit.
"""

import json
import statistics
import sys

MILLISECOND = 1_000_000
MAXIMUM_GAP = 400 * MILLISECOND
AUDIO_TOLERANCE = 40 * MILLISECOND


def number(row, key):
    value = row.get(key)
    if type(value) is not int or value < 0:
        raise ValueError(f'invalid reference {key}')
    return value


def observations(trace, key, flag):
    rows = trace.get(key)
    if not isinstance(rows, list) or len(rows) < 2:
        raise ValueError(f'missing reference {key}')
    previous = None
    for row in rows:
        if not isinstance(row, dict) or type(row.get(flag)) is not bool:
            raise ValueError(f'invalid reference {key} observation')
        stamp = number(row, 'clock_ns')
        if previous is not None and stamp <= previous:
            raise ValueError(f'reference {key} timestamps do not increase')
        previous = stamp
    return rows


def onsets(rows, flag, start, end):
    events = []
    previous = False
    for row in rows:
        if row[flag] and not previous and start <= row['clock_ns'] <= end:
            events.append(row['clock_ns'])
        previous = row[flag]
    return events


def inspect(trace):
    """Return measurements only when every reference health requirement passes."""
    if not isinstance(trace, dict) or trace.get('version') != 1 or trace.get('errors') != []:
        raise ValueError('the reference is incomplete or reported errors')
    draws = observations(trace, 'draws', 'lit')
    audio = observations(trace, 'audio', 'tone')
    for before, after in zip(draws, draws[1:]):
        if after['clock_ns'] - before['clock_ns'] > MAXIMUM_GAP:
            raise ValueError('reference drawing gap exceeds 400 ms')
    for row in audio:
        if number(row, 'duration_ns') == 0:
            raise ValueError('reference audio has no samples')
    for before, after in zip(audio, audio[1:]):
        gap = after['clock_ns'] - before['clock_ns'] - before['duration_ns']
        if gap > AUDIO_TOLERANCE:
            raise ValueError('reference audio capture gap exceeds 40 ms')
        if gap < -AUDIO_TOLERANCE:
            raise ValueError('reference audio samples overlap by more than 40 ms')
    # Both paths must have observed the same interval. Startup latency cannot
    # be treated as missing markers, but a stopped path cannot be cropped away.
    starts = (draws[0]['clock_ns'], audio[0]['clock_ns'])
    ends = (draws[-1]['clock_ns'], audio[-1]['clock_ns'] + audio[-1]['duration_ns'])
    if abs(ends[0] - ends[1]) > MAXIMUM_GAP:
        raise ValueError('reference observation endings differ by more than 400 ms')
    start, end = max(starts) + MAXIMUM_GAP, min(ends) - MAXIMUM_GAP
    if end <= start:
        raise ValueError('reference observation interval is empty')
    video_events = onsets(draws, 'lit', start, end)
    audio_events = onsets(audio, 'tone', start, end)
    pairs = []
    left = right = 0
    while left < len(video_events) and right < len(audio_events):
        difference = audio_events[right] - video_events[left]
        if abs(difference) <= MAXIMUM_GAP:
            pairs.append(difference)
            left += 1
            right += 1
        elif difference < 0:
            right += 1
        else:
            left += 1
    if len(pairs) < 10:
        raise ValueError('reference has fewer than ten matched markers')
    if any(len(pairs) / len(events) < .8 for events in (video_events, audio_events)):
        raise ValueError('reference marker coverage is below 80 percent')
    offsets = [abs(value) / MILLISECOND for value in pairs]
    if statistics.median(offsets) > 200 or max(offsets) > 400:
        raise ValueError('reference synchronization exceeds its timing limits')
    return {
        'matched_markers': len(pairs),
        'visual_markers': len(video_events),
        'audio_markers': len(audio_events),
        'median_offset_ms': statistics.median(offsets),
        'maximum_offset_ms': max(offsets),
        'draws': len(draws), 'audio_chunks': len(audio),
    }


if __name__ == '__main__':
    if len(sys.argv) != 2:
        raise SystemExit('usage: reference_health.py reference.json')
    with open(sys.argv[1], encoding='utf-8') as source:
        result = inspect(json.load(source))
    print(json.dumps(result, sort_keys=True))
