# ADR 0062 — The menu a machine starts at is alo OS's, and Windows stands behind it

**Status:** **accepted, 2026-09-21, by the owner**, with the three terms under
*As the owner accepted it*, which are part of the decision rather than
commentary on it. Written for task 4 of `docs/autonomy/v0-5-the-installer-plan.md`
(*Alongside Windows, switching between them easily, and back again*) before any
of its code, because the task needs the answer and neither decision it rests on
gives one.
**Date:** 2026-09-21
**Proposed by:** the installer workstream
**Context:** [ADR 0023](0023-installed-from-the-machine-it-replaces.md) §4
(*Windows is either retained alongside … or replaced*) and its constraint, *a
tested bail-out that leaves Windows bootable at every step until the final,
named point of no return*;
[ADR 0033](0033-the-certified-laptop-is-installed-the-way-a-customer-installs.md)
(alongside is the default, and on the certified laptop the only mode);
[ADR 0011](0011-the-base-is-rented-and-the-image-is-a-container.md) (the base
is rented); `docs/booting.md`, which records the base's signed shim and GRUB
2.12 as what starts alo OS today; `crates/alo-installer/src/program.rs`, which
already sets the firmware's next start for one restart.

## The question in one line

**When a machine with both systems on it starts, which program shows the menu
that offers them?**

Task 4's acceptance asks for a menu **on every start, with a short countdown
and the last-chosen system preselected**, in the owner's words of 2026-09-14:
*run the two operating systems and switch from one to the other easily.* A
firmware's own boot menu is none of that — it is reached by a key a person has
to know, it has no countdown, and it remembers nothing. So a program has to
show it, and there are two that already sit on the disk.

Neither ADR 0023 nor ADR 0033 says which. Both were read for it before this was
written: 0023 mentions a boot-menu key only as the thing a person should never
have to find, and 0033 does not mention one.

## The two that were weighed

**A. alo OS's loader shows it.** The base already ships GRUB 2.12 behind its
signed shim, installed by `bootupd`, and `docs/booting.md` records it starting
alo OS on every machine this repository has booted. GRUB offers a second entry
that starts `\EFI\Microsoft\Boot\bootmgfw.efi`, counts down, and keeps the last
choice as its own saved default — *last chosen* is something it already does,
not something built. This is how a Linux system sits beside Windows almost
everywhere it does.

Its cost is position. alo OS's loader stands in front of Windows, so a loader
that fails stands in front of Windows too — and ADR 0023 says Windows is
reachable at every step.

**B. Windows Boot Manager shows it.** Both entries in its `displayorder`, a
`timeout`, Windows first in the firmware's order. Windows stays in front and
nothing of ours can stand in its way.

Its cost is the third requirement. Windows Boot Manager has no notion of *the
last one chosen*: something would have to rewrite its `default` on every
start, from both sides — which means alo OS editing Windows's BCD store, a
registry hive on the EFI partition, from Linux, after every boot. That is the
fragile half, and it would be ours.

## The decision

**Option A.** The owner chose it on 2026-09-21, having been told the
fall-through below and the one case it does not cover, in plain words.

## As the owner accepted it, 2026-09-21

1. **Windows stands directly behind alo OS in the firmware's start order.**
   The installer puts Windows Boot Manager second, immediately after alo OS's
   entry, so a loader that cannot be started is passed over by the firmware
   itself and Windows starts with no keypress. **A test proves it** in the
   virtual machine task 10 built: make alo OS's loader unstartable, start the
   machine, and show Windows reaches its desktop with nothing typed.
2. **The case it does not cover is named rather than claimed.** A loader that
   *starts* and is then broken — a configuration it cannot read, a prompt
   instead of a menu — is not passed over, because to the firmware it started.
   `docs/booting.md` says so in the steps a person reads, and names the
   machine's boot-menu key as the way to Windows in that case. Nothing claims
   the fall-through reaches it.
3. **The last choice is the loader's own saved default, and nobody keeps a
   copy.** Neither Windows nor alo OS remembers which system was chosen; GRUB
   does, in its environment block, and both sides read it from there. Two
   copies drift, and a menu that preselects one thing while a setting says
   another is the bug this term exists to prevent.

## What stays as it was

- **The base's GRUB is configured, never patched** (ADR 0011). Windows's entry
  is configuration the installer writes, and a base update that replaces the
  loader keeps it or the test in term 1 says it did not.
- **Restart into alo OS** and **Restart into Windows** stay one-restart
  switches through the firmware's next-start entry — `bootsequence` from
  Windows, which `program.rs` already sets, and `BootNext` from alo OS. They
  pick the next start and leave the menu's default untouched. The menu is for
  a person at the machine when it starts; the switches are for a person in
  either system.
- **Secure Boot** stays as ADR 0033 §4 has it. The shim chain-loading Windows's
  own signed manager is the ordinary case, and nothing here changes what the
  installer refuses.

## What this does not decide

**Fast Startup.** Found by task 10 on 2026-09-21: the settled Windows reads
`HiberbootEnabled 0x1`, on by default, so *Shut down* leaves the Windows volume
hibernated rather than closed. A restart is a full shutdown, so the switches
above are safe; a person who shuts down and then picks alo OS at the menu has a
hibernated Windows volume on the disk. **alo OS never mounts the Windows
partition read-write**, which holds either way, and task 4 writes the test that
holds it. Whether the installer also turns Fast Startup off, and tells the
person why, is a separate question for the owner — it changes Windows's
behaviour, and a feature update is known to turn it back on — and is left open
here rather than decided in passing.

## Consequences

- Task 4 builds option A, with term 1's fall-through test and term 2's words in
  `docs/booting.md`.
- `docs/quirks.md` gains whatever the base's GRUB turns out to need to carry a
  second entry across a `bootupd` update, when task 4 measures it.
- Task 7 (*Replace Windows*) is unaffected: with Windows gone there is one
  entry and nothing to choose between.
