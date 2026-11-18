"""The person-flow instrument must actually exercise the promised paths.

This tests the instrument, not an installed app. Guest acceptance is separate.
"""

import json
import pathlib
import tempfile
import unittest
from unittest.mock import patch

from installed_session import Session


class RecordingFlow(Session):
    def __init__(self):
        self.calls = []
        self.home = pathlib.Path('/home/walker')
        self.library = self.home / 'library'
        self.state = dict(phase='ready', microphone_enabled=False, microphone_active=False,
                          countdown_seconds=0, elapsed_ms=0, recording_id='1-2-3')

    def call(self, method, *args):
        self.calls.append(method)
        if method == 'NewRecording':
            self.state.update(phase='ready', microphone_enabled=False, microphone_active=False)
        elif method == 'EnableMicrophone':
            self.state['microphone_enabled'] = True
        elif method == 'DisableMicrophone':
            self.state['microphone_enabled'] = False
        elif method == 'SetCountdown':
            self.state['countdown_seconds'] = args[0]
        elif method in ('Pause', 'Resume', 'Finish', 'CancelCountdown'):
            self.state['phase'] = {'Pause': 'paused', 'Resume': 'recording',
                                   'Finish': 'review', 'CancelCountdown': 'selected'}[method]
            self.state['microphone_active'] = method == 'Resume' and self.state['microphone_enabled']

    def status(self):
        return dict(self.state)

    def wait(self, predicate, **_):
        state = self.status()
        if not predicate(state):
            raise ValueError('the instrument waited for an impossible test state')
        return state

    def begin(self, phase='recording'):
        self.calls.append('Start')
        self.state.update(phase=phase, microphone_active=self.state['microphone_enabled'])

    def active_for(self, seconds):
        self.calls.append(('active', seconds))
        self.state['elapsed_ms'] += seconds * 1000

    def refused(self, method):
        self.calls.append(('refuse', method))

    def start_reference(self):
        self.calls.append('start-reference')

    def stop_reference(self):
        self.calls.append('stop-reference')

    def picker(self, source):
        self.calls.append(('picker', source))
        self.state['phase'] = 'selected'

    def export(self, *_):
        self.calls.append('export')

    def upload(self, *_):
        pass


