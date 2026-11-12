# VSCodium can be pressed and read only in Chromium's complete mode and VSCodium's screen-reader mode

- **Version:** VSCodium 1.143.17232 from Flathub (`com.vscodium.codium`, runtime
  `org.freedesktop.Sdk` 26.08), the same build as
  [`chromium-describes-only-its-window-frame-until-asked.md`](chromium-describes-only-its-window-frame-until-asked.md).
- **Whose:** Chromium's (how it names and carries out actions) and VSCodium's
  (when its editor shows its text). alo OS configures both and patches neither.
- **Date:** 2026-11-12, after the boot walk of that day, where
  `alo-app com.vscodium.codium press button Search` was refused and `read`
  did not show the open README.

## What this means for a person

Started the way a machine comes (`--force-renderer-accessibility=basic`), alo
can **list** what is in a VSCodium window but **cannot press anything inside it
and cannot read the document**. A press answers *done* and nothing happens; the
editor says *"The editor is not accessible at this time"* and gives no text.
Started with `=complete`, both work, and VSCodium's editor goes into its screen
reader mode, which changes how it behaves. So alo OS offers it as a switch the
person turns on: *Let alo work fully inside apps like VSCodium*, off as a
machine comes (the owner, 2026-11-12, *"Person's choice, off"*).

## What was measured

**Measured once, by hand, on 2026-11-12. No test or gate runs it again. Not on
an alo machine.** Setup: WSL Ubuntu 24.04 (at-spi2-core 2.52) under a headless
Weston, with a real accessibility bus. `ACCESSIBILITY_ENABLED=1` was set, as on
the image. VSCodium was run **as root**, which forced `--no-sandbox` and
`--user-data-dir=/home/vscm-ud`.

Each row was run **twice** with the same result both times. The switch each
process actually got was read from `/proc/<pid>/cmdline` of the
`--type=browser` process. A press was checked by what changed, never by the
call's return value: the window title changing from `README.md - VSCodium` to
`VSCodium` after *Close (Ctrl+W)*, or new nodes appearing — the Search view's
input, the panel's tabs. The document was a README holding the marker
`ZEBRA-PLUM-7731`.

