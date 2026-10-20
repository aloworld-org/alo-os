# ADR 0128 — Whether a Wi-Fi password may be written to the staging partition

**Status:** **accepted, 2026-10-20, by the owner — with an option this document
ruled out in its own premise.** The ruling: **the environment asks, and no
pre-shared key is written to the staging partition.** Relayed by the desktop
lane, who put A and D to the owner and added that Windows 11, Ubuntu, Fedora,
macOS recovery and ChromeOS all ask at the moment they need the network. The
owner's words were *"can we do it as the windows 11"* and, against the desktop
lane's recommendation of a console picker first and a graphical one later,
*"I agree with the rest"*.

**The answer reached this document through another agent's session, and that is
recorded rather than smoothed over.** What is written below as the decision is
what was relayed; the owner is the one who can confirm it, and the desktop lane
is putting their two sentences on #850 so the provenance is in the repository
and not only in two agent transcripts.

*Originally proposed the same day by the Mac lane, with four options and a
recommendation, under*
[ADR 0023](0023-installed-from-the-machine-it-replaces.md)'s *words that this
one is its own decision and the owner's.*

## What was wrong with this document, and it is the important part

**It presented a thing we had not built as a constraint to design around.** The
context below says the environment *has no interactive surface at all*, and
states it as a fact — which it is. But it is a fact about **what we built**, not
a limit imposed from outside, and putting it in the premise rather than in the
options meant the four options were all answers to *where do we put the
password*, when the question the owner answered was *why is there a password to
put anywhere*.

So the fifth option was never on the page: **give the environment the surface,
and ask.** It removes the exposure rather than narrowing it — the password
exists only in memory on the machine asking for it — and with it, three of the
four facts this document assembled stop applying at all: FAT32 having no
permissions, `tidy_up` not running on the failure road, and BitLocker not
covering the area. They were all facts about storing a secret that now is not
stored.

**Everything below remains correct.** What changed is the premise above it, and
the lesson is not about Wi-Fi: *a document that opens by naming our own
unbuilt thing as a given will produce options that all accept it.*

## Reversed by the owner on 2026-10-20, directly: the pickers are drawn

**Console-first is off.** Shown the text sketch it would have produced —

    Networks found:
      1) BT-Hub-7A2C
      2) Virgin Media
    Type a number: 2
    Password: ________

— the owner answered: *"we have design these in figma, ask desktop to do, and
the rest ask desktop will answer"*. So the pickers follow the design like every
other surface in this product, and the desktop lane draws them.

**This answer came from the owner directly**, which the two before it did not.
The ruling that the environment asks, and the console-first recommendation, both
reached this document through the desktop lane's session; this reversal was said
to the Mac lane in its own words. The distinction is recorded because it is the
difference between a decision one can act on and one that wants confirming.

**What is unchanged:** the environment asks, and **no pre-shared key is written
to the staging partition.** That is the decision this ADR exists for and the
reversal does not touch it. What changed is the front end.

**Why the reversal is affordable**, and it is the section below rather than
anything here: three of the four reasons for console-first were measured and
wrong. The fonts are already in the base, `libdrm` is already in the base, Mesa
and EGL and GBM are neither needed nor present for a software raster, and the
area has 779 MiB spare. Had that been measured before the recommendation, this
reversal would not have been needed.

**And the departure is withdrawn rather than written down.** The section below
records a console picker as a deliberate departure from a design that exists.
It is no longer a departure, because it is no longer being built. The reasoning
stays, because *the design was already there and both lanes reported an absence*
is the part worth keeping.

### What was recommended, and what it would have cost

*Superseded above. Kept because the owner agreed to it and the record should show
what they were told.*

**A plain text picker on the console first; a graphical one later.** Networks
listed, type the number, type the password — the numbered-list shape Debian has
used for twenty years. The desktop lane's reasoning, which the owner agreed to:

- the environment already has the two hard pieces, a console and the tool that
  lists networks and joins one. What is missing is the asking;
- what happens underneath is identical either way — NetworkManager joining a
  network is the same call whether a text list or a drawn picker chose it — so
  the graphical version replaces a front end later and throws nothing away;
- a graphical picker needs a display stack, fonts and keyboard input inside an
  initramfs that #847 has just taken from 79 MB to 124 MB. That is weeks, and it
  would hold the whole road.

### That third reason was not measured, and most of it is wrong — 2026-10-20

**Both lanes repeated it and neither had measured it.** It was the reason put to
the owner, it is recorded above as agreed, and it is corrected here rather than
quietly edited, because what the owner agreed to should stay visible beside what
turned out to be true.

