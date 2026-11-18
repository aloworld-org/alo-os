"""Native restoration must be reachable even when a genuine request fails.

This checks person-controller sequencing with injected UI; it does not establish
native Settings visibility, kernel confinement or an installed application run.
"""

import subprocess
import unittest
from unittest.mock import Mock, patch

import approved_session


class RestoreRendezvous(unittest.TestCase):
    def session(self):
        session = object.__new__(approved_session.ApprovalSession)
        session.settings = Mock(owns_grant=True, owns_policy=True)
        session.runtime = Mock(safe_to_restore=True)
        session.child = None
        session.start = Mock()
        session.event = Mock()
        session.publish = Mock()
        return session

    def test_startup_failure_still_requests_native_settings_before_policy_restoration(self):
        session = self.session()
        trace = []
        session.start.side_effect = ValueError('startup refused')
        session.open_settings = Mock(side_effect=lambda phase: trace.append(('settings', phase)))
        session.settings.restore.side_effect = lambda: trace.append(('restore',))
        with self.assertRaisesRegex(ValueError, '^startup refused$'):
            session.run()
        self.assertEqual(trace, [('settings', 'restore'), ('restore',)])
        session.settings.boundary.assert_not_called()
        session.publish.assert_called_once()

    def test_failed_pending_task_requests_settings_but_only_withdraws_its_own_grant(self):
        session = self.session()
        trace = []
        session.open_settings = Mock(side_effect=lambda phase: trace.append(('settings', phase)))
        session.settings.revoke.side_effect = lambda: trace.append(('revoke',))

        def failed(*args, **kwargs):
            session.runtime.safe_to_restore = False
            raise ValueError('asking process still pending')

        with patch.object(approved_session, 'check_microphone_turn', side_effect=failed):
            with self.assertRaisesRegex(ValueError, '^asking process still pending$'):
                session.run()
        self.assertEqual(trace, [('settings', 'initial'), ('settings', 'restore'), ('revoke',)])
        self.assertEqual([call.args for call in session.settings.boundary.call_args_list],
                         [('ask-before-grant',), ('granted',)])
        session.settings.restore.assert_not_called()
        session.publish.assert_called_once()

    def test_success_moves_the_final_ask_boundary_inside_the_native_cleanup_rendezvous(self):
        session = self.session()
        trace = []
        session.open_settings = Mock(side_effect=lambda phase: trace.append(('settings', phase)))
        session.settings.boundary.side_effect = lambda token: trace.append(('boundary', token))
        session.settings.restore.side_effect = lambda: trace.append(('restore',))
        with patch.object(approved_session, 'check_microphone_turn',
                          side_effect=[{'kind': kind} for kind in ('enable', 'withdraw', 'full')]):
            session.run()
        self.assertEqual(trace, [('settings', 'initial'), ('boundary', 'ask-before-grant'),
                                 ('boundary', 'granted'), ('settings', 'full'), ('boundary', 'full'),
                                 ('settings', 'restore'), ('boundary', 'ask-again'), ('restore',)])
        session.publish.assert_called_once()

    def test_unavailable_cleanup_ui_never_restores_policy_but_still_stops_owned_app_and_reports(self):
        session = self.session()
        session.start.side_effect = ValueError('startup refused')
        session.open_settings = Mock(side_effect=OSError('cleanup Settings unavailable'))
        session.child = Mock()
        session.child.poll.return_value = None
        session.child.wait.return_value = 0
        with patch.object(approved_session.subprocess, 'run',
                          return_value=subprocess.CompletedProcess(['flatpak', 'kill'], 0)) as stop:
            with self.assertRaisesRegex(ValueError, '^startup refused$') as raised:
                session.run()
        session.settings.restore.assert_not_called()
        session.settings.revoke.assert_not_called()
        stop.assert_called_once()
        session.child.wait.assert_called()
        session.publish.assert_called_once()
        self.assertTrue(any('cleanup Settings unavailable' in note
                            for note in getattr(raised.exception, '__notes__', [])))


if __name__ == '__main__':
    unittest.main()
