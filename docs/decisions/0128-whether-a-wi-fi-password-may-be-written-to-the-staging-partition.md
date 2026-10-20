# ADR 0128 — Whether a Wi-Fi password may be written to the staging partition

**Status:** **proposed, 2026-10-20, by the Mac lane.** Four options, with a
recommendation. Not decided; [ADR 0023](0023-installed-from-the-machine-it-replaces.md)
says in its own words that this one *is its own decision and the owner's*, and
nothing is built until it is made.
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
