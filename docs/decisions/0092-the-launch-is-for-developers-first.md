# ADR 0092 — The launch is for developers first, and what that scopes

**Status:** **accepted, 2026-09-21, by the owner.** Recorded 2026-10-04, at the
owner's instruction, from the development PC's session notes of 21 and 22
September. **These are not the owner's dictated words.** They are what this
machine wrote down at the time, written up now because nothing in the repository
held them. Correct anything that misstates the decision; a record of a direction
is worth less than the direction and more than nothing.

## Why this is being written three weeks late

**A decision can only be called stale against a written direction.** The owner
asked on 2026-10-04 whether the accepted decisions still match what alo OS is
meant to be. That question cannot be answered — not by a machine and not by a
person — while the thing they would be measured against exists only in a
session's notes. This record is the baseline, and it is the whole reason it
exists.

**And the half that became rules was already recorded, which this machine had
been reporting wrongly.** [ADR 0064](0064-the-person-chooses-how-code-runs-and-every-protection-they-may-change.md)
carries the code-running levels and the freedom audit, accepted by the owner on
2026-09-22; `CLAUDE.md`'s second law already cites it; the installer already asks
about Fast Startup; *alo Bar* appears in eleven documents. What was missing is
not the rules. **It is the product direction the rules were decided inside.**

## The direction

**alo OS launches for developers first.**

Nothing below changes `docs/features.md` today. The direction is recorded here;
the feature lines wait until the v0.5 plans are finished and then go in as a
developer tier, which was the owner's own sequencing.

## What is settled, and should not be reopened without new facts

- **Penpot is installed and used as it is, and never forked.** Hosted on EU
  servers, offered in the app market, reached by the agent through Penpot's own
  MCP server. It is MPL-2.0 and about 72% Clojure, so a fork would also break the
  two-languages rule in `CLAUDE.md`. Penpot's enterprise tier is closed, and
  alo's own tenancy and sign-in fill that gap as alo's own files.
- **No Mac version.** PCs only. Somebody on a Mac may run alo OS in a virtual
  machine.
- **There is an alo app market** — a Flatpak repository and a storefront. A
  person sees *Install*, never *Flatpak*.
- **The boot menu is alo OS's own, with Windows directly behind it**, which is
  [ADR 0062](0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md)
  and already landed.
- **The same rule governs every app alo adds: host and install as it is, never
  fork.** That is what makes the app market a market rather than a set of
  branches.

## What this direction opens, and has not yet been decided

Each of these is a decision that will need its own record. None is made here, and
naming them is the point — an open question that nobody has written down is
indistinguishable from a settled one.

- **The licence.** The owner chose *GPL-3.0-or-later OR EUPL-1.2*, the
  recipient's choice, for European public-sector sales, and declined a commercial
  dual licence. **Nothing has been changed**: `Cargo.toml` still reads
  `license = "GPL-3.0-or-later"` and there is no EUPL text in the tree. The
  owner's own note was that this waits for their lawyer, and `CLAUDE.md`'s
  autonomy rule makes *what needs a lawyer* one of the two things a machine may
  not decide. **It stays unwritten until a lawyer has read it.**
- **NVIDIA.** Whether to ship a separate image carrying NVIDIA's driver, which is
  a licence question, and whether to rent a European GPU machine to show parity.
  **A developer launch is where this bites**, because the people being launched
  to are the ones with those cards.
- **The product's name to a customer.** The owner's later note replaces *alo
  workplace* with an app store as the product idea — mail, calendar, drive,
  design, code editors, each installed by whoever wants it. How that reads on a
  price list is not decided, and neither is per-app against subscription.
- **An ecosystem beyond the PC.** The owner wants companion applications on
  phones that pair with an alo PC, and possibly an alo phone much later. The
  second of those reverses a stated non-goal and cannot be done by inference from
  this record.

## What this record deliberately does not contain

**The feature list.** The owner's notes from the same two days carry a long list
— cross-device work, the v1 interface, ideas taken from other systems — and a
decision record is the wrong home for it. It belongs in `docs/features.md`, with
a tier against each line, once the v0.5 plans are finished, which is what the
owner said at the time. Writing it here would create a second list of what gets
built, and `CLAUDE.md` is explicit that there is only one.

**Anything already decided elsewhere.** The code-running levels, the freedom
audit, the Fast Startup question and the boot menu all have their own records.
This one points at them rather than restating them, because a direction that
repeats its own consequences is a direction that will disagree with them later.

## Consequences

- **The ADR audit now has something to measure against.** The question *does this
  decision still match where the product is going* is answerable for the first
  time. Expect most records to be **scoped** by this rather than contradicted:
  a decision taken for a general audience is usually still right for a narrower
  one.
- **A decision that conflicts with this is marked superseded, never
  overwritten.** The reasoning in an old record is why it was right at the time,
  and deleting it loses the only account of what was weighed.
- **This record is reconstructed, and that is a weakness to carry openly.** If
  the owner's memory of 21 September differs from what is written here, the
  owner is right and this file is wrong. It is a baseline because one was needed,
  not because it is authoritative.

Roadmap: v0.5
