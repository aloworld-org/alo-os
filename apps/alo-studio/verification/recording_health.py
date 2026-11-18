"""Audit decoded media without changing pictures, samples or their timestamps.

The caller supplies actual decoder observations and separately checks the live
reference. Synthetic observations in unit tests are never acceptance evidence.
"""

import statistics

MILLISECOND = 1_000_000
SECOND = 1_000_000_000
VIDEO_LIMIT = 400 * MILLISECOND
AUDIO_LIMIT = 40 * MILLISECOND


def channel(value, count, key, flag):
    """Validate one decoded track inventory and its ordered observations."""
    if (not isinstance(value, dict) or type(value.get('track_count')) is not int
            or value['track_count'] != count or value.get('decoded_to_eos') is not True):
        raise ValueError('unexpected track inventory or incomplete decoding')
    rows = value.get(key)
    if not isinstance(rows, list) or (count and len(rows) < 2) or (not count and rows):
        raise ValueError('missing or unexpected decoded observations')
    previous = None
    for row in rows:
        if (not isinstance(row, dict) or type(row.get('pts_ns')) is not int
                or type(row.get('duration_ns')) is not int or row['duration_ns'] <= 0
                or type(row.get(flag)) is not bool):
            raise ValueError('invalid decoded observation')
        if previous is not None and row['pts_ns'] <= previous:
            raise ValueError('decoded timestamps do not increase')
        previous = row['pts_ns']
    return rows


def rising_edges(rows, flag):
    """Read observed rising edges, never a generated marker schedule."""
    events = []
    previous = False
    for row in rows:
        if row[flag] and not previous:
            events.append(row['pts_ns'])
        previous = row[flag]
    return events


def irregular(events):
    gaps = [after - before for before, after in zip(events, events[1:])]
    # Differences within the permitted video timing uncertainty alone cannot
    # distinguish an irregular source from a periodic flashing picture.
    return bool(gaps) and max(gaps) - min(gaps) > VIDEO_LIMIT


def inspect(video, audio, *, microphone):
    """Return measurements only when the selected recording profile passes."""
    if type(microphone) is not bool:
        raise ValueError('the microphone expectation must be explicit')
    frames = channel(video, 1, 'frames', 'lit')
    chunks = channel(audio, int(microphone), 'chunks', 'tone')
    for frame in frames:
        if not isinstance(frame.get('fingerprint'), str) or not frame['fingerprint']:
            raise ValueError('missing motion-region fingerprint')
        if frame['duration_ns'] > VIDEO_LIMIT:
            raise ValueError('a held picture cannot conceal lost motion')
    for before, after in zip(frames, frames[1:]):
        if after['pts_ns'] - before['pts_ns'] - before['duration_ns'] > VIDEO_LIMIT:
            raise ValueError('uncovered video gap exceeds 400 ms')
        if after['fingerprint'] == before['fingerprint']:
            raise ValueError('the observed motion region did not advance')
    video_end = frames[-1]['pts_ns'] + frames[-1]['duration_ns']
    duration = video_end - frames[0]['pts_ns']
    fps = len(frames) * SECOND / duration
    if not 4.5 <= fps <= 5:
        raise ValueError('decoded motion is outside the selected 4.5 to 5 fps profile')
    report = dict(video_frames=len(frames), video_fps=fps, matched_markers=0,
                  median_offset_ms=None, maximum_offset_ms=None)
    if not microphone:
        return report
    for before, after in zip(chunks, chunks[1:]):
        gap = after['pts_ns'] - before['pts_ns'] - before['duration_ns']
        if abs(gap) > AUDIO_LIMIT:
            raise ValueError('decoded PCM gap or overlap exceeds 40 ms')
    audio_end = chunks[-1]['pts_ns'] + chunks[-1]['duration_ns']
    if abs(audio_end - video_end) > VIDEO_LIMIT:
        raise ValueError('decoded track endings differ by more than 400 ms')
    visual, heard = rising_edges(frames, 'lit'), rising_edges(chunks, 'tone')
    if not irregular(visual) or not irregular(heard):
        raise ValueError('observed markers do not establish an irregular sequence')
    left = right = 0
    offsets = []
    while left < len(visual) and right < len(heard):
        offset = heard[right] - visual[left]
        if abs(offset) <= VIDEO_LIMIT:
            offsets.append(abs(offset) / MILLISECOND)
            left += 1
            right += 1
        elif offset < 0:
            right += 1
        else:
            left += 1
    if len(offsets) < 10 or any(len(offsets) / len(events) < .8 for events in (visual, heard)):
        raise ValueError('fewer than ten matches or marker coverage below 80 percent')
    median, maximum = statistics.median(offsets), max(offsets)
    if median > 200 or maximum > 400:
        raise ValueError('decoded synchronization exceeds its timing limits')
    report.update(matched_markers=len(offsets), median_offset_ms=median,
                  maximum_offset_ms=maximum)
    return report
