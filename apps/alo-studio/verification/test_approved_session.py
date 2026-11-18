"""Private entry, fixed phase order and cleanup ownership for approval tests."""
import os
import unittest
from unittest.mock import Mock, patch

import approved_session


class ApprovedSession(unittest.TestCase):
    def test_restoration_wait_outlasts_the_hosts_failed_phase_deadline(self):
        session = self.session()
        session.open_settings('restore')
        session.runtime.wait_settings.assert_called_once_with(seconds=360)
        session.runtime.reset_mock()
        session.open_settings('initial')
        session.runtime.wait_settings.assert_called_once_with(seconds=30)

    def test_unrequested_or_wrong_user_sessions_refuse_before_process_or_bus_access(self):
        for env, uid, booted in (({}, 1000, True),
                                 ({'STUDIO_PRIVATE_INSTALLED_WALK': '1'}, 0, True),
                                 ({'STUDIO_PRIVATE_INSTALLED_WALK': '1'}, 1000, False)):
            with self.subTest(env=env, uid=uid, booted=booted), patch.dict(os.environ, env, clear=True), patch.object(approved_session.os, 'getuid', return_value=uid), patch.object(approved_session.pathlib.Path, 'exists', return_value=booted), patch.object(approved_session, 'command') as command:
                with self.assertRaises(ValueError):
                    approved_session.ApprovalSession()
                command.assert_not_called()

    def session(self):
        session = object.__new__(approved_session.ApprovalSession)
        session.settings = Mock(owns_grant=False, owns_policy=True)
        session.runtime = Mock(safe_to_restore=True)
        session.child = None
        session.event = Mock()
        session.start = Mock()
        session.publish = Mock()
        return session

    def test_success_walk_uses_exact_three_genuine_requests_then_returns_to_ask(self):
        session = self.session()
        outcomes = [dict(kind=kind) for kind in ('enable', 'withdraw', 'full')]
        with patch.object(approved_session, 'check_microphone_turn', side_effect=outcomes) as turn:
            session.run()
        self.assertEqual([call.kwargs for call in turn.call_args_list], [
            dict(enabled=True, full_access=False), dict(enabled=False, full_access=False, withdraw=True),
            dict(enabled=False, full_access=True)])
        self.assertEqual([call.args for call in session.settings.boundary.call_args_list], [
            ('ask-before-grant',), ('granted',), ('full',), ('ask-again',)])
        session.settings.prepare.assert_called_once()
        session.settings.restore.assert_called_once()
        session.publish.assert_called_once()

    def test_unconfirmed_exit_never_restores_full_trust_or_starts_later_requests(self):
        session = self.session()
        def failed(*args, **kwargs):
            session.runtime.safe_to_restore = False
            raise ValueError('no process exit')
        with patch.object(approved_session, 'check_microphone_turn', side_effect=failed) as turn:
            with self.assertRaisesRegex(ValueError, '^no process exit$'):
                session.run()
        turn.assert_called_once()
        session.settings.restore.assert_not_called()
        session.publish.assert_called_once()

    def test_cleanup_attempts_owned_grant_withdrawal_without_increasing_policy(self):
        session = self.session()
        session.runtime.safe_to_restore = False
        session.settings.owns_grant = True
        with self.assertRaises(ValueError):
            session.cleanup()
        session.settings.revoke.assert_called_once()
        session.settings.restore.assert_not_called()

    def test_report_failure_does_not_replace_the_original_recording_failure(self):
        session = self.session()
        session.start.side_effect = ValueError('startup failed')
        session.publish.side_effect = OSError('report failed')
        with self.assertRaisesRegex(ValueError, '^startup failed$') as raised:
            session.run()
        self.assertTrue(any('report failed' in note for note in raised.exception.__notes__))


if __name__ == '__main__':
    unittest.main()
