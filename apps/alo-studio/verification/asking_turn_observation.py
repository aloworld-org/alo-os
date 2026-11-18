"""Read-only system-bus observation of one fresh Ask alo process.

This proves process identity and exit, not that the requested action succeeded.
The installed controller must separately check native consent, record and app.
Injected connections are a test seam and remain owned by the caller.
"""

import math
import time

import dbus
from dbus.mainloop.glib import DBusGMainLoop
from gi.repository import GLib

from asking_turn_shape import UNIT, SERVICE, asking, unit_path, pending, starting, unchanged, completed

DBusGMainLoop(set_as_default=True)
NAME = 'org.freedesktop.systemd1'
MANAGER = NAME + '.Manager'
PATH = '/org/freedesktop/systemd1'
PROPERTIES = 'org.freedesktop.DBus.Properties'
LIMIT = 2048


class AskingTurnObservation:
    """Subscribe before the empty baseline; keep terminal facts through unit GC."""

    def __init__(self, *, bus_connection=None, timeout=120):
        if type(timeout) not in (int, float) or not math.isfinite(timeout) or not 0 < timeout <= 300:
            raise ValueError('The asking observation timeout must be finite and at most five minutes')
        self.timeout = timeout
        self.owned = bus_connection is None
        self.bus = bus_connection if bus_connection is not None else dbus.SystemBus(private=True)
        self.bus.set_exit_on_disconnect(False)
        self.context = GLib.MainContext.default()
        self.closed = False
        self.finished = False
        self.subscribed = False
        self.filter_added = False
        self.filter = self.received
        self.rules = []
        self.failure = None
        self.count = 0
        self.selected = None
        self.identity = None
        self.capturing = False
        self.properties = {UNIT: {}, SERVICE: {}}
        self.removed = False
        try:
            self.owner = str(self.bus.get_name_owner(NAME))
            self.bus.add_message_filter(self.filter)
            self.filter_added = True
            for rule in [f"type='signal',sender='{self.owner}',interface='{MANAGER}',path='{PATH}'",
                         f"type='signal',sender='{self.owner}',interface='{PROPERTIES}',path_namespace='{PATH}/unit'",
                         f"type='signal',sender='org.freedesktop.DBus',interface='org.freedesktop.DBus',member='NameOwnerChanged',arg0='{NAME}'"]:
                self.bus.add_match_string(rule)
                self.rules.append(rule)
            self.call(PATH, MANAGER, 'Subscribe')
            self.subscribed = True
            units = self.call(PATH, MANAGER, 'ListUnits')
            if any(asking(row[0]) for row in units):
                raise ValueError('An asking turn already exists at the observation baseline')
            self.pump()
            if self.selected is not None or str(self.bus.get_name_owner(NAME)) != self.owner:
                raise ValueError('The asking baseline changed while subscribing')
        except BaseException:
            self.close()
            raise

    def call(self, path, interface, member, signature='', args=()):
        return self.bus.call_blocking(self.owner, path, interface, member, signature, args,
                                      timeout=min(self.timeout, 3))

    def received(self, connection, message):
        if message.get_type() != dbus.lowlevel.MESSAGE_TYPE_SIGNAL:
            return
        if message.get_sender() not in (self.owner, 'org.freedesktop.DBus'):
            return
        self.count += 1
        if self.count > LIMIT:
            self.failure = 'The asking observation exceeded its signal bound'
        if self.failure:
            return
        try:
            interface, member = message.get_interface(), message.get_member()
            if (message.get_sender() == 'org.freedesktop.DBus' and interface == 'org.freedesktop.DBus'
                    and member == 'NameOwnerChanged'):
                values = message.get_args_list()
                if message.get_signature() != 'sss' or values[0] == NAME:
                    raise ValueError('The systemd bus owner changed')
            elif interface == MANAGER and message.get_path() == PATH and member in ('UnitNew', 'UnitRemoved'):
                if message.get_signature() != 'so':
                    raise ValueError('The systemd unit signal has an invalid signature')
                name, path = message.get_args_list()
                if asking(name):
                    if path != unit_path(name):
                        raise ValueError('The asking unit path does not match its name')
                    if member == 'UnitNew':
                        if self.selected is not None:
                            raise ValueError('More than one asking unit was announced')
                        self.selected = (str(name), str(path))
                    else:
                        if self.identity is None or self.selected != (name, path) or completed(self.identity, self.properties) is None:
                            raise ValueError('The asking unit disappeared without matching exit facts')
                        self.removed = True
            elif (interface == PROPERTIES and member == 'PropertiesChanged'
                    and self.selected is not None and message.get_path() == self.selected[1]):
                if message.get_signature() != 'sa{sv}as':
                    raise ValueError('The asking property signal has an invalid signature')
                kind, changed, invalidated = message.get_args_list()
                if kind in (UNIT, SERVICE):
                    required = ({'Id', 'ActiveState', 'SubState', 'InvocationID'} if kind == UNIT else
                                {'Type', 'User', 'Group', 'MainPID', 'ExecMainPID', 'Result',
                                 'ExecMainCode', 'ExecMainStatus', 'ExecMainStartTimestampMonotonic',
                                 'ExecMainExitTimestampMonotonic'})
                    # ExecStart also contains runtime execution-status tuples:
                    # systemd invalidates it on normal exit, before GC. The
                    # captured command belongs to the anchored PID/invocation;
                    # do not claim continuing unit configuration attestation.
                    if required.intersection(invalidated):
                        raise ValueError('The asking identity or exit properties were invalidated')
                    if self.identity is not None:
                        unchanged(self.identity, kind, changed)
                        self.properties[kind].update(changed)
        except (ValueError, TypeError, IndexError) as error:
            self.failure = str(error)

    def pump(self):
        if self.closed:
            raise ValueError('The asking observation is closed')
        for _ in range(LIMIT):
            if not self.context.pending():
                break
            self.context.iteration(False)
        if not self.bus.get_is_connected():
            raise ValueError('The asking observation lost its bus connection')
        if self.failure:
            raise ValueError(self.failure)

    def capture_pending(self):
        """Capture only after the genuine request reaches its native approval card."""
        return self.capture(min(self.timeout, 3), wait_for_start=False)

    def capture_started(self):
        """Wait for a fresh Full turn to start, before its process can disappear."""
        return self.capture(self.timeout, wait_for_start=True)

    def capture(self, budget, *, wait_for_start):
        if self.closed or self.finished or self.capturing:
            raise ValueError('The asking observation cannot capture another turn')
        self.capturing = True
        try:
            deadline = time.monotonic() + budget
            while True:
                self.pump()
                if time.monotonic() >= deadline:
                    raise ValueError('No fresh asking turn reached the observation')
                if self.selected is not None:
                    name, path = self.selected
                    for interface in (UNIT, SERVICE):
                        self.properties[interface] = dict(self.call(path, PROPERTIES, 'GetAll', 's', (interface,)))
                    if wait_for_start and starting(name, path, self.properties):
                        time.sleep(.01)
                        continue
                    self.identity = pending(name, path, self.properties)
                    self.pump()
                    return dict(self.identity)
                time.sleep(.01)
        except dbus.DBusException as error:
            raise ValueError('The pending asking identity could not be read') from error

    def finish(self):
        """Wait for this invocation's exit; do not query an already collected unit."""
        if self.closed or self.finished or self.identity is None:
            raise ValueError('The asking observation has no captured unfinished turn')
        self.finished = True
        try:
            deadline = time.monotonic() + self.timeout
            while True:
                self.pump()
                result = completed(self.identity, self.properties)
                if result is not None:
                    return result
                if time.monotonic() >= deadline:
                    raise ValueError('The asking process has no matching successful exit')
                time.sleep(.01)
        finally:
            self.close()

    def close(self):
        """Remove our subscription and handlers, leaving injected buses open."""
        if self.closed:
            return
        self.closed = True
        if self.subscribed:
            try:
                self.call(PATH, MANAGER, 'Unsubscribe')
            except dbus.DBusException:
                pass
        if self.filter_added:
            self.bus.remove_message_filter(self.filter)
        for rule in self.rules:
            try:
                self.bus.remove_match_string(rule)
            except dbus.DBusException:
                pass
        if self.owned:
            self.bus.close()
