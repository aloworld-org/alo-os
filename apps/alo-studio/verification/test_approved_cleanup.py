"""Independent failure checks for private person orchestration, not OS evidence."""

from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import Mock, patch

import dbus

import approved_runtime
import approved_session


class OwnedChild:
    def __init__(self, *, wait_error=None):
        self.calls = []
        self.returncode = None
        self.wait_error = wait_error

    def poll(self):
        return self.returncode

    def wait(self, *, timeout):
        self.calls.append(('wait', timeout))
        if self.returncode is None:
            if self.wait_error is not None:
                raise self.wait_error
            raise subprocess.TimeoutExpired('owned Studio', timeout)
        return self.returncode

    def kill(self):
        self.calls.append(('kill',))
        self.returncode = -9


class ApprovalCleanup(unittest.TestCase):
    def session(self, *, safe=True, owns_grant=False):
        session = object.__new__(approved_session.ApprovalSession)
        session.settings = Mock(owns_grant=owns_grant, owns_policy=True)
        session.runtime = Mock(safe_to_restore=safe)
        session.child = OwnedChild()
        session.event = Mock()
        return session

    def assert_reaped(self, child):
        self.assertIn(('kill',), child.calls)
        self.assertIsNotNone(child.poll())
        self.assertTrue(any(call[0] == 'wait' for call in child.calls))
        self.assertEqual(child.calls[-1][0], 'wait')

    def test_flatpak_kill_failure_cannot_skip_owned_launcher_cleanup(self):
        for error in (OSError('owned kill failed'),
                      subprocess.TimeoutExpired(['flatpak', 'kill'], 10)):
            with self.subTest(error=type(error).__name__):
                session = self.session()
                with patch.object(approved_session.subprocess, 'run', side_effect=error):
                    with self.assertRaises(type(error)) as raised:
                        session.cleanup()
                self.assertIs(raised.exception, error)
                self.assert_reaped(session.child)

    def test_non_timeout_child_wait_failure_still_attempts_forced_reap(self):
        session = self.session()
        error = OSError('owned wait failed')
        session.child = OwnedChild(wait_error=error)
        with patch.object(approved_session.subprocess, 'run'):
            with self.assertRaisesRegex(OSError, '^owned wait failed$'):
                session.cleanup()
        self.assert_reaped(session.child)

    def test_nonzero_flatpak_stop_cannot_be_reported_as_successful_cleanup(self):
        session = self.session()
        session.child.wait = Mock(return_value=0)
        with patch.object(approved_session.subprocess, 'run',
                          return_value=subprocess.CompletedProcess(['flatpak', 'kill'], 1)):
            with self.assertRaises((ValueError, OSError, subprocess.CalledProcessError)):
                session.cleanup()
        session.child.wait.assert_called()

    def test_unconfirmed_agent_exit_never_increases_policy_even_when_app_stop_also_fails(self):
        session = self.session(safe=False, owns_grant=True)
        with patch.object(approved_session.subprocess, 'run', side_effect=OSError('owned stop failed')):
            with self.assertRaisesRegex(ValueError, 'actual agent exit is unconfirmed') as raised:
                session.cleanup()
        session.settings.revoke.assert_called_once()
        session.settings.restore.assert_not_called()
        self.assertTrue(any('owned stop failed' in note
                            for note in getattr(raised.exception, '__notes__', [])))
        self.assert_reaped(session.child)

    def test_native_revoke_failure_still_attempts_app_cleanup_and_never_restores_full(self):
        session = self.session(safe=False, owns_grant=True)
        session.settings.revoke.side_effect = ValueError('native revoke failed')
        with patch.object(approved_session.subprocess, 'run') as stop:
            with self.assertRaisesRegex(ValueError, '^native revoke failed$'):
                session.cleanup()
        stop.assert_called_once()
        session.settings.restore.assert_not_called()
        self.assert_reaped(session.child)

    def test_confirmed_exit_allows_restore_but_later_evidence_failure_is_retained_and_published(self):
        session = self.session()
        session.child = None
        session.start = Mock()
        session.open_settings = Mock()
        session.publish = Mock()
        failure = ValueError('record evidence failed after actual exit')
        with patch.object(approved_session, 'check_microphone_turn', side_effect=failure) as turn:
            with self.assertRaisesRegex(ValueError, '^record evidence failed after actual exit$'):
                session.run()
        turn.assert_called_once()
        session.settings.restore.assert_called_once()
        session.publish.assert_called_once()
        self.assertTrue(any(call.args == ('approved-flow-failed',)
                            and call.kwargs.get('stage') == 'approved-enable'
                            for call in session.event.call_args_list))


