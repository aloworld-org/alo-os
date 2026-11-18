"""Person-side observation of Studio method delivery on a private test session.

The caller must first establish that the real agent turn ended. Only then may
finish() send its read barrier and certify the calls observed before that reply.
This module never approves a change or sends a changing application method.
"""

import math
import time

import dbus
from dbus.mainloop.glib import DBusGMainLoop
from gi.repository import GLib

NAME = 'org.aloworld.Studio'
PATH = '/org/aloworld/Studio/Recorder'
INTERFACE = 'org.aloworld.Studio.Recorder1'
LIMIT = 512


class DeliveryObservation:
    """Keep bounded call headers, refusing owner changes and incomplete evidence.

Connections are private to this observer. pump() services their GLib callbacks;
the controller calls it while waiting for native input and agent completion.
No message bodies or unrelated traffic are retained in the returned summary.
"""

    def __init__(self, *, timeout=3):
        if type(timeout) not in (int, float) or not math.isfinite(timeout) or not 0 < timeout <= 10:
            raise ValueError('The delivery barrier timeout must be finite and at most ten seconds')
        self.timeout = timeout
        self.client = None
        self.monitor = None
        self.closed = False
        self.finished = False
        self.failure = None
        self.messages = 0
        self.changes = []
        self.change_keys = set()
        self.successful_changes = set()
        self.barrier = None
        self.barrier_call = False
        self.barrier_reply = False
        self.actual_reply = False
        self.context = GLib.MainContext.default()
        try:
            loop = DBusGMainLoop()
            self.client = dbus.SessionBus(private=True, mainloop=loop)
            self.client.set_exit_on_disconnect(False)
            self.owner = self.current_owner()
            self.monitor = dbus.SessionBus(private=True, mainloop=loop)
            self.monitor.set_exit_on_disconnect(False)
            self.monitor.add_message_filter(self.received)
            rules = [f"type='method_call',interface='{INTERFACE}',path='{PATH}'",
                     f"type='method_return',sender='{self.owner}'",
                     f"type='error',sender='{self.owner}'",
                     f"type='signal',interface='org.freedesktop.DBus',member='NameOwnerChanged',arg0='{NAME}'"]
            self.monitor.call_blocking('org.freedesktop.DBus', '/org/freedesktop/DBus',
                'org.freedesktop.DBus.Monitoring', 'BecomeMonitor', 'asu', (rules, 0), timeout=self.timeout)
            if self.current_owner() != self.owner:
                raise ValueError('Studio changed owner before observation began')
        except BaseException:
            self.close()
            raise

    def current_owner(self):
        return str(self.client.call_blocking('org.freedesktop.DBus', '/org/freedesktop/DBus',
            'org.freedesktop.DBus', 'GetNameOwner', 's', (NAME,), timeout=self.timeout))

    def received(self, connection, message):
        self.messages += 1
        if self.messages > LIMIT:
            self.failure = 'The delivery observation exceeded its message bound'
        if self.failure is not None:
            return
        kind = message.get_type()
        if (kind == dbus.lowlevel.MESSAGE_TYPE_SIGNAL
                and message.get_sender() == 'org.freedesktop.DBus'
                and message.get_interface() == 'org.freedesktop.DBus'
                and message.get_member() == 'NameOwnerChanged'):
            values = message.get_args_list()
            if len(values) != 3 or values[0] == NAME:
                self.failure = 'Studio changed owner during observation'
        elif (kind == dbus.lowlevel.MESSAGE_TYPE_METHOD_CALL
                and message.get_destination() in (NAME, self.owner)
                and message.get_path() == PATH and message.get_interface() == INTERFACE):
            member = message.get_member()
            if member == 'GetStatus' and message.get_signature() == '':
                if (message.get_sender() == self.client.get_unique_name()
                        and message.get_serial() == self.barrier):
                    self.barrier_call = True
            else:
                self.changes.append((member, str(message.get_signature())))
                self.change_keys.add((message.get_sender(), message.get_serial()))
                if message.get_no_reply():
                    self.failure = 'The Studio change requested no execution reply'
        elif (kind in (dbus.lowlevel.MESSAGE_TYPE_METHOD_RETURN, dbus.lowlevel.MESSAGE_TYPE_ERROR)
                and message.get_sender() == self.owner):
            key = (message.get_destination(), message.get_reply_serial())
            if key in self.change_keys:
                if (kind != dbus.lowlevel.MESSAGE_TYPE_METHOD_RETURN or message.get_signature() != ''
                        or key in self.successful_changes):
                    self.failure = 'The Studio change has no unique successful execution reply'
                else:
                    self.successful_changes.add(key)
            elif key == (self.client.get_unique_name(), self.barrier):
                if kind != dbus.lowlevel.MESSAGE_TYPE_METHOD_RETURN or message.get_signature() != 's':
                    self.failure = 'Studio did not answer the delivery observation barrier'
                else:
                    self.barrier_reply = True

    def pump(self):
        """Process ready observations without waiting for an unrelated event."""
        if self.closed:
            raise ValueError('The delivery observation is closed')
        for _ in range(LIMIT):
            if not self.context.pending():
                break
            self.context.iteration(False)
        if not self.monitor.get_is_connected() or not self.client.get_is_connected():
            raise ValueError('The delivery observation lost its bus connection')
        if self.failure is not None:
            raise ValueError(self.failure)

    def finish(self, *, enabled):
        """After actual turn completion, require one call and its observed barrier."""
        if self.closed or self.finished or type(enabled) is not bool:
            raise ValueError('The delivery observation cannot certify this action')
        self.finished = True
        try:
            self.pump()
            if self.current_owner() != self.owner:
                raise ValueError('Studio changed owner before the read barrier')
            message = dbus.lowlevel.MethodCallMessage(NAME, PATH, INTERFACE, 'GetStatus')

            def replied(reply):
                if (reply.get_type() == dbus.lowlevel.MESSAGE_TYPE_METHOD_RETURN
                        and reply.get_sender() == self.owner and reply.get_signature() == 's'):
                    self.actual_reply = True
                else:
                    self.failure = 'The delivery observation read barrier failed'

            self.client.send_message_with_reply(message, replied, timeout_s=self.timeout)
            self.barrier = message.get_serial()
            deadline = time.monotonic() + self.timeout
            while True:
                self.pump()
                if self.actual_reply and self.barrier_call and self.barrier_reply:
                    break
                if time.monotonic() >= deadline:
                    raise ValueError('The delivery observation read barrier timed out')
                time.sleep(.01)
            method = 'EnableMicrophone' if enabled else 'DisableMicrophone'
            if (self.changes != [(method, '')] or len(self.successful_changes) != 1
                    or self.change_keys != self.successful_changes):
                raise ValueError('Expected exactly one delivery of the requested Studio change')
            return dict(method=method, deliveries=1, barrier_observed=True, owner=self.owner)
        except dbus.DBusException as error:
            raise ValueError('The delivery observation lost its D-Bus evidence') from error
        finally:
            self.close()

    def close(self):
        """Close only our own observer and read-barrier connections."""
        if not self.closed:
            self.closed = True
            for connection in (self.monitor, self.client):
                if connection is not None:
                    connection.close()