| what the reason claimed | measured |
|---|---|
| fonts must be added | **already in the base.** `AdwaitaSans-Regular.ttf` and `AdwaitaMono-Regular.ttf` are in `/usr/share/fonts/`, which is where the design's own faces live. They cost a line in `install_items`, not a package |
| a display stack must be added | **`libdrm` is already in the base.** A screen rastered in software draws to a DRM dumb buffer and needs nothing else |
| Mesa, EGL, GBM | **not needed, and not there.** The base has no `libEGL`, no `libgbm`, no `libgallium` at all — and a software raster wants none of them. Only a GL-accelerated screen would, and the design's 06 is a wallpaper, a card, text and a progress bar |
| the initramfs is 124 MB and growing | **the area has 779 MiB spare.** Measured from the published candidate: 229 MiB staged of the 1008 MiB the area holds, 22.7% used |

**So size is not the obstacle, and three orders of magnitude of headroom is not
a close call.** The sentence *that is weeks* may still be right; the reason given
for it was not.

### What it does cost, named so the next estimate is better than this one

- **Keyboard input.** The design's 06 carries `Action / View details` and
  `Action / Accessibility`; its own asking screens would carry a text field. The
  environment reads a console today and has no input road to a drawn surface.
- **The DRM road, without giving up static linking.** `alo-installing` is built
  statically against musl **on purpose** — `image/installing/Containerfile` says
  why: *what is copied into the environment depends on nothing in it.* Linking
  `libdrm` would give that up, so drawing wants the modeset and dumb-buffer
  ioctls written directly. Bounded, and real.

  **And smaller than that, measured 2026-10-20 out of the kernel the
  environment already boots** (`6.19.14-101.fc42.x86_64`, its own `config`
  file, with a control — a config name that cannot exist reports absent):

  | | |
  |---|---|
  | `CONFIG_DRM=y` | the DRM core, built in |
  | `CONFIG_DRM_SIMPLEDRM=y` | **a DRM device from the firmware's framebuffer, with no GPU driver at all** |
  | `CONFIG_SYSFB_SIMPLEFB=y`, `CONFIG_FB_EFI=y` | the handoff from UEFI's own graphics output |
  | `CONFIG_DRM_FBDEV_EMULATION=y` | and `/dev/fb0` beside it |
  | `CONFIG_DEVTMPFS=y`, `CONFIG_DEVTMPFS_MOUNT=y` | so the device node appears without anything creating it |

  All **built in**, not modules — which is why `find` for `simpledrm.ko` comes
  back empty and why a reader checking for modules would conclude the opposite.
  **So nothing has to be added to the initramfs for a drawable surface**: no
  module, no library, no Mesa, no `libdrm`. The kernel presents
  `/dev/dri/card0` from the mode the firmware left, and a framebuffer beside it.

  What is left is our own code — open it, take a dumb buffer, put pixels in it —
  and with fbdev emulation present even the ioctls may be avoidable for a first
  screen. That is not weeks.

- **The mode is the firmware's, not the design's.** `simpledrm` presents
  whatever the firmware set, which is not 1440×960 and is not ours to choose.
  `CLAUDE.md` already requires this of every surface — *nothing is built to one
  screen size… every layout is derived from the display it is on* — and the
  environment is where it bites first, because there is no compositor to
  normalise anything. The frames are drawn at 1440×960; the environment has to
  derive from what it is given.
- **A real display.** The environment runs before anything has configured a
  screen, on whatever mode the firmware left. Nothing in this repository has
  drawn under those conditions, and a boot that comes up black is worse than a
  console that comes up readable.

### And the design already requires it, which changes the question