| Switch (as the browser process got it) | `editor.accessibilitySupport` | Search tab | Toggle Panel | File menu | Close (Ctrl+W) | Press works? | Editor text |
|---|---|---|---|---|---|---|---|
| `=basic` (alo's default) | auto (the default) | `doDefault` | `doDefault` | `doDefault` | `doDefault` | **no.** Returns true, nothing changes (title the same; 229 nodes before and after) | **no.** Entry is named *"The editor is not accessible…"*, 0 characters |
| `=basic` | on | `doDefault` | `doDefault` | — | — | **no**, same as above | **yes**: entry `README.md`, 130 characters, the whole file |
| `=basic` | off | — | — | — | — | not tried | no |
| `=complete` | auto | `click` | `check` | `select` | `press` | **yes**: `click` opened Search (205→253 nodes), `press` closed the tab (title changed) | **yes**, because VSCodium switched itself to *Screen Reader Optimized* (shown in its status bar) |
| `=complete` | on | `click` | — | `select` | — | yes (`click`) | yes |
| `=complete` | off | `click` | `check` | — | `press` | **yes**: `check` opened the panel (225→294), `click` opened Search (294→344), `press` closed the tab | **no** |
| bare `--force-renderer-accessibility` | auto | `click` | — | `select` | — | yes (`click`) | yes, *Screen Reader Optimized* shown |
| `EDITOR_RUNTIME_ARGS=--force-renderer-accessibility=complete` and `=basic` after it | auto | `doDefault` | — | — | — | no | no |

Every control also offers `showContextMenu`. **The action names seen in
complete mode** were `click`, `press`, `clickAncestor`, `check`, `uncheck`,
`select`, `open`, `activate`, `doDefault` and `scroll*`. **In basic mode** the
only `click` and `press` actions were on Chromium's own window-frame buttons
(Close, Minimize, Maximize, Restore); everything inside the page was
`doDefault`.

**Text was read through the Text interface** of the editor's entry (role 61),
the role alo already reads. With screen-reader mode on, the entry holds the
document text.

### `check`, `uncheck` and `select`, pressed

Measured the same day, `=complete` with `auto`, the same setup, **run twice
with the same result both times** (`vscfull-probe.py`, by name and action, with
the control's states and actions read before and after):

| Step | Control offered | Pressed by | What changed |
|---|---|---|---|
| 1 | *Toggle Panel (Ctrl+J)*, not pressed: `check`, `showContextMenu` | `check` | the panel opened (228→327 nodes: TERMINAL, OUTPUT, PROBLEMS…); the button became PRESSED and offered `uncheck`, `showContextMenu` |
| 2 | the same, now pressed | `check` asked for | **not offered** — the control offers one of the two, never both |
| 3 | the same, pressed: `uncheck` | `uncheck` | the panel closed (327→228); the button offered `check` again |
| 4 | the same, not pressed | `uncheck` asked for | not offered |
| 5 | menu item *File*: `select`, `showContextMenu` | `select` | the File menu opened (228→309: *New File…*, *Close Editor*, *Exit*…) |
| 6 | menu item *New Text File Ctrl+N*: `select` | `select` | a new editor opened: `Untitled-1`, window title `Untitled-1 - VSCodium [Superuser]` |

**So a toggle offers exactly the action that turns it over**, and a press
choosing whichever of `check` or `uncheck` it offers always toggles it. `open`
and `clickAncestor` were seen and **not pressed**.

## Conclusions, and whether each was checked

1. **`doDefault` is not a press. Confirmed:** in basic mode `DoAction(doDefault)`
   returned true four times out of four, with no effect. alo's `PRESSING` never
   holds it.
2. **Pressing needs `=complete`. Confirmed.** `click` (tabs), `press`
   (buttons), `check` / `uncheck` (toggle buttons) and `select` (menu items)
   were each measured to change something. Why basic gives `doDefault` fits
   Chromium working out an element's action verb only in complete mode;
   **Chromium's source was not read to confirm it.**
3. **Reading the document needs VSCodium's screen-reader-optimized mode,
   whichever Chromium switch is used. Confirmed:** basic with `on` read the
   text; complete with `off` did not. `=complete` reads it because, like the
   bare switch, it makes VSCodium's `auto` turn screen-reader mode on — which
   **corrects** the 2026-11-11 entry, where only the bare switch was said to.
4. **A `flatpak override` setting `EDITOR_RUNTIME_ARGS` cannot change alo's
   switch. The mechanism is confirmed here; that it is the cause on the alo
   machine only fits.** The Flathub launcher (`/app/bin/com.vscodium.codium`)
   puts `${EDITOR_RUNTIME_ARGS}` **before** the arguments it is given, so the
   switch alo hands after the command comes later and wins.

## What alo OS does about it

- **Settings → Accessibility → Applications → *Let alo work fully inside apps
  like VSCodium***, off as a machine comes, with what it changes said under it.
  On, the desktop starts VSCodium, VS Code and Chromium with `=complete`
  instead of `=basic` (`alo_launching::describing_themselves`); off, `=basic`.
  A change reaches the apps opened after it.
- `PRESSING` (`crates/alo-adapters/src/activating.rs`) gains `check`,
  `uncheck` and `select`, after the words it already had; `doDefault`, `open`
  and `clickAncestor` are not added.
- When a control in one of those apps offers no press and the switch is off,
  the refusal names the switch and where it is
  (`adapters.fallback.cannot-be-pressed-until-alo-works-fully`).
- **Not done:** writing `editor.accessibilitySupport` into a person's VSCodium
  settings. That is their application's setting, and changing it is a choice
  alo would have to offer, not make.

## What to measure next, on a booted alo machine

- each press above through `alo-app`, with the switch off (the new sentence)
  and on (the press happens), VSCodium started from the Dock after the change;
- the exact list of things screen-reader mode changes in the editor — not
  measured here;
- whether running as a normal user rather than root, with the sandbox on,
  changes any row.

Scripts: `/root/vsc-measure/` in WSL Ubuntu on the measuring machine (`run.sh`,
`walk.py`, `pressing.py`, `closetest.py`, `vscfull-run.sh`, `vscfull-probe.py`).
They are not in the repository.
