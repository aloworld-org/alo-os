# ADR 0069 — The one copy of the default lives where every reader of it can reach it, and the installer creates it

**Status:** proposed, 2026-09-27 — the rule below is the owner's, settled
2026-09-27; it stays proposed until the one measurement under *Consequences*
lands. This amends
[ADR 0066](0066-which-system-a-machine-starts-by-default-is-changed-by-a-verb.md)
term 1, which its own last consequence reserved for a finding like this one, and
it is not a quiet fix.

Written by task 4 of `docs/autonomy/v0-5-the-installer-plan.md`, whose last owed
walk cannot pass until it is answered: the walk measures two sides agreeing about
the default, and today both sides read nothing.

## What was measured

ADR 0066 keeps the default in *the loader's environment on the EFI system
partition — the one filesystem both systems can read and write*. On the computer
a whole-road install actually produces there is no such partition in the
singular. Read off the kept base of 2026-09-26, both disks opened read-only:

- Windows' EFI system partition is **disk 0 partition 1**, 300 MiB, holding only
  `EFI/Boot/` and `EFI/Microsoft/`.
- alo OS's is **disk 1 partition 2**, 512 MiB, holding `EFI/BOOT/` and
  `EFI/fedora/`.
- **Neither holds `EFI/fedora/grubenv`.** Nothing in the tree creates or
  pre-allocates it, and `save_env` can neither create a file nor grow one.
- The only environment block on the machine is `/boot/grub2/grubenv`, 1024 bytes
  and empty of values, on alo OS's **btrfs root** — which Windows cannot read at
  all, and which `/boot/efi` does not reach because nothing mounts the ESP
  there.

The full measurement is in `docs/quirks.md`, *The default's two sides reach two
different partitions, and the one copy is on neither*.

## The hole this opens, and the one it does not

The doubt written down on 2026-09-26 predicted that a default changed from
Windows would be a file alo OS never reads. **That does not follow**, and the
reason is the part of the design worth keeping: alo OS's generated menu does not
read the partition it lives on. It finds the partition by the block itself —

```
search --no-floppy --set=esp --file /EFI/fedora/grubenv
load_env -f (${esp})/EFI/fedora/grubenv saved_entry
```

— with a test holding that no `--fs-uuid` and no `hd0` appear in those lines. A
block on Windows' ESP is a block alo OS finds.

The real hole is different and larger: **the count of copies on a real installed
machine is zero.** With no block, the Windows side returns `NotThere`, the
loader's `search` sets nothing, `load_env -f` fails, and the default is set from
an empty variable. ADR 0066 term 1 is unimplemented rather than wrong.

Two consequences make the placement a decision rather than an implementation
detail:

1. Because the loader finds the partition **by the file**, whoever creates the
   block chooses which partition every later read goes to, for the life of the
   machine.
2. Nothing prevents a second copy. `search --file` takes the first filesystem it
   enumerates, so a block on both ESPs is not an error — it is a silent,
   enumeration-order answer. That is precisely the two answers ADR 0066 exists
   to forbid, and today only the absence of any block is preventing it.

## The decision, as the owner settled it

**One copy, created by the installer, on the EFI system partition that *every
reader this machine has* can reach — and a refusal rather than a second copy if
no single partition satisfies that.**

The first draft of this decision named Windows' partition. That is the right
answer for the road the installer offers today and the wrong rule, because the
next road it offers is *replace Windows*, where there is no Windows partition and
no Windows reader. So the rule is the test, and the partition is what the test
resolves to.

### The test

1. **Count the readers before choosing the partition.** A reader is a program
   that reads or writes this answer on this machine: alo OS's loader always, the
   small Windows program only on a machine that kept Windows, and alo OS's own
   privileged side through the broker's verb.
2. **The one copy goes on the one EFI system partition every one of those readers
   can reach.** Reachable means it can name and open the file there as it already
   stands — not after being taught a new way to find a partition.
3. **If no single partition is reachable by all of them, the install says so and
   writes nothing.** Two copies is the failure ADR 0066 exists to forbid, and
   discovering it at install is worth more than proceeding.

### What the test resolves to, road by road

