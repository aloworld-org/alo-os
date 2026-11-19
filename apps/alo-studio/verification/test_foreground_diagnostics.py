"""Failed native activation must retain bounded facts, never arbitrary app text.

The existing identity, active/showing/visible and one-press policy remains the
acceptance condition. Timeout ValueError text is a strict JSON object, at most
1600 characters: application/title; observed roots_count, bounded roots bus/path
identities, candidate frames with role/states and shell_active; read_error is
null or a bounded node identity and exception kind. No tree text or raw exception
message belongs in the uploaded person-flow diagnostic. These are orchestration
checks, not installed activation or pixel evidence.
"""

import json
import unittest
from unittest.mock import patch

import dbus

from native_controls import activate
from reference_window import foreground
from test_reference_window import (
    DOCK, FRAME, REFERENCE_ROOT, SETTINGS, SHELL, StudioTree, VISIBLE, clock, node,
)

APP = 'org.aloworld.Studio'
TITLE = 'alo Studio'
STUDIO_FRAME = (FRAME[0], '/org/aloworld/Studio/a11y/normal-window')


class ReadFailure(StudioTree):
    """A real transport-shaped failure only after the Dock action succeeds."""
    def facts(self, at):
        if self.pressed and at == STUDIO_FRAME:
            raise ValueError('PRIVATE ARBITRARY APP TEXT ' + 'x' * 5000)
        return super().facts(at)


class ForegroundFailureDiagnostics(unittest.TestCase):
    def failed(self, tree):
        calls = []
        def press(process, matches):
            calls.append(process)
            self.assertTrue(matches(APP))
            self.assertFalse(matches(APP + '.Foreign'))
            activate(tree, process, matches)
        with patch('reference_window.time', clock(tree)):
            with self.assertRaises(ValueError) as caught:
                foreground(tree, press, application=APP, title=TITLE)
        message = str(caught.exception)
        self.assertLessEqual(len(message), 1600)
        self.assertNotIn('PRIVATE ARBITRARY APP TEXT', message)
        self.assertEqual(calls, [42])
        self.assertEqual(tree.actions, [(DOCK, 0)])
        try:
            report = json.loads(message, parse_constant=lambda value: self.fail(value))
        except json.JSONDecodeError:
            self.fail('activation refusal must retain strict bounded JSON facts: ' + message)
        self.assertIsInstance(report, dict)
        self.assertEqual(report['application'], APP)
        self.assertEqual(report['title'], TITLE)
        self.assertEqual(report['reason'], 'window_activation_not_observed')
        return report

    def test_inactive_frame_retains_identity_role_states_and_final_roots(self):
        tree = StudioTree()
        tree.allow_readback = False
        tree.nodes[SETTINGS]['states'] = set(VISIBLE)
        report = self.failed(tree)
        observed = report['observed']
        self.assertEqual(observed['roots_count'], len(tree.roots_now))
        self.assertIn(list(SHELL), observed['roots'])
        self.assertIn(list(REFERENCE_ROOT), observed['roots'])
        self.assertEqual(observed['frames'], [{'role': 23, 'states': [25, 30]}])
        self.assertIs(observed['shell_active'], False)
        self.assertIsNone(report['read_error'])

    def test_shell_obstruction_is_distinct_from_the_active_gtk_frame(self):
        tree = StudioTree()
        tree.keep_shell_active = True
        report = self.failed(tree)
        self.assertIs(report['observed']['shell_active'], True)
        self.assertEqual(report['observed']['frames'], [{'role': 23, 'states': [1, 25, 30]}])

    def test_wrong_role_and_missing_application_are_diagnosable_without_accepting_them(self):
        for failure in ('role', 'missing'):
            with self.subTest(failure=failure):
                tree = StudioTree()
                if failure == 'role':
                    tree.roles[FRAME] = 29
                else:
                    tree.roots_now.remove(REFERENCE_ROOT)
                report = self.failed(tree)
                if failure == 'role':
                    self.assertEqual(report['observed']['frames'],
                                     [{'role': 29, 'states': [1, 25, 30]}])
                else:
                    self.assertEqual(report['observed']['frames'], [])
                    self.assertNotIn(list(REFERENCE_ROOT), report['observed']['roots'])

    def test_failing_read_retains_exact_node_and_exception_type_without_app_text(self):
        report = self.failed(ReadFailure())
        self.assertEqual(report['read_error']['node'], list(STUDIO_FRAME))
        self.assertEqual(report['read_error']['kind'], 'ValueError')

    def test_nested_dbus_failure_retains_only_its_machine_error_name(self):
        class NestedFailure(StudioTree):
            def facts(self, at):
                if self.pressed and at == STUDIO_FRAME:
                    try:
                        raise dbus.DBusException(
                            'PRIVATE ARBITRARY APP TEXT ' + 'x' * 5000,
                            name='org.freedesktop.DBus.Error.UnknownMethod')
                    except dbus.DBusException as error:
                        raise ValueError('the native control could not be read') from error
                return super().facts(at)
        report = self.failed(NestedFailure())
        self.assertEqual(report['read_error']['node'], list(STUDIO_FRAME))
        self.assertEqual(report['read_error']['kind'], 'ValueError')
        self.assertEqual(report['read_error']['error_name'],
                         'org.freedesktop.DBus.Error.UnknownMethod')

    def test_changed_shell_owner_still_refuses_after_exactly_one_press(self):
        class ChangedOwner(StudioTree):
            def roots(self):
                found = super().roots()
                if self.pressed:
                    self.pids[SHELL[0]] = 1234
                return found
        self.failed(ChangedOwner())

    def test_late_observation_replaces_earlier_frame_states(self):
        class Changing(StudioTree):
            def facts(self, at):
                if self.pressed and self.root_reads > self.roots_at_press + 2:
                    self.allow_readback = False
                    self.nodes[FRAME]['states'] = set(VISIBLE)
                return super().facts(at)
        tree = Changing()
        tree.keep_shell_active = True
        report = self.failed(tree)
        self.assertEqual(report['observed']['frames'], [{'role': 23, 'states': [25, 30]}])

    def test_large_inventory_and_unrelated_accessible_text_do_not_expand_the_report(self):
        tree = StudioTree()
        tree.keep_shell_active = True
        tree.nodes[REFERENCE_ROOT]['name'] = 'PRIVATE ARBITRARY APP TEXT ' + 'z' * 10000
        tree.nodes[SETTINGS]['name'] = 'PRIVATE ARBITRARY APP TEXT ' + 'y' * 10000
        for index in range(24):
            root = (f':extra{index}', '/object/' + 'q' * 300 + str(index))
            tree.roots_now.append(root)
            tree.nodes[root] = node('PRIVATE ARBITRARY APP TEXT', toolkit='gtk')
            tree.pids[root[0]] = 100 + index
            tree.roles[root] = 75
        report = self.failed(tree)
        self.assertEqual(report['observed']['roots_count'], len(tree.roots_now))
        self.assertLess(len(report['observed']['roots']), len(tree.roots_now))


if __name__ == '__main__':
    unittest.main()
