"""Test-first orchestration of genuine person-submitted Studio requests.

These injected observations establish ordering and refusal propagation, never
installed/native/model success. Concrete record, bus and lifecycle instruments
have separate real-file/private-bus tests. The runtime owns person-side native
UI/journal rendezvous only; it has no approval protocol or changing Studio API.

microphone_turn.check_microphone_turn(runtime, *, enabled, full_access,
withdraw=False) supports only enable=(True,False,False), withdraw=(False,False,
True), full=(False,True,False). Its constructors use fixed/default buses. All
facts are armed before request(kind) lets host QMP submit the genuine Ask task.
runtime.turn_exited(facts) is called immediately after successful asking.finish,
so outer cleanup can know whether policy restoration is safe even when later
evidence fails. This function never changes/restores policy itself.
"""

from pathlib import Path
import unittest
from unittest.mock import patch

import microphone_turn


TURN = {'unit': 'alo-asking@11-1000-alo.service', 'path': '/unit/11',
        'invocation_id': 'a' * 32, 'pid': 2710, 'start_us': 100,
        'result': 'success', 'exit_code': 1, 'exit_status': 0, 'exit_us': 200}
CARD = {'number': 0, 'sentence': 'enable the microphone for this alo Studio recording'}
DISABLE_CARD = {'number': 7, 'sentence': 'disable the microphone for this alo Studio recording'}
DELIVERY = {'owner': ':1.8', 'method': 'EnableMicrophone',
            'deliveries': 1, 'barrier_observed': True}
REFUSED_DELIVERY = {'owner': ':1.8', 'method': None,
                    'deliveries': 0, 'barrier_observed': True}
RUN = {'agent': 'alo', 'verb': 'studio.enable_microphone', 'recorded_runs': 1,
       'from_approval': 0, 'against': [0], 'at_full_access': False, 'by_full_access': False}
REFUSAL = {'agent': 'alo', 'verb': 'studio.disable_microphone',
           'recorded_refusals': 1, 'how': 'at-the-moment', 'at_full_access': False}


class Scenario:
    """Strict observer/runtime seam: no alternate transport or recorder calls."""
    def __init__(self, test, *, enabled=True, full=False, withdraw=False,
                 fail=None, close_fail=()):
        self.test = test
        self.enabled = enabled
        self.full = full
        self.withdrawn = withdraw
        self.fail = fail
        self.close_fail = close_fail
        self.events = []
        self.record_path = Path('/var/lib/alo/record.jsonl')
        self.checkpoint = object()
        self.card = dict(CARD if enabled else DISABLE_CARD)
        self.turn = dict(TURN)
        self.delivery_facts = dict(REFUSED_DELIVERY if withdraw else DELIVERY)
        if not withdraw and not enabled:
            self.delivery_facts['method'] = 'DisableMicrophone'
        self.record_facts = dict(REFUSAL if withdraw else RUN)
        if full:
            self.record_facts.update(verb='studio.disable_microphone', from_approval=None,
                                     against=[], at_full_access=True, by_full_access=True)
        self.asking = self.Asking(self)
        self.delivery = self.Delivery(self)

    def note(self, event):
        self.events.append(event)
        name = event[0] if isinstance(event, tuple) else event
        if name == self.fail or name in self.close_fail:
            raise RuntimeError('broken ' + name)

    def require_status(self, enabled):
        self.test.assertIs(type(enabled), bool)
        self.note(('status', enabled))

    def request(self, kind):
        self.note(('request', kind))

    def wait_pending(self, enabled, asking, delivery):
        self.test.assertIs(asking, self.asking)
        self.test.assertIs(delivery, self.delivery)
        self.test.assertIs(enabled, self.enabled)
        self.note(('pending', enabled))
        return self.card

    def card_ready(self, kind, card):
        self.test.assertIs(card, self.card)
        self.note(('card-ready', kind, card))

    def withdraw(self, card):
        self.test.assertIs(card, self.card)
        self.note(('withdraw', card))

    def turn_exited(self, facts):
        self.test.assertIs(facts, self.turn)
        self.note(('turn-exited', facts))

    class Asking:
        def __init__(self, scenario):
            self.s = scenario

        def capture_pending(self):
            self.s.note('capture-pending')
            return {key: self.s.turn[key] for key in
                    ('unit', 'path', 'invocation_id', 'pid', 'start_us')}

        def capture_started(self):
            self.s.note('capture-started')
            return {key: self.s.turn[key] for key in
                    ('unit', 'path', 'invocation_id', 'pid', 'start_us')}

        def finish(self):
            self.s.note('turn-finish')
            return self.s.turn

        def close(self):
            self.s.note('asking-close')

    class Delivery:
        def __init__(self, scenario):
            self.s = scenario

        def finish(self, *, enabled):
            self.s.test.assertIs(enabled, self.s.enabled)
            self.s.note(('delivery-finish', enabled))
            return self.s.delivery_facts

        def finish_refused(self):
            self.s.note('delivery-refused')
            return self.s.delivery_facts

        def close(self):
            self.s.note('delivery-close')

    def snapshot(self, path):
        self.test.assertEqual(path, self.record_path)
        self.note('snapshot')
        return self.checkpoint

    def verify_change(self, path, checkpoint, *, enabled, from_approval, full_access):
        self.test.assertEqual(path, self.record_path)
        self.test.assertIs(checkpoint, self.checkpoint)
        self.test.assertIs(enabled, self.enabled)
        self.test.assertIs(full_access, self.full)
        self.test.assertEqual(from_approval, None if self.full else self.card['number'])
        self.note('record-change')
        return self.record_facts

    def verify_refusal(self, path, checkpoint, *, enabled):
        self.test.assertEqual(path, self.record_path)
        self.test.assertIs(checkpoint, self.checkpoint)
        self.test.assertIs(enabled, False)
        self.note('record-refusal')
        return self.record_facts

    def invoke(self):
        def construct_asking():
            self.note('asking-arm')
            return self.asking

        def construct_delivery():
            self.note('delivery-arm')
            return self.delivery

        with patch.multiple(microphone_turn,
                            AskingTurnObservation=construct_asking,
                            DeliveryObservation=construct_delivery,
                            snapshot=self.snapshot,
                            verify_microphone_change=self.verify_change,
                            verify_microphone_refusal=self.verify_refusal):
            return microphone_turn.check_microphone_turn(
                self, enabled=self.enabled, full_access=self.full, withdraw=self.withdrawn)

    def names(self):
        return [event[0] if isinstance(event, tuple) else event for event in self.events]


