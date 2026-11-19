"""Source selection must happen before package downloads on each test runner."""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


class RunnerSources(unittest.TestCase):
    def test_every_dependency_setup_selects_the_archive_before_updating_indexes(self):
        for workflow, name in (
            ('studio.yml', 'Private test transports'),
            ('studio.yml', 'Native desktop libraries'),
            ('studio.yml', 'Native libraries and test display'),
            ('walk.yml', 'Install the emulator and its firmware'),
        ):
            with self.subTest(workflow=workflow, step=name):
                text = (ROOT / '.github' / 'workflows' / workflow).read_text()
                step = text.split('      - name: ' + name + '\n', 1)[1].split('\n      - ', 1)[0]
                self.assertIn('timeout-minutes: 15', step)
                self.assertLess(step.index('sudo python3 -I tools/ci/ubuntu_archive.py'),
                                step.index('sudo apt-get update'))
                self.assertNotIn('|| true', step)

    def test_helper_changes_trigger_both_studio_paths_and_run_the_instrument_tests(self):
        text = (ROOT / '.github' / 'workflows' / 'studio.yml').read_text()
        triggers = text.split('permissions:', 1)[0]
        self.assertEqual(triggers.count("'tools/ci/**'"), 2)
        self.assertIn("python3 -I -B -m unittest discover -s tools/ci -p 'test_*.py'", text)


if __name__ == '__main__':
    unittest.main()
