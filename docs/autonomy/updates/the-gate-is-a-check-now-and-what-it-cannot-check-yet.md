# The gate is a check now, and one thing it cannot check yet

**2026-09-27.** `ROADMAP.md`'s v0.5 exit gate was read by hand seven times over
three months. The eighth reading took a day and got four of its own eight refusals
wrong. A ninth reading was never the answer: **everything that reading found is
mechanically detectable.** A person reading 92 promises is a rule; a crate that
fails the gate is a check.

This is what was built, what it found on the day it was written, and the one
question it cannot answer — with the numbers that say why, and what would change
that.

## What was built

`crates/alo-reconciling` already read every `[v0.01]` promise in
`docs/features.md` against `docs/autonomy/v0-01-evidence.md`. It now has a second
half, `src/the_gate.rs`, which reads this gate against the definition, against the
plans, and against itself; and a second ledger,
`docs/autonomy/v0-5-evidence.md` — **92 entries for 92 promises**, one to one,
every heading an exact substring of the promise it is about.

Seven checks, each named after the thing that specified it:

| check | what specified it | armed in the gate |
|---|---|---|
| a half under the wrong promise | the v0.5 cut left four boxes behind, two of them ticked | yes |
| a box denying work a plan says is done | pairing *open* twelve days after it was done; the index *open* thirteen days after | yes |
| a status word its own section contradicts | 74 tasks across six plans | yes |
| a refusal a report contradicts | six refusals written from searches narrower than the claim | yes, as a signal |
| a count that drifted | *five designed hues* ticked when `Accent::ALL` became four | yes |
| a promise with no box in the gate | three `[v0.5]` promises had none | **no — see below** |
| a tier that disagrees with the definition | the camera at `[v0.5]`, in v1's *Devices*, *carried to v2* in its plan | **no — see below** |

The refusal check is deliberately a weak signal shown loudly. It cannot know
whether a report is about the promise whose box denies something, and it does not
need to: it only has to make somebody look before writing *nothing exists*, which
is the exact failure it was written for. A refusal that survives being shown its
reports is a refusal somebody looked at, and the test lists those by name with the
reason.

**None of the seven reads a plan's status to decide whether work is done.** That is
the one thing the reading established twice and forgot twice. A status is read only
to find a disagreement with something else, never as the answer.

## What they found on the day they were written

- **A box calling a finished task open.** *"Where is that file?"* said *one known
  limit, open as task 11*. Task 11 of `v0-5-the-machine-measured-plan.md` was done
  on 2026-09-14 — thirteen days earlier — and the index is made whole.
- **Two more wrong refusals, making six.** *The ordinary desktop* said **four of
  the applications do not exist** and that the choice between writing them and
  pinning upstream ones was *owed a decision*. *A web browser for the open web*
  said **no browser is pinned**. `crates/alo-software/shipped.toml` has pinned all
  seven since **2026-09-15**: Firefox 156.0, Dolphin 26.04.3 with its trash, Ark,
  GNOME Text Editor 50.1, Loupe 50.0, Papers, and Ptyxis 50.1 as a terminal
  `Shipped::decided` refuses the list without unless no agent can be granted it
  (ADR 0043) — each with a licence, a source and the version it was decided at,
  held by a test and explained in `updates/what-a-fresh-machine-has.md`. Both code
  halves are now ticked and what is owed moved to the machine half, which is the
  honest place: nothing installs that list yet.
- **A box that withdrew a wrong claim was being read as still making it.**
  *Settings, as one place* carries *This box said the one place does not exist on
  2026-09-26, which was wrong*, and a check reading the box's words found *does not
  exist* in it. Deleting that record to satisfy a check would be the check making
  the documents worse, so the convention is named instead: from `*This box said` to
  the end of a box is the record of a claim no longer made, and nothing reads it as
  a claim.

## And the ledger made the same mistake five more times

Five entries were first drafted as **Shown by: nothing** — the lock-screen image,
the dock's size, a file manager, USB storage, a text editor and a terminal — and
every one was wrong, written from the crate whose *name* matched rather than the
crate that held the answer:

- `alo-appearance/src/lock.rs` has decided the lock screen's independence from the
  desktop all along, including the case it exists for: following does **not** follow
  a rotating folder, because the desktop is seen by whoever is signed in and the
  lock screen by whoever walks past, and a person who pointed their background at a
  folder of their own photographs did not choose to show them to a corridor;
- `alo-dock/src/layout.rs` sizes the dock, gives names way to icons where there is
  no room, never takes more than its share, and says in writing that *whether it
  hides when a window needs the room is v0.5* at the place that work would go;
- `alo-drives` reads the rented disk service, with mount and eject through the
  broker and *a drive is only ever mounted or ejected* as its test.

Each of those entries now carries its own correction. **The pattern is the
finding**: a negative claim made from a search narrower than the claim. It is the
same error as all six of the gate's wrong refusals, and it is the argument for a
crate rather than a ninth reading.

## What the check cannot do, and what would fix it

**It cannot say whether a box covers a promise**, and that is the question
*a promise with no box* and *a tier that disagrees* both rest on.

The gate groups on purpose: **31 boxes over 92 promises.** *Settings, as one place*
is one box over nine areas. *Making it yours* is one box over six promises about
appearance. So most promises are covered by a box that does not quote them, and
coverage is a judgement the gate expresses in prose.

Two settings were measured against the real documents:

| how a box is taken to answer a promise | what it reported |
|---|---|
| half the promise's words over two characters appear anywhere in the section | **3** promises with no box — and it missed real ones. Half the words of *Audio in and out, with device switching that works mid-call* are `and`, `out`, `with`, `that`, which appear in any prose at all, so the gate was reported as answering a promise it has no box for |
| a run of four consecutive words of five characters or more | **69** of the 92, including *Multi-monitor, display scaling, hotplug*, which has a box titled *Multi-monitor, scaling, hotplug* |

**Tuning that number until the answer looks right would be fitting the check to the
documents** — the reading it replaces, wearing a test's name. So neither is armed.
Both functions are kept, unit-tested against fixtures, and run against the real
documents asserting only what is reliable: that each still finds the thing it was
written for. The three promises with no box at all — input methods for non-Latin
scripts, Bluetooth, full-disk encryption — are asserted to be found, so the check
cannot silently stop working.

### What would fix it

**Each box in the gate naming the promises it answers.** Then the check is exact:
two sets, compared, with no heuristic anywhere. One line under each of 31 boxes,
listing the `[v0.5]` lines it covers; a promise in neither any box's list nor the
definition is a finding, and so is a promise in two.

It is a change to `ROADMAP.md`'s own shape and therefore its owner's to make, so it
is written here rather than done. Three things recommend it beyond this check:

- a reader of a grouped box currently cannot tell which promises it is claiming,
  which is how *night light* was built, reported twice and counted nowhere;
- the ledger already holds the other half of that mapping, promise by promise, so
  the two documents would join exactly;
- it would make the two counts — *26 of 93 boxes* and *92 promises* — answerable
  against each other, which nothing in this repository can do today.

## The gate

`crates/alo-reconciling`: `cargo fmt --check`, `clippy -D warnings`, and its tests
— 31 unit tests including one fixture per check, the v0.01 ledger's 15, and 7
against the real documents. Then the workspace.
