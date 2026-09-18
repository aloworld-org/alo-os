# One list of crates that cannot drift from the workspace it describes

**Date:** 2026-09-17
**Workstream:** v0.5 — software and the web
(`docs/autonomy/v0-5-software-and-the-web-plan.md`, task 10)
**Contributor:** Claude Code, in `C:\dev\alo-os-3`; recovered by Codex in `C:\dev\alo-os`
**Status:** ready for integration

## What a person would notice

Nothing. This change is entirely about the repository: it removes a class of
failure where a lane is refused for a change it never made, and where the fix is
a file that lane's task never touched. A person using alo OS sees no difference,
and no sentence anybody reads in their own language was added, removed or
reworded.

## The problem, as it actually happened

Three times on 2026-09-17 a lane's gates failed on a list that has to move in
step with the workspace and had nothing tying it there:

- `alo-software`'s `the_terminal_is_a_persons_and_no_verb_reaches_it` spelled out
  the crates that declare verbs. It broke when `alo-converting` was added, and
  again when `alo-capturing` was (f9054b0).
- `alo-by-hand`'s `every_verb_can_be_done_by_hand` kept `WHO_DECLARES_THEM` and
  its own copy of the ten `declare_into` calls. It broke for `alo-capturing` too
  (38ccba8).
- `alo-saying`'s `EVERY_LIST` and `ONE_STRING_EACH` were arrays with their length
  written into the type. Two lanes each adding a crate merge cleanly — the
  entries are on different lines — and leave the length one short, so the
  combined tree does not compile. Five times on 2026-09-16 alone
  (`docs/quirks.md`, *Fixed-length lists of crates break when two lanes each add
  a crate*).

Each broke every lane on every machine until somebody fixed a list their own task
never touched.

## What changed

### `crates/alo-declared` — the one list, and the check that holds it

A new crate. It is the sibling of `alo-collected`, which does the same job for
the crates that declare **words**; this one is for the crates that declare
**verbs**.

- `src/shipped.rs` holds one registry pairing each crate name with its declaration
  function. The macro derives `WHO_DECLARES_THEM`, the combined verbs, and the
  per-crate declarations used by acceptance. There are no separate copies of
  those names and calls. Tests run every declaration and refuse duplicate names.
- `src/holding.rs` is the check. `held(listed, manifest, reading)` walks the
  workspace's own member list through `alo_by_hand::whoever_declares_verbs` and
  answers in two directions: a crate that declares verbs and is not on the list,
  and a name on the list that declares no verbs any more.
- `src/finding.rs` is what it says when they disagree. The finding this crate
  exists for names the crate, names the convention it read (`src/verbs.rs`), and
  names **the one file to add it in** — `crates/alo-declared/src/shipped.rs`,
  with one registry entry deriving `WHO_DECLARES_THEM` and the `declare_into` call — because its reader has
  just written that crate and has one registry entry and its Cargo dependency to write.
- `src/not_declared.rs` is the refusal when a crate's own verbs will not declare,
  naming the crate rather than handing on a message about a verb with no crate
  attached to it.
- `tests/the_one_list_of_crates_that_declare_verbs.rs` runs the check against
  this repository, and puts every finding in front of a fixture beside it.

### The two tests that kept copies

`crates/alo-by-hand/tests/every_verb_can_be_done_by_hand.rs` and
`crates/alo-software/tests/what_a_fresh_machine_has.rs` are both handed
`alo_declared::WHO_DECLARES_THEM` and
`alo_declared::every_verb_this_machine_ships()`, and keep nothing of their own.
Their dev-dependencies shrank accordingly: `alo-by-hand` now dev-depends on
`alo-declared` and `alo-files` (the fixtures need real verbs to be fixtures about
anything), and `alo-software` on `alo-declared` and `alo-adapters` (its other
tests use the latter).

### The lengths in `alo-saying`

`EVERY_LIST` is `&[&str]` and `ONE_STRING_EACH` is `&[(&str, &str)]`, with no
length written on either. `alo-collected`'s test passes `EVERY_LIST` rather than
`&EVERY_LIST`. The counts the written lengths were there for are unchanged and
still made — `the_lists_of_crates_agree` compares the two lists, and
`the_machine_says_what_the_crates_say_between_them` compares the machine's
vocabulary against the crates' own totals — because both compare one list against
another rather than against a literal.

`tools/kernel-loop`'s `REGISTRATIONS` became a slice for the same reason.

### `tools/kernel-loop` — the registration a plan may make

`inside_the_plan::REGISTRATIONS` is the short list of files any plan may change,
because a workspace test makes every new crate with words or verbs register
there. Its verb half named `crates/alo-by-hand/tests/…` and its `Cargo.toml`; it
now names `crates/alo-declared/src/shipped.rs` and that crate's `Cargo.toml`.
Without this, the next plan that adds a verb-declaring crate would be refused by
the supervisor for registering it — the exact failure the file exists to prevent,
one move behind.

### Documentation

