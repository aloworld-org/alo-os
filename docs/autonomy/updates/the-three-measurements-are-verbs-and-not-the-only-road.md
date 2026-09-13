# The three measurements are asked the way everything else is asked

- Date: 2026-09-13
- Workstream: v0.5 the machine, measured, task 5 (`docs/autonomy/v0-5-the-machine-measured-plan.md`)
- Contributor: Claude Code, as a development worker under the kernel-loop supervisor
- Status: **ready for integration**

The agent's *"where is that file?"* and *"why is it slow?"* are verbs, and a
verb is what ADR 0001 says it is: on a closed list, checked against a grant,
recorded. This task declares the three measurements as verbs in the crates
that answer them, carries each out under an authorisation the capability
model made, hands the authorisation back for the record — and shows, with a
test in each crate, that the same answers are reached with no agent and no
grant at all, because a person searching their own files is not an agent and
is not asking anybody.

## What changed

**Three read verbs, declared in the shape `alo-files` declares its six.**
`crates/alo-finding/src/verbs.rs` declares `search_files`, with `folder`
(path) and `named` (name, at most `A_NAME` characters), whose sentence is
*search the index of {folder} for files whose name contains {named}*.
`crates/alo-measuring/src/verbs.rs` declares `what_is_running`, with `proc`
(path) — *list what is running and what it is using, read from {proc}* — and
`what_is_filling`, with `folder` (path) — *count what is filling {folder}*.
Every one is `Effect::Read`, so it runs inside the turn, nobody is asked to
approve it, and the record's entry has no approval to name. Every one
requires a grant over the one folder it reads. Each crate hands its verbs
over through a `pub fn declare_into` on its own `Verbs`, all or none, with a
name already taken refused rather than replaced; `finding_verbs()` and
`measuring_verbs()` are the lists on their own. The root `declare_into` of
each crate stays the **words**' one, which `alo-saying` already calls, so the
verbs' is reached as `alo_finding::verbs::declare_into` and
`alo_measuring::verbs::declare_into` — additive, and nothing that existed
moved.

**Two doors from an authorisation to an answer.**
`crates/alo-finding/src/searched.rs`: `Searched::of(Touching, &Index)` takes
a call the capability model permitted and `alo_files::Touching` made real —
the folder resolved and the grants asked again about where it really leads
— checks the index it was handed is of exactly that folder, builds the query
from the call's one name, and asks `Index::answer`. It hands the
`Authorised` back through `into_parts`, whether or not the index answered.
`crates/alo-measuring/src/measured.rs`: `Measured::of(Touching, interval,
waiting)` does the same for the two measurements: two readings of the
granted directory through `Reading::of_kernel(&Disk, proc)` with the
caller's `waiting` between them, made into rates by `Reading::since` over
the caller's `interval`; or `Holding::of` on the granted folder. A
`Measurement` is `Running` or `Filling`.

**What can be said when a permitted call is not answered.**
`crates/alo-finding/src/unanswered.rs`: `NotAnswered` — the index's own
refusal of a query that is not one, an index of another folder, a verb that
is not this crate's, an argument that did not arrive — each said in the
person's language. `NotMeasured` in `alo-measuring` gains `NotThisCrates`
and `Missing` for the same two cases; it was `#[non_exhaustive]`, so the
addition breaks nothing.

**Words.** Six new strings under `finding.` and eight under `measuring.`:
each verb's purpose, sentence and argument purposes, and the two refusals
above — every one with a note for whoever translates it. The finding list is
now forty-one and the measuring list twenty-five; `alo-saying` counts each
crate's words at run time and its suite is green with them.