class GenuineMicrophoneTurn(unittest.TestCase):
    def assert_before(self, scenario, first, second):
        self.assertLess(scenario.names().index(first), scenario.names().index(second), scenario.events)

    def assert_common_success(self, scenario, result, kind):
        self.assertEqual(result, {'kind': kind,
                                 'pending': None if scenario.full else scenario.card,
                                 'turn': scenario.turn, 'delivery': scenario.delivery_facts,
                                 'record': scenario.record_facts})
        self.assertIs(result['turn'], scenario.turn)
        self.assertIs(result['delivery'], scenario.delivery_facts)
        self.assertIs(result['record'], scenario.record_facts)
        for armed in ('snapshot', 'asking-arm', 'delivery-arm'):
            self.assertEqual(scenario.names().count(armed), 1)
            self.assert_before(scenario, armed, 'request')
        self.assertEqual(scenario.names().count('request'), 1)
        self.assertEqual(scenario.names().count('turn-finish'), 1)
        finished = scenario.names().index('turn-finish')
        self.assertEqual(scenario.names()[finished + 1], 'turn-exited')
        self.assertIn(('status', not scenario.enabled), scenario.events[:1])
        self.assertIn('asking-close', scenario.names())
        self.assertIn('delivery-close', scenario.names())

    def test_native_approved_enable_arms_before_input_and_reads_evidence_after_actual_exit(self):
        scenario = Scenario(self)
        result = scenario.invoke()
        self.assert_common_success(scenario, result, 'enable')
        self.assertIn(('request', 'enable'), scenario.events)
        self.assert_before(scenario, 'request', 'pending')
        self.assert_before(scenario, 'pending', 'capture-pending')
        self.assert_before(scenario, 'capture-pending', 'card-ready')
        self.assert_before(scenario, 'card-ready', 'turn-finish')
        self.assert_before(scenario, 'turn-exited', 'delivery-finish')
        self.assert_before(scenario, 'delivery-finish', 'record-change')
        self.assertGreater(scenario.events.index(('status', True)),
                           scenario.names().index('record-change'))
        self.assertNotIn('capture-started', scenario.names())
        self.assertNotIn('withdraw', scenario.names())

    def test_stale_same_card_is_seen_then_withdrawn_then_refused_without_delivery_or_option_change(self):
        scenario = Scenario(self, enabled=False, withdraw=True)
        result = scenario.invoke()
        self.assert_common_success(scenario, result, 'withdraw')
        self.assertIn(('request', 'withdraw'), scenario.events)
        self.assertIs(result['pending'], scenario.card)
        for first, second in (('capture-pending', 'card-ready'), ('card-ready', 'withdraw'),
                              ('withdraw', 'turn-finish'), ('turn-exited', 'delivery-refused'),
                              ('delivery-refused', 'record-refusal')):
            self.assert_before(scenario, first, second)
        self.assertEqual([event for event in scenario.events if isinstance(event, tuple)
                          and event[0] == 'status'], [('status', True), ('status', True)])
        self.assertNotIn('delivery-finish', scenario.names())
        self.assertNotIn('record-change', scenario.names())

    def test_full_real_turn_uses_start_observation_without_inventing_pending_approval(self):
        scenario = Scenario(self, enabled=False, full=True)
        result = scenario.invoke()
        self.assert_common_success(scenario, result, 'full')
        self.assertIn(('request', 'full'), scenario.events)
        self.assert_before(scenario, 'request', 'capture-started')
        self.assert_before(scenario, 'capture-started', 'turn-finish')
        self.assert_before(scenario, 'turn-exited', 'delivery-finish')
        self.assert_before(scenario, 'delivery-finish', 'record-change')
        self.assertEqual([event for event in scenario.events if isinstance(event, tuple)
                          and event[0] == 'status'], [('status', True), ('status', False)])
        for absent in ('pending', 'capture-pending', 'card-ready', 'withdraw', 'record-refusal'):
            self.assertNotIn(absent, scenario.names())

    def test_invalid_or_unpromised_policy_combinations_refuse_before_any_io(self):
        values = [(a, b, c) for a in (False, True) for b in (False, True)
                  for c in (False, True) if (a, b, c) not in
                  ((True, False, False), (False, False, True), (False, True, False))]
        values += [(1, False, False), (True, 0, False), (True, False, 0),
                   (None, False, False), (False, True, None)]
        for enabled, full, withdraw in values:
            with self.subTest(enabled=enabled, full=full, withdraw=withdraw):
                scenario = Scenario(self, enabled=enabled, full=full, withdraw=withdraw)
                with self.assertRaises(ValueError):
                    scenario.invoke()
                self.assertEqual(scenario.events, [])

    def test_each_pre_exit_failure_stops_later_evidence_and_never_announces_safe_exit(self):
        failures = ('snapshot', 'delivery-arm', 'asking-arm', 'request', 'pending',
                    'capture-pending', 'card-ready', 'withdraw', 'turn-finish')
        for failure in failures:
            with self.subTest(failure=failure):
                scenario = Scenario(self, enabled=False, withdraw=True, fail=failure)
                with self.assertRaisesRegex(RuntimeError, '^broken ' + failure + '$'):
                    scenario.invoke()
                for absent in ('turn-exited', 'delivery-refused', 'record-refusal'):
                    self.assertNotIn(absent, scenario.names())
                if 'asking-arm' in scenario.names() and failure != 'asking-arm':
                    self.assertIn('asking-close', scenario.names())
                if 'delivery-arm' in scenario.names() and failure != 'delivery-arm':
                    self.assertIn('delivery-close', scenario.names())

    def test_full_start_capture_failure_cannot_become_a_completed_turn(self):
        scenario = Scenario(self, enabled=False, full=True, fail='capture-started')
        with self.assertRaisesRegex(RuntimeError, 'broken capture-started'):
            scenario.invoke()
        self.assertNotIn('turn-finish', scenario.names())
        self.assertNotIn('turn-exited', scenario.names())
        self.assertNotIn('delivery-finish', scenario.names())

    def test_initial_state_refusal_cannot_start_any_observer_or_native_request(self):
        scenario = Scenario(self, fail='status')
        with self.assertRaisesRegex(RuntimeError, '^broken status$'):
            scenario.invoke()
        self.assertEqual(scenario.events, [('status', False)])

    def test_failed_exit_callback_stops_evidence_but_still_closes_observers(self):
        scenario = Scenario(self, fail='turn-exited')
        with self.assertRaisesRegex(RuntimeError, '^broken turn-exited$'):
            scenario.invoke()
        self.assert_before(scenario, 'turn-finish', 'turn-exited')
        self.assertNotIn('delivery-finish', scenario.names())
        self.assertNotIn('record-change', scenario.names())
        self.assertIn('asking-close', scenario.names())
        self.assertIn('delivery-close', scenario.names())

    def test_successful_exit_is_exposed_even_if_barrier_or_record_evidence_later_refuses(self):
        for failure in ('delivery-finish', 'record-change'):
            with self.subTest(failure=failure):
                scenario = Scenario(self, fail=failure)
                with self.assertRaisesRegex(RuntimeError, '^broken ' + failure + '$'):
                    scenario.invoke()
                self.assert_before(scenario, 'turn-finish', 'turn-exited')
                self.assert_before(scenario, 'turn-exited', failure)
                self.assertNotIn(('status', True), scenario.events)
                self.assertIn('asking-close', scenario.names())
                self.assertIn('delivery-close', scenario.names())

    def test_cleanup_attempts_both_observers_and_preserves_original_failure(self):
        scenario = Scenario(self, fail='turn-finish', close_fail=('asking-close', 'delivery-close'))
        with self.assertRaisesRegex(RuntimeError, '^broken turn-finish$'):
            scenario.invoke()
        self.assertIn('asking-close', scenario.names())
        self.assertIn('delivery-close', scenario.names())
        self.assertNotIn('turn-exited', scenario.names())

    def test_cleanup_failure_itself_refuses_after_attempting_both_observers(self):
        scenario = Scenario(self, close_fail=('asking-close', 'delivery-close'))
        with self.assertRaises(RuntimeError):
            scenario.invoke()
        self.assertIn('asking-close', scenario.names())
        self.assertIn('delivery-close', scenario.names())
        self.assertIn('turn-exited', scenario.names())


if __name__ == '__main__':
    unittest.main()
