"""Read real native controls through the private session's accessibility bus."""

import dbus

ACCESSIBLE = 'org.a11y.atspi.Accessible'
ACTION = 'org.a11y.atspi.Action'
ROOT = '/org/a11y/atspi/accessible/root'


class AtspiTree:
    """Small synchronous transport; activation policy lives in native_controls."""

    def __init__(self):
        self.session = dbus.SessionBus()
        address = dbus.Interface(
            self.session.get_object('org.a11y.Bus', '/org/a11y/bus'),
            'org.a11y.Bus').GetAddress(timeout=3)
        self.bus = dbus.bus.BusConnection(str(address))
        self.driver = dbus.Interface(
            self.bus.get_object('org.freedesktop.DBus', '/org/freedesktop/DBus'),
            'org.freedesktop.DBus')

    def owner(self, name):
        try:
            return int(self.driver.GetConnectionUnixProcessID(name, timeout=3))
        except dbus.DBusException:
            return None

    def interface(self, node, name):
        return dbus.Interface(self.bus.get_object(node[0], node[1], introspect=False), name)

    def children(self, node):
        return [(str(bus), str(path)) for bus, path in
                self.interface(node, ACCESSIBLE).GetChildren(timeout=3)]

    def roots(self):
        return self.children(('org.a11y.atspi.Registry', ROOT))

    def facts(self, node):
        try:
            name = self.interface(node, 'org.freedesktop.DBus.Properties').Get(
                ACCESSIBLE, 'Name', timeout=3)
            words = self.interface(node, ACCESSIBLE).GetState(timeout=3)
            states = {index * 32 + bit for index, word in enumerate(words)
                      for bit in range(32) if int(word) & (1 << bit)}
            interfaces = self.interface(node, ACCESSIBLE).GetInterfaces(timeout=3)
            actions = []
            if ACTION in interfaces:
                actions = [str(action[0]) for action in
                           self.interface(node, ACTION).GetActions(timeout=3)]
            return {'name': str(name), 'states': states, 'actions': actions,
                    'children': self.children(node)}
        except dbus.DBusException as error:
            raise ValueError('the native control could not be read') from error

    def action(self, node, index):
        try:
            return bool(self.interface(node, ACTION).DoAction(index, timeout=3))
        except dbus.DBusException as error:
            raise ValueError('the native control could not be activated') from error
