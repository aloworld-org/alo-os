"""Native boundary ordering and refusal checks, not an installed GUI result."""
import unittest
from unittest.mock import Mock, patch

import approved_runtime


CARD = {'number': 7, 'sentence': 'disable the microphone for this alo Studio recording'}


class ApprovalRuntime(unittest.TestCase):
    def test_cleanup_wait_is_bounded_and_allows_the_host_to_leave_a_failed_section(self):
        runtime = self.make()
        missing = approved_runtime.subprocess.CalledProcessError(1, ['alo-app'])
        with patch.object(approved_runtime, 'command', side_effect=missing), patch.object(approved_runtime.time, 'monotonic', side_effect=[0, 301, 361]), patch.object(approved_runtime.time, 'sleep'):
            with self.assertRaisesRegex(ValueError, 'Settings window did not open'):
                runtime.wait_settings(seconds=360)
        for seconds in (True, 0, 361, float('inf')):
            with self.subTest(seconds=seconds), patch.object(approved_runtime, 'command') as command:
                with self.assertRaises(ValueError):
                    runtime.wait_settings(seconds=seconds)
                command.assert_not_called()

    def make(self):
        self.settings = Mock()
        self.events = []
        self.runtime = approved_runtime.ApprovalRuntime(self.settings,
            lambda kind, **facts: self.events.append((kind, facts)))
        return self.runtime

    def test_no_existing_card_can_be_reused_as_this_request(self):
        runtime = self.make()
        with patch.object(runtime, 'pending', return_value=CARD), patch.object(runtime, 'say') as say:
            with self.assertRaises(ValueError):
                runtime.request('withdraw')
            say.assert_not_called()

    def test_request_marks_cleanup_unsafe_before_releasing_host_input(self):
        runtime = self.make()
        def observe(message):
            self.assertFalse(runtime.safe_to_restore)
            self.assertEqual(message, 'Studio approval enable request ready')
        with patch.object(runtime, 'pending', return_value=None), patch.object(runtime, 'say', side_effect=observe):
            runtime.request('enable')

    def test_withdrawal_requires_the_same_live_card_before_and_after_native_revoke(self):
        runtime = self.make()
        with patch.object(runtime, 'wait_settings'), patch.object(runtime, 'pending', return_value=CARD) as pending, patch.object(runtime, 'say') as say:
            runtime.withdraw(CARD)
            self.settings.boundary.assert_called_once_with('revoked')
            self.assertEqual(pending.call_count, 2)
            say.assert_called_once_with('Studio approval withdraw grant revoked')

    def test_substituted_or_missing_card_cannot_release_stale_card_input(self):
        for replies, should_revoke in (([None], False), ([CARD, None], True),
                                       ([CARD, {**CARD, 'number': 8}], True)):
            with self.subTest(replies=replies):
                runtime = self.make()
                with patch.object(runtime, 'wait_settings'), patch.object(runtime, 'pending', side_effect=replies), patch.object(runtime, 'say') as say:
                    with self.assertRaises(ValueError):
                        runtime.withdraw(CARD)
                    self.assertEqual(self.settings.boundary.called, should_revoke)
                    say.assert_not_called()

    def test_observer_failures_during_pending_wait_cannot_release_the_card(self):
        runtime = self.make()
        asking, delivery = Mock(), Mock()
        asking.pump.side_effect = ValueError('lost asking evidence')
        with patch.object(runtime, 'pending', return_value=CARD):
            with self.assertRaisesRegex(ValueError, 'lost asking evidence'):
                runtime.wait_pending(False, asking, delivery)

    def test_card_wait_has_an_absolute_deadline(self):
        runtime = self.make()
        with patch.object(runtime, 'pending', return_value=None), patch.object(approved_runtime.time, 'monotonic', side_effect=[0, 0, 121]):
            with self.assertRaises(ValueError):
                runtime.wait_pending(True, Mock(), Mock())

    def test_only_a_successful_actual_exit_marks_policy_restoration_safe(self):
        for facts in ({}, {'result': 'failed', 'exit_code': 1, 'exit_status': 0},
                      {'result': 'success', 'exit_code': 1, 'exit_status': 1}):
            runtime = self.make()
            runtime.safe_to_restore = False
            with self.subTest(facts=facts), self.assertRaises(ValueError):
                runtime.turn_exited(facts)
            self.assertFalse(runtime.safe_to_restore)
        runtime.turn_exited({'result': 'success', 'exit_code': 1, 'exit_status': 0})
        self.assertTrue(runtime.safe_to_restore)


if __name__ == '__main__':
    unittest.main()
