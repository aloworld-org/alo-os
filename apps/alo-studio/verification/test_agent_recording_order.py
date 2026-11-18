"""The installed controller must finish agent refusal checks before capture."""

import pathlib
import tempfile
import unittest
from unittest.mock import patch

from installed_session import Session


class AgentRecordingOrder(unittest.TestCase):
    def test_permissions_run_after_launch_and_before_any_capture_and_failure_stops_capture(self):
        for fail in (False, True):
            with self.subTest(fail=fail), tempfile.TemporaryDirectory() as folder:
                flow = object.__new__(Session)
                flow.folder, flow.events = pathlib.Path(folder), []
                calls = []
                flow.record_provenance = lambda *a: calls.append('provenance')
                flow.start = lambda: calls.append('start')
                def permissions():
                    calls.append('permissions')
                    if fail:
                        raise ValueError('test permission check refused')
                flow.verify_agent_permissions = permissions
                flow.refused = lambda *a: calls.append('refusal')
                flow.start_reference = lambda: calls.append('reference')
                flow.stop_reference = lambda: None
                flow.picker = lambda *a, **k: calls.append('picker')
                flow.recording = lambda *a: calls.append('recording')
                flow.cleanup = lambda: calls.append('cleanup')
                flow.upload = lambda *a: calls.append('upload')
                if fail:
                    with self.assertRaisesRegex(ValueError, 'permission check refused'):
                        flow.run()
                    self.assertEqual(calls, ['provenance', 'start', 'permissions', 'cleanup', 'upload'])
                else:
                    flow.run()
                    self.assertEqual(calls[:3], ['provenance', 'start', 'permissions'])
                    self.assertEqual(calls.count('recording'), 2)
                    self.assertEqual(calls[-2:], ['cleanup', 'upload'])

    def test_only_ready_inactive_recording_can_enter_probe_and_must_remain_unchanged(self):
        ready = dict(version=1, phase='ready', microphone_enabled=False, microphone_active=False)
        bad = [dict(ready, phase='recording'), dict(ready, microphone_enabled=True),
               dict(ready, microphone_active=True)]
        for before, after in [(ready, ready), *[(state, ready) for state in bad],
                              *[(ready, state) for state in bad]]:
            with self.subTest(before=before, after=after):
                flow = object.__new__(Session)
                states = iter([before, after])
                flow.status = lambda: next(states)
                flow.event = lambda *a, **k: None
                with patch('installed_session.run_permission_check', create=True) as run:
                    if before != ready or after != ready:
                        with self.assertRaises(ValueError):
                            flow.verify_agent_permissions()
                    else:
                        flow.verify_agent_permissions()
                    self.assertEqual(run.call_count, int(before == ready))


if __name__ == '__main__':
    unittest.main()
