"""Actual GTK4 AT-SPI compatibility for the private installed-walk selector.

GTK4's state builder exposes SENSITIVE for a usable control but does not set
ENABLED. Test actual named actions and effects rather than mirroring guessed
state dictionaries. Every app belongs to this test's isolated session/display;
these are test-only GTK widgets, not installed consent or recording evidence.
"""

import json
import os
from pathlib import Path
import select
import subprocess
import tempfile
import time
import unittest
from unittest.mock import patch

import dbus

from atspi_tree import AtspiTree
from native_controls import activate


FIXTURE = Path(__file__).with_name('fixtures') / 'gtk4_controls.py'


@unittest.skipUnless(os.environ.get('STUDIO_PRIVATE_UI_SESSION') == '1',
                     'requires an owned isolated DBus session and X display')
class ActualGtk4Controls(unittest.TestCase):
    def setUp(self):
        self.held = tempfile.TemporaryDirectory(prefix='studio-gtk4-actions-')
        self.addCleanup(self.held.cleanup)
        self.folder = Path(self.held.name)
        runtime = self.folder / 'runtime'
        runtime.mkdir(mode=0o700)
        self.environment = dict(os.environ, XDG_RUNTIME_DIR=str(runtime),
                                XDG_CONFIG_HOME=str(self.folder / 'config'),
                                XDG_CACHE_HOME=str(self.folder / 'cache'),
                                GSETTINGS_BACKEND='memory')
        # GTK starts portal services during normal initialization. Give those
        # dependencies an owned session instead of contaminating other live
        # fixtures which deliberately own the native portal's well-known name.
        errors = (self.folder / 'bus-stderr.log').open('w')
        self.addCleanup(errors.close)
        daemon = subprocess.Popen(['dbus-daemon', '--session', '--nofork', '--print-address=1'],
                                  env=self.environment, stdout=subprocess.PIPE,
                                  stderr=errors, text=True)

        def stop_bus():
            if daemon.poll() is None:
                daemon.terminate()
            try:
                daemon.communicate(timeout=5)
            except subprocess.TimeoutExpired:
                daemon.kill()
                daemon.communicate(timeout=5)

        self.addCleanup(stop_bus)
        if not select.select([daemon.stdout], [], [], 5)[0]:
            self.fail('the owned GTK4 session bus did not announce readiness')
        address = daemon.stdout.readline(4097).strip()
        self.assertTrue(address.startswith('unix:') and len(address) <= 4096,
                        'the owned GTK4 session bus did not announce a bounded Unix address')
        self.session = dbus.bus.BusConnection(address)
        self.session.set_exit_on_disconnect(False)
        self.addCleanup(self.session.close)
        with patch('atspi_tree.dbus.SessionBus', return_value=self.session):
            self.tree = AtspiTree()
        self.tree.bus.set_exit_on_disconnect(False)
        self.addCleanup(self.tree.bus.close)
        accessibility = dbus.Interface(self.session.get_object('org.a11y.Bus', '/org/a11y/bus'),
                                       'org.a11y.Bus').GetAddress(timeout=3)
        self.environment.update(DBUS_SESSION_BUS_ADDRESS=address, AT_SPI_BUS_ADDRESS=str(accessibility))
        # Start the real registry before GTK embeds its first accessible root.
        self.tree.roots()

    def start(self, buttons, suffix='Owned'):
        folder = self.folder / suffix
        folder.mkdir()
        configuration = folder / 'configuration.json'
        configuration.write_text(json.dumps(dict(
            folder=str(folder), application='org.aloworld.Studio.PrivateControls' + suffix,
            buttons=buttons)))
        errors = (folder / 'stderr.log').open('w')
        self.addCleanup(errors.close)
        environment = dict(self.environment, GTK_A11Y='atspi', NO_AT_BRIDGE='0',
                           GDK_BACKEND='x11', GSK_RENDERER='cairo')
        child = subprocess.Popen(['/usr/bin/python3', '-I', '-B', str(FIXTURE), str(configuration)],
                                 env=environment, stdout=subprocess.DEVNULL, stderr=errors)

        def cleanup():
            if child.poll() is None:
                child.terminate()
            try:
                child.wait(timeout=5)
            except subprocess.TimeoutExpired:
                child.kill()
                child.wait(timeout=5)

        self.addCleanup(cleanup)
        deadline = time.monotonic() + 15
        last = []
        last_error = None
        focused = False
        while time.monotonic() < deadline and child.poll() is None:
            try:
                if not focused and (folder / 'ready').is_file():
                    windows = subprocess.run(
                        ['xdotool', 'search', '--onlyvisible', '--pid', str(child.pid)],
                        capture_output=True, text=True, timeout=3)
                    if windows.returncode == 0 and windows.stdout.splitlines():
                        subprocess.run(['xdotool', 'windowfocus', '--sync', windows.stdout.splitlines()[0]],
                                       check=True, timeout=3)
                        focused = True
                last = self.observed(child.pid)
                names = {facts['name'] for facts in last}
                expected = {item['name'] for item in buttons if item.get('visible', True)}
                if (folder / 'ready').is_file() and expected <= names and last:
                    return child, folder, last
            except (ValueError, dbus.DBusException) as error:
                last_error = repr(error) + ' caused by ' + repr(error.__cause__)
            time.sleep(.05)
        self.fail(f'actual GTK4 controls not readable: exit={child.poll()}, focused={focused}, '
                  f'ready={(folder / "ready").exists()}, observed={last}, '
                  f'pid={child.pid}, roots={[(node, self.tree.owner(node[0])) for node in self.tree.roots()]}, '
                  f'last_error={last_error}, '
                  f'stderr={(folder / "stderr.log").read_text()[-2000:]}')

    def observed(self, process):
        pending, seen, result = list(self.tree.roots()), set(), []
        while pending:
            node = pending.pop()
            if node in seen or self.tree.owner(node[0]) != process:
                continue
            seen.add(node)
            self.assertLessEqual(len(seen), 128)
            # A disabled GTK4 button's child label advertises actions but
            # refuses GetName. Read state premises without those action groups.
            words = self.tree.interface(node, 'org.a11y.atspi.Accessible').GetState(timeout=3)
            states = {index * 32 + bit for index, word in enumerate(words)
                      for bit in range(32) if int(word) & (1 << bit)}
            role = int(self.tree.interface(node, 'org.a11y.atspi.Accessible').GetRole(timeout=3))
            name = self.tree.interface(node, 'org.freedesktop.DBus.Properties').Get(
                'org.a11y.atspi.Accessible', 'Name', timeout=3)
            facts = dict(name=str(name), states=states, actions=[], children=self.tree.children(node), role=role)
            # Only inspect actual usable buttons' actions for fixture readiness;
            # labels inherit action groups but are never our clickable buttons.
            if role == 43 and 24 in states:
                facts['actions'] = self.tree.facts(node)['actions']
            facts['node'] = node
            result.append(facts)
            pending.extend(facts['children'])
        return result

    def effects(self, folder):
        path = folder / 'actions.jsonl'
        return [json.loads(line) for line in path.read_text().splitlines()] if path.exists() else []

    def test_sensitive_showing_gtk4_warning_is_activated_once_without_enabled_bit(self):
        child, folder, found = self.start([dict(id='accept-warning', name='Record anyway')])
        buttons = [item for item in found if item['name'] == 'Record anyway' and 'click' in item['actions']]
        self.assertEqual(len(buttons), 1)
        self.assertLessEqual({24, 25, 30}, buttons[0]['states'])
        self.assertNotIn(8, buttons[0]['states'], 'actual GTK4 premise, not a simulated missing flag')
        root = next(item['node'] for item in found if item['role'] == 75)
        properties = self.tree.interface(root, 'org.freedesktop.DBus.Properties')
        self.assertEqual(properties.Get('org.a11y.atspi.Application', 'ToolkitName', timeout=3), 'GTK')
        version = properties.Get('org.a11y.atspi.Application', 'Version', timeout=3)
        self.assertIsInstance(version, dbus.String)
        self.assertRegex(version, r'^4\.\d+\.\d+$')
        activate(self.tree, child.pid, lambda name: name == 'Record anyway')
        until = time.monotonic() + 3
        while not self.effects(folder) and time.monotonic() < until:
            time.sleep(.02)
        self.assertEqual(self.effects(folder), ['accept-warning'])

    def test_disabled_or_hidden_actual_gtk4_warning_never_activates(self):
        for kind in ('disabled', 'hidden'):
            with self.subTest(kind=kind):
                child, folder, found = self.start([
                    dict(id='unusable', name='Record anyway', enabled=kind != 'disabled', visible=kind != 'hidden'),
                    dict(id='other', name='Other action')], suffix=kind.title())
                if kind == 'disabled':
                    matched = [item for item in found if item['name'] == 'Record anyway' and item['role'] == 43]
                    self.assertEqual(len(matched), 1)
                    self.assertNotIn(24, matched[0]['states'])
                with self.assertRaises(ValueError):
                    activate(self.tree, child.pid, lambda name: name == 'Record anyway')
                self.assertEqual(self.effects(folder), [])

    def test_a_disabled_gtk4_sibling_does_not_block_the_one_usable_warning(self):
        child, folder, found = self.start([
            dict(id='unused', name='Unavailable other action', enabled=False),
            dict(id='accept-warning', name='Record anyway')])
        disabled = [item for item in found if item['name'] == 'Unavailable other action' and item['role'] == 43]
        self.assertEqual(len(disabled), 1)
        self.assertNotIn(24, disabled[0]['states'])
        activate(self.tree, child.pid, lambda name: name == 'Record anyway')
        until = time.monotonic() + 3
        while not self.effects(folder) and time.monotonic() < until:
            time.sleep(.02)
        self.assertEqual(self.effects(folder), ['accept-warning'])

    def test_duplicate_actual_gtk4_warning_controls_refuse_without_delivery(self):
        child, folder, _ = self.start([dict(id='first', name='Record anyway'),
                                      dict(id='second', name='Record anyway')])
        with self.assertRaises(ValueError):
            activate(self.tree, child.pid, lambda name: name == 'Record anyway')
        self.assertEqual(self.effects(folder), [])

    def test_foreign_actual_gtk4_warning_is_not_an_owned_control(self):
        owned, owned_folder, _ = self.start([dict(id='owned-other', name='Other action')])
        _foreign, foreign_folder, _ = self.start([dict(id='foreign-warning', name='Record anyway')], suffix='Foreign')
        with self.assertRaises(ValueError):
            activate(self.tree, owned.pid, lambda name: name == 'Record anyway')
        self.assertEqual(self.effects(owned_folder), [])
        self.assertEqual(self.effects(foreign_folder), [])


if __name__ == '__main__':
    unittest.main()
