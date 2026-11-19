"""Installed evidence must retain startup failure before a recording exists."""

import json
import pathlib
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from installed_session import Session, command
from agent_settings import NativeSettings


class SessionFailures(unittest.TestCase):
    def test_native_settings_command_stderr_is_retained_but_bounded_in_the_uploaded_failure(self):
        with tempfile.TemporaryDirectory() as folder:
            flow = object.__new__(Session)
            flow.folder, flow.events = pathlib.Path(folder), []
            flow.record_provenance = lambda *a: None
            calls = []
            flow.start = lambda: None
            flow.verify_agent_permissions = lambda: NativeSettings(command).prepare()
            flow.cleanup = lambda: calls.append('cleanup')
            diagnostic = 'fixture native click unavailable: machine action not found\n' + 'x' * 20_000
            diagnostic += 'DIAGNOSTIC_TAIL_SENTINEL'
            def run(argv, **options):
                self.assertTrue(options['capture_output'])
                self.assertTrue(options['check'])
                if tuple(argv) == ('alo-app', 'alo', '--in', 'settings', 'controls'):
                    return subprocess.CompletedProcess(argv, 0,
                        stdout='tab\tRunning code\tselected\n', stderr='')
                self.assertEqual(tuple(argv),
                    ('alo-app', 'alo', '--in', 'settings', 'press', 'tab', 'Running code'))
                raise subprocess.CalledProcessError(1, argv, output='', stderr=diagnostic)
            def upload(name, path):
                calls.append('upload')
                self.assertEqual(name, 'session-report.json')
                events = json.loads(path.read_text())['events']
                self.assertEqual(len(events), 1)
                failure = events[0]
                self.assertEqual(failure['kind'], 'person-flow-failed')
                self.assertEqual(failure['stage'], 'agent-permissions')
                self.assertIn('fixture native click unavailable', failure['message'])
                self.assertLessEqual(len(failure['message']), 2048)
                self.assertNotIn('DIAGNOSTIC_TAIL_SENTINEL', failure['message'])
            flow.upload = upload
            with patch('installed_session.subprocess.run', run):
                with self.assertRaises((subprocess.CalledProcessError, ValueError, RuntimeError)):
                    flow.run()
            self.assertEqual(calls, ['cleanup', 'upload'])

    def test_startup_error_is_retained_in_uploaded_report_after_cleanup(self):
        with tempfile.TemporaryDirectory() as folder:
            flow = object.__new__(Session)
            flow.folder, flow.events = pathlib.Path(folder), []
            flow.record_provenance = lambda *a: None
            calls = []
            def fail():
                raise ValueError('default metadata is unavailable ' + 'x' * 3000)
            flow.start = fail
            flow.cleanup = lambda: calls.append('cleanup')
            def upload(name, path):
                calls.append('upload')
                self.assertEqual(name, 'session-report.json')
                events = json.loads(path.read_text())['events']
                self.assertEqual(len(events), 1)
                event = events[0]
                self.assertEqual(event['kind'], 'person-flow-failed')
                self.assertEqual(event['stage'], 'start')
                self.assertEqual(event['error_type'], 'ValueError')
                self.assertTrue(event['message'].startswith('default metadata is unavailable'))
                self.assertLessEqual(len(event['message']), 2048)
            flow.upload = upload
            with self.assertRaisesRegex(ValueError, 'default metadata'):
                flow.run()
            self.assertEqual(calls, ['cleanup', 'upload'])

    def test_cleanup_failure_is_also_retained_without_a_recording(self):
        with tempfile.TemporaryDirectory() as folder:
            flow = object.__new__(Session)
            flow.folder, flow.events = pathlib.Path(folder), []
            def fail():
                raise ValueError('startup refusal')
            def cleanup():
                raise RuntimeError('cleanup refusal')
            flow.record_provenance = lambda *a: None
            flow.start, flow.cleanup = fail, cleanup
            flow.upload = lambda *a: None
            with self.assertRaises(RuntimeError):
                flow.run()
            events = json.loads((flow.folder / 'session-report.json').read_text())['events']
            self.assertEqual([item['stage'] for item in events], ['start', 'cleanup'])
            self.assertEqual([item['message'] for item in events], ['startup refusal', 'cleanup refusal'])


if __name__ == '__main__':
    unittest.main()
