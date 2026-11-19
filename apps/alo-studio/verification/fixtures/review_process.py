"""Actual Studio/GTK process and test-only media on an exclusively owned bus.

No mock player or fabricated stream state is used. The test runs under its own
Xvfb; each process has a further private session/a11y bus and XDG directories.
"""

import hashlib
import json
import os
from pathlib import Path
import select
import shutil
import subprocess
import time
from xml.sax.saxutils import escape

import dbus

ACCESSIBLE = 'org.a11y.atspi.Accessible'
PROPERTIES = 'org.freedesktop.DBus.Properties'
VALUE = 'org.a11y.atspi.Value'
ROOT = '/org/a11y/atspi/accessible/root'
APPLICATION = 'org.aloworld.Studio'


def checked(argv, **kwargs):
    """Bound harmless owned-process operations, retaining stderr on failure."""
    return subprocess.run(argv, check=True, capture_output=True, text=True,
                          timeout=kwargs.pop('timeout', 5), **kwargs)


def stop(child):
    """Reap only a process this fixture created."""
    if child.poll() is None:
        child.terminate()
    try:
        child.wait(timeout=5)
    except subprocess.TimeoutExpired:
        child.kill()
        child.wait(timeout=5)


def media(folder, name, duration):
    """Generate a sealed test-only VP8/Opus file with visibly changing frames."""
    path = folder / (name + '.webm')
    checked(['ffmpeg', '-v', 'error', '-nostdin', '-f', 'lavfi', '-i',
             'testsrc2=size=320x240:rate=10', '-f', 'lavfi', '-i',
             'sine=frequency=659:sample_rate=16000', '-t', str(duration),
             '-c:v', 'libvpx', '-deadline', 'realtime', '-cpu-used', '8',
             '-c:a', 'libopus', '-frame_duration', '60', str(path)], timeout=30)
    result = checked(['ffprobe', '-v', 'error', '-show_entries', 'format=duration',
                      '-of', 'json', str(path)])
    return path, float(json.loads(result.stdout)['format']['duration'])


