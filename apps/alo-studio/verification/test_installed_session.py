"""The person-flow instrument must actually exercise the promised paths.

This tests the instrument, not an installed app. Guest acceptance is separate.
"""

import json
import pathlib
import os
import subprocess
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


class AudioRestoration(unittest.TestCase):
    """Only mocked orchestration: never launch audio, Flatpak or an installed VM."""

    def fixture(self, folder, *, configured=False):
        flow = object.__new__(Session)
        flow.home = folder / 'home'
        flow.folder = folder / 'logs'
        flow.folder.mkdir()
        flow.events, flow.children = [], []
        flow.reference = None
        flow.record_provenance = lambda *args: None
        flow.verify_agent_permissions = lambda: None
        flow.wait = lambda *args, **kwargs: None
        flow.refused = flow.start_reference = flow.stop_reference = lambda *args: None
        flow.picker = lambda *args, **kwargs: None
        flow.recording = lambda *args: None
        flow.upload = lambda *args: None
        trace = []
        state_home = folder / 'selected-state-home'
        state_home.mkdir()
        default = dict(id=34, type='PipeWire:Interface:Metadata',
                       props={'metadata.name': 'default'}, metadata=[
                           dict(subject=0, key='default.audio.source',
                                type='Spa:String:JSON', value={'name': 'automatic-source'}),
                           dict(subject=0, key='default.configured.audio.sink',
                                type='Spa:String:JSON', value={'name': 'chosen-speakers'}),
                           dict(subject=0, key='default.configured.video.source',
                                type='Spa:String:JSON', value={'name': 'chosen-camera'}),
                       ])
        if configured:
            default['metadata'].append(dict(subject=0, key='default.configured.audio.source',
                                             type='Spa:String:JSON', value={'name': 'old-source'}))
        nodes = [default]

        class Child:
            def __init__(self):
                self.returncode = None

            def poll(self):
                return self.returncode

            def terminate(self):
                self.returncode = 0

            def wait(self, **kwargs):
                return self.returncode

        def popen(args, **kwargs):
            trace.append(tuple(args))
            if args[0] == 'pw-loopback':
                nodes.append(dict(id=93, type='PipeWire:Interface:Node', info={'props': {
                    'node.name': 'studio-verification-microphone', 'media.class': 'Audio/Source'}}))
            return Child()

        def command(*args):
            trace.append(args)
            if args == ('pw-dump',):
                return json.dumps(nodes)
            if args == ('wpctl', 'set-default', '93'):
                default['metadata'].append(dict(subject=0, key='default.configured.audio.source',
                    type='Spa:String:JSON', value={'name': 'studio-verification-microphone'}))
                return ''
            if args == ('wpctl', 'clear-default', '1'):
                default['metadata'][:] = [v for v in default['metadata']
                                           if v['key'] != 'default.configured.audio.source']
                return ''
            raise AssertionError(f'unexpected private instrument command: {args!r}')

        return flow, trace, state_home, command, popen

    def test_preconfigured_source_refuses_before_spawning_or_overriding_audio(self):
        with tempfile.TemporaryDirectory() as directory:
            flow, trace, state_home, command, popen = self.fixture(pathlib.Path(directory), configured=True)
            with patch.dict(os.environ, XDG_STATE_HOME=str(state_home)), \
                    patch('installed_session.command', command), \
                    patch('installed_session.subprocess.Popen', popen):
                with self.assertRaises(ValueError):
                    flow.start()
            self.assertFalse(any(args[0] in ('pw-loopback', 'wpctl') for args in trace))

    def test_either_wireplumber_state_filename_with_source_history_refuses_before_spawn(self):
        for filename in ['default-nodes', 'default-streams']:
            with self.subTest(filename=filename), tempfile.TemporaryDirectory() as directory:
                flow, trace, state_home, command, popen = self.fixture(pathlib.Path(directory))
                state = state_home / 'wireplumber' / filename
                state.parent.mkdir()
                state.write_text(f'[{filename}]\ndefault.configured.audio.source.0=old-source\n')
                with patch.dict(os.environ, XDG_STATE_HOME=str(state_home)), \
                        patch('installed_session.command', command), \
                        patch('installed_session.subprocess.Popen', popen):
                    with self.assertRaises(ValueError):
                        flow.start()
                self.assertFalse(any(args[0] in ('pw-loopback', 'wpctl') for args in trace))

    def test_empty_xdg_state_home_uses_the_home_default_before_overriding_audio(self):
        for filename in ['default-nodes', 'default-streams']:
            with self.subTest(filename=filename), tempfile.TemporaryDirectory() as directory:
                flow, trace, _, command, popen = self.fixture(pathlib.Path(directory))
                state = flow.home / '.local/state/wireplumber' / filename
                state.parent.mkdir(parents=True)
                state.write_text(f'[{filename}]\ndefault.configured.audio.source.0=old-source\n')
                with patch.dict(os.environ, XDG_STATE_HOME=''), \
                        patch('installed_session.command', command), \
                        patch('installed_session.subprocess.Popen', popen):
                    with self.assertRaises(ValueError):
                        flow.start()
                self.assertFalse(any(args[0] in ('pw-loopback', 'wpctl') for args in trace))

    def test_source_only_clear_runs_after_success_or_failure_once_override_occurred(self):
        for failure in [None, 'after-override', 'during-recording']:
            with self.subTest(failure=failure), tempfile.TemporaryDirectory() as directory:
                flow, trace, state_home, command, popen = self.fixture(pathlib.Path(directory))

                def fail(*args, **kwargs):
                    raise ValueError('test-only person flow failure')

                if failure == 'after-override':
                    flow.wait = fail
                elif failure == 'during-recording':
                    flow.recording = fail
                with patch.dict(os.environ, XDG_STATE_HOME=str(state_home)), \
                        patch('installed_session.command', command), \
                        patch('installed_session.subprocess.Popen', popen), \
                        patch('installed_session.subprocess.run', return_value=subprocess.CompletedProcess([], 0)), \
                        patch('installed_session.record_provenance', create=True):
                    if failure:
                        with self.assertRaisesRegex(ValueError, 'person flow failure'):
                            flow.run()
                    else:
                        flow.run()
                self.assertIn(('wpctl', 'set-default', '93'), trace)
                self.assertEqual(trace.count(('wpctl', 'clear-default', '1')), 1)
                self.assertLess(trace.index(('wpctl', 'set-default', '93')),
                                trace.index(('wpctl', 'clear-default', '1')))
                self.assertTrue(all(child.poll() is not None for child in flow.children))

    def test_refusal_before_any_override_does_not_clear_an_existing_preference(self):
        with tempfile.TemporaryDirectory() as directory:
            flow, trace, state_home, command, popen = self.fixture(pathlib.Path(directory), configured=True)
            with patch.dict(os.environ, XDG_STATE_HOME=str(state_home)), \
                    patch('installed_session.command', command), \
                    patch('installed_session.subprocess.Popen', popen), \
                    patch('installed_session.subprocess.run', return_value=subprocess.CompletedProcess([], 0)), \
                    patch('installed_session.record_provenance', create=True):
                with self.assertRaises(ValueError):
                    flow.run()
            self.assertFalse(any(args[:2] == ('wpctl', 'clear-default') for args in trace))

    def test_restore_refusal_fails_the_walk_and_still_stops_owned_children(self):
        with tempfile.TemporaryDirectory() as directory:
            flow, trace, state_home, command, popen = self.fixture(pathlib.Path(directory))

            def clear_fails(*args):
                if args == ('wpctl', 'clear-default', '1'):
                    trace.append(args)
                    raise RuntimeError('test-only audio restoration refusal')
                return command(*args)

            with patch.dict(os.environ, XDG_STATE_HOME=str(state_home)), \
                    patch('installed_session.command', clear_fails), \
                    patch('installed_session.subprocess.Popen', popen), \
                    patch('installed_session.subprocess.run', return_value=subprocess.CompletedProcess([], 0)):
                with self.assertRaisesRegex(RuntimeError, 'audio restoration refusal'):
                    flow.run()
            self.assertTrue(flow.children)
            self.assertTrue(all(child.poll() is not None for child in flow.children))

    def cleanup_fixture(self, *, child_error=None, clear_error=None):
        flow = object.__new__(Session)
        flow.audio_overridden = True
        flow.event = lambda *args, **kwargs: None
        trace = []

        class Child:
            def __init__(self, first):
                self.first, self.stopped = first, False

            def poll(self):
                return None

            def terminate(self):
                trace.append(('terminate', self.first))
                if self.first and child_error and child_error[0] == 'terminate':
                    raise child_error[1]
                self.stopped = True

            def wait(self, **kwargs):
                trace.append(('wait', self.first))
                if self.first and child_error and child_error[0] == 'wait':
                    raise child_error[1]
                return 0

        flow.children = [Child(True), Child(False)]

        def command(*args):
            trace.append(args)
            if clear_error:
                raise clear_error

        return flow, trace, command

    def test_one_flatpak_kill_timeout_still_attempts_every_other_owned_resource(self):
        flow, trace, command = self.cleanup_fixture()
        refusal = subprocess.TimeoutExpired(['flatpak', 'kill'], 10)

        def kill(args, **kwargs):
            trace.append(tuple(args))
            if args[-1] == 'org.aloworld.Studio':
                raise refusal
            return subprocess.CompletedProcess(args, 0)

        with patch('installed_session.subprocess.run', kill), \
                patch('installed_session.command', command):
            with self.assertRaises(subprocess.TimeoutExpired) as caught:
                flow.cleanup()
        self.assertIs(caught.exception, refusal)
        self.assertIn(('flatpak', 'kill', 'org.aloworld.Studio'), trace)
        self.assertIn(('flatpak', 'kill', 'org.aloworld.Studio.Verification'), trace)
        self.assertEqual(trace.count(('wpctl', 'clear-default', '1')), 1)
        self.assertTrue(all(child.stopped for child in flow.children))

    def test_one_owned_child_cleanup_error_still_attempts_later_children(self):
        for operation in ['terminate', 'wait']:
            with self.subTest(operation=operation):
                refusal = RuntimeError('test-only child cleanup refusal')
                flow, trace, command = self.cleanup_fixture(child_error=(operation, refusal))
                with patch('installed_session.subprocess.run', return_value=subprocess.CompletedProcess([], 0)), \
                        patch('installed_session.command', command):
                    with self.assertRaises(RuntimeError) as caught:
                        flow.cleanup()
                self.assertIs(caught.exception, refusal)
                self.assertIn(('terminate', False), trace)
                self.assertTrue(flow.children[1].stopped)
                self.assertEqual(trace.count(('wpctl', 'clear-default', '1')), 1)

    def test_original_source_clear_refusal_survives_a_later_child_cleanup_error(self):
        original = RuntimeError('test-only original source-clear refusal')
        later = RuntimeError('test-only later child refusal')
        flow, trace, command = self.cleanup_fixture(child_error=('terminate', later), clear_error=original)
        with patch('installed_session.subprocess.run', return_value=subprocess.CompletedProcess([], 0)), \
                patch('installed_session.command', command):
            with self.assertRaises(RuntimeError) as caught:
                flow.cleanup()
        self.assertIs(caught.exception, original)
        self.assertIn(('terminate', False), trace)
        self.assertTrue(flow.children[1].stopped)


if __name__ == '__main__':
    unittest.main()