- `docs/contracts/agent-verbs.md`, *Where verbs are declared, which that check
  reads*: an additive paragraph saying the list that is handed in is one list, in
  one file, and what to add to it. The rule above it — `src/verbs.rs` with a `pub
  fn declare_into` — is unchanged, and no public surface moved.
- `docs/quirks.md`: the 2026-09-16 entry gains a **Fixed, 2026-09-17** line
  saying what was done and that `WHO_DECLARES_THEM` did not become a slice where
  it was — it moved.
- The plan: task 10 marked done, `crates/alo-declared` named in the header, and
  no additional task appended.

## Decisions I made, and why

**A new crate rather than a module in `alo-by-hand`.** `alo-by-hand`'s own
documentation is emphatic that it holds no verb's name and reads no disk: it is
handed an `alo_capability::Verbs` and checks whatever list it is given, and its
test is the measurement. Putting ten crate dependencies and the list into it
would have made the method and the measurement one file, which is the split that
makes its refusals believable. `alo-collected` already set the precedent for a
separate crate whose whole job is holding a hand-written list to the workspace,
and it borrows `alo-by-hand`'s workspace walk exactly as this one does.

**Named `alo-declared`.** It pairs with `alo-collected` — words are collected,
verbs are declared — and reads as what it holds rather than as a mechanism.

**The dependency cycle is through dev-dependencies, deliberately.**
`alo-declared` depends on `alo-software`, `alo-by-hand` and eight others; those
two depend back on `alo-declared` only as dev-dependencies. Cargo permits that
and nothing else would work: the crate that assembles every verb must depend on
every crate that declares one, and the tests that ask questions of every verb
live in two of those crates.

**The check lives in `alo-declared`, not in `alo-by-hand`'s test.**
`alo-by-hand`'s `every_crate_that_declares_verbs_is_one_this_check_was_handed`
was deleted, and it is the only test removed. What it asserted is now
`the_one_list_is_the_crates_this_workspace_has` in `alo-declared`, run against
the same real workspace — and, separately, `alo-by-hand`'s own
`Finding::AVerbListNobodyHandedIn` still fires inside
`every_verb_this_machine_ships_can_be_done_by_hand`, which is handed the real
workspace reader. So the failure is caught twice, as it was before, and named in
one place a person is sent to.

**`DELIBERATELY_APART` was left as an array.** The acceptance names `EVERY_LIST`
and `ONE_STRING_EACH`. `DELIBERATELY_APART` has one entry, is not a list two
lanes both add to, and adding to it is a decision somebody argues for rather than
a registration — the failure mode this task is about does not reach it.

## What this does not claim

**The list does not write itself, and it cannot.** The obvious fix would be to
walk the workspace, find the crates with a `src/verbs.rs`, and call each of them.
A test cannot call a crate it does not depend on: `alo_files::declare_into` is a
path the compiler resolves against `alo-declared`'s own `Cargo.toml`, so the
calls are Cargo dependencies, and a dependency is written down by a person.
Nothing derived at build time reaches into the dependency graph to add one. The registry derives the names and declaration calls from each entry. The
workspace check derives **whether the list agrees with the workspace** — and that is what fails, naming the crate and the file, in the change
that adds the crate.

The same is true of `alo-saying`: its words reach the vocabulary through real
dependencies, and `alo-collected` derives only their agreement with the
workspace. Neither list maintains itself. Both now say so where they are, and
neither can drift without a check naming what is missing and where to put it.

## Original worker verification (2026-09-17, before recovery)

Historical results below describe the original worker tree, not the recovered
tree. They do not authorize publication of this integration. These former build
directories are recorded as provenance only and are not reused or recreated.

Run from `C:\dev\alo-os-3` through WSL Ubuntu against that checkout
(`/mnt/c/dev/alo-os-3`), with `CARGO_TARGET_DIR=/root/alo-builds/claude-declared`
for the workspace and `/root/alo-builds/claude-declared-loop` for the supervisor,
so neither reuses another lane's build directory.

| Check | Result |
|---|---|
| `cargo fmt --all --check` (workspace) | clean |
| `cargo fmt --all --check` (`tools/kernel-loop`) | clean |
| `cargo clippy --all-targets --workspace -- -D warnings` | clean |
| `cargo clippy --all-targets -- -D warnings` (`tools/kernel-loop`) | clean |
| `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` | clean |
| `cargo test -p alo-declared` | 16 passed (8 unit, 8 integration) |
| `cargo test -p alo-by-hand -p alo-collected` | 39 + 19 passed |
| `cargo test -p alo-software -p alo-saying` | 112 + 68 passed |
| `cargo test -p alo-citing -p alo-conforming` | 40 passed |
| `cargo test` (`tools/kernel-loop`) | 139 passed |

One clippy finding was fixed while gating: `len_zero` on
`assert!(verbs.len() > 0, …)` in the new test, now `!verbs.is_empty()`.

**Not run here:** `cargo test --workspace`. The supervisor runs it after this
worker, and the task's instructions are explicit that a worker must not spend the
hour on it.

**Not shown:** nothing on real hardware, and nothing needed to be — this change
touches no device, no bus and no daemon. The two-lane merge it prevents is shown
by construction (no length is written anywhere the two lists grow) rather than by
merging two branches.

