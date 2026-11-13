# `orca --version` answering is not Orca starting

**Concluded:** *Orca starts, at or above 48* — the line the image recipe printed
on every build from the day the screen reader landed (ADR 0151, pull request
#1204), and which the recipe's own comment calls *each claim is checked by the
thing itself, not by a file being there.*

**What was true.** On the first booted machine that turned the switch on (qemu
walk, image from `main` at `ddfe125f`, 2026-11-13), Orca fell over during
startup, every time, and its unit started it again:

```text
File "/usr/lib/python3.13/site-packages/orca/orca.py", line 111, in loadUserSettings
  orca_modifier_manager.get_manager().refresh_orca_modifiers("Loading user settings.")
File ".../orca/orca_modifier_manager.py", line 233, in refresh_orca_modifiers
  with subprocess.Popen(["xkbcomp", os.environ["DISPLAY"], "-"],
KeyError: 'DISPLAY'
```

Reproduced in `localhost/alo-os:dev` booted as a container with systemd, a
person's own manager, and a headless Wayland compositor: the image's own unit,
started as Settings starts it, fell over five times in forty seconds with that
traceback. And a reader given the `DISPLAY` it asks for falls over one step
later, in a different engine (`docs/quirks/orca-on-wayland-needs-an-x-display-twice.md`).

## The mechanism

**A program that answers a question about itself has not been asked to do its
job.** `orca --version` parses its arguments, prints a string and exits. It
never asks for a display, never reads settings, never opens the keyboard — the
three things it falls over on. It does prove what it was written to prove, that
Orca's Python and its modules import: the same check caught a missing
`python3-dbus` on 2026-11-12. That success is what made it believable. It had
caught a real fault once, so it read as *the reader starts*, when what it
measured was *the reader loads*.

This is the first entry in `CLAUDE.md`'s list in a new costume: `test -x` on a
converter stood in for the converter starting. Here a version string stood in
for a session program starting **in a session**, which needs the session — a
compositor, a session bus, the accessibility bus — and the build had none of
them, so the check was chosen from what could run there rather than from what
the reader does.

**And the desktop's own watch had the same shape, one level up.** It looked at
the unit for two seconds and called a reader that was still running then
*started*. On the walk machine Orca ran for more than a minute before it fell
over. Two seconds was *longer than Orca took to fall over when it could not
start* — one fault, measured once in the pinned base — written into the desktop
as a property of every fault on every machine.

## The cure

**Start a session program the way the session starts it, and hold that it
stays up.** For the reader: the unit's own `ExecStart` and `Environment`, with
a Wayland compositor, a session bus and the accessibility bus, for long enough
that the startup path has run — and fail the build if it is not still there.
That check was built and measured on 2026-11-13 — it failed on the image as it
was, with Orca's own `KeyError`, and passed on a reader that could stay up — and
it lands with the move to Fedora 43, where the reader can pass it (the owner's
decision of that day, so images keep building until then).

**And a watch on something that can fall over later watches for as long as it
matters**, not for a guess at how long falling over takes:
`crates/alo-desktop/src/watching_the_screen_reader.rs` looks for as long as the
switch is on.

**Which of the two caught it:** neither. A person turning the switch on, on a
booted machine, did.
