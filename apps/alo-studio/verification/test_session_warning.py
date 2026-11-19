"""GTK4's application root uses program name, independently of display label.

Normal Studio explicitly sets its GLib program name to org.aloworld.Studio.
The warning selector must activate only that native application's control;
another process named with the display label is not that application.
"""

import unittest

from installed_session import Session
from native_controls import activate
from test_native_controls import Tree, control


STUDIO = 'org.aloworld.Studio'


class SessionWarning(unittest.TestCase):
    def flow(self, *, foreign=False, real=True):
        nodes = {}
        if real:
            root = control(STUDIO, actions=())
            root['children'] = [(':1.4', '/record-anyway')]
            nodes[(':1.4', '/root')] = root
            nodes[(':1.4', '/record-anyway')] = control('Record anyway')
        if foreign:
            root = control('alo Studio', actions=())
            root['children'] = [(':1.9', '/record-anyway')]
            nodes[(':1.9', '/root')] = root
            nodes[(':1.9', '/record-anyway')] = control('Record anyway')
        tree = Tree(nodes)
        # Tree.roots normally includes every node; expose real application roots.
        tree.roots = lambda: [node for node in nodes if node[1] == '/root']
        flow = object.__new__(Session)
        flow.tree = tree
        flow.press = lambda process, matches: activate(tree, process, matches)
        events = []
        flow.event = lambda kind, **fields: events.append((kind, fields))
        return flow, tree, events

    def test_warning_uses_the_normal_applications_actual_program_name(self):
        flow, tree, events = self.flow()
        flow.warning()
        self.assertEqual(tree.sent, [((':1.4', '/record-anyway'), 0)])
        self.assertEqual(events, [('performance-warning-accepted', {})])

    def test_a_foreign_process_with_the_display_label_is_not_the_normal_application(self):
        flow, tree, events = self.flow(foreign=True)
        flow.warning()
        self.assertEqual(tree.sent, [((':1.4', '/record-anyway'), 0)])
        self.assertEqual(events, [('performance-warning-accepted', {})])

    def test_display_label_alone_never_authorizes_a_foreign_warning(self):
        flow, tree, events = self.flow(foreign=True, real=False)
        try:
            flow.warning()
        except ValueError:
            pass
        self.assertEqual(tree.sent, [])
        self.assertEqual(events, [])


if __name__ == '__main__':
    unittest.main()