| The road | The readers | The one copy |
|---|---|---|
| Windows kept, alo OS on a second disk — the only road offered today | the loader, the Windows program, the broker's verb | **Windows' EFI system partition.** Both already reach it unchanged: Windows by `mountvol S: /S`, the loader by `search --file` |
| Windows kept, both systems on one disk — the certified laptop's road | the same three | **that one partition**, because Windows' and alo OS's are the same partition there and the test is trivially satisfied |
| Windows replaced — task 7, the road with no way back | the loader and the broker's verb only; there is no Windows program and no Windows partition | **alo OS's own EFI system partition.** With no second system there is also no menu and nothing to choose between, so the block records a default that has one answer |
| Windows kept, but its partition cannot be reached from alo OS — declined, or protected | the readers disagree | **a refusal.** The install says which reader could not reach which partition, and no block is written |

The unobvious half is worth stating plainly: on a machine that keeps Windows the
copy belongs on **Windows'** partition, not alo OS's. Put it on alo OS's and the
Windows program reads nothing on every two-disk install, which is every install
this road offers.

### The rest of the decision

1. **Who creates it.** The installer, at install, while it is already elevated and
   already writing that partition's firmware entry. It is pre-allocated at its
   full length with the person's answer to *which system starts* already in it —
   the question task 4 asks once — because the loader can only overwrite a block
   in place and can never make one.
2. **A second copy is a refusal, not a fallback.** Before writing, the installer
   holds that no `EFI/fedora/grubenv` exists on any other EFI system partition on
   the machine. `search --file` takes the first filesystem it enumerates, so a
   second copy is a silent, order-dependent answer rather than an error, and
   nothing but this check prevents it.
3. **alo OS still owes the mount.** `/boot/efi` is empty and nothing mounts the
   ESP, so `THE_ENVIRONMENT_BLOCK` resolves to nothing until alo OS mounts
   whichever partition the test chose. Where that is Windows' partition, the mount
   is for this one file and is read-only except under the broker's verb — alo OS
   must not mount the Windows partition read-write for any other purpose.
4. **What does not change.** The verb, its place on the broker's fixed list, the
   record, and *reading is not privileged* all stand as ADR 0066 wrote them. The
   loader is still configured and not patched: a pre-allocated environment block
   is what GRUB already keeps, and the measurement of 2026-09-23 showed the base's
   own `grubx64.efi` reading and saving into one on FAT.

## What this does not decide

- **Whether alo OS mounting Windows' partition is acceptable at all.** If it is
  not, the test resolves to a refusal on the two-disk road, and the alternative
  is that the two sides are reconciled at boot instead — which is a second copy
  by another name, and is why it is not proposed.
- **What a machine does when the answer's partition is later removed** — the disk
  pulled, or Windows reinstalled over its own partition. The loader falls back to
  a block that says nothing, which `alo-starting` already reads as *alo OS*; that
  it should also be *noticed* rather than silently absorbed is a question for the
  road back, not for this decision.
- **A managed machine** may have the default set by policy (ADR 0004, ADR 0016);
  that is the organisation's bound around the person's choice, as ever.

## Consequences

- Task 4 gains the creation step and the no-second-copy refusal, and its walk of
  *the default changed from either side* becomes a walk that can pass: today it
  would measure two sides both reading nothing.
- **The measurement this decision waited on is done, 2026-09-27, and it holds.**
  GRUB's `search --no-floppy --set=esp --file /EFI/fedora/grubenv`, run verbatim
  from the loader the firmware starts on disk 1, set `esp` to **`hd0,gpt1`** —
  Windows' EFI system partition — and `load_env` read back the value the block
  there carried. So on the road offered today the test's answer is reachable by
  both readers in fact, not only in argument. Measured on the kept install's own
  two disks under the same firmware, shim and grubx64 the walk uses;
  `docs/quirks.md` carries it.
- **A second copy is worse than term 2 assumed, and the measurement is why it is
  a refusal.** With a block on *both* ESPs, `search` still returned `hd0,gpt1`
  and read Windows' copy, while the per-device check confirmed both partitions
  held one. GRUB neither complained nor preferred the partition it booted from:
  it took the lower-numbered disk. On this machine that happens to agree with the
  partition the test chooses — **by the accident of Windows being on disk 0.**
  Install alo OS on a disk that enumerates first and the same two copies resolve
  the other way, silently. That is the argument for term 2 being a refusal at
  install rather than a tie-break rule: there is no order worth trusting.
- **What is still owed is code, not a measurement.** Nothing creates the block,
  so the walk of *the default changed from either side* cannot pass until the
  creation step of this decision is built. That is task 4's remaining work.
