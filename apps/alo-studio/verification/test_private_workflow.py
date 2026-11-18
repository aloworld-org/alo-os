"""Private instrument tests rerun in CI with actual decoder/native transports."""

import pathlib
import re
import unittest


class PrivateWorkflow(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        root = pathlib.Path(__file__).resolve().parents[3]
        cls.workflow = (root / '.github/workflows/studio.yml').read_text()

    def require_private_instruments(self, workflow):
        """Check the runnable job, not matching words in unrelated jobs/comments."""
        pull_request = re.search(r'^  pull_request:\n(.*?)(?=^  \w+:)',
                                 workflow, re.M | re.S)
        self.assertIsNotNone(pull_request, 'Studio changes must trigger PR CI')
        self.assertIn("'apps/alo-studio/**'", pull_request[1])
        match = re.search(r'^  verification-instruments:\n(.*?)(?=^  [\w-]+:|\Z)',
                          workflow, re.M | re.S)
        self.assertIsNotNone(match, 'private instruments need their own job')
        section = match[1]
        # An env gate, continue-on-error or a changed shell can turn failures
        # into a successful/omitted job even with the expected commands present.
        self.assertNotRegex(section, r'(?m)^\s*(?:if|continue-on-error|shell):')
        self.assertRegex(section, r'(?m)^          ref: \$\{\{ github\.event\.pull_request\.head\.sha \|\| github\.sha \}\}$')
        run = re.search(r'^      - name: Check real decoder, native controls, reference and artifact refusals\n'
                        r'        run: \|\n((?:          [^\n]*\n?)+)', section, re.M)
        self.assertIsNotNone(run, 'required checks must execute in one failing shell')
        commands = [line[10:] for line in run[1].splitlines()]
        self.assertEqual(commands, [
            'command -v ffmpeg ffprobe',
            "/usr/bin/python3 -I -c 'import dbus, gi'",
            "dbus-run-session -- xvfb-run -a env STUDIO_PRIVATE_UI_SESSION=1 GDK_BACKEND=x11 /usr/bin/python3 -I -m unittest discover -s apps/alo-studio/verification -p 'test_*.py'",
        ], 'preflights and the complete live suite must run without failure masking')

    def test_pr_runs_the_complete_live_suite_and_fails_missing_dependencies(self):
        self.require_private_instruments(self.workflow)

    def test_missing_preflights_and_skip_gates_are_detected(self):
        mutations = [
            ("    paths: ['apps/alo-studio/**'", "    paths: ['unrelated/**'"),
            ('          command -v ffmpeg ffprobe\n', ''),
            ("          /usr/bin/python3 -I -c 'import dbus, gi'\n", ''),
            ('          command -v ffmpeg ffprobe', '          command -v ffmpeg ffprobe || true'),
            ("          /usr/bin/python3 -I -c 'import dbus, gi'", "          /usr/bin/python3 -I -c 'import dbus, gi' || true"),
            ('env STUDIO_PRIVATE_UI_SESSION=1', 'env STUDIO_PRIVATE_UI_SESSION=0'),
            ('GDK_BACKEND=x11 ', ''),
            ('dbus-run-session -- ', ''),
            ('xvfb-run -a ', ''),
            ('-I -m unittest discover', '-I -m unittest'),
            ("-p 'test_*.py'", "-p 'test_recording_health.py'"),
            ('github.event.pull_request.head.sha || github.sha', 'github.sha'),
            ('    timeout-minutes: 20', '    timeout-minutes: 20\n    if: false'),
            ('    timeout-minutes: 20', '    timeout-minutes: 20\n    continue-on-error: true'),
            ('        run: |\n          command -v', '        shell: bash {0}\n        run: |\n          command -v'),
        ]
        for original, replacement in mutations:
            with self.subTest(mutation=original):
                self.assertIn(original, self.workflow)
                broken = self.workflow.replace(original, replacement, 1)
                with self.assertRaises(AssertionError):
                    self.require_private_instruments(broken)

    def test_commands_in_other_jobs_cannot_replace_the_private_job(self):
        # The previous split-to-EOF check also accepted commands moved into
        # desktop/native jobs, where live helpers could silently stop running.
        broken = self.workflow.replace('  verification-instruments:',
                                       '  unrelated-job:', 1)
        broken += '\n  verification-instruments:\n    runs-on: ubuntu-24.04\n'
        with self.assertRaises(AssertionError):
            self.require_private_instruments(broken)

if __name__ == '__main__':
    unittest.main()
