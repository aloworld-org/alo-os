"""Bring the private reference forward through the person's real Dock control.

Activation readback is a setup condition, not proof of visible moving pixels.
The retained recording still needs independent decoding and region inspection.
"""

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


def nodes(tree, root, process):
    """Read a bounded tree belonging entirely to the observed native owner."""
    pending, seen, found = [root], set(), []
    while pending:
        node = pending.pop()
        if node in seen:
            continue
        seen.add(node)
        if len(seen) > 512 or tree.owner(node[0]) != process:
            raise ValueError('the reference activation tree changed owner or exceeded its limit')
        facts = tree.facts(node)
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
        try:
            current = roots(tree)
            if shell not in current or tree.owner(shell[0]) != owner:
                raise ValueError('the native Dock owner changed')
            # GTK's root Name is the program name, not the window title.
            # Registered GApplication children use its D-Bus object path.
            reference = [root for root in current
                         if any(child[1].startswith(application_path)
                                for child in tree.facts(root)['children'])]
            if len(reference) != 1:
                raise ValueError('exactly one reference application is required')
            root = reference[0]
            process = tree.owner(root[0])
            if type(process) is not int or process <= 0:
                raise ValueError('the reference application owner is unavailable')
            frames = [facts for node, facts in nodes(tree, root, process)
                      if node != root and node[1].startswith(application_path)
                      and facts['name'] == title
                      and int(tree.interface(node, ACCESSIBLE).GetRole(timeout=3)) in {23, 69}]
            shell_active = any({1, 25} <= facts['states']
                               for _, facts in nodes(tree, shell, owner))
            if (len(frames) == 1 and ACTIVE_WINDOW <= frames[0]['states']
                    and not shell_active):
                return dict(name=frames[0]['name'], process=process)
        except (ValueError, dbus.DBusException):
            pass
        if time.monotonic() >= deadline:
            raise ValueError('the reference window activation was not observed')
        time.sleep(.1)
