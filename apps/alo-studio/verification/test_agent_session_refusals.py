"""Independent launch refusals using owned files and fake privileged commands.

These tests never execute sudo/setpriv or claim guest process containment.
"""

import pathlib
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from agent_session import run_permission_check


class LaunchRefusals(unittest.TestCase):
    def fixture(self, root):
        (root / 'studio-agent-probe').write_bytes(b'test-only executable bytes')
        directory = root / 'owned-launch'
        def staging(**options):
            self.assertEqual(options, {'prefix': 'studio-private-agent-'})
            directory.mkdir(mode=0o700)
            return str(directory)
        calls = []
        def run(argv, **options):
            calls.append(argv)
            return subprocess.CompletedProcess(argv, 0, stdout='', stderr='')
        return directory, staging, calls, run

    def test_staging_failure_is_removed_before_any_marker_or_privileged_command(self):
        with tempfile.TemporaryDirectory() as folder:
            root = pathlib.Path(folder)
            directory, staging, calls, run = self.fixture(root)
            with patch('agent_session.tempfile.mkdtemp', staging), \
                    patch('agent_session.shutil.copyfileobj', side_effect=OSError('copy-refusal')), \
                    patch('agent_session.subprocess.run', run), \
                    patch('agent_session.subprocess.Popen') as launch:
                with self.assertRaisesRegex(OSError, 'copy-refusal'):
                    run_permission_check(root, lambda *a: '', lambda *a, **k: None)
            self.assertFalse(directory.exists())
            self.assertEqual(calls, [])
            launch.assert_not_called()

    def test_settings_unavailable_refuses_before_authentication_or_probe_launch(self):
        with tempfile.TemporaryDirectory() as folder:
            root = pathlib.Path(folder)
            directory, staging, calls, run = self.fixture(root)
            attempts = []
            def absent(*argv):
                attempts.append(argv)
                raise subprocess.CalledProcessError(1, argv)
            with patch('agent_session.tempfile.mkdtemp', staging), \
                    patch('agent_session.time.monotonic', side_effect=[0, 31]), \
                    patch('agent_session.subprocess.run', run), \
                    patch('agent_session.subprocess.Popen') as launch:
                with self.assertRaises(subprocess.CalledProcessError):
                    run_permission_check(root, absent, lambda *a, **k: None)
            self.assertEqual(attempts, [('alo-app', 'alo', '--in', 'Settings', 'controls')])
            self.assertEqual(calls, [['systemd-cat', '-t', 'studio-walk']])
            self.assertFalse(directory.exists())
            launch.assert_not_called()

    def test_missing_sudo_refuses_launch_and_still_attempts_timestamp_invalidation(self):
        with tempfile.TemporaryDirectory() as folder:
            root = pathlib.Path(folder)
            directory, staging, calls, original = self.fixture(root)
            def unavailable(argv, **options):
                original(argv, **options)
                if argv[0] == 'sudo':
                    raise FileNotFoundError('sudo-unavailable')
            with patch('agent_session.tempfile.mkdtemp', staging), \
                    patch('agent_session.subprocess.run', unavailable), \
                    patch('agent_session.subprocess.Popen') as launch:
                with self.assertRaisesRegex(FileNotFoundError, 'sudo-unavailable') as caught:
                    run_permission_check(root, lambda *a: '', lambda *a, **k: None)
            self.assertEqual(calls[-1], ['sudo', '-k'])
            self.assertTrue(any('sudo-unavailable' in note for note in getattr(caught.exception, '__notes__', [])))
            self.assertFalse(directory.exists())
            launch.assert_not_called()

    def test_file_cleanup_refusal_does_not_skip_sudo_cleanup_or_replace_protocol_failure(self):
        with tempfile.TemporaryDirectory() as folder:
            root = pathlib.Path(folder)
            directory, staging, calls, run = self.fixture(root)
            original = pathlib.Path.unlink
            def refuse_owned_file(path, **options):
                if path == directory / 'probe':
                    raise OSError('owned-file-cleanup-refusal')
                return original(path, **options)
            with patch('agent_session.tempfile.mkdtemp', staging), \
                    patch('agent_session.subprocess.run', run), \
                    patch('agent_session.subprocess.Popen'), \
                    patch('agent_session.verify_process', side_effect=ValueError('protocol-refusal')), \
                    patch.object(pathlib.Path, 'unlink', refuse_owned_file):
                with self.assertRaisesRegex(ValueError, 'protocol-refusal') as caught:
                    run_permission_check(root, lambda *a: '', lambda *a, **k: None)
            self.assertEqual(calls[-1], ['sudo', '-k'])
            self.assertTrue(any('owned-file-cleanup-refusal' in note for note in getattr(caught.exception, '__notes__', [])))
            self.assertTrue((directory / 'probe').exists(), 'fixture must establish the failed cleanup')
            original(directory / 'probe')
            directory.rmdir()


if __name__ == '__main__':
    unittest.main()
