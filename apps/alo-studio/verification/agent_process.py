"""Bounded pipes between the person controller and its one private agent process."""

import math
import os
import selectors
import subprocess
import time

TOKENS = ('ask-before-grant', 'granted', 'revoked', 'full', 'ask-again')


class Replies:
    """Read exact small tokens while draining bounded error diagnostics."""

    def __init__(self, process, deadline):
        self.deadline = deadline
        self.selector = selectors.DefaultSelector()
        self.selector.register(process.stdout, selectors.EVENT_READ, 'stdout')
        self.selector.register(process.stderr, selectors.EVENT_READ, 'stderr')
        self.diagnostic_bytes = 0

    def left(self):
        left = self.deadline - time.monotonic()
        if left <= 0:
            raise ValueError('the private agent process deadline expired')
        return left

    def line(self):
        line = bytearray()
        while self.selector.get_map():
            ready = self.selector.select(self.left())
            if not ready:
                raise ValueError('the private agent process deadline expired')
            for key, _ in ready:
                data = os.read(key.fd, 4096 if key.data == 'stderr' else 1)
                if not data:
                    self.selector.unregister(key.fileobj)
                    continue
                if key.data == 'stderr':
                    self.diagnostic_bytes += len(data)
                    if self.diagnostic_bytes > 16_384:
                        raise ValueError('the private agent diagnostics exceeded their limit')
                    continue
                line.extend(data)
                if len(line) > 64:
                    raise ValueError('the private agent token exceeded its limit')
                if data == b'\n':
                    try:
                        return line.decode('ascii')[:-1]
                    except UnicodeDecodeError as error:
                        raise ValueError('the private agent token is not ASCII') from error
        if line:
            raise ValueError('the private agent token ended before its newline')
        return None

    def close(self):
        self.selector.close()


def verify_process(process, settings, event, timeout=300):
    """Own one process; acknowledge only verified native choices, then restore.

    This controller makes person-side Settings choices. It does not send agent
    verbs or approvals. A completed event requires all five boundaries, exit
    zero, no extra protocol output and successful cleanup. Local pipe tests are
    instrument checks, not evidence of installed agent permission enforcement.
    """
    failure = None
    replies = None
    try:
        if not isinstance(timeout, (float, int)) or not math.isfinite(timeout) or not 0 < timeout <= 300:
            raise ValueError('the private agent timeout must be positive and at most five minutes')
        replies = Replies(process, time.monotonic() + timeout)
        settings.prepare()
        for expected in TOKENS:
            if replies.line() != expected:
                raise ValueError('the private agent permission stages arrived out of order')
            settings.boundary(expected)
            replies.left()
            process.stdin.write((expected + '\n').encode('ascii'))
            process.stdin.flush()
            event('agent-permission-boundary', stage=expected)
        if replies.line() is not None:
            raise ValueError('the private agent emitted extra protocol output')
        if process.wait(timeout=replies.left()) != 0:
            raise ValueError('the private agent permission probe failed')
    except BaseException as error:
        failure = error
    finally:
        def retain(error):
            nonlocal failure
            if failure is None:
                failure = error
            else:
                failure.add_note(f'Additional private permission cleanup failure: {error}')
        try:
            if process.poll() is None:
                try:
                    process.terminate()
                except BaseException as error:
                    retain(error)
                else:
                    try:
                        process.wait(timeout=.5)
                    except subprocess.TimeoutExpired:
                        pass
                    except BaseException as error:
                        retain(error)
                if process.poll() is None:
                    try:
                        process.kill()
                    except BaseException as error:
                        retain(error)
                    # A failed signal does not establish that the child lives
                    # or died. Independently attempt the bounded reap.
                    try:
                        process.wait(timeout=.5)
                    except BaseException as error:
                        retain(error)
        except BaseException as error:
            retain(error)
        if replies is not None:
            try:
                replies.close()
            except BaseException as error:
                retain(error)
        for stream in (process.stdin, process.stdout, process.stderr):
            try:
                stream.close()
            except BaseException as error:
                retain(error)
        try:
            if process.poll() is None:
                raise RuntimeError('The private agent is still running; native policy restoration was refused')
            settings.restore()
        except BaseException as error:
            retain(error)
    if failure is not None:
        raise failure
    event('agent-permission-probe-completed')
