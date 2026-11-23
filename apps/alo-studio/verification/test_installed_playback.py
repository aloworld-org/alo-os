"""Requirements-first person-side Review action/Value observation checks.

check(tree, process, press) receives the actual AT-SPI owner established by
native Dock foreground readback (Flatpak's session proxy PID may differ).
It reads the unique visible sensitive role51 'Playback position', min/max/current
as finite dbus.Double seconds; uses actual named Play recording/Pause playback;
requires >=0.5 seconds advancement and >=1 second stable paused observation.
It returns scope='review-playback-position', process and numeric observed samples.

These stateful transport fixtures test refusal/orchestration only. They are not
installed playback, rendered-pixel, audible-output or keyboard evidence.
"""

import copy
import math
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import dbus

from native_controls import activate
from review_playback import check

ROOT = (':studio', '/org/a11y/atspi/accessible/root')
FRAME = (':studio', '/org/aloworld/Studio/a11y/frame')
BUTTON = (':studio', '/org/aloworld/Studio/a11y/play')
POSITION = (':studio', '/org/aloworld/Studio/a11y/position')
DUPLICATE = (':studio', '/org/aloworld/Studio/a11y/other-position')
ACCESSIBLE = 'org.a11y.atspi.Accessible'
VALUE = 'org.a11y.atspi.Value'
APPLICATION = 'org.a11y.atspi.Application'


class Clock:
    def __init__(self):
        self.now, self.sleeps = 0., []

    def monotonic(self):
        return self.now

    def sleep(self, seconds):
        if not 0 < seconds <= 1:
            raise AssertionError('playback polling must be bounded')
        self.sleeps.append(seconds)
        self.now += seconds


class Interface:
    def __init__(self, tree, node, interface):
        self.tree, self.node, self.interface_name = tree, node, interface

    def GetRole(self, *, timeout):
        if not 0 < timeout <= 3:
            raise AssertionError('native role read must be bounded')
        return {ROOT: 75, FRAME: 23, BUTTON: 43, POSITION: 51, DUPLICATE: 51}[self.node]

    def Get(self, interface, name, *, timeout):
        if not 0 < timeout <= 3:
            raise AssertionError('native property read must be bounded')
        if interface == APPLICATION and self.node == ROOT:
            return {'ToolkitName': 'GTK', 'Version': '4.20.4'}[name]
        if interface != VALUE or self.node not in (POSITION, DUPLICATE):
            raise AssertionError('only actual selected Value properties should be read')
        self.tree.value_reads += 1
        if self.tree.change_owner and self.tree.played:
            self.tree.pid = 99
        if name in self.tree.bad:
            return self.tree.bad[name]
        result = dbus.Double({'MinimumValue': 0., 'MaximumValue': 24.,
                              'CurrentValue': self.tree.current()}[name])
        if name == 'CurrentValue' and self.tree.change_role_during_values:
            self.tree.position_role = 29
        return result


class Tree:
    """Contract fixture: only real named action dispatch changes player state."""
    def __init__(self):
        self.clock = Clock()
        self.pid = 77
        self.playing = self.played = False
        self.started, self.stopped, self.position = 0., None, 0.
        self.bad, self.actions = {}, []
        self.value_reads = 0
        self.advance, self.drift, self.change_owner = True, False, False
        self.duplicate = False
        self.missing_position = False
        self.position_states = {24, 25, 30}
        self.button_states = {24, 25, 30}
        self.position_role = 51
        self.change_role_during_values = False
        self.replace_position_after_play = False
        self.action_result = True

    def roots(self):
        return [ROOT]

    def owner(self, name):
        return self.pid if name == ':studio' else 999

    def interface(self, node, interface):
        result = Interface(self, node, interface)
        if interface == ACCESSIBLE and node == POSITION and self.position_role != 51:
            result.GetRole = lambda **_: self.position_role
        return result

    def current(self):
        if self.playing and self.advance:
            return min(24., self.position + self.clock.now - self.started)
        if self.drift and self.stopped is not None:
            return self.position + self.clock.now - self.stopped
        return self.position

    def facts(self, node):
        children, actions, states, name = [], [], {24, 25, 30}, ''
        if node == ROOT:
            name, states, children = 'arbitrary-program-name', set(), [FRAME]
        elif node == FRAME:
            name, states, children = 'alo Studio', {1, 24, 25, 30}, [BUTTON, POSITION]
            if self.missing_position:
                children.remove(POSITION)
            if self.replace_position_after_play and self.played:
                children = [BUTTON, DUPLICATE]
            if self.duplicate:
                children.append(DUPLICATE)
        elif node == BUTTON:
            name = 'Pause playback' if self.playing else 'Play recording'
            states, actions = self.button_states, ['click']
        elif node in (POSITION, DUPLICATE):
            name, states = 'Playback position', self.position_states
        return copy.deepcopy(dict(name=name, states=states, actions=actions, children=children))

    def action(self, node, index):
        if node != BUTTON or index != 0:
            raise AssertionError('the helper must not activate a scale or unrelated node')
        action = 'Pause playback' if self.playing else 'Play recording'
        self.actions.append(action)
        if not self.action_result:
            return False
        if self.playing:
            self.position = self.current()
            self.playing = False
            self.stopped = self.clock.now
        else:
            self.playing = self.played = True
            self.started = self.clock.now
        return True


