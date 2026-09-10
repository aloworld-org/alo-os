# Every verb's by-hand answer, and a check that it has one

**Date:** 2026-09-11
**Workstream:** the v0.01 delivery plan, task 14
**Contributor:** Claude Code, in `C:\dev\alo-os-claude`
**Status:** ready for integration

## What this was

`docs/features.md` promises at v0.01 that **anything an agent verb can do, a
person can do by hand** (ADR 0009). Task 11's audit found it was the one promise
of forty-one that is *a standing rule with nothing checking it*: ten verbs ship
today and not one of them had ever been asked the question, so a verb arriving
tomorrow with no by-hand answer would break nothing, fail nothing, and nobody
would be told.

It is one of the six promises with no evidence at all, and the only one of them
this lane could close without a decision, a screen or a machine.

## What changed

**`docs/by-hand.md`** — a new document beside `docs/features.md`: every verb alo
OS ships, and how a person does the same thing without the agent. Ten entries,
prose rather than a table, because the sentences are the half no test replaces.

**`crates/alo-by-hand`** — what makes the document true of the verbs rather than
of a list somebody wrote once. Eleven modules, one responsibility each:

| | |
|---|---|
| `holding` | The check: `held(verbs, from, document, features, reading)` |
| `document` | `docs/by-hand.md` read as entries under one heading |
| `entry` | One entry: a verb, its plain way, what it owes |
| `by_hand` | The plain way: a sentence, and what it quotes |
| `owed` | The debt, and the release that owns it |
| `answer` | How much of an answer a sentence has to be |
| `promised` | Every promise in `docs/features.md`, with its tier |
| `release` | A tier, as the definition marks one |
| `declaring` | Which crates of this workspace declare verbs |
| `wrapping` | Reading a quotation that markdown wrapped |
| `finding` | The eleven things that can be wrong |

**`docs/contracts/agent-verbs.md`** — rule 7 of *adding a verb*, and the section
after it recording where verbs are declared, because the check reads that
convention and a convention a check depends on is a rule.

**`docs/autonomy/v0-01-evidence.md`** — the entry for this promise now names the
test and this report, with what the check cannot reach written as what is owed.
The audit's own summary paragraphs are left exactly as they were and the closure
is recorded under them: a finding rewritten by whoever closed it is a finding
nobody can check.

**`Cargo.toml`** — the new crate as a workspace member.

## The decisions, and why

**An answer is a promise in `docs/features.md`, quoted — not a sentence somebody
wrote in the new document.** This is the decision the whole design rests on. ADR
0009's rule is not *have an idea about how a person might manage without the
agent*; it is *no surface may be left out because an agent can do it instead*,
and the only place a surface is committed to is the definition, with a tier. A
document of reassuring sentences would have passed every check I could write
against it. Quoting the definition is what makes *there is a plain way* stop
being an opinion — and it means **the release that owns an answer is read off the
line rather than asserted beside it**, so a tier can never disagree with itself.

**The second form is *owed at a release*, and it costs something to write.** One
verb needs it: `archive_folder` **makes** an archive, `docs/features.md` promises
`archives that open`, and stretching the one to cover the other would have been
this check lying in the first change that used it. So *owed* names a release, and
the release has to be one the definition actually makes promises for — a verb owed
at a release nobody ships is owed at nothing.

**The verbs are read out of `alo_capability::Verbs`, and the workspace is walked
as well.** Reading the registry is what the task asked for and it buys the easy
half: a verb added to `alo-files` or `alo-applications` is a verb this check sees,
with nothing to update. It does not buy the half that actually goes wrong — **a
new crate that declares verbs and is connected to nothing.** That happened one
floor down in this repository: `crates/alo-overlay` declared nine strings that
`alo-saying` did not collect, and the only reason a shell would not have shown a
key where a sentence belongs is that a person noticed. So `declaring` reads the
workspace's own member list and refuses a crate that declares verbs in
`src/verbs.rs` and was not handed in. The convention that makes this readable is
now in the contract, where whoever is deciding where to put their `verbs.rs`
reads it.

