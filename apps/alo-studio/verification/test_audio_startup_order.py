"""Startup order for a fresh profile whose default metadata appears with a node.

All process/command observations below are test-only. The metadata shape follows
pw-dump: Metadata props/entries are root fields; Node props live under info.
These checks neither start PipeWire nor establish installed audio acceptance.
"""

import copy
import json
import os
import pathlib
import subprocess
import tempfile
import unittest
from contextlib import ExitStack
from unittest.mock import patch

from installed_session import Session


SETTINGS = dict(id=28, type='PipeWire:Interface:Metadata',
                props={'metadata.name': 'settings'}, metadata=[])
DEFAULT = dict(id=34, type='PipeWire:Interface:Metadata',
               props={'metadata.name': 'default'}, metadata=[
                   dict(subject=0, key='default.audio.source', type='Spa:String:JSON',
                        value={'name': 'studio-verification-microphone'}),
                   dict(subject=0, key='default.configured.audio.sink',
                        type='Spa:String:JSON', value={'name': 'chosen-speakers'}),
                   dict(subject=0, key='default.configured.video.source',
                        type='Spa:String:JSON', value={'name': 'chosen-camera'}),
               ])
MIC = dict(id=93, type='PipeWire:Interface:Node', info={'props': {
    'node.name': 'studio-verification-microphone', 'media.class': 'Audio/Source'}})


def configured():
    result = copy.deepcopy(DEFAULT)
    result['metadata'].append(dict(subject=0, key='default.configured.audio.source',
                                  type='Spa:String:JSON', value={'name': 'old-source'}))
    return result


class Startup:
    def __init__(self, folder, *, initial=None, after=None, dies=False):
        self.flow = object.__new__(Session)
        self.flow.home, self.flow.folder = folder / 'home', folder / 'logs'
        self.flow.folder.mkdir()
        self.flow.events, self.flow.children = [], []
        self.flow.reference, self.flow.audio_overridden = None, False
        self.state_home = folder / 'state'
        self.state_home.mkdir()
        self.trace = []
        self.initial = [SETTINGS] if initial is None else initial
        self.after = [[SETTINGS, DEFAULT]] if after is None else after
        self.post_reads, self.seconds = 0, 0
        self.audio = None
        self.dies = dies
        self.flow.record_provenance = lambda *a: None
        self.flow.upload = lambda *a: self.trace.append(('upload',))
        self.flow.wait = lambda *a, **k: self.trace.append(('ready-wait',))
        def forbidden(*args, **kwargs):
            raise AssertionError('refused startup must not enter permission or recording flow')
        self.flow.verify_agent_permissions = forbidden
        self.flow.recording = forbidden

    def monotonic(self):
        self.seconds += 1
        return self.seconds

    def command(self, *argv):
        self.trace.append(argv)
        if argv == ('pw-dump',):
            if self.audio is None:
                nodes = self.initial
            else:
                self.post_reads += 1
                if self.post_reads > 80:
                    raise AssertionError('metadata readiness must have a finite deadline')
                nodes = self.after[min(self.post_reads - 1, len(self.after) - 1)] + [MIC]
            self.trace.append(('observed', copy.deepcopy(nodes)))
            return json.dumps(nodes)
        if argv in [('wpctl', 'set-default', '93'), ('wpctl', 'clear-default', '1')]:
            return ''
        raise AssertionError(f'unexpected command: {argv!r}')

    def popen(self, argv, **options):
        self.trace.append(tuple(argv))
        owner = self
        class Child:
            def __init__(self, dead=False):
                self.returncode = 1 if dead else None
            def poll(self):
                return self.returncode
            def terminate(self):
                owner.trace.append(('terminate',))
                self.returncode = 0
            def wait(self, **kwargs):
                return self.returncode
        child = Child(self.dies if argv[0] == 'pw-loopback' else False)
        if argv[0] == 'pw-loopback':
            self.audio = child
        elif argv[:2] != ['flatpak', 'run']:
            raise AssertionError(f'unexpected process: {argv!r}')
        return child

    def patched(self):
        stack = ExitStack()
        stack.enter_context(patch.dict(os.environ, XDG_STATE_HOME=str(self.state_home)))
        stack.enter_context(patch('installed_session.command', self.command))
        stack.enter_context(patch('installed_session.subprocess.Popen', self.popen))
        stack.enter_context(patch('installed_session.subprocess.run',
                                  return_value=subprocess.CompletedProcess([], 0)))
        stack.enter_context(patch('installed_session.time.monotonic', self.monotonic))
        stack.enter_context(patch('installed_session.time.sleep'))
        return stack

    def assert_no_override_or_studio(self, test):
        test.assertFalse(any(row[:2] == ('wpctl', 'set-default') for row in self.trace))
        test.assertFalse(any(row[:2] == ('wpctl', 'clear-default') for row in self.trace),
                         'an override never made must not clear a saved preference')
        test.assertFalse(any(row[:2] == ('flatpak', 'run') for row in self.trace))


