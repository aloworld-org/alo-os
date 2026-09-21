# ADR 0061 — A locked screen offers a road to the greeter, and nothing else new

**Status:** accepted, 2026-09-21. Written by task 8 of
`docs/autonomy/v0-5-the-session-and-the-displays-plan.md` (*Switching to another
person at a locked screen*), whose code is built on it. The plan asked for this
record in so many words: *whichever way it goes, the argument is written down
before the code is.*
**Date:** 2026-09-21
**Proposed by:** the session-and-the-displays workstream
**Context:** task 1 of that plan, which published what a lock screen may show
and is what this record widens; task 5 of it, which built *switch user* and
found it unreachable from a locked screen;
[ADR 0024](0024-what-a-person-signs-in-at.md) (what a person signs in at, and
that the greeter asks for a name rather than offering a list of them);
[ADR 0001](0001-the-capability-model.md) §5 (approval is the signed-in person's
act, which is why a waiting change is still refused at a lock screen);
[ADR 0018](0018-the-boundary-is-loaded-by-a-loader-not-by-the-agent.md) (one
privileged component, which is why nothing here opens a session);
[ADR 0016](0016-the-organisation-bounds-and-the-person-chooses.md) (the person
chooses on their own machine); `crates/alo-locking/src/screen.rs`,
`crates/alo-leaving/src/switching.rs`, `docs/contracts/lock-screen-rendering.md`,
`docs/features.md` v0.5 *Session management: log out, switch user, lock, and
reopen what was open*.

## The question in one line

**Two people share one machine. The first has locked it and gone. May the
second reach the sign-in without the first coming back to type their password?**

Today they may not. `alo_leaving::switching::asked` locks this session and then
makes the `alo_greeting::Standing` a greeter draws from, and a screen that is
**already** locked is refused — because task 1 published what a lock screen may
show, it is four things, and a road to the sign-in is not one of them.

So a household sharing a machine has the first person unlock, in front of the
second, before the second can sign in at all. That is the wrong way round. The
person who has to prove who they are is the one who wants in, not the one who
has gone — and the thing a rule like that actually teaches a household is to
stop locking the screen, or to share the password. **A lock nobody uses
protects nothing**, and that cost is paid on the same surface the rule was
written to protect.

## §1 What task 1 promised, said exactly

> nothing that would reach a person is shown on the lock screen except the
> time, the lock image, the battery, and that the machine is locked

Read closely, that is a rule about **this session**: what is shown *on* the
lock screen, which is what `alo_locking::LockScreen` is the type of. Every one
of the leaks it was written against is a fact about the person behind the lock —
a notification's first line, the document they were writing, the agent's last
question, a change waiting to be tapped. The reasons given for it are reasons
about **disclosure**, and every one of them still holds after this record.

They are not, on their own, reasons about **affordance**. A lock screen already
offers one thing that is not on that list of four: a name and a password, which
is the way back in. Nobody has ever read the rule as forbidding that, because it
discloses nothing — a password box on a locked machine tells a stranger only
that the machine is locked, which is the fourth thing it already says.

This record makes that distinction explicit rather than leaving it implied, and
then applies it once more.

## §2 The options

- **(A) The rule stands. A household is told to unlock first.** Honest, and it
  is what shipped. It costs the first person's password, typed in front of the
  second, every time somebody wants to check their mail — or, much more likely,
  it costs the lock. **Refused**, because the failure it produces is *people
  stop locking their screens*, which is worse on this exact surface than
  anything it prevents.
- **(B) The lock screen shows who else has an account here, and the second
  person picks themselves.** This is what most desktops do, and it is the one
  this record refuses outright. A list of the accounts on a machine is a
  disclosure about people who are not there, made to a stranger at the desk, and
  it is exactly what task 1 was written against. ADR 0024 already decided that
  this product's greeter asks for a name rather than offering a list, for the
  same reason, at a screen with no lock behind it at all. **Refused, and the
  refusal is the load-bearing half of this record.**
- **(C) A road to the greeter, which discloses nothing.** **Chosen.** The lock
  screen may offer to hand the screen to the sign-in. What the sign-in stands at
  is `alo_greeting::Standing`, which on any machine with an account on it is the
  single value `SignIn`: ask for a name and a password, and say nothing above
  them. It names nobody, counts nobody, and is derived from the machine's
  accounts rather than from the session behind the lock.

## §3 Why (C) discloses nothing, stated as a property rather than a promise

