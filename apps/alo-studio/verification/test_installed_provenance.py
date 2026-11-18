"""Installed evidence must identify actual deployed commits, not build guesses."""

import pathlib
import tempfile
import unittest
from unittest.mock import patch

from installed_session import Session, STUDIO, REFERENCE


class InstalledProvenance(unittest.TestCase):
    def test_report_uses_normal_source_and_actual_application_and_runtime_commits(self):
        with tempfile.TemporaryDirectory() as folder:
            inputs = pathlib.Path(folder)
            source = 'a' * 40
            (inputs / 'source.txt').write_text(source + '\n')
            (inputs / 'features.txt').write_text('normal; verification disabled\n')
            runtime = 'org.gnome.Platform/x86_64/49'
            commits = {STUDIO: '1' * 64, REFERENCE: '2' * 64, runtime: '3' * 64}
            events = []
            flow = object.__new__(Session)
            flow.event = lambda kind, **facts: events.append((kind, facts))

            def command(*words):
                if '--show-runtime' in words:
                    self.assertEqual(words[-1], STUDIO)
                    return runtime + '\n'
                self.assertIn('--show-commit', words)
                return commits[words[-1]] + '\n'

            with patch('installed_session.command', command):
                flow.record_provenance(inputs)
            self.assertEqual(len(events), 1)
            kind, facts = events[0]
            self.assertEqual(kind, 'installed-provenance')
            self.assertEqual(facts['source'], source)
            self.assertEqual(facts['runtime'], runtime)
            self.assertEqual(facts['studio_commit'], commits[STUDIO])
            self.assertEqual(facts['reference_commit'], commits[REFERENCE])
            self.assertEqual(facts['runtime_commit'], commits[runtime])

    def test_report_refuses_missing_or_unverified_normal_identity(self):
        for features in ['normal; verification enabled\n', '', 'normal; verification disabled']:
            with self.subTest(features=features), tempfile.TemporaryDirectory() as folder:
                inputs = pathlib.Path(folder)
                (inputs / 'source.txt').write_text('a' * 40 + '\n')
                (inputs / 'features.txt').write_text(features)
                flow = object.__new__(Session)
                flow.event = lambda *args, **kwargs: self.fail('invalid provenance was reported')
                with self.assertRaises(ValueError):
                    flow.record_provenance(inputs)


if __name__ == '__main__':
    unittest.main()