class ReviewProcess:
    """Public control API opens real saved files; all input is native keyboard."""

    def __init__(self, folder, binary, clips):
        self.folder = folder
        self.process = None
        self.session = None
        self.a11y = None
        self.daemon = None
        self.logs = []
        self.window = None
        self.audio = []
        runtime = folder / 'runtime'
        runtime.mkdir(mode=0o700)
        self.env = dict(os.environ, XDG_RUNTIME_DIR=str(runtime),
                        XDG_CONFIG_HOME=str(folder / 'config'),
                        XDG_DATA_HOME=str(folder / 'data'),
                        XDG_CACHE_HOME=str(folder / 'cache'),
                        XDG_STATE_HOME=str(folder / 'state'),
                        GSETTINGS_BACKEND='memory', GTK_A11Y='atspi',
                        NO_AT_BRIDGE='0', GDK_BACKEND='x11', GSK_RENDERER='cairo',
                        LANG='C.UTF-8', LANGUAGE='en')
        self.env.update(PULSE_SERVER='unix:' + str(runtime / 'pulse' / 'native'),
                        PULSE_SINK='studio_review_test', PIPEWIRE_RUNTIME_DIR=str(runtime),
                        PIPEWIRE_REMOTE='pipewire-0')
        library = folder / 'data' / 'alo-studio' / 'recordings'
        library.mkdir(parents=True)
        for name, (source, duration) in clips.items():
            shutil.copyfile(source, library / (name + '.webm'))
            (library / (name + '.json')).write_text(json.dumps(dict(
                id=name, title='Synthetic playback ' + name, created=1000,
                duration_ms=round(duration * 1000), width=320, height=240,
                microphone=True)))
        try:
            # Playback does not use file/source portals. Restrict activation on
            # this owned bus to the real accessibility launcher, so GTK cannot
            # leave an unrelated document-portal FUSE mount in the runtime dir.
            services = folder / 'services'
            services.mkdir()
            shutil.copyfile('/usr/share/dbus-1/services/org.a11y.Bus.service',
                            services / 'org.a11y.Bus.service')
            configuration = folder / 'session.conf'
            configuration.write_text('<busconfig><type>session</type>'
                                     '<listen>unix:tmpdir=/tmp</listen><auth>EXTERNAL</auth>'
                                     '<servicedir>' + escape(str(services)) + '</servicedir>'
                                     '<policy context="default"><allow send_destination="*"/>'
                                     '<allow receive_sender="*"/><allow own="*"/>'
                                     '</policy></busconfig>')
            errors = self.log('bus-stderr.log')
            self.daemon = subprocess.Popen(
                ['dbus-daemon', '--config-file=' + str(configuration), '--nofork', '--print-address=1'],
                env=self.env, stdout=subprocess.PIPE, stderr=errors, text=True)
            if not select.select([self.daemon.stdout], [], [], 5)[0]:
                raise AssertionError('owned session bus did not announce readiness')
            address = self.daemon.stdout.readline(4097).strip()
            if not address.startswith('unix:') or len(address) > 4096:
                raise AssertionError('invalid owned session bus address')
            self.env['DBUS_SESSION_BUS_ADDRESS'] = address
            self.session = dbus.bus.BusConnection(address)
            self.session.set_exit_on_disconnect(False)
            a11y_address = dbus.Interface(
                self.session.get_object('org.a11y.Bus', '/org/a11y/bus'),
                'org.a11y.Bus').GetAddress(timeout=3)
            self.env['AT_SPI_BUS_ADDRESS'] = str(a11y_address)
            self.a11y = dbus.bus.BusConnection(str(a11y_address))
            self.a11y.set_exit_on_disconnect(False)
            self.interface(('org.a11y.atspi.Registry', ROOT), ACCESSIBLE).GetChildren(timeout=3)
            self.start_audio()
            self.process = subprocess.Popen([str(binary)], env=self.env,
                                            stdout=self.log('app-stdout.log'),
                                            stderr=self.log('app-stderr.log'))
            self.until(self.ready, 'actual Studio did not publish its control service', 15)
            self.control = dbus.Interface(self.session.get_object(
                APPLICATION, '/org/aloworld/Studio/Recorder', introspect=False),
                'org.aloworld.Studio.Recorder1')
            self.until(self.find_window, 'actual Studio window did not map', 10)
            checked(['xdotool', 'windowfocus', '--sync', self.window])
            # Keep pointer outside the window: hidden GTK hover controls must
            # not be what accidentally makes these keyboard tests pass.
            checked(['xdotool', 'mousemove', '1279', '1023'])
        except BaseException:
            self.close()
            raise

    def log(self, name):
        stream = (self.folder / name).open('w')
        self.logs.append(stream)
        return stream

    def start_audio(self):
        """Route synthetic tone exclusively to a private null sink, never host audio."""
        for name in ('pipewire', 'wireplumber', 'pipewire-pulse'):
            self.audio.append(subprocess.Popen([name], env=self.env,
                              stdout=self.log(name + '-stdout.log'),
                              stderr=self.log(name + '-stderr.log')))
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            if any(child.poll() is not None for child in self.audio):
                raise AssertionError('owned audio server exited: ' + '\n'.join(
                    (self.folder / (name + '-stderr.log')).read_text(errors='replace')[-1500:]
                    for name in ('pipewire', 'wireplumber', 'pipewire-pulse')))
            info = subprocess.run(['pactl', 'info'], env=self.env, capture_output=True,
                                  text=True, timeout=2)
            if info.returncode == 0:
                checked(['pactl', 'load-module', 'module-null-sink',
                         'sink_name=studio_review_test', 'rate=48000', 'channels=2'], env=self.env)
                while time.monotonic() < deadline:
                    listed = checked(['pactl', '-f', 'json', 'list', 'sinks'], env=self.env)
                    if any(item['name'] == 'studio_review_test' for item in json.loads(listed.stdout)):
                        break
                    time.sleep(.05)
                else:
                    raise AssertionError('owned null sink did not publish after module load')
                checked(['pactl', 'set-default-sink', 'studio_review_test'], env=self.env)
                return
            time.sleep(.05)
        raise AssertionError('owned PulseAudio protocol did not become ready')

    def ready(self):
        return self.session.name_has_owner(APPLICATION)

    def find_window(self):
        found = subprocess.run(['xdotool', 'search', '--onlyvisible', '--pid',
                                str(self.process.pid)], capture_output=True,
                               text=True, timeout=3)
        windows = found.stdout.splitlines()
        if found.returncode == 0 and len(windows) == 1:
            self.window = windows[0]
            return True
        return False

    def until(self, condition, message, seconds=10):
        deadline = time.monotonic() + seconds
        last = None
        while time.monotonic() < deadline:
            if self.process is not None and self.process.poll() is not None:
                raise AssertionError(f'{message}; app exited {self.process.returncode}; '
                                     f'{self.diagnostic()}')
            try:
                result = condition()
                if result:
                    return result
            except dbus.DBusException as error:
                last = repr(error)
            time.sleep(.05)
        raise AssertionError(f'{message}; last={last}; {self.diagnostic()}')

    def diagnostic(self):
        path = self.folder / 'app-stderr.log'
        return path.read_text(errors='replace')[-3000:] if path.exists() else ''

    def interface(self, node, name):
        return dbus.Interface(self.a11y.get_object(node[0], node[1], introspect=False), name)

    def nodes(self):
        """Read bounded real state without enumerating disabled action groups."""
        pending = list(self.interface(('org.a11y.atspi.Registry', ROOT), ACCESSIBLE)
                       .GetChildren(timeout=3))
        result, seen = [], set()
        driver = dbus.Interface(self.a11y.get_object('org.freedesktop.DBus',
                                '/org/freedesktop/DBus'), 'org.freedesktop.DBus')
        while pending:
            raw = pending.pop()
            node = (str(raw[0]), str(raw[1]))
            if node in seen or int(driver.GetConnectionUnixProcessID(node[0], timeout=3)) != self.process.pid:
                continue
            seen.add(node)
            if len(seen) > 512:
                raise AssertionError('actual Review accessibility tree exceeded bound')
            accessible = self.interface(node, ACCESSIBLE)
            words = accessible.GetState(timeout=3)
            states = {index * 32 + bit for index, word in enumerate(words)
                      for bit in range(32) if int(word) & (1 << bit)}
            result.append(dict(node=node, states=states,
                               role=int(accessible.GetRole(timeout=3)),
                               name=str(self.interface(node, PROPERTIES).Get(
                                   ACCESSIBLE, 'Name', timeout=3))))
            pending.extend(accessible.GetChildren(timeout=3))
        return result

    def named(self, name, role=None):
        matches = [item for item in self.nodes() if item['name'] == name
                   and (role is None or item['role'] == role)]
        if len(matches) > 1:
            raise AssertionError(f'ambiguous actual control {name!r}: {matches}')
        return matches[0] if matches else None

    def open(self, clip):
        self.control.ReviewRecording(clip, timeout=3)
        self.until(lambda: json.loads(self.control.GetStatus(timeout=3))
                   .get('recording_id') == clip, 'ReviewRecording did not select the requested saved clip')

    def values(self):
        position = self.named('Playback position', 51)
        if position is None:
            return None
        values = [self.interface(position['node'], PROPERTIES).Get(VALUE, name, timeout=3)
                  for name in ('MinimumValue', 'MaximumValue', 'CurrentValue')]
        if not all(isinstance(value, dbus.Double) for value in values):
            raise AssertionError(f'playback Value must expose real double seconds: {values!r}')
        return position, tuple(float(value) for value in values)

    def keyboard(self, name):
        """Find the visible button using real Tab focus, then press real Space."""
        self.key_control(name, 43, 'space')

    def key_control(self, name, role, key):
        """Reach the selected native control through Tab, then send one key."""
        button = self.named(name, role)
        if button is None or not {24, 25, 30} <= button['states']:
            raise AssertionError(f'no visible sensitive {name!r} button')
        checked(['xdotool', 'windowfocus', '--sync', self.window])
        for _ in range(40):
            current = self.named(name, role)
            if current and 12 in current['states']:
                checked(['xdotool', 'key', '--clearmodifiers', key])
                return
            checked(['xdotool', 'key', '--clearmodifiers', 'Tab'])
            time.sleep(.03)
        raise AssertionError(f'keyboard Tab could not reach actual {name!r}')

    def capture(self, name):
        """Retain actual owned window pixels, not a claim inferred from Value."""
        folder = Path(os.environ['STUDIO_REVIEW_PLAYBACK_EVIDENCE'])
        folder.mkdir(parents=True, exist_ok=True)
        path = folder / (name + '.png')
        checked(['import', '-window', self.window, str(path)])
        values = self.values()
        (folder / (name + '.json')).write_text(json.dumps(dict(
            window=int(self.window), process=self.process.pid,
            values=values[1] if values else None,
            scope='local actual Review window; generated test-only media')))

    def close(self):
        if self.process is not None:
            stop(self.process)
        for child in reversed(self.audio):
            stop(child)
        for bus in (self.a11y, self.session):
            if bus is not None:
                bus.close()
        if self.daemon is not None:
            stop(self.daemon)
            if self.daemon.stdout is not None:
                self.daemon.stdout.close()
        for stream in self.logs:
            stream.close()


def provenance(binary):
    """Record only binary identity and explicit caller-supplied feature scope."""
    return dict(binary=str(binary), sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),
                feature_scope=os.environ.get('STUDIO_REVIEW_PLAYBACK_FEATURE_SCOPE', 'unverified'))