**The guard tests, widened by exactly two files each, and made stricter.**
`tests/nothing_here_opens_a_socket_or_asks_anybody.rs` and
`tests/nothing_here_acts_or_asks_who_is_asking.rs` forbade `alo_capability`
in every shipped file, and a verb cannot be declared without naming it. They
now hold both halves of the new fact: `verbs.rs` and the door (`searched.rs`
/ `measured.rs`) **must** name the capability model, and **no third file
may** — a third file naming it would be the index, the walk or the reading
starting to care who asked. `Grant`, `Caller`, `Agent` and the rest stay
forbidden everywhere, including in the two. The dependency lists are pinned
again at their new lengths, and the one dev-dependency each crate now has,
`alo-record`, is pinned by name with the reason: so a test can show a
measurement being written down rather than stopping at the authorisation
coming back.

**The registrations that had to agree.** `crates/alo-by-hand` walks the
workspace for any `src/verbs.rs` with a `declare_into` and refuses a crate
it was not handed, so its test now hands it `alo-finding` and
`alo-measuring` beside `alo-files` and `alo-applications`, and its manifest
takes them as dev-dependencies. `docs/by-hand.md` answers the three verbs —
each quoting the promise in `docs/features.md` that gives a person the same
thing without the agent. `docs/contracts/agent-verbs.md` gains *The
measurement verbs* and a row in the verb classes. `Cargo.lock` follows the
new edges.

**Front pages.** `lib.rs` of both crates gains a section — *an agent asks
the same index/numbers, under a grant* — and the table rows for the new
types.

## Decisions

**`what_is_running` takes `/proc` as an argument, and the grant is over it.**
The acceptance says each verb is *evaluated against a grant naming the
directory it reads*, and the directory *what is running* reads is `/proc`.
The alternative was `Requires::nothing_because(...)`, which would have let
any agent enumerate every process on the machine at will; a process list is
a fingerprint of who somebody is and what they do, which is exactly the
argument `alo-applications` makes for asking the grants before the installed
list. So the kernel's directory is an argument, the grant that permits the
verb names exactly what the verb reads, and revoking it stops the verb the
same instant it stops `list_folder`. A second benefit fell out: a kernel a
test writes into a directory of its own is measured by exactly the code that
measures the real one, so the whole road runs on every host.

**No interval on the verb.** A rate is two readings with time between them,
and a count of seconds the model chooses would be a way to make a turn wait
as long as the model likes. Whoever carries the verb out chooses the
interval and passes it in, and passes the waiting with it, so a test's kernel
can change between the readings without anybody sleeping and the rates are
over exactly the interval that was waited. A call of `what_is_running` with
an `over` argument is refused at the door, and a test says so.

**`search_files`, by name only, and not `find_in_folder` again.**
`find_in_folder` walks the disk; this asks the index the file manager asks,
answers with what matched beside what the index does not hold, and never
touches the folder. The name says which. It asks by name and by nothing
else: every argument of a verb is required, so a verb that also took words
would make every search by name carry a sentence of words, and a verb per
axis would be four names for one action on a list a model picks from. The
other three axes are the window's, and a second verb for words is a decision
for `docs/features.md`.

**`alo_files::Touching` is the check, not a second one.** The plan says the
verbs are declared *the way `alo-files` declares its own*; `Touching::of` is
already the three questions the contract orders — granted as written, where
it really leads, granted there — and a second resolver in either crate would
be a second opinion about what a grant covers. Both doors take a `Touching`
by value and hand back the `Authorised` inside it.

**The index is handed in.** `Searched::of` takes `&Index` rather than
finding one, because nothing in `alo-finding` may read the environment and
nothing here may walk a folder to make an index on an agent's behalf. It
checks the index is of the real granted folder and refuses otherwise. Who
holds the indexes for a folder is the gap this leaves, and it is written as
task 6 in the plan.

**Not offered by a turn yet.** `alo-turn`'s `Machine::carrying_out_file_verbs`
offers exactly the file verbs it has an executor for, and adding an executor
and adding to the offered list is one edit there. `alo-turn` is lane A's
this week and this plan may not edit it, so the three are declared, carried
out and recorded here, and the contract says in so many words that they are
not yet on the list a shipped machine offers. That is the honest state
rather than a narrowing of the promise: the acceptance asks for
declaration, evaluation, refusal and record, and for the road that needs no
agent, and all of that is here.

