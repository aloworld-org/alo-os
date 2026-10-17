# A person signing in gets their folder

**Branch:** `task/mac/a-person-signing-in-gets-their-folder`, stacked on
`task/mac/one-helper-gives-a-person-their-folder` (#744), which is stacked on
#743
**Date:** 2026-10-17
**Decisions:** [ADR 0121](../../decisions/0121-the-thing-that-opens-a-session-makes-the-persons-folder.md)
(written here), amending [ADR 0024](../../decisions/0024-what-a-person-signs-in-at.md)
by one sentence

## What this change is

**The caller #744 said was owed.** `alo-giving-a-folder` shipped on the image
and nothing on the machine ever asked for it, so the folder was still absent at
runtime for every person. `alo-sessiond` now starts
`alo-giving-a-folder@<account>.service` when it opens a session.

## The judgement, in one sentence

**This makes an existing promise true rather than adding one**: ADR 0045 term 8
has asked since 2026-09-21 for a privileged step that makes a person's folder at
sign-in, and this is that step being called — no new promise to a person, and
nothing a person can now ask for that they could not before.

It does amend a charter, which is why there is an ADR rather than only a diff.

## Where the call went, and why not where it obviously should have

The obvious caller is account creation. It is wrong, and the reason came from
reading the code rather than from reasoning:

```rust
// crates/alo-greeting/src/greeting.rs:208
if self.accounts.how_many() != 0 {
    return Greeted::Refused(crate::words::ALREADY_HAS_AN_ACCOUNT.key());
}
```

`made_an_account` is the first-account road and **refuses to be anything else**.
So an account road reaches nobody who already has an account — which is every
person on every machine that has been switched on once. Covering them would need
a second caller walking `/etc/passwd`, because `Accounts` has no enumerator and
`numbers`' own rustdoc says it "is not an enumeration door".

**Everybody signs in; not everybody is created.** So the call goes in
`Opening::heard`, which is one call site where the account road is two, needs no
new surface on `Accounts`, and reaches a person whose account is five years old.

*This was found with the desktop lane, which owns ADR 0118.* It corrected my
claim that ADR 0118 names two callers both unbuilt — only one is possible — and
raised the `/etc/passwd` reconcile; I found the sign-in road made the reconcile
unnecessary. The exchange is in ADR 0121's own text, including a bullet of mine
the desktop lane showed contradicted my own measurement, which is now in the
record in its weaker, true form.

## What it cost, stated plainly

`alo-sessiond`'s lib doc said:

> **No path reaches it.** It is handed nothing to open, write, or run.

*run* is now untrue. The sentence is rewritten rather than quietly left, and the
new capability is bounded by four things, each a test:

| | held by |
|---|---|
| one unit, named by a constant | `the_unit_is_named_by_a_constant_and_instanced_by_a_number` |
| instanced by a `u32`, so no text reaches a unit name | the same test, reading the parameter's type |
| three methods, none that stops, kills, masks, reloads, enables or writes | `this_component_says_nothing_to_systemd_that_changes_it`, over twelve verbs, **with a control proving the filter can find one** |
| no new capability | measured, below |

It still does not authenticate, does not draw, and is handed nothing to open or
write.

## Why `alo-brokerd` was not used, which was the strongest alternative

The desktop lane pointed out that `alo-brokerd/src/units.rs` **already** starts
units, with a closed enum of names and a refusal for every method that is not
read-or-start — so the mechanism and the discipline exist in a component we have,
and using them would leave ADR 0024 alone. It does know uids: its own table
lists a `logins` module.

Rejected **on where it would be written down**. `alo-brokerd`'s surface is verbs
the agent asks and the person approves, and `docs/contracts/kept-undo-folder.md`
says of this folder: *"There is no verb over this folder and there is not going
to be one… an agent that can forget an undo can erase the evidence of what it
did."* A third variant beside `Carriers` would not be a verb on the day it
landed; the risk is the lane that opens that file next year to add one. The
record is the cheaper thing to amend.

## What was measured, and where each measurement runs again

**By hand, on a machine, and these do not run again** — no gate here starts a
unit or talks to a bus:

*The port against the real unit.* A program calling `TheMachinesFolders::made_for`
directly, run under **exactly `alo-sessiond`'s own settings**:

```
User=root  CapabilityBoundingSet=  AmbientCapabilities=  NoNewPrivileges=yes
Result=success
OK: the port says account 1000 has their folder
/var/lib/alo-undo/1000  mode=700  owner=alo
```

*The control, which refuses.* The same program as the person, not root:

```
ERR: systemd would not start alo-giving-a-folder@1000.service for account 1000:
     org.freedesktop.DBus.Error.InteractiveAuthorizationRequired
/var/lib/alo-undo/1000: No such file or directory
```

So *what authorises starting a unit is uid and not capability* has both answers,
and the D-Bus error name this crate decides on is the one that really arrives.

*Two accounts.* 1000 and 1001, each `0700` and owned by the right person inside a
`0711 root:root` folder.

**Three faults in my own measuring, found and named** — because a measurement
that is wrong in a direction that reads as a pass is worse than none:

- `systemd-run --collect` then `systemctl show` reported the **default**
  capability bounding set, not the unit's, because the unit no longer existed.
  It printed forty capabilities for a unit that had none.
- My probe printed `ERR:` and **exited 0 either way**, so a control that said
  `status=0` said nothing at all. Fixed by reading what it printed.
- The first run failed `216/GROUP` because the `alo-greeter` group does not
  exist in the development virtual machine. A fixture gap, not a product fault,
  and it would have read as one.

**What runs forever**, in the gate, on any host:

- `opening.rs`: `a_folder_that_was_not_made_refuses_nobody`,
  `the_folder_is_asked_for_once_for_one_knock`,
  `a_refused_knock_asks_for_no_folder` (over two kinds of refused knock) and
  `a_second_knock_asks_for_no_second_folder`.
- `tests/this_component_only_starts_one_unit.rs`: four tests, including a control
  that the filter can find a forbidden verb and a check that the file being read
  is the one that talks to systemd.
- `tests/the_unit_this_starts_is_on_the_image.rs`: four tests reading the
  **image's own units directory**, so `THE_UNIT` cannot name a unit that does not
  ship — with a control asserting two units that certainly do ship, so *the
  directory moved* and *this unit is absent* cannot be confused.

**Mutation-proved.** Adding `return Answered::Refused(NOT_OPENED.key())` to the
folder's failure road fails exactly one test —
`a_folder_that_was_not_made_refuses_nobody` — and no other. That test exists
because the obvious way to write that call site is one line from being the
opposite.

## A failure does not refuse the sign-in

The only step in `Opening::heard` whose failure is not a refusal. Three reasons
in ADR 0121; the one that decides is `CLAUDE.md` law 5 — refusing would take the
machine away from its owner to protect a feature nothing yet writes to.

## Also here

- **`machine.rs` was split.** Adding systemd's manager to the file holding
  `logind` took it to 475 lines with two reasons to change, so
  `making_a_folder.rs` is its own file. Law 4, in the change that discovered it.
- **No new dependency**: `alo-sessiond` already carries `zbus` for `logind`.
- **No new word**, because nothing new is said to a person — so the vocabulary
  snapshot is untouched.
- **The unit's name is read off the image**, not imported from
  `alo-giving-a-folder`. A constant agreeing with another crate's constant passes
  while both are wrong.
- **17 construction sites** of `Opening::at` updated for the third type
  parameter, in `alo-sessiond` and `alo-greeting`.

## What is still owed

- **Nothing takes a bracket.** `alo-turn`'s, unchanged. What this makes true is
  that the folder exists for every person who signs in, which is the
  precondition the bracket was blocked on — not the bracket.
- **`alo-letting-go`'s four units are still unshipped.** I installed all four on
  a machine and they load, start and exit 0, and the timer and path activate. So
  shipping them is packaging rather than repair.
- **The remover has never run over a folder that has something in it.** On a
  non-`btrfs` machine it returns at its first question and says *"this machine
  keeps nothing an undo could put back"* — correct per ADR 0045 term 6, and
  **unmeasurable in a way that reads as a pass.** The desktop lane has a `btrfs`
  machine booting and has a script of mine with its pass and fail readings.

## The gate

`fmt` clean. `clippy --workspace --all-targets --all-features -D warnings`
clean. `doc --workspace --no-deps --all-features` with `RUSTDOCFLAGS=-D warnings`
clean. `test --no-fail-fast` over `alo-sessiond`, `alo-greeting`,
`alo-giving-a-folder`, `alo-image`, `alo-citing` and `alo-saying`: **0 failures
summed across every result line**, not read off the last one.

**What this did not run, in the sentence rather than only in the method.**
`alo-running`'s two BPF tests need a writable `/sys/fs/bpf` and fail as the user
— measured both ways on the same already-built binary, 2 failed as the user and
2 passed as root — and this change does not touch that crate. CI is the required
check. This is a head status: these gates passed on this tree, not on the tree
that becomes `main`.
