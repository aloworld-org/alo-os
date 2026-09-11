# A record that is not what it says it is

- **Date:** 2026-09-11
- **Workstream:** lane B — accounts and session entry
  (`docs/autonomy/v0-01-lane-b-plan.md`, task 6: *The account a person asks for
  is the one their machine kept*)
- **Contributor:** Claude worker in `C:\dev\alo-os-b`
- **Status:** ready for integration

## What changed, in a person's words

When somebody asks their machine what its agent did, the answer now says one
more true thing: whether the record behind it is what it claims to be. Every
rule the last task put on the record file was about its place — who owns it,
who can write it, whether it is a link — and all of those are satisfied by a
record replaced whole by a believable copy. The one thing such a copy cannot
cheaply forge is agreement between the file's own first line and its entries.
Two disagreements are noticed while the file is read: an entry from before the
moment the record says it starts at (which no shortening leaves behind), and
moments that run backwards (an order that appending as things happen never
writes). Either is reported as *this record is not what it says it is*, in
whole sentences beside everything that could be read — never as a refusal of
the file, because a reader that threw the record away would let a forger choose
between being believed and being unreadable. A record the machine wrote and
shortened itself is one the check is silent about.

## What changed, in code

- `crates/alo-keeping/src/disagreeing.rs` — **new.** `Disagreement` (which
  lines disagree, and the two sentences) and the crate-private `Noticing` that
  builds it one entry at a time while the file is walked. Rustdoc carries the
  reasoning: why this is not a signature, why it is never a refusal, and why
  the legitimate cases are legitimate.
- `crates/alo-keeping/src/reading.rs` — `Reading` gains the `disagreement`
  field, computed inside `Reading::of` where the line numbers are known, and a
  `disagreement()` accessor. Both doors (`at` and `believed_at`) go through
  `of`, so there is no reading without the check.
- `crates/alo-keeping/src/words.rs` — two new strings,
  `keeping.disagreement.before-it-begins` and
  `keeping.disagreement.runs-backwards`, both opening with the promised
  sentence, each with a translator's note; `EVERY_WORD` is 19. Collected by
  `alo-saying` through the existing `declare_into`, so no collection change was
  needed.
- `crates/alo-keeping/src/lib.rs` — module, re-export, and the crate table.
- `crates/alo-keeping/tests/what_this_crate_says.rs` — the two new sentences
  are shown by something real (a record written by `Writing`, its head replaced
  by hand), so the *nothing is declared that nothing says* test keeps holding.
- `crates/alo-recounting/src/account.rs` — `Account` carries the
  `Disagreement`; `is_what_it_says_it_is()` and `disagreement()` beside the
  existing accessors, and `said()` answers the disagreement's sentences between
  the head's and the damage's. The module doc's "four ways an account is worth
  less than it looks" is now five.
- `crates/alo-recounting/tests/a_record_that_is_not_what_it_says.rs` — **new.**
  The acceptance, one test per criterion, read end-to-end through `Recounting`
  off a machine description on a disk, with the machine's real vocabulary
  (`alo_saying::everything_this_machine_can_say`).
- `docs/contracts/record-file.md` — the reader's rule added to "Reading one",
  additively. Not a byte of the file shape moved; the contract now says an
  entry *at* `since` and two entries in one moment are legitimate, which is the
  off-by-one a second implementation would otherwise get wrong.
- `docs/autonomy/v0-01-lane-b-plan.md` — task 6 marked done. Task 7 was
  already written, so no new task was added. No task in
  `v0-01-delivery-plan.md` matched this one, so nothing was marked there.

## Decisions taken (nobody was available to ask)

- **The check lives in `alo-keeping`, at read time.** The head, the entries and
  the line numbers meet only inside `Reading::of`; putting the rule there means
  no caller can read a record without it, and `alo-recounting` only carries the
  result into the account. The alternative — a check in `alo-recounting` —
  would have left the daemon's own reads unchecked and been skippable by any
  future reader.
- **Two named disagreements rather than one generic flag**, following
  `Damage`'s two-sentence precedent: which of the two happened is exactly what
  somebody inspecting a suspect file needs, and the join between two sentences
  is not punctuation a program can pick. Both open with the same claim, so the
  promised sentence is shown whichever way the record disagrees.
- **Which lines disagree are numbers beside the sentences**, never inside them
  — `Damage::unreadable()`'s i18n rule, kept.
- **Strictness:** only a *strictly earlier* moment disagrees, in both
  comparisons. An entry exactly at `since` is what a shortening's boundary
  legitimately leaves; equal neighbouring moments are a busy second. Both are
  measured.

