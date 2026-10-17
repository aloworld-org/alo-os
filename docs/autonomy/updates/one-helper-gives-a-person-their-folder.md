# One helper gives a person their folder — and the folder was in the wrong place

**Branch:** `task/mac/one-helper-gives-a-person-their-folder`, stacked on
`task/mac/one-component-gains-cap-chown` (#743)
**Date:** 2026-10-17
**Decisions:** [ADR 0119](../../decisions/0119-the-capability-to-give-a-directory-away-goes-to-the-smallest-thing-that-can-hold-it.md)
(written by #743), [ADR 0120](../../decisions/0120-the-undo-folder-is-roots-own-and-not-inside-the-persons.md)
(written here)

## What this change is

`crates/alo-giving-a-folder` — the privileged step
[ADR 0045](../../decisions/0045-what-undoing-rewinds-to.md) term 8 has asked for
since 2026-09-21 and nothing has ever provided: the one program on this machine
that may give a directory to a person. It makes the folder that holds what an
undo would put back, makes one directory per person inside it, and gives each
person theirs. Its unit holds `CAP_CHOWN` and no other capability.

**And it moved that folder.** The location ADR 0045 and
`docs/contracts/kept-undo-folder.md` both named — `/var/lib/alo/undo` — made two
of ADR 0045's own terms false. It is now `/var/lib/alo-undo`. ADR 0120 is the
decision and carries every readback.

## The judgement this change had to make, in one sentence

**This makes existing promises true rather than adding one**, because the folder
ADR 0045 term 8 specified could not be created where it was specified, and where
it could be created it let a person move the whole thing away and kept a second
person out of their own directory — so moving it adds no promise to anybody and
makes three that were already written down (ADR 0045 terms 7 and 8, ADR 0118's
second person) actually hold.

The owner approved the work itself on 2026-10-16 — *"1: Make it when the account
is made"* — and ADR 0118 named the next record as *which component gains the
capability, written against a measurement rather than a preference*. The path
move was not foreseen by either and is reported here as its own decision so it
can be disagreed with on the reasoning and not only discovered in a diff.

## How the fault was found, and why no test could have

The crate was built, gated green, and then **installed on a machine and
started**, which is the step that found everything below. In order:

**1. The unit would not start.** `226/NAMESPACE`:

```
alo-giving-a-folder@1000.service: Failed to set up mount namespacing:
  /var/lib/alo: No such file or directory
```

`ReadWritePaths=` on a path that does not exist fails the sandbox before the
program runs. **Half of this was my test machine, not the product** — the image
creates `/var/lib/alo` through `tmpfiles.d` at boot and the virtual machine had
never applied it. Separating those two took one command and is why the fix is
`StateDirectory=alo-undo` rather than a wider `ReadWritePaths=`: systemd makes
the directory and then opens it, so a fresh machine behaves like every later
start.

**2. With the tmpfiles line applied by hand, the program failed instead** —
`Permission denied`, **as root**:

```
alo-giving-a-folder: /var/lib/alo/undo could not be made, so nothing of this
  person's files can be kept for an undo: Permission denied (os error 13)
```

`CapabilityBoundingSet=CAP_CHOWN` drops `CAP_DAC_OVERRIDE`, and
`image/usr/lib/tmpfiles.d/alo.conf` says `d /var/lib/alo 0700 alo alo -`. That
directory is the **person's**, deliberately and correctly — it holds the record
and `grants.toml`, and the comment on the line says why. A root process without
`CAP_DAC_OVERRIDE` cannot write into a directory somebody else owns.

**3. Granting the second capability works and fixes nothing.** Measured, because
it was the obvious move and would have been the wrong one:

| | `/var/lib/alo/undo` + `CAP_DAC_OVERRIDE` | `/var/lib/alo-undo`, `CAP_CHOWN` only |
|---|---|---|
| the unit starts | yes | yes |
| a person writes in their own directory | yes | yes |
| **a person can move the whole folder away** | **yes** | no |
| **a second person reaches their own directory** | **no** | yes |

Running as `alo` with no privilege at all, `mv /var/lib/alo/undo
/var/lib/alo/gone` **succeeded**: the person owns the parent, and an entry in a
directory you can write is an entry you can rename. ADR 0045's seventh term
exists because *an agent that can forget an undo can erase the evidence of what
it did*, and at full trust the agent acts with everything the person may do.
Running as a second account, `ls -d /var/lib/alo/undo/1001` → `Permission
denied`: the `0711` chosen so that every person may traverse was being defeated
one level up.

**Neither is a bug in a program, and that is the point.** The tmpfiles line is
right about the person's directory. Term 8 is right about the folder's mode. The
seam between them was wrong, and **nothing could have caught it**: nothing
created the folder and nothing took a bracket, so there was nothing to run. This
is the fault family this repository keeps meeting — *declared place versus actual
place* — and the only thing that found it was installing the unit and starting
it.

## What was measured, and where each measurement runs again

**The new location, through the real unit, caller holding no capabilities at
all** — `capsh --drop=all`, because what authorises starting a unit is who you
are and not what you hold:

```
Result=success   ExecMainStatus=0   CapabilityBoundingSet=cap_chown
/var/lib/alo-undo        mode=711  owner=root:root
/var/lib/alo-undo/1000   mode=700  owner=alo:root
/var/lib/alo-undo/1001   mode=700  owner=bea:root
```

The five properties, each by **reading the disk back** rather than by an exit
code — which matters, because an earlier version of this script printed
`chown=ok` for a refused case, `sed` having been last in the pipe:

| | |
|---|---|
| a person writes in their own directory | **yes** |
| a person can list the folder | no — `Permission denied` |
| a person can read another person's directory | no — `Permission denied` |
| a person can move the folder away | no — `Permission denied` |
| a person can delete another person's directory | no — `Permission denied` |

**The control**, the same unit with `CapabilityBoundingSet=` and
`AmbientCapabilities=` emptied: `Result=exit-code`, `ExecMainStatus=1`, and the
person's directory left `owner=root:root`. So `CAP_CHOWN` is necessary **in this
location** — which is the measurement ADR 0119 reported from a root-owned parent,
right about the capability and taken about the wrong subject. ADR 0119 now says
so, and its transcript is a measurement of where the folder actually lives.

**These are hand measurements and they do not run again.** They are a machine
start, a systemd unit and five readbacks, and no gate in this repository starts a
unit. What *does* run forever is the arithmetic and the document-reading:

- `crates/alo-giving-a-folder/src/giving_tests.rs` — 7 tests over `what_to_make`
  and `made_for`, which decide the shape and the modes on any host.
- `crates/alo-giving-a-folder/tests/only_here_may_give_a_directory_away.rs` —
  this crate is the only one in the workspace that changes a file's owner, with a
  skip for `#[cfg(test)]` modules and a control proving the skip is not greedy.
- `crates/alo-giving-a-folder/tests/the_contract_describes_where_an_undo_keeps.rs`
  — 5 tests reading the **published contract** for the path and both modes, not
  another crate's constant. Mutation-proved three ways: the folder's mode changed
  in the diagram (2 fail), the group put back (1 fails — the one written
  separately for exactly that case), the diagram line deleted (2 fail).

**And the gap this leaves, named:** nothing checks that the unit starts. A unit
that fails `226/NAMESPACE` is invisible to every gate here, which is how the
first failure above reached a green branch.

## The group is not set, and two documents moved for it

A person's directory is `0700` and **owned** by them; its group is left as it
was, which is `root`. ADR 0045 term 8 and the contract both said
`0700 <person>:<person>`; both now say `0700 <person>`, and ADR 0045 gains term
10 with the reasoning. Three measurements, in the order they bear on it:

1. **It was achievable, and the amendment says so.** `systemd-sysusers --root`
   on the image's own line for the person — `u alo 1000`, no group number —
   writes `alo:x:1000:1000` and creates a group `alo:x:1000:`; a control with no
   line produced neither. On a certified machine a person's primary group *is*
   their account number.
2. **It is false off the image.** The same `chown(uid, uid)` was refused `EPERM`
   on the development machine, where account 501's primary group is not 501.
   A privileged step taking a different road in a gate than on a machine is the
   thing to avoid.
3. **Reading the real group costs a dependency** in the one component ADR 0119
   chose *for being small*, to write a number into bits that grant nobody
   anything at `0700`.

## What the change touches

- **New:** `crates/alo-giving-a-folder` (lib, 3 modules, a binary, 3 test files,
  17 tests), `image/usr/lib/systemd/system/alo-giving-a-folder@.service`,
  `docs/decisions/0120-…`.
- **The path, 21 references in 16 files**, all in this one change because a
  constant and its call sites must land together. `docs/quirks.md` and the
  published reports under `docs/autonomy/updates/` keep the old path: a recorded
  measurement is not edited to match a later decision.
- **Stale claims this change makes false, corrected here:** `alo-letting-go`'s
  and `alo-running`'s *nothing creates the folder*, and the contract's *neither
  of those is built*. The contract now says which half is built and which is not.
- **`alo-citing` rejected the parent branch** for a sentence describing how the
  citation check was mutation-proved — it wrote a number where a decision's
  number goes. The check gains no exemption; the sentence was reworded. Fixed on
  #743 rather than here, because a forward reference to ADR 0120 could not exist
  on that branch.

## What is still owed, and by whom

- **Nothing starts the unit.** This is the next task. *An earlier draft said ADR
  0118 names two callers and both are unbuilt; the desktop lane, which owns that
  ADR, pointed out that only one is even possible — `alo-greeting`'s
  `made_an_account` refuses outright when any account exists, so the later route
  is unwritten.* Which matters more than the count: **a road reached only by
  account creation would never reach a person whose account already exists**,
  which is every person on every machine that has been switched on once. The
  caller has to be a road everybody takes.
- **Nothing takes a bracket.** `alo-turn`'s, unchanged, and ADR 0045 still puts
  it there.
- **`alo-letting-go`'s four units are still unshipped**, reported 2026-10-16.
  They must not ship later than the bracket-taking.

## What was found after the first gate passed, and fixed here

**The second call failed with `EPERM`.** Measured by installing the unit and
starting it twice:

```
alo-giving-a-folder: /var/lib/alo-undo/1000 could not be made, so nothing of
  this person's files can be kept for an undo: Operation not permitted (os error 1)
```

Setting a mode needs **ownership or `CAP_FOWNER`**, the unit holds only
`CAP_CHOWN`, and by the second call the directory belongs to the person. So
*puts the mode and the owner back on every call* — written in this crate's
rustdoc, in ADR 0119, in ADR 0120 and in the unit's own comment — **was false on
the day it was written**.

The fix keeps one capability: take the directory back, set the mode, give it
away again. `CAP_CHOWN` permits giving a file to anybody including oneself.
Measured in isolation with `cap_fowner`, `cap_dac_override` and
`cap_dac_read_search` dropped — a `0777` directory owned by the person was
narrowed to `0700` and returned — and the control with `cap_chown` **also**
dropped was refused at the first step and left the directory as it was. Then
through the real unit: first, second and third starts all `Result=success`, a
hand-widened `0777` back to `0700 alo`, `CapabilityBoundingSet=cap_chown`.

**Why no test caught it, which is the part worth keeping.** There was a test
called `signing_in_again_narrows_a_folder_somebody_widened`, and it passed. It
widens `under` — the folder **above** the person's directory, the one with no
owner, which this program never gives away and can therefore always `chmod`. The
directory that broke is the one *with* an owner. The right measurement of the
wrong subject, for the third time in this one change.

What now runs forever: `a_second_call_puts_a_widened_mode_back`, which widens
the person's own directory; and `the_decision_to_take_it_back_covers_both_owners`,
which holds the two-number decision to both answers — because *always take it
back* would work on a machine and fail in every gate, since a non-root process
cannot give a file to root, and *never* is the bug that was just fixed. **The
repeat road with a different owner can only run as root**, so that half is the
hand measurement above and is not claimed as a test.

## The gate

`cargo fmt --all --check` clean. `cargo clippy --workspace --all-targets
--all-features -- -D warnings` clean. `cargo doc --workspace --no-deps
--all-features` with `RUSTDOCFLAGS=-D warnings` clean. `cargo test --no-fail-fast`
over `alo-giving-a-folder`, `alo-letting-go`, `alo-changing-undo`,
`alo-remembering`, `alo-installing`, `alo-sessiond`, `alo-image` and
`alo-citing`: **0 failures across every result line**, summed rather than read
off the last one.

**What this did not run, stated rather than left in the method.** `alo-running`
is not in that list. Two of its tests — `a_command_reaches_what_its_box_was_given`
and `the_kernel_refuses_what_its_box_was_not_given` — need a writable
`/sys/fs/bpf` and fail as the user. Measured both ways on the same already-built
binary: **2 failed as the user, 2 passed as root**, so they are the known
root-only set and not this change, whose only edit to that crate is one word in a
doc comment. CI is the required check and runs the whole suite.
