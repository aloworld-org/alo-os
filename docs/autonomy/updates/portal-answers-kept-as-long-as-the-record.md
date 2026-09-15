# What applications were answered is kept as long as the machine's record, and no longer

**Date:** 2026-09-15
**Workstream:** v0.5 — applications, and what they expect
(`docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`, task 9)
**Contributor:** Claude (Mac lane worker), for the repository's owner
**Status:** ready for integration

## What changed

Task 8 made the portal answers file durable, and it could only grow. The agent's
record is shortened under `[record].keeping`, the organisation's retention rule on
a managed machine (ADR 0004). Nothing shortened the answers file, so a machine
set to keep ninety days kept applications' requests forever. The answers file is
now shortened under the same rule.

- **`AnswersFile::shortened(keeping, now) -> Result<Shortened, NotRecorded>`**
  (`crates/alo-portals/src/shortening.rs`). It takes an `alo_keeping::Keeping`
  and a moment, and nothing else. It asks the rule `oldest_kept` and `keeps`, so
  what a day is and what a clock put back means are decided in `alo-keeping` and
  not restated. `Shortened` reports how many answers were removed and kept, and
  where the file now starts.
- **What goes:** every answer that reads and is older than the rule keeps. An
  answer at the rule's own edge stays. So does an answer dated after `now`, and
  on a clock that says 1970 nothing goes.
- **What never goes:** a line that did not read. It stays byte for byte, in its
  place. A torn last line is ended with a newline and stays one line.
- **A file that would lose nothing is not rewritten.** This covers `"forever"`
  and a rule nothing is old enough for yet. The first line is untouched too, and
  so is the inode.
- **The first line** (`crates/alo-portals/src/answers_head.rs`, split out of
  `answers_file.rs` because the head now has rules of its own) gains `since` and
  `under`, as the agent's record's first line has them. `since` is the later of
  the rule's edge and any `since` already there, so it never moves back. `format`
  stays `1`. `ReadBack` gains `since` and `under`.
- **Whole or not at all.** The shortened file is written **new** beside the file
  (`portal-answers.jsonl.shortening`), made `0600` under `believed_file`'s rules
  (`made_to_replace`: no link followed, create-new), synced, and renamed over the
  file. Anything left at that path is removed first. Removing a link removes the
  link and never what it points to. Whatever will not go, such as a folder with
  something in it, refuses the shortening.
- **Nothing answered meanwhile.** The file's lock is held from the first byte
  read to the rename. Every door keeps its answer under that lock before sending
  it (task 8), so no answer is kept or sent while the file is replaced. The new
  file is held open to append, and it becomes the file answers go into. There is
  no moment after the rename when the backend holds the file that was renamed
  away.
- **`NotRecorded::Replaced`**: the path no longer holds the file the backend
  has open (moved, removed or replaced underneath). Renaming over it would
  remove a file nobody asked this backend to shorten, so it is refused.
- `docs/contracts/portal-answers-file.md` gains `since` and `under` in *The
  first line*, and a *Shortening it* section. Both additions are additive, and
  `format` is still `1`.
- `crates/alo-portals/Cargo.toml` depends on `alo-keeping`, a workspace crate. No
  crate is added to the tree.

**User-readable change description (proposed for `CHANGELOG.md`):** *What your
applications asked for through portals, and what they were told, is now kept for
as long as the machine's record is kept and no longer. On a machine whose record
is kept for 90 days, applications' requests older than that are removed, and the
file says from when it now starts. Nothing is removed on a machine that keeps
everything.*

## Decisions

- **`Keeping` is used, not restated, and `alo-keeping` is not edited.** Its
  public `Keeping::oldest_kept` and `Keeping::keeps` are exactly the question.
  `alo_keeping::Head` can only be made inside that crate, so the answers file has
  its own head over the same two fields and the same forwards-only rule for
  `since`. No proposal to `alo-keeping`'s owner is needed.
- **An unreadable line is kept, not refused.** The agent's record refuses to
  shorten at all when a line is damaged. The plan asks for two things at once
  here: the rule holds over every answer, and a torn line survives. Refusing
  would break the first, and dropping would break the second. Keeping the line
  where it was does both, and the contract names the difference.
- **A shortening that removes nothing writes nothing.** That includes `since`.
  A first line claiming a shortening when nothing was removed would make the
  claim worthless. This matches `alo-keeping`.
- **Replacing the open file, not reopening the path.** The replacement is opened
  once, before the rename, and kept. Reopening the path after the rename can fail
  and would leave the backend appending into an unlinked file while
  `Recording::keep` said *kept*. Holding the replacement makes that state
  impossible.
