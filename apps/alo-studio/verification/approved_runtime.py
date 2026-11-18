"""Person-side native Settings and read-only evidence for real Ask requests."""

import json
import pathlib
import socket
import subprocess
import time

import dbus

from agent_settings import BASE
from pending_microphone import read_pending
from record_shape import require


def command(*words):
    return subprocess.run(words, check=True, text=True, capture_output=True, timeout=30).stdout


class ApprovalRuntime:
    """The host supplies keyboard input; this side only observes and uses Settings."""

    record_path = pathlib.Path('/var/lib/alo/record.jsonl')

    def __init__(self, settings, event):
        self.settings = settings
        self.event = event
        self.safe_to_restore = True

    def say(self, message):
        subprocess.run(['systemd-cat', '-t', 'studio-walk'], input=message + '\n',
                       text=True, check=True, capture_output=True, timeout=3)

    def pending(self, enabled):
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as connection:
            connection.settimeout(3)
            connection.connect('/run/alo/1000/agentd.sock')
            return read_pending(connection, enabled=enabled)

    def status(self):
        bus = dbus.SessionBus()
        interface = dbus.Interface(bus.get_object('org.aloworld.Studio',
            '/org/aloworld/Studio/Recorder', introspect=False), 'org.aloworld.Studio.Recorder1')
        return json.loads(str(interface.GetStatus(timeout=3)))

    def require_status(self, enabled):
        state = self.status()
        require(isinstance(state, dict) and type(state.get('version')) is int and state['version'] == 1
                and state.get('phase') == 'ready' and state.get('microphone_enabled') is enabled
                and state.get('microphone_active') is False and state.get('preview_ready') is False,
                'The microphone request did not leave the expected inactive Studio state')
        self.event('approval-status', state=state)

    def request(self, kind):
        require(kind in ('enable', 'withdraw', 'full'), 'Unknown private microphone request')
        require(self.pending(kind == 'enable') is None, 'Another native approval is already pending')
        # Once this line can reach QMP, a real request may have started. Closing
        # observers alone never makes it safe to restore Full access.
        self.safe_to_restore = False
        self.say(f'Studio approval {kind} request ready')

    def wait_pending(self, enabled, asking, delivery):
        deadline = time.monotonic() + 120
        while True:
            require(time.monotonic() < deadline, 'The real Ask request has no matching pending card')
            asking.pump()
            delivery.pump()
            card = self.pending(enabled)
            if card is not None:
                return card
            time.sleep(.05)

    def card_ready(self, kind, card):
        self.event('native-card-read', kind_of_request=kind, pending=card)
        self.say(f'Studio approval {kind} card verified')

    def wait_settings(self, *, seconds=30):
        require(type(seconds) is int and 0 < seconds <= 360, 'Invalid native Settings wait bound')
        deadline = time.monotonic() + seconds
        while True:
            try:
                command(*BASE, 'controls')
                return
            except subprocess.CalledProcessError:
                require(time.monotonic() < deadline, 'The native Settings window did not open')
                time.sleep(.1)

    def withdraw(self, card):
        self.wait_settings()
        require(self.pending(False) == card, 'The pending card changed before native withdrawal')
        self.settings.boundary('revoked')
        require(self.pending(False) == card, 'The pending card changed during native withdrawal')
        self.event('grant-withdrawn-while-pending', pending=card)
        self.say('Studio approval withdraw grant revoked')

    def turn_exited(self, facts):
        require(facts.get('result') == 'success' and type(facts.get('exit_code')) is int
                and facts['exit_code'] == 1 and type(facts.get('exit_status')) is int
                and facts['exit_status'] == 0, 'The real asking process has no successful exit')
        self.safe_to_restore = True
        self.event('asking-process-exited', facts=facts)
