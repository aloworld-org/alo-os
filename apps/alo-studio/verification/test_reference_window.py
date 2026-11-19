"""A person-side Dock activation needs fresh, unambiguous native readback.

foreground(tree, press) must find the shell by Application.ToolkitName='alo',
press exactly the owned reference's Dock identifier once, then identify its
registered GApplication accessibility path, regardless of program name. Return
the observed {'name', 'process'} only when that frame is active/showing/
visible and no showing shell surface remains active. State fixtures exercise
this policy through actual native_controls.activate; no installed pixels or
capture acceptance are claimed.
"""

import copy
from pathlib import Path
import re
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import installed_session
from installed_session import Session
from native_controls import activate
from reference_window import foreground
from test_installed_session import RecordingFlow


APPLICATION = 'org.a11y.atspi.Application'
PROPERTIES = 'org.freedesktop.DBus.Properties'
ACCESSIBLE = 'org.a11y.atspi.Accessible'
REFERENCE_SOURCE = (Path(installed_session.__file__).resolve().parents[3]
                    / 'apps/alo-studio/examples/studio-reference-window.rs').read_text()
REFERENCE_IDENTITIES = re.findall(r'\.application_id\("([^"]+)"\)', REFERENCE_SOURCE)
REFERENCE_TITLES = re.findall(r'\.title\("([^"]+)"\)', REFERENCE_SOURCE)
if len(REFERENCE_IDENTITIES) != 1 or len(REFERENCE_TITLES) != 1:
    raise AssertionError('the independent reference fixture must declare one ID and title')
REFERENCE, TITLE = REFERENCE_IDENTITIES[0], REFERENCE_TITLES[0]
REFERENCE_PATH = '/' + REFERENCE.replace('.', '/') + '/a11y/'
SHELL = (':shell', '/root')
DOCK = (':shell', '/dock/reference')
SETTINGS = (':shell', '/settings')
REFERENCE_ROOT = (':reference', '/root')
FRAME = (':reference', REFERENCE_PATH + 'window-1')
FOREIGN = (':other', '/reference-dock-label')
VISIBLE = {25, 30}
ACTIVE = {1, 25, 30}


def node(name, *, states=VISIBLE, actions=(), children=(), toolkit=None):
    """The same small facts transport used by the real AtspiTree."""
    return dict(name=name, states=set(states), actions=list(actions),
                children=list(children), toolkit=toolkit)


class Properties:
    def __init__(self, tree, at):
        self.tree, self.at = tree, at

    def Get(self, interface, key, *, timeout):
        if interface != APPLICATION or key != 'ToolkitName' or not 0 < timeout <= 3:
            raise AssertionError('only a bounded native toolkit property is needed')
        return self.tree.nodes[self.at]['toolkit']


class Accessible:
    def __init__(self, tree, at):
        self.tree, self.at = tree, at

    def GetRole(self, *, timeout):
        if not 0 < timeout <= 3:
            raise AssertionError('the native role read must remain bounded')
        return self.tree.roles[self.at]


class NativeTree:
    """An initially covered reference; only its real Dock action changes state."""
    def __init__(self):
        self.roots_now = [SHELL, REFERENCE_ROOT, (':other', '/root')]
        self.nodes = {
            SHELL: node('', children=[DOCK, SETTINGS], toolkit='alo'),
            DOCK: node(REFERENCE, states={8, 24, 25, 30}, actions=['click']),
            SETTINGS: node('Settings', states=ACTIVE),
            REFERENCE_ROOT: node('private-reference-program', children=[FRAME], toolkit='GTK'),
            FRAME: node(TITLE),
            (':other', '/root'): node('Unrelated', children=[FOREIGN], toolkit='gtk'),
            FOREIGN: node(REFERENCE, states={8, 24, 25, 30}, actions=['click']),
        }
        self.pids = {':shell': 42, ':reference': 77, ':other': 99}
        self.roles = {SHELL: 75, DOCK: 32, SETTINGS: 69, REFERENCE_ROOT: 75,
                      FRAME: 23, (':other', '/root'): 75, FOREIGN: 32}
        self.actions, self.observations = [], []
        self.root_reads, self.roots_at_press = 0, 0
        self.pressed = False
        self.change_after = 0
        self.reference_reads = 0
        self.allow_readback = True
        self.keep_shell_active = False
        self.action_result = True

    def roots(self):
        self.root_reads += 1
        return list(self.roots_now)

    def owner(self, bus):
        return self.pids.get(bus)

    def interface(self, at, interface):
        if interface == ACCESSIBLE:
            return Accessible(self, at)
        if interface != PROPERTIES:
            raise AssertionError('the toolkit is an Application property')
        return Properties(self, at)

    def facts(self, at):
        self.observations.append((at, self.pressed))
        if len(self.observations) > 512:
            raise AssertionError('foreground readback did not honor a deadline')
        if at == FRAME and self.pressed:
            self.reference_reads += 1
            if self.allow_readback and self.reference_reads > self.change_after:
                self.nodes[FRAME]['states'] = set(ACTIVE)
                if not self.keep_shell_active:
                    self.nodes[SETTINGS]['states'] = set(VISIBLE)
        return {key: copy.deepcopy(value) for key, value in self.nodes[at].items()
                if key != 'toolkit'}

    def action(self, at, index):
        self.actions.append((at, index))
        if at == DOCK and self.action_result:
            self.pressed = True
            self.roots_at_press = self.root_reads
        return self.action_result