class ApprovalBounds(unittest.TestCase):
    def test_native_withdrawal_rereads_the_exact_disable_card_on_both_sides(self):
        runtime = approved_runtime.ApprovalRuntime(Mock(), Mock())
        card = {'number': 0, 'sentence': 'disable the microphone for this alo Studio recording'}
        for before, after in (({**card, 'sentence': 'enable the microphone for this alo Studio recording'}, card),
                              (card, {**card, 'sentence': 'enable the microphone for this alo Studio recording'})):
            with self.subTest(before=before, after=after), \
                    patch.object(runtime, 'wait_settings'), \
                    patch.object(runtime, 'pending', side_effect=[before, after]) as pending, \
                    patch.object(runtime, 'say') as say:
                runtime.settings.reset_mock()
                with self.assertRaises(ValueError):
                    runtime.withdraw(card)
                self.assertTrue(all(call.args == (False,) for call in pending.call_args_list))
                self.assertEqual(runtime.settings.boundary.called, before == card)
                say.assert_not_called()

    def test_delivery_failure_while_waiting_cannot_release_a_pending_card(self):
        runtime = approved_runtime.ApprovalRuntime(Mock(), Mock())
        asking, delivery = Mock(), Mock()
        delivery.pump.side_effect = ValueError('delivery owner lost')
        with patch.object(runtime, 'pending') as pending:
            with self.assertRaisesRegex(ValueError, '^delivery owner lost$'):
                runtime.wait_pending(True, asking, delivery)
        asking.pump.assert_called_once()
        delivery.pump.assert_called_once()
        pending.assert_not_called()

    def test_settings_wait_refuses_after_its_absolute_deadline(self):
        runtime = approved_runtime.ApprovalRuntime(Mock(), Mock())
        missing = subprocess.CalledProcessError(1, ['alo-app'])
        with patch.object(approved_runtime, 'command', side_effect=missing) as command, \
                patch.object(approved_runtime.time, 'monotonic', side_effect=[0, 0, 31]), \
                patch.object(approved_runtime.time, 'sleep'):
            with self.assertRaisesRegex(ValueError, 'Settings window did not open'):
                runtime.wait_settings()
        self.assertEqual(command.call_count, 2)
        self.assertTrue(all(call.args == ('alo-app', 'alo', '--in', 'settings', 'controls')
                            for call in command.call_args_list))

    def test_studio_startup_has_a_deadline_and_retains_ownership_for_cleanup(self):
        session = object.__new__(approved_session.ApprovalSession)
        session.runtime = Mock()
        session.runtime.require_status.side_effect = dbus.DBusException('not yet owned')
        session.event = Mock()
        child = OwnedChild()
        with tempfile.TemporaryDirectory() as directory:
            session.folder = Path(directory)
            with patch.object(approved_session.pathlib.Path, 'read_bytes',
                              side_effect=[b'a' * 40 + b'\n', b'normal; verification disabled\n']), \
                    patch.object(approved_session, 'command', return_value='b' * 64), \
                    patch.object(approved_session.subprocess, 'Popen', return_value=child), \
                    patch.object(approved_session.time, 'monotonic', side_effect=[0, 31]):
                with self.assertRaisesRegex(ValueError, 'ready state'):
                    session.start()
        self.assertIs(session.child, child)
        session.runtime.require_status.assert_called_once_with(False)

    def test_dead_owned_app_cannot_be_hidden_by_a_responding_status(self):
        session = object.__new__(approved_session.ApprovalSession)
        session.runtime = Mock()
        session.event = Mock()
        child = OwnedChild()
        child.returncode = 0
        with tempfile.TemporaryDirectory() as directory:
            session.folder = Path(directory)
            with patch.object(approved_session.pathlib.Path, 'read_bytes',
                              side_effect=[b'a' * 40 + b'\n', b'normal; verification disabled\n']), \
                    patch.object(approved_session, 'command', return_value='b' * 64), \
                    patch.object(approved_session.subprocess, 'Popen', return_value=child):
                with self.assertRaisesRegex(ValueError, 'exited during startup'):
                    session.start()
        session.runtime.require_status.assert_not_called()

    def test_live_recording_or_preview_cannot_satisfy_option_only_status(self):
        runtime = approved_runtime.ApprovalRuntime(Mock(), Mock())
        baseline = dict(version=1, phase='ready', microphone_enabled=False,
                        microphone_active=False, preview_ready=False)
        for key, bad in (('version', True), ('version', 2), ('phase', 'recording'),
                         ('microphone_enabled', 0), ('microphone_active', True),
                         ('preview_ready', True)):
            with self.subTest(key=key), patch.object(runtime, 'status',
                                                   return_value={**baseline, key: bad}):
                with self.assertRaises(ValueError):
                    runtime.require_status(False)
        runtime.event.assert_not_called()


if __name__ == '__main__':
    unittest.main()
