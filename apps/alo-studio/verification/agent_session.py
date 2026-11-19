"""Launch the private permission instrument from the disposable person's session.

The executable is a checksum-verified walk input, copied into a fresh directory
the agent can traverse but cannot change. Only the system's setpriv runs as root;
it drops identities, groups and all capabilities before executing the instrument.
No capability or approval is added to the application's public interface.
"""

import os
import pathlib
import shutil
import stat
import subprocess
import tempfile
import time

from agent_process import verify_process
from agent_settings import BASE, NativeSettings


def run(*argv, **options):
    return subprocess.run(list(argv), check=True, capture_output=True, text=True, timeout=30, **options)


def run_permission_check(inputs, command, event):
    """Use one confined process and native person choices, then remove our inputs.

    The password below is the public disposable-account fixture defined by
    the_first_walk.rs. It is supplied only to sudo's authentication stdin,
    separately from the probe protocol, never to command arguments or reports.
    Failure to authenticate refuses this check; there is no privilege fallback.
    """
    directory = None
    authenticated = False
    failure = None
    try:
        source = inputs / 'studio-agent-probe'
        with os.fdopen(os.open(source, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK), 'rb') as stream:
            metadata = os.fstat(stream.fileno())
            if not stat.S_ISREG(metadata.st_mode) or not 0 < metadata.st_size <= 128 * 1024 * 1024:
                raise ValueError('The private agent input must be a bounded regular executable')
            directory = pathlib.Path(tempfile.mkdtemp(prefix='studio-private-agent-'))
            executable = directory / 'probe'
            with executable.open('xb') as output:
                shutil.copyfileobj(stream, output)
            executable.chmod(0o555)
            directory.chmod(0o755)

        run('systemd-cat', '-t', 'studio-walk', input='Studio native permission settings requested\n')
        deadline = time.monotonic() + 30
        while True:
            try:
                command(*BASE, 'controls')
                break
            except subprocess.CalledProcessError:
                if time.monotonic() >= deadline:
                    raise
                time.sleep(.1)

        # Record the attempted authentication before calling it: even a failed
        # response must invalidate a timestamp it may have created.
        authenticated = True
        # Combining -k with -v authenticates without saving a ticket. The
        # following noninteractive launch needs a fresh, cached validation.
        run('sudo', '-k')
        run('sudo', '-S', '-p', '', '-v', input='walkpass1\n')
        settings = NativeSettings(command)
        process = subprocess.Popen([
            'sudo', '-n', '--', '/usr/bin/setpriv', '--reuid=60989', '--regid=60989',
            '--clear-groups', '--bounding-set=-all', '--inh-caps=-all', '--ambient-caps=-all',
            '--no-new-privs', '--pdeathsig=KILL', '/usr/bin/env', '-i',
            'STUDIO_PRIVATE_AGENT_WALK=1', str(executable),
        ], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=0)
        # This function owns and reaps the one child before restoring Settings.
        verify_process(process, settings, event)
    except BaseException as error:
        failure = error
    finally:
        def retain(error):
            nonlocal failure
            if failure is None:
                failure = error
            else:
                failure.add_note(f'Additional private launch cleanup failure: {error}')
        if directory is not None:
            try:
                (directory / 'probe').unlink(missing_ok=True)
                directory.rmdir()
            except BaseException as error:
                retain(error)
        if authenticated:
            try:
                run('sudo', '-k')
            except BaseException as error:
                retain(error)
    if failure is not None:
        raise failure