**`alo-capability` was read and not edited.** Nothing about declaring the
three needed a change there; the constraint's finding is that none was
needed.

**The record tests cross a crate on purpose.** *Every execution and every
refusal leaves a record* is a guarantee `CLAUDE.md` names, and a test that
stopped at `into_parts` would be taking it on trust. So each crate's
integration test writes `Entry::ran` and `Entry::refused` into a real
`Record` and reads them back, as `alo-applications`' does — which is why
`alo-record` is a dev-dependency and why the "nothing rented" tests now pin
it by name rather than forbidding the section.

## Acceptance criteria and the tests that hold them

| Acceptance | Test |
|---|---|
| three read verbs, declared in the shape `alo-files` declares its own, an argument each and a sentence each | `alo-finding` `verbs::tests::the_list_is_the_one_search_verb`, `alo-measuring` `verbs::tests::the_list_is_the_two_measurements`; the sentences in `verbs::tests::what_a_person_reads_is_a_sentence_naming_the_folder` (both crates) |
| every one is a read: no approval is asked | `alo-finding` `verbs::tests::a_search_is_a_read_that_nobody_is_asked_to_approve`, `alo-measuring` `verbs::tests::both_are_reads_that_nobody_is_asked_to_approve` |
| the record's entry has no approval to name, and says the verb ran | `search_files_is_asked_the_way_everything_else_is::a_search_under_its_grant_answers_inside_the_turn_and_is_recorded_with_no_approval`; `the_measurements_are_asked_the_way_everything_else_is::what_is_running_under_a_grant_over_proc_answers_inside_the_turn_and_is_recorded` and `…::what_is_filling_under_a_grant_over_the_folder_answers_inside_the_turn_and_is_recorded` |
| each is evaluated against a grant naming the directory it reads | `alo-finding` `verbs::tests::the_grant_is_over_the_folder_whose_index_is_searched`, `alo-measuring` `verbs::tests::each_requires_a_grant_over_the_folder_it_reads` |
| refused outside the grant, before anything is read, and the refusal is recorded | `…::a_folder_outside_the_grant_is_refused_before_the_index_is_asked_and_the_refusal_is_recorded`, `…::a_directory_outside_the_grant_is_refused_before_anything_is_read_and_the_refusal_is_recorded` |
| a grant revoked after the authorisation, or expired, still stops it | `…::a_grant_taken_away_or_run_out_stops_the_search`, `…::a_grant_taken_away_or_run_out_stops_the_measurement` |
| the same three answers are reachable with no agent and no grant, by calling the crates directly | `…::a_person_with_no_agent_and_no_grant_gets_the_same_answer`, `…::a_person_with_no_agent_and_no_grant_gets_the_same_numbers` |
| a call that is not one never reaches the grants or the disk | `verbs::tests::a_call_that_does_not_survive_the_door_is_not_a_call` (both crates), `…::a_call_that_is_not_one_never_reaches_the_grants_or_the_index`, `…::a_call_that_is_not_one_never_reaches_the_grants_or_the_kernel` |
| a door answers only its own crate's verbs; an index of another folder does not answer for the granted one | `…::only_the_search_verb_is_this_crates_to_answer`, `…::only_the_two_measurements_are_this_crates_to_answer`, `…::an_index_of_another_folder_does_not_answer_for_the_granted_one` |
| every way of not answering is a sentence a person reads | `alo-finding` `unanswered::tests::every_way_of_not_answering_is_said_in_a_sentence_a_person_reads`, `alo-measuring` `refusing::tests::every_refusal_is_said_in_a_sentence_a_person_reads` |
| every word a verb is declared with is one its crate declares | `verbs::tests::everything_the_verb_says_is_something_this_crate_declares`, `verbs::tests::everything_the_two_say_is_something_this_crate_declares`; the lists in `words::tests::the_list_declares_into_a_vocabulary_once` (both) |
| constraint: the capability model is named by the verb and its door and by nothing else; nothing else rented | `nothing_here_opens_a_socket_or_asks_anybody::only_the_verb_and_its_door_name_the_capability_model` and `…::the_walk_is_alo_files_and_nothing_else_is_rented`; `nothing_here_acts_or_asks_who_is_asking::only_the_verbs_and_their_door_name_the_capability_model` and `…::the_numbers_come_from_proc_and_from_no_rented_crate` |
| rule 7 of adding a verb: every verb names how a person does it by hand, and every declaring crate was handed to the check | `alo-by-hand` `every_verb_can_be_done_by_hand::every_verb_this_machine_ships_can_be_done_by_hand` and `…::every_crate_that_declares_verbs_is_one_this_check_was_handed` |

