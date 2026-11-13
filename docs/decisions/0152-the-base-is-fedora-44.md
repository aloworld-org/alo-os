# ADR 0152 — the base is Fedora 44

**Status:** **accepted, 2026-11-13, by the owner**, relayed by the coordinator
in these words (the quotation marks are the relay's). First, offered the choice between moving the base, giving the screen reader a
private X server, or teaching alo's compositor the accessibility keyboard
interface: *"Move the base to Fedora 43"*, *"the proper fix"*, *"no private
Xvfb"*. Then, told that Fedora 43's own support ends on 2026-12-02: *"Fedora 44"*.
And on the check: *"With the fix"* — the build's honest start check for the
screen reader lands in the same change as the move, so images keep building
until then.
**Date:** 2026-11-13

## What it changes for the person

**The screen reader starts.** On Fedora 42, turning on Settings → Accessibility
→ *Screen reader* started Orca and Orca fell over during startup, every time,
for ever (`docs/quirks/orca-on-wayland-needs-an-x-display-twice.md`). On Fedora
44 it starts and stays running with no X display at all, which the image build
now proves on every build (below).

**And the machine stops telling its owner, in red, that it is out of support.**
alo's own `os-release` already removed Fedora's support promise, but the
machine *was* running a release Fedora stopped fixing on 2026-05-13. Fedora 44
is supported until **2027-05-19** (its own `SUPPORT_END`, read from the pinned
base).

Everything else on the machine moves a release too. What a person could notice
is in *What moves with it*, below.

## Why

Three facts, each measured on 2026-11-13:

1. **Orca 48.9 cannot start on alo's session.** It reads an X display it is
   never given (`KeyError: 'DISPLAY'`), and given one, libatspi 2.56.8's
   keyboard segfaults asking X for its modifiers. No configuration of Fedora 42
   avoids either; both are the newest Fedora 42 carries.
2. **Fedora 42 has been out of support since 2026-05-13.**
3. **Fedora 43 fixes the reader but ends on 2026-12-02**, nineteen days after
   this decision. A move to it would be a second move in a month.

Fedora 44 carries Orca 50.3 and at-spi2-core 2.60.7. **That they start was not
inferred from the version numbers; it was measured**, on 2026-11-13, three
times on the pinned base with the reader's packages and the unit as shipped,
under the recipe's own start check:

1. With the package list as Fedora 42 needed it, Orca 50.3 did not even load:
   *Typelib file for namespace 'DBus', version '1.0' not found*. Its Fedora 44
   package names none of the GObject libraries it imports.
2. With `gobject-introspection` added: *Namespace Atk not available*.
3. With every library Orca's own `require_version` lines name —
   `gobject-introspection`, `atk`, `gtk3`, `gstreamer1`, `libwnck3` — it
   printed *Orca version 50.3, AT-SPI2 version: 2.60.7*, and the check said
   *the screen reader started as the session starts it, and was still running
   fifteen seconds later*, under a Wayland compositor with no X display.

So the recipe names those five, and says why where it names them. **The move
alone would not have made the reader start**, and only a check that starts it
could have said so.

## The decision

- `image/Containerfile`'s `THE_BASE` and `image/installing/Containerfile`'s are
  `quay.io/fedora/fedora-bootc:44@sha256:ea70c130d55794252b081c606603c6d4405561f28ae642a83fe43b1872f82663`,
  the tag's digest when this was written. Every test that holds the recipe's
  base moves with it.
- `image/usr/lib/os-release` says `VERSION_ID=44` and states **no
  `PLATFORM_ID`**, because Fedora 44's own file has none. Repeating a field the
  base no longer states would be alo saying something about Fedora that Fedora
  does not.
- **The screen reader is started in the build the way the session starts it**:
  the unit's own command and environment, a Wayland compositor and no X display,
  a session bus and the accessibility bus, and it must still be running fifteen
  seconds later with no traceback, or the build fails. The compositor and bus
  daemon it needs are installed for the check and removed in the same step.

## What moves with it

The newest of each in each release's repositories, 2026-11-13:

| | Fedora 42 | Fedora 44 | What a person could notice |
|---|---|---|---|
| Linux kernel | 6.19.14 | 7.3.3 | newer hardware support; the BPF LSM and BTF alo's boundary needs are still built in |
| systemd | 257.13 | 259.10 | |
| Mesa | 25.1.9 | 26.2.6 | the drawing of every window, and the software renderer on a machine with no graphics card |
| PipeWire / WirePlumber | 1.4.11 / 0.5.14 | 1.6.11 / 0.5.19 | sound, screen sharing and the camera |
| Flatpak | 1.16.6 | 1.18.4 | how applications are installed and run |
| NetworkManager | 1.52.2 | 1.56.1 | Wi-Fi |
| polkit | 126 | 127 | the administrator password sheet's engine |
| gnome-keyring | 48.0 | 50.0 | where saved passwords live |
| GTK 4 | 4.18.6 | 4.22.5 | applications drawn with GTK |
| Orca / at-spi2-core | 48.9 / 2.56.8 | 50.3 / 2.60.7 | the screen reader starts |
| eSpeak NG | 1.51.1 | 1.52.0 | the voice |
| Python | 3.13 | 3.14 | |
| glibc | 2.41 | 2.43 | |
| LLVM | 20 | 22 | nothing yet: the builder stays on Ubuntu (below) |

### What the image build found that the version table could not

Each of these stopped a local build of the image on the Fedora 44 base, on
2026-11-13, and each is fixed in the recipe where it was found:

- **Orca's libraries** (above).
- **The base no longer carries Bluetooth.** Fedora 42's base shipped `bluez`
  and the recipe never asked for it; on 44 the check for an ordinary computer's
  daemons stopped at *missing /usr/libexec/bluetooth/bluetoothd*. The recipe
  names `bluez` now. Comparing the two bases' package lists, 44 also drops
  `dmidecode`, `pigz`, `pcsc-lite` and `flashrom` — nothing here uses them —
  and adds `kmscon`, installed and not enabled, so the text console is as it
  was.
- **gnome-keyring 50 refuses to start as root** in the build (*error dropping
  process capabilities*), where 48 answered. The machine runs it as the person,
  and as an unprivileged login it answers; the build now asks it that way.

## What the move risks

- **Every measurement this repository made of Fedora 42 is a measurement of
  something it no longer ships.** The ones a test holds were re-read on 44 and
  recorded where they live: terminfo, `/usr/sbin` being a link, the Wi-Fi
  modules, the kernel's BPF configuration, `cosign` still not packaged,
  `xdg-user-dirs` still 0.18. The rest — sizes of files in the installer, the
  bootloader's environment block, polkit's behaviour on a session subject — were
  not re-measured, and each says which release it was measured on.
- **Nothing has booted it yet.** The image was built once, on the lane machine;
  a walk on a booted machine is the next proof, and it is the coordinator's.
- **An update to an installed Fedora 42 machine is a release upgrade underneath
  it.** bootc switches the whole image, so this is the same operation as every
  other update, but it is the first one that changes the release.
- **The builder stays on Ubuntu.** Fedora 44 ships LLVM 22, which removes the
  reason `image/Containerfile` gives for building on Ubuntu; moving it is its own
  change and is not made here.
- **Fedora 44 ends on 2027-05-19.** The next move is a decision for before then,
  not after.
