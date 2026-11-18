"""Independent requirements for decoded installed-recording acceptance.

Proposed API: recording_health.inspect(video, audio, *, microphone) -> dict.
It raises ValueError for invalid or unacceptable observations, never edits the
inputs, and returns measurements rather than repaired media.

video = {
    'track_count': 1, 'decoded_to_eos': True,
    'frames': [{'pts_ns': int, 'duration_ns': positive_int,
                'fingerprint': nonempty_str, 'lit': bool}, ...],
}
audio = {
    'track_count': 0 or 1, 'decoded_to_eos': True,
    'chunks': [{'pts_ns': int, 'duration_ns': positive_int, 'tone': bool}, ...],
}

Track counts come from actual container inspection, EOS from complete decoding,
and rows from actual decoded frames/PCM. PTS values are signed integral ns in a
shared media clock; a common clock origin need not be zero. PCM duration comes
from decoded sample count and rate. fingerprint describes the independently
changing motion region: identical adjacent fingerprints are repeated source
pictures, not additional motion. lit/tone describe observed marker state;
markers are their rising edges, not scheduled cues. Flags are actual bools.

This API audits the selected up-to-5-fps profile. video_fps is decoded frame
count / (last frame end - first frame PTS). Uncovered gaps use the next PTS
minus the previous decoded observation's end. A long held source picture must
not conceal lost motion by extending its duration. Success reports at least
video_frames, video_fps, matched_markers, median_offset_ms, maximum_offset_ms.
Silent recordings report zero matched markers and None for both offsets.

These synthetic observations exist ONLY to test the auditor. They are not
generated acceptance media, a live reference, or installed-system evidence.
The caller must separately validate the independent live reference over the
recording interval. Passing this API cannot establish that prerequisite.
"""

import copy
import unittest

from recording_health import inspect


SECOND = 1_000_000_000
MILLISECOND = 1_000_000
LENGTH = 24 * SECOND
# All onsets fall on the fixture's video grid, with deliberately varying gaps.
MARKERS = tuple(ms * MILLISECOND for ms in
                [1000, 2200, 3800, 4800, 6200, 7800, 8600, 10400,
                 11600, 13400, 15000, 16200, 18000, 19400, 21800])


def set_video_markers(video, markers):
    for frame in video['frames']:
        frame['lit'] = any(start <= frame['pts_ns'] < start + 200 * MILLISECOND
                           for start in markers)


def set_audio_markers(audio, markers, offsets=None):
    offsets = offsets if offsets is not None else [0] * len(markers)
    for chunk in audio['chunks']:
        chunk['tone'] = any(start + offset <= chunk['pts_ns']
                            < start + offset + 80 * MILLISECOND
                            for start, offset in zip(markers, offsets))


