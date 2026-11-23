"""Saved media must survive a later installed native playback refusal.

Caller checks instrument orchestration only; native helper transport and actual
local normal-app keyboard/rendering tests are separate evidence.
"""

import unittest
from unittest.mock import patch

from installed_session import Session
from test_installed_session import RecordingFlow


class PlaybackFlow(RecordingFlow):
    def __init__(self, *, fail=False):
        super().__init__()
        self.fail = fail

    def upload(self, name, path):
        self.calls.append(('uploaded', name))

    def playback(self, identifier):
        self.calls.append(('playback', identifier))
        if self.fail:
            raise ValueError('actual playback position did not advance')


class InstalledPlaybackOrder(unittest.TestCase):
    def test_both_sources_upload_media_before_native_playback_and_check_reopened_recording(self):
        for source, microphone in (('screen', False), ('window', True)):
            with self.subTest(source=source), patch('installed_session.time.sleep'):
                flow = PlaybackFlow()
                flow.recording(source, microphone)
                playback = ('playback', '1-2-3')
                self.assertEqual(flow.calls.count(playback), 1)
                where = flow.calls.index(playback)
                self.assertLess(flow.calls.index('ReviewRecording'), where)
                self.assertLess(flow.calls.index('stop-reference'), where)
                expected = [source + '.webm', source + '.json']
                if microphone:
                    expected.append('reference.json')
                for name in expected:
                    self.assertLess(flow.calls.index(('uploaded', name)), where)

    def test_playback_failure_keeps_all_previously_sealed_media_uploads(self):
        for source, microphone in (('screen', False), ('window', True)):
            with self.subTest(source=source), patch('installed_session.time.sleep'):
                flow = PlaybackFlow(fail=True)
                with self.assertRaisesRegex(ValueError, 'position did not advance'):
                    flow.recording(source, microphone)
                self.assertIn(('uploaded', source + '.webm'), flow.calls)
                self.assertIn(('uploaded', source + '.json'), flow.calls)
                if microphone:
                    self.assertIn(('uploaded', 'reference.json'), flow.calls)

    def test_session_uses_observed_accessibility_owner_after_normal_dock_foreground(self):
        flow = object.__new__(Session)
        flow.tree, flow.press = object(), object()
        flow.status = lambda: dict(phase='review', recording_id='1-2-3')
        flow.process = lambda *_: self.fail('session-bus proxy PID is not the AT-SPI owner')
        events, calls = [], []
        flow.event = lambda kind, **facts: events.append((kind, facts))
        result = dict(scope='review-playback-position', process=77, start=0.,
                      playing=.8, paused=.9, stable=.9)
        def foreground(tree, press, **kwargs):
            self.assertIs(tree, flow.tree)
            self.assertIs(press, flow.press)
            self.assertEqual(kwargs, dict(application='org.aloworld.Studio', title='alo Studio'))
            calls.append('foreground')
            return dict(name='alo Studio', process=77)
        def observe(tree, process, press):
            self.assertIs(tree, flow.tree)
            self.assertEqual(process, 77)
            self.assertIs(press, flow.press)
            calls.append('playback')
            return result
        with patch('installed_session.foreground', foreground), \
             patch('installed_session.check_playback', observe, create=True):
            flow.playback('1-2-3')
        self.assertEqual(calls, ['foreground', 'playback'])
        self.assertEqual(events, [('review-playback-position', dict(recording_id='1-2-3', **result))])

    def test_changed_recording_or_failed_foreground_or_observation_cannot_emit_pass_event(self):
        for failure in ('changed-recording', 'wrong-phase', 'foreground', 'playback', 'changed-during-playback'):
            with self.subTest(failure=failure):
                flow = object.__new__(Session)
                flow.tree, flow.press = object(), object()
                status_reads = []
                def status():
                    status_reads.append(True)
                    changed = failure == 'changed-recording' or (
                        failure == 'changed-during-playback' and len(status_reads) > 1)
                    return dict(phase='ready' if failure == 'wrong-phase' else 'review',
                                recording_id='other' if changed else '1-2-3')
                flow.status = status
                events = []
                flow.event = lambda *a, **k: events.append((a, k))
                with patch('installed_session.foreground') as foreground, \
                     patch('installed_session.check_playback', create=True) as observation:
                    foreground.return_value = dict(name='alo Studio', process=77)
                    observation.return_value = dict(scope='review-playback-position', process=77,
                                                    start=0., playing=.8, paused=.9, stable=.9)
                    if failure == 'foreground':
                        foreground.side_effect = ValueError('foreground unavailable')
                    if failure == 'playback':
                        observation.side_effect = ValueError('position unavailable')
                    with self.assertRaises(ValueError):
                        flow.playback('1-2-3')
                    if failure in ('changed-recording', 'wrong-phase'):
                        foreground.assert_not_called()
                        observation.assert_not_called()
                    if failure == 'foreground':
                        observation.assert_not_called()
                self.assertEqual(events, [])


if __name__ == '__main__':
    unittest.main()
