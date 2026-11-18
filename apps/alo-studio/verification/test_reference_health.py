"""Acceptance limits for a genuinely observed reference, not generated media."""

import copy
import unittest

from reference_health import inspect


def healthy():
    """Synthetic observations used only to exercise the acceptance checker."""
    marks = {0, 1, 3, 6, 7, 8, 10, 13, 14, 15, 17, 20, 21, 22}
    return {
        'version': 1, 'errors': [],
        'draws': [{'clock_ns': n * 40_000_000,
                   'lit': (n * 40 // 1000 in marks and n * 40 % 1000 < 500)}
                  for n in range(600)],
        'audio': [{'clock_ns': n * 10_000_000, 'duration_ns': 10_000_000,
                   'tone': n // 100 in marks and n % 100 < 20}
                  for n in range(2400)],
    }


class ReferenceHealth(unittest.TestCase):
    def test_observed_irregular_markers_and_continuous_samples_pass(self):
        report = inspect(healthy())
        self.assertGreaterEqual(report['matched_markers'], 10)
        self.assertLessEqual(report['maximum_offset_ms'], 40)

    def test_a_common_clock_origin_preserves_healthy_evidence(self):
        trace = healthy()
        for key in ['draws', 'audio']:
            for row in trace[key]:
                row['clock_ns'] += 123_000_000_000
        report = inspect(trace)
        self.assertGreaterEqual(report['matched_markers'], 10)
        self.assertLessEqual(report['maximum_offset_ms'], 40)

    def test_nine_matches_with_complete_marker_coverage_are_insufficient(self):
        trace = healthy()
        for key, flag in [('draws', 'lit'), ('audio', 'tone')]:
            for row in trace[key]:
                if row['clock_ns'] >= 15_500_000_000:
                    row[flag] = False
        with self.assertRaises(ValueError):
            inspect(trace)

    def test_ten_matches_do_not_excuse_less_than_eighty_percent_coverage(self):
        for key, flag in [('draws', 'lit'), ('audio', 'tone')]:
            with self.subTest(channel=key):
                trace = healthy()
                for row in trace[key]:
                    if row['clock_ns'] // 1_000_000_000 in {7, 13, 20}:
                        row[flag] = False
                # Ten of thirteen observed rising edges remain: enough
                # matches, but only 76.9 percent of the other sequence.
                with self.assertRaises(ValueError):
                    inspect(trace)

    def test_an_audio_capture_gap_is_refused(self):
        trace = healthy()
        del trace['audio'][900:921]
        with self.assertRaisesRegex(ValueError, 'audio.*gap'):
            inspect(trace)

    def test_repeated_or_overlapping_audio_is_refused(self):
        trace = healthy()
        trace['audio'][900]['clock_ns'] -= 60_000_000
        with self.assertRaises(ValueError):
            inspect(trace)

    def test_stalled_drawing_is_refused(self):
        trace = healthy()
        del trace['draws'][150:165]
        with self.assertRaisesRegex(ValueError, 'drawing.*gap'):
            inspect(trace)

    def test_a_static_or_unmatched_reference_is_refused(self):
        for key, flag in [('draws', 'lit'), ('audio', 'tone')]:
            trace = healthy()
            for row in trace[key]:
                row[flag] = False
            with self.assertRaises(ValueError):
                inspect(trace)

    def test_one_missing_pulse_does_not_shift_all_later_matches(self):
        baseline = inspect(healthy())
        trace = healthy()
        for row in trace['audio']:
            if 7_000_000_000 <= row['clock_ns'] < 7_200_000_000:
                row['tone'] = False
        report = inspect(trace)
        self.assertGreaterEqual(report['matched_markers'], 10)
        self.assertEqual(report['matched_markers'], baseline['matched_markers'] - 1)
        self.assertLessEqual(report['maximum_offset_ms'], 40)

    def test_missing_live_coverage_at_either_end_is_refused(self):
        # Plenty of matching pulses survive each truncation. Pulse count and
        # coverage percentages alone cannot establish a whole live reference.
        for key in ['audio', 'draws']:
            for end in ['start', 'finish']:
                with self.subTest(channel=key, end=end):
                    trace = healthy()
                    if end == 'start':
                        trace[key] = [row for row in trace[key]
                                      if row['clock_ns'] >= 600_000_000]
                    else:
                        trace[key] = [row for row in trace[key]
                                      if row['clock_ns'] < 23_400_000_000]
                    with self.assertRaises(ValueError):
                        inspect(trace)

    def test_markers_without_observed_motion_are_refused(self):
        # The irregular audio remains healthy. A drawing callback count alone
        # must not validate a reference whose visible marker never changes.
        for lit in [False, True]:
            with self.subTest(lit=lit):
                trace = healthy()
                for row in trace['draws']:
                    row['lit'] = lit
                with self.assertRaises(ValueError):
                    inspect(trace)

    def test_duplicate_and_reordered_drawing_observations_are_refused(self):
        for reorder in [False, True]:
            with self.subTest(reorder=reorder):
                trace = healthy()
                if reorder:
                    trace['draws'][150], trace['draws'][152] = (
                        trace['draws'][152], trace['draws'][150])
                else:
                    trace['draws'].insert(150, copy.deepcopy(trace['draws'][150]))
                with self.assertRaises(ValueError):
                    inspect(trace)

    def test_nonfinite_timing_cannot_be_counted_as_healthy_observation(self):
        for key, field in [('draws', 'clock_ns'), ('audio', 'clock_ns'),
                           ('audio', 'duration_ns')]:
            for value in [float('nan'), float('inf'), float('-inf')]:
                with self.subTest(channel=key, field=field, value=value):
                    trace = healthy()
                    trace[key][100][field] = value
                    with self.assertRaises(ValueError):
                        inspect(trace)

    def test_nonpositive_audio_durations_are_refused(self):
        for duration in [0, -10_000_000]:
            with self.subTest(duration=duration):
                trace = healthy()
                trace['audio'][100]['duration_ns'] = duration
                with self.assertRaises(ValueError):
                    inspect(trace)

    def test_large_median_offset_within_the_maximum_is_refused(self):
        trace = healthy()
        for row in trace['audio']:
            row['clock_ns'] += 250_000_000
        with self.assertRaises(ValueError):
            inspect(trace)

    def test_large_offsets_and_missing_coverage_are_refused(self):
        trace = healthy()
        for row in trace['audio']:
            row['clock_ns'] += 500_000_000
        with self.assertRaises(ValueError):
            inspect(trace)
        trace = healthy()
        for row in trace['audio']:
            if row['clock_ns'] > 12_000_000_000:
                row['tone'] = False
        with self.assertRaises(ValueError):
            inspect(trace)

    def test_errors_empty_or_truncated_observations_are_refused(self):
        for trace in [{}, {'version': 1, 'errors': [], 'draws': [], 'audio': []}]:
            with self.assertRaises(ValueError):
                inspect(trace)
        trace = healthy()
        trace['errors'].append('Pulse stream failed')
        with self.assertRaises(ValueError):
            inspect(trace)
        trace = copy.deepcopy(healthy())
        trace['audio'] = trace['audio'][:400]
        with self.assertRaises(ValueError):
            inspect(trace)


if __name__ == '__main__':
    unittest.main()
