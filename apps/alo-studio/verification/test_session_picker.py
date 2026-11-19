"""Actual private Session.picker through native GTK choices and live AT-SPI.

The labels come from the portal's published repository vocabulary, not the
controller's expected English prefix. The recorder's call/status boundary is
simulated; selection, process ownership, visibility and Share/Cancel are real
native UI effects. This is not installed capture or media acceptance evidence.

Run on an owned isolated session/display with STUDIO_PRIVATE_UI_SESSION=1, as
the existing verification-instruments workflow runs its live GTK tests.
"""

import contextlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import time
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import dbus

from atspi_tree import AtspiTree
import installed_session
from installed_session import PORTAL, Session


ROOT = Path(installed_session.__file__).resolve().parents[3]
FIXTURE = Path(__file__).with_name('fixtures') / 'native_source_picker.py'
ELIGIBLE = {8, 24, 25}


def published_kind(constant):
    """Read the portal's own current English, so stale client wording is caught."""
    text = (ROOT / 'crates/alo-portals/src/words.rs').read_text()
    matches = re.findall(r'pub const ' + re.escape(constant)
                         + r': Word = Word::saying\(\s*"[^"]+",\s*"([^"]+)"', text)
    if len(matches) != 1:
        raise AssertionError('expected one published portal kind ' + constant)
    return matches[0]


def reference_title():
    """The independently owned reference's actual native window title."""
    text = (ROOT / 'apps/alo-studio/examples/studio-reference-window.rs').read_text()
    matches = re.findall(r'\.title\("([^"]+)"\)', text)
    if len(matches) != 1:
        raise AssertionError('expected one owned reference window title')
    return matches[0]


def choice(identifier, title, **flags):
    """The source label capture_choice publishes for a window's own title."""
    return dict(id=identifier, label=published_kind('SHARE_WINDOW') + ' · ' + title, **flags)


@contextlib.contextmanager
def native_dialog(folder, options, bus_name=PORTAL):
    """Own one real native dialog and reap it on every success/refusal path."""
    folder.mkdir()
    config = folder / 'config.json'
    config.write_text(json.dumps(dict(folder=str(folder), options=options, bus_name=bus_name)))
    environment = dict(os.environ, GTK_MODULES='atk-bridge', NO_AT_BRIDGE='0', GDK_BACKEND='x11')
    with (folder / 'stderr.log').open('w') as errors:
        process = subprocess.Popen(['/usr/bin/python3', '-I', '-B', str(FIXTURE), str(config)],
                                   env=environment, stdout=subprocess.DEVNULL, stderr=errors)
        try:
            yield process
        finally:
            if process.poll() is None:
                process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)


class NativePickerSession(Session):
    """Fake recorder state only; actual picker/press/process methods are retained."""
    def __init__(self, tree, folder):
        self.tree = tree
        self.bus = dbus.SessionBus()
        self.folder = folder
        self.calls = []
        self.events = []

    def call(self, method, *arguments):
        self.calls.append((method, arguments))

    def event(self, kind, **fields):
        self.events.append(dict(kind=kind, **fields))

    def wait(self, wanted, **_):
        response = self.folder / 'response.json'
        if not response.exists():
            state = dict(phase='choosing', preview_ready=False, microphone_active=False)
            if wanted(state):
                return state
        deadline = time.monotonic() + 5
        while not response.exists() and time.monotonic() < deadline:
            time.sleep(.02)
        if not response.exists():
            raise AssertionError('native Share/Cancel did not produce an actual response')
        result = json.loads(response.read_text())
        state = dict(phase='selected' if result['code'] == 0 else 'ready',
                     preview_ready=result['code'] == 0, microphone_active=False)
        if not wanted(state):
            raise AssertionError('the actual native response did not satisfy the recorder wait')
        return state


@unittest.skipUnless(os.environ.get('STUDIO_PRIVATE_UI_SESSION') == '1',
                     'requires an owned isolated DBus session and X display')
