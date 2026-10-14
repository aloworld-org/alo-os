# ADR 0115 — The desktop runs inside the session alo-sessiond opened for it

**Status:** proposed, 2026-10-14, by the third-PC lane, from a measurement on a
booted machine.

**This decides launch item 2's named race.** `ROADMAP.md` says of *it boots and
a person signs in*: "the handover of the graphics card from the greeter to the
desktop is a named race — `alo-desktop.service` says its ordering is intent
rather than a guarantee, and is owed a measurement on a machine." This is that
measurement, and **the race is not what was wrong.**

## What a booted machine said

A person made the first account on a machine written from `5c19bf1b`: chose a
language, typed a name, set a PIN. `alo-sessiond` opened their session.
`alo-desktop` then crash-looped, and the screen said — correctly, and without
blaming them:

> Signing in did not finish: the part of this machine that starts a session was
> reached and did not answer. Nothing you typed was wrong

The serial console, with timestamps, is the whole argument:

```
[ 1723.840704] systemd-logind: New session c1 of user alo
[ 1726.203464] systemd-logind: New session 3 of user alo
[ 1735.311535] systemd[1]: Started alo-desktop.service
[ 1739.766759] alo-desktop: ... libseat found no login session for this process
```

**Two sessions for that person existed nine seconds before the desktop
started**, and the desktop still found none for itself. Six restarts created no
further sessions, though PAM reported `session_open ... res=success` every time.

So the fault is not ordering, and it is not registration. **It is
association.** `logind` answers *which session is this process in* from the
process's control group, and a systemd service lives in
`alo-desktop.service`, not in the session's scope. `pam_systemd` can open a
session for a service; it cannot move the service into one.

## Why the greeter works and the desktop does not

Both take the same code path — `DirectSession::new` calls
`LibSeatSession::new()` in each. The only difference is the user:
`alo-compositor.service` is root and `alo-desktop.service` is `User=alo`.

`libseat` tries its logind backend and falls back to a builtin backend that
opens devices directly, **and the builtin backend requires root**. So the
greeter has never had a login session either; it has been getting the card
because it is root, and nothing has ever depended on the handover working.

That is worth stating plainly: *the greeter working was not evidence that the
mechanism worked.* It was evidence that root does not need it.

A `pam_selinux` line appears beside the failure — `Setting file context ...
failed for /dev/tty1: Operation not permitted`, seven times, on `alo-desktop`
and never on `alo-compositor`. It is real and it is **not** the cause: PAM
reported success regardless, and the session count is the same with or without
it. It is recorded here so the next reader does not spend an afternoon on it.

## The decision

**`alo-sessiond` starts the person's desktop, as its child, inside the session
it already opened and holds.**

**And it opens that session with a seat.** `seat0` and vtnr `1`, in place of
the empty string and the zero it passes today — the section below shows that
this is the change `machine.rs` predicted would be owed, and that neither half
of the fix works without the other.

`alo-sessiond` calls `CreateSession` and keeps the descriptor — `logind.rs`
says so in its own words: *the session lasts as long as somebody holds it*, and
that is why the trait takes `&mut self`. A child of that process inherits its
control group, lands in the session's scope, and `logind` then answers the
question `libseat` asks.

So the session a person signs in to becomes the session their desktop runs in,
which is what `alo-desktop.service`'s own header already claims is true:
*whose session this is, is whose session this is.*

`alo-desktop.service`'s `WantedBy=user@1000.service` stops being how the
desktop starts. `user@1000.service` is the user manager and is in no session,
which is the whole fault.

## Two things are wrong, not one, and the code named the second first

The paragraph above is half the fault. `crates/alo-sessiond/src/machine.rs`
calls `CreateSession` like this:

```rust
person,                 // the account number
std::process::id(),     // Leader: alo-sessiond's own pid
US, NOT_SAID, A_PERSONS,
"",                     // the seat
"", 0_u32,              // and the virtual terminal
```

**The session is created with no seat and no VT**, deliberately, and that file
says why in its own words:

> They are passed empty and zero, which is a session with no seat. A seat and a
> VT are facts about *what draws*, and this component must not draw, start a
> compositor, or decide what a person sees — that is task 13's and the desktop
> lane's. [...] Both are additive to change — **a later task that starts a
> compositor is the one that knows which VT it is on, and it will be changing
> an argument here rather than adding a door.**

That reasoning was right when it was written and this is that later task.

So the desktop needs **both** of these, and neither alone is enough:

1. **the session must have a seat** — `seat0` and vtnr `1`, the two arguments
   above, which is the change that file predicted;
2. **the desktop must be inside that session** — because even a seated session
   answers nothing for a process `logind` cannot place in it.

Fixing only (2) yields a session `libseat` can find and that has no seat to
lend. Fixing only (1) yields a seated session the desktop is still not in. The
measurement could not distinguish them, because the machine fails at the first
of the two and never reaches the second.

**The leader is already right.** `CreateSession` is passed `alo-sessiond`'s own
pid, so `logind` puts *that* process in `session-N.scope` — which means a child
forked after the call inherits the scope and needs nothing else. The order is
the whole mechanism: open the session first, start the desktop second.

## What was rejected, and why it is worth recording

**Shipping `seatd`.** Available (`seatd 0.9.3-1.fc42`), one package and one
unit, and it works whatever the control group says — it is what compositors
outside logind do. Rejected because it adds a second daemon handing out device
access *beside a session we already create and hold*, so the machine would
have two answers to **who may use this card** and they could disagree. The
cheaper fix buys a day and costs the clarity of there being one seat.

**Passing the card's descriptor** from `alo-sessiond`, which is privileged, to
the desktop. This is the most alo-shaped of the three and the most work, and it
makes the shell depend on a handover with no upstream behind it. If the
decision above proves wrong on hardware, this is the next one to try, not
`seatd`.

## How this is known to be done

Not by a unit starting. **On a machine:** a person makes an account, the
desktop appears, and `loginctl show-session` names a session whose `Seat` is
`seat0` and whose scope contains `alo-desktop`'s pid. The gate cannot run this;
`docs/booting.md` says what a virtual machine can never show about one, and
this is a reading a virtual machine *can* make.

**And a test that fails today:** nothing in the workspace asserts that the
process which takes the card is in a login session, because until this boot
nobody knew the greeter was not. Whatever lands this writes that test.
