# Every decision this repository points at, and the ones nobody can find

**Date:** 2026-09-11
**Workstream:** v0.01 delivery plan, task 18
**Contributor:** Claude (separate checkout, `C:\dev\alo-os-claude`)
**Status:** ready for integration

## What this is

`CLAUDE.md` makes the ADRs binding — *read the ADR before proposing an
alternative* — and this repository points at them **1,926 times** in its Rust
and Markdown, plus **195** links to a decision by the name of its file. Nothing
checked that any of those pointers landed.

An ADR reference is the one kind of pointer a reader believes without opening,
because the number looks like a fact. A number with no file behind it is
therefore not a broken link; it is a sentence that has borrowed the authority of
a decision nobody made, and it reads exactly like one that has not.

`crates/alo-citing` is the check. It is the fourth in a family — `alo-reconciling`
(promises against evidence), `alo-by-hand` (verbs against their plain way),
`alo-collected` (words against the one vocabulary) — and it has the same shape
for the same reason: **the thing being checked is read out of the repository, not
out of a list kept beside it**, so a citation added anywhere is one the check
sees, with nothing to update. Nothing in the crate holds a decision's number or a
file's name.

## What it found, and it is not hypothetical

Two real pointers in this repository did not land. Both were written by people
who were right about the argument and wrong about the address, which is exactly
the failure a number nobody verifies produces.

1. **`docs/decisions/0019-a-runtime-is-found-not-configured.md` line 5 linked to
   a decision file that has not existed for days.** Its `**Status:**` line says
   it keeps ADR 0006's one-file rule and links to a filename carrying that
   decision's *old* title — `ollama-is-the-pinned-model-runtime`, from before it
   became `0006-the-pinned-model-runtime.md`. (Written without its number and
   suffix here on purpose: spelled in full it would be a dead pointer this
   report really carries, and the check would refuse this file too.) The
   sentence is true, the link is dead, and a reader following it to check the
   claim gets nothing. That is the whole argument for this check in one line: a
   rename leaves every pointer at it reading exactly as it did before.
2. **`docs/decisions/0001-the-capability-model.md` line 80 cited two of
   `alo-workplace`'s decisions as if they were ours.** It said *unchanged from
   the workspace's* and then two bare numbers, which are
   `alo-workplace` ADR 0047 and ADR 0057 — carrying here the name the line
   itself now carries, and for the same reason. Nothing under `docs/decisions/`
   answers for either, and a reader who did not already know would look for them
   here and find the wrong thing or nothing. The fix is one phrase: name the
   repository.

Two more fixture pointers in `alo-reconciling` were assembled rather than
written; see *A pointer in an example* below.

## What is checked

| | |
|---|---|
| A number, or a filename, that no decision answers for | `Finding::ADecisionNobodyWrote`, `Finding::AFileNobodyWrote` |
| Two files claiming one number | `Finding::TwoDecisionsOneNumber` |
| A decision whose own file does not say what its status is | `Finding::ADecisionWithNoStatus` |
| A neighbouring repository named as cited, and cited nowhere | `Finding::ANeighbourNobodyCites` |
| Nothing to check against, or nothing found to check | `Finding::NoDecisionsToCheck`, `Finding::NothingCitesAnything` |

The measurement against this repository, on the disk the test runs on:
**25 decisions, 1,926 citations by number, 18 of them into `alo-workplace`, and
195 references by filename.** All of them land.

The third row is the one that is not about a pointer landing, and it is in the
check for the reason the task gives: an ADR nobody recorded as accepted or
proposed is one every reader will read as settled. The citation resolves, the
file opens, and the weight of a decision is taken by a recommendation still
waiting on the owner — which is exactly what ADR 0025 is today.

## Decisions this task had to make, and why

**It judges the pointer, never the argument — including the section.** Whether
`ADR 0001 §3` really is about granting a folder is a reader's job. So is `§3`
itself, deliberately: ADR 0019 numbers its headings, ADR 0020 is cited as `§2`
and `§3` and has no numbered sections at all, and `alo-models` cites
`ADR 0004 §policy`, which is not a number. Resolving a section would mean either
imposing a document convention this task had no standing to decide, or producing
findings nobody could act on. Checking the number and stopping there is the whole
of what was asked, and the exclusion is written into the crate's own
documentation so the next reader does not have to infer it.

**A neighbouring repository's decisions are named, not excused.** `alo-workplace`
has ADRs of its own and this repository cites four of them; refusing those would
be a check demanding that a true sentence be deleted. So a citation is read as
this repository's **unless the line names the repository it points into, before
the number** — `alo-workplace` ADR 0047, the way `docs/contracts/agent-verbs.md`
already writes it. The unit is the line, which makes the rule checkable and makes
a demand on writing at the same time: keep the repository's name with the number
it qualifies, because a reader skimming for which repository to open cannot
scroll up for it either. Writing `docs/decisions/README.md` broke this rule on
its first draft, by wrapping a name onto the line above its number, and the check
refused it — which is the rule paying for itself before it was ten minutes old.

The list of neighbours is a standing permission for a number not to resolve, so
it is held the way `alo-collected` holds its exceptions: **a neighbour nothing
cites is a finding**, because an unused permission is one more place a typo can
hide. It is one entry today.

