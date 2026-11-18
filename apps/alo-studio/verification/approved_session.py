"""Run the private native-approval walk after provider setup in a fresh VM."""

import json
import os
import pathlib
import re
import subprocess
import time
import urllib.request

import dbus

from agent_settings import NativeSettings
from approved_runtime import ApprovalRuntime, command
from microphone_turn import check_microphone_turn
from record_shape import require

STUDIO = 'org.aloworld.Studio'


class ApprovalSession:
    """Own one app instance and restore policy only after observed agent exit."""

    def __init__(self):
        require(os.environ.get('STUDIO_PRIVATE_INSTALLED_WALK') == '1'
                and os.getuid() == 1000 and pathlib.Path('/run/ostree-booted').exists(),
                'Native approval verification requires the disposable alo OS person session')
        require(STUDIO not in command('flatpak', 'ps', '--columns=application').splitlines(),
                'The private approval walk must own its Studio instance')
        self.folder = pathlib.Path.home() / '.local/share/studio-approved-walk'
        self.folder.mkdir(parents=True, exist_ok=False)
        self.events = []
        self.child = None
        self.settings = NativeSettings(command)
        self.runtime = ApprovalRuntime(self.settings, self.event)

    def event(self, kind, **facts):
        self.events.append(dict(kind=kind, observed_ns=time.monotonic_ns(), **facts))

    def start(self):
        source = pathlib.Path('source.txt').read_bytes()
        require(re.fullmatch(rb'[0-9a-f]{40}\n', source)
                and pathlib.Path('features.txt').read_bytes() == b'normal; verification disabled\n',
                'The approval walk requires verified normal package provenance')
        commit = command('flatpak', 'info', '--user', '--show-commit', STUDIO).strip()
        require(re.fullmatch('[0-9a-f]{64}', commit), 'Studio has no deployed commit identity')
        self.event('installed-approval-provenance', source=source[:-1].decode(), studio_commit=commit)
        with (self.folder / 'studio.log').open('w') as output:
            self.child = subprocess.Popen(['flatpak', 'run', STUDIO], stdout=output, stderr=subprocess.STDOUT)
        deadline = time.monotonic() + 30
        while True:
            require(self.child.poll() is None, 'The owned Studio instance exited during startup')
            try:
                self.runtime.require_status(False)
                return
            except dbus.DBusException:
                require(time.monotonic() < deadline, 'Studio did not publish its ready state')
                time.sleep(.1)

    def open_settings(self, phase):
        self.runtime.say(f'Studio approval {phase} settings requested')
        # A failed host phase has up to 300 seconds before it advances to the
        # independent cleanup section and supplies the Settings shortcut.
        self.runtime.wait_settings(seconds=360 if phase == 'restore' else 30)

    def observed(self, result):
        self.event('microphone-turn-observed', evidence=result)
        self.runtime.say(f"Studio approval {result['kind']} observed")

    def run(self):
        failure = None
        stage = 'start'
        try:
            self.start()
            stage = 'initial-settings'
            self.open_settings('initial')
            self.settings.prepare()
            self.settings.boundary('ask-before-grant')
            self.settings.boundary('granted')
            stage = 'approved-enable'
            self.observed(check_microphone_turn(self.runtime, enabled=True, full_access=False))
            stage = 'withdraw-pending-disable'
            self.observed(check_microphone_turn(self.runtime, enabled=False, full_access=False, withdraw=True))
            stage = 'full-disable'
            self.open_settings('full')
            self.settings.boundary('full')
            self.observed(check_microphone_turn(self.runtime, enabled=False, full_access=True))
        except BaseException as error:
            failure = error
            self.event('approved-flow-failed', stage=stage, error_type=type(error).__name__,
                       message=str(error)[:2048])
        finally:
            native_ready = False
            try:
                self.open_settings('restore')
                native_ready = True
                if failure is None:
                    self.settings.boundary('ask-again')
            except BaseException as error:
                self.event('approved-cleanup-failed', error_type=type(error).__name__,
                           message=str(error)[:2048])
                if failure is None:
                    failure = error
                else:
                    failure.add_note(f'Additional approval cleanup failure: {error}')
            for action in (lambda: self.cleanup(native_ready=native_ready), self.publish):
                try:
                    action()
                except BaseException as error:
                    self.event('approved-cleanup-failed', error_type=type(error).__name__,
                               message=str(error)[:2048])
                    if failure is None:
                        failure = error
                    else:
                        failure.add_note(f'Additional approval cleanup failure: {error}')
        if failure is not None:
            raise failure

    def cleanup(self, *, native_ready=True):
        failure = None
        try:
            require(native_ready, 'Native permissions could not be restored because Settings is unavailable')
            if self.runtime.safe_to_restore:
                self.settings.restore()
                self.event('native-permissions-restored')
            else:
                if self.settings.owns_grant:
                    self.settings.revoke()
                raise ValueError('Full access was not restored because actual agent exit is unconfirmed')
        except BaseException as error:
            failure = error
        if self.child is not None:
            try:
                stopped = subprocess.run(['flatpak', 'kill', STUDIO], capture_output=True, timeout=10)
                stopped.check_returncode()
            except BaseException as error:
                if failure is None:
                    failure = error
                else:
                    failure.add_note(f'Additional owned application cleanup failure: {error}')
            try:
                self.child.wait(timeout=10)
            except BaseException as error:
                if failure is None:
                    failure = error
                else:
                    failure.add_note(f'Additional owned application cleanup failure: {error}')
                try:
                    self.child.kill()
                    self.child.wait(timeout=5)
                except BaseException as error:
                    failure.add_note(f'Additional owned application reap failure: {error}')
        if failure is not None:
            raise failure

    def publish(self):
        report = json.dumps({'version': 1, 'events': self.events}).encode()
        require(0 < len(report) <= 1024 * 1024, 'The private approval report exceeds its bound')
        (self.folder / 'approval-report.json').write_bytes(report)
        transport = urllib.request.build_opener(urllib.request.ProxyHandler({}))
        request = urllib.request.Request('http://10.0.2.2:8767/approval-report.json',
                                         data=report, method='POST', headers={'Content-Type': 'application/json'})
        with transport.open(request, timeout=30) as response:
            require(response.status == 201, 'The host did not retain the complete approval report')