class AudioStartupOrder(unittest.TestCase):
    def test_owned_node_can_create_default_metadata_but_override_waits_for_complete_readback(self):
        for delay in (0, 2):
            with self.subTest(delayed_reads=delay), tempfile.TemporaryDirectory() as folder:
                fixture = Startup(pathlib.Path(folder),
                                  after=[[SETTINGS]] * delay + [[SETTINGS, DEFAULT]])
                with fixture.patched():
                    fixture.flow.start()
                    fixture.flow.cleanup()
                trace = fixture.trace
                spawn = next(i for i, row in enumerate(trace) if row[0] == 'pw-loopback')
                valid = next(i for i, row in enumerate(trace) if row[0] == 'observed'
                             and DEFAULT in row[1])
                selected = trace.index(('wpctl', 'set-default', '93'))
                studio = next(i for i, row in enumerate(trace) if row[:2] == ('flatpak', 'run'))
                self.assertLess(spawn, valid)
                self.assertLess(valid, selected)
                self.assertLess(selected, studio)
                self.assertEqual(trace.count(('wpctl', 'clear-default', '1')), 1)
                self.assertTrue(all(child.poll() is not None for child in fixture.flow.children))

    def test_persisted_source_history_is_checked_before_fixture_even_without_default_metadata(self):
        for filename in ('default-nodes', 'default-streams'):
            with self.subTest(filename=filename), tempfile.TemporaryDirectory() as folder:
                fixture = Startup(pathlib.Path(folder))
                state = fixture.state_home / 'wireplumber' / filename
                state.parent.mkdir()
                state.write_text(f'[{filename}]\ndefault.configured.audio.source.0=old-source\n')
                with fixture.patched(), self.assertRaisesRegex(ValueError, 'preference|history'):
                    fixture.flow.run()
                self.assertIsNone(fixture.audio, 'saved source history must refuse before fixture spawn')
                fixture.assert_no_override_or_studio(self)

    def test_existing_live_configured_source_still_refuses_before_fixture_spawn(self):
        with tempfile.TemporaryDirectory() as folder:
            fixture = Startup(pathlib.Path(folder), initial=[SETTINGS, configured()])
            with fixture.patched(), self.assertRaisesRegex(ValueError, 'preference'):
                fixture.flow.run()
            self.assertIsNone(fixture.audio)
            fixture.assert_no_override_or_studio(self)

    def test_live_metadata_after_fixture_must_be_complete_unambiguous_and_unconfigured(self):
        malformed = copy.deepcopy(DEFAULT)
        malformed['metadata'] = [None]
        missing_entries = copy.deepcopy(DEFAULT)
        del missing_entries['metadata']
        for after in ([SETTINGS, configured()], [SETTINGS, DEFAULT, DEFAULT],
                      [SETTINGS, malformed], [SETTINGS, missing_entries]):
            with self.subTest(after=after), tempfile.TemporaryDirectory() as folder:
                fixture = Startup(pathlib.Path(folder), after=[after, [SETTINGS, DEFAULT]])
                with fixture.patched(), self.assertRaises(ValueError):
                    fixture.flow.run()
                self.assertIsNotNone(fixture.audio)
                self.assertIsNotNone(fixture.audio.poll(), 'finally must reap the owned fixture')
                self.assertEqual(fixture.post_reads, 1,
                                 'only absent metadata may retry; an observed invalid store must refuse')
                fixture.assert_no_override_or_studio(self)

    def test_a_configured_source_appearing_after_an_initial_automatic_snapshot_is_never_overwritten(self):
        with tempfile.TemporaryDirectory() as folder:
            fixture = Startup(pathlib.Path(folder), initial=[SETTINGS, DEFAULT],
                              after=[[SETTINGS, configured()]])
            with fixture.patched(), self.assertRaises(ValueError):
                fixture.flow.run()
            self.assertIsNotNone(fixture.audio)
            self.assertIsNotNone(fixture.audio.poll())
            fixture.assert_no_override_or_studio(self)

    def test_missing_default_metadata_has_a_deadline_and_stops_the_owned_fixture(self):
        with tempfile.TemporaryDirectory() as folder:
            fixture = Startup(pathlib.Path(folder), after=[[SETTINGS]])
            with fixture.patched(), self.assertRaises(ValueError):
                fixture.flow.run()
            self.assertIsNotNone(fixture.audio)
            self.assertIsNotNone(fixture.audio.poll())
            self.assertLessEqual(fixture.post_reads, 80)
            fixture.assert_no_override_or_studio(self)

    def test_fixture_exit_before_metadata_is_ready_refuses_without_an_override(self):
        with tempfile.TemporaryDirectory() as folder:
            fixture = Startup(pathlib.Path(folder), after=[[SETTINGS]], dies=True)
            with fixture.patched(), self.assertRaises(ValueError):
                fixture.flow.run()
            self.assertIsNotNone(fixture.audio)
            self.assertIsNotNone(fixture.audio.poll())
            fixture.assert_no_override_or_studio(self)


if __name__ == '__main__':
    unittest.main()
