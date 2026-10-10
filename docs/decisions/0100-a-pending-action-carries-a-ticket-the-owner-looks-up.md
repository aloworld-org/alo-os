# ADR 0100 — A pending action carries a ticket the owner looks up

**Status:** **proposed, 2026-10-10**, by the Mac lane, for the owner and for the two
lanes whose crates it touches.

## What this is for

[ADR 0087](0087-the-signed-in-session-is-kept-by-one-owner-and-notifications-ask-it-twice.md)
settled that the signed-in session is kept by one owner and that a notification is
asked about **twice** — once when it is shown, once when its button is pressed. The
owner's ruling of 2026-10-04 then named what the second ask has to survive:

> **Logout:** destroy that authenticated session and invalidate its pending actions;
> the seat owner remains.
>
> … Reusing the same UID must not revive an old session's buttons.
>
> Bind **Put it back** to the originating session and window.

This record is how the second ask is built, and it exists because the obvious way
cannot be made to work.

## The obvious way cannot express the rule

Bind the action to the session and compare the two at the press. Measured on
`origin/main`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session { name: String, uid: u32 }
```

A session is a name and a number, and **both are the same at every sign-in**. So two
sign-ins by one person produce values that are equal. *Reusing the same UID must not
revive an old session's buttons* has nothing to compare: a pending action naming *the
session for uid 1000* validates perfectly against a brand-new session for uid 1000.

It is not that the check is missing. **It is that the type cannot hold the
difference**, and every layer above it can be correct while that is true.

## Decision

**A pending action carries a ticket, and the seat owner keeps the list.**

When a notice with an action is shown, the owner writes the action down and issues a
number for it. The notice carries the number and nothing else — no permission, no
session, no capability. At the press, the owner looks the number up:

- on the list → act;
- not on the list → refuse.

**The counter belongs to the long-lived seat owner and never restarts.**

That sentence is the whole of the security property, so it is stated on its own.
Numbers count up for as long as the owner lives, across every lock, logout and
switch. If a list were numbered from zero per session, a stale ticket 7 would match a
fresh ticket 7 — which is the uid problem again, smaller and harder to see.

## What falls out of it

| | what happens, with no check written for it |
|---|---|
| **lock** | the list is kept; nothing is dispatched while locked, so the notice is intact at unlock |
| **unlock** | tickets were never invalidated; nothing to restore |
| **logout** | the list is cleared — *invalidate its pending actions*, exactly |
| **switch user** | each session's list is its own; isolation is the shape, not a comparison |
| **a click racing a lock** | the owner is asked **at the press**; whichever lands first, the answer is right |

The last is the case the owner singled out, and it is the reason the ticket is a
lookup rather than a value. **A ticket cannot be read for its own validity.** The
only thing anybody can do with a number is present it, so *asked, never read* stops
being a rule people have to remember.

## Why not the alternatives

**Give `Session` a per-opening identity.** It derives `Eq`, and everything comparing
two sessions for *is this the same person* would silently come to mean *is this the
same sign-in*. It is also `alo-accounts`, which belongs to lane B, and it changes a
type the authentication path rests on.

**Put the identity on the `Seat`.** `Seat::Open(Session)` has no identity of its own
either, so a value still has to be minted — in `alo-locking` instead of
`alo-accounts`. It moves the problem rather than dissolving it.

**A ticket needs neither.** No type changes meaning, no crate boundary is crossed for
an identity, and ADR 0087's rule that the drawing layer holds no authority is kept by
construction: a number is not authority.

## This is the repository's own pattern, one level up

`alo_notifying::Missed` already does it for a different hazard, and its note is this
record in miniature:

> The next handle to give out. **Never reused inside one session**, so a dismissal
> cannot land on a notification that took a dismissed one's place.

Same fault — a stale handle landing on a later thing that took its place — and the
same remedy. What this record adds is the scope: *never reused inside one session* is
not enough when the thing being survived **is** the session ending, so the counter
moves up to the owner that outlives it.

## Consequences

**Three lanes touch this and none of them owns all of it.** The ticket list and the
counter live with the seat owner, in `alo-locking`, which is the third PC's. The
notice and its button are `alo-shell`'s, which is this lane's. Nothing is asked of
`alo-accounts`, which is lane B's — and that is the point of choosing this shape over
the first alternative.

**It is not finished when the list exists.** The owner's acceptance stands: *notifications are complete only once delivery, visibility and action handling are
connected and tested*, with lock/unlock, logout/login, user switching, and a click
racing a lock. The fourth is the one a passing suite otherwise misses, because both
checks are correct and the answer still changes between them.

**And a ticket is not a secret.** It is not guessed at, it is looked up, and the list
is the authority. If a later change ever needs a ticket to be unguessable, that is a
sign something started treating it as permission — which is the thing this record
exists to prevent.
