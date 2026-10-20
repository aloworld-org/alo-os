# The environment in full

**What this is.** The boot environment is one program — `alo-installing` — and
ADR 0128 has just given it screens of its own. This is the Mac lane's ordered
list of what that needs, with what *done* means for each, so that a reader can
check the lane rather than take its word.

**The split it is written under.** The desktop lane owns every sentence a person
reads and every rule about what may be offered; this lane owns every byte of
I/O. A pull request of this lane's that adds a sentence is in the wrong half.

**Status, 2026-10-20.** Nothing below is started. Six items, in dependency
order — each one is startable the moment the one above it lands, and items 1 to
3 are the ones nothing else can proceed without.

## What is already true, measured rather than assumed

| | |
|---|---|
| the drawable surface | **in the kernel**, built in: `CONFIG_DRM_SIMPLEDRM=y`, `CONFIG_FB_EFI=y`, `CONFIG_DRM_FBDEV_EMULATION=y`, `CONFIG_DEVTMPFS_MOUNT=y`. Nothing to add to the initramfs (#868) |
| the typeface | **in the binary**, not the file system: `include_bytes!` from `crates/alo-shell/fonts/`, Manrope SemiBold at `Weight(600)` (#869) |
| room in the area | **779 MiB spare** — 229 MiB staged of 1008 MiB |
| how big a disk must be | `alo_image::can_hold_alo_os` (#866), one answer for both roads |
| the wireless parts | aboard and verified in the shipped candidate's own initramfs (#847) |
| what is on the disks | `alo-installing`'s `lsblk` read, which the desktop lane widened to `SIZE,MODEL,FSUSED` |
| the layout rule | **proportions of the display**, not logical pixels — the desktop lane's call, and it means nothing here asks for a scale |

**And one thing that is not true and is easy to read as true:** no machine has
put a pixel on a screen from this environment. Every row above is a
configuration or a file, not a demonstration.

## 1. What display the firmware left

**Why first:** the desktop lane's layout is a proportion of the display, so it
cannot place anything until something says how large the display is. It is also
the smallest piece here.

`DRM_IOCTL_MODE_GETRESOURCES` then `DRM_IOCTL_MODE_GETCONNECTOR` on
`/dev/dri/card0`: the connector that is connected, its current mode in pixels,
and `mmWidth`/`mmHeight` beside it in the same call. Handed over as
`alo_displays::Resolution` and `Option<Millimetres>` — the second **labelled a
legibility check and never a layout driver**, which is the desktop lane's
condition for taking it at all.

**Done when:** the environment can say, on a machine, what mode it was left and
how large the glass is; and a connector reporting nothing is `None` rather than
a zero that arithmetic divides by.

## 2. A surface with pixels in it

`DRM_IOCTL_MODE_CREATE_DUMB`, `MAP_DUMB`, `mmap`, `ADDFB2`, `SETCRTC`. No
`libdrm`: `alo-installing` is static against musl so that what is copied into
the environment depends on nothing in it, and linking a C library would give
that up for a handful of ioctls.

**Done when:** a solid colour fills a real screen from the environment, and the
mode is put back on the way out. **This is the first row in this document that
a photograph can prove**, and until somebody has one, this item is not done
whatever the tests say.

## 3. A person can type

The disk name consent and the Wi-Fi password are both typing, and the
environment has no input road to a drawn surface. `/dev/input/event*` through
`EVIOCGBIT` to find a keyboard, then key events to characters through the
keymap the person chose at the loader — not a US layout assumed, because a
person typing their own Wi-Fi password on a French keyboard gets it wrong
silently otherwise.

**Done when:** a key pressed on a real keyboard reaches the program as the
character on the key, and a layout that is not US is held by a test.

## 4. A stick that boots

A program that writes one: the partition table, a FAT32 partition **labelled
`ALO-INSTALL`**, and the staged files. The label is not a documented step but a
test, because the desktop lane's rule 1 — *never offer the medium you booted
from* — rests on `disks.rs:269` finding that label, and a stick written without
it would offer itself.

**Done when:** a written stick boots on the certified machine, and a test fails
if the label is absent.

## 5. The asking, joined up

The flow: which network, then which disk, then the typed consent, then the
install step that already exists. The sentences and the rules are the desktop
lane's; this is the order they are asked in and what is done with the answers.

**Done when:** a person boots a stick and reaches `bootc install` having chosen
a network and a disk, with nothing changed before the typed name.

## 6. Joining the network the person chose

`nmcli` is aboard (#847). What is missing is the scan, the join, and **which of
five failures it was** — a wrong password, no networks, an association that
timed out, an association with no address, and a card whose firmware is not
there. #854 named three of those for the wired road from the kernel's own
flags; the wireless ones need NetworkManager's numeric state, because its words
are translated.

**Done when:** each of the five is a different sentence, and none of them is
*no networks found* standing in for the other four.

## What this lane is waiting on, and is not blocked by

- **The desktop lane's derivation** — rules and proportions off the six frames
  rather than the pixel table measured at 1440×960. Item 1 does not need it;
  item 5 does.
- **A real screen.** Items 1 to 3 can be written here and **cannot be proved
  here**: this build machine has no DRM device at all, no `/dev/dri`, no card.
  The demonstration needs the certified machine or a virtual machine with a
  GPU, and no test in this repository can stand in for it.
- **Image 0.0.8 is unsigned**, which is the owner's. Nothing here needs it.