## Recovery and integration (2026-09-18)

The supervisor recovered `parked/task-10-1789691356` (`e7b236b`) onto published
main `0e276811450c14886ed1c7b7ffd87a3acd4d85e8` in `C:\dev\alo-os`. The original
local implementation `77eebea` and parked snapshot remain preserved in
`C:\dev\alo-os-3`. Cargo integration retains the newer `alo-desktops` package;
vocabulary integration retains every newer registration and the published slice
fixes. Those slice fixes and the `alo-collected` caller were already on main.

The recovered implementation still repeated crate names and calls in its source
and acceptance test. A single registry now derives both and supplies per-crate
acceptance declarations. The original missing, extra and duplicate-crate refusal
checks remain. The unauthorized appended task 11 was excluded.

Current worker tests are deferred to the supervisor, which serializes validation
using this machine's existing gate lock and `/root/alo-builds/this-machine`.
`.kernel-loop/handoff.toml` requests all nine gates and the named task acceptance
checks on this integrated tree. No historical result above is a current pass.
The supervisor source changed and its binary must be rebuilt before publication.
The publication log and archived handoff provide the resulting commit's evidence.

## Acceptance, clause by clause

| Clause | Where it is held |
|---|---|
| No length is written anywhere: `EVERY_LIST` and `ONE_STRING_EACH` are slices | `crates/alo-saying/src/collecting.rs`; `collecting::tests::the_lists_of_crates_agree` and `…::the_machine_says_what_the_crates_say_between_them` still count |
| One list of verb-declaring crates, with the calls, in one place both tests use | `crates/alo-declared/src/shipped.rs`; `shipped::tests::every_name_has_a_call_and_every_call_has_a_name` |
| A test holds that list to `Cargo.toml`'s members through `alo_by_hand::whoever_declares_verbs` | `the_one_list_is_the_crates_this_workspace_has`, and `a_real_crate_left_off_the_real_list_is_refused` beside it |
| A missing crate is named, with where to add it | `a_crate_that_declares_verbs_and_is_on_no_list_is_named_with_the_file`; `finding::tests::a_finding_names_the_crate_and_the_file_to_add_it_in` |
| Only the check reads the workspace; the report says so plainly | *What this does not claim*, above; `crates/alo-declared/src/lib.rs` says the same where an author reads it |
| Proof it holds: a crate added to a fixture workspace is named, with the file | `a_crate_that_declares_verbs_and_is_on_no_list_is_named_with_the_file` |
| No test gets weaker | `alo-by-hand` keeps `AVerbListNobodyHandedIn` against the real workspace; the ten crates checked before are the ten checked now; `a_listed_crate_that_declares_no_verbs_is_a_finding` and `a_crate_named_twice_is_refused` are new |

*Not this task, and untouched:* the release number written in six places, which
belongs to the installer plan's `image/` and `alo-image`.

## Files

- `Cargo.toml` — `crates/alo-declared` as a member
- `Cargo.lock`
- `crates/alo-declared/Cargo.toml`
- `crates/alo-declared/src/lib.rs`
- `crates/alo-declared/src/shipped.rs`
- `crates/alo-declared/src/holding.rs`
- `crates/alo-declared/src/finding.rs`
- `crates/alo-declared/src/not_declared.rs`
- `crates/alo-declared/tests/the_one_list_of_crates_that_declare_verbs.rs`
- `crates/alo-by-hand/Cargo.toml`
- `crates/alo-by-hand/tests/every_verb_can_be_done_by_hand.rs`
- `crates/alo-software/Cargo.toml`
- `crates/alo-software/tests/what_a_fresh_machine_has.rs`
- `crates/alo-saying/src/collecting.rs`
- `tools/kernel-loop/src/inside_the_plan.rs`
- `docs/contracts/agent-verbs.md`
- `docs/quirks.md`
- `docs/autonomy/v0-5-software-and-the-web-plan.md`
- `docs/autonomy/updates/one-list-of-crates-that-cannot-drift.md`

## Proposed shared-document updates

For the integration owner (`docs/autonomy/SHARED_MAIN.md` — this contributor does
not edit `CHANGELOG.md`, `ROADMAP.md`, `docs/autonomy/QUEUE.md` or
`docs/autonomy/STATE.md`).

**`CHANGELOG.md`**, under Unreleased:

> **The lists of crates the repository keeps about itself cannot drift from it
> any more.** The crates whose verbs alo OS ships were written out in two tests
> that had no other reason to change, and a crate arriving broke both; the crates
> whose words it collects were written in arrays with their length in the type,
> and two people adding a crate at once merged cleanly into a workspace that
> would not compile. There is now one list of the verb-declaring crates, in
> `crates/alo-declared`, held to the workspace's own member list and naming both
> the crate and the file to add it in when they disagree; and the word lists
> carry no length. Nothing a person uses changed.

**`docs/autonomy/QUEUE.md` / `STATE.md`:** task 10 of the software-and-the-web
plan is implemented and awaits supervisor validation; task 9 remains
blocked on the owner.
