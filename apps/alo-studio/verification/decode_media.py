"""Read actual decoded picture regions and PCM with their container timestamps.

No frame-rate conversion, timestamp rebasing, asynchronous resampling or silence
insertion is permitted. Region coordinates are explicit measured pixel bounds;
the caller must select the reference marker and independently changing content.
"""

import array
import hashlib
import json
import pathlib
import subprocess
import sys
from fractions import Fraction

SECOND = 1_000_000_000
DECODE_LIMIT = 64 * 1024 * 1024


def run(words):
    """Refuse decoder errors and warnings rather than certify damaged media."""
    try:
        result = subprocess.run(words, capture_output=True, timeout=180, check=True)
    except (OSError, subprocess.SubprocessError) as error:
        raise ValueError('media could not be completely decoded') from error
    if result.stderr.strip():
        raise ValueError('the decoder reported a media warning or error')
    return result.stdout


def region(value, width, height):
    if (not isinstance(value, (tuple, list)) or len(value) != 4
            or any(type(item) is not int for item in value)):
        raise ValueError('a picture region requires four integer pixel bounds')
    x, y, w, h = value
    if min(x, y) < 0 or min(w, h) <= 0 or x + w > width or y + h > height:
        raise ValueError('picture region lies outside the decoded frame')
    return x, y, w, h


def stamp(frame, stream):
    try:
        return round(int(frame['pts']) * Fraction(stream['time_base']) * SECOND)
    except (KeyError, ValueError, TypeError, ZeroDivisionError) as error:
        raise ValueError('the decoder supplied no valid frame timestamp') from error


def pictures(path, stream, frames, bounds):
    x, y, width, height = bounds
    size = width * height
    expected = size * len(frames)
    if expected > DECODE_LIMIT:
        raise ValueError('decoded picture observations exceed the private audit bound')
    data = run(['ffmpeg', '-nostdin', '-hide_banner', '-v', 'warning', '-xerror',
                '-err_detect', 'explode', '-guess_layout_max', '0', '-i', str(path),
                '-map', f"0:{stream['index']}",
                '-an', '-vf', f'crop={width}:{height}:{x}:{y}', '-fps_mode', 'passthrough',
                '-pix_fmt', 'gray', '-f', 'rawvideo', 'pipe:1'])
    if len(data) != expected:
        raise ValueError('decoded pictures disagree with the container frame observations')
    return [data[start:start + size] for start in range(0, len(data), size)]


def decoded_audio(path, stream, frames):
    try:
        rate = int(stream['sample_rate'])
        counts = [int(frame['nb_samples']) for frame in frames]
    except (KeyError, ValueError, TypeError) as error:
        raise ValueError('audio has no decoded sample inventory') from error
    if rate < 100 or not counts or min(counts) <= 0 or stream.get('channels') != 1:
        raise ValueError('the private microphone audit expects nonempty mono PCM')
    expected = sum(counts) * 2
    if expected > DECODE_LIMIT:
        raise ValueError('decoded PCM exceeds the private audit bound')
    pcm = run(['ffmpeg', '-nostdin', '-hide_banner', '-v', 'warning', '-xerror',
               '-err_detect', 'explode', '-guess_layout_max', '0', '-i', str(path),
               '-map', f"0:{stream['index']}",
               '-vn', '-c:a', 'pcm_s16le', '-f', 's16le', 'pipe:1'])
    if len(pcm) != expected:
        raise ValueError('decoded PCM disagrees with the container sample inventory')
    samples = array.array('h')
    samples.frombytes(pcm)
    if sys.byteorder != 'little':
        samples.byteswap()
    chunks, consumed = [], 0
    step = rate // 100
    for frame, count in zip(frames, counts):
        base = stamp(frame, stream)
        for offset in range(0, count, step):
            length = min(step, count - offset)
            start = round(Fraction(offset * SECOND, rate))
            end = round(Fraction((offset + length) * SECOND, rate))
            peak = max(abs(value) for value in samples[consumed + offset:consumed + offset + length])
            chunks.append(dict(pts_ns=base + start, duration_ns=end - start, tone=peak > 1200))
        consumed += count
    return chunks


def decode(path, *, marker_region, motion_region):
    """Return unmodified decoded timing observations for recording_health.inspect."""
    path = pathlib.Path(path)
    if not path.is_file() or path.is_symlink() or path.stat().st_size > DECODE_LIMIT:
        raise ValueError('a bounded regular recording file is required')
    try:
        probe = json.loads(run(['ffprobe', '-v', 'warning', '-show_streams', '-show_frames',
                                '-of', 'json', str(path)]))
        streams = probe['streams']
        frames = probe['frames']
        videos = [row for row in streams if row['codec_type'] == 'video']
        audios = [row for row in streams if row['codec_type'] == 'audio']
    except (KeyError, TypeError, json.JSONDecodeError) as error:
        raise ValueError('invalid decoded container inventory') from error
    if len(videos) != 1 or len(audios) > 1 or len(streams) != len(videos) + len(audios):
        raise ValueError('unexpected recording tracks')
    stream = videos[0]
    marker = region(marker_region, stream['width'], stream['height'])
    motion = region(motion_region, stream['width'], stream['height'])
    decoded = [row for row in frames if row['stream_index'] == stream['index']]
    if len(decoded) < 2:
        raise ValueError('the recording has no advancing decoded pictures')
    lights = pictures(path, stream, decoded, marker)
    changes = pictures(path, stream, decoded, motion)
    observations = []
    for frame, light, change in zip(decoded, lights, changes):
        try:
            ticks = int(frame.get('duration', frame.get('pkt_duration')))
            duration = round(ticks * Fraction(stream['time_base']) * SECOND)
        except (TypeError, ValueError, ZeroDivisionError) as error:
            raise ValueError('a decoded picture has no duration') from error
        if duration <= 0:
            raise ValueError('a decoded picture has no positive duration')
        observations.append(dict(pts_ns=stamp(frame, stream), duration_ns=duration,
                                 fingerprint=hashlib.sha256(change).hexdigest(),
                                 lit=sum(light) / len(light) > 180))
    audio = dict(track_count=len(audios), decoded_to_eos=True, chunks=[])
    if audios:
        audio_frames = [row for row in frames if row['stream_index'] == audios[0]['index']]
        audio['chunks'] = decoded_audio(path, audios[0], audio_frames)
    return dict(track_count=1, decoded_to_eos=True, frames=observations), audio