def clock(tree):
    """Enough time to read once, followed by definite expiry without sleeping."""
    return SimpleNamespace(monotonic=lambda: 600 if tree.pressed and
                           tree.root_reads > tree.roots_at_press + 2 else 0,
                           sleep=lambda _: None)


class ReferenceForeground(unittest.TestCase):
    def invoke(self, tree, *, expired=False):
        calls = []
        def press(process, matches):
            calls.append(process)
            self.assertTrue(matches(REFERENCE))
            self.assertFalse(matches(REFERENCE + '.Foreign'))
            self.assertFalse(matches('org.aloworld.Studio'))
            activate(tree, process, matches)
        if expired:
            with patch('reference_window.time', clock(tree)):
                result = foreground(tree, press)
        else:
            result = foreground(tree, press)
        return result, calls

    def test_empty_named_shell_is_identified_by_toolkit_and_only_its_dock_is_pressed(self):
        tree = NativeTree()
        result, calls = self.invoke(tree)
        self.assertEqual(calls, [42])
        self.assertEqual(tree.actions, [(DOCK, 0)])
        self.assertEqual(result, dict(name=TITLE, process=77))
        self.assertTrue(any(at == FRAME and fresh for at, fresh in tree.observations))
        self.assertNotIn((FOREIGN, 0), tree.actions)

    def test_readback_waits_for_actual_reference_activation_without_pressing_again(self):
        tree = NativeTree()
        tree.change_after = 2
        result, calls = self.invoke(tree)
        self.assertGreaterEqual(tree.reference_reads, 3)
        self.assertEqual(result, dict(name=TITLE, process=77))
        self.assertEqual(calls, [42])
        self.assertEqual(tree.actions, [(DOCK, 0)])

    def test_missing_or_duplicate_shell_toolkit_is_refused_without_any_action(self):
        for duplicate in (False, True):
            with self.subTest(duplicate=duplicate):
                tree = NativeTree()
                if duplicate:
                    extra = (':extra-shell', '/root')
                    tree.roots_now.append(extra)
                    tree.nodes[extra] = node('alo', toolkit='alo')
                    tree.pids[':extra-shell'] = 43
                else:
                    tree.nodes[SHELL]['name'] = 'alo'
                    tree.nodes[SHELL]['toolkit'] = 'gtk'
                with self.assertRaises(ValueError):
                    self.invoke(tree, expired=True)
                self.assertEqual(tree.actions, [])

    def test_missing_duplicate_hidden_or_disabled_owned_dock_entry_is_refused(self):
        for failure in ('missing', 'duplicate', 'hidden', 'disabled', 'similar'):
            with self.subTest(failure=failure):
                tree = NativeTree()
                if failure == 'duplicate':
                    extra = (':shell', '/dock/another-reference')
                    tree.nodes[extra] = copy.deepcopy(tree.nodes[DOCK])
                    tree.nodes[SHELL]['children'].append(extra)
                elif failure in ('missing', 'similar'):
                    tree.nodes[DOCK]['name'] = REFERENCE + '.Foreign'
                elif failure == 'hidden':
                    tree.nodes[DOCK]['states'].discard(25)
                else:
                    tree.nodes[DOCK]['states'].discard(8)
                with self.assertRaises(ValueError):
                    self.invoke(tree, expired=True)
                self.assertEqual(tree.actions, [])

    def test_failed_native_activation_is_not_retried_or_reported_as_foreground(self):
        tree = NativeTree()
        tree.action_result = False
        with self.assertRaises(ValueError):
            self.invoke(tree, expired=True)
        self.assertEqual(tree.actions, [(DOCK, 0)])

    def test_inactive_hidden_invisible_or_wrong_title_reference_never_passes(self):
        for failure in ('inactive', 'hidden', 'invisible', 'wrong-frame-title', 'missing'):
            with self.subTest(failure=failure):
                tree = NativeTree()
                tree.allow_readback = False
                tree.nodes[SETTINGS]['states'] = set(VISIBLE)
                tree.nodes[FRAME]['states'] = set(ACTIVE)
                if failure in ('inactive', 'hidden', 'invisible'):
                    tree.nodes[FRAME]['states'].discard({'inactive': 1, 'hidden': 25, 'invisible': 30}[failure])
                elif failure == 'wrong-frame-title':
                    tree.nodes[FRAME]['name'] = 'Other window'
                else:
                    tree.roots_now.remove(REFERENCE_ROOT)
                with self.assertRaises(ValueError):
                    self.invoke(tree, expired=True)
                self.assertEqual(tree.actions, [(DOCK, 0)])

    def test_two_reference_applications_or_frames_are_ambiguous(self):
        for duplicate in ('application', 'frame'):
            with self.subTest(duplicate=duplicate):
                tree = NativeTree()
                extra = (':second-reference' if duplicate == 'application' else ':reference',
                         REFERENCE_PATH + 'another-window')
                tree.nodes[extra] = node(TITLE, states=ACTIVE)
                tree.roles[extra] = 23
                if duplicate == 'application':
                    extra_root = (':second-reference', '/root')
                    tree.pids[':second-reference'] = 88
                    tree.nodes[extra_root] = node('other-program-name', children=[extra], toolkit='GTK')
                    tree.roles[extra_root] = 75
                    tree.roots_now.append(extra_root)
                else:
                    tree.nodes[REFERENCE_ROOT]['children'].append(extra)
                with self.assertRaises(ValueError):
                    self.invoke(tree, expired=True)
                self.assertLessEqual(len(tree.actions), 1)

    def test_a_same_title_frame_under_another_application_path_is_not_the_reference(self):
        tree = NativeTree()
        tree.allow_readback = False
        tree.nodes[REFERENCE_ROOT]['name'] = TITLE
        other = (':reference', '/org/aloworld/Other/a11y/window-1')
        tree.nodes[other] = node(TITLE, states=ACTIVE)
        tree.roles[other] = 23
        tree.nodes[REFERENCE_ROOT]['children'] = [other]
        tree.nodes[SETTINGS]['states'] = set(VISIBLE)
        with self.assertRaises(ValueError):
            self.invoke(tree, expired=True)
        self.assertEqual(tree.actions, [(DOCK, 0)])

    def test_an_active_same_title_label_is_not_a_native_frame(self):
        tree = NativeTree()
        tree.nodes[REFERENCE_ROOT]['name'] = TITLE
        tree.roles[FRAME] = 29
        with self.assertRaises(ValueError):
            self.invoke(tree, expired=True)
        self.assertEqual(tree.actions, [(DOCK, 0)])

    def test_active_showing_shell_surface_still_refuses_even_when_gtk_says_active(self):
        tree = NativeTree()
        tree.keep_shell_active = True
        with self.assertRaises(ValueError):
            self.invoke(tree, expired=True)
        self.assertEqual(tree.actions, [(DOCK, 0)])

    def test_missing_readback_refuses_with_a_bounded_clock_after_one_press(self):
        tree = NativeTree()
        tree.allow_readback = False
        with self.assertRaises(ValueError):
            self.invoke(tree, expired=True)
        self.assertEqual(tree.actions, [(DOCK, 0)])
        self.assertLessEqual(tree.reference_reads, 4)

    def test_session_reports_observed_activation_only_after_real_helper_success(self):
        for succeed in (True, False):
            with self.subTest(succeed=succeed):
                tree = NativeTree()
                tree.keep_shell_active = not succeed
                flow = object.__new__(Session)
                flow.tree = tree
                flow.press = lambda process, matches: activate(tree, process, matches)
                events = []
                flow.event = lambda kind, **fields: events.append((kind, fields))
                with patch('reference_window.time', clock(tree)):
                    if succeed:
                        flow.foreground_reference()
                        self.assertEqual(events, [('reference-window-activated', dict(name=TITLE, process=77))])
                    else:
                        with self.assertRaises(ValueError):
                            flow.foreground_reference()
                        self.assertEqual(events, [])


