"""Installed evidence must identify actual deployed commits, not build guesses."""

import pathlib
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from installed_session import Session, STUDIO, REFERENCE


class InstalledProvenance(unittest.TestCase):
    def probe(self, inputs, *, stdout=None, failure=None):
        runtime = 'org.gnome.Platform/x86_64/49'
        events, calls = [], []
        flow = object.__new__(Session)
        flow.event = lambda kind, **facts: events.append((kind, facts))
        outputs = {('runtime', STUDIO): runtime + '\n',
                   ('commit', STUDIO): '1' * 64 + '\n',
                   ('commit', REFERENCE): '2' * 64 + '\n',
                   ('commit', runtime): '3' * 64 + '\n'}
        outputs.update(stdout or {})

        def command(*words):
            self.assertEqual(words[:2], ('flatpak', 'info'))
            key = ('runtime' if '--show-runtime' in words else 'commit', words[-1])
            self.assertIn('--show-' + key[0], words)
            self.assertIn(key, outputs)
            calls.append(key)
            if key == failure:
                raise subprocess.CalledProcessError(17, words)
            return outputs[key]

        return flow, events, calls, command

    def write_valid_inputs(self, inputs):
        (inputs / 'source.txt').write_text('a' * 40 + '\n')
        (inputs / 'features.txt').write_text('normal; verification disabled\n')

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

    def test_source_must_be_one_lowercase_git_sha_before_queries_or_reporting(self):
        for source in ['', 'a' * 39 + '\n', 'a' * 41 + '\n', 'A' * 40 + '\n',
                       'g' * 40 + '\n', 'a' * 40 + '\n' + 'b' * 40 + '\n',
                       ' ' + 'a' * 40 + '\n', 'a' * 40 + '\nextra']:
            with self.subTest(source=source), tempfile.TemporaryDirectory() as folder:
                inputs = pathlib.Path(folder)
                self.write_valid_inputs(inputs)
                (inputs / 'source.txt').write_text(source)
                flow, events, calls, command = self.probe(inputs)
                with patch('installed_session.command', command):
                    with self.assertRaises(ValueError):
                        flow.record_provenance(inputs)
                self.assertEqual(events, [])
                self.assertEqual(calls, [])

    def test_either_missing_identity_file_refuses_without_queries_or_event(self):
        for missing in ['source.txt', 'features.txt']:
            with self.subTest(missing=missing), tempfile.TemporaryDirectory() as folder:
                inputs = pathlib.Path(folder)
                self.write_valid_inputs(inputs)
                (inputs / missing).unlink()
                flow, events, calls, command = self.probe(inputs)
                with patch('installed_session.command', command):
                    with self.assertRaises(ValueError):
                        flow.record_provenance(inputs)
                self.assertEqual(events, [])
                self.assertEqual(calls, [])

    def test_each_actual_deployed_commit_must_be_a_single_lowercase_checksum(self):
        runtime = 'org.gnome.Platform/x86_64/49'
        for target in [STUDIO, REFERENCE, runtime]:
            for output in ['', 'a' * 63 + '\n', 'a' * 65 + '\n', 'A' * 64 + '\n',
                           'g' * 64 + '\n', 'a' * 64 + '\n' + 'b' * 64 + '\n']:
                with self.subTest(target=target, output=output), tempfile.TemporaryDirectory() as folder:
                    inputs = pathlib.Path(folder)
                    self.write_valid_inputs(inputs)
                    flow, events, _, command = self.probe(inputs, stdout={('commit', target): output})
                    with patch('installed_session.command', command):
                        with self.assertRaises(ValueError):
                            flow.record_provenance(inputs)
                    self.assertEqual(events, [], 'partial identities must not be reported as provenance')

    def test_empty_or_multiple_runtime_identities_are_not_a_runtime_revision(self):
        for runtime in ['', '\n', 'org.gnome.Platform/x86_64/49\norg.gnome.Platform/x86_64/48\n']:
            with self.subTest(runtime=runtime), tempfile.TemporaryDirectory() as folder:
                inputs = pathlib.Path(folder)
                self.write_valid_inputs(inputs)
                flow, events, _, command = self.probe(inputs, stdout={('runtime', STUDIO): runtime})
                with patch('installed_session.command', command):
                    with self.assertRaises(ValueError):
                        flow.record_provenance(inputs)
                self.assertEqual(events, [])

    def test_each_failed_flatpak_observation_refuses_without_partial_provenance(self):
        for query in [('runtime', STUDIO), ('commit', STUDIO), ('commit', REFERENCE),
                      ('commit', 'org.gnome.Platform/x86_64/49')]:
            with self.subTest(query=query), tempfile.TemporaryDirectory() as folder:
                inputs = pathlib.Path(folder)
                self.write_valid_inputs(inputs)
                flow, events, calls, command = self.probe(inputs, failure=query)
                with patch('installed_session.command', command):
                    with self.assertRaises((ValueError, subprocess.CalledProcessError)):
                        flow.record_provenance(inputs)
                self.assertEqual(events, [])
                self.assertEqual(calls[-1], query)


if __name__ == '__main__':
    unittest.main()
