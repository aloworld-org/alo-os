"""Bring the private reference forward through the person's real Dock control.

Activation readback is a setup condition, not proof of visible moving pixels.
The retained recording still needs independent decoding and region inspection.
"""

import json
import time

import dbus

APPLICATION = 'org.aloworld.Studio.Verification'
TITLE = 'Studio verification window'
PROPERTIES = 'org.freedesktop.DBus.Properties'
NATIVE_APPLICATION = 'org.a11y.atspi.Application'
ACCESSIBLE = 'org.a11y.atspi.Accessible'
# AT-SPI state numbers: active, showing, visible.
ACTIVE_WINDOW = {1, 25, 30}


def roots(tree):
    """Refuse an incomplete or unexpectedly large application inventory."""
    found = list(tree.roots())
    if len(found) > 64 or len(set(found)) != len(found):
        raise ValueError('the native application inventory is ambiguous or too large')
    return found


def observed_read(report, node, read):
    """Keep only the identity and kind of a failed read, never application text."""
    try:
        return read()
    except (ValueError, dbus.DBusException) as error:
        report['read_error'] = dict(node=[str(part)[:96] for part in node],
                                    kind=type(error).__name__[:64])
        cause = error if isinstance(error, dbus.DBusException) else error.__cause__
        if isinstance(cause, dbus.DBusException):
            name = cause.get_dbus_name()
            if isinstance(name, str):
                report['read_error']['error_name'] = name[:128]
        raise


def refusal(report):
    """Fit the final observation in the installed report's bounded error field."""
    while True:
        text = json.dumps(report, ensure_ascii=False, separators=(',', ':'))
        if len(text) <= 1600:
            return text
        observed = report['observed']
        if observed['roots']:
            observed['roots'].pop()
        elif observed['frames']:
            observed['frames'].pop()
        else:
            # Identities and exception classes have individual bounds. This
            # final branch also bounds unusual escaped control characters.
            report['application'] = report['application'][:32]
            report['title'] = report['title'][:32]
            if report['read_error']:
                report['read_error']['node'] = [part[:32] for part in report['read_error']['node']]
                report['read_error']['kind'] = report['read_error']['kind'][:32]
                if 'error_name' in report['read_error']:
                    report['read_error']['error_name'] = report['read_error']['error_name'][:32]


def nodes(tree, root, process, report):
    """Read a bounded tree belonging entirely to the observed native owner."""
    pending, seen, found = [root], set(), []
    while pending:
        node = pending.pop()
        if node in seen:
            continue
        seen.add(node)
        if len(seen) > 512 or tree.owner(node[0]) != process:
            raise ValueError('the reference activation tree changed owner or exceeded its limit')
        facts = observed_read(report, node, lambda: tree.facts(node))
        found.append((node, facts))
        pending.extend(facts['children'])
    return found


def foreground(tree, press, *, application=APPLICATION, title=TITLE):
    """Press the owned application's Dock entry once, then await fresh readback.

The shell's root has no display name. Its native toolkit property identifies
it; the reference's exact application identifier identifies the Dock action.
Ordinary application focus is insufficient because it deliberately does not
reorder the person's native Settings and task windows.
"""
    application_path = '/' + application.replace('.', '/') + '/a11y/'
    shells = []
    for root in roots(tree):
        try:
            toolkit = tree.interface(root, PROPERTIES).Get(
                NATIVE_APPLICATION, 'ToolkitName', timeout=3)
        except dbus.DBusException:
            continue
        if toolkit == 'alo':
            shells.append(root)
    if len(shells) != 1:
        raise ValueError('exactly one native alo Dock owner is required')
    shell = shells[0]
    owner = tree.owner(shell[0])
    if type(owner) is not int or owner <= 0:
        raise ValueError('the native Dock owner is unavailable')
    press(owner, lambda name: name == application)
    deadline = time.monotonic() + 10
    while True:
        report = dict(reason='window_activation_not_observed',
                      application=application[:128], title=title[:96], read_error=None,
                      observed=dict(roots_count=None, roots=[], frames=[], shell_active=None))
        observed = report['observed']
        try:
            current = roots(tree)
            observed['roots_count'] = len(current)
            observed['roots'] = [[str(part)[:96] for part in root] for root in current[:4]]
            if shell not in current or tree.owner(shell[0]) != owner:
                raise ValueError('the native Dock owner changed')
            # GTK's root Name is the program name, not the window title.
            # Registered GApplication children use its D-Bus object path.
            reference = [root for root in current
                         if any(child[1].startswith(application_path)
                                for child in observed_read(report, root, lambda: tree.facts(root))['children'])]
            if len(reference) != 1:
                raise ValueError('exactly one reference application is required')
            root = reference[0]
            process = tree.owner(root[0])
            if type(process) is not int or process <= 0:
                raise ValueError('the reference application owner is unavailable')
            frames = []
            for node, facts in nodes(tree, root, process, report):
                if node == root or not node[1].startswith(application_path) or facts['name'] != title:
                    continue
                role = observed_read(report, node, lambda: int(
                    tree.interface(node, ACCESSIBLE).GetRole(timeout=3)))
                if len(observed['frames']) < 4:
                    observed['frames'].append(dict(role=role, states=sorted(facts['states'])[:16]))
                if role in {23, 69}:
                    frames.append(facts)
            shell_active = any({1, 25} <= facts['states']
                               for _, facts in nodes(tree, shell, owner, report))
            observed['shell_active'] = shell_active
            if (len(frames) == 1 and ACTIVE_WINDOW <= frames[0]['states']
                    and not shell_active):
                return dict(name=frames[0]['name'], process=process)
        except (ValueError, dbus.DBusException):
            pass
        if time.monotonic() >= deadline:
            raise ValueError(refusal(report))
        time.sleep(.1)
