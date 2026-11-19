"""Observe real installed Review actions and position, without claiming pixels/audio."""

import math
import time

import dbus

from native_controls import gtk4, usable
from reference_window import roots

ACCESSIBLE = 'org.a11y.atspi.Accessible'
PROPERTIES = 'org.freedesktop.DBus.Properties'
VALUE = 'org.a11y.atspi.Value'


def position(tree, process, expected=None):
    """Read exactly one current, owned, usable GTK playback scale in seconds."""
    applications = set(roots(tree))
    pending, seen, found = list(applications), set(), []
    while pending:
        node = pending.pop()
        if node in seen:
            continue
        seen.add(node)
        if len(seen) > 512:
            raise ValueError('the playback tree exceeded its limit')
        if tree.owner(node[0]) != process:
            continue
        facts = tree.facts(node)
        if node in applications or 24 in facts['states'] or not gtk4(tree, node, process):
            pending.extend(facts['children'])
        if (facts['name'] == 'Playback position' and usable(tree, node, process, facts)
                and int(tree.interface(node, ACCESSIBLE).GetRole(timeout=3)) == 51):
            found.append(node)
    if len(found) != 1:
        raise ValueError('exactly one usable playback position is required')
    node = found[0]
    if expected is not None and node != expected:
        raise ValueError('the playback scale was replaced during observation')
    properties = tree.interface(node, PROPERTIES)
    values = [properties.Get(VALUE, name, timeout=3)
              for name in ('MinimumValue', 'MaximumValue', 'CurrentValue')]
    if not all(isinstance(value, dbus.Double) and math.isfinite(value) for value in values):
        raise ValueError('playback position must contain finite Double seconds')
    low, high, current = map(float, values)
    if low != 0 or high <= low or not low <= current <= high:
        raise ValueError('playback position is outside its recording duration')
    facts = tree.facts(node)
    if (facts['name'] != 'Playback position' or not usable(tree, node, process, facts)
            or int(tree.interface(node, ACCESSIBLE).GetRole(timeout=3)) != 51
            or tree.owner(node[0]) != process):
        raise ValueError('the playback control changed during its read')
    return node, current


def check(tree, process, press):
    """Play the selected recording, observe advancement, pause and verify stability.

    The caller foregrounds the normal app and supplies its observed AT-SPI owner.
    The result measures position only; decoded pixels and audio have separate checks.
    """
    if type(process) is not int or process <= 0:
        raise ValueError('the playback owner is unavailable')
    deadline = time.monotonic() + 10
    while True:
        try:
            node, start = position(tree, process)
            break
        except ValueError:
            if time.monotonic() >= deadline:
                raise
            time.sleep(.1)
    started = False
    try:
        press(process, lambda name: name == 'Play recording')
        started = True
        deadline = time.monotonic() + 8
        while True:
            _, playing = position(tree, process, node)
            if playing >= start + .5:
                break
            if time.monotonic() >= deadline:
                raise ValueError('playback did not advance after native Play')
            time.sleep(.1)
        press(process, lambda name: name == 'Pause playback')
        started = False
        time.sleep(.15)
        _, paused = position(tree, process, node)
        deadline = time.monotonic() + 1
        while True:
            _, stable = position(tree, process, node)
            if abs(stable - paused) > .08:
                raise ValueError('playback position continued advancing after Pause')
            if time.monotonic() >= deadline:
                break
            time.sleep(.1)
        return dict(scope='review-playback-position', process=process, start=start,
                    playing=playing, paused=paused, stable=stable)
    finally:
        if started:
            try:
                press(process, lambda name: name == 'Pause playback')
            except (ValueError, dbus.DBusException):
                # Preserve the original failure; never target a replacement owner.
                pass
