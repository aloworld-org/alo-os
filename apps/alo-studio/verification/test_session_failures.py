"""Installed evidence must retain startup failure before a recording exists."""

import json
import pathlib
import tempfile
import unittest

from installed_session import Session


class SessionFailures(unittest.TestCase):
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
