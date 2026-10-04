# ADR 0090 — A person who signs in gets their own agent daemon

**Status:** **accepted, 2026-10-04**, by the development PC under
`docs/autonomy/a-new-machine-becomes-a-lane.md`'s rule for a blocker that is a
decision: *write the ADR. You are the machine that understands why it matters.*
Its two exceptions are what needs the owner personally and what needs a lawyer,
and a service's start-up lifecycle is neither.

**This record is deliberately smaller than the question it came from.** The
installer lane raised the seat owner and the per-person daemon as one question.
They are not one question, and [ADR 0087](0087-the-signed-in-session-is-kept-by-one-owner-and-notifications-ask-it-twice.md)
— accepted by the owner earlier the same day — already decided the half that
carries the authority. What is left is a lifecycle fact about one unit, and that
is all this decides.

## Two different things are called a seat

The word collided, and an ADR spending it once would have appeared to settle
both:

```
alo_locking::Seat<N>            seat.rs:34 — Seat::Open(Session) / Seat::Locked
                                mentions of libseat, card, device, drm, seatd,
                                kms across the whole crate: 0
libseat::LibSeatSession         crates/alo-shell/src/direct_session.rs and six
                                other alo-shell files — the DRM card
```

One is **who may act now**; the other is **who holds the hardware**. ADR 0087
owns the first and this record does not touch it. This record is about neither:
it is about **when a process is started**.

## The decision

**A unit that serves one person is started once per person who signs in. It is
never pinned to an account number chosen when the image was built.**

`alo-agentd.service` is pinned:

```
User=alo
WantedBy=user@1000.service      ← in [Install]
```

uid 1000 appears in **six directives** — `BindsTo=`, `After=`,
`RuntimeDirectory=`, two `Environment=` lines, and `WantedBy=` inside
`[Install]` — and in **three comments** besides, so a grep for the number
returns nine lines and only six of them are the change. A second
account signing in therefore gets **no agent daemon at all** — `user@1001.service`
has no `.wants/` entry pointing at it, so nothing starts and nothing fails. **A
machine that can only ever have one person is not a decision anybody took**; it
is six literals.

### What this is not

**It is not a rule about authority, and must not be read as one.** ADR 0087 is
explicit that *a UID identifies an account; it does not establish that its
session is currently unlocked*, and that **the thing an action is bound to cannot
be the uid**. So:

- **Started for a person** is keyed by uid, because starting is a question about
  an *account*: which home, which runtime directory, which grants file.
- **Permitted to act** is not, and is answered each time by the session owner
  holding the `Seat`, at display and again at the press.

A daemon running for uid 1001 is a daemon belonging to that account. It is not
standing proof that anyone is signed in, and it must not be used as one.

[ADR 0088](0088-a-machines-grants-belong-to-a-person.md) sits on the first side
of that line: a grant is a **stored fact** about an account, written by a
deliberate act, and keyed by uid because `believing.rs` asks the open file's
owner as a uid. It binds no action, which is why the two records agree.

## What this costs, named rather than waved through

- **52 files named that unit on main at `0d2ac4db`** — 27 `.md`, 18 `.rs`,
  4 `.service`, 1 `.toml`, 1 `.conf`, and `image/Containerfile`. This record is
  the 53rd, by naming it here. This is a real change and not a tidy.
- **`alo-image` asserts the numbers.** `checking.rs` rewrites
  `RuntimeDirectory=alo/1000` to `alo/1001` and expects
  `Wrong::ThePersonsDoorIsNotMade { person: 1000 }`; `asserted.rs` holds
  `test "$(id -u alo)" = 1000`. A change here is a change there.
- **The image ships no user units at all** — 8 system units, nothing under
  `systemd/user/`. The obvious shape, a user unit started for each session, is
  **a new pattern on this image** rather than a smaller version of the current
  one. That is not an argument against it; it is a reason not to call it cheap.
- **`%U` is a trap this repository has already paid for**, and its own comments
  record why: in a system unit systemd expands it to 0, before `User=` is
  resolved, making a directory for root and a service that stops. Any template
  or user-unit answer must not reintroduce it.

## What this does not decide

- **Which shape the unit takes** — a template `alo-agentd@.service`, a user
  unit, or a generator. That belongs to whoever holds `image/`, and it wants
  measuring against those 52 files rather than picking here.
- **Whether `seatd` is ever wanted.** Out of scope entirely: that is the device
  seat, and nothing here is about the card.
- **What a second person sees before signing in.** One screen on one card;
  whether it offers a chooser is a surface decision.

## Consequences

- `accounts-and-session-entry-plan.md` task 18's acceptance becomes reachable.
  `crates/alo-remembering/src/whose.rs` is **built and inert**: it moves a
  machine's grants to the person who had them, and nothing calls it until a
  daemon starts for a second person. That is stated in its own PR rather than
  implied by a green suite.
- **Nobody can act on the implementation at the time of writing.** The charter
  gives `image/` and `alo-image` to the installer plan, and that lane is excluded
  from those paths by a standing instruction. **That is a permissions question,
  not an architectural one**, and it is recorded here so the two are not mistaken
  for each other.
- A future unit that serves one person inherits this: if it needs a number from
  the image to know whose it is, that is the signal it has crossed the line this
  record draws.

Roadmap: v0.5
