# A sealed box runs a command

**Date:** 2026-10-16
**Lane:** Mac
**Branch:** `task/mac/a-sealed-box-runs-a-command`

## What changed, for a person outside this repository

The agent can now be allowed to run a command on your machine inside a box the
kernel locks. The box sees the folders you gave it and nothing else. If the
command tries to read anything else, the kernel refuses it — not our code, the
kernel.

This is the default level of the three in ADR 0064. It is the mechanism only:
nothing asks you yet, because the setting and the surface are not built.

## What it was waiting on, and why that was wrong

`docs/features.md` said the sealed box *needs the kernel boundary*, and three
lanes including this one read that as *the boundary is not built*. It is built,
and has been: `alo-bounding` is 5,278 lines, ten BPF LSM hooks are attached,
paths become a bound, network destinations are carried in the same map entry,
and `alo-agentd` already drives a full turn through it.

**What was actually missing was one thing.** `Turns::doing` runs work *"on this
thread, in this process, with nothing started"* — its own words. A run is a
child process. That is the whole of the gap, and it was described as *blocked
on the kernel boundary* for weeks.

## The decision: ADR 0117

The obvious move is a *run a program* method beside `Turns::doing`. **That is
the one thing `alo-bounding` must not have**, and it is tested, not promised:
`a_turn_is_this_thread.rs` reads that crate's own source and fails on `Command`,
`fork(` or any spelling of `exec`, with a positive control. Law 2 is the reason,
and the crate says so.

So the box is a new crate, `alo-running`, which uses `alo-bounding`'s control
group and boundary without changing it. The guarantee stays; the `exec` lives
where somebody chose it rather than where a closed verb list is served.

**I reported the opposite to the owner first** — that adding the method would be
purely additive. It would have broken a deliberate architectural guarantee and
failed an existing test. Caught by reading the crate's own `lib.rs` before
writing code.

## How a command gets inside its box

```text
1. the boundary is asked whether it is still there   (Boundary::in_place)
2. a control group is made for this run              (Cgroup::made_under)
3. { its id -> the places } goes into the map        (Boundary::bound)
4. alo-enter-the-box starts, told the group and the command
     it writes "0" into cgroup.procs  <- it is now inside
     it execs the command             <- the command starts inside
5. the command is waited for
6. the entry is removed, 7. the group goes
```

The window between a process existing and the kernel knowing its box contains
**one program of ours and no program of anybody's choosing**. The command starts
at `exec`, which is after the admission.

**It is a helper program rather than a `pre_exec` hook because `unsafe` is
forbidden.** The workspace sets `unsafe_code = "forbid"`, which no attribute can
lift; `pre_exec` is unsafe and `CommandExt::exec` is not. `clone3` with
`CLONE_INTO_CGROUP` is the better mechanism and is rejected for the same rule,
with the condition for revisiting it written into the ADR.

## Three things measured that reasoning got wrong

Each of these would have shipped as a bug. None was found by a passing test.

**A traversal hole in my own code, found by my own test.** The helper checked
its control group with `Path::starts_with`, which is purely lexical:
`inside/../elsewhere` starts with `inside`, so the refusal never happened and
the write was attempted outside the service. A symlink defeats it identically.
Both paths are now resolved before comparing. Two tests cover it, and I
mutation-tested them: restoring the lexical check fails both and passes the
other twelve.

**A domain control group cannot sit beside a threaded one.** `Turns::under`
makes its `home` threaded, which makes the root above it `domain threaded`; a
box made beside it comes out `domain invalid` and **refuses a process
outright**. A runner built on the obvious placement looks correct, passes
review, and refuses every command. So `alo-running` takes the folder its boxes
live in as an argument instead of deriving it from `Turns`.

**A dynamically linked command needs its libraries in the bound.** `exec`
happens after the admission, so the dynamic loader and `libc` are opened inside
the boundary. A box granted only the project cannot run `/bin/sh` — it dies
before its first instruction. Two of a run's four places go on being able to
start at all. This is `CLAUDE.md`'s converter-that-died-on-eleven-libraries
story one layer down.

All three are in `docs/quirks.md` with the commands and the output.

## What proves it works

`crates/alo-running/tests/the_kernel_refuses_what_a_run_was_not_given.rs`, on a
real kernel, as root:

```text
a_command_reaches_what_its_box_was_given   this is what the file says
the_kernel_refuses_what_its_box_was_not_given
    /bin/cat: /tmp/alo-running-1964847-refused/what-it-says: Permission denied
```

Same command, same box, one line different in the bound. The first is the
control: without it a box that refused everything would satisfy the second.

The fixture is `alo-bounding`'s own `on_this_kernel/mod.rs`, included by path
rather than copied, because its own documentation says why it is shared — *a
fixture that drifted between three files would be three different machines being
tested*. **The cost is a coupling across crates**, named in the test's header: a
lane that moves that file breaks this test. A copy was the alternative and would
drift silently, which is worse because nothing announces it.

## Measurements: where each one runs again

| what | where it runs again |
|---|---|
| input validation, five refusals | `cargo test -p alo-running`, every build |
| the admission happens before the command | same |
| traversal and symlink refusals | same |
| only two files start a program, and which | same |
| a command is refused outside its bound | `--profile boundary` after a merge, on the machine that can run it; **excluded from the merge gate** |
| the helper is installed where the constant says | the image build's `COPY`, which fails if absent |

**The kernel test does not run in the merge gate, and that is deliberate.** It
needs root, a BPF filesystem and `bpf` among the modules the kernel *started*,
and it fails loudly rather than skipping — so on a runner without them it goes
red, not green. CI has none of them.

So it is excluded by binary name from the merge gate, beside the three packages
already excluded for the same reason, and it joins the `kernel-boundary` test
group so it is serialised with them when it does run: `--profile boundary`, on
`main` after a merge, on the machine that can run it.

**Only that one binary is excluded, not the package.** The other sixteen tests
run everywhere, every build. Verified rather than assumed: the merge gate's
filter selects 16 tests from this crate and 18 without the exclusion, and the
two it drops are exactly the two kernel tests.

It was run by hand in the Lima VM on 2026-10-16 and the output is above.

## What is still owed

- **The three protections ADR 0064 keeps on at every level are the caller's**,
  and nothing calls this yet: the record, the snapshot taken before each run,
  and the egress indicator. `WhatHappened` exists so a caller can write a
  record; this crate writes none. `lib.rs` says so where somebody wiring it up
  will read it.
- **Where a box's control group lives on a booted machine**, which depends on
  what systemd delegates to the service and has not been measured on an image.
- **The setting and the surface**: nothing asks a person anything.
