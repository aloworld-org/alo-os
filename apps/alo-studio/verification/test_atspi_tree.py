"""Live native accessibility transport on an isolated session and display."""

import contextlib
import os
import pathlib
import subprocess
import tempfile
import time
import unittest

from atspi_tree import AtspiTree
from native_controls import activate

FIXTURE = r'''
import gi, pathlib, sys
gi.require_version('Gtk', '3.0')
from gi.repository import Gtk
window = Gtk.Window(title='Studio walk native control fixture')
button = Gtk.Button(label='Choose reference')
def chosen(_):
    pathlib.Path(sys.argv[1]).write_text('activated once')
    Gtk.main_quit()
button.connect('clicked', chosen)
window.add(button)
window.show_all()
Gtk.main()
'''


CONTROLS_FIXTURE = r'''
import gi, pathlib, sys
gi.require_version('Gtk', '3.0')
from gi.repository import Gtk
window = Gtk.Window(title='Studio private native-control cases')
box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
window.add(box)
mode = sys.argv[2]
rows = [('chosen', True, True)]
if mode == 'ambiguous':
    rows.append(('second', True, True))
elif mode == 'ineligible':
    rows.extend([('hidden', False, True), ('disabled', True, False)])
hidden = []
def clicked(_, identifier):
    with pathlib.Path(sys.argv[1]).open('a') as output:
        output.write(identifier + '\n')
for identifier, visible, sensitive in rows:
    button = Gtk.Button(label='Choose reference')
    button.set_sensitive(sensitive)
    button.connect('clicked', clicked, identifier)
    box.add(button)
    if not visible:
        hidden.append(button)
window.show_all()
for button in hidden:
    button.hide()
Gtk.main()
'''


@contextlib.contextmanager
def native_fixture(result, mode):
    """Keep the application alive so every actual activation leaves a line."""
    environment = dict(os.environ, GTK_MODULES='atk-bridge', NO_AT_BRIDGE='0',
                       GDK_BACKEND='x11')
    process = subprocess.Popen(
        ['python3', '-I', '-c', CONTROLS_FIXTURE, str(result), mode],
        env=environment, stdout=subprocess.DEVNULL)
    try:
        yield process
    finally:
        if process.poll() is None:
            process.terminate()
        process.wait(timeout=5)


@unittest.skipUnless(os.environ.get('STUDIO_PRIVATE_UI_SESSION') == '1',
                     'requires an owned isolated DBus session and X display')
class LiveNativeTree(unittest.TestCase):
    def wait_for_named_controls(self, tree, process, count):
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline and process.poll() is None:
            pending, seen, matches = list(tree.roots()), set(), []
            while pending and len(seen) < 128:
                node = pending.pop()
                if node in seen or tree.owner(node[0]) != process.pid:
                    continue
                seen.add(node)
                facts = tree.facts(node)
                if facts['name'] == 'Choose reference':
                    matches.append(facts)
                pending.extend(facts['children'])
            if len(matches) == count:
                return matches
            time.sleep(.1)
        self.fail(f'Expected {count} native controls from process {process.pid}')

    def assert_one_native_effect(self, result):
        deadline = time.monotonic() + 5
        while not result.exists() and time.monotonic() < deadline:
            time.sleep(.05)
        time.sleep(.2)
        self.assertEqual(result.read_text(), 'chosen\n')

    def test_a_foreign_process_with_the_same_label_is_left_untouched(self):
        tree = AtspiTree()
        with tempfile.TemporaryDirectory(prefix='studio-atspi-process-') as folder:
            selected = pathlib.Path(folder) / 'selected'
            foreign = pathlib.Path(folder) / 'foreign'
            with native_fixture(selected, 'unique') as target:
                with native_fixture(foreign, 'unique') as other:
                    self.wait_for_named_controls(tree, target, 1)
                    self.wait_for_named_controls(tree, other, 1)
                    activate(tree, target.pid, lambda name: name == 'Choose reference')
                    self.assert_one_native_effect(selected)
                    self.assertFalse(foreign.exists())

    def test_hidden_and_disabled_native_duplicates_do_not_receive_the_action(self):
        tree = AtspiTree()
        with tempfile.TemporaryDirectory(prefix='studio-atspi-states-') as folder:
            result = pathlib.Path(folder) / 'chosen'
            with native_fixture(result, 'ineligible') as process:
                self.wait_for_named_controls(tree, process, 3)
                activate(tree, process.pid, lambda name: name == 'Choose reference')
                self.assert_one_native_effect(result)

    def test_two_eligible_native_controls_are_refused_without_any_effect(self):
        tree = AtspiTree()
        with tempfile.TemporaryDirectory(prefix='studio-atspi-ambiguous-') as folder:
            result = pathlib.Path(folder) / 'chosen'
            with native_fixture(result, 'ambiguous') as process:
                self.wait_for_named_controls(tree, process, 2)
                with self.assertRaises(ValueError):
                    activate(tree, process.pid, lambda name: name == 'Choose reference')
                time.sleep(.2)
                self.assertFalse(result.exists())

    def test_an_exited_native_process_cannot_be_activated_from_stale_discovery(self):
        tree = AtspiTree()
        with tempfile.TemporaryDirectory(prefix='studio-atspi-exited-') as folder:
            result = pathlib.Path(folder) / 'chosen'
            with native_fixture(result, 'unique') as process:
                self.wait_for_named_controls(tree, process, 1)
                process.terminate()
                process.wait(timeout=5)
                with self.assertRaises(ValueError):
                    activate(tree, process.pid, lambda name: name == 'Choose reference')
                self.assertFalse(result.exists())

    def test_the_actual_native_button_is_read_and_activated_by_process(self):
        tree = AtspiTree()
        with tempfile.TemporaryDirectory(prefix='studio-atspi-test-') as folder:
            result = pathlib.Path(folder) / 'chosen'
            environment = dict(os.environ, GTK_MODULES='atk-bridge', NO_AT_BRIDGE='0',
                               GDK_BACKEND='x11')
            process = subprocess.Popen(['python3', '-I', '-c', FIXTURE, str(result)],
                                       env=environment, stdout=subprocess.DEVNULL)
            try:
                deadline = time.monotonic() + 15
                while True:
                    try:
                        activate(tree, process.pid, lambda name: name == 'Choose reference')
                        break
                    except ValueError:
                        if process.poll() is not None or time.monotonic() >= deadline:
                            pending = list(tree.roots())
                            seen = set()
                            while pending and len(seen) < 32:
                                node = pending.pop()
                                if node in seen or tree.owner(node[0]) != process.pid:
                                    continue
                                seen.add(node)
                                facts = tree.facts(node)
                                print('PRIVATE_FIXTURE_NODE', node, facts)
                                pending.extend(facts['children'])
                            raise
                        time.sleep(.1)
                process.wait(timeout=5)
                self.assertEqual(process.returncode, 0)
                self.assertEqual(result.read_text(), 'activated once')
            finally:
                if process.poll() is None:
                    process.terminate()
                    process.wait(timeout=5)


if __name__ == '__main__':
    unittest.main()
