# The guard looks for an owner-changing chown

**Date:** 2026-10-16
**Lane:** Mac
**Branch:** `task/mac/the-guard-looks-for-an-owner-changing-chown`

## What changed, for a person outside this repository

Nothing they can see. A test that exists to stop one specific bug reaching a
booted machine had a hole in it, and the hole is closed.

## What happened

ADR 0045 term 8 says a privileged step makes `/var/lib/alo/undo` and each
person's directory inside it when they first sign in. I built that step in
`alo-sessiond`: it runs `User=root`, and it is what learns a person exists.

It cannot work. `alo-sessiond.service` says `CapabilityBoundingSet=` with
nothing after it and `NoNewPrivileges=yes`, so the process holds **no
capabilities** — and giving a directory to another account needs `CAP_CHOWN`.

**Measured, 2026-10-16**, rather than read off a manual page:

```text
as root with capabilities      chown 65534 victim   owner becomes 65534
as root, capsh --drop=all      chown 65534 victim   Operation not permitted
                                                    owner stays 0
```

*(My own measuring script printed `chown=ok` on the second line, because `sed`
was the last stage of the pipe and the exit code was its. The **owner readback**
is what settles it — three fields beat one, which is this repository's own rule
about reports.)*

## What stopped it, and why that is not reassuring

`tests/this_crate_cannot_do_what_its_unit_forbids.rs` failed. That test exists
for exactly this: its header records that on 2026-10-14 this crate was taught to
start a desktop with `Command::uid()`, compiled, passed clippy, rustdoc, 72
tests and CI, merged, and then failed on a booted machine with *Permission
denied*.

**But it caught me by accident.** Its matcher is `[".uid(", ".gid(",
"setuid"]`, and what it fired on was a `.uid()` **read** in a neighbouring test
file — a false positive. The real fault, `rustix::fs::chown(at, Some(uid),
None)` in production code, is not in that list at all.

So the guard had a false positive that happened to land on a real fault, and a
false negative for the fault itself. Without the accident, the same story its
own header tells would have repeated three days later.

## What the fix had to avoid

**A bare `chown` cannot be the thing looked for**, because this crate makes a
legitimate one. `unix.rs` does `chown(path, None, Some(group))` and explains
why it may: changing **only the group** is permitted without any capability
when the process is already in that group, and that is how a socket root
created becomes a socket the sign-in surface can connect to. Forbidding `chown`
outright would fail on correct code, which is how a guard gets weakened by
whoever meets it next.

So what is looked for is the **owner argument**: a `chown` whose owner is
anything but `None`.

## How it is held to that

`the_owner_check_tells_the_two_chowns_apart` — three forbidden shapes, two
permitted ones including `unix.rs`'s real call, and three ordinary lines that
merely say *owner*. A source test that finds nothing has two
indistinguishable outcomes, and the permitted form here is the same function as
the forbidden one with a different argument, so the control matters more than
usual.

**And it is mutation-proved.** The line I nearly shipped was pasted into
`unix.rs` and the guard failed, naming it:

```text
crates/alo-sessiond/src/unix.rs:119
  rustix::fs::chown(at, Some(rustix::fs::Uid::from_raw(owner)), None)?;
```

with the legitimate call two screens above it still passing.

## Measurements: where each one runs again

| what | where it runs again |
|---|---|
| no source in this crate changes an owner while its unit grants nothing | every build |
| the check tells an owner change from a group change *(control)* | every build |
| the check does not fire on ordinary lines *(control)* | every build |
| `chown` without capabilities is refused | **once**, 2026-10-16, `capsh --drop=all` — nothing re-runs it |

## What this does not do

**It does not build term 8's privileged step**, and that work is now blocked on
a decision rather than on effort. The module, its tests and a contract test
against `docs/contracts/kept-undo-folder.md` are written and green, and they
are not in this change, because a module that cannot set an owner cannot do
what it says.

**Nothing on this machine may chown.** Searched: no unit in `image/` mentions
`CAP_CHOWN`, and the only unit holding any capability is `alo-boundaryd`
(`CAP_BPF CAP_SYS_ADMIN`), which is ADR 0018's loader. So *which component may
give a directory to a person* is an open question and not a wiring choice.

The options each cost something real, and the choice is the owner's:

1. **Grant `CAP_CHOWN` to `alo-sessiond`.** Smallest change, and it ends the
   claim that unit makes by granting nothing — which this very guard's message
   calls a decision and not a fix.
2. **A separate privileged oneshot**, started at sign-in, holding the one
   capability. Keeps `alo-sessiond` as it is; adds a unit and a trigger.
3. **The person's own session makes their directory**, with the parent's mode
   allowing it. No capability anywhere — and it lets one person pre-create a
   directory named for another, which the sticky bit does not prevent.

ADR 0064's *snapshot before each run* waits on whichever is chosen.
