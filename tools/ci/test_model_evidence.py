"""Exercise the real fixed-trial shell's diagnostic retention without a provider.

The checked workflow still asks the same five trials/all requests at both
policies and preserves failure. Only the existing opt-in prompt transcript is
retained, in distinct files, including on failure. Fake transcript content is
explicitly synthetic test input, never installed execution or model evidence.
All shell effects are confined to an owned temporary working directory.
"""

import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import textwrap
import unittest


ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / '.github/workflows/every-try.yml'
SHIM = Path(__file__).with_name('fixtures') / 'model_trial.py'
TRANSCRIPTS = ('model-evidence/full-access.txt', 'model-evidence/ask-each-time.txt')


def trial_step():
    """Extract the unique existing executable step, rather than copy its shell."""
    source = WORKFLOW.read_text()
    marker = '      - name: Try the set against the real model\n'
    if source.count(marker) != 1:
        raise AssertionError('the real trial step must remain uniquely identifiable')
    step = source.split(marker, 1)[1].split('\n      - ', 1)[0]
    header, body = step.split('        run: |\n', 1)
    script = []
    for line in body.splitlines():
        if line and not line.startswith('          '):
            break
        script.append(line)
    return header, textwrap.dedent('\n'.join(script)) + '\n'


def upload_step():
    """Find the transcript upload's action block without accepting unrelated uploads."""
    blocks = WORKFLOW.read_text().split('\n      - ')[1:]
    found = [block for block in blocks
             if 'uses: actions/upload-artifact@v4' in block and 'model-evidence/' in block]
    if len(found) != 1:
        raise AssertionError('exactly one explicit model transcript upload is required')
    return found[0]


class ModelEvidence(unittest.TestCase):
    def execute(self, failed='none', *, key=True):
        """Run extracted shell in a fresh directory, with the trial path replaced."""
        held = tempfile.TemporaryDirectory(prefix='studio-model-evidence-')
        self.addCleanup(held.cleanup)
        folder = Path(held.name)
        trial = folder / 'target/release/alo-every-try'
        trial.parent.mkdir(parents=True)
        shutil.copyfile(SHIM, trial)
        trial.chmod(0o700)
        # Keep the environment deliberately small; no real provider credential
        # or host filesystem/network command is supplied to the fake runner.
        fake_key = 'PRIVATE-TEST-ONLY-SENTINEL-NOT-A-PROVIDER-CREDENTIAL'
        environment = {
            'PATH': '/usr/bin:/bin',
            'HOME': str(folder),
            'GITHUB_WORKSPACE': str(folder),
            'RUNNER_TEMP': str(folder / 'runner-temp'),
            'GITHUB_STEP_SUMMARY': str(folder / 'summary.txt'),
            'ALO_PROVIDER_KEY': fake_key if key else '',
            'ALO_EVERY_TRY_ENDPOINT': 'https://api.openai.com',
            'ALO_EVERY_TRY_MODEL': 'gpt-4o-mini',
            'ALO_EVERY_TRY_TRIES': '5',
            'ALO_EVERY_TRY_ONLY': '',
            'MODEL_FAKE_FAILURE': failed,
        }
        _, script = trial_step()
        result = subprocess.run(
            ['/bin/bash', '--noprofile', '--norc', '-e', '-o', 'pipefail', '-c', script],
            cwd=folder, env=environment, capture_output=True, text=True, timeout=10)
        calls_path = folder / 'trial-calls.jsonl'
        calls = ([json.loads(line) for line in calls_path.read_text().splitlines()]
                 if calls_path.exists() else [])
        files = {str(path.relative_to(folder)): path.read_text()
                 for path in folder.glob('model-evidence/*') if path.is_file()}
        self.assertNotIn(fake_key, result.stdout + result.stderr + ''.join(files.values()))
        return result, calls, files

    def assert_both_policies_and_transcripts(self, calls, files):
        self.assertEqual([call['level'] for call in calls], ['full-access', 'ask-each-time'])
        self.assertEqual([call['shown'] for call in calls], list(TRANSCRIPTS))
        self.assertEqual(set(files), set(TRANSCRIPTS), 'retain only the two fixed prompt transcripts')
        for call, name in zip(calls, TRANSCRIPTS):
            self.assertEqual(call['tries'], '5')
            self.assertEqual(call['only'], '')
            self.assertEqual(call['model'], 'gpt-4o-mini')
            self.assertEqual(call['endpoint'], 'https://api.openai.com')
            self.assertEqual(call['argv'], [])
            self.assertEqual(files[name],
                             f"=== question 1 ===\nThe request: synthetic {call['level']} recording request.\n"
                             '=== question 2 ===\nThe answer: synthetic completed status.\n'
                             'Compare the whole request with what came back.\n')

    def test_success_retains_separate_actual_opt_in_outputs_for_both_policies(self):
        result, calls, files = self.execute()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assert_both_policies_and_transcripts(calls, files)

    def test_either_policy_failure_still_runs_both_and_retains_both_transcripts(self):
        for failed in ('full-access', 'ask-each-time', 'both'):
            with self.subTest(failed=failed):
                result, calls, files = self.execute(failed)
                self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
                self.assert_both_policies_and_transcripts(calls, files)

    def test_missing_key_refuses_before_any_trial_and_writes_no_prompt_artifact(self):
        result, calls, files = self.execute(key=False)
        self.assertEqual(result.returncode, 1)
        self.assertEqual(calls, [])
        self.assertEqual(files, {})

    def test_upload_retains_only_fixed_transcripts_even_after_trial_failure(self):
        step = upload_step()
        self.assertRegex(step, r'(?m)^        if: (?:\$\{\{\s*)?always\(\)(?:\s*\}\})?\s*$')
        self.assertIn('if-no-files-found: ignore', step,
                      'missing provider key must preserve its original refusal without inventing prompts')
        self.assertIn('path: |\n', step)
        paths = []
        for line in step.split('path: |\n', 1)[1].splitlines():
            if line and not line.startswith('            '):
                break
            if line.strip():
                paths.append(line.strip())
        self.assertEqual(paths, list(TRANSCRIPTS), 'never upload the workspace, summary or environment')
        self.assertNotIn('secrets.', step)
        self.assertNotIn('ALO_PROVIDER_KEY', step)

    def test_fixed_workflow_inputs_and_no_environment_dump_are_preserved(self):
        header, script = trial_step()
        for entry in (
            'ALO_PROVIDER_KEY: ${{ secrets.ALO_WALK_PROVIDER_KEY }}',
            'ALO_EVERY_TRY_ENDPOINT: https://api.openai.com',
            'ALO_EVERY_TRY_MODEL: gpt-4o-mini',
            "ALO_EVERY_TRY_TRIES: ${{ inputs.tries || '5' }}",
            "ALO_EVERY_TRY_ONLY: ${{ inputs.only || '' }}",
        ):
            self.assertIn(entry, header)
        executable = '\n'.join(line for line in script.splitlines()
                               if not line.lstrip().startswith('#'))
        self.assertNotRegex(executable, r'(?m)(?:^|[;|])\s*(?:printenv|env\s*$|set\s*$|export\s+-p)')
        self.assertNotRegex(executable, r'(?m)^\s*set\s+-[^\n]*x')


if __name__ == '__main__':
    unittest.main()
