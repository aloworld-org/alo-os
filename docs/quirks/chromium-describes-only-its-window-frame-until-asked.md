# Chromium describes only its window frame until it is asked for more

- **Version:** VSCodium 1.143.17232 from Flathub (`com.vscodium.codium`, runtime
  `org.freedesktop.Sdk` 26.08), measured beside Firefox 160 and KCalc 26.08
  (Qt 6.11) from Flathub, and GTK 3.24 and GTK 4 windows of our own.
- **Whose:** Chromium's, so every Electron application has it. alo OS configures
  it and does not patch it.
- **Date:** 2026-11-11, after the boot walk of that day, for the owner's
  decision that applications describe themselves from the start
  (`docs/features.md`, Language and access).

## Behaviour

On the boot walk, VSCodium was started with `ACCESSIBILITY_ENABLED=1`, and alo
then saw its window and nothing inside it: *This window shows no control that
can be named*. Chromium has two separate switches:

- **the window frame** is put on the accessibility bus when `ACCESSIBILITY_ENABLED=1`
  (or `GNOME_ACCESSIBILITY`, `QT_ACCESSIBILITY`, or `org.a11y.Status IsEnabled`)
  is set at start — read from `ui/accessibility/platform/atk_util_auralinux.cc`,
  `ShouldEnableAccessibility`;
- **the page inside it** — for VSCodium, the editor, the side bar, every
  button — only when started with `--force-renderer-accessibility`, or once
  something on the bus asks a node for its attributes or relations
  (`AtkGetAttributes`, `AtkRefRelationSet`), or when a process named `orca` is
  running.

The bare `--force-renderer-accessibility` also tells the application **a screen
reader is running**. VSCodium answers that, with its default
`editor.accessibilitySupport: auto`, by switching to *screen reader optimized*
mode, which changes how its editor behaves (`src/vs/platform/accessibility`).
`--force-renderer-accessibility=basic` gives the page without that claim.

## What was measured

**Once, by hand, on 2026-11-11, in Ubuntu 24.04 (at-spi2-core 2.52) under a
headless Weston, on a real accessibility bus — not on an alo machine.** Each
application was started under the variables named, then its tree walked by
name, role and children (`walk.py`, kept with the lane's scratch files, not in
this repository). One run per row.

| Started with | On the bus | Things in its tree | Named | *Screen Reader* shown |
|---|---|---|---|---|
| GTK 3, nothing | yes | 10 | 5 | — |
| GTK 3, `NO_AT_BRIDGE=1` | **no** | — | — | — |
| GTK 4, nothing | yes | 12 | 6 | — |
| GTK 4, `GTK_A11Y=none` | **no** | — | — | — |
| KCalc (Qt 6, Flatpak), nothing | yes | 237 | 122 | — |
| Firefox (Flatpak), nothing | **no** | — | — | — |
| Firefox, `GNOME_ACCESSIBILITY=1` | yes | 51 | 14 | — |
| Firefox, `GNOME_ACCESSIBILITY=0` | **no** | — | — | — |
| VSCodium, nothing | **no** | — | — | — |
| VSCodium, `ACCESSIBILITY_ENABLED=1` | yes, the frame only | 18 | 6 | no |
| … then each window asked for its attributes, and walked again | yes | 235 | 116 | no |
| VSCodium, `ACCESSIBILITY_ENABLED=1` and the bare switch | yes | 245 | 123 | **yes** |
| VSCodium, `ACCESSIBILITY_ENABLED=1` and `=basic` | yes | 235 | 116 | no |
| VSCodium, `ACCESSIBILITY_ENABLED=0` and the bare switch | **no** | — | — | — |

Two more things this showed:

- **Qt 6 inside Flatpak describes itself whatever it is told.** Flatpak sets
  `AT_SPI_BUS_ADDRESS` in every sandbox with the bus, and Qt 6's
  `QAtSpiDBusConnection` turns itself on whenever that is set
  (`src/gui/accessible/linux/dbusconnection.cpp`). `QT_LINUX_ACCESSIBILITY_ALWAYS_ON`
  matters for a Qt application outside Flatpak, and for Qt 5, whose
  connection has no such check. **So turning the switch off does not stop a
  Qt 6 Flatpak**, and Settings says *apps are asked not to*, not that they stop.
- `--force-renderer-accessibility=complete`, and the bare switch with no
  `ACCESSIBILITY_ENABLED`, each gave 35 things — the frame and its buttons —
  in their one run. Not chased: alo does not use either.

## What alo OS does about it

- The image names `ACCESSIBILITY_ENABLED=1`, `GNOME_ACCESSIBILITY=1` and
  `QT_LINUX_ACCESSIBILITY_ALWAYS_ON=1` for every person's session, with GTK's
  two at their default (`image/usr/lib/environment.d/60-alo-apps-describe-themselves.conf`).
- The desktop adds `--force-renderer-accessibility=basic` to the command of
  VSCodium, VS Code and Chromium when it starts them
  (`alo_launching::describing_themselves::DRAWN_BY_CHROMIUM`).
- **A Chromium application started outside the desktop** — `flatpak run` typed
  in a terminal — gets the frame from the session's variables and not the page,
  until something asks it for attributes; alo's own fallback does ask
  (`crates/alo-adapters/src/accessibility_bus.rs`), and the table says the page
  then appears a moment later. Neither VSCodium's nor VS Code's Flathub wrapper
  reads a flags file; VSCodium's reads `EDITOR_RUNTIME_ARGS`, which in its one
  run here showed no window and was not used.

**Not yet seen on a booted alo machine.** The next boot walk is where VSCodium
started from the Dock is read again with `alo-app codium controls`.
