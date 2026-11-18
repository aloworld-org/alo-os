"""Person-side acceptance walk for normal Studio on a disposable alo OS VM.

This is a test, never an agent tool. It operates the application's real D-Bus
handlers and visible consent controls. Host-side decoded media checks and the
separate agent permission walk must still pass before recording acceptance.
"""

import hashlib
import json
import os
import pathlib
import re
import subprocess
import time
import urllib.request

import dbus

from atspi_tree import AtspiTree
from native_controls import activate

STUDIO = 'org.aloworld.Studio'
REFERENCE = 'org.aloworld.Studio.Verification'
PORTAL = 'org.freedesktop.impl.portal.desktop.gtk'
HOST = 'http://10.0.2.2:8767'


def command(*words):
    return subprocess.run(words, check=True, text=True, capture_output=True, timeout=30).stdout


def require(condition, message):
    if not condition:
        raise ValueError(message)


class Session:
    """Collect actual transitions and native choices without inventing outcomes."""

    def __init__(self):
        require(os.environ.get('STUDIO_PRIVATE_INSTALLED_WALK') == '1',
                'the installed walk must be explicitly enabled')
        require(pathlib.Path('/run/ostree-booted').exists() and os.getuid() == 1000,
                'the installed walk requires the disposable alo OS person session')
        self.home = pathlib.Path.home().resolve()
        self.library = self.home / '.var/app' / STUDIO / 'data/alo-studio/recordings'
        require(not self.library.exists() or not list(self.library.glob('*.json')),
                'use a fresh test account with an empty Studio library')
        running = command('flatpak', 'ps', '--columns=application').splitlines()
        require(STUDIO not in running and REFERENCE not in running,
                'the walk must own its application instances')
        self.folder = self.home / '.local/share/studio-installed-walk'
        self.folder.mkdir(parents=True, exist_ok=False)
        self.bus = dbus.SessionBus()
        self.tree = AtspiTree()
        self.events = []
        self.children = []
        self.reference = None
        self.transport = urllib.request.build_opener(urllib.request.ProxyHandler({}))

    def event(self, kind, **facts):
        self.events.append(dict(kind=kind, observed_ns=time.monotonic_ns(), **facts))

    def start(self):
        audio = subprocess.Popen([
            'pw-loopback', '--name', 'studio-verification-loop', '--channels', '1',
            '--capture-props', 'media.class=Audio/Sink node.name=studio-verification-audio node.autoconnect=false',
            '--playback-props', 'media.class=Audio/Source node.name=studio-verification-microphone node.autoconnect=false',
        ], stdout=(self.folder / 'virtual-audio.log').open('w'), stderr=subprocess.STDOUT)
        self.children.append(audio)
        deadline = time.monotonic() + 15
        while True:
            nodes = json.loads(command('pw-dump'))
            microphone = [node['id'] for node in nodes
                          if node.get('info', {}).get('props', {}).get('node.name')
                          == 'studio-verification-microphone']
            if len(microphone) == 1:
                command('wpctl', 'set-default', str(microphone[0]))
                break
            require(audio.poll() is None and time.monotonic() < deadline,
                    'the owned virtual microphone did not become available')
            time.sleep(.1)
        self.children.append(subprocess.Popen(
            ['flatpak', 'run', STUDIO], stdout=(self.folder / 'studio.log').open('w'),
            stderr=subprocess.STDOUT))
        self.wait(lambda state: state['phase'] == 'ready')

    def interface(self):
        return dbus.Interface(self.bus.get_object(
            STUDIO, '/org/aloworld/Studio/Recorder', introspect=False),
            'org.aloworld.Studio.Recorder1')

    def status(self):
        state = json.loads(str(self.interface().GetStatus(timeout=3)))
        require(state.get('version') == 1, 'the normal status interface is unavailable')
        self.event('status', state=state)
        return state

    def call(self, method, *arguments):
        self.event('call', method=method, arguments=list(arguments))
        getattr(self.interface(), method)(*arguments, timeout=5)

    def wait(self, wanted, seconds=30):
        deadline = time.monotonic() + seconds
        while True:
            try:
                state = self.status()
                if wanted(state):
                    return state
            except dbus.DBusException:
                if time.monotonic() >= deadline:
                    raise
            require(time.monotonic() < deadline, 'Studio did not reach the expected state')
            time.sleep(.1)

    def process(self, service):
        driver = dbus.Interface(self.bus.get_object(
            'org.freedesktop.DBus', '/org/freedesktop/DBus'), 'org.freedesktop.DBus')
        return int(driver.GetConnectionUnixProcessID(service, timeout=3))

    def press(self, process, matches):
        activate(self.tree, process, matches)
        self.event('native-control', process=process)

    def picker(self, source, cancel=False):
        self.call('ChooseScreen' if source == 'screen' else 'ChooseWindow')
        self.wait(lambda state: state['phase'] == 'choosing')
        deadline = time.monotonic() + 10
        while True:
            try:
                owner = self.process(PORTAL)
                if cancel:
                    self.press(owner, lambda name: name == 'Cancel')
                else:
                    self.press(owner, lambda name: name.startswith('Screen · ') if source == 'screen'
                               else name == 'Window · Studio verification window')
                    self.press(owner, lambda name: name == 'Share')
                break
            except (ValueError, dbus.DBusException):
                require(time.monotonic() < deadline, 'the native source picker could not be operated')
                time.sleep(.1)
        if cancel:
            state = self.wait(lambda state: state['phase'] == 'ready')
            require(not state['preview_ready'] and not state['microphone_active'],
                    'cancelled consent left a preview or active microphone')
        else:
            self.wait(lambda state: state['phase'] == 'selected' and state['preview_ready'])

    def start_reference(self):
        self.reference = subprocess.Popen(
            ['flatpak', 'run', REFERENCE],
            stdout=(self.folder / f'reference-{len(self.events)}.log').open('w'),
            stderr=subprocess.STDOUT)
        self.children.append(self.reference)
        time.sleep(2)
        require(self.reference.poll() is None, 'the reference window did not start')

    def stop_reference(self):
        actions = dbus.Interface(self.bus.get_object(
            REFERENCE, '/org/aloworld/Studio/Verification'), 'org.gtk.Actions')
        actions.Activate('finish-reference', dbus.Array([], signature='v'),
                         dbus.Dictionary({}, signature='sv'), timeout=5)
        self.reference.wait(timeout=10)
        require(self.reference.returncode == 0, 'the real reference reported a failure')

    def warning(self):
        # Flatpak's session-bus and accessibility proxies have different PIDs.
        # Identify the unique application root on the actual accessibility bus.
        roots = [node for node in self.tree.roots()
                 if self.tree.facts(node)['name'] == 'alo Studio']
        if len(roots) == 1:
            owner = self.tree.owner(roots[0][0])
            self.press(owner, lambda name: name == 'Record anyway')
            self.event('performance-warning-accepted')

    def begin(self, phase='recording'):
        self.call('Start')
        deadline = time.monotonic() + 30
        while True:
            state = self.status()
            if state['phase'] == phase:
                return state
            if state['phase'] == 'checking':
                try:
                    self.warning()
                except ValueError:
                    pass
            require(time.monotonic() < deadline, 'recording preparation did not finish')
            time.sleep(.1)

    def active_for(self, seconds):
        deadline = time.monotonic() + seconds
        while time.monotonic() < deadline:
            require(self.status()['phase'] == 'recording', 'recording stopped unexpectedly')
            time.sleep(min(1, max(0, deadline - time.monotonic())))

    def refused(self, method):
        before = self.status()
        try:
            self.call(method)
        except dbus.DBusException as error:
            require(error.get_dbus_name() == 'org.aloworld.Studio.Error.InvalidState',
                    f'{method} returned the wrong refusal')
        else:
            raise ValueError(f'{method} unexpectedly ran')
        after = self.status()
        require(after['phase'] == before['phase'], 'a refused command changed the phase')

    def upload(self, name, path):
        request = urllib.request.Request(HOST + '/' + name, data=path.read_bytes(), method='POST')
        with self.transport.open(request, timeout=30) as reply:
            require(reply.status == 201, f'{name} was not retained by the host')

    def export(self, identifier, title):
        self.call('RenameRecording', title)
        metadata = json.loads((self.library / f'{identifier}.json').read_text())
        require(metadata['title'] == title, 'the name was not saved in the private library')
        self.call('ExportRecording')
        self.wait(lambda state: state['export_pending'])
        deadline = time.monotonic() + 15
        while True:
            read = command('alo-app', 'alo', 'read').splitlines()
            folders = [pathlib.Path(line[3:]).resolve() for line in read if line.startswith('In /')]
            controls = command('alo-app', 'alo', 'controls').splitlines()
            save = [line for line in controls if line.startswith('button\tSave\t')
                    and 'greyed-out' not in line]
            if len(folders) == 1 and len(save) == 1:
                break
            require(time.monotonic() < deadline, 'the native Save a file sheet was not readable')
            time.sleep(.1)
        destination = folders[0] / f'{title}.webm'
        require(destination.is_relative_to(self.home) and not destination.exists(),
                'the native destination is not a fresh file in the test account')
        self.event('native-export-destination', path=str(destination))
        command('alo-app', 'alo', 'press', 'button', 'Save')
        self.wait(lambda state: not state['export_pending'])
        original = self.library / f'{identifier}.webm'
        require(destination.is_file() and hashlib.sha256(destination.read_bytes()).digest()
                == hashlib.sha256(original.read_bytes()).digest(), 'the exported copy differs')

    def recording(self, source, microphone):
        self.call('NewRecording')
        require(not self.status()['microphone_enabled'], 'a new recording enabled the microphone')
        self.start_reference()
        self.picker(source)
        self.call('SetQuality', 'lightweight')
        for seconds in (3, 5, 0):
            self.call('SetCountdown', dbus.Int64(seconds))
            require(self.status()['countdown_seconds'] == seconds, 'countdown selection did not change')
        self.call('EnableMicrophone')
        require(self.status()['microphone_enabled'], 'the microphone option did not turn on')
        self.call('DisableMicrophone')
        require(not self.status()['microphone_enabled'], 'the microphone option did not turn off')
        if microphone:
            self.call('EnableMicrophone')
            self.call('SetCountdown', dbus.Int64(3))
            self.begin('countdown')
            self.call('CancelCountdown')
            self.wait(lambda state: state['phase'] == 'selected')
            self.call('SetCountdown', dbus.Int64(0))
        self.begin()
        self.refused('Resume')
        self.active_for(18 if microphone else 12)
        if microphone:
            self.call('Pause')
            before = self.wait(lambda state: state['phase'] == 'paused')
            self.refused('Pause')
            time.sleep(5)
            after = self.status()
            require(after['phase'] == 'paused' and after['elapsed_ms'] == before['elapsed_ms']
                    and not after['microphone_active'], 'pause did not freeze capture time')
            self.call('Resume')
            self.wait(lambda state: state['phase'] == 'recording' and state['microphone_active'])
            self.active_for(12)
        self.call('Finish')
        state = self.wait(lambda state: state['phase'] == 'review', seconds=45)
        identifier = state['recording_id']
        require(isinstance(identifier, str) and re.fullmatch(r'\d+-\d+-\d+', identifier),
                'the saved recording identifier is invalid')
        self.export(identifier, 'Window demonstration' if microphone else 'Screen demonstration')
        self.call('ShowLibrary')
        self.call('NewRecording')
        self.call('ReviewRecording', identifier)
        require(self.status()['recording_id'] == identifier, 'the library recording did not reopen')
        self.stop_reference()
        self.upload(source + '.webm', self.library / f'{identifier}.webm')
        self.upload(source + '.json', self.library / f'{identifier}.json')
        if microphone:
            reference = self.home / '.var/app' / REFERENCE / 'data/studio-walk/reference.json'
            self.upload('reference.json', reference)

    def run(self):
        try:
            self.start()
            self.refused('Pause')
            self.refused('Start')
            self.start_reference()
            self.picker('screen', cancel=True)
            self.picker('window', cancel=True)
            self.stop_reference()
            self.recording('screen', False)
            self.recording('window', True)
            self.event('person-recording-flow-completed', media_acceptance='requires host decoding',
                       agent_permissions='requires separate real-agent walk')
        finally:
            report = self.folder / 'session-report.json'
            report.write_text(json.dumps({'version': 1, 'events': self.events}))
            for application in (STUDIO, REFERENCE):
                subprocess.run(['flatpak', 'kill', application], capture_output=True, timeout=10)
            for child in self.children:
                if child.poll() is None:
                    child.terminate()
                    child.wait(timeout=5)
            self.upload('session-report.json', report)


if __name__ == '__main__':
    Session().run()
