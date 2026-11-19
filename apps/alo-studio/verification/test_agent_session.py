"""Private launch contract; fake commands never count as installed acceptance."""

import os
import pathlib
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from agent_session import run_permission_check


class AgentSession(unittest.TestCase):
    def fixture(self, root):
        (root / 'studio-agent-probe').write_bytes(b'private test executable')
        trace = []
        child = object()

        def run(argv, **options):
            trace.append(('run', argv, options))
            return subprocess.CompletedProcess(argv, 0, stdout='', stderr='')

        def launch(argv, **options):
            path = pathlib.Path(argv[-1])
            self.assertEqual(path.read_bytes(), b'private test executable')
            self.assertEqual(path.stat().st_mode & 0o777, 0o555)
            self.assertEqual(path.parent.stat().st_mode & 0o777, 0o755)
            self.assertEqual(path.parent.stat().st_uid, os.getuid())
            trace.append(('launch', argv, options))
            return child

        def verify(process, settings, event):
            self.assertIs(process, child)
            self.assertIsNotNone(settings)
            trace.append(('verify',))

        def command(*argv):
            trace.append(('command', argv))
            return ''

        return trace, run, launch, verify, command

    def test_one_fixed_confined_child_after_marker_and_authentication_then_owned_cleanup(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            trace, run, launch, verify, command = self.fixture(root)
            with patch('agent_session.subprocess.run', run), patch('agent_session.subprocess.Popen', launch), \
                    patch('agent_session.verify_process', verify):
                run_permission_check(root, command, lambda *a, **k: None)
            launched = [row for row in trace if row[0] == 'launch']
            self.assertEqual(len(launched), 1)
            _, argv, options = launched[0]
            self.assertEqual(argv[:-1], [
                'sudo', '-n', '--', '/usr/bin/setpriv', '--reuid=60989', '--regid=60989',
                '--clear-groups', '--bounding-set=-all', '--inh-caps=-all', '--ambient-caps=-all',
                '--no-new-privs', '--pdeathsig=KILL', '/usr/bin/env', '-i', 'STUDIO_PRIVATE_AGENT_WALK=1'])
            self.assertEqual(options, dict(stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                           stderr=subprocess.PIPE, bufsize=0))
            self.assertFalse(pathlib.Path(argv[-1]).parent.exists())
            runs = [row for row in trace if row[0] == 'run']
            self.assertEqual(runs[0][1], ['systemd-cat', '-t', 'studio-walk'])
            self.assertEqual(runs[0][2]['input'], 'Studio native permission settings requested\n')
            auth = next(row for row in runs if row[1] == ['sudo', '-S', '-p', '', '-v'])
            # sudo(8): combining -k with validation suppresses ticket updates.
            invalidated = next(row for row in runs if row[1] == ['sudo', '-k'])
            self.assertLess(trace.index(invalidated), trace.index(auth))
            self.assertTrue(auth[2]['input'].endswith('\n'))
            self.assertLess(trace.index(auth), trace.index(launched[0]))
            self.assertEqual(runs[-1][1], ['sudo', '-k'])
            final_run = max(index for index, row in enumerate(trace) if row[0] == 'run')
            self.assertLess(trace.index(('verify',)), final_run)
            for _, _, options in runs:
                self.assertTrue(options['check'])
                self.assertLessEqual(options['timeout'], 30)

    def test_missing_or_symlink_probe_refuses_before_any_authentication(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            for symlink in (False, True):
                with self.subTest(symlink=symlink):
                    if symlink:
                        target = root / 'target'
                        target.write_bytes(b'other')
                        (root / 'studio-agent-probe').symlink_to(target)
                    with patch('agent_session.subprocess.run') as run, patch('agent_session.subprocess.Popen') as launch:
                        with self.assertRaises((ValueError, OSError)):
                            run_permission_check(root, lambda *a: '', lambda *a, **k: None)
                    run.assert_not_called()
                    launch.assert_not_called()

    def test_authentication_refusal_does_not_launch_or_leave_a_cached_credential(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            trace, run, launch, verify, command = self.fixture(root)
            def refuse(argv, **options):
                if '-v' in argv:
                    trace.append(('refused-auth',))
                    raise subprocess.CalledProcessError(1, argv)
                return run(argv, **options)
            with patch('agent_session.subprocess.run', refuse), patch('agent_session.subprocess.Popen') as launch:
                with self.assertRaises(subprocess.CalledProcessError):
                    run_permission_check(root, command, lambda *a, **k: None)
            launch.assert_not_called()
            self.assertEqual(trace[-1][1], ['sudo', '-k'])

    def test_launch_or_protocol_failure_cleans_owned_file_and_invalidates_auth(self):
        for failure in ('launch', 'protocol'):
            with self.subTest(failure=failure), tempfile.TemporaryDirectory() as directory:
                root = pathlib.Path(directory)
                trace, run, launch, verify, command = self.fixture(root)
                def start(argv, **options):
                    child = launch(argv, **options)
                    if failure == 'launch':
                        raise OSError('test launch failure')
                    return child
                def check(*args):
                    raise ValueError('test protocol failure')
                with patch('agent_session.subprocess.run', run), patch('agent_session.subprocess.Popen', start), \
                        patch('agent_session.verify_process', check):
                    with self.assertRaisesRegex((ValueError, OSError), 'test .* failure'):
                        run_permission_check(root, command, lambda *a, **k: None)
                argv = next(row[1] for row in trace if row[0] == 'launch')
                self.assertFalse(pathlib.Path(argv[-1]).parent.exists())
                self.assertEqual(trace[-1][1], ['sudo', '-k'])


if __name__ == '__main__':
    unittest.main()