def observations(*, microphone=True, frame_count=120, markers=MARKERS, audio_chunk_ms=10):
    """Test-only decoded observations; no file or recording is produced."""
    video = {'track_count': 1, 'decoded_to_eos': True, 'frames': []}
    for index in range(frame_count):
        start = index * LENGTH // frame_count
        end = (index + 1) * LENGTH // frame_count
        video['frames'].append({'pts_ns': start, 'duration_ns': end - start,
                                'fingerprint': f'motion-{index}', 'lit': False})
    audio = {'track_count': int(microphone), 'decoded_to_eos': True, 'chunks': []}
    if microphone:
        audio['chunks'] = [{'pts_ns': index * audio_chunk_ms * MILLISECOND,
                            'duration_ns': audio_chunk_ms * MILLISECOND, 'tone': False}
                           for index in range(24_000 // audio_chunk_ms)]
    set_video_markers(video, markers)
    set_audio_markers(audio, markers)
    return video, audio


class RecordingHealth(unittest.TestCase):
    def refuse(self, video, audio, *, microphone=True):
        with self.assertRaises(ValueError):
            inspect(video, audio, microphone=microphone)

    def test_continuous_decoded_motion_and_pcm_with_irregular_markers_pass(self):
        video, audio = observations()
        report = inspect(video, audio, microphone=True)
        self.assertEqual(report['video_frames'], 120)
        self.assertAlmostEqual(report['video_fps'], 5.0)
        self.assertEqual(report['matched_markers'], len(MARKERS))
        self.assertEqual(report['median_offset_ms'], 0)
        self.assertEqual(report['maximum_offset_ms'], 0)

    def test_silent_recording_needs_no_audio_or_sync_markers(self):
        video, audio = observations(microphone=False, markers=())
        report = inspect(video, audio, microphone=False)
        self.assertEqual(report['matched_markers'], 0)
        self.assertIsNone(report['median_offset_ms'])
        self.assertIsNone(report['maximum_offset_ms'])

    def test_common_positive_or_negative_clock_origin_changes_no_measurement(self):
        for origin in [-50 * SECOND, 123 * SECOND]:
            with self.subTest(origin=origin):
                video, audio = observations()
                for row in video['frames'] + audio['chunks']:
                    row['pts_ns'] += origin
                report = inspect(video, audio, microphone=True)
                self.assertAlmostEqual(report['video_fps'], 5.0)
                self.assertEqual(report['matched_markers'], len(MARKERS))
                self.assertEqual(report['maximum_offset_ms'], 0)

    def test_exactly_four_point_five_fps_passes_and_less_is_refused(self):
        video, audio = observations(microphone=False, frame_count=108)
        self.assertAlmostEqual(inspect(video, audio, microphone=False)['video_fps'], 4.5)
        video, audio = observations(microphone=False, frame_count=107)
        self.refuse(video, audio, microphone=False)
        video, audio = observations(microphone=False, frame_count=108)
        video['frames'][-1]['duration_ns'] += MILLISECOND
        self.refuse(video, audio, microphone=False)

    def test_exceeding_the_selected_five_fps_ceiling_is_refused(self):
        video, audio = observations(microphone=False, frame_count=121)
        self.refuse(video, audio, microphone=False)

    def test_uncovered_video_gap_of_four_hundred_ms_is_the_limit(self):
        video, audio = observations(microphone=False)
        del video['frames'][35:37]
        inspect(video, audio, microphone=False)
        video, audio = observations(microphone=False)
        del video['frames'][35:38]
        # Average fps still exceeds 4.5: the gap needs its own check.
        self.refuse(video, audio, microphone=False)
        video, audio = observations(microphone=False)
        del video['frames'][35:37]
        video['frames'][35]['pts_ns'] += MILLISECOND
        video['frames'][35]['duration_ns'] -= MILLISECOND
        self.refuse(video, audio, microphone=False)

    def test_extending_a_held_picture_cannot_conceal_missing_motion(self):
        video, audio = observations(microphone=False)
        del video['frames'][35:38]
        video['frames'][34]['duration_ns'] += 600 * MILLISECOND
        self.refuse(video, audio, microphone=False)

    def test_video_pts_must_be_strictly_increasing(self):
        for stamp in [34 * 200 * MILLISECOND, 33 * 200 * MILLISECOND]:
            with self.subTest(stamp=stamp):
                video, audio = observations(microphone=False)
                video['frames'][35]['pts_ns'] = stamp
                self.refuse(video, audio, microphone=False)

    def test_a_static_picture_and_single_padded_frame_are_both_refused(self):
        for static in [False, True]:
            with self.subTest(static=static):
                video, audio = observations(microphone=False)
                if static:
                    for frame in video['frames']:
                        frame['fingerprint'] = 'same-source-picture'
                else:
                    video['frames'][35]['fingerprint'] = video['frames'][34]['fingerprint']
                self.refuse(video, audio, microphone=False)

    def test_expected_track_counts_are_exact_not_at_least_one(self):
        for video_count, audio_count, microphone in [
                (0, 1, True), (2, 1, True), (1, 0, True), (1, 2, True),
                (1, 1, False), (1, 2, False)]:
            with self.subTest(video=video_count, audio=audio_count, microphone=microphone):
                video, audio = observations(microphone=microphone)
                video['track_count'], audio['track_count'] = video_count, audio_count
                self.refuse(video, audio, microphone=microphone)

    def test_empty_decoded_media_is_not_a_completed_recording(self):
        for channel in ['frames', 'chunks']:
            with self.subTest(channel=channel):
                video, audio = observations()
                (video if channel == 'frames' else audio)[channel] = []
                self.refuse(video, audio)

    def test_one_picture_cannot_establish_advancing_motion(self):
        video, audio = observations(microphone=False)
        video['frames'] = video['frames'][:1]
        self.refuse(video, audio, microphone=False)

    def test_a_silent_recording_cannot_hide_pcm_under_zero_track_metadata(self):
        video, audio = observations()
        audio['track_count'] = 0
        self.refuse(video, audio, microphone=False)

    def test_both_present_tracks_must_decode_to_eos(self):
        for channel in ['video', 'audio']:
            with self.subTest(channel=channel):
                video, audio = observations()
                (video if channel == 'video' else audio)['decoded_to_eos'] = False
                self.refuse(video, audio)

    def test_pcm_gap_of_forty_ms_passes_but_forty_one_ms_is_refused(self):
        for extra, passes in [(0, True), (1, False)]:
            with self.subTest(extra=extra):
                video, audio = observations()
                del audio['chunks'][350:354]
                audio['chunks'][350]['pts_ns'] += extra * MILLISECOND
                audio['chunks'][350]['duration_ns'] -= extra * MILLISECOND
                if passes:
                    inspect(video, audio, microphone=True)
                else:
                    self.refuse(video, audio)

    def test_pcm_overlap_of_forty_ms_passes_but_forty_one_ms_is_refused(self):
        for overlap, passes in [(40, True), (41, False)]:
            with self.subTest(overlap=overlap):
                video, audio = observations()
                audio['chunks'][350]['duration_ns'] += overlap * MILLISECOND
                if passes:
                    inspect(video, audio, microphone=True)
                else:
                    self.refuse(video, audio)

    def test_track_endings_may_differ_by_four_hundred_ms_in_either_direction(self):
        for extra in [-400, 400]:
            with self.subTest(extra=extra):
                video, audio = observations()
                if extra < 0:
                    del audio['chunks'][-40:]
                else:
                    audio['chunks'].extend(
                        {'pts_ns': LENGTH + index * 10 * MILLISECOND,
                         'duration_ns': 10 * MILLISECOND, 'tone': False}
                        for index in range(40))
                inspect(video, audio, microphone=True)

    def test_track_ending_difference_above_four_hundred_ms_is_refused(self):
        for extra in [-401, 401]:
            with self.subTest(extra=extra):
                video, audio = observations()
                if extra < 0:
                    del audio['chunks'][-40:]
                    audio['chunks'][-1]['duration_ns'] -= MILLISECOND
                else:
                    audio['chunks'].extend(
                        {'pts_ns': LENGTH + index * 10 * MILLISECOND,
                         'duration_ns': 10 * MILLISECOND, 'tone': False}
                        for index in range(41))
                    audio['chunks'][-1]['duration_ns'] = MILLISECOND
                self.refuse(video, audio)

    def test_ten_irregular_matches_pass_but_nine_do_not(self):
        video, audio = observations(markers=MARKERS[:10])
        self.assertEqual(inspect(video, audio, microphone=True)['matched_markers'], 10)
        video, audio = observations(markers=MARKERS[:9])
        self.refuse(video, audio)

    def test_eighty_percent_coverage_of_each_sequence_is_inclusive(self):
        for missing_channel in ['video', 'audio']:
            with self.subTest(missing_channel=missing_channel):
                video, audio = observations()
                if missing_channel == 'video':
                    set_video_markers(video, MARKERS[3:])
                else:
                    set_audio_markers(audio, MARKERS[3:])
                report = inspect(video, audio, microphone=True)
                self.assertEqual(report['matched_markers'], 12)
                self.assertEqual(report['maximum_offset_ms'], 0)

    def test_enough_matches_cannot_excuse_less_than_eighty_percent_coverage(self):
        for missing_channel in ['video', 'audio']:
            with self.subTest(missing_channel=missing_channel):
                video, audio = observations()
                if missing_channel == 'video':
                    set_video_markers(video, MARKERS[4:])
                else:
                    set_audio_markers(audio, MARKERS[4:])
                # Eleven matches remain, but only 73.3% of the full sequence.
                self.refuse(video, audio)

    def test_a_missing_middle_marker_does_not_shift_later_matches(self):
        for missing_channel in ['video', 'audio']:
            with self.subTest(missing_channel=missing_channel):
                video, audio = observations()
                remaining = MARKERS[:6] + MARKERS[7:]
                if missing_channel == 'video':
                    set_video_markers(video, remaining)
                else:
                    set_audio_markers(audio, remaining)
                report = inspect(video, audio, microphone=True)
                self.assertEqual(report['matched_markers'], 14)
                self.assertEqual(report['maximum_offset_ms'], 0)

    def test_median_absolute_offset_of_two_hundred_ms_is_inclusive(self):
        for offset in [-200, 200]:
            with self.subTest(offset=offset):
                video, audio = observations(markers=MARKERS[:10])
                set_audio_markers(audio, MARKERS[:10], [offset * MILLISECOND] * 10)
                report = inspect(video, audio, microphone=True)
                self.assertEqual(report['matched_markers'], 10)
                self.assertEqual(report['median_offset_ms'], 200)
                self.assertEqual(report['maximum_offset_ms'], 200)

    def test_opposite_signed_offsets_cannot_cancel_an_excessive_absolute_median(self):
        video, audio = observations(markers=MARKERS[:10], audio_chunk_ms=1)
        set_audio_markers(audio, MARKERS[:10], [-201 * MILLISECOND, 201 * MILLISECOND] * 5)
        self.refuse(video, audio)

    def test_maximum_absolute_offset_of_four_hundred_ms_is_inclusive(self):
        for offset in [-400, 400]:
            with self.subTest(offset=offset):
                video, audio = observations(markers=MARKERS[:10])
                set_audio_markers(audio, MARKERS[:10], [offset * MILLISECOND] + [0] * 9)
                report = inspect(video, audio, microphone=True)
                self.assertEqual(report['matched_markers'], 10)
                self.assertEqual(report['median_offset_ms'], 0)
                self.assertEqual(report['maximum_offset_ms'], 400)

    def test_one_excessive_maximum_cannot_hide_behind_a_good_median(self):
        for offset in [-401, 401]:
            with self.subTest(offset=offset):
                video, audio = observations(markers=MARKERS[:10], audio_chunk_ms=1)
                set_audio_markers(audio, MARKERS[:10], [offset * MILLISECOND] + [0] * 9)
                self.refuse(video, audio)

    def test_continuous_pcm_and_a_changing_picture_are_not_sync_evidence_alone(self):
        for channel in ['video', 'audio']:
            for value in [False, True]:
                with self.subTest(channel=channel, constant=value):
                    video, audio = observations()
                    rows, flag = ((video['frames'], 'lit') if channel == 'video'
                                  else (audio['chunks'], 'tone'))
                    for row in rows:
                        row[flag] = value
                    self.refuse(video, audio)

    def test_periodic_pulses_do_not_establish_irregular_marker_sync(self):
        video, audio = observations(markers=tuple(index * SECOND for index in range(1, 20)))
        self.refuse(video, audio)

    def test_observations_are_never_rewritten_on_success_or_refusal(self):
        for broken in [False, True]:
            with self.subTest(broken=broken):
                video, audio = observations()
                if broken:
                    audio['chunks'][350]['duration_ns'] += 100 * MILLISECOND
                original = copy.deepcopy((video, audio))
                if broken:
                    self.refuse(video, audio)
                else:
                    inspect(video, audio, microphone=True)
                self.assertEqual((video, audio), original)

    def test_missing_fields_and_malformed_wrappers_are_refused(self):
        for channel, key in [('video', 'track_count'), ('audio', 'track_count'),
                             ('video', 'decoded_to_eos'), ('audio', 'decoded_to_eos'),
                             ('video', 'frames'), ('audio', 'chunks')]:
            with self.subTest(channel=channel, key=key):
                video, audio = observations()
                del (video if channel == 'video' else audio)[key]
                self.refuse(video, audio)
        for malformed in [None, [], 'decoded']:
            video, audio = observations()
            self.refuse(malformed, audio)
            self.refuse(video, malformed)

    def test_missing_decoded_row_fields_are_refused(self):
        for channel, fields in [('video', ['pts_ns', 'duration_ns', 'fingerprint', 'lit']),
                                ('audio', ['pts_ns', 'duration_ns', 'tone'])]:
            for field in fields:
                with self.subTest(channel=channel, field=field):
                    video, audio = observations()
                    rows = video['frames'] if channel == 'video' else audio['chunks']
                    del rows[35][field]
                    self.refuse(video, audio)

    def test_decoded_sequences_and_their_rows_must_have_the_declared_shape(self):
        for key in ['frames', 'chunks']:
            for value in [None, {}, 'decoded', [None], [123], ['frame']]:
                with self.subTest(key=key, value=value):
                    video, audio = observations()
                    (video if key == 'frames' else audio)[key] = value
                    self.refuse(video, audio)

    def test_invalid_decoded_timing_fields_are_refused(self):
        for channel in ['video', 'audio']:
            for field in ['pts_ns', 'duration_ns']:
                invalid = [None, True, '10000000', 1.5, float('nan'), float('inf'), float('-inf')]
                if field == 'duration_ns':
                    invalid += [0, -1]
                for value in invalid:
                    with self.subTest(channel=channel, field=field, value=value):
                        video, audio = observations()
                        rows = video['frames'] if channel == 'video' else audio['chunks']
                        rows[35][field] = value
                        self.refuse(video, audio)

    def test_flags_counts_and_motion_fingerprints_are_not_coerced(self):
        for channel, field, values in [
                ('video', 'track_count', [True, 1.0, '1']),
                ('audio', 'track_count', [True, 1.0, '1']),
                ('video', 'decoded_to_eos', [1, 'true', None]),
                ('audio', 'decoded_to_eos', [1, 'true', None]),
                ('frame', 'lit', [1, 'false', None]),
                ('chunk', 'tone', [1, 'false', None]),
                ('frame', 'fingerprint', ['', None, 123])]:
            for value in values:
                with self.subTest(channel=channel, field=field, value=value):
                    video, audio = observations()
                    target = {'video': video, 'audio': audio,
                              'frame': video['frames'][35], 'chunk': audio['chunks'][35]}[channel]
                    target[field] = value
                    self.refuse(video, audio)
        for invalid in [None, 0, 1, 'false']:
            video, audio = observations()
            self.refuse(video, audio, microphone=invalid)


if __name__ == '__main__':
    unittest.main()
