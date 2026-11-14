# VSCodium started with no display asks for X11, and Chromium stops itself with a trap

- **Version:** VSCodium 1.143.17232 from Flathub (`com.vscodium.codium`, commit
  `b63788f2…`, runtime `org.freedesktop.Sdk` 26.08), with the Flathub
  `ide-flatpak-wrapper` launcher it ships.
- **Whose:** the launcher's choice is Flathub's, and the trap is Chromium's, so
  every Electron application launched the same way has it. alo OS configures
  them and does not patch either.
- **Date:** 2026-11-14, from the boot walk of that day.

## Behaviour

On the walk, `flatpak run com.vscodium.codium hello.txt &` typed into **alo's
own terminal** printed the launcher's *Setting up NPM/Cargo/Python packages…
'ruby' is not available*, and then the journal said
`systemd-coredump: Process … (com.vscodium.co) of user 1000 terminated
abnormally with signal 5/TRAP`. No window appeared. The same application
started from the Dock opened its window.

The 2026-11-12 walk's serial log has the same thing, unnoticed:
`traps: com.vscodium.co[9135] trap int3 … in com.vscodium.codium[4d2addd,…+9c8a000]`
after two starts with no `alo-desktop … asked to start` line before them, while
the start from the Dock earlier that day printed the launcher's Wayland
warnings and stayed up.

## Why

1. **alo's terminal handed its shell no display.** The terminal runs inside the
   desktop process (`alo-shell`, ADR 0114), and alacritty's engine starts the
   shell with that process's environment plus what it is given
   (`alacritty_terminal` 0.26.0, `tty/unix.rs`, lines 230–241). The desktop's
   unit sets `XDG_SESSION_TYPE=wayland` and no `WAYLAND_DISPLAY`
   (`image/usr/lib/systemd/system/alo-desktop@.service`): the desktop *is* the
   display, and it tells the person's systemd manager and the session bus where
   it draws (`alo-desktop`'s `telling_the_session_where_alo_draws`), not
   itself. The terminal added only `TERM`.
2. **VSCodium's launcher then chooses X11.** `/app/bin/com.vscodium.codium`
   passes `--ozone-platform=wayland` only when `XDG_SESSION_TYPE` is `wayland`
   **and** `WAYLAND_DISPLAY` names a socket that exists; otherwise it passes
   `--ozone-platform=x11`.
3. **alo has no X server, and Chromium traps.** It prints
   `ozone_platform_x11.cc:257 Missing X server or $DISPLAY` and
   `aura/env.cc:246 The platform failed to initialize. Exiting.`, then stops on
   `int3` — signal 5, at the same offset in the binary (`4d2addd`) as both
   walks.

**An `ELECTRON_OZONE_PLATFORM_HINT` does not help.** The launcher's explicit
`--ozone-platform=x11` wins over the hint; measured below.

## What was measured

**By hand on 2026-11-14, not on an alo machine**: Ubuntu 24.04 in WSL, headless
Weston, its socket moved to an absolute path as alo's is, VSCodium run as an
ordinary user (uid 1000) with `--verbose` so Electron's own lines reach the
log, and `--no-sandbox` in every run — this setup's Flatpak portal cannot find
the requesting process for zypak's sandbox (*Could not find requesting pid*),
which is the rig's fault and the same for every row. Each row was run twice
with the same result, except the hint row, run once.

| Started with | Launcher chose | Result |
|---|---|---|
| `XDG_SESSION_TYPE=wayland`, no `WAYLAND_DISPLAY` — what alo's terminal gave | `--ozone-platform=x11` | *Missing X server or $DISPLAY*, trap at `4d2addd` |
| the same and `ELECTRON_OZONE_PLATFORM_HINT=wayland` | `--ozone-platform=x11` | the same trap |
| `XDG_SESSION_TYPE=wayland`, `WAYLAND_DISPLAY=<absolute path>` — what the desktop hands a start | `--ozone-platform=wayland` | window loaded, still running at 50 s |

The first attempt ran as root and also died with signal 5, for a different
reason: *Running as root without --no-sandbox is not supported*. The signal
alone does not identify the cause; the line before it does.

## What alo OS does about it

alo's terminal now tells its shell what the desktop tells the session —
`WAYLAND_DISPLAY` as the desktop's own socket path, and
`XDG_SESSION_TYPE=wayland` (`alo_terminal::AShellOnAPseudoterminal::the_persons_own`'s
`session`, filled by `alo-shell` from `alo_launching::what_the_session_is_told`).
`alo-shell`'s `the_terminal_the_chord_opens_tells_its_shell_where_alo_draws`
holds it on every build, by reading the shell's environment from the kernel.

**Not yet seen on a booted alo machine.** The next walk checks it with
`echo $WAYLAND_DISPLAY` in alo's terminal, then the same `flatpak run`.

**And then the whole session, not only the display.** The display alone left
the shell without the person's `XDG_DATA_DIRS` with Flatpak's exports (Flatpak
warned about it in the terminal) and without the *applications describe
themselves* variables from
`/usr/lib/environment.d/60-alo-apps-describe-themselves.conf`, which only the
person's systemd manager reads — so a Chromium application started from the
terminal was not described to alo or a screen reader the way one started from
the Dock is. It also left the desktop's own variables in the shell. The
terminal's shell now starts with **exactly** the environment the person's
systemd manager holds, read when the terminal opens
(`alo-shell`'s `the_persons_session`), with the display laid over it and
`TERM`; the desktop's own names are removed by starting the shell through
`/usr/bin/env -u NAME … -- <their shell>`
(`alo-terminal`'s `what_a_shell_starts_with`). Where the manager cannot be
asked, the shell gets the desktop's environment and the display, as above, and
the desktop's log says so. `alo-terminal`'s
`a_real_shell_starts_with_exactly_the_persons_session` holds the exact
environment on every build, read from the kernel. Reading the manager itself
needs a person's session bus, which no test runner has: **that half is checked
only on a booted machine**, with `env` typed into alo's terminal.
