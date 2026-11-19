"""Strict process-owned native UI selection for the private installed walk."""

import re

import dbus


def gtk4(tree, node, process):
    """Identify GTK4 through the same owner's real application root."""
    interface = getattr(tree, 'interface', None)
    root = (node[0], '/org/a11y/atspi/accessible/root')
    if interface is None or root not in tree.roots() or tree.owner(root[0]) != process:
        return False
    try:
        properties = interface(root, 'org.freedesktop.DBus.Properties')
        toolkit = properties.Get('org.a11y.atspi.Application', 'ToolkitName', timeout=3)
        version = properties.Get('org.a11y.atspi.Application', 'Version', timeout=3)
    except dbus.DBusException:
        return False
    return (isinstance(toolkit, str) and toolkit == 'GTK'
            and isinstance(version, str) and re.fullmatch(r'4\.[0-9]+\.[0-9]+', version) is not None
            and tree.owner(root[0]) == process)


def usable(tree, node, process, facts):
    """Accept GTK4's SENSITIVE flag without inventing an ENABLED state."""
    return ({8, 24, 25} <= facts['states']
            or ({24, 25, 30} <= facts['states'] and gtk4(tree, node, process)))


def activate(tree, process, matches):
    """Activate exactly one usable named control, refusing ambiguous trees.

    tree is a real AT-SPI connection in the installed walk. Unit fixtures supply
    observed node facts through the same small interface. No coordinates,
    keyboard shortcuts or direct portal consent replies are manufactured.
    """
    if type(process) is not int or process <= 0:
        raise ValueError('the native dialog owner is unavailable')
    applications = set(tree.roots())
    pending = list(applications)
    seen = set()
    candidates = []
    while pending:
        node = pending.pop()
        if node in seen:
            continue
        seen.add(node)
        if len(seen) > 512:
            raise ValueError('the native dialog tree exceeded the walk limit')
        if tree.owner(node[0]) != process:
            continue
        facts = tree.facts(node)
        # A GTK4 disabled parent's child label may report SENSITIVE yet refuse
        # GetName for its advertised actions. The parent's effective state
        # excludes that entire subtree. Application roots have no such state.
        if node in applications or 24 in facts['states'] or not gtk4(tree, node, process):
            pending.extend(facts['children'])
        if not matches(facts['name']) or not usable(tree, node, process, facts):
            continue
        actions = [index for index, name in enumerate(facts['actions'])
                   if name.casefold() in {'click', 'press', 'activate', 'toggle', 'select'}]
        if actions:
            candidates.append((node, actions[0]))
    if len(candidates) != 1:
        raise ValueError(f'expected one native control, found {len(candidates)}')
    node, index = candidates[0]
    if tree.owner(node[0]) != process:
        raise ValueError('the native dialog owner changed')
    current = tree.facts(node)
    if (not matches(current['name']) or not usable(tree, node, process, current)
            or index >= len(current['actions'])
            or current['actions'][index].casefold() not in {'click', 'press', 'activate', 'toggle', 'select'}):
        raise ValueError('the native control changed before activation')
    if tree.owner(node[0]) != process:
        raise ValueError('the native dialog owner changed during its final read')
    if tree.action(node, index) is not True:
        raise ValueError('the native control refused activation')
