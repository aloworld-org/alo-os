"""Native Settings orchestration against a stateful, test-only person UI.

The command seam is command(*argv) -> str. It receives only the real
alo-app/alo/--in/settings controls, read and press forms. Piped controls have
exactly three TSV columns, including an empty final flags column for buttons.
Labels/roles are independently taken from settings_page_running.rs,
settings_page_folders.rs, settings_as_read.rs, the_pickers_words.rs and the
normal org.aloworld.Studio.desktop. This fake is not installed acceptance,
does not open Settings, launch an agent, write grants or acknowledge tokens.
"""

import pathlib
import sys
import unittest
import io
from contextlib import redirect_stdout
from unittest.mock import patch

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from agent_settings import NativeSettings


# Surface::Settings.on_the_wire() is "settings". The visible title "Settings"
# is distinct: terminal-commands.md explicitly preserves the lowercase locator.
BASE = ('alo-app', 'alo', '--in', 'settings')
RUNNING = 'Running code'
ACCESS = 'What alo can use'
ASK = 'After showing you the command and waiting for you'
FULL = 'Straight away, without asking'
SEALED = 'In a sealed box that sees only this project'
STUDIO = 'alo Studio'
GIVE = 'Give alo an application'
BACK = 'Take back'
HOUR = 'For an hour'
EMPTY = ('Nothing is granted right now. No agent and no application has been '
         'granted anything on this machine, and there is nothing here to revoke')
CONFIRMED = ('alo can now use alo Studio. It is in the list below, and you can '
             'take it back there')
TAKING_BACK = ('Taking a folder or an application back stops it at once: the next '
               'thing asked under it is refused. It does not undo what was already '
               'done — the record and undo are for that.')
TOKENS = ('ask-before-grant', 'granted', 'revoked', 'full', 'ask-again')
REFUSAL = (RuntimeError, ValueError)


class Desktop:
    """Reduced real-format Settings pages, changing only through native presses.

    Observation filters deliberately do not alter press execution. A helper
    that ignores a duplicate/disabled/malformed control could still act here;
    negative tests therefore inspect attempted presses as well as exceptions.
    """

    def __init__(self):
        self.page = RUNNING
        self.policy = FULL
        self.picker = None
        self.grants = []
        self.pairings = []
        self.duration = None
        self.confirmation = CONFIRMED
        self.show_empty = True
        self.commands = []
        self.presses = []
        self.filter_controls = lambda rows: rows
        self.filter_read = lambda text: text
        self.freeze = set()
        self.failures = {}
        self.fail_at = None

    def rows(self):
        rows = [('tab', page, 'selected' if page == self.page else '')
                for page in (RUNNING, ACCESS)]
        if self.page == RUNNING:
            rows += [('radio_button', name, 'on' if name == self.policy else 'off')
                     for name in (SEALED, ASK, FULL)]
        elif self.picker == 'applications':
            rows += [('button', name, '') for name in
                     ('Firefox', 'Studio verification window', STUDIO)]
        elif self.picker == 'duration':
            rows += [('button', name, '') for name in
                     (HOUR, 'For a day', 'For a month', 'Until I remove it')]
        else:
            rows += [('button', 'Give alo a folder', ''), ('button', GIVE, '')]
            rows += [('button', BACK, '') for _ in self.grants + self.pairings]
        return rows

    def read(self):
        if self.page == RUNNING:
            return f'Running code\n{self.policy}\n'
        if self.picker == 'applications':
            return 'Select an application\nChoose one application to grant access.\n'
        if self.picker == 'duration':
            return ('How long may alo use alo Studio? You can take it back '
                    'whenever you like\n')
        lines = [ACCESS]
        if not self.grants and self.show_empty:
            lines.append(EMPTY)
        if self.grants:
            lines += [self.confirmation, 'Given to alo', *self.grants, TAKING_BACK]
        lines += ['What alo did']
        if self.pairings:
            lines += ['Computers paired with this one', *self.pairings]
        lines += ['In applications']
        return '\n'.join(lines) + '\n'

    def __call__(self, *argv):
        self.commands.append(argv)
        if argv[:4] != BASE:
            raise AssertionError(f'not the person native Settings command: {argv!r}')
        tail = argv[4:]
        if len(self.commands) > 300:
            raise RuntimeError('test-only command bound: no unbounded stale-state polling')
        if self.fail_at == len(self.commands):
            raise RuntimeError('test-only command refusal')
        if tail in self.failures:
            raise RuntimeError(self.failures[tail])
        if tail == ('controls',):
            return '\n'.join('\t'.join(row) for row in
                             self.filter_controls(self.rows())) + '\n'
        if tail == ('read',):
            return self.filter_read(self.read())
        if len(tail) != 3 or tail[0] != 'press':
            raise AssertionError(f'no raw grants, files, keyboard guesses or other API: {tail!r}')
        _, role, label = tail
        self.presses.append((role, label))
        if label in self.freeze:
            return ''
        if role == 'tab' and label in (RUNNING, ACCESS):
            self.page, self.picker = label, None
        elif self.page == RUNNING and role == 'radio_button' and label in (ASK, FULL):
            self.policy = label
        elif self.page == ACCESS and role == 'button' and label == GIVE and self.picker is None:
            self.picker = 'applications'
        elif self.page == ACCESS and role == 'button' and label == STUDIO and self.picker == 'applications':
            self.picker = 'duration'
        elif self.page == ACCESS and role == 'button' and label == HOUR and self.picker == 'duration':
            self.grants, self.duration, self.picker = [STUDIO], HOUR, None
        elif self.page == ACCESS and role == 'button' and label == BACK and self.picker is None:
            # A loose helper could revoke the wrong entry. Ambiguity must be
            # rejected before this native action is attempted.
            if self.grants:
                self.grants.pop(0)
            elif self.pairings:
                self.pairings.pop(0)
        else:
            raise AssertionError(f'not a visible source-derived native action: {tail!r}')
        return ''


