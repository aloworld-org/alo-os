# The environment in full

**What this is.** The boot environment is one program — `alo-installing` — and
ADR 0128 has just given it screens of its own. This is the Mac lane's ordered
list of what that needs, with what *done* means for each, so that a reader can
check the lane rather than take its word.

**The split it is written under.** The desktop lane owns every sentence a person
reads and every rule about what may be offered; this lane owns every byte of
I/O. A pull request of this lane's that adds a sentence is in the wrong half.

## Status, and the correction that produced this version

**Written at 2026-10-20 morning saying *nothing below is started*. Corrected the
same evening, because by then four of the six had pull requests and — the part
worth reading — two of them were built by a different method than this page
specified.** The page is corrected rather than re-dated, because a plan whose
stated method is not the method anyone used is a plan the next person builds the
wrong thing from, confidently.

| item | where it stands, evening of 2026-10-20 |
|---|---|
| 1 what display there is | **#872**, in the merge queue. **Method changed** — see the item |
| 2 a surface with pixels | **#873**, queued behind #872. **Method changed** — see the item |
| 3a a key arrives | **#874**, queued behind #873 |
| 3b a key becomes a character | **not started, and not startable** — see the item |
| 4a Windows starts the stick | written; **cannot land** — no caller, and the screen that would call it is in no frame |
| 4b a program that writes a stick | **not this lane's to start** — it is not in `docs/features.md` |
| 5 the asking, joined up | not started; the one item that genuinely waits on the other lane |
| 6 joining the network | **#875**, queued behind #874 |

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
configuration or a file, not a demonstration. Four merged or queued pull
requests do not change that row, which is the row this whole page is for.

## 1. What display the firmware left

**Why first:** the desktop lane's layout is a proportion of the display, so it
cannot place anything until something says how large the display is. It is also
the smallest piece here.

**This page specified ioctls and #872 does not use one.**
`DRM_IOCTL_MODE_GETRESOURCES` then `DRM_IOCTL_MODE_GETCONNECTOR` would have
meant hand-written `repr(C)` structures and an `unsafe` block in the one program
that runs as root on a disk it is about to overwrite. The kernel publishes the
same three answers as text under `/sys/class/drm/*/`: `status`, `modes` and
`edid`. So #872 reads those — three parsers, no ioctl, no `unsafe`, nothing
linked.

**What that cost, said plainly:** `modes` is the connector's mode *list* with the
current one first, not a field labelled *current*, and the physical size comes
out of EDID bytes 21 and 22 in **centimetres**, which is coarser than the
`mmWidth`/`mmHeight` an ioctl reports and is why those two are a legibility
check and never a layout driver. A connector reporting zero, no EDID header, or
a short read is `None` rather than a zero that arithmetic divides by.

**Done when:** the environment can say, on a machine, what mode it was left and
how large the glass is. Not yet true on a machine — see *waiting on* below.

## 2. A surface with pixels in it

**This page specified `CREATE_DUMB`, `MAP_DUMB`, `mmap`, `ADDFB2`, `SETCRTC`,
and #873 uses none of them.** The reason is the row above: the pinned kernel
has `CONFIG_DRM_FBDEV_EMULATION=y`, so the same simpledrm device the firmware
left is already presented as `/dev/fb0`. Writing a frame there is
`write_all_at` from the standard library — no ioctl, no `mmap`, no `unsafe`, and
nothing to put back on the way out, because no mode was ever set.

**Two things that follow and are easy to get wrong.** A row is `stride` bytes
wide and **not** `width × bytes`, which is the bug that draws a diagonal; and
`alo-installing` is cross-compiled for Windows in the gate, where
`FileExt::write_all_at` does not exist — so the writing function and its
re-export are `#[cfg(unix)]`, and the geometry and colour arithmetic around them
are not, so they are still compiled and tested on both. *That gate step broke
three stacked branches in a row before this was written down.*

**Done when:** a solid colour fills a real screen from the environment. **This
is the first row in this document that a photograph can prove**, and until
somebody has one, this item is not done whatever the tests say.

## 3. A person can type

The disk-name consent and the Wi-Fi password are both typing, and the
environment had no input road at all. This is two halves, and the second one is
blocked for a reason this page originally got wrong.

### 3a. A key arrives — #874

`/dev/input/event*`, the 24-byte `input_event` struct, and `EVIOCGBIT` to tell a
keyboard from the other things on that road. Two details that cost a mutation
each: a keyboard is recognised by **A, Z and Enter together**, because a power
button reports `KEY_POWER` and would otherwise pass; and the capabilities
bitmask is read **last word first**, because that is the order the kernel fills
it.

### 3b. A key becomes a character — not started, and not startable here

**This page said *the keymap the person chose at the loader*. There is no such
keymap, and there is nothing at the loader that chooses one.** The environment
reads four keys off the kernel command line — `alo.installing.to=`,
`.replacing=`, `.into=`, `.efi=` — and none of them is a layout. The real
mechanism, `/etc/alo/keyboards.toml`, is written by the language sheet on the
**installed** system, which during an install does not exist yet.

So the sentence this page gave as a requirement described a facility nobody
built, and the work it implied cannot start. What it needs first is a decision:
either a layout screen in the environment — which is a sentence a person reads,
so it is the desktop lane's and is **not among the six frames** — or a fifth
command-line key staged by the Windows half, which is a promise to a person and
therefore wants a line in `docs/features.md` with a tier.

**Why this cannot be quietly defaulted to US.** A person typing their own Wi-Fi
password on a French keyboard gets it wrong silently, and the environment's only
report would be *the password was refused*. That is the fifth failure in item 6
telling the truth about the wrong thing.

## 4. The stick: two halves that this page ran together

