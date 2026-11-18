"""Exercise actual container decoding and preserve its discontinuities."""

import json
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

    def probe(self, target, *args):
        result = subprocess.run(['ffprobe', '-v', 'error', *args, '-of', 'json', str(target)],
                                check=True, capture_output=True, text=True)
        return json.loads(result.stdout)

    def test_marker_changes_do_not_masquerade_as_motion(self):
        pictures = bytearray()
        for frame in range(6):
            for _y in range(16):
                for x in range(16):
                    value = (255 if frame % 2 == 0 else 0) if x < 8 else 50
                    pictures.extend([value] * 3)
        self.raw.write_bytes(pictures)
        video, _ = self.read(self.fixture(microphone=False))
        self.assertEqual(len({row['fingerprint'] for row in video['frames']}), 1)
        self.assertEqual([row['lit'] for row in video['frames']], [True, False, False, True, False])

    def test_motion_changes_do_not_masquerade_as_marker_light(self):
        pictures = bytearray()
        for frame in range(6):
            for _y in range(16):
                for x in range(16):
                    value = 0 if x < 8 else 20 + frame * 25
                    pictures.extend([value] * 3)
        self.raw.write_bytes(pictures)
        video, _ = self.read(self.fixture(microphone=False))
        self.assertEqual(len({row['fingerprint'] for row in video['frames']}), 5)
        self.assertTrue(all(row['lit'] is False for row in video['frames']))

    def test_pixels_outside_both_explicit_regions_change_neither_observation(self):
        pictures = bytearray()
        for frame in range(6):
            for _y in range(16):
                for x in range(16):
                    value = (0 if x < 4 else 50) if x < 8 else 20 + frame * 25
                    pictures.extend([value] * 3)
        self.raw.write_bytes(pictures)
        video, _ = decode(self.fixture(microphone=False), marker_region=(0, 0, 4, 16),
                          motion_region=(4, 0, 4, 16))
        self.assertEqual(len({row['fingerprint'] for row in video['frames']}), 1)
        self.assertTrue(all(row['lit'] is False for row in video['frames']))

    def test_actual_pcm_holes_are_retained_rather_than_filled_or_rebased(self):
        target = self.folder / 'pcm-gap.mkv'
        subprocess.run([
            'ffmpeg', '-v', 'error', '-y', '-itsoffset', '2', '-f', 'rawvideo',
            '-pixel_format', 'rgb24', '-video_size', '16x16', '-framerate', '5',
            '-i', str(self.raw), '-itsoffset', '2', '-i', str(self.sound),
            '-af', r'asetnsamples=n=80:p=0,aselect=not(between(n\,40\,59))',
            '-c:a', 'pcm_s16le', '-c:v', 'ffv1', '-pix_fmt', 'bgr0',
            '-fps_mode', 'passthrough', str(target)], check=True, capture_output=True)
        # Check the generated file independently; requested filter timing alone
        # is not evidence that a real container contains the intended hole.
        actual = self.probe(target, '-select_streams', 'a:0', '-show_frames')['frames']
        starts = [round(float(row['pts_time']) * 1_000_000_000) for row in actual]
        durations = [int(row['nb_samples']) * 1_000_000_000 // 8000 for row in actual]
        gaps = [start - previous - duration
                for previous, duration, start in zip(starts, durations, starts[1:])]
        self.assertEqual(max(gaps), 200_000_000)
        self.assertEqual(sum(durations), 1_000_000_000)
        _, audio = self.read(target)
        chunks = audio['chunks']
        self.assertEqual(chunks[0]['pts_ns'], 2_000_000_000)
        self.assertEqual(sum(row['duration_ns'] for row in chunks), sum(durations))
        self.assertEqual(chunks[-1]['pts_ns'] + chunks[-1]['duration_ns'], 3_200_000_000)
        decoded_gaps = [following['pts_ns'] - current['pts_ns'] - current['duration_ns']
                        for current, following in zip(chunks, chunks[1:])]
        self.assertEqual(max(decoded_gaps), 200_000_000)

    def test_actual_overlapping_pcm_is_not_retimed_into_continuity(self):
        target = self.folder / 'pcm-overlap.mkv'
        subprocess.run([
            'ffmpeg', '-v', 'error', '-y', '-itsoffset', '2', '-f', 'rawvideo',
            '-pixel_format', 'rgb24', '-video_size', '16x16', '-framerate', '5',
            '-i', str(self.raw), '-itsoffset', '2', '-i', str(self.sound),
            '-af', 'asetnsamples=n=800:p=0,asetpts=2/TB+N/(2*SR*TB)',
            '-c:a', 'pcm_s16le', '-c:v', 'ffv1', '-pix_fmt', 'bgr0',
            '-fps_mode', 'passthrough', str(target)], check=True, capture_output=True)
        actual = self.probe(target, '-select_streams', 'a:0', '-show_frames')['frames']
        starts = [round(float(row['pts_time']) * 1_000_000_000) for row in actual]
        durations = [int(row['nb_samples']) * 1_000_000_000 // 8000 for row in actual]
        overlaps = [start - previous - duration
                    for previous, duration, start in zip(starts, durations, starts[1:])]
        self.assertEqual(min(overlaps), -50_000_000)
        _, audio = self.read(target)
        chunks = audio['chunks']
        self.assertEqual(sum(row['duration_ns'] for row in chunks), sum(durations))
        decoded_overlaps = [following['pts_ns'] - current['pts_ns'] - current['duration_ns']
                            for current, following in zip(chunks, chunks[1:])]
        self.assertEqual(min(decoded_overlaps), -50_000_000)

    def test_an_audio_only_container_cannot_claim_decoded_video(self):
        target = self.folder / 'audio-only.mkv'
        subprocess.run(['ffmpeg', '-v', 'error', '-y', '-i', str(self.sound),
                        '-c:a', 'pcm_s16le', str(target)], check=True, capture_output=True)
        actual = self.probe(target, '-show_streams')['streams']
        self.assertTrue(all(row['codec_type'] == 'audio' for row in actual))
        with self.assertRaises(ValueError):
            self.read(target)

    def test_declared_audio_without_any_decoded_samples_is_refused(self):
        target = self.folder / 'empty-audio.mkv'
        subprocess.run([
            'ffmpeg', '-v', 'error', '-y', '-f', 'rawvideo', '-pixel_format', 'rgb24',
            '-video_size', '16x16', '-framerate', '5', '-i', str(self.raw),
            '-i', str(self.sound), '-af', 'aselect=0', '-c:a', 'pcm_s16le',
            '-c:v', 'ffv1', '-pix_fmt', 'bgr0', str(target)], check=True, capture_output=True)
        actual = self.probe(target, '-show_streams')['streams']
        self.assertEqual(sum(row['codec_type'] == 'audio' for row in actual), 1)
        self.assertEqual(self.probe(target, '-select_streams', 'a:0', '-show_frames')['frames'], [])
        with self.assertRaises(ValueError):
            self.read(target)

    def test_real_pcm_sample_counts_include_the_partial_final_chunk_at_both_rates(self):
        for rate in [16000, 48000]:
            with self.subTest(rate=rate):
                samples = rate + 37
                with wave.open(str(self.sound), 'wb') as output:
                    output.setnchannels(1)
                    output.setsampwidth(2)
                    output.setframerate(rate)
                    output.writeframes(struct.pack('<h', 2000) * samples)
                target = self.fixture(microphone=True)
                actual = self.probe(target, '-select_streams', 'a:0', '-show_frames')['frames']
                self.assertEqual(sum(int(row['nb_samples']) for row in actual), samples)
                _, audio = self.read(target)
                duration = sum(row['duration_ns'] for row in audio['chunks'])
                self.assertLessEqual(abs(duration - round(samples * 1_000_000_000 / rate)), 1000)
                self.assertTrue(audio['chunks'][-1]['tone'])
                self.assertTrue(any(row['duration_ns'] < 10_000_000 for row in audio['chunks']))

    def test_extra_container_tracks_are_refused_or_retained_in_the_actual_count(self):
        original = self.fixture(microphone=True)
        for kind in ['video', 'audio']:
            with self.subTest(kind=kind):
                target = self.folder / f'two-{kind}.mkv'
                maps = ['-map', '0:v:0', '-map', '0:a:0', '-map',
                        '0:v:0' if kind == 'video' else '0:a:0']
                subprocess.run(['ffmpeg', '-v', 'error', '-y', '-i', str(original),
                                *maps, '-c', 'copy', str(target)], check=True, capture_output=True)
                actual = self.probe(target, '-show_streams')['streams']
                self.assertEqual(sum(row['codec_type'] == kind for row in actual), 2)
                try:
                    video, audio = self.read(target)
                except ValueError:
                    continue
                self.assertEqual((video if kind == 'video' else audio)['track_count'], 2)

    def test_motion_region_and_both_coordinate_axes_are_validated(self):
        target = self.fixture(microphone=False)
        invalid = [None, '0,0,8,16', (-1, 0, 8, 16), (0, -1, 8, 16), (0, 0, 0, 16), (0, 0, 8, 0),
                   (0, 0, 8, 17), (15, 0, 2, 16), (0, 15, 8, 2),
                   (0, 0, 8), (0, 0, 8, 16, 1), (False, 0, 8, 16), (0, 0., 8, 16)]
        for key in ['marker_region', 'motion_region']:
            for region in invalid:
                with self.subTest(key=key, region=region), self.assertRaises(ValueError):
                    arguments = {'marker_region': (0, 0, 8, 16), 'motion_region': (8, 0, 8, 16)}
                    arguments[key] = region
                    decode(target, **arguments)

    def test_near_complete_truncated_containers_cannot_claim_decode_eos(self):
        target = self.fixture(microphone=True)
        original = target.read_bytes()
        for missing in [1, 32, 128]:
            with self.subTest(missing=missing):
                target.write_bytes(original[:-missing])
                with self.assertRaises(ValueError):
                    self.read(target)

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