class NativeSessionPicker(unittest.TestCase):
    def facts_of(self, tree, process):
        """Read all actual nodes of this process without admitting foreign roots."""
        pending, seen, facts = list(tree.roots()), set(), []
        while pending:
            node = pending.pop()
            if node in seen or tree.owner(node[0]) != process.pid:
                continue
            seen.add(node)
            self.assertLessEqual(len(seen), 256)
            observed = tree.facts(node)
            facts.append(observed)
            pending.extend(observed['children'])
        return facts

    def ready(self, tree, process, options):
        """Require real Share/Cancel and every expected radio before the test starts."""
        deadline = time.monotonic() + 15
        last = []
        while time.monotonic() < deadline and process.poll() is None:
            try:
                last = self.facts_of(tree, process)
                if (any(item['name'] == 'Share' and ELIGIBLE <= item['states'] for item in last)
                        and any(item['name'] == 'Cancel' and ELIGIBLE <= item['states'] for item in last)
                        and all(any(item['name'] == option['label'] for item in last) for option in options)):
                    return last
            except (ValueError, dbus.DBusException):
                pass
            time.sleep(.05)
        self.fail(f'native picker did not become readable: exit={process.poll()}, facts={last}')

    def attempt(self, flow, source='window', *, cancel=False):
        """Only the picker's retry clock is accelerated; the native tree is ready."""
        clock = iter((0, 11))
        with patch('installed_session.time', SimpleNamespace(monotonic=lambda: next(clock),
                                                            sleep=lambda _: None)):
            flow.picker(source, cancel=cancel)

    def effects(self, folder):
        path = folder / 'choices.jsonl'
        return [json.loads(line) for line in path.read_text().splitlines()] if path.exists() else []

    def test_the_actual_picker_selects_the_owned_reference_then_shares_once(self):
        options = [choice('other', 'Unrelated application'), choice('owned', reference_title())]
        tree = AtspiTree()
        with tempfile.TemporaryDirectory(prefix='studio-picker-owned-') as held:
            folder = Path(held) / 'native'
            with native_dialog(folder, options) as process:
                self.ready(tree, process, options)
                flow = NativePickerSession(tree, folder)
                self.attempt(flow)
                self.assertEqual(json.loads((folder / 'response.json').read_text()),
                                 dict(code=0, selected='owned'))
                self.assertEqual(self.effects(folder), ['owned'])
                self.assertEqual(flow.calls, [('ChooseWindow', ())])
                self.assertEqual([item['process'] for item in flow.events if item['kind'] == 'native-control'],
                                 [process.pid, process.pid])

    def test_a_foreign_dialogs_same_title_is_never_selected_or_shared(self):
        options = [choice('other', 'Unrelated application')]
        foreign_options = [choice('foreign-default', 'Another window'), choice('foreign', reference_title())]
        tree = AtspiTree()
        with tempfile.TemporaryDirectory(prefix='studio-picker-foreign-') as held:
            folder, foreign = Path(held) / 'native', Path(held) / 'foreign'
            with native_dialog(folder, options) as process:
                with native_dialog(foreign, foreign_options,
                                   'org.aloworld.Studio.PrivatePickerForeign') as other:
                    self.ready(tree, process, options)
                    self.ready(tree, other, foreign_options)
                    with self.assertRaisesRegex(ValueError, 'native source picker could not be operated'):
                        self.attempt(NativePickerSession(tree, folder))
                    for untouched in (folder, foreign):
                        self.assertFalse((untouched / 'response.json').exists())
                        self.assertEqual(self.effects(untouched), [])

    def test_two_eligible_owned_choices_with_the_reference_title_are_refused(self):
        options = [choice('other', 'Unrelated application'), choice('one', reference_title()),
                   choice('two', reference_title())]
        tree = AtspiTree()
        with tempfile.TemporaryDirectory(prefix='studio-picker-ambiguous-') as held:
            folder = Path(held) / 'native'
            with native_dialog(folder, options) as process:
                self.ready(tree, process, options)
                with self.assertRaisesRegex(ValueError, 'native source picker could not be operated'):
                    self.attempt(NativePickerSession(tree, folder))
                self.assertFalse((folder / 'response.json').exists())
                self.assertEqual(self.effects(folder), [])

    def test_hidden_or_disabled_reference_choices_cannot_be_selected_or_shared(self):
        for flags in (dict(visible=False), dict(enabled=False)):
            with self.subTest(flags=flags), tempfile.TemporaryDirectory(prefix='studio-picker-ineligible-') as held:
                options = [choice('other', 'Unrelated application'), choice('ineligible', reference_title(), **flags)]
                tree = AtspiTree()
                folder = Path(held) / 'native'
                with native_dialog(folder, options) as process:
                    facts = self.ready(tree, process, options)
                    actual = [item for item in facts if item['name'] == options[1]['label']]
                    self.assertTrue(actual)
                    self.assertTrue(all(not ELIGIBLE <= item['states'] for item in actual))
                    with self.assertRaisesRegex(ValueError, 'native source picker could not be operated'):
                        self.attempt(NativePickerSession(tree, folder))
                    self.assertFalse((folder / 'response.json').exists())
                    self.assertEqual(self.effects(folder), [])

    def test_cancelling_either_picker_shares_nothing_and_never_selects_a_source(self):
        for source in ('screen', 'window'):
            with self.subTest(source=source), tempfile.TemporaryDirectory(prefix='studio-picker-cancel-') as held:
                options = [choice('other', 'Unrelated application')]
                tree = AtspiTree()
                folder = Path(held) / 'native'
                with native_dialog(folder, options) as process:
                    self.ready(tree, process, options)
                    flow = NativePickerSession(tree, folder)
                    self.attempt(flow, source, cancel=True)
                    self.assertEqual(json.loads((folder / 'response.json').read_text()), dict(code=1, selected=None))
                    self.assertEqual(self.effects(folder), [])
                    self.assertEqual(flow.calls, [('ChooseScreen' if source == 'screen' else 'ChooseWindow', ())])
                    self.assertEqual([item['process'] for item in flow.events if item['kind'] == 'native-control'],
                                     [process.pid])


if __name__ == '__main__':
    unittest.main()