`alo_locking::SomebodyElse` — the type this decision is held as — **has no
generic parameter and carries no session**. That matters, because the seat it
comes from is generic over what a notification is and holds every notification
that arrived while the machine was locked. A value with nowhere to put one
cannot leak one, whatever a later change does to the shell.

What it carries is a `Standing` and, on the one refusal below, the lock
screen's own existing sentence. `Standing` has two values and both are facts
about the machine's account store rather than about anybody's session: *make an
account*, or *sign in*. The second is the value every locked machine produces,
because a machine with a locked session on it has at least one account, and it
is the same value a cold machine of this product shows every morning.

And the road is `&self`. It cannot end the session, sign it out, unlock it or
take anything it holds, because it is handed a shared reference to the seat and
gives back a value with no room for one — task 1's clause 8 kept by the
signature rather than by a paragraph.

## §4 The one refusal, and it is a real one

**A machine whose accounts stand at *make an account* is refused**, in the lock
screen's own existing sentence.

`Standing::of` answers `MakeAnAccount` when the store holds no account at all.
On a cold machine that is ADR 0024's first-boot screen and it is right. On a
**locked** machine it would be an offer, made to whoever is standing at the
desk, to create an account on somebody else's machine and sign in on it — the
store having been emptied underneath a running session, by an administrator, by
a restore, or by a disk that came back wrong. It is rare, and it is the shape
of thing this product is judged on.

So the road hands over `Standing::SignIn` and nothing else. `MakeAnAccount` at a
lock screen is `alo_locking::NotWhileLocked`, the same answer the agent's key
and a waiting change already get, saying the same thing the lock screen already
says.

## §5 `LockScreen` still has exactly four things

The road is **not** a field of `alo_locking::LockScreen`, and that is a
decision rather than an omission. `LockScreen` is the type of *what is shown
about this session at this moment* — the time, the lock image, the battery, the
locked sentence, and the egress lamp that names nothing. It gains nothing here,
its `compile_fail` examples stand, and `docs/contracts/lock-screen-rendering.md`
continues to describe its public view correctly.

What a lock screen may **offer** is a separate question from what it may
**show**, and this record answers only the first. Keeping the two in separate
types is what stops the next person who wants a fifth thing from arguing that
the line has already moved: a field on `LockScreen` is a disclosure and still
needs the argument task 1 made; a road that carries no session is not.

## §6 What this is not

- **Not a second authenticator.** Signing in is `alo_greeting::Greeting::signs_in`
  and there is no other road. This record adds a value that says *ask for a name
  and a password*; it checks nothing, holds no credential and reaches no store.
  `crates/alo-locking/tests/unlocking_is_signing_in.rs` already reads this
  crate's whole source for a second road through the lock and keeps doing so
  over the file this decision adds.
- **Not a way past the lock.** The person behind it is still the only one who
  unlocks it, still by their own password, still through the greeting knocking
  at their own session's number. A second person signing in arrives at their own
  session, and the first person's is still locked when they come back.
- **Not something an agent may ask for.** No verb reaches it, and no crate an
  agent's request is carried out in depends on `alo-locking`. A test in that
  crate reads every manifest in the workspace for it.
- **Not a new requirement on the machine.** Opening the second session is
  `alo-sessiond`'s, untouched here, and handing a `Standing` to a greeter is
  what `alo_leaving::switching` has done from an unlocked desktop since task 5.
  What a locked screen may offer changes; what the machine underneath has to be
  able to do does not.
- **Not a list, a count, or a hint.** Nothing on the way names who was signed
  in, what was waiting, or what was open. What was open is
  `alo_leaving`'s keeping and is never consulted: `alo-locking` does not depend
  on that crate and cannot read its file.

## What this record does not decide

- **The drawing.** Where the road sits on a locked screen, what it looks like
  and how it is reached by somebody using a screen reader are the shell plan's,
  from `alo_locking::words::SOMEBODY_ELSE` and the type here. Nothing in
  `alo-locking` draws.
- **`docs/contracts/lock-screen-rendering.md`.** It describes what the shell's
  nested lock surface does today, which this change does not alter. It gains the
  road in the change that draws one, and that change is the shell plan's.
- **Whether a machine may hold two sessions at once, and how a person returns
  to theirs.** That is `alo-sessiond`'s and the shell's, and it was already the
  condition of *switch user* before this record existed.
- **Anything about an organisation's machine.** ADR 0016 stands: an
  organisation that does not want a second person signing in at this machine
  says so by which accounts exist on it, which is the same lever it always had.
