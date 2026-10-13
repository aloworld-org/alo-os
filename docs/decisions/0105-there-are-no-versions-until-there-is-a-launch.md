# ADR 0105 — There are no versions until there is a launch

**Status:** **accepted, 2026-10-12**, by the owner.

- **Removes the version tiers** from `docs/features.md`. There is no `[v0.01]`,
  no `[v0.5]`, no `[v1]`.
- **Changes `CLAUDE.md`'s scope gate** from *in the list, with a tier, inside
  the current release* to **in the list, outside Non-goals**.
- **Leaves `docs/features.md` as the only list of what gets built**, and leaves
  Non-goals forbidding exactly what they forbade.
- **Versions return at the launch**, numbered from whatever we ship.

The owner's words: *we no longer care about versions, no need. We need a
working OS that the users can use, we want to launch. Delete all the versions
and we shall start the version later when we launch. Currently we are blocked
and we are moving in circles.*

## Why a tier stopped being an ordering and became a wall

A tier was a judgement about **when**, written before anybody knew what the
work cost. That is a reasonable thing to write down and a dangerous thing to
enforce, because the enforcement outlives the judgement.

Measured on the day this was accepted, in one afternoon:

- **Running code at the level the person picks** — the thing ADR 0064 was
  written for, in the owner's own words about freedom rather than dictatorship
  — is `[v1]`. So the capability the owner had decided a month earlier could
  not be built, on the authority of a marker nobody had revisited since.
- **Installing outside the sandbox** is `[v1]` while ADR 0064's own table says
  it moves to `[v0.5]`. The file and the decision disagreed and the file won,
  because the file is what the gate reads.
- **A throwaway box for anything downloaded** is `[v1]` and depends on the same
  sealed box as the first.

Three refusals in an afternoon, none of them a judgement anybody made that day.
**A tier is a decision with no expiry that gets consulted forever**, and the
cost of that is a product that cannot be finished because each piece is waiting
for a release that does not exist.

## What the gate still does, and it is the half that was working

**Somebody decided this.** A feature nobody wrote down is still not built, and
that rule has caught real scope creep: a Dock menu, a landmark cycle and a
grant level each had to be written into `docs/features.md` before a line of
them was written, on the same day as this decision.

`CLAUDE.md`'s own words on why that half matters are unchanged: *the cost of
asking is a message, and the cost of guessing wrong is a release nobody
decided.*

**And Non-goals still forbid.** Nothing here touches them. A thing in Non-goals
is not built because we decided not to build it, which is a different statement
from *not yet*.

## What replaces the ordering

**`ROADMAP.md` carries one list: what stands between today and a machine
somebody outside this team can use.** That is an ordering somebody maintains
because they are walking it, rather than a marker on three hundred lines that
nobody revisits.

The difference is not cosmetic. A tier is attached to the feature and consulted
when somebody wants to build it. A launch list is attached to the launch and
consulted when somebody asks *are we there*. The second gets looked at.

## What this costs, named rather than discovered

- **`docs/features.md` no longer says what is next.** That is the point, and
  the roadmap answers it instead.
- **The reconciliation audit keyed on `[v0.01]`.** It held that every promise in
  the current release names the test that shows it or says what is owed, and it
  caught six promises that nothing asked about. **It must now key on the launch
  list**, or it silently covers nothing. Changed in the same release as this
  decision, or the audit becomes a test that passes because its subject is
  empty — which is the shape `docs/misreadings/` is full of.
- **Seventy-four files mention a tier.** Most are prose citing one in passing.
  They are corrected as they are touched rather than in one sweep, because a
  sweep across seventy-four files is a change nobody can review.

## What does not change

The five laws. The gate in `CLAUDE.md` — formatting, clippy, the refusal path
tested as carefully as the happy path, an integration test on real hardware,
documentation in the same change. The capability model. Non-goals.

**This decision is about when things get built, and about nothing else.**
