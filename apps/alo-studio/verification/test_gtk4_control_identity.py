"""The missing-ENABLED compatibility rule belongs only to verified GTK4 owners.

Properties and values follow GTK4.20.4 gtkatspiroot.c: Application.ToolkitName
is 'GTK' and Application.Version is PACKAGE_VERSION (for example '4.20.4').
Other backends retain the existing enabled/sensitive/showing refusal rules.
"""

import unittest

import dbus

from native_controls import activate


ROOT = (':1.4', '/org/a11y/atspi/accessible/root')
TARGET = (':1.4', '/org/aloworld/Studio/a11y/warning')
FOREIGN = (':1.9', '/org/a11y/atspi/accessible/root')
PROPERTIES = 'org.freedesktop.DBus.Properties'
APPLICATION = 'org.a11y.atspi.Application'


class RootProperties:
    def __init__(self, tree, root):
        self.tree, self.root = tree, root

    def Get(self, interface, name, *, timeout):
        if interface != APPLICATION or name not in {'ToolkitName', 'Version'} or not 0 < timeout <= 3:
            raise AssertionError('only bounded native Application toolkit/version reads are expected')
        self.tree.reads.append((self.root, name))
        if self.tree.change_identity and name == 'Version':
            self.tree.owners[':1.4'] = 99
        return self.tree.identity[self.root][name]


class GtkTree:
    def __init__(self, toolkit=dbus.String('GTK'), version=dbus.String('4.20.4')):
        self.identity = {ROOT: {'ToolkitName': toolkit, 'Version': version}}
        self.owners = {':1.4': 42, ':1.9': 900}
        self.root_nodes = [ROOT]
        self.sent, self.reads = [], []
        self.change_identity = False
        self.change_before_action = False
        self.target_reads = 0
        self.nodes = {
            ROOT: dict(name='org.aloworld.Studio', states=set(), actions=[], children=[TARGET]),
            TARGET: dict(name='Record anyway', states={24, 25, 30}, actions=['click'], children=[]),
        }

    def roots(self):
        return self.root_nodes.copy()

    def owner(self, bus):
        return self.owners.get(bus)

    def facts(self, node):
        if node == TARGET:
            self.target_reads += 1
            if self.change_before_action and self.target_reads >= 2:
                self.owners[':1.4'] = 99
        return self.nodes[node]

    def interface(self, node, name):
        if name != PROPERTIES or node not in self.identity:
            raise AssertionError('toolkit compatibility must be anchored to an application root')
        return RootProperties(self, node)

    def action(self, node, index):
        self.sent.append((node, index))
        return True


class Gtk4Identity(unittest.TestCase):
    def test_verified_gtk4_uses_sensitive_state_without_inventing_enabled(self):
        tree = GtkTree()
        activate(tree, 42, lambda name: name == 'Record anyway')
        self.assertEqual(tree.sent, [(TARGET, 0)])
        self.assertIn((ROOT, 'ToolkitName'), tree.reads)
        self.assertIn((ROOT, 'Version'), tree.reads)
        self.assertEqual(tree.nodes[TARGET]['states'], {24, 25, 30})

    def test_other_unknown_or_malformed_toolkit_versions_do_not_get_the_exception(self):
        for toolkit, version in [
            ('alo', '4.20.4'), ('gtk', '4.20.4'), ('GTKish', '4.20.4'), ('', '4.20.4'),
            ('GTK', '3.24.43'), ('GTK', '40.1.0'), ('GTK', ''), ('GTK', '4.bad'),
            ('GTK', True), ('GTK', 4), ('GTK', None), (True, '4.20.4'),
        ]:
            with self.subTest(toolkit=toolkit, version=version):
                tree = GtkTree(toolkit, version)
                with self.assertRaises(ValueError):
                    activate(tree, 42, lambda name: name == 'Record anyway')
                self.assertEqual(tree.sent, [])

    def test_a_foreign_gtk4_root_cannot_authorize_another_processs_missing_enabled_bit(self):
        tree = GtkTree('alo', '1.0.0')
        tree.root_nodes.append(FOREIGN)
        tree.nodes[FOREIGN] = dict(name='foreign', states=set(), actions=[], children=[])
        tree.identity[FOREIGN] = {'ToolkitName': 'GTK', 'Version': '4.20.4'}
        with self.assertRaises(ValueError):
            activate(tree, 42, lambda name: name == 'Record anyway')
        self.assertEqual(tree.sent, [])

    def test_verified_gtk4_still_requires_sensitive_showing_and_visible(self):
        for missing in (24, 25, 30):
            with self.subTest(missing=missing):
                tree = GtkTree()
                tree.nodes[TARGET]['states'].remove(missing)
                with self.assertRaises(ValueError):
                    activate(tree, 42, lambda name: name == 'Record anyway')
                self.assertEqual(tree.sent, [])

    def test_owner_change_during_toolkit_read_or_final_refresh_refuses_delivery(self):
        for when in ('toolkit', 'final-refresh', 'legacy-final-refresh'):
            with self.subTest(when=when):
                tree = GtkTree()
                if when == 'legacy-final-refresh':
                    tree.nodes[TARGET]['states'].add(8)
                tree.change_identity = when == 'toolkit'
                tree.change_before_action = when.endswith('final-refresh')
                with self.assertRaises(ValueError):
                    activate(tree, 42, lambda name: name == 'Record anyway')
                self.assertEqual(tree.sent, [])


if __name__ == '__main__':
    unittest.main()