class ForegroundFlow(RecordingFlow):
    """Caller-only fixture: no reference, recorder or native OS is launched."""
    def __init__(self, fail_on=None):
        super().__init__()
        self.fail_on, self.foregrounds = fail_on, 0

    def foreground_reference(self):
        self.foregrounds += 1
        self.calls.append('foreground-reference')
        if self.foregrounds == self.fail_on:
            raise ValueError('reference activation was not observed')


class RecordingForegroundOrder(unittest.TestCase):
    def test_each_actual_or_countdown_start_follows_verified_reference_activation(self):
        for source, microphone, starts in (('screen', False, 1), ('window', True, 2)):
            with self.subTest(source=source), patch('installed_session.time.sleep'):
                flow = ForegroundFlow()
                flow.recording(source, microphone)
                positions = [index for index, value in enumerate(flow.calls) if value == 'Start']
                self.assertEqual(len(positions), starts)
                self.assertEqual(flow.foregrounds, starts)
                for position in positions:
                    self.assertGreater(position, flow.calls.index(('picker', source)))
                    self.assertEqual(flow.calls[position - 1], 'foreground-reference')

    def test_failed_initial_activation_prevents_start_and_saved_output_for_both_sources(self):
        for source, microphone in (('screen', False), ('window', True)):
            with self.subTest(source=source), patch('installed_session.time.sleep'):
                flow = ForegroundFlow(fail_on=1)
                with self.assertRaisesRegex(ValueError, 'activation was not observed'):
                    flow.recording(source, microphone)
                self.assertNotIn('Start', flow.calls)
                self.assertNotIn('Finish', flow.calls)
                self.assertNotIn('export', flow.calls)

    def test_failed_activation_after_countdown_cancel_prevents_the_real_recording_start(self):
        flow = ForegroundFlow(fail_on=2)
        with patch('installed_session.time.sleep'):
            with self.assertRaisesRegex(ValueError, 'activation was not observed'):
                flow.recording('window', True)
        self.assertEqual(flow.calls.count('Start'), 1)
        self.assertIn('CancelCountdown', flow.calls)
        self.assertNotIn('Finish', flow.calls)
        self.assertNotIn('export', flow.calls)