- **The directory is not synced after the rename.** A power cut leaves either
  the old file or the new one at the path, and both are correct records. This
  matches `alo-keeping`.
- **The `keeping::` guard in `open_with_is_answered_from_what_opens_what.rs` was
  made exact.** It exists to catch `alo_applications::keeping` (the person's
  *what opens what* file), and `alo_keeping::Keeping` matched it. The check now
  ignores `keeping::` only when the character before it is `_`. It still catches
  `keeping::` reached any other way (`alo_applications::keeping::`,
  `use …::{keeping}` then `keeping::…`). An alias such as `use alo_keeping as
  retention` would have hidden the word from the guard. I chose to make the guard
  exact instead, and say so here.
- **When a shortening runs, and reading `[record].keeping` off the machine
  description, are not here.** The plan's constraint says so. They belong to the
  session that starts the backend, as they do for the agent's record.

## Acceptance criteria and their tests

All in
`crates/alo-portals/tests/what_applications_were_answered_is_kept_as_long_as_the_record.rs`:

| Criterion | Test |
|---|---|
| Shortened under `[record].keeping` through `Keeping`. An answer past the rule is gone and one inside it is not. `since`/`under` are in the format line and `format` stays `1` | `the_file::an_answer_past_the_rule_is_gone_and_one_inside_it_is_not` |
| `"forever"` removes nothing (nor a young rule, nor a clock in 1970), and nothing is rewritten | `the_file::forever_removes_nothing_and_touches_nothing` |
| Never a line that did not read. A torn line survives | `the_file::a_line_that_did_not_read_survives_a_shortening` |
| A file shortened twice still says it was | `the_file::a_file_shortened_twice_still_says_it_was` |
| A shortening that cannot write leaves the file as it was (a folder in the way, the path replaced or removed, a newer format), and a link where the replacement goes is never written through | `the_file::a_shortening_that_cannot_write_leaves_the_file_as_it_was` |
| Whole or not at all, and the backend answers nothing while the file is replaced | `the_file::no_answer_is_kept_while_the_file_is_replaced` |
| The contract gains the two fields additively | `the_contract_names_where_a_shortened_file_starts` |

Unit tests: `answers_head::tests` (four tests: a fresh head is task 8's byte for
byte, `since` only moves forwards, round trip, refusals) and
`shortening::tests::the_replacement_is_written_beside_the_file`.

**The concurrency test was checked against the bug it exists for.** I released
the lock between reading and renaming, then put it back after the rename, and
ran the test three times. It failed all three with answers missing from the file.
The code was then restored byte for byte (`diff` against the saved copy).

## Verification

Run on the Mac lane's Lima VM (`alo`, Ubuntu 24.04 aarch64, as root), with
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-a96435785b5e7d7d`, the checkout shared
with the Mac:

- `cargo fmt --all` (on the Mac): clean.
- `cargo clippy -p alo-portals -p alo-secrets -p alo-granted -p alo-saying --all-targets -- -D warnings`: clean.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p alo-portals`: clean.
- `cargo test -p alo-portals`: all pass (40 unit; integration 8 + 9 + 5 + 5 + 5
  + 11 + 7; 1 doc).

On the Mac itself the crate's tests do not build: `alo-sessiond` (a dependency
elsewhere in the graph) calls `rustix::net::sockopt::socket_peercred`, which
rustix does not offer on macOS. That was already true before this change, which
is why the lane gates in the VM.

**Not run:** the full workspace suite, which the supervisor runs.
**Not measured:** a real full disk during a shortening. Write failure is shown
by a folder in the way of the replacement, which also stops root.
**Not physical acceptance:** no session runs a shortening on a timer, and no
machine description is read. Both are outside this plan.

## Limitations

- Nothing calls `shortened` on a machine yet. When it runs, and reading
  `[record].keeping`, are the session and image work.
- A second backend process writing the same file at the same time is not
  guarded. The lock is per process, and a machine runs one backend.
- Nothing in this change words the file for a person. Task 10, now written in the
  plan, does that.

## Proposed updates to shared documents

- **CHANGELOG.md:** the change description above.
- **docs/autonomy/QUEUE.md / STATE.md:** v0.5 applications plan, task 9 done
  (this report). Task 10 is written and ready.
- **docs/contracts/machine-description.md** (not edited here: another lane's
  specification). The `keeping` row could add that the rule also governs the
  portal answers file (`docs/contracts/portal-answers-file.md`, *Shortening it*).
