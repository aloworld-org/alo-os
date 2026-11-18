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
        trace = healthy()
        for row in trace['audio']:
            if 7_000_000_000 <= row['clock_ns'] < 7_200_000_000:
                row['tone'] = False
        report = inspect(trace)
        self.assertGreaterEqual(report['matched_markers'], 10)

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