## Verification

Platform: WSL Ubuntu on the Windows checkout (`CARGO_TARGET_DIR=$HOME/target-claude`),
which is where this workstream's gates run. All executed, none pending:

- `cargo fmt --all --check` — clean.
- `cargo clippy --workspace --all-targets -- -D warnings` — clean (also clean
  on the Windows host toolchain).
- `cargo test --workspace` — exit 0, 131 test-suite results, all ok. One
  pre-existing flake was observed once under contention
  (`alo-secrets connections_come_and_go`) and passes alone and in the full
  rerun; it is the shared-machine timing case the supervisor's second-run
  tolerance exists for, in a crate this change does not touch.
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` — clean.
- Each acceptance test run alone with `--exact` — one passing test each:
  - `a_record_starting_after_its_own_entries_is_reported_beside_everything_it_holds`
  - `moments_that_run_backwards_are_a_sentence_in_the_account_itself`
  - `a_record_legitimately_shortened_is_not_reported`
  - `what_this_machine_writes_reads_back_as_what_it_says_it_is`
  - `the_check_is_part_of_reading_rather_than_a_question_a_surface_may_forget`

The refusal paths are tested beside the legitimate ones throughout:
`crates/alo-keeping/src/disagreeing.rs` measures the forged head, the backwards
moments, the boundary entry, the shared moment, a disagreement noticed across
an unreadable line, the two-sentence rule, and the undeclared-vocabulary key.

Note for the Windows host: `alo-recounting`'s pre-existing `--lib` fixtures
that read a record through `believed_at` fail on the Windows toolchain with
`NotKept::NotRead { why: "unsupported" }` on clean `main` too — the believing
questions are Unix questions. The gates' own environment (WSL) is green.

## The gates refused this once, and what the second worker fixed

The supervisor's gates refused the first handoff — twice, which is the whole
of the second-run tolerance — on
`alo-secrets/tests/connections_come_and_go.rs`:
`each_keyring_is_a_connection_of_its_own_and_giving_it_up_closes_it` counted
6 connections where it expected `before + 4 = 7`. The note above calling it a
contention flake was wrong about the mechanism, and the mechanism is why it
could fail twice in a row: the fixture's readiness probe
(`alo-keyring-fixture`'s `wait_until_it_serves`) opens a connection per poll
and drops the successful one as `started` returns, and the bus goes on
listing a closed connection until it notices the socket shut. The test took
`before` in that window, counting a connection already on its way out — so
`before` was one high, and both later comparisons were against a beginning
that no longer existed.

The fix is in the test, and it is the smallest one that makes the measurement
honest: `before` is now taken by `once_it_is_quiet`, which polls until the
bus gives the same count for a whole second (bounded at ten) before believing
it. Nothing about what is measured changed — four keyrings are still four
connections, and giving them up still returns the count to where it began —
only the beginning is now one that has stopped moving. The fixture is
untouched: holding its probe connection open instead would have changed what
every consumer's bus looks like to spare one test a wait. Re-run five times
in a row after the fix, green each time.

## Limitations, honestly

- **A forged file that is also pruned becomes clean.** `Writing::prune`
  refuses a *damaged* record and does not yet refuse a *disagreeing* one, so a
  machine whose file was replaced and whose hourly shortening then runs would
  rewrite the evidence of the replacement away. The window is the retention
  rule's, and only on machines that set one. Making prune refuse a
  disagreement the way it refuses damage is a small, worthwhile follow-up; it
  was not done here because it adds a `NotKept` variant (a new public string
  and refusal path) beyond this task's acceptance, and half-doing it in the
  same change would have rushed both.
- **A clock stepped backwards between two entries reads as a disagreement.**
  The daemon writes moments as they happen; an operating-system clock step is
  the one honest way `runs_backwards` can appear. The sentence reports what
  the file shows and does not guess at causes, which is the most honest thing
  a reader can say; if it proves noisy on real hardware, `docs/quirks.md` is
  where that observation belongs.

## Proposed shared-document updates (for the integration owner)

- `CHANGELOG.md`: *An account of what the machine did now says when the record
  behind it is not what it says it is — an entry from before the record's own
  beginning, or moments out of order, is reported in words beside everything
  that could be read, and a record the machine wrote and shortened itself is
  never flagged.*
- `docs/autonomy/QUEUE.md` / `STATE.md`: lane B task 6 done, task 7 (the
  pinned model runtime on the image) is next and already written.