**Whitespace is the one thing not compared exactly.** `docs/features.md` writes a
promise on one line however long; `docs/by-hand.md` wraps at eighty like every
document here. Comparing verbatim would have refused the longest and most
specific quotations — the ones least likely to fit two promises — and rewarded
short vague ones, which is a check that teaches you to quote less. `wrapping`
reads both sides as one line and nothing else is normalised.

**The four application verbs are held to this even though the daemon does not
carry them out yet.** `alo-agentd` only carries out the file verbs today. The rule
is about verbs the machine *declares* — `docs/features.md` promises open, focus,
arrange and close at v0.01 — and a verb exempted until it runs is a verb whose
answer is written by somebody who has forgotten why it was needed.

**Six verbs shipping at v0.01 with a v0.5 plain way is reported, not refused.**
It is the finding worth reading twice, and it is written into `docs/by-hand.md`'s
own prose: on a v0.01 machine with no agent, or one whose provider is down, a
person cannot list, read, find, rename, move or archive anything at all, because
the file manager, the search, the text editor and the terminal are all v0.5.
Failing the gate on it would be this crate overruling the release plan — v0.01 is
*it boots and the agent acts*, the desktop is v0.5 — and moving either is the
owner's decision. What was not acceptable is that it was true and unwritten.

**Nothing here says anything to a person.** No strings, no vocabulary, and
`alo-saying` does not collect this crate, exactly as the task constrained and for
the reason `alo_reconciling::Finding` gives: the reader is whoever is adding a
verb, in the repository, with both documents open.

## What is refused, and how each refusal is shown

Eleven findings, each demonstrated against a fixture beside the real measurement,
because a check that has never been seen to refuse anything passes on the day it
stops looking in exactly the same colour.

| Finding | Shown by |
|---|---|
| A verb the document says nothing about | `a_verb_the_document_says_nothing_about_is_the_finding` |
| A crate declaring verbs that nothing handed in | `a_crate_that_declares_verbs_behind_the_checks_back_is_refused` |
| A verb with no plain way and nothing owed | `a_verb_with_no_plain_way_and_nothing_owed_is_refused` |
| A plain way quoting nothing from the definition | `an_answer_that_quotes_no_promise_is_refused` |
| A quotation the definition does not make | `an_answer_quoting_a_promise_that_is_not_there_is_refused` |
| A quotation that fits two promises | `an_answer_that_fits_two_promises_is_refused` |
| A debt with no release, and a release nobody ships | `a_debt_with_no_release_or_a_release_nobody_ships_is_refused` |
| A shrug, in either form an answer can take | `a_shrug_is_not_an_answer_in_either_form` |
| An entry about no verb, and a verb answered twice | `an_entry_about_no_verb_or_a_verb_answered_twice_is_refused` |
| A document with no entries, and a workspace that cannot be walked | `a_document_or_a_workspace_that_cannot_be_read_is_checking_nothing` |

The unit tests beside each module hold the parsing itself: a markdown checkbox is
not a promise for a release called nothing, a dash is not the way a person does
something, `[v0.5]` with no sentence after it is not a debt, a release with
nothing after the `v` is not a tier, and a heading that moved leaves nothing to
check rather than a check that quietly holds nothing.

## Verification

**On Linux, which is the gate** — WSL Ubuntu, from `/mnt/c/dev/alo-os-claude`,
`CARGO_TARGET_DIR=/root/target-claude` as `docs/autonomy/SHARED_MAIN.md` requires
so the desktop worker's target is untouched.

| Command | Result |
|---|---|
| `cargo fmt --all` | clean |
| `cargo clippy --all-targets --workspace -- -D warnings` | clean — zero errors, zero warnings |
| `cargo test --workspace` | exit 0, 172 test-result groups, no failures |
| `cargo test -p alo-by-hand` | 27 unit tests, 13 integration tests, all green |

Each of the five evidence tests was also run on its own with `--exact`, and each
reported one test passing.

**On Windows, the host** — `cargo fmt --all` clean, `cargo clippy -p alo-by-hand
--all-targets -- -D warnings` clean, `cargo test -p alo-by-hand` green.

