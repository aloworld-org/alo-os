"""Private launch regressions: fake sudo policy and real unprivileged pipes.

No sudo/setpriv, account changes, native Settings or installed permissions run.
The ticket double follows sudo(8)'s documented -k behavior: with validation it
authenticates without updating credentials. Its result is not guest evidence.
Only authentication stdin is inspected for presence; fixture credentials are
never copied into traces, exceptions or reports.
"""

import pathlib
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

from agent_process import verify_process
from agent_session import run_permission_check


TOKENS = ('ask-before-grant', 'granted', 'revoked', 'full', 'ask-again')
CHILD = """
import sys
for token in ('ask-before-grant','granted','revoked','full','ask-again'):
    print(token, flush=True)
    if sys.stdin.readline() != token + '\\n':
        raise SystemExit(23)
"""
CONFINED = [
    'sudo', '-n', '--', '/usr/bin/setpriv', '--reuid=60989', '--regid=60989',
    '--clear-groups', '--bounding-set=-all', '--inh-caps=-all', '--ambient-caps=-all',
    '--no-new-privs', '--pdeathsig=KILL', '/usr/bin/env', '-i',
    'STUDIO_PRIVATE_AGENT_WALK=1',
]


class NativeChoices:
    """Observe orchestration only; no grant or policy actually changes."""

    def __init__(self, failure=None):
        self.calls = []
        self.failure = failure

    def prepare(self):
        self.calls.append('prepare')
        if self.failure == 'prepare':
            raise ValueError('fixture Settings setup unavailable')

    def boundary(self, token):
        self.calls.append(token)

    def restore(self):
        self.calls.append('restore')


class SudoTicket:
    """Minimal documented ticket semantics, not a replica of launch code."""

    def __init__(self, test, *, reject=False, launch_error=False):
        self.test = test
        self.reject = reject
        self.launch_error = launch_error
        self.cached = False
        self.calls = []
        self.started = False
        self.processes = []
        self.popen = subprocess.Popen

    def run(self, argv, **options):
        if argv == ['systemd-cat', '-t', 'studio-walk']:
            return subprocess.CompletedProcess(argv, 0, stdout='', stderr='')
        self.test.assertEqual(argv[0], 'sudo')
        if argv == ['sudo', '-k']:
            self.calls.append('invalidate')
            self.cached = False
        elif '-v' in argv:
            self.calls.append('authenticate')
            self.test.assertIn('-S', argv)
            self.test.assertTrue(options.get('input', '').endswith('\n'),
                                 'authentication requires its own stdin line')
            if '-k' in argv:
                self.cached = False
            if self.reject:
                raise subprocess.CalledProcessError(1, argv, stderr='fixture authentication refused')
            # sudo(8): -k plus a password-requiring option does not update cache.
            if '-k' not in argv:
                self.cached = True
        else:
            self.test.fail('unexpected privileged fixture operation')
        return subprocess.CompletedProcess(argv, 0, stdout='', stderr='')

    def launch(self, argv, **options):
        self.calls.append('launch')
        self.test.assertEqual(argv[:-1], CONFINED)
        executable = pathlib.Path(argv[-1])
        self.test.assertEqual(executable.stat().st_mode & 0o777, 0o555)
        self.test.assertEqual(executable.parent.stat().st_mode & 0o777, 0o755)
        if self.launch_error:
            raise OSError('fixture child spawn refused')
        self.started = self.cached
        source = CHILD if self.cached else (
            "import sys;sys.stderr.write('fixture cached authentication unavailable\\n');"
            "raise SystemExit(1)"
        )
        process = self.popen([sys.executable, '-I', '-c', source], **options)
        self.processes.append(process)
        return process

    def reap(self):
        for process in self.processes:
            if process.poll() is None:
                process.kill()
            process.wait(timeout=3)
            for stream in (process.stdin, process.stdout, process.stderr):
                stream.close()


