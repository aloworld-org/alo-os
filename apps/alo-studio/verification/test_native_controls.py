"""A private walk may activate only one visible control of its chosen process."""

import unittest

from native_controls import activate


class Tree:
    def __init__(self, nodes):
        self.nodes = nodes
        self.sent = []

    def owner(self, bus):
        return 42 if bus == ':1.4' else 900

    def roots(self):
        return list(self.nodes)

    def facts(self, node):
        return self.nodes[node]

    def action(self, node, index):
        self.sent.append((node, index))
        return True


def control(name='Share', states=(8, 24, 25), actions=('click',)):
    return {'name': name, 'states': set(states), 'actions': list(actions), 'children': []}


class NativeControls(unittest.TestCase):
    def test_one_visible_control_of_the_selected_process_is_activated_once(self):
        target = (':1.4', '/dialog/share')
        tree = Tree({target: control()})
        activate(tree, 42, lambda name: name == 'Share')
        self.assertEqual(tree.sent, [(target, 0)])

    def test_native_action_names_may_be_capitalized(self):
        target = (':1.4', '/dialog/share')
        tree = Tree({target: control(actions=('Click',))})
        activate(tree, 42, lambda name: name == 'Share')
        self.assertEqual(tree.sent, [(target, 0)])

    def test_another_application_with_the_same_label_is_never_activated(self):
        tree = Tree({(':1.9', '/other'): control()})
        with self.assertRaises(ValueError):
            activate(tree, 42, lambda name: name == 'Share')
        self.assertEqual(tree.sent, [])

    def test_hidden_disabled_and_non_action_labels_cannot_be_activated(self):
        for states, actions in [((8, 24), ('click',)), ((25, 24), ('click',)),
                                ((8, 25), ('click',)), ((8, 24, 25), ())]:
            tree = Tree({(':1.4', '/label'): control(states=states, actions=actions)})
            with self.assertRaises(ValueError):
                activate(tree, 42, lambda name: name == 'Share')
            self.assertEqual(tree.sent, [])

    def test_ambiguous_labels_fail_without_pressing_either_button(self):
        tree = Tree({(':1.4', '/one'): control(), (':1.4', '/two'): control()})
        with self.assertRaises(ValueError):
            activate(tree, 42, lambda name: name == 'Share')
        self.assertEqual(tree.sent, [])

    def test_cycles_do_not_repeat_or_escape_the_process(self):
        parent, child, other = (':1.4', '/root'), (':1.4', '/share'), (':1.9', '/other')
        root = control('Dialog', actions=())
        root['children'] = [child, parent, other]
        tree = Tree({parent: root, child: control(), other: control()})
        activate(tree, 42, lambda name: name == 'Share')
        self.assertEqual(tree.sent, [(child, 0)])

    def test_a_vanished_owner_or_failed_action_is_not_a_success(self):
        tree = Tree({(':1.4', '/share'): control()})
        tree.owner = lambda _: None
        with self.assertRaises(ValueError):
            activate(tree, 42, lambda name: name == 'Share')
        self.assertEqual(tree.sent, [])
        tree.owner = lambda _: 42
        tree.action = lambda *_: False
        with self.assertRaises(ValueError):
            activate(tree, 42, lambda name: name == 'Share')


if __name__ == '__main__':
    unittest.main()
