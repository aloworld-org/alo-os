"""Actual AT-SPI wire distinction; private fixture, never installed acceptance.

GNOME org.a11y.atspi.Action specifies GetActions as localized tuples, NActions
as signed i32, and GetName(index) as the machine identifier. No translated
label may choose the action sent to a native consent control.
"""

import contextlib
import json
import os
import select
import subprocess
import sys
import unittest

import dbus

from atspi_tree import AtspiTree
from native_controls import activate


SERVICE = r'''
import json, os, sys
import dbus, dbus.service
from dbus.mainloop.glib import DBusGMainLoop
from gi.repository import GLib
DBusGMainLoop(set_as_default=True)
config = json.loads(sys.argv[1])
bus = dbus.SessionBus(private=True)
names = [dbus.service.BusName(name, bus, do_not_queue=True)
         for name in ('org.a11y.Bus', 'org.a11y.atspi.Registry')]
holder = bus.get_unique_name()
ROOT = '/org/a11y/atspi/accessible/root'
CONTROL = '/org/a11y/atspi/accessible/control'
calls = []
class Address(dbus.service.Object):
    @dbus.service.method('org.a11y.Bus', in_signature='', out_signature='s')
    def GetAddress(self): return os.environ['DBUS_SESSION_BUS_ADDRESS']
class Registry(dbus.service.Object):
    @dbus.service.method('org.a11y.atspi.Accessible', in_signature='', out_signature='a(so)')
    def GetChildren(self): return [(holder, CONTROL)]
class Control(dbus.service.Object):
    @dbus.service.method('org.freedesktop.DBus.Properties', in_signature='ss', out_signature='v')
    def Get(self, interface, key):
        if interface == 'org.a11y.atspi.Accessible' and key == 'Name': return 'Running code'
        if interface == 'org.a11y.atspi.Action' and key == 'NActions':
            calls.append('count')
            constructors = {'i': dbus.Int32, 'u': dbus.UInt32, 'b': dbus.Boolean}
            return constructors[config.get('wire', 'i')](config['count'])
        raise dbus.exceptions.DBusException('unexpected fixture property')
    @dbus.service.method('org.a11y.atspi.Accessible', in_signature='', out_signature='au')
    def GetState(self): return [(1 << 8) | (1 << 24) | (1 << 25), 0]
    @dbus.service.method('org.a11y.atspi.Accessible', in_signature='', out_signature='as')
    def GetInterfaces(self): return ['org.a11y.atspi.Accessible', 'org.a11y.atspi.Action']
    @dbus.service.method('org.a11y.atspi.Accessible', in_signature='', out_signature='a(so)')
    def GetChildren(self): return []
    @dbus.service.method('org.a11y.atspi.Action', in_signature='', out_signature='a(sss)')
    def GetActions(self):
        calls.append('localized-list')
        return [(label, '', '') for label in config['localized']]
    @dbus.service.method('org.a11y.atspi.Action', in_signature='i', out_signature='s')
    def GetName(self, index):
        calls.append('name:' + str(index))
        return config['machine'][index]
    @dbus.service.method('org.a11y.atspi.Action', in_signature='i', out_signature='b')
    def DoAction(self, index):
        calls.append('do:' + str(index))
        return True
class Evidence(dbus.service.Object):
    @dbus.service.method('org.example.ActionFixture', in_signature='', out_signature='as')
    def Calls(self): return calls
objects = [Address(bus, '/org/a11y/bus'), Registry(bus, ROOT), Control(bus, CONTROL), Evidence(bus, '/fixture')]
print(holder, flush=True)
GLib.MainLoop().run()
'''


@contextlib.contextmanager
def fixture(*, machine, localized, count=None, wire='i'):
    config = dict(machine=machine, localized=localized,
                  count=len(machine) if count is None else count, wire=wire)
    child = subprocess.Popen([sys.executable, '-I', '-B', '-c', SERVICE, json.dumps(config)],
                             stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    tree = None
    try:
        if not select.select([child.stdout], [], [], 5)[0]:
            raise AssertionError('private AT-SPI fixture did not become ready')
        holder = child.stdout.readline().strip()
        if not holder.startswith(':'):
            raise AssertionError('private AT-SPI fixture exited before announcing its owner')
        tree = AtspiTree()
        tree.bus.set_exit_on_disconnect(False)
        tree.session.set_exit_on_disconnect(False)
        evidence = dbus.Interface(tree.bus.get_object(holder, '/fixture', introspect=False),
                                  'org.example.ActionFixture')
        yield tree, child.pid, (holder, '/org/a11y/atspi/accessible/control'), evidence
    finally:
        if tree is not None:
            tree.bus.close()
        if child.poll() is None:
            child.terminate()
        child.communicate(timeout=5)


@unittest.skipUnless(os.environ.get('STUDIO_PRIVATE_UI_SESSION') == '1',
                     'requires an explicitly isolated private D-Bus session')
class MachineActionNames(unittest.TestCase):
    def test_localized_labels_still_invoke_exactly_one_machine_click_at_its_actual_index(self):
        for label in ('Running code', 'Exécuter du code'):
            with self.subTest(label=label), fixture(machine=['show-menu', 'click'],
                                                    localized=['Menu', label]) as (tree, pid, _, evidence):
                activate(tree, pid, lambda name: name == 'Running code')
                self.assertEqual(list(evidence.Calls(timeout=3)),
                                 ['count', 'name:0', 'name:1',
                                  'count', 'name:0', 'name:1', 'do:1'])

    def test_a_localized_click_does_not_authorize_an_unrecognized_machine_action(self):
        with fixture(machine=['run-arbitrary-action'], localized=['click']) as (tree, pid, _, evidence):
            with self.assertRaises(ValueError):
                activate(tree, pid, lambda name: name == 'Running code')
            self.assertEqual(list(evidence.Calls(timeout=3)), ['count', 'name:0'])

    def test_invalid_action_counts_refuse_before_any_name_or_action_request(self):
        for count, wire in ((-1, 'i'), (65, 'i'), (1, 'u'), (True, 'b')):
            with self.subTest(count=count, wire=wire), fixture(machine=['click'], localized=['click'],
                                                              count=count, wire=wire) as (tree, _, node, evidence):
                with self.assertRaises(ValueError):
                    tree.facts(node)
                self.assertEqual(list(evidence.Calls(timeout=3)), ['count'])

    def test_zero_actions_are_empty_without_a_localized_action_query(self):
        with fixture(machine=[], localized=[]) as (tree, _, node, evidence):
            self.assertEqual(tree.facts(node)['actions'], [])
            self.assertEqual(list(evidence.Calls(timeout=3)), ['count'])

    def test_the_exact_limit_is_read_in_full_without_silent_truncation(self):
        names = ['focus'] * 64
        with fixture(machine=names, localized=['localized'] * 64) as (tree, _, node, evidence):
            self.assertEqual(tree.facts(node)['actions'], names)
            self.assertEqual(list(evidence.Calls(timeout=3)), ['count'] + [f'name:{i}' for i in range(64)])


if __name__ == '__main__':
    unittest.main()