class InstalledSession(unittest.TestCase):
    def test_both_recording_sources_are_paused_for_five_seconds_then_resumed(self):
        for source, microphone in [('screen', False), ('window', True)]:
            with self.subTest(source=source), patch('installed_session.time.sleep') as sleep:
                flow = RecordingFlow()
                flow.recording(source, microphone)
                self.assertEqual(flow.calls.count('Pause'), 1)
                self.assertEqual(flow.calls.count('Resume'), 1)
                self.assertLess(flow.calls.index('Pause'), flow.calls.index('Resume'))
                self.assertLess(flow.calls.index('Resume'), flow.calls.index('Finish'))
                sleep.assert_any_call(5)

    def test_five_second_wait_is_between_distinct_frozen_pause_snapshots(self):
        for source, microphone in [('screen', False), ('window', True)]:
            with self.subTest(source=source):
                flow = RecordingFlow()
                status = flow.status

                def observe():
                    state = status()
                    flow.calls.append(('observed', state))
                    return state

                def sleep(seconds):
                    self.assertEqual(flow.state['phase'], 'paused')
                    flow.calls.append(('sleep', seconds))

                flow.status = observe
                with patch('installed_session.time.sleep', sleep):
                    flow.recording(source, microphone)
                calls = flow.calls
                pause, resume = calls.index('Pause'), calls.index('Resume')
                delay = calls.index(('sleep', 5))
                self.assertLess(pause, delay)
                self.assertLess(delay, resume)
                before = [item[1] for item in calls[pause:delay]
                          if isinstance(item, tuple) and item[0] == 'observed']
                after = [item[1] for item in calls[delay:resume]
                         if isinstance(item, tuple) and item[0] == 'observed']
                self.assertTrue(before, 'pause must be observed before waiting')
                self.assertTrue(after, 'pause must be observed again after waiting')
                self.assertIsNot(before[-1], after[0])
                for state in (before[-1], after[0]):
                    self.assertEqual(state['phase'], 'paused')
                    self.assertFalse(state['microphone_active'])
                self.assertEqual(before[-1]['elapsed_ms'], after[0]['elapsed_ms'])
                self.assertTrue(any(isinstance(item, tuple) and item[0] == 'active'
                                    and item[1] > 0 for item in calls[resume + 1:calls.index('Finish')]))

    def test_changed_pause_state_refuses_completion_for_either_source(self):
        for source, microphone in [('screen', False), ('window', True)]:
            for field, value in [('elapsed_ms', 999999), ('phase', 'recording'),
                                 ('microphone_active', True)]:
                with self.subTest(source=source, field=field):
                    flow = RecordingFlow()

                    def change_during_pause(seconds):
                        self.assertEqual(seconds, 5)
                        flow.state[field] = value

                    with patch('installed_session.time.sleep', change_during_pause):
                        with self.assertRaises(ValueError):
                            flow.recording(source, microphone)
                    self.assertNotIn('Resume', flow.calls)
                    self.assertNotIn('Finish', flow.calls)
                    self.assertNotIn('export', flow.calls)

    def test_wrong_resume_phase_or_microphone_refuses_completion(self):
        for source, microphone in [('screen', False), ('window', True)]:
            for change in [dict(phase='paused'), dict(microphone_active=not microphone)]:
                with self.subTest(source=source, change=change):
                    flow = RecordingFlow()
                    call = flow.call

                    def wrong_resume(method, *args):
                        call(method, *args)
                        if method == 'Resume':
                            flow.state.update(change)

                    flow.call = wrong_resume
                    with patch('installed_session.time.sleep'):
                        with self.assertRaises(ValueError):
                            flow.recording(source, microphone)
                    self.assertNotIn('Finish', flow.calls)
                    self.assertNotIn('export', flow.calls)

    def test_export_refuses_a_copy_created_by_cancel_or_a_different_saved_copy(self):
        for failure in ['cancel-created-copy', 'saved-different-bytes']:
            with self.subTest(failure=failure), tempfile.TemporaryDirectory() as folder:
                home = pathlib.Path(folder)
                identifier, title = '1-2-3', 'Demonstration'
                (home / f'{identifier}.json').write_text(json.dumps({'title': title}))
                (home / f'{identifier}.webm').write_bytes(b'test-only original')
                flow = object.__new__(Session)
                flow.home = flow.library = home
                effects = []
                flow.call = lambda method, *args: effects.append(method)
                flow.wait = lambda predicate, **kwargs: None
                flow.event = lambda *args, **kwargs: None

                def command(*words):
                    if words[-1] == 'read':
                        return f'In {home}\n'
                    if words[-1] == 'controls':
                        return 'button\tSave\tready\nbutton\tCancel\tready\n'
                    choice = words[-1]
                    effects.append(choice)
                    if choice == 'Save' or failure == 'cancel-created-copy':
                        (home / f'{title}.webm').write_bytes(b'test-only wrong copy')
                    return ''

                with patch('installed_session.command', command):
                    with self.assertRaises(ValueError):
                        flow.export(identifier, title)
                if failure == 'cancel-created-copy':
                    self.assertEqual(effects, ['RenameRecording', 'ExportRecording', 'Cancel'])
                else:
                    self.assertEqual(effects, ['RenameRecording', 'ExportRecording', 'Cancel',
                                               'ExportRecording', 'Save'])

    def test_export_cancellation_precedes_a_new_native_save_and_creates_no_copy(self):
        with tempfile.TemporaryDirectory(prefix='studio-session-test-') as folder:
            home = pathlib.Path(folder)
            identifier, title = '1-2-3', 'Demonstration'
            (home / f'{identifier}.json').write_text(json.dumps({'title': title}))
            payload = b'inert test-only recording'
            (home / f'{identifier}.webm').write_bytes(payload)
            flow = object.__new__(Session)
            flow.home = flow.library = home
            effects = []
            flow.call = lambda method, *args: effects.append(method)
            flow.wait = lambda predicate, **kwargs: None
            flow.event = lambda *args, **kwargs: None

            def command(*words):
                if words[-1] == 'read':
                    return f'In {home}\n'
                if words[-1] == 'controls':
                    return 'button\tSave\tready\nbutton\tCancel\tready\n'
                choice = words[-1]
                effects.append(choice)
                if choice == 'Cancel':
                    self.assertFalse((home / f'{title}.webm').exists())
                if choice == 'Save':
                    (home / f'{title}.webm').write_bytes(payload)
                return ''

            with patch('installed_session.command', command):
                flow.export(identifier, title)
            self.assertEqual(effects,
                             ['RenameRecording', 'ExportRecording', 'Cancel', 'ExportRecording', 'Save'])


if __name__ == '__main__':
    unittest.main()
