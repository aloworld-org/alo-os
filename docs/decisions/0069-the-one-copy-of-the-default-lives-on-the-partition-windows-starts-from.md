# ADR 0069 — The one copy of the default lives on the partition Windows starts from, and the installer creates it

**Status:** proposed, 2026-09-27 — the owner decides. This amends
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

## The decision proposed

**One copy, created by the installer, on the EFI system partition the running
Windows starts from — and a refusal rather than a second copy if it cannot be
made there.**

1. **Where it lives.** ADR 0066 term 1's *the EFI system partition* is amended to
   name it: the ESP that Windows starts from, which is the one `mountvol S: /S`
   reaches. The unobvious half of this is that it is **not** alo OS's own ESP.
   Both sides already reach Windows' ESP as they stand — Windows by `mountvol`,
   alo OS by `search --file` — so this placement needs no change to either
   reader. The reverse placement would leave the Windows side reading nothing on
   every two-disk install, which is every install this installer offers.
2. **Who creates it.** The installer, at install, while it is already elevated
   and already writing that partition's firmware entry. It is pre-allocated at
   its full length with the person's answer to *which system starts* already in
   it — the question task 4 asks once — because the loader can only overwrite a
   block in place and can never make one.
3. **A second copy is a refusal, not a fallback.** Before writing, the installer
   holds that no `EFI/fedora/grubenv` exists on any other EFI system partition on
   the machine. If one does, the install says so and does not write a second;
   two copies is the failure ADR 0066 names, and discovering it is worth more
   than proceeding.
4. **alo OS still owes the mount, and it is now a smaller debt.** `/boot/efi` is
   empty and nothing mounts the ESP. `THE_ENVIRONMENT_BLOCK` resolves to nothing
   until alo OS mounts **Windows' ESP** there — which is the partition alo OS
   must not mount read-write for any other purpose, so the mount is for this file
   and is read-only except under the broker's verb.
5. **What does not change.** The verb, its place on the broker's fixed list, the
   record, and *reading is not privileged* all stand as ADR 0066 wrote them. The
   loader is still configured and not patched: a pre-allocated environment block
   is what GRUB already keeps, and the measurement of 2026-09-23 showed the
   base's own `grubx64.efi` reading and saving into one on FAT.

## What this does not decide

- **The single-disk road.** When the installer offers replacing Windows or a
  single-disk install beside it, there is one ESP and the question does not
  arise; term 1 as amended still names the right partition, because it is the
  same one.
- **A machine with no Windows.** After *replace Windows* there is no second
  system to choose between and no menu, which `on_this_machine.rs` already reads
  as *this machine offers alo OS* rather than as an error.
- **Whether alo OS mounting Windows' ESP is acceptable at all.** If it is not,
  the alternative is that alo OS's side reads the block through the loader's own
  saved default on its root and the two sides are reconciled at boot instead —
  which is a second copy by another name, and is why it is not proposed here.

## Consequences

- Task 4 gains the creation step and the no-second-copy refusal, and its walk of
  *the default changed from either side* becomes a walk that can pass: today it
  would measure two sides both reading nothing.
- One measurement is still owed and no reading substitutes for it: that GRUB's
  `search --file`, running from the loader on disk 1, reaches disk 0's ESP on
  this firmware. Every other claim above is read off an installed machine.
- If that measurement fails, this decision fails with it and the block belongs on
  alo OS's own ESP with the Windows side taught to find alo OS's start partition
  rather than its own — which is the more code and the reason it is the second
  choice, not the first.
