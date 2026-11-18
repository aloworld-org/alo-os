"""Real local pipes exercise the person controller, never an installed daemon."""

import subprocess
import sys
import time
import unittest
from unittest.mock import patch

from agent_process import Replies, verify_process

TOKENS = ('ask-before-grant', 'granted', 'revoked', 'full', 'ask-again')
CHILD = """
import sys
for token in ('ask-before-grant','granted','revoked','full','ask-again'):
    print(token, flush=True)
    if sys.stdin.readline() != token + '\\n':
        raise SystemExit(23)
"""


class Settings:
    def __init__(self, fail=None):
        self.calls = []
        self.fail = fail

    def prepare(self):
        self.calls.append('prepare')
        if self.fail == 'prepare':
            raise ValueError('native preparation failed')

    def boundary(self, token):
        self.calls.append(token)
        if self.fail == token:
            raise ValueError('native boundary failed')

    def restore(self):
        self.calls.append('restore')
        if self.fail == 'restore':
            raise ValueError('native restoration failed')


class ProcessFlow(unittest.TestCase):
    def ignoring_term(self):
        process = self.start("import signal,time;signal.signal(signal.SIGTERM,signal.SIG_IGN);print('ready',flush=True);time.sleep(10)")
        self.assertEqual(process.stdout.readline(), b'ready\n')
        return process

    def start(self, source):
        process = subprocess.Popen([sys.executable, '-I', '-c', source],
                                   stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE)
        self.addCleanup(self.reap, process)
        return process

    @staticmethod
    def reap(process):
        if process.poll() is None:
            process.kill()
        process.wait(timeout=3)
        for stream in (process.stdin, process.stdout, process.stderr):
            stream.close()

    @staticmethod
    def events(target):
        return lambda kind, **facts: target.append((kind, facts))

    def test_one_owned_process_acknowledges_all_boundaries_and_exits_before_restoration(self):
        process = self.start(CHILD)
        settings, events = Settings(), []
        original = settings.restore
        def restore():
            self.assertIsNotNone(process.poll(), 'restore must follow probe shutdown')
            original()
        settings.restore = restore
        verify_process(process, settings, self.events(events), timeout=3)
        self.assertEqual(settings.calls, ['prepare', *TOKENS, 'restore'])
        self.assertEqual(process.returncode, 0)
        self.assertEqual([facts['stage'] for kind, facts in events
                          if kind == 'agent-permission-boundary'], list(TOKENS))
        self.assertEqual(sum(kind == 'agent-permission-probe-completed' for kind, _ in events), 1)

    def test_native_failure_never_acknowledges_or_advances_and_still_reaps_and_restores(self):
        for failure in ('prepare', *TOKENS):
            with self.subTest(failure=failure):
                process, settings, events = self.start(CHILD), Settings(failure), []
                with self.assertRaises(ValueError):
                    verify_process(process, settings, self.events(events), timeout=3)
                expected = ['prepare'] if failure == 'prepare' else ['prepare', *TOKENS[:TOKENS.index(failure) + 1]]
                self.assertEqual(settings.calls, [*expected, 'restore'])
                self.assertIsNotNone(process.poll())
                self.assertFalse(any(kind == 'agent-permission-probe-completed' for kind, _ in events))

    def test_wrong_out_of_order_partial_and_extra_protocol_output_never_passes(self):
        for source in ("print('granted', flush=True)", "print('ask-before-grant extra', flush=True)",
                       "print('ask-before-grant', end='', flush=True)", "print('x'*500, flush=True)",
                       CHILD + "\nprint('extra-stage', flush=True)", CHILD + "\nraise SystemExit(17)"):
            with self.subTest(source=source):
                process, events = self.start(source), []
                with self.assertRaises(ValueError):
                    verify_process(process, Settings(), self.events(events), timeout=3)
                self.assertIsNotNone(process.poll())
                self.assertFalse(any(kind == 'agent-permission-probe-completed' for kind, _ in events))

    def test_partial_line_and_excessive_diagnostics_are_bounded(self):
        for source in ("import sys,time;sys.stdout.write('ask-');sys.stdout.flush();time.sleep(3)",
                       "import sys,time;sys.stderr.write('x'*17000);sys.stderr.flush();time.sleep(3)"):
            process = self.start(source)
            began = time.monotonic()
            with self.assertRaises(ValueError):
                verify_process(process, Settings(), self.events([]), timeout=.15)
            self.assertLess(time.monotonic() - began, 1.5)
            self.assertIsNotNone(process.poll())

    def test_failed_restoration_prevents_completion_and_preserves_primary_failure(self):
        process, events = self.start(CHILD), []
        with self.assertRaisesRegex(ValueError, 'restoration'):
            verify_process(process, Settings('restore'), self.events(events), timeout=3)
        self.assertFalse(any(kind == 'agent-permission-probe-completed' for kind, _ in events))
        settings = Settings('granted')
        def failed_restore():
            raise ValueError('secondary restoration failure')
        settings.restore = failed_restore
        with self.assertRaisesRegex(ValueError, 'native boundary') as caught:
            verify_process(self.start(CHILD), settings, self.events([]), timeout=3)
        self.assertTrue(any('restoration' in note for note in getattr(caught.exception, '__notes__', [])))

    def test_a_child_ignoring_terminate_is_killed_and_reaped_before_settings_restore(self):
        process = self.ignoring_term()
        settings, events = Settings('prepare'), []
        original = settings.restore
        def restore():
            self.assertIsNotNone(process.poll(), 'an owned child must be reaped before Settings restoration')
            original()
        settings.restore = restore
        began = time.monotonic()
        with patch.object(process, 'kill', wraps=process.kill) as kill:
            with self.assertRaisesRegex(ValueError, 'native preparation'):
                verify_process(process, settings, self.events(events), timeout=3)
            kill.assert_called_once()
        self.assertLess(time.monotonic() - began, 2)
        self.assertEqual(settings.calls, ['prepare', 'restore'])
        self.assertFalse(any(kind == 'agent-permission-probe-completed' for kind, _ in events))

    def test_terminate_refusal_still_attempts_kill_and_reap_and_preserves_first_failure(self):
        process = self.ignoring_term()
        settings, events = Settings('prepare'), []
        with patch.object(process, 'terminate', side_effect=PermissionError('terminate-refusal')), \
                patch.object(process, 'kill', wraps=process.kill) as kill:
            with self.assertRaisesRegex(ValueError, 'native preparation') as caught:
                verify_process(process, settings, self.events(events), timeout=3)
            kill.assert_called_once()
        self.assertIsNotNone(process.poll())
        self.assertEqual(settings.calls, ['prepare', 'restore'])
        self.assertTrue(any('terminate-refusal' in note for note in getattr(caught.exception, '__notes__', [])))
        self.assertFalse(any(kind == 'agent-permission-probe-completed' for kind, _ in events))

    def test_wait_refusal_still_attempts_kill_and_reap_and_preserves_first_failure(self):
        process = self.ignoring_term()
        settings, events = Settings('prepare'), []
        wait = process.wait
        attempts = []
        def failing_first_wait(*args, **kwargs):
            attempts.append(None)
            if len(attempts) == 1:
                raise OSError('wait-refusal')
            return wait(*args, **kwargs)
        with patch.object(process, 'wait', side_effect=failing_first_wait), \
                patch.object(process, 'kill', wraps=process.kill) as kill:
            with self.assertRaisesRegex(ValueError, 'native preparation') as caught:
                verify_process(process, settings, self.events(events), timeout=3)
            kill.assert_called_once()
        self.assertGreaterEqual(len(attempts), 2)
        self.assertIsNotNone(process.poll())
        self.assertEqual(settings.calls, ['prepare', 'restore'])
        self.assertTrue(any('wait-refusal' in note for note in getattr(caught.exception, '__notes__', [])))

    def test_selector_close_refusal_does_not_skip_pipe_closes_or_native_restoration(self):
        process = self.start(CHILD)
        settings, events = Settings('prepare'), []
        close = Replies.close
        def failing_close(replies):
            close(replies)
            raise OSError('selector-close-refusal')
        with patch.object(Replies, 'close', failing_close):
            with self.assertRaisesRegex(ValueError, 'native preparation') as caught:
                verify_process(process, settings, self.events(events), timeout=3)
        self.assertEqual(settings.calls, ['prepare', 'restore'])
        self.assertTrue(all(stream.closed for stream in (process.stdin, process.stdout, process.stderr)))
        self.assertTrue(any('selector-close-refusal' in note for note in getattr(caught.exception, '__notes__', [])))
        self.assertFalse(any(kind == 'agent-permission-probe-completed' for kind, _ in events))


if __name__ == '__main__':
    unittest.main()
