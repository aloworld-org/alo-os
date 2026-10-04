# ADR 0087 — The signed-in session is kept by one owner, and notifications ask it twice

**Status:** **accepted, 2026-10-04**, by the owner, in these words:

> **Yes — retain the signed-in session, but keep it with the trusted session
> owner. Do not pass the full session object throughout the drawing code.**
>
> The right fix is to connect the existing notification machinery to that owner,
> rather than build a separate canvas-only notification system.
>
> Use these boundaries:
>
> - **Session owner:** retains the authenticated session and manages lock, unlock,
>   logout and user switching.
> - **Notification service:** receives only the session context it needs to route
>   and authorize notifications. No credentials or unrelated capabilities.
> - **Canvas:** submits the recovery notice and its action through that service.
> - **Drawing layer:** receives presentation data approved for the current session,
>   not authentication authority.
>
> A UID identifies an account; it does not establish that its session is currently
> unlocked.
>
> **Check visibility when displaying the notice and authorization again when its
> button is pressed.** A notice approved before locking must not remain visible
> afterward. Locking conceals its content; logout or session replacement
> invalidates its actions. Reusing the same UID must not revive an old session's
> buttons.
>
> Bind **Put it back** to the originating session and window. Revalidate that the
> window and destination still exist before acting.
>
> Record this as a short architecture decision and proceed with the session and
> notification owners. Test lock/unlock, logout/login, user switching and a click
> racing with lock.
>
> **Retaining the session is necessary plumbing; notifications are complete only
> once delivery, visibility and action handling are connected and tested.**

Quoted whole because the four boundaries and the two-check rule are the decision,
and a summary would be a lane choosing which of them bound it.

## What this reverses, in one line

`crates/alo-shell/src/booting.rs` built a real `alo_accounts::Session` at sign-in
and then kept only its number:

```rust
Signing::HandedOver(session) => Stood::SomebodySignedIn {
    person: session.uid(),
},
```

Everything after that point had a `u32`. That is why **no notification is shown
anywhere in this system** — `alo_notifying::deciding::arrives` wants a
`Seat<Notification>`, a seat wants a `Session`, and the only one ever built was
dropped on the line that made it.

**The uid was not a smaller version of the session. It was a different fact.** The
owner's sentence is the general form: *a UID identifies an account; it does not
establish that its session is currently unlocked.* A screen that had the number
could tell you whose machine it was and could not tell you whether to draw.

## The machinery already has this shape, and that is the argument for it

Nothing here is new design. `alo_locking::Seat<N>` **is** the session owner the
ruling describes:

| the ruling | what exists |
|---|---|
| retains the authenticated session | `Seat::Open(Session)` / `Seat::Locked(Locked<N>)`, `Seat::session()` |
| manages lock and unlock | `Seat::locked(&mut Summoning)`, `Seat::is_locked()`, `alo_locking::unlocking` |
| manages logout and user switching | `alo_leaving::switching::asked(seat, summoning, accounts)` |
| routes and authorises a notification | `Seat::arrives(notification) -> Arrived`, and `deciding::arrives(notification, seat, quiet, missed)` |
| holds what must not be shown | `Arrived::Held`, `Missed::keeps` |

So *connect the existing machinery to that owner* is the whole instruction, and
the alternative the owner refused — a canvas-only notice surface — would have had
to re-implement `Arrived::Held` at a second place. `crate::notification_raster`'s
own note says why that is the expensive mistake: **never while locked, shared or
recorded is a rule the type carries**, not one that file implements, because it
takes a `Shown` and `deciding::arrives` is the only thing that makes one.

## The four boundaries, as they land in this tree

- **Session owner.** Holds `Seat<Notification>` from sign-in. `booting.rs` stops
  discarding the session; `Stood::SomebodySignedIn` carries what the owner needs
  rather than a number.
- **Notification service.** `alo-notifying` already takes only what it needs:
  `arrives(notification, &mut Seat, Quiet, &mut Missed)`. It is handed the seat,
  never an account store, a password, or a grant.
- **Canvas.** `alo-shell` submits the recovery notice through
  `arriving::from_alo_os`, with *put it back* as an offered action. It does not
  decide whether the notice may be shown.
- **Drawing layer.** `notification_raster` keeps taking `&[Shown]` and nothing
  else. A `Shown` is presentation data that has **already** been approved for this
  session; the raster has no authority and must not gain any.

## Asked twice, because the answer can change between the two

**At display:** whether this notice may be on the screen now.
**At the press:** whether this action may still run, for this session.

They are different questions because a lock, a logout or a switch can land
between them. The owner's three consequences, each a test:

- a notice approved before locking **must not remain visible** afterwards;
- **locking conceals content**, it does not discard it — the session carries on
  and `Missed` is where a held notification waits;
- **logout or session replacement invalidates actions**, and *reusing the same UID
  must not revive an old session's buttons* — so the thing an action is bound to
  cannot be the uid.

*Put it back* is therefore bound to **the originating session and the window**,
and revalidates that the window and the destination still exist before acting. A
frame can be closed, and a Place can be gone, between a person reading a notice and
pressing it.

## Consequences

**This is plumbing, and the promise is not kept by the plumbing.** The owner's last
sentence is the acceptance: *notifications are complete only once delivery,
visibility and action handling are connected and tested.* Retaining the session
makes the first production notification possible; `docs/features.md`'s **When the
machine moves a window, the person is told** is kept when a person is told and can
act, under lock, logout and switching.

**The tests the owner named are the gate**: lock/unlock, logout/login, user
switching, and a click racing with a lock. The last is the one a passing suite
would otherwise miss, because it is the only one where both checks are correct and
the answer still changes between them.

**Nothing in the drawing path gains authority.** If a change to this feature ever
needs `notification_raster` to know who is signed in, that is the signal it has
crossed a boundary this record drew.