class InstalledPlayback(unittest.TestCase):
    def invoke(self, tree, process=77):
        def press(owner, matches):
            self.assertEqual(owner, process)
            activate(tree, owner, matches)
        with patch('review_playback.time', tree.clock):
            return check(tree, process, press)

    def test_named_native_play_and_pause_establish_only_positional_observation(self):
        tree = Tree()
        result = self.invoke(tree)
        self.assertEqual(tree.actions, ['Play recording', 'Pause playback'])
        self.assertFalse(tree.playing)
        self.assertGreaterEqual(tree.clock.now - tree.stopped, 1)
        self.assertEqual(result['scope'], 'review-playback-position')
        self.assertEqual(result['process'], 77)
        self.assertGreaterEqual(result['playing'] - result['start'], .5)
        self.assertLessEqual(abs(result['stable'] - result['paused']), .08)
        self.assertTrue(all(type(result[key]) in (int, float) and math.isfinite(result[key])
                            for key in ('start', 'playing', 'paused', 'stable')))
        self.assertFalse({'pixels', 'audible', 'installed_acceptance'} & set(result))

    def test_missing_ambiguous_hidden_insensitive_or_wrong_role_position_refuses_before_play(self):
        for kind in ('missing', 'duplicate', 'hidden', 'insensitive', 'wrong-role'):
            with self.subTest(kind=kind):
                tree = Tree()
                if kind == 'missing':
                    tree.missing_position = True
                elif kind == 'duplicate':
                    tree.duplicate = True
                elif kind == 'hidden':
                    tree.position_states.remove(25)
                elif kind == 'insensitive':
                    tree.position_states.remove(24)
                else:
                    tree.position_role = 29
                with self.assertRaises(ValueError):
                    self.invoke(tree)
                self.assertEqual(tree.actions, [])

    def test_numeric_wire_types_bounds_and_finiteness_refuse_before_play(self):
        cases = [(name, value) for name in ('MinimumValue', 'MaximumValue', 'CurrentValue')
                 for value in (True, dbus.Int32(0), '0', dbus.Double(float('nan')),
                               dbus.Double(float('inf')), dbus.Double(float('-inf')))]
        cases += [('MinimumValue', dbus.Double(-1)), ('MaximumValue', dbus.Double(0)),
                  ('CurrentValue', dbus.Double(-.1)), ('CurrentValue', dbus.Double(25))]
        for name, value in cases:
            with self.subTest(name=name, value=value):
                tree = Tree()
                tree.bad[name] = value
                with self.assertRaises(ValueError):
                    self.invoke(tree)
                self.assertEqual(tree.actions, [])

    def test_no_actual_advancement_is_bounded_refused_and_native_pause_still_attempted(self):
        tree = Tree()
        tree.advance = False
        with self.assertRaises(ValueError):
            self.invoke(tree)
        self.assertEqual(tree.actions, ['Play recording', 'Pause playback'])
        self.assertFalse(tree.playing)
        self.assertLessEqual(tree.clock.now, 30)

    def test_position_drift_after_pause_refuses_instead_of_claiming_playback_passed(self):
        tree = Tree()
        tree.drift = True
        with self.assertRaises(ValueError):
            self.invoke(tree)
        self.assertEqual(tree.actions, ['Play recording', 'Pause playback'])

    def test_foreign_owner_and_invalid_pid_never_activate_controls(self):
        for process in (None, True, 0, -1, 88):
            with self.subTest(process=process):
                tree = Tree()
                with self.assertRaises(ValueError):
                    self.invoke(tree, process)
                self.assertEqual(tree.actions, [])

    def test_changed_owner_after_play_is_refused_without_pressing_replacement(self):
        tree = Tree()
        tree.change_owner = True
        with self.assertRaises(ValueError):
            self.invoke(tree)
        self.assertEqual(tree.actions, ['Play recording'])

    def test_native_action_refusal_cannot_be_reported_as_playback_success(self):
        tree = Tree()
        tree.action_result = False
        with self.assertRaises(ValueError):
            self.invoke(tree)

    def test_position_role_changed_during_initial_value_read_cannot_start_playback(self):
        tree = Tree()
        tree.change_role_during_values = True
        with self.assertRaises(ValueError):
            self.invoke(tree)
        self.assertEqual(tree.actions, [], 'the scale role must still be valid after its values were read')

    def test_replaced_scale_node_after_play_cannot_supply_a_different_progress_sequence(self):
        tree = Tree()
        tree.replace_position_after_play = True
        with self.assertRaises(ValueError):
            self.invoke(tree)
        self.assertFalse(tree.playing, 'the owned started player must still be paused on refusal')


if __name__ == '__main__':
    unittest.main()
