# ADR 0066 — Which system a machine starts by default is changed by a verb, and kept in one place both systems can reach

**Status:** **accepted, 2026-09-23.** Written to unblock task 17 of
`docs/autonomy/v0-5-the-installer-plan.md`, which found the hole and said it
probably needed a decision: it does. Nothing is built in the change that adds
this.
**Date:** 2026-09-23
**Proposed by:** the dev PC, for the third PC's lane, which holds the alo OS
side of installing alongside Windows
**Context:** [ADR 0062](0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md)
term 3 — *the last choice is the loader's own saved default, and nobody keeps a
copy*; [ADR 0001](0001-the-capability-model.md) §2 (a privileged act is a verb
on the broker's fixed list, with the person's approval);
[ADR 0011](0011-the-base-is-rented-and-the-image-is-a-container.md) (the base's
loader is configured, never patched); task 16, which built the menu and the
*Restart into Windows* verb; task 4, which owns the Windows side.

## The hole

ADR 0062 says the default is **the loader's own saved default and there is only
one copy of it**. Two facts then collide:

- **A person changing it in Settings is not root**, and the loader's saved
  default lives under `/boot`, which is root's.
- **The same setting must be changeable from Windows**, and Windows cannot
  write a Linux filesystem at all.

A copy on each side would answer both and break the term that matters: two
copies drift, and a machine that starts one system while Settings says the
other is exactly the confusion ADR 0062 exists to prevent.

## The decision

**One copy, on the EFI system partition, and every writer goes through
something privileged that says what it did.**

1. **Where it lives.** The saved default is kept in the loader's environment on
   the **EFI system partition** — the one filesystem both systems can read and
   write, and the one the loader already reads at start. It is not duplicated
   in alo OS's settings, in the Windows program, or in the record; those read
   it.
2. **How alo OS changes it: a verb on the broker's fixed list.** Settings does
   not write `/boot`. It asks the broker, as it does for printers, the network
   and updates (ADR 0001 §2). The verb takes **the identity of a system the
   menu already offers** — not a path, not a string — so the closed, typed verb
   list stays closed and typed. The person's confirmation is the same as any
   verb's, an agent may ask for it under a grant, and it is in the record like
   everything else.
3. **How Windows changes it:** the small program the installer leaves behind
   (task 4), which already runs elevated, writes the same file on the same
   partition. It is the Windows side of one setting, not a second setting.
4. **Reading is not privileged.** The environment block is world-readable;
   `alo-starting` reads it to show which system is the default, and the Windows
   program reads it for the same sentence. **Both sides show the same answer
   because there is one answer**, and a test holds that changing it on either
   side is seen by the other.
5. **The loader is configured, never patched** (ADR 0011). Nothing here changes
   the base's loader; the environment block is what it already keeps.

## What this does not decide

- **Restart into the other system for one start** is `BootNext`, which task 16
  already built as its own verb. It is not the default and does not touch it.
- **Which system the installer sets as the default at install** is the
  installer's question and stays in task 4, where the person is asked once.
- **A managed machine** may have the default set by policy (ADR 0004, ADR 0016);
  that is the organisation's bound around the person's choice, as ever.

## Consequences

- Task 17 builds the verb, its words in every language, and the test that the
  two sides agree. `SystemVerb` gains one member; nothing else about the list
  changes.
- Task 4 makes the Windows program write the same file and show the same
  sentence.
- If the EFI system partition turns out to be unwritable from one side on a
  real machine, that is a finding for `docs/quirks.md` and a change to this
  decision, not a second copy added quietly.