## Verified

WSL Ubuntu on this development machine (Dell Latitude 5550, Intel Core Ultra
7 155U, 15 GiB), as root, `CARGO_TARGET_DIR` the supervisor's own
(`$HOME/alo-builds/alo-os-b-72aa4fda7f7d8fc1`), foreground, exit codes read:

| Gate | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings` (whole workspace) | clean, 0 warnings, exit 0 |
| `cargo test -p alo-finding -p alo-measuring -p alo-by-hand -p alo-saying` | all passed: finding 53 unit + 3 + 4 + 7 + 8 + 3 + 2 doctests; measuring 60 unit + 4 + 7 + 7 + 5 + 3 + 2 doctests; by-hand 27 + 13; saying 63 + 4 + 1 |
| `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-finding -p alo-measuring --no-deps` | clean |
| every evidence line, alone, `--exact` | 1 passed each, 34 lines |

Windows, this development machine: `cargo fmt --all`; `cargo clippy -p
alo-finding -p alo-measuring -p alo-by-hand --all-targets -- -D warnings`
clean; `cargo test -p alo-finding`, `-p alo-measuring`, `-p alo-by-hand` all
passed (the `what_is_filling` road on Windows answers `NotOnThisHost` and is
still recorded as having run, which the test asserts on that host).

**Not run by this worker:** the full workspace suite, per the task's
instruction; the supervisor runs it. **Not verified:** anything on a
certified machine; nothing here opens a window.

## Remaining limitations

- **Not offered by a turn.** `alo-turn`'s machine does not carry the three;
  that edit is lane A's crate this week, and the contract says so.
- **Who holds the index** for a granted folder is the caller's, and task 6
  in the plan is written to close it.
- **A search by kind, date or words is the window's**, not the verb's.
- **The new words have no Polish yet** in the two crates' translation
  tests; they are declared, noted and checked in English, and the Polish
  fixtures are unchanged.
- **The interval `Measured::of` is given is trusted**: the rates are over
  the interval the caller says it waited, which is the crate's existing
  contract for `Reading::since`; a daemon that sleeps for it on a loaded
  machine may have waited slightly longer.

## Proposed changelog entry

*The three measurements are verbs* (v0.5): `search_files`,
`what_is_running` and `what_is_filling` are declared in the crates that
answer them, each a read over the one folder it reads — for *what is
running* that folder is the kernel's own, `/proc`, so the grant names exactly
what is read. Each runs inside the turn with no approval to name, is refused
outside its grant before anything is read, and leaves a record whether it
ran or was stopped. And each answer is still reachable by a person with no
agent and no grant, because the verb is a road to the same function the
window asks. The verbs are declared and carried out; a turn does not offer
them yet.

## Proposed queue and roadmap updates

Task 5 of `v0-5-the-machine-measured-plan.md` is marked done in the plan.
Task 6 — which folders are indexed, and the index for a folder found by its
name — is written there, ready, depending on 3 and 5. Offering the three
verbs from `alo-turn`'s machine is one edit in lane A's crate when the lane
split allows it.
