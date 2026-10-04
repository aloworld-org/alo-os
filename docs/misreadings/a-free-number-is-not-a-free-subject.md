# A free number is not a free subject

**Concluded** — *0089 is taken, so this decision is 0090, and I may write it.*

**True**, measured on main at `0d2ac4db`:

```
docs/decisions/0090-*                      nothing — the number was free
docs/decisions/0087-the-signed-in-session-is-kept-by-one-owner-...
    Status: accepted, 2026-10-04, by the owner, in these words
    "A UID identifies an account; it does not establish that its session
     is currently unlocked."
    "...the thing an action is bound to cannot be the uid."
```

The draft's third point read **anything a person's authority reaches is keyed by
their uid**, generalised from [ADR 0088](../decisions/0088-a-machines-grants-belong-to-a-person.md).
**That is the sentence 0087 forbids**, and 0087 was accepted by the owner
earlier the same day, in the same directory.

0088 itself survives, and the reason is the useful part: **the line is not what
a thing is keyed by, it is whether anything acts on the key.** A grant is a
stored fact about an account, written by a deliberate act, and nothing acts on
the uid; a notification's button would. **Only the generalisation crossed the
line** — and generalising was the whole shape of the draft.

**A generalisation from one's own accepted record is the hardest kind to
doubt.** It arrives feeling like consistency rather than like a new claim, so
the step that would have caught it — *is this a decision, and has it been
made?* — never gets taken, because it does not present as a decision at all.

**The mechanism.** `ls docs/decisions | tail` answers *which number is free*. It
is read as *whether this is decided*, because both questions get asked with one
command and the answer to the first arrives looking complete. A number is cheap
to check, **and that is exactly why it gets substituted for the expensive
question** — which CLAUDE.md already names: *read the ADR before proposing an
alternative; relitigating without new facts wastes the scarcest resource we
have.*

**A second thing made it easy.** Two different things in this tree are called a
**seat**, and the draft had spent the word on the wrong one:

```
alo_locking::Seat<N>      seat.rs:34 — who may act now. Mentions of libseat,
                          card, device, drm, seatd, kms in the crate: 0
libseat::LibSeatSession   alo-shell/src/direct_session.rs and six others —
                          who holds the card
```

A peer lane had raised *the seat owner and the per-person daemon are one
question*; the framing was accepted for most of a day. **They are two questions
wearing one noun.**

**The cure is not carefulness — it is provenance.** The applications lane got
the same step right the same day and said plainly it was luck rather than
virtue: they checked ADR 0089's *number* across every remote branch, which
answers this warning only by accident. **The reason they did not collide on
subject is that the subject came from a plan** — a queue row naming *the ADR
about what controls are called*, and a task saying it was owed. So:

- **Subject from a task or plan** — the number check is enough. Something else
  already owns the question of whether it is open.
- **Subject from the author** — grep the decisions for the *subject*
  (`grep -il 'seat\|uid' docs/decisions/*.md`), and read the two or three
  records nearest it in time, because a directory recording an active argument
  changes under you.

Then check whether the record's central noun means one thing in this tree: if
the crate that owns the word never mentions the hardware, the word is doing two
jobs. That the boundary holds in the file that would cross it was confirmed by
the lane that owns it rather than inferred here — `notification_raster.rs:107`
takes `&[Shown]` and no uid, person or session.

**What landed instead** is narrow: [ADR 0090](../decisions/0090-a-person-who-signs-in-gets-their-own-agent-daemon.md),
a lifecycle rule about when one unit starts, citing 0087 for the half it does
not own.