class AgentSettings(unittest.TestCase):
    def test_the_published_surface_id_is_distinct_from_the_visible_window_title(self):
        desktop = Desktop()
        desktop.filter_read = lambda text: 'Settings\n' + text
        NativeSettings(desktop).prepare()
        self.assertEqual(desktop.commands[0],
                         ('alo-app','alo','--in','settings','controls'))

    def prepared(self):
        desktop = Desktop()
        settings = NativeSettings(desktop)
        settings.prepare()
        return desktop, settings

    def at_grant(self):
        desktop, settings = self.prepared()
        settings.boundary('ask-before-grant')
        settings.boundary('granted')
        return desktop, settings

    def test_five_boundaries_use_exact_native_controls_and_restore_initial_full_policy(self):
        desktop, settings = self.prepared()
        self.assertEqual(desktop.policy, FULL)
        for token, policy, grants in [
            ('ask-before-grant', ASK, []), ('granted', ASK, [STUDIO]),
            ('revoked', ASK, []), ('full', FULL, []), ('ask-again', ASK, []),
        ]:
            settings.boundary(token)
            self.assertEqual((desktop.policy, desktop.grants), (policy, grants))
        self.assertEqual(desktop.duration, HOUR)
        self.assertEqual([press for press in desktop.presses if press[0] != 'tab'], [
            ('radio_button', ASK), ('button', GIVE), ('button', STUDIO),
            ('button', HOUR), ('button', BACK), ('radio_button', FULL),
            ('radio_button', ASK),
        ])
        settings.restore()
        self.assertEqual((desktop.policy, desktop.grants), (FULL, []))
        settings.restore()
        self.assertEqual((desktop.policy, desktop.grants), (FULL, []))
        self.assertTrue(all(command[:4] == BASE for command in desktop.commands))

    def test_prepare_requires_full_policy_and_no_existing_grants_or_pairings(self):
        for change in [lambda d: setattr(d, 'policy', ASK),
                       lambda d: setattr(d, 'policy', SEALED),
                       lambda d: d.grants.append(STUDIO),
                       lambda d: d.grants.append('Documents'),
                       lambda d: d.pairings.append('Another computer'),
                       lambda d: setattr(d, 'show_empty', False)]:
            with self.subTest(change=change):
                desktop = Desktop()
                change(desktop)
                with self.assertRaises(REFUSAL):
                    NativeSettings(desktop).prepare()
                self.assertTrue(all(role == 'tab' for role, _ in desktop.presses),
                                'preflight may navigate but may not change unowned state')

    def test_prepare_rejects_ambiguous_disabled_missing_or_malformed_policy_controls(self):
        def replace_full(rows, replacement):
            return [replacement if row[1] == FULL else row for row in rows]
        mutations = [
            lambda rows: rows + [row for row in rows if row[1] == FULL],
            lambda rows: [row for row in rows if row[1] != FULL],
            lambda rows: replace_full(rows, ('radio_button', FULL, 'on,greyed-out')),
            lambda rows: replace_full(rows, ('radio_button', FULL, '')),
            lambda rows: replace_full(rows, ('radio_button', FULL, 'on,off')),
            lambda rows: replace_full(rows, ('radio_button', FULL, 'on,on')),
            lambda rows: replace_full(rows, ('radio_button', FULL, 'selected')),
            lambda rows: replace_full(rows, ('radio_button', FULL, 'on,unknown')),
            lambda rows: replace_full(rows, ('button', FULL, 'on')),
            lambda rows: replace_full(rows, ('radio_button', FULL)),
            lambda rows: replace_full(rows, ('radio_button', FULL, 'on', 'extra')),
            lambda rows: [('radio_button', ASK, 'on') if r[1] == ASK else r for r in rows],
        ]
        for mutate in mutations:
            with self.subTest(mutate=mutate):
                desktop = Desktop()
                desktop.filter_controls = mutate
                with self.assertRaises(REFUSAL):
                    NativeSettings(desktop).prepare()
                self.assertTrue(all(role == 'tab' for role, _ in desktop.presses))

    def test_native_tabs_must_be_unique_enabled_and_actually_selected_after_press(self):
        for label in (RUNNING, ACCESS):
            for fault in ('duplicate', 'disabled', 'missing', 'stale', 'missing-selected'):
                with self.subTest(label=label, fault=fault):
                    desktop = Desktop()
                    if fault == 'duplicate':
                        desktop.filter_controls = lambda rows, label=label: rows + [r for r in rows if r[1] == label]
                    elif fault == 'disabled':
                        desktop.filter_controls = lambda rows, label=label: [(r[0], r[1], 'greyed-out') if r[1] == label else r for r in rows]
                    elif fault == 'missing':
                        desktop.filter_controls = lambda rows, label=label: [r for r in rows if r[1] != label]
                    elif fault == 'missing-selected':
                        desktop.filter_controls = lambda rows, label=label: [(r[0], r[1], '') if r[1] == label else r for r in rows]
                    else:
                        desktop.page = ACCESS if label == RUNNING else RUNNING
                        desktop.freeze.add(label)
                    with self.assertRaises(REFUSAL):
                        NativeSettings(desktop).prepare()
                    if fault not in ('stale', 'missing-selected'):
                        self.assertNotIn(('tab', label), desktop.presses)

    def test_invalid_or_out_of_order_boundary_has_no_native_side_effect(self):
        desktop, settings = self.prepared()
        for token in ('granted', 'revoked', 'full', 'ask-again', 'unknown', ' ask-before-grant', 'ask-before-grant\n'):
            before = list(desktop.commands)
            with self.assertRaises(REFUSAL):
                settings.boundary(token)
            self.assertEqual(desktop.commands, before)
        settings.boundary('ask-before-grant')
        before = list(desktop.commands)
        with self.assertRaises(REFUSAL):
            settings.boundary('ask-before-grant')
        self.assertEqual(desktop.commands, before)
        other = Desktop()
        with self.assertRaises(REFUSAL):
            NativeSettings(other).boundary('ask-before-grant')
        self.assertEqual(other.commands, [])

    def test_a_policy_press_without_fresh_on_readback_cannot_complete_a_boundary(self):
        desktop, settings = self.prepared()
        desktop.freeze.add(ASK)
        with self.assertRaises(REFUSAL):
            settings.boundary('ask-before-grant')
        self.assertEqual(desktop.policy, FULL)

    def test_each_grant_control_must_be_unique_usable_and_exactly_named_before_press(self):
        for label in (GIVE, STUDIO, HOUR):
            for fault in ('duplicate', 'disabled', 'missing', 'lookalike', 'malformed'):
                with self.subTest(label=label, fault=fault):
                    desktop, settings = self.prepared()
                    settings.boundary('ask-before-grant')
                    def changed(rows):
                        if fault == 'duplicate':
                            return rows + [r for r in rows if r[1] == label]
                        if fault == 'missing':
                            return [r for r in rows if r[1] != label]
                        return [(r[0], label + ' extra', r[2]) if r[1] == label and fault == 'lookalike'
                                else (r[0], r[1], 'greyed-out') if r[1] == label and fault == 'disabled'
                                else (r[0], r[1]) if r[1] == label and fault == 'malformed'
                                else r for r in rows]
                    desktop.filter_controls = changed
                    with self.assertRaises(REFUSAL):
                        settings.boundary('granted')
                    self.assertNotIn(('button', label), desktop.presses)

    def test_missing_or_partial_grant_confirmation_or_uncreated_grant_is_not_success(self):
        for confirmation, freeze_hour in [('', False), ('alo can now use alo Studio', False),
                                           (CONFIRMED, True)]:
            with self.subTest(confirmation=confirmation, freeze=freeze_hour):
                desktop, settings = self.prepared()
                settings.boundary('ask-before-grant')
                desktop.confirmation = confirmation
                if freeze_hour:
                    desktop.freeze.add(HOUR)
                with self.assertRaises(REFUSAL):
                    settings.boundary('granted')

    def test_revoke_requires_unique_usable_owned_button_and_fresh_empty_readback(self):
        for fault in ('foreign-grant', 'replaced-grant', 'pairing', 'disabled', 'duplicate', 'stale', 'missing-empty'):
            with self.subTest(fault=fault):
                desktop, settings = self.at_grant()
                if fault == 'foreign-grant':
                    desktop.grants.append('Documents')
                elif fault == 'replaced-grant':
                    desktop.grants = ['Documents']
                elif fault == 'pairing':
                    desktop.pairings.append('Another computer')
                elif fault == 'disabled':
                    desktop.filter_controls = lambda rows: [(r[0], r[1], 'greyed-out') if r[1] == BACK else r for r in rows]
                elif fault == 'duplicate':
                    desktop.filter_controls = lambda rows: rows + [r for r in rows if r[1] == BACK]
                elif fault == 'stale':
                    desktop.freeze.add(BACK)
                else:
                    desktop.show_empty = False
                with self.assertRaises(REFUSAL):
                    settings.boundary('revoked')
                if fault in ('foreign-grant', 'replaced-grant', 'pairing', 'disabled', 'duplicate'):
                    self.assertNotIn(('button', BACK), desktop.presses)

    def test_empty_boundaries_refuse_missing_exact_empty_text_and_unowned_grants(self):
        for target in ('ask-before-grant', 'full', 'ask-again'):
            for fault in ('partial-text', 'foreign-grant', 'pairing'):
                with self.subTest(target=target, fault=fault):
                    desktop, settings = self.prepared()
                    for token in TOKENS:
                        if token == target:
                            break
                        settings.boundary(token)
                    if fault == 'partial-text':
                        desktop.filter_read = lambda text: text.replace(EMPTY, 'Nothing is granted right now.')
                    elif fault == 'foreign-grant':
                        desktop.grants.append('Documents')
                    else:
                        desktop.pairings.append('Another computer')
                    with self.assertRaises(REFUSAL):
                        settings.boundary(target)

    def test_any_command_error_propagates_instead_of_acknowledging_a_boundary(self):
        baseline = Desktop()
        settings = NativeSettings(baseline)
        settings.prepare()
        for token in TOKENS:
            settings.boundary(token)
        for failing in range(1, len(baseline.commands) + 1):
            with self.subTest(command=failing):
                desktop = Desktop()
                desktop.fail_at = failing
                settings = NativeSettings(desktop)
                with self.assertRaises(REFUSAL):
                    settings.prepare()
                    for token in TOKENS:
                        settings.boundary(token)

    def test_restore_removes_only_its_own_remaining_hour_grant_and_restores_full(self):
        desktop, settings = self.at_grant()
        settings.restore()
        self.assertEqual((desktop.grants, desktop.policy), ([], FULL))
        self.assertEqual(desktop.presses.count(('button', BACK)), 1)

    def test_restore_ambiguity_does_not_revoke_foreign_state_but_still_restores_policy(self):
        for foreign in ([STUDIO, 'Documents'], ['Documents']):
            with self.subTest(grants=foreign):
                desktop, settings = self.at_grant()
                desktop.grants = list(foreign)
                with self.assertRaises(REFUSAL):
                    settings.restore()
                self.assertEqual(desktop.grants, foreign)
                self.assertNotIn(('button', BACK), desktop.presses)
                self.assertEqual(desktop.policy, FULL)

    def test_restore_attempts_both_actions_and_preserves_the_first_cleanup_error(self):
        desktop, settings = self.at_grant()
        desktop.failures[('press', 'button', BACK)] = 'first-revoke-refusal'
        desktop.failures[('press', 'radio_button', FULL)] = 'second-policy-refusal'
        with self.assertRaisesRegex(REFUSAL, 'first-revoke-refusal'):
            settings.restore()
        self.assertIn(BASE + ('press', 'button', BACK), desktop.commands)
        self.assertIn(BASE + ('press', 'radio_button', FULL), desktop.commands)
        desktop, settings = self.at_grant()
        desktop.failures[('press', 'radio_button', FULL)] = 'policy-refusal'
        with self.assertRaises(REFUSAL):
            settings.restore()
        self.assertEqual(desktop.grants, [], 'policy failure must not skip independent grant cleanup')

    def test_restore_refuses_an_unowned_policy_change_without_overwriting_it(self):
        desktop, settings = self.at_grant()
        desktop.policy = SEALED
        with self.assertRaises(REFUSAL):
            settings.restore()
        self.assertEqual(desktop.policy, SEALED)
        self.assertNotIn(('radio_button', FULL), desktop.presses)
        self.assertEqual(desktop.grants, [], 'own grant cleanup remains independent')

    def test_helper_never_launches_a_process_or_acknowledges_a_probe_token(self):
        desktop = Desktop()
        output = io.StringIO()
        with redirect_stdout(output), patch('subprocess.Popen', side_effect=AssertionError('no probe process in NativeSettings')), patch('subprocess.run', side_effect=AssertionError('use injected person command only')):
            settings = NativeSettings(desktop)
            settings.prepare()
            for token in TOKENS:
                settings.boundary(token)
            settings.restore()
        self.assertEqual(output.getvalue(), '', 'caller sends acknowledgements only after helper success')


if __name__ == '__main__':
    unittest.main()
