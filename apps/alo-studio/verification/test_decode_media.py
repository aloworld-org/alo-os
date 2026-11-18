"""Exercise actual container decoding and preserve its discontinuities."""

import pathlib
import shutil
import struct
import subprocess
import tempfile
import unittest
import wave

from decode_media import decode


@unittest.skipUnless(shutil.which('ffmpeg') and shutil.which('ffprobe'), 'requires real FFmpeg tools')
class DecodeMedia(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='studio-decoder-test-')
        self.addCleanup(self.temporary.cleanup)
        self.folder = pathlib.Path(self.temporary.name)
        self.raw = self.folder / 'pictures.rgb'
        pictures = bytearray()
        for frame in range(6):
            for _y in range(16):
                for x in range(16):
                    value = (255 if frame % 2 == 0 else 0) if x < 8 else 20 + frame * 25
                    pictures.extend([value] * 3)
        self.raw.write_bytes(pictures)
        self.sound = self.folder / 'sound.wav'
        with wave.open(str(self.sound), 'wb') as output:
            output.setnchannels(1)
            output.setsampwidth(2)
            output.setframerate(8000)
            output.writeframes(b''.join(struct.pack('<h', 2000 if n % 800 < 160 else 0)
                                        for n in range(9600)))

    def fixture(self, *, microphone):
        target = self.folder / ('audio.mkv' if microphone else 'silent.mkv')
        args = ['ffmpeg', '-v', 'error', '-y', '-itsoffset', '2', '-f', 'rawvideo',
                '-pixel_format', 'rgb24', '-video_size', '16x16', '-framerate', '5',
                '-i', str(self.raw)]
        if microphone:
            args += ['-itsoffset', '2', '-i', str(self.sound), '-c:a', 'pcm_s16le']
        args += ['-vf', r'select=not(eq(n\,2))', '-fps_mode', 'passthrough',
                 '-c:v', 'ffv1', '-pix_fmt', 'bgr0', str(target)]
        subprocess.run(args, check=True, capture_output=True)
        return target

    def read(self, target, **kwargs):
        return decode(target, marker_region=(0, 0, 8, 16), motion_region=(8, 0, 8, 16), **kwargs)

    def test_decoded_pictures_keep_real_pts_gap_and_observed_regions(self):
        video, audio = self.read(self.fixture(microphone=False))
        self.assertEqual(video['track_count'], 1)
        self.assertTrue(video['decoded_to_eos'])
        self.assertEqual([row['pts_ns'] for row in video['frames']],
                         [2_000_000_000, 2_200_000_000, 2_600_000_000,
                          2_800_000_000, 3_000_000_000])
        self.assertEqual([row['duration_ns'] for row in video['frames']], [200_000_000] * 5)
        self.assertEqual([row['lit'] for row in video['frames']], [True, False, False, True, False])
        self.assertEqual(len({row['fingerprint'] for row in video['frames']}), 5)
        self.assertEqual(audio, {'track_count': 0, 'decoded_to_eos': True, 'chunks': []})

    def test_pcm_keeps_container_clock_and_sample_count(self):
        video, audio = self.read(self.fixture(microphone=True))
        self.assertEqual(audio['track_count'], 1)
        self.assertTrue(audio['decoded_to_eos'])
        chunks = audio['chunks']
        self.assertEqual(chunks[0]['pts_ns'], 2_000_000_000)
        self.assertEqual(sum(row['duration_ns'] for row in chunks), 1_200_000_000)
        self.assertEqual(chunks[-1]['pts_ns'] + chunks[-1]['duration_ns'], 3_200_000_000)
        self.assertTrue(chunks[0]['tone'])
        self.assertFalse(chunks[5]['tone'])
        self.assertTrue(all(row['duration_ns'] <= 10_000_000 for row in chunks))
        self.assertEqual(len(video['frames']), 5)

    def test_invalid_regions_refuse_before_measurement(self):
        target = self.fixture(microphone=False)
        for region in [(-1, 0, 8, 16), (0, 0, 0, 16), (0, 0, 17, 16),
                       (0, 0, 8), (True, 0, 8, 16), (0., 0, 8, 16)]:
            with self.subTest(region=region), self.assertRaises(ValueError):
                decode(target, marker_region=region, motion_region=(8, 0, 8, 16))

    def test_invalid_or_truncated_container_is_not_eos(self):
        target = self.fixture(microphone=True)
        original = target.read_bytes()
        for data in [b'not a video', original[:len(original) // 2]]:
            target.write_bytes(data)
            with self.assertRaises(ValueError):
                self.read(target)


if __name__ == '__main__':
    unittest.main()