**The original item read *a program that writes one* and then gave a stick
booting on the certified machine as its definition of done. Those are two
different promises with two different owners, and only one of them is in
`docs/features.md`.** They are separated here because the one that is promised
was buildable immediately and the one that is not would have been built on the
way past.

### 4a. Windows starts the stick — promised, written, and cannot land yet

`docs/features.md`: *and Windows starts the stick, so nobody hunts for a boot
key… no two-second window, no F2, F10, F12 or Esc that differs on every
machine*. The mechanism is one we hold already: `alo-installer` writes boot
entries, and restarting into a device means pointing the firmware's one-shot
boot sequence at the firmware's own entry for it.

`the_one_that_starts` finds that entry in `bcdedit /enum firmware` output by the
disk's friendly name, narrowed to descriptions the firmware marks `UEFI`, and
**refuses rather than guessing** when the firmware names no device or names two:
a firmware that says only *UEFI OS* cannot be matched, and pointing the next
start at whichever removable entry came first is how somebody's machine restarts
into the wrong disk.

**It does not land, and `clippy -D warnings` is why — correctly.** `entries` is
a private module, so a function nothing calls is dead code and the gate refuses
it. That is the gate saying *build the caller or do not start*, and the caller
is the **offer** — *restart from the stick now* — which is a sentence a person
reads, so it is the desktop lane's half.

**And the offer is drawn nowhere, which was found while correcting this page and
is reported here the minute it was found.** Every frame in
`docs/design/figma-snapshot/ROOTS.md` that mentions restarting belongs to the
*staging* road — `370:30179` 05 · Prepare and restart, `370:30371` 05a ·
Preparation could not finish, `375:30307` 05b · Replace · Ready to restart,
`370:30206` 06 · Install after restart — which reboots into the environment
staged on the internal disk. The six stick frames begin at `530:30835` **S1 ·
Start from a stick**, which is the environment's own first screen *after* a
machine has already booted from one. Nothing in either set is Windows offering
to restart into a stick.

**So 4a needs two things decided, and neither is a line of code.** Where the
stick comes from — a writer of ours (4b) or an image the owner publishes and a
person writes with another tool — and a frame for the offer. The lookup works
either way, which is why it was worth writing first; it cannot be called either
way.

Two further notes for whoever picks it up: `Entry::of` accepts **only a braced
GUID**, which made the first test for the identifier-reset bug pass for the
wrong reason; and `crates/alo-installer` is the **Windows lane's** crate by the
table in `two-lanes-that-do-not-wait.md`, so this is a lane crossing and should
be said out loud rather than discovered in a diff.

### 4b. A program that writes a stick — not in `docs/features.md`

`docs/features.md` promises *a road from removable media beside it*, and says in
the same line that the phrase *no USB stick* had *hardened into a promise nobody
decided to make*. A road is not a writer: a stick can arrive from an image the
owner publishes as much as from a program of ours.

**And a writer is the one destructive capability in this whole list.** It erases
a USB device, which is somebody's photographs if they picked the wrong one. By
the standing rule, *when the answer is not obvious, it is scope* — so this wants
a line in `docs/features.md` with a tier, from the owner, before one line of it
is written.

**What is already true and stays true either way:** whatever writes a stick must
label its FAT32 partition `ALO-INSTALL`, because the desktop lane's rule 1 —
*never offer the medium you booted from* — rests on `disks.rs:277` finding that
label, and a stick written without it would offer itself as a target. That is
held by a test, not by a documented step.

## 5. The asking, joined up

The flow: which network, then which disk, then the typed consent, then the
install step that already exists. The sentences and the rules are the desktop
lane's; this is the order they are asked in and what is done with the answers.

**This is the one item that genuinely waits on the other lane**, and the owner
settled why on 2026-10-20: the pickers are drawn in Figma, the desktop lane
builds them, and `docs/design/figma-snapshot/ROOTS.md` has carried all seventeen
installer frames — including `370:30206 06 · Install after restart`, the
environment's own screen — the whole time. **Both lanes reported that design as
absent without opening that file.**

**Done when:** a person boots a stick and reaches `bootc install` having chosen
a network and a disk, with nothing changed before the typed name.

## 6. Joining the network the person chose — #875

`nmcli` is aboard (#847). What was missing was the scan, the join, and **which
of five failures it was** — a wrong password, no networks, an association that
timed out, an association with no address, and a card whose firmware is not
there. #854 named three of those for the wired road from the kernel's own flags;
the wireless ones read NetworkManager's numeric state, because its words are
translated.

Two details worth keeping: `nmcli -t` escapes a colon inside a value as `\:`, so
a network whose name contains one is not two fields; and the join is made
`--temporary`, so **no Wi-Fi profile is written to disk** — which is the second
benefit of the decision that the environment asks rather than carrying a
password across the restart.

**Done when:** each of the five is a different sentence, and none of them is *no
networks found* standing in for the other four. The sentences exist; the reading
of a real card's numeric state is still owed from a real boot.

## What this lane is waiting on, and is not blocked by

- **A real screen.** Items 1 to 3a are written and **cannot be proved here**:
  this build machine has no DRM device at all, no `/dev/dri`, no card, no
  `/dev/fb0`. The demonstration needs the certified machine or a virtual machine
  with a GPU, and no test in this repository can stand in for it.
- **The desktop lane**, for item 5's derivation off the six frames and for item
  4a's caller. Items 1, 2, 3a and 6 needed neither, which is why they are done
  first.
- **The owner**, for two scope rulings that are named above rather than assumed:
  a stick writer (4b), and where a keyboard layout comes from in the environment
  (3b).
- **Image 0.0.8 is unsigned**, which is the owner's. Nothing here needs it.
