"""Real variable-rate pictures keep the container's timestamp precision."""

import json
import pathlib
import shutil
import subprocess
import tempfile
import unittest

from decode_media import decode
from recording_health import inspect


@unittest.skipUnless(shutil.which('ffmpeg') and shutil.which('ffprobe'), 'requires FFmpeg')
class PictureTimebase(unittest.TestCase):
    def test_close_picture_timestamps_are_decoded_without_rounding_or_dropping(self):
        with tempfile.TemporaryDirectory(prefix='studio-picture-timebase-') as temporary:
            folder = pathlib.Path(temporary)
            raw, movie = folder / 'pictures.rgb', folder / 'pictures.mkv'
            raw.write_bytes(b''.join(bytes([30 + frame * 30]) * 16 * 16 * 3
                                     for frame in range(6)))
            subprocess.run([
                'ffmpeg', '-v', 'error', '-f', 'rawvideo', '-pixel_format', 'rgb24',
                '-video_size', '16x16', '-framerate', '5', '-i', str(raw),
                '-vf', r'settb=1/1000,setpts=if(eq(N\,4)\,610\,N*200)',
                '-fps_mode', 'passthrough', '-enc_time_base', '1/1000',
                '-c:v', 'ffv1', '-pix_fmt', 'bgr0', str(movie),
            ], check=True, capture_output=True)
            result = subprocess.run([
                'ffprobe', '-v', 'error', '-show_streams', '-show_frames',
                '-of', 'json', str(movie),
            ], check=True, capture_output=True)
            actual = json.loads(result.stdout)
            self.assertEqual(actual['streams'][0]['time_base'], '1/1000')
            self.assertEqual(actual['streams'][0]['avg_frame_rate'], '5/1')
            self.assertEqual([frame['pts'] for frame in actual['frames']],
                             [0, 200, 400, 600, 610, 1000])
            video, audio = decode(movie, marker_region=(0, 0, 8, 16),
                                  motion_region=(8, 0, 8, 16))
            self.assertEqual([frame['pts_ns'] for frame in video['frames']],
                             [0, 200_000_000, 400_000_000, 600_000_000,
                              610_000_000, 1_000_000_000])
            self.assertEqual(len({frame['fingerprint'] for frame in video['frames']}), 6)
            self.assertEqual(audio['track_count'], 0)
            # Decoding is distinct from accepting timing. Invalid input stays invalid.
            video['frames'][4]['pts_ns'] = video['frames'][3]['pts_ns']
            with self.assertRaisesRegex(ValueError, 'timestamps do not increase'):
                inspect(video, audio, microphone=False)
