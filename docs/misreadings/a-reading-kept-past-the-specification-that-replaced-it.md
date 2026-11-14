# A reading kept past the specification that replaced it

**Found 2026-11-14, by the owner, on a booted machine.** Nothing caught it: every
test of alo's own windows' bands asserted the behaviour that was wrong.

## What was concluded

*"**Always shown**: alo's own windows carry Close alone, and it stays on their
bands as it always has."* — `crates/alo-shell/src/an_alo_windows_edge.rs`, written
after the owner's window-controls specification of 2026-11-09 had said **No
controls are drawn at rest**. The terminal's and the task windows' bands said the
same, each citing a reading of 2026-10-28 or 2026-11-01: *it carries the only
Close this window has*.

## What was true

The specification was about every band, and `docs/features.md` promised it *on
any window*. A walk machine at `3959dd65` drew Minimise and Close on the terminal
and the task windows with the pointer in the far corner, beside applications
whose bands drew none (`C:\dev\boot-evidence\the-walk\w28-29-codium-up.png`). The
owner called it *the old window controls*.

## The mechanism

The new specification was built window kind by window kind, and its stage table
named the kinds it changed. alo's own windows were each reached later for other
reasons — their icon removed, Minimise added — and each time the old reading sat
in the file being edited, **stated as a reason**, so it read as a decision rather
than as something the specification had replaced. A reading recorded *for the
owner to overrule* has no expiry, and nothing tells it that a later rule
answered the same question.

## The cure

When a specification changes a behaviour, **grep for every place that decided
the old behaviour for itself** — here, every `how_shown: ALL_THE_WAY` and every
`edge_of(…, true, …)` that draws — not only for the places the specification's
frames show. Then make the old decision impossible to keep by name:
`crates/alo-shell/tests/every_band_is_drawn_through_the_band.rs` now fails if a
band is drawn at a constant reveal.