`docs/design/figma-snapshot/ROOTS.md` lists all 17 installer frames, and
`70-28.xml` exports them. **`370:30206 — 06 · Install after restart` is the
environment's own screen**: 1440×960, the product wallpaper, `Choice / Image
verified`, `Measured progress`, `Action / View details`, `Action /
Accessibility`. Its words, read from the file: *Installing alo OS on the disk you
confirmed* — past tense, after a restart — and `05 · Prepare and restart` says
*it will not ask you to choose the disk again*. Windows is not running at that
point and cannot draw it.

**So the display stack is not a cost the picker introduces. It is a cost the
design has been owed since 06 was drawn**, and a picker is one more screen on a
surface already owed. The question is not *picker or console*; it is *when is 06
paid for*.

**And the frames were in this repository the whole time.** Both lanes reasoned
from code and from ADR 0023's prose, reported an absence, and neither opened
`docs/design/figma-snapshot/ROOTS.md` — the file whose only purpose is to say
what the design contains. `CLAUDE.md` says the design is followed and is right
until an ADR says otherwise. **A console picker is therefore a departure from a
design that exists, and this section is where it is written down.** It is not
decided here: what is decided above is that the environment asks, and the owner
agreed to console-first without this measurement in front of them.

**What it costs, said plainly as the rejected options were.** A person
installing alo OS types their Wi-Fi password at a text prompt on a console, which
is less than the rest of this product's interface promises and is the thing a
graphical picker later repays. And the environment grows an interactive surface
it did not have — which is a surface to review, and law 2's concerns do not
apply to it only because there is no agent in the environment at all
(`crates/alo-installing/src/program.rs` says so in its own header).

## What this means for the work

- **Nothing is written to the staging partition**, so the three facts about
  storing a secret there are moot and the removal road does not have to wipe
  bytes.
- **802.1X is unchanged.** ADR 0023 already said enterprise Wi-Fi cannot be
  carried as a pre-shared key; it equally cannot be typed into a two-field
  prompt. A cable or the removable-media road, and #848 says so before the
  shrink.
- **#848 stands and is not withdrawn.** Telling a person before the shrink that
  the next step needs a cable stays true until the picker exists, and after it
  exists it is still true for a machine whose card has no firmware.
- **#854's reading becomes load-bearing rather than incidental.** A picker has
  to name networks something a person recognises and must tell *no carrier* and
  *wrong password* apart, which is what the numeric state and the kernel's own
  flags are for.
**Context:** ADR 0023 §*What is still owed, and is not decided here*;
`docs/features.md` — *installed from the machine it replaces* and *carrying the
network across the restart is how the road above reaches Wi-Fi*; law 1 (*nothing
leaves silently*), law 5 (*the person chooses*);
`crates/alo-installer/src/{staging,removing,reaching_the_network}.rs`;
`image/installing/alo-installing.conf`; #847, #848.

## The question

The installer's road is: a program on Windows checks the machine, shrinks
Windows, makes a 1 GiB area, writes a boot entry, restarts. The environment in
that area then **pulls alo OS from a registry over the network**.

That environment is wired-only. As of #847 it has a supplicant, firmware and a
regulatory database, so it *can* join a wireless network — but it has nowhere to
choose one and nobody to ask. It has no interactive surface at all: the only
thing a person decided before the restart is which disk, handed over as one word
on the kernel's command line.

So for the road to reach a Wi-Fi-only machine, the network has to be **carried
across the restart**: chosen on the Windows side, written somewhere the
environment reads, and used once.

**The thing to decide is whether a pre-shared key may be written to that
staging partition, and under what conditions.** Everything else follows.

## Why it is a decision and not an implementation detail

Four facts, each measured in this repository rather than assumed:

- **The staging partition is FAT32 and has no permissions.** It is the area
  `Program::MakingTheArea` creates and `PreparingTheArea` formats —
  `Format-Volume -FileSystem FAT32`, read out of `program.rs`. Both systems can
  read it. There is no owner, no mode and no encryption. Anyone who can boot the
  machine, or read the disk, can read the file.
- **Nothing puts it back after the restart.** ADR 0023 says *`tidy_up` runs only
  after a successful install so the failure case leaves it there*, and that is
  **half right in a way worth correcting**, because the half it gets wrong is
  the reassuring half. A failure in the **Windows** half does put everything
  back, newest first, including removing the area
  (`staging.rs`, and `a_failure_at_each_step_puts_back_everything_before_it`
  holds it). But the password would be used *after* the restart, and by then the
  Windows program has exited. The environment's `tidy_up`
  (`alo-installing/src/tidying.rs`) is reached from exactly one place —
  `sequence.rs:120`, on the arm where `bootc install` **succeeded**. Every other
  road out returns `Ended::NotInstalled` and tidies nothing. So the failure case
  that matters here is the one with no put-back at all: no network after the
  restart, a wrong password, a card with no firmware, a signature that did not
  verify. The area and everything in it stay on the disk until the person
  removes them.
- **Removing takes the partition out of the table rather than wiping bytes.**
  Both roads use `Remove-Partition` / the partition table (`program.rs`, and
  `tidying.rs` says so in its own words: *removed from the partition table*).
  The sectors keep their contents until something else writes over them.
- **BitLocker does not cover it.** Windows' own encryption is on the Windows
  volume; a partition made beside it is not inside that volume. Our own
  `bitlocker.rs` reads that volume and claims nothing about any other.

So the naive version of this feature writes a person's home or office Wi-Fi
password, in the clear, to a place that survives failure and survives removal.
That is not a thing a lane decides on a Tuesday.

## What is not in question

- **802.1X is out of reach either way.** Enterprise Wi-Fi cannot be carried as a
  pre-shared key. ADR 0023 already says *plug in a cable, said properly and
  before the shrink, may be the honest answer*, and #848 now says it before the
  shrink. Whatever is decided here, a machine on enterprise Wi-Fi uses a cable
  or the removable-media road.
- **The person is told before anything changes.** #848 landed that
  independently: a computer on Wi-Fi is told, in plain words, that the next step
  needs a cable, before the disk is touched. It refuses nothing. That holds under
  every option below, including *no*.
- **Nothing here touches the removable-media road.** ADR 0023 adds that beside
  the road from Windows for machines with no Windows to start from. It is a
  different road and is not specified yet.

## The options

### A. No. The road from Windows stays wired-only.

A Wi-Fi-only person is told before the shrink (#848) and uses a cable or the
removable-media road.

- *Costs:* the product's front door — *download, click, reboot, sign in* — does
  not work for most of the audience it is aimed at, and ADR 0023 says that
  audience is *developers, most of whom have no ethernet cables*. The second
  road has to carry them, and it does not exist yet.
- *Gains:* no secret is ever written anywhere. The simplest thing to explain and
  the simplest thing to be sure of.

### B. Yes, written in the clear, and wiped by the environment on first use.

The Windows half asks for the network and the password and writes them to the
staging partition. The environment reads them, joins, and overwrites the bytes
before it pulls anything.

- *Costs:* the window is small but real and it is exactly the failure case —
  if the environment never gets far enough to wipe (it did not boot, the card
  has no firmware, the password was wrong), the secret stays. And *wiped* on a
  FAT volume on an SSD means *overwritten where the filesystem says*, which the
  drive's own wear levelling may not honour. A promise we cannot check.
- *Gains:* the road works. One file, one read, no new mechanism.

### C. Yes, but only what the machine already holds, and only for one network.

Windows is asked for the profile of the network **it is currently connected
to** and only that one is carried. *Whether Windows will hand the key over at
all is unmeasured* — `netsh wlan show profile name=<name> key=clear` is the
documented way and this installer already runs as an administrator, but nobody
here has run it, and this ADR does not claim it works. If this option is chosen
that is the first thing to measure, on the certified machine, before a line is
written. The person is shown which network will be carried and agrees to it by
name, in the same typed consent that already names what is destroyed.

- *Costs:* the same bytes in the same place as B. All it narrows is *which*
  secret and *whose decision* — which is not nothing, but it is not the risk.
- *Gains:* the person cannot be made to type a password into our program, which
  removes a phishing shape we would otherwise be teaching. They agree to a
  network by name rather than handing over a credential.

### D. Yes, and it is the person's own choice, made with the cost in plain
words — defaulting to not carrying it.

The consent screen says what carrying it means: *your Wi-Fi password is written
to a 1 GB area on this disk. It is removed when installing finishes. If
installing fails, it stays there until you remove the area.* The person says
yes or no. No is the default, and no still installs — over a cable.

- *Costs:* a sentence most people will not read carefully, about a risk that is
  hard to feel. And it is still B's bytes when they say yes.
- *Gains:* law 5 in its own shape — *anything with a risk is offered with the
  risk in plain words and switched on by the person*, and *it is never removed
  from them for their own good*. It is the only option that does not decide on
  somebody's behalf, and the only one where a person who knows their own threat
  model gets to use it.

## What this lane recommends, and why

**D, with C's narrowing inside it.** The person is shown the network their
machine is already on, by name, and asked whether to carry it, with the cost in
one sentence; the default is no; no still installs over a cable.

Three reasons:

1. **Law 5 answers it more directly than law 1 does.** The review question the
   constitution gives is *can the person choose this, knowing what it costs?*
   Here the cost can be stated in one sentence that is true and checkable, and
   the person is the only party who knows whether their disk is likely to be
   read by anybody else.
2. **A is a decision too, and it is the one that gets made by not deciding.**
   Every day this stays open, the front door does not work for the audience it
   is for. That is not a safe default; it is a different cost, paid by a
   different person.
3. **C alone is the attractive answer and does not reduce the risk.** It feels
   safer because nobody types a password. The bytes are identical. It is worth
   having *inside* D, because not teaching people to type Wi-Fi passwords into
   downloaded installers is worth something on its own — but it must not be
   mistaken for an answer to the question this ADR asks.

**What this lane does not recommend: B.** Not because wiping is worthless, but
because *wiped on first use* reads as a guarantee and is not one — the failure
case is the case where it does not happen, and SSD wear levelling means we
cannot check it even when it does. If the answer is yes, it should be yes with
the cost said, not yes with a mitigation that sounds like a fix.

## What is owed whatever is decided

- **The environment must name which failure it hit.** Five are distinguishable
  and one sentence covers them all today: a wrong password, no networks found,
  an association that timed out, an association with no address, and a card
  whose firmware is not there. A person who is told *no networks found* when the
  truth is *this card has no firmware* cannot do anything with that.
- **`tidy_up` must run on the failure road too**, whatever is in the area. That
  is true already and is not about secrets.
- **If the answer is yes, the record says so.** Law 1 is about egress and this
  is not egress, but a person who carried a password across a restart should be
  able to find out afterwards that they did, and that it was removed.

## The one sentence this needs

*May the installer write a Wi-Fi password to the staging partition, and if so,
is it the person's own choice with the cost said (D), or not at all (A)?*