`cargo test --workspace` and `cargo clippy --all-targets --workspace` **do not
pass on Windows in this repository, and did not before this change either.** Six
`alo-recounting` unit tests fail and two dead-code warnings appear in
`alo-keeping` and `alo-recounting`, all of them because `alo_keeping::believing`
has a `cfg(unix)` half and a non-unix half that answers *unsupported* — so the
tests assert Linux behaviour and `NotKept::a_link` reads as dead code. Confirmed
pre-existing by stashing this change and rerunning. They are another
workstream's crates, they are clean on the gate above, and they are left rather
than silenced from here — the same shape as the `broken_intra_doc_links`
exception `docs/autonomy/LOOP.md` already records for Windows.

**Not run:** hardware acceptance. Nothing here touches a kernel, a device or a
compositor; it reads two documents and a manifest.

## Proposed changelog entry

> **Every verb says how a person does it by hand.** `docs/by-hand.md` answers,
> for each of the ten verbs alo OS ships, what a person does instead when there
> is no agent — because the agent is unavailable for six reasons and only one of
> them is a choice, and the money running out is one of the others (ADR 0009). It
> is held by a check rather than by memory: a verb added with nothing said about
> it fails the build in the change that adds it, and so does an answer pointing
> at a promise the feature list no longer makes. The document records what it
> found: six of the ten verbs ship at v0.01 and the plain way to do them arrives
> at v0.5.

## Proposed roadmap and queue updates

- v0.01's *anything an agent verb can do, a person can do by hand* has evidence:
  `crates/alo-by-hand/tests/every_verb_can_be_done_by_hand.rs`. Five of the
  audit's six promises with no evidence at all remain.
- Task 14 of `docs/autonomy/v0-01-delivery-plan.md` is marked done in this change,
  and task 15 — *a machine that cannot reach a model says so once* — is written
  there for the next worker.
- `CLAUDE.md`'s **Map** lists the documents an outside contributor is sent to, and
  `docs/by-hand.md` is now one of them. The line is not added here because the
  constitution is the owner's voice; `docs/contracts/agent-verbs.md` points at the
  document in the meantime, which is where whoever adds a verb actually reads.

## For the owner: one scope decision, and one worth looking at

**A line for making an archive.** `archive_folder` is the one verb with no
promised plain way. `docs/features.md` promises `A file manager, with trash, and
archives that open`, which is the other direction. The proposed addition, for the
owner's scope gate and not made here:

> `- [v0.5] **Make an archive from a folder**, in the file manager — the plain
> way to do what the agent's archive verb does`

Until it is made, the verb reads as owed at v0.5 in `docs/by-hand.md`, which is
honest and which the check enforces.

**Six verbs whose plain way is a release away.** Not a decision this task can
take, and the one thing in here somebody should look at twice: v0.01 ships six
file verbs an agent can run and no surface a person can. If v0.01 is ever shown
to anybody outside this team, that is the sentence they will find.

## Files

- `Cargo.toml`
- `crates/alo-by-hand/Cargo.toml`
- `crates/alo-by-hand/src/lib.rs`
- `crates/alo-by-hand/src/answer.rs`
- `crates/alo-by-hand/src/by_hand.rs`
- `crates/alo-by-hand/src/declaring.rs`
- `crates/alo-by-hand/src/document.rs`
- `crates/alo-by-hand/src/entry.rs`
- `crates/alo-by-hand/src/finding.rs`
- `crates/alo-by-hand/src/holding.rs`
- `crates/alo-by-hand/src/owed.rs`
- `crates/alo-by-hand/src/promised.rs`
- `crates/alo-by-hand/src/release.rs`
- `crates/alo-by-hand/src/wrapping.rs`
- `crates/alo-by-hand/tests/every_verb_can_be_done_by_hand.rs`
- `docs/by-hand.md`
- `docs/contracts/agent-verbs.md`
- `docs/autonomy/v0-01-evidence.md`
- `docs/autonomy/v0-01-delivery-plan.md`
- `docs/autonomy/updates/every-verbs-by-hand-answer.md`
