"""Validate systemd's typed identity and exit properties for one asking turn."""

import dbus

UNIT = 'org.freedesktop.systemd1.Unit'
SERVICE = 'org.freedesktop.systemd1.Service'


def unit_path(name):
    return '/org/freedesktop/systemd1/unit/' + ''.join(
        char if char.isascii() and char.isalnum() else '_%02x' % ord(char)
        for char in name)


def asking(name):
    return (isinstance(name, str) and name.startswith('alo-asking@')
            and name.endswith('.service') and len(name) > len('alo-asking@.service'))


def unsigned(value, kind, *, positive=False):
    if type(value) is not kind or (positive and value == 0):
        raise ValueError('The asking process has an invalid typed identity')
    return int(value)


def invocation(value):
    if (not isinstance(value, dbus.Array) or value.signature != 'y' or len(value) != 16
            or any(type(byte) is not dbus.Byte for byte in value) or not any(value)):
        raise ValueError('The asking process has no valid invocation identity')
    return bytes(value).hex()


def command(value):
    if (not isinstance(value, dbus.Array) or value.signature != '(sasbttttuii)'
            or len(value) != 1 or len(value[0]) != 10
            or value[0][0] != '/usr/bin/alo-agent'
            or list(value[0][1]) != ['/usr/bin/alo-agent']):
        raise ValueError('The asking process is not the fixed alo agent command')


def starting(name, path, properties):
    """Only a typed, correctly configured activating service may keep waiting."""
    unit, service = properties[UNIT], properties[SERVICE]
    if unit.get('ActiveState') != 'activating' or unit.get('SubState') != 'start':
        return False
    try:
        if (unit['Id'] != name or path != unit_path(name) or not asking(name)
                or service['Type'] != 'exec' or service['User'] != 'alo-agent'
                or service['Group'] != 'alo-agent'):
            raise ValueError('The starting asking unit has the wrong identity')
        command(service['ExecStart'])
        invocation(unit['InvocationID'])
        unsigned(service['MainPID'], dbus.UInt32)
        unsigned(service['ExecMainPID'], dbus.UInt32)
        unsigned(service['ExecMainStartTimestampMonotonic'], dbus.UInt64)
        return True
    except (KeyError, TypeError, IndexError) as error:
        raise ValueError('The starting asking identity is incomplete') from error


def pending(name, path, properties):
    unit, service = properties[UNIT], properties[SERVICE]
    try:
        if (not asking(name) or path != unit_path(name) or unit['Id'] != name
                or unit['ActiveState'] != 'active' or unit['SubState'] != 'running'
                or service['Type'] != 'exec' or service['User'] != 'alo-agent'
                or service['Group'] != 'alo-agent'):
            raise ValueError('The asking unit is not a pending alo agent turn')
        command(service['ExecStart'])
        pid = unsigned(service['MainPID'], dbus.UInt32, positive=True)
        if unsigned(service['ExecMainPID'], dbus.UInt32) != pid:
            raise ValueError('The asking process identity does not agree')
        return dict(unit=str(name), path=str(path), invocation_id=invocation(unit['InvocationID']),
                    pid=pid, start_us=unsigned(service['ExecMainStartTimestampMonotonic'],
                                              dbus.UInt64, positive=True))
    except (KeyError, TypeError, IndexError) as error:
        raise ValueError('The asking process identity is incomplete') from error


def unchanged(identity, interface, changed):
    """Identity substitutions remain failures even if a later signal restores them."""
    for key, value in changed.items():
        if interface == UNIT:
            if key == 'Id' and value != identity['unit']:
                raise ValueError('The asking unit identity changed')
            if key == 'InvocationID' and invocation(value) != identity['invocation_id']:
                raise ValueError('The asking invocation changed')
        if interface == SERVICE:
            if key in ('User', 'Group', 'Type') and value != {
                    'User': 'alo-agent', 'Group': 'alo-agent', 'Type': 'exec'}[key]:
                raise ValueError('The asking service identity changed')
            if key == 'ExecStart':
                command(value)
            if key == 'MainPID' and unsigned(value, dbus.UInt32) not in (0, identity['pid']):
                raise ValueError('The asking process was replaced')
            if key == 'ExecMainPID' and unsigned(value, dbus.UInt32) != identity['pid']:
                raise ValueError('The executed asking process changed')
            if (key == 'ExecMainStartTimestampMonotonic'
                    and unsigned(value, dbus.UInt64) != identity['start_us']):
                raise ValueError('The asking process start changed')


def completed(identity, properties):
    """Inactive alone is insufficient: require a matching successful child exit."""
    unit, service = properties[UNIT], properties[SERVICE]
    if unit.get('ActiveState') != 'inactive' or unit.get('SubState') != 'dead':
        return None
    try:
        if (unsigned(service['MainPID'], dbus.UInt32) != 0
                or unsigned(service['ExecMainPID'], dbus.UInt32) != identity['pid']
                or unsigned(service['ExecMainStartTimestampMonotonic'], dbus.UInt64) != identity['start_us']
                or service['Result'] != 'success'
                or type(service['ExecMainCode']) is not dbus.Int32 or service['ExecMainCode'] != 1
                or type(service['ExecMainStatus']) is not dbus.Int32 or service['ExecMainStatus'] != 0):
            return None
        exit_us = unsigned(service['ExecMainExitTimestampMonotonic'], dbus.UInt64, positive=True)
        if exit_us < identity['start_us']:
            return None
        return dict(identity, result='success', exit_code=1, exit_status=0, exit_us=exit_us)
    except (KeyError, ValueError):
        return None