class AuthenticationContract(unittest.TestCase):
    def fixture(self, root, **options):
        (root / 'studio-agent-probe').write_bytes(b'fixture only; never executed')
        ticket = SudoTicket(self, **options)
        self.addCleanup(ticket.reap)
        settings = NativeChoices()
        events = []
        return ticket, settings, events

    def invoke(self, root, ticket, settings, events):
        # Patching the command boundary never changes PATH or process-wide env.
        with patch('agent_session.subprocess.run', ticket.run), \
                patch('agent_session.subprocess.Popen', ticket.launch), \
                patch('agent_session.NativeSettings', return_value=settings):
            run_permission_check(root, lambda *a: '',
                                 lambda kind, **facts: events.append((kind, facts)))

    def test_successful_authentication_starts_the_fixed_child_then_invalidates_credentials(self):
        with tempfile.TemporaryDirectory() as folder:
            root = pathlib.Path(folder)
            ticket, settings, events = self.fixture(root)
            self.invoke(root, ticket, settings, events)
            self.assertTrue(ticket.started,
                            'successful validation must actually authorize the subsequent child')
            self.assertEqual(ticket.calls.count('launch'), 1)
            self.assertEqual(settings.calls, ['prepare', *TOKENS, 'restore'])
            self.assertEqual(ticket.calls[-1], 'invalidate')
            self.assertFalse(ticket.cached)
            self.assertEqual(sum(kind == 'agent-permission-probe-completed'
                                 for kind, _ in events), 1)

    def test_failed_authentication_never_launches_and_invalidates_even_an_existing_ticket(self):
        with tempfile.TemporaryDirectory() as folder:
            root = pathlib.Path(folder)
            ticket, settings, events = self.fixture(root, reject=True)
            ticket.cached = True
            with self.assertRaises(subprocess.CalledProcessError):
                self.invoke(root, ticket, settings, events)
            self.assertNotIn('launch', ticket.calls)
            self.assertEqual(ticket.calls[-1], 'invalidate')
            self.assertFalse(ticket.cached)
            self.assertEqual(settings.calls, [])
            self.assertEqual(events, [])

    def test_spawn_failure_still_invalidates_credentials_without_native_setup(self):
        with tempfile.TemporaryDirectory() as folder:
            root = pathlib.Path(folder)
            ticket, settings, events = self.fixture(root, launch_error=True)
            with self.assertRaisesRegex(OSError, 'fixture child spawn refused'):
                self.invoke(root, ticket, settings, events)
            self.assertEqual(ticket.calls[-1], 'invalidate')
            self.assertFalse(ticket.cached)
            self.assertEqual(settings.calls, [])
            self.assertEqual(events, [])


class EarlyChildFailure(unittest.TestCase):
    def start(self, source):
        process = subprocess.Popen([sys.executable, '-I', '-c', source],
                                   stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE, bufsize=0)
        def reap():
            if process.poll() is None:
                process.kill()
            process.wait(timeout=3)
            for stream in (process.stdin, process.stdout, process.stderr):
                stream.close()
        self.addCleanup(reap)
        return process

    def test_early_eof_and_nonzero_exit_are_reported_before_failing_native_preparation(self):
        process = self.start("import sys;sys.stderr.write('fixture confined child launch refused\\n');"
                             "raise SystemExit(19)")
        process.wait(timeout=3)
        settings, events = NativeChoices('prepare'), []
        with self.assertRaises(ValueError) as caught:
            verify_process(process, settings,
                           lambda kind, **facts: events.append((kind, facts)), timeout=.2)
        self.assertNotIn('prepare', settings.calls,
                         'a dead child must be diagnosed before native setup can obscure it')
        diagnostic = str(caught.exception) + '\n'.join(getattr(caught.exception, '__notes__', []))
        diagnostic += str(events)
        self.assertIn('fixture confined child launch refused', diagnostic)
        self.assertNotIn('fixture Settings setup unavailable', str(caught.exception))
        self.assertIsNotNone(process.poll())
        self.assertFalse(any(kind == 'agent-permission-probe-completed' for kind, _ in events))

    def test_missing_start_token_times_out_before_native_preparation(self):
        process = self.start('import time;time.sleep(3)')
        settings = NativeChoices('prepare')
        began = time.monotonic()
        with self.assertRaisesRegex(ValueError, 'deadline'):
            verify_process(process, settings, lambda *a, **k: None, timeout=.1)
        self.assertLess(time.monotonic() - began, 1.5)
        self.assertNotIn('prepare', settings.calls)
        self.assertIsNotNone(process.poll())


if __name__ == '__main__':
    unittest.main()