class StudioTree(NativeTree):
    """Same owner/path contract for normal Studio, not the reference title."""
    def __init__(self):
        super().__init__()
        self.studio_frame = (FRAME[0], '/org/aloworld/Studio/a11y/normal-window')
        self.use_reference_path = False
        self.use_reference_title = False

    def facts(self, at):
        original = FRAME if at == self.studio_frame else at
        found = super().facts(original)
        if at == DOCK:
            found['name'] = 'org.aloworld.Studio'
        elif at == REFERENCE_ROOT:
            found['name'] = 'unrelated-program-name'
            found['children'] = [FRAME if self.use_reference_path else self.studio_frame]
        elif original == FRAME:
            found['name'] = TITLE if self.use_reference_title else 'alo Studio'
        return found

    def interface(self, at, interface):
        return super().interface(FRAME if at == self.studio_frame else at, interface)


class NormalStudioForeground(unittest.TestCase):
    def invoke(self, tree):
        calls = []
        def press(process, matches):
            calls.append(process)
            self.assertTrue(matches('org.aloworld.Studio'))
            self.assertFalse(matches(REFERENCE))
            self.assertFalse(matches('org.aloworld.Studio.Foreign'))
            activate(tree, process, matches)
        with patch('reference_window.time', clock(tree)):
            result = foreground(tree, press, application='org.aloworld.Studio', title='alo Studio')
        return result, calls

    def test_normal_app_uses_its_own_exact_dock_identifier_registered_path_and_title(self):
        tree = StudioTree()
        result, calls = self.invoke(tree)
        self.assertEqual(calls, [42])
        self.assertEqual(tree.actions, [(DOCK, 0)])
        self.assertEqual(result, dict(name='alo Studio', process=77))

    def test_reference_registered_path_cannot_substitute_for_normal_app_with_same_title(self):
        tree = StudioTree()
        tree.use_reference_path = True
        with self.assertRaises(ValueError):
            self.invoke(tree)

    def test_reference_title_cannot_substitute_for_normal_app_even_at_its_path(self):
        tree = StudioTree()
        tree.use_reference_title = True
        with self.assertRaises(ValueError):
            self.invoke(tree)


if __name__ == '__main__':
    unittest.main()
