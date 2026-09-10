# A session that really ended

- Date: 2026-09-10
- Workstream: model selection and configuration (`alo-secrets`)
- Contributor: Claude Code
- Task: What a credential does when a session really ends
- Status: **the three cases are measured.** One half of case 1 is not, and this
  says which.

## Why this sat unscheduled, and why that was avoidable

Two premises in the plan were wrong, and neither had been checked.

**There is no `sshd` on this machine.** The binary is absent and the unit is
`not-found`. Every case was written as *reachable with one or two `ssh` logins*,
so all three were silently blocked on installing a network service — a
machine-wide change with its own handoff. Nobody had looked.

`su` reaches the same sessions. `pam_systemd` is in `common-session`, so a login
through PAM registers with `logind` and gets a `/run/user/<uid>` and a user bus,
with no listener anywhere. Nothing here installs one, and **no case needed a
second seat** — which was the other premise the plan had already corrected on
paper and this confirms in practice.

## What the machine actually does

| What happened | `/run/user/<uid>` and the bus | `TheBus::found` |
|---|---|---|
| **One login, logged out**, no lingering | both gone | `Unavailable` |
| **Logged out with `enable-linger` on** | both **survive with nobody signed in** | `Ok` |
| **Two logins, one ended** | both stay; the other login holds them | `Ok` |

Case 1 is the one that matters for `connections_come_and_go.rs`: the fixture's
stopped private bus **is** representative of a real logout at the level the
daemon branches on. `alo-agentd` asks `TheBus` before it opens any store, so a
bus that is gone is a lookup that never happens.

Case 2 is the one a daemon's author would not predict from case 1, and **it is a
policy rather than a bug**. With lingering on, the person's credential store is
reachable while nobody is signed in at all. That is what `enable-linger` is for;
whether alo OS wants it is the owner's question, and this workstream deciding it
quietly would be the wrong kind of tidy. ADR 0022 records it as a decision
outstanding.

## Three mistakes of mine, all caught by tests failing

**`su` takes a name, not a number.** `su 1000` is *user 1000 does not exist* —
a sentence about an account, not a uid. The uid stays what the file is about,
since `/run/user/<uid>` and `TheBus` are both keyed by it; the name is looked up
once so the two cannot drift apart in a hard-coded pair.

**`terminate-user` is not a logout.** It removes the user manager too, which is
exactly what lingering exists to prevent — so case 2 would have been answered by
definition instead of measured. Logging out means ending the *sessions*;
whether the manager and its bus go with them is `logind`'s decision, and that
decision is the subject.

**The `manager` session is not a login.** `su` produces two sessions: a login
and a `manager`. Counting the manager as *still signed in* made logout look like
it never completed, and the wait timed out. Told apart by class now.

A fourth, of ordering rather than concept: killing the shell **before** ending
the session orphans it into a session nobody is ending, and then the logout has
nothing left to end. Ending the session is what takes its processes with it.

And a fifth, which only appeared when the supervisor ran each case **on its
own** after a full workspace run. A session listed a moment ago can be gone by
the time its class is asked for, and `loginctl` then answers with nothing — and
an empty class was being counted as a login. *Logged out* flickered back to
*still signed in* between two polls and the wait never finished: a test failing
on the gap between two `loginctl` calls rather than on anything the machine did.
A login is now a session whose class was **read** and is not the manager's.

A sixth, and the one that actually mattered, showed up as **a case that passed
one run and failed the next on the same machine and the same code**. A
terminated session does not leave the list: it sits there in state `closing`
while its processes are reaped, which Ubuntu's `KillUserProcesses=no` stretches
out. Counting a closing session as somebody signed in made a logout that had
plainly happened look like it never did. Somebody is signed in when a session is
**up** — class read, not the manager's, and not `closing`. Three consecutive
suite runs and two consecutive supervisor runs since.

That one is worth the space because the first fix was wrong: an empty class from
a vanished session is a real flicker and worth handling, but it was not what
made this intermittent, and stopping at it would have published a test that
failed for somebody else next week.

Worth recording beside it: **the bus does not go the instant the login ends.**
The user manager stops on its own schedule, several seconds later. Case 1 waits
for it, bounded, and that wait *is* the assertion — if the bus never goes the
test fails loudly rather than passing on a technicality. Asserting immediately
after the logout would have reported the opposite of the truth.

## The boot id, which is why any of this is evidence

An earlier look appeared to show sessions surviving a logout. They had not: the
distribution had restarted between the two observations, and **a restart removes
`/run/user/<uid>` exactly as a logout does**. Every case now records the boot id
either side of what it did and refuses to draw a conclusion across a restart.

Without that, all three cases would pass on a machine that never ended a session
at all.

## Machine state, before and after

Nothing is left behind. Lingering is turned on for the length of one case by a
value whose `Drop` turns it off again, so a failure or an early return cannot
leave it on. Checked after the run: no lingering, no sessions for that uid, only
`/run/user/0`, no temporary directories. The account signed in is the one alo
OS's own image creates — nothing here invents an account on somebody's machine.

The suite is `#[ignore]`d and run deliberately, `--test-threads=1`, because
every case signs the same person in and out and two at once would take each
other's sessions away.

## What this does **not** cover

**A keyring handle held across the logout**, which is the other half of case 1's
expectation in the plan.

Putting a real Secret Service on the person's *session* bus needs a process
running **as them**: root does not complete the D-Bus handshake on their bus —
measured, after an empty keyring log sent me looking in the wrong place — and
`alo-keyring-fixture` deliberately starts a private bus of its own, which is the
very thing this file exists to stop standing in for a real one. Worse, a process
of theirs is **killed by the logout being measured**, so the test needs a helper
that reports across the event rather than one that holds a handle through it.

That is a piece of work of its own. It is not pretended here, the plan says so,
and ADR 0022 says so.

## Proposed integration updates

**CHANGELOG.md** — nothing user-visible.

**ROADMAP.md** — no tick; the task is not finished.

**docs/autonomy/QUEUE.md** — real-session logout is measured at the bus for all
three cases; the held-handle half remains, and lingering raises a policy question
for the owner.

**docs/autonomy/STATE.md** — `alo-secrets` has a deliberate, `#[ignore]`d suite
that signs a person in and out of this machine; ADR 0022 carries the table.