**A list continues.** `alo-workplace`'s own `ADRs 0023, 0047, 0057, 0058` is
four citations and not one. A number introduced only by a comma is the easiest
place in this repository for a pointer to go unchecked, and ADR 0001's own
context line is written exactly that way. A continuation has to be introduced by a comma or by `and`, and its
digits have to begin with `0` and not run into a hyphen, so the date in
`ADR 0018, 2026-09-04` is not read as a decision nobody wrote.

**A pointer in an example is still a pointer.** A citation of a decision nobody
wrote is refused wherever it is written, including inside a test that is *about*
a dead pointer — because a reader who finds one in a fixture cannot tell it from
one in a sentence, and the check reads its own crate like every other. Where a
test needs a pointer that must not resolve, it assembles it from a constant
rather than writing it out. That applied to this crate's own tests, and to two
places in `alo-reconciling`:

- `crates/alo-reconciling/tests/every_v0_01_promise_is_reconciled.rs` needed a
  ledger naming a decision nobody wrote, which is the point of that test. Its
  fixture is now assembled, with the reason beside it.
- `crates/alo-reconciling/src/evidence.rs` and the same test file used a
  plausible-but-invented decision filename to prove *a decision is not evidence
  that anything was built*. Both now use a real one, which makes the test
  stronger rather than weaker: a decision is refused as evidence even when it is
  one this repository really has.

This is the one place the check costs a writer something, and the price is
written into `docs/decisions/README.md` beside the rule.

**A decision misnamed is not seen as a decision.** `among` reads a file under
`docs/decisions/` as a decision when it is four digits, a hyphen and `.md`; a
`README.md` is not one and nothing is asked of it. A decision misnamed therefore
does not appear as *a decision with a bad name* but as every citation of its
number naming a decision nobody wrote — the same finding, wearing the name of the
thing somebody has to fix.

## What changed

- `crates/alo-citing/` — the check. `citation.rs` is what a pointer is;
  `citing.rs` reads references by number out of the text, with the neighbour rule;
  `naming.rs` reads references by filename, in both forms this repository writes;
  `decisions.rs` is the decisions that exist and whether each says what it is;
  `finding.rs` is the sentences whoever wrote the citation acts on; `holding.rs`
  is the check. It reads no disk — the test in `tests/` is what puts this
  repository behind it, which is what lets every refusal be shown happening
  against a fixture.
- `docs/decisions/README.md` — new, and where whoever writes the next decision or
  the next citation meets the convention this makes load-bearing rather than
  tidy. It summarises no decision, deliberately: a list of what each ADR says
  would be a second place to keep them current and the first place to read
  something no longer true.
- `docs/decisions/0019-a-runtime-is-found-not-configured.md` — the dead link,
  repaired. Nothing about the decision changed.
- `docs/decisions/0001-the-capability-model.md` — `alo-workplace` named where two
  of its decisions are cited. Nothing about the decision changed.
- `crates/alo-reconciling/{src/evidence.rs,tests/every_v0_01_promise_is_reconciled.rs}`
  — the three fixture pointers described above.
- `Cargo.toml`, `Cargo.lock` — the new member.

It says nothing to a person and declares no strings: it is a repository check
like `alo-reconciling`, `alo-by-hand` and `alo-collected`, and `alo-saying` does
not collect it. `alo-collected`'s own measurement is what proves that, since a
crate with no `src/words.rs` is not a crate that declares words — and it still
passes, at twenty-three declaring, twenty-two collected and one apart. Nothing in
`crates/alo-shell` was touched.

## Verification

Windows 11, `cargo 1.97`, from `C:\dev\alo-os-claude`:

- `cargo fmt --all -- --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean, zero warnings.
- `cargo test --workspace` — every crate passes **except the six pre-existing
  Windows-only failures in `alo-recounting`'s unit tests**, which are the ones
  task 17's report named and deliberately did not cut to green. `alo-recounting`
  is not among the files this task touched; those tests pass on Linux and the fix
  is a decision about what `alo-keeping` says on a host that cannot ask.
- Each acceptance criterion's test run on its own — see the `evidence` block of
  `.kernel-loop/handoff.toml`.

Not run here: anything on hardware. This task touches no machine, no screen and
no image, and no *On the machine* box moves.

## Limitations

- **A citation in a file that is neither Rust nor Markdown is not read.** The
  task scoped it to those two, and they are where every citation in this
  repository is. A `.toml` or a shell comment citing an ADR would go unchecked.
- **A qualifier that wraps onto the line above is refused**, and the fix is to
  keep it with its number. That is a real cost to a writer and it is the price of
  a rule a reader can apply too.
- **A document elsewhere whose filename begins with four digits and a hyphen
  would be read as a decision** and would have to be renamed. Nothing in this
  repository is named that way today.
- **Sections are not resolved**, as argued above.

## Proposed shared-document updates

For the integration owner; this contributor does not edit these files.

- **`CHANGELOG.md`** — under the unreleased heading: *Every ADR this repository
  cites is now held to being a decision somebody wrote. A link in ADR 0019 that
  had pointed at a renamed file since ADR 0006 was renamed, and two of
  `alo-workplace`'s decisions cited in ADR 0001 as if they were ours, are
  repaired. `docs/decisions/README.md` is where the convention is written down.*
- **`docs/autonomy/QUEUE.md`** — task 18 of the v0.01 delivery plan is done; no
  new item is owed by it.
- **`docs/autonomy/STATE.md`** — reference this report.
- **`ROADMAP.md`** — no change. This closes no promise in `docs/features.md` and
  moves no exit gate.
