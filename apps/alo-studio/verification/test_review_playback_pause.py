"""AT-SPI dispatch precedes GTK's actual pause acknowledgement.

Real normal-app observation in ack.log found Pause playback still named at
161ms after successful DoAction, with position advancing until the same button
became Play recording at269ms and then stayed stable. GTK4.20.4 gtkbutton.c
ACTIVATE_TIMEOUT is250ms; gtkatspiaction.c returns after gtk_widget_activate.
These test-only observations model that asynchronous boundary, not playback
acceptance. check must await the same owned usable role43 button's Play label
once, then retain the existing >=1second/.08second stability requirement.
No extra Pause, arbitrary retry, changed tolerance or synthesized value is used.
"""

from types import SimpleNamespace
import unittest
from unittest.mock import patch

from native_controls import activate
from review_playback import check, playback_button
from test_installed_playback import ACCESSIBLE, BUTTON, FRAME, Tree

REPLACEMENT = (BUTTON[0], '/org/aloworld/Studio/a11y/replacement-play')
CHILD_LABEL = (BUTTON[0], '/org/aloworld/Studio/a11y/button-label')


class DelayedPause(Tree):
    """Only the delayed native click changes actual playing/position state."""
    def __init__(self, delay=.25):
        super().__init__()
        self.delay, self.pause_at, self.acknowledged = delay, None, False
        self.replace_button = False
        self.ack_states = {24, 25, 30}
        self.ack_role = 43

    def update(self):
        if self.pause_at is not None and self.clock.now >= self.pause_at:
            self.position += self.pause_at - self.started
            self.playing = False
            self.stopped = self.pause_at
            self.acknowledged = True
            self.pause_at = None

    def current(self):
        self.update()
        return super().current()

    def facts(self, at):
        self.update()
        if self.acknowledged and self.replace_button and at == BUTTON:
            raise ValueError('the originally activated button no longer exists')
        facts = super().facts(BUTTON if at == REPLACEMENT else at)
        if self.acknowledged:
            if at == FRAME and self.replace_button:
                facts['children'] = [REPLACEMENT if child == BUTTON else child
                                     for child in facts['children']]
            if at in (BUTTON, REPLACEMENT):
                facts['states'] = set(self.ack_states)
        return facts

    def interface(self, at, interface):
        self.update()
        if interface == ACCESSIBLE and at in (BUTTON, REPLACEMENT):
            return SimpleNamespace(GetRole=lambda **_: self.ack_role if self.acknowledged else 43)
        return super().interface(BUTTON if at == REPLACEMENT else at, interface)

    def action(self, at, index):
        self.update()
        if self.playing:
            if at != BUTTON or index != 0:
                raise AssertionError('only the actual owned playback button may be activated')
            self.actions.append('Pause playback')
            if self.delay is not None:
                self.pause_at = self.clock.now + self.delay
            return True
        return super().action(at, index)


class NativePauseAcknowledgement(unittest.TestCase):
    def invoke(self, tree):
        def press(owner, matches):
            self.assertEqual(owner, 77)
            activate(tree, owner, matches)
        with patch('review_playback.time', tree.clock):
            return check(tree, 77, press)

    def test_dispatched_pause_waits_for_same_button_ack_before_stable_baseline(self):
        tree = DelayedPause()
        result = self.invoke(tree)
        self.assertTrue(tree.acknowledged)
        self.assertEqual(tree.actions, ['Play recording', 'Pause playback'])
        self.assertGreaterEqual(tree.clock.now - tree.stopped, 1)
        self.assertGreaterEqual(result['playing'] - result['start'], .5)
        self.assertLessEqual(abs(result['stable'] - result['paused']), .08)

    def test_actual_gtk_same_named_label_child_does_not_count_as_another_button(self):
        class Labelled(DelayedPause):
            def facts(self, at):
                self.update()
                if at == CHILD_LABEL:
                    return dict(name='Pause playback' if self.playing else 'Play recording',
                                states={24, 25, 30}, actions=[], children=[])
                facts = super().facts(at)
                if at == BUTTON:
                    facts['children'].append(CHILD_LABEL)
                return facts

            def interface(self, at, interface):
                if at == CHILD_LABEL and interface == ACCESSIBLE:
                    return SimpleNamespace(GetRole=lambda **_: 29)
                return super().interface(at, interface)
        tree = Labelled()
        result = self.invoke(tree)
        self.assertTrue(tree.acknowledged)
        self.assertEqual(tree.actions, ['Play recording', 'Pause playback'])
        self.assertGreaterEqual(tree.clock.now - tree.stopped, 1)
        self.assertLessEqual(abs(result['stable'] - result['paused']), .08)

    def test_unacknowledged_pause_refuses_within_bound_without_a_second_toggle(self):
        tree = DelayedPause(delay=None)
        with self.assertRaises(ValueError):
            self.invoke(tree)
        self.assertFalse(tree.acknowledged)
        self.assertEqual(tree.actions, ['Play recording', 'Pause playback'])
        self.assertLessEqual(tree.clock.now, 30)

    def test_real_position_drift_after_button_ack_still_refuses(self):
        tree = DelayedPause()
        tree.drift = True
        with self.assertRaises(ValueError):
            self.invoke(tree)
        self.assertTrue(tree.acknowledged)
        self.assertEqual(tree.actions, ['Play recording', 'Pause playback'])

    def test_replacement_button_cannot_acknowledge_the_original_native_pause(self):
        tree = DelayedPause(delay=.05)
        tree.replace_button = True
        with self.assertRaises(ValueError):
            self.invoke(tree)
        self.assertEqual(tree.actions, ['Play recording', 'Pause playback'])

    def test_hidden_insensitive_or_nonbutton_play_label_cannot_acknowledge_pause(self):
        for failure in ('hidden', 'insensitive', 'wrong-role'):
            with self.subTest(failure=failure):
                tree = DelayedPause(delay=.05)
                if failure == 'hidden':
                    tree.ack_states.remove(25)
                elif failure == 'insensitive':
                    tree.ack_states.remove(24)
                else:
                    tree.ack_role = 29
                with self.assertRaises(ValueError):
                    self.invoke(tree)
                self.assertEqual(tree.actions, ['Play recording', 'Pause playback'])

    def test_role_changed_between_candidate_and_fresh_read_refuses_before_return(self):
        class ChangedRole(DelayedPause):
            def __init__(self):
                super().__init__()
                self.button_reads = 0

            def facts(self, at):
                facts = super().facts(at)
                if at == BUTTON:
                    self.button_reads += 1
                return facts

            def interface(self, at, interface):
                if at == BUTTON and interface == ACCESSIBLE:
                    return SimpleNamespace(GetRole=lambda **_: 29 if self.button_reads >= 2 else 43)
                return super().interface(at, interface)
        tree = ChangedRole()
        with self.assertRaises(ValueError):
            playback_button(tree, 77)
        self.assertGreaterEqual(tree.button_reads, 2)
        self.assertEqual(tree.actions, [])


if __name__ == '__main__':
    unittest.main()
