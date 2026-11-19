"""Execute the real gate dependency shell with harmless temporary command shims.

Initial fetch and one explicit fallback retain bounded update/download clocks.
Fallback must invoke the existing isolated-Python Ubuntu helper, retain HTTPS
and signing/source fields, and stop if its checked fetch fails. This is CI
setup coverage, not a network performance measurement or an installed OS test.
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
WORKFLOW = ROOT / '.github/workflows/gate.yml'
SHIM = Path(__file__).with_name('fixtures') / 'gate_commands.py'
CENTRAL = 'https://archive.ubuntu.com/ubuntu'


def dependency_step():
    """Extract only the existing named step, retaining its executable body."""
    text = WORKFLOW.read_text()
    marker = '      - name: the libraries this workspace links, and a compositor for the\n'
    if text.count(marker) != 1:
        raise AssertionError('the unique dependency setup step must remain identifiable')
    step = text.split(marker, 1)[1].split('\n      - ', 1)[0]
    before, script = step.split('        run: |\n', 1)
    body = []
    for line in script.splitlines():
        if line and not line.startswith('          '):
            break
        body.append(line)
    return before, textwrap.dedent('\n'.join(body)) + '\n'


class GateNetwork(unittest.TestCase):
    def execute(self, mode):
        """Run the actual script with all side effects confined to a TempDir."""
        held = tempfile.TemporaryDirectory(prefix='studio-gate-network-')
        self.addCleanup(held.cleanup)
        place = Path(held.name)
        apt = place / 'apt'
        (apt / 'sources.list.d').mkdir(parents=True)
        before = {
            'sources.list':
                '# keep this comment\n'
                'deb [arch=amd64 signed-by=/usr/share/keyrings/ubuntu.gpg] '
                'mirror+file:/etc/apt/apt-mirrors.txt noble main restricted\n'
                'deb https://security.ubuntu.com/ubuntu noble-security main\n',
            'sources.list.d/ubuntu.sources':
                'Types: deb deb-src\nURIs:mirror+file:/etc/apt/apt-mirrors.txt\n'
                'Suites: noble noble-updates noble-security\nComponents: main universe\n'
                'Signed-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg\n',
            'sources.list.d/vendor.list':
                'deb [signed-by=/usr/share/keyrings/vendor.gpg] https://vendor.example/repo stable main\n',
        }
        for name, content in before.items():
            (apt / name).write_text(content)
        binary = place / 'bin'
        binary.mkdir()
        for name in ('sudo', 'tee', 'apt-get', 'sed', 'python3', 'getent', 'curl',
                     'weston', 'dbus-daemon', 'broadwayd', 'gtk-builder-tool'):
            target = binary / name
            shutil.copyfile(SHIM, target)
            target.chmod(0o700)
        _header, script = dependency_step()
        environment = dict(os.environ, PATH=str(binary) + ':/usr/bin:/bin',
                           GATE_FAKE_ROOT=str(place), GATE_FAKE_MODE=mode,
                           GATE_SOURCE_ROOT=str(ROOT))
        result = subprocess.run(['/bin/bash', '--noprofile', '--norc', '-e', '-o', 'pipefail', '-c', script],
                                cwd=ROOT, env=environment, capture_output=True, text=True, timeout=15)
        trace_path = place / 'trace.jsonl'
        trace = [json.loads(line) for line in trace_path.read_text().splitlines()] if trace_path.exists() else []
        after = {name: (apt / name).read_text() for name in before}
        return result, trace, before, after

    def test_fallback_uses_tested_helper_and_retains_https_sources_and_signatures(self):
        result, trace, before, after = self.execute('fallback')
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        for name, content in before.items():
            self.assertEqual(after[name], content.replace('mirror+file:/etc/apt/apt-mirrors.txt', CENTRAL), name)
        helpers = [item for item in trace if item['kind'] == 'archive-helper']
        self.assertEqual(len(helpers), 1, 'fallback must reuse ubuntu_archive.py exactly once')
        self.assertEqual(helpers[0]['changed'], 2)
        updates = [item for item in trace if item['kind'] == 'update']
        self.assertEqual(len(updates), 2)
        self.assertEqual(updates[-1]['sources'], after)
        helper_at = next(index for index, item in enumerate(trace) if item['kind'] == 'archive-helper')
        retry_at = next(index for index, item in enumerate(trace)
                        if item['kind'] == 'update' and item['number'] == 2)
        self.assertLess(helper_at, retry_at, 'HTTPS selection precedes the fallback fetch')
        self.assertIn('::warning title=Ubuntu mirror::', result.stdout)
        self.assertIn('apt path: archive.ubuntu.com (fallback', result.stdout)

    def test_fetch_clocks_dns_fallback_and_signature_defaults_remain_bounded(self):
        header, script = dependency_step()
        self.assertRegex(header, r'(?m)^        timeout-minutes: 8$')
        result, trace, _before, _after = self.execute('fallback')
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual([(item['kind'], item['seconds']) for item in trace
                          if item['kind'] in ('update', 'download')],
                         [('update', '60'), ('download', '90'), ('update', '60'), ('download', '120')])
        self.assertEqual([item['argv'] for item in trace if item['kind'] == 'resolver'],
                         [['ahostsv4', 'archive.ubuntu.com'], ['ahostsv4', 'security.ubuntu.com']])
        self.assertEqual(len([item for item in trace if item['kind'] == 'public-dns']), 2)
        self.assertEqual([item['text'] for item in trace if item['kind'] == 'pinned-host'],
                         ['198.51.100.24 archive.ubuntu.com\n', '198.51.100.24 security.ubuntu.com\n'])
        options = [item['text'] for item in trace if item['kind'] == 'network-options']
        self.assertEqual(options, ['Acquire::Retries "3";\nAcquire::http::Timeout "15";\nAcquire::https::Timeout "15";\n'])
        executable = '\n'.join(line for line in script.splitlines() if not line.lstrip().startswith('#'))
        self.assertIn('timeout 5 getent ahostsv4', executable)
        self.assertNotRegex(executable.lower(), r'allow[-_]?unauthenticated|allow[-_]?insecure|trusted\s*=\s*yes|check-valid-until\s*=\s*no')

    def test_successful_fetch_installs_only_the_same_checked_download_without_fallback(self):
        result, trace, before, after = self.execute('healthy')
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(before, after)
        self.assertFalse(any(item['kind'] in ('archive-helper', 'resolver', 'public-dns') for item in trace))
        downloads = [item for item in trace if item['kind'] == 'download']
        installs = [item for item in trace if item['kind'] == 'install']
        self.assertEqual(len(downloads), 1)
        self.assertEqual(len(installs), 1)
        self.assertTrue(downloads[0]['packages'])
        self.assertEqual(downloads[0]['packages'], installs[0]['packages'])
        self.assertIn('apt path: mirror', result.stdout)

    def test_failed_fallback_stops_before_install_or_success_report(self):
        result, trace, _before, _after = self.execute('retry-fails')
        self.assertEqual(result.returncode, 124, result.stdout + result.stderr)
        self.assertEqual(len([item for item in trace if item['kind'] == 'update']), 2)
        self.assertFalse(any(item['kind'] == 'install' for item in trace))
        self.assertNotIn('apt path:', result.stdout)
        self.assertFalse(any(item['kind'] == 'program' for item in trace))


if __name__ == '__main__':
    unittest.main()
