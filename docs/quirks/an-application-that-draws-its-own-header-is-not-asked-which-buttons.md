# An application that draws its own header is never asked which buttons it shows

- **Version:** from Flathub on 2026-11-12: GNOME Text Editor 51.0, Papers and
  Loupe (GNOME runtime 51), Ptyxis (GNOME runtime 50), Firefox 160.0
  (freedesktop runtime 25.08), VSCodium 1.143.17232 (freedesktop SDK 26.08),
  KCalc 26.08.3 (KDE runtime 6.11). Plus GTK 4 and libadwaita windows of our own,
  run in GNOME runtime 51 and in GNOME SDK 49.
- **Whose:** the platform's. Wayland has no way for a compositor to ask which
  window buttons an application draws in its own header. `xdg_decoration` says
  only *who* draws the frame. alo OS configures the toolkits; it patches none of
  them.
- **Date:** 2026-11-12.

## Behaviour

The owner, 2026-11-12: *"No need to add controls if the app has, we can only add
the ones that are not there."* To follow that, alo has to know which of
Minimise and Close an application already shows. Nothing on the protocol says.
Reading the application's pixels or accessibility tree to find out would be
watching it, which `CLAUDE.md` forbids. So what each application does was
measured once, by hand, and written down.

**Method.** Ubuntu 24.04 under WSL, not an alo machine. A headless sway 1.9
(which, like alo, advertises `xdg_decoration` and answers server-side), with
every window floating, inside `dbus-run-session`. A small stand-in for alo's
Settings portal owned `org.freedesktop.portal.Desktop`. It was run in three
ways:

- **refused**: every key `NotFound`, which is what alo answers an application
  without the `appearance-settings` grant. On a fresh machine that is every
  application.
- **granted**: the appearance keys and the two pointer keys, as alo answers a
  granted application before this change.
- **layout served**: the same, plus `org.gnome.desktop.wm.preferences`
  `button-layout`.

Each application started with `WAYLAND_DEBUG=1` to read its decoration requests
and its `app_id`. Its window buttons were read from its accessibility tree,
**once, at measuring time, by the measuring script**. alo itself never does
this. A screenshot was taken with `grim`. The scripts (`run.sh`, `portal.py`,
`buttons.py`, `probe.py`) are in the lane's scratch, `C:\dev\window-buttons`,
and not in this repository. **Nothing here runs them again.**

| application | decoration request | refused | granted | layout `appmenu:minimize,maximize,close` |
|---|---|---|---|---|
| GNOME Text Editor | none; binds KDE's server-decoration manager, requests nothing | Minimise, Maximise, Close | **Close** | Minimise, Maximise, Close |
| Papers | the same | Minimise, Maximise, Close | **Close** | Minimise, Maximise, Close |
| Loupe | the same | Minimise, Maximise, Close | **Close** | Minimise, Maximise, Close |
| Ptyxis | the same | Minimise, Maximise, Close | **Close** | Minimise, Maximise, Close |
| GTK 4 / libadwaita, runtime 51 | the same | Minimise, Maximise, Close (`menu:minimize,maximize,close`) | Close (`menu:close`) | as served (`appmenu:minimize,maximize,close`) |
| GTK 4 / libadwaita, SDK 49 | the same | **Close** (`menu:close`) | Close | — |
| Firefox 160 | KDE server-decoration, asks for **client-side** | **Close** at the end of the tab strip, no Minimise, no Maximise (screenshot) | the same; it reads only `org.freedesktop.appearance` | — |
| VSCodium, default `window.titleBarStyle` | none seen; its wrapper does not pass `WAYLAND_DEBUG` through | Minimise, Maximise/Restore, Close in its own title bar | the same | — |
| VSCodium, `window.titleBarStyle: native` | none seen | none of its own; the compositor drew the frame (screenshot) | — | — |
| KCalc (Qt 6) | `xdg_decoration`, `set_mode(2)`: **server-side** | none: the compositor draws them | — | — |

## What this showed

1. **GTK changes its buttons when the person grants an application their
   appearance settings.** Ungranted, GTK in runtime 51 draws Minimise, Maximise
   and Close. Granted (the portal answers but has no `button-layout`), it draws
   Close alone. A grant about colours was deciding which window buttons appear.
2. **When the portal answers `button-layout`, GTK draws what it says.** That is
   what makes the buttons a fact alo controls rather than one it guesses.
3. **GTK's own default moved between runtimes**: SDK 49 drew Close alone
   ungranted, runtime 51 draws three. A Flathub update that moves an application
   to another runtime can change its buttons.
4. **No GTK application asks `xdg_decoration`**, so "draws its own header" is
   read correctly from protocol state. Qt agrees to server-side, as alo asks.

## What alo OS does about it

- **The Settings portal answers `button-layout` =
  `appmenu:minimize,maximize,close`** to every granted application
  (`crates/alo-portals/src/the_window_buttons_gtk_is_told.rs`). That is the
  layout GTK in runtime 51 draws ungranted, so a GTK application shows the same
  three buttons granted or not, and none it had is taken away.
- **The shell reads who draws each window's header from protocol state**: a
  window whose acknowledged state is server-side gets the full band
  (`crates/alo-shell/src/window_edge_who_draws.rs`).
- **For a window that draws its own, the shell looks up its `app_id` in a
  reviewed table** (`crates/alo-shell/src/the_buttons_an_application_draws.rs`):
  the four GNOME applications and VSCodium show Minimise and Close, so alo adds
  Window actions alone. Firefox shows only Close, so alo adds Window actions
  and Minimise. A test fails if `alo-software/shipped.toml` gains an application
  that is in neither the table nor the list of applications that agree on
  server-side.
- **Every other application that draws its own header is one alo cannot
  know.** It gets **Window actions only**, never a duplicate on a guess.

## What that costs, and who is affected

- **An unmeasured application that draws its own header and has no Minimise
  gets none from alo.** For example: a GTK application installed later on a
  runtime whose default is `menu:close` and that the person has not granted, or
  one that never declares an `app_id`. It can still be minimised with the
  keyboard and closed with its own Close. The cure is a row, measured.
- **A GTK application outside a sandbox** reads its own GSettings, not the
  portal. None is shipped.
- **Dolphin and Ark were not measured themselves.** They are counted as
  agreeing to server-side because KCalc (same Qt 6 and KDE runtime family)
  was measured doing so. Their runtime, KDE 6.10, was not installed here.
- **VS Code (`com.visualstudio.code`) and Chromium were not measured**, so they
  are not in the table and get Window actions only. On VSCodium alo does not
  set `window.titleBarStyle`: switching off an application's own buttons is
  what the owner's 2026-11-12 sentence rules out.
- **The table can go stale.** Each row is a measurement, and nothing measures it
  again. **Not yet seen on a booted alo machine.** The next boot walk is where
  the band over Firefox, Text Editor and VSCodium is looked at.
