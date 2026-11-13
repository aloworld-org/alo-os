# Orca 48.9 on Fedora 42 cannot start without an X display, twice over

- **Version:** Orca 48.9 (`orca-48.9-2.fc42`) and at-spi2-core 2.56.8
  (`at-spi2-core-2.56.8-1.fc42`), the pinned Fedora 42 base. Both are the newest
  the base's repositories carry (`dnf list --showduplicates orca at-spi2-core`,
  2026-11-13).
- **Whose:** the engines'. alo OS configures them and patches neither.
- **Date:** found 2026-11-13 on a qemu walk of an image from `main` at
  `ddfe125f`; reproduced and taken apart the same day in `localhost/alo-os:dev`
  booted as a container with systemd, a person's own manager, and a headless
  Weston as the Wayland compositor. **Measured once, by hand**; what runs again
  is the image recipe's start check (below).

## Behaviour

alo's session is Wayland only: no X server, no Xwayland, no `DISPLAY`. Orca
falls over on that at two separate places, one after the other.

**1. Orca reads `DISPLAY` unguarded.** `orca.py` line 111, `loadUserSettings`,
calls `refresh_orca_modifiers`, which runs `xkbcomp` against
`os.environ["DISPLAY"]`:

```text
File ".../orca/orca_modifier_manager.py", line 233, in refresh_orca_modifiers
  with subprocess.Popen(["xkbcomp", os.environ["DISPLAY"], "-"],
KeyError: 'DISPLAY'
```

No setting and no environment variable skips it. Given a `DISPLAY` with no X
server behind it, Orca runs `xkbcomp`, which this image does not carry
(`FileNotFoundError: 'xkbcomp'`). With `xkbcomp` installed, it fails quietly
(*Cannot open display ":9"*, nothing on standard output) and Orca goes on.

**2. libatspi's keyboard opens an X display unguarded.** Orca then calls
`Atspi.Device.new_full("org.gnome.Orca")`. With no compositor offering
`org.freedesktop.a11y.Manager`, libatspi makes its *legacy* device, whose
initialiser asks X for the modifier masks without checking the display opened:

```text
Thread 1 "python3" received signal SIGSEGV, Segmentation fault.
#0  XkbKeysymToModifiers () from /lib64/libX11.so.6
#1  atspi_device_legacy_init () from /lib64/libatspi.so.0
```

The same segfault with and without `DISPLAY`, and with and without
`WAYLAND_DISPLAY` — measured with a five-line Python program that only makes
the device. libatspi's switches are `ATSPI_USE_LEGACY_DEVICE` (the same device)
and `ATSPI_USE_A11Y_MANAGER_DEVICE`, which needs the compositor to offer the
interface and otherwise falls back to the legacy device.

**What the unit then does:** `Restart=on-failure` starts Orca again two seconds
later, for ever, because the starts come more slowly than systemd's default
start limit (five in ten seconds) stops. Five falls in forty seconds in the
container; on the walk machine,
under qemu's software CPU, a fall about every ninety seconds — **active for a
minute and more before each**, which is why a two-second watch called it
started.

## What was measured against it

| configuration | Orca after 40–60 s |
|---|---|
| as shipped | falls over: `KeyError: 'DISPLAY'` (5 times in 40 s) |
| `DISPLAY=:9`, no `xkbcomp` | falls over: `FileNotFoundError: 'xkbcomp'` |
| `DISPLAY=:9`, `xkbcomp` installed | falls over: segfault in `atspi_device_legacy_init` |
| `DISPLAY=:37` with an Xvfb server on it, `xkbcomp` installed | **stays up**, 60 s, no restart |
| Fedora 43's libatspi 2.58.9 alone, no X at all | the device is made, and the key watcher added |

And **Fedora 43 carries both fixes**: Orca 49.7 reads
`os.environ.get("DISPLAY")` and skips `xkbcomp` when there is none
(`orca_modifier_manager.py` lines 232–234, *DISPLAY not set, skipping xkbcomp
operations*), and libatspi 2.58.9 makes the legacy device with no X display.
Orca 49.7 needs Python 3.14, which is Fedora 43's.

## What alo OS does about it

**Nothing that makes the reader start yet — that is a decision, and it is not
this file's.** There is no configuration of the pinned base that starts Orca
without an X server. The roads, each the owner's:

- **Move the pinned base to Fedora 43**, where both engines are fixed upstream.
- **Give the reader a private X server** (Xvfb on a display only its unit
  names, plus `xkbcomp`): configuration only, measured to stay up, and an X
  server on a machine whose session is Wayland only.
- **Offer `org.freedesktop.a11y.Manager` from alo's compositor** and set
  `ATSPI_USE_A11Y_MANAGER_DEVICE`: alo's own code, the Wayland way for a reader
  to hear the keyboard (ADR 0151 already names that the compositor does not pass
  keys yet) — and it fixes only the second fall, not the first.
- A source patch to either engine, which needs an ADR first.

What **is** done: the desktop says *did not start* under the switch whenever the
reader falls over, not only in its first two seconds
(`crates/alo-desktop/src/watching_the_screen_reader.rs`); and the image recipe
starts Orca the way the session starts it and fails the build when it does not
stay up (`image/Containerfile`, held by
`crates/alo-image/tests/the_screen_reader_speaks_on_this_machine.rs`). **So
until one of the roads above is taken, an image built from this recipe stops at
that check** — which is the truth about the reader, said at the one place that
cannot be skipped.
