# A staging decided from an approval that arrived from elsewhere

**Date:** 2026-09-19
**Workstream:** v0.5 — the machine keeps itself
(`docs/autonomy/v0-5-the-machine-keeps-itself-plan.md`, task 9)
**Contributor:** this development PC's lane
**Status:** ready for integration.

## What this is

`alo-keeping-up` decides the one instruction the base is given when an update is
applied. Until today it could decide it from exactly one thing: a `Ready`, which
exists only inside a check **this** machine made and put on the indicator. That
is the right shape for the road a person walks through the shell, and it is the
wrong shape for the road [ADR
0053](../../decisions/0053-an-update-is-carried-out-by-a-unit-the-broker-starts-never-by-the-broker.md)
describes, where what crosses into the privileged unit is an **approval** — a
`{from, to}` pair a person approved — with no `Ready` behind it and nothing of
this crate's in it.

So the crate gains a second door onto the same instruction, and the plan that
needs it (`v0-5-the-broker-and-the-disk-plan.md` task 8) is not allowed to write
it, because `alo-keeping-up` is this plan's crate.

**What must not happen is a second way to stage.** That is the whole risk here:
the temptation is for the broker's unit to assemble the base's arguments itself
out of two digests, and then two places know what staging an update means and
one of them drifts. This change is shaped so that cannot happen — the two doors
are not two implementations that agree, they are two callers of one private
decision.

## A user-readable change description

A person can now be told that a version of alo OS they approved somewhere else
— through the assistant, with the approval carried across to the part of the
machine that is allowed to install it — is prepared in exactly the same way as
one they chose in Settings. The machine checks, at that moment rather than at
the moment they approved, that it is still running the version they were told it
would change from; that the version they approved is not already waiting; and
that the two versions named are actually different. If any of those is not true
it says so in its own sentence and changes nothing. An update approved this way
always waits for a restart the person makes: it can never restart the machine
by itself.

## What changed

- `crates/alo-keeping-up/src/staging.rs`
  - **`Staging::approved(from, to, deployments, source)`**, the second door. It
    takes two digests and the base's `Deployments` **read now** — after the
    approval, before the instruction is made — so every refusal is about the
    machine as it is rather than as it was when somebody approved.
  - **`Staging::of` and `Staging::approved` both call one private `decided`.**
    `of` keeps its signature exactly; nothing that used it changed.
  - **`NotStaged::NotAnUpdate`**, the fourth refusal: `from` and `to` are one
    build. It is the refusal `Staging::of` never needed — `Standing::between`
    answers *up to date* rather than making a `Ready` from one build twice — and
    the one an approval made elsewhere can carry, because an approval is only
    two digests.
  - Eight new tests, listed under *Evidence*.
- `crates/alo-keeping-up/src/words.rs` — `NOT_AN_UPDATE`
  (`keeping-up.not-an-update`), with a translator's note; `EVERY_WORD` is 43 → 44.
- `crates/alo-keeping-up/src/lib.rs` — the crate header says there are two doors
  and one instruction.
- `crates/alo-updating/tests/what_a_person_is_told.rs` — the new sentence is
  reached through its real constructor in `every_sentence_reached`, so task 5's
  *nothing declared is unreachable* stays a measurement; and it joins the
  refusals that must read differently from every other refusal on the road and
  must tell the person their machine is unchanged.
- `docs/autonomy/v0-5-the-machine-keeps-itself-plan.md` — task 9 marked done;
  task 10 written, because the plan named nothing after it.
- `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md` — task 8's status line no
  longer names two blockers that are closed. See *A stale blocker is invisible
  work* in `SHARED_MAIN.md`: the machine that finishes a task clears the lines
  naming it. Only the blocker clause changed; the acceptance and constraint are
  untouched.

## Decisions I made, and why

### The new constructor takes no `WhenItApplies`, and always stages for the next restart

This is the one thing the task left open, and it is the decision worth reading.

The obvious shape is to mirror `Staging::of` and take a `WhenItApplies`, on the
argument that a narrower door is a second, lesser way to stage. I chose the
other way, for a reason that comes out of what an approval actually is: **ADR
0053 §B says the bytes handed over are `{from, to}`, and the approved identity
is the digest of exactly those bytes.** There is no `when` in an approval. A
constructor that took one would be asking its caller to invent a value the
person never approved — and the value it could invent is *restart now and apply
it*, which closes the applications somebody has open.

So `Staging::approved` decides `WhenItApplies::AtTheNextRestart` and the
instruction it writes can never contain `--apply`. ADR 0053's *no instruction
the broker causes carries `--apply`* is then true by construction in the crate
that owns the instruction, rather than by the unit on the other side of the
boundary remembering to pass the right enum member. A promise held where the
argument list is written is a different quality of promise from one held by a
caller's good manners.

What this costs: a future caller with a legitimate *restart now* that did not
come through a `Ready` cannot use this door. That is the right cost. Such a
caller would have something more than `{from, to}` in its hands, and the honest
response then is to widen the approval's shape and say so, not to leave a
restart reachable from a door whose input cannot express one.

### One private decision, not two constructors that agree

`of` and `approved` both delegate to a private `decided`. The identity the plan
asks for — *the two `arguments()` are identical, element for element* — is
therefore structural rather than something a test has to keep true. The test is
still written, because a structural guarantee that nothing measures is one
refactor away from being prose; it is the first item under *Evidence*, and I
measured it failing before trusting it (below).

### `NotAnUpdate` is decided before the machine is read

`from == to` is a fact about the approval, not about the machine, so it is
answered first. The alternative — read the status, then notice — would report a
machine that cannot be named (`NotRunningABuild`) for an approval that was never
an update in the first place, which tells the person about the wrong thing.

### An approval whose two digests arrived swapped

There is no separate refusal for it and there should not be: a swapped approval
says *change from the build you are about to run, to the build you are running*,
and the machine is not running the build it claims to be changing from. That is
exactly `TheMachineMovedOn`, which already carries both digests so a surface can
say what moved. A test pins this so nobody adds a fifth refusal for it later.

### The sentence

> This machine was asked to change its system from one version to that same
> version, so there was nothing to do and nothing was changed

It names no machinery, does not hedge, ends on *nothing was changed* like every
other refusal on this road, and reads differently from all of them. It does not
blame the person, because it is not about anything they did — the translator's
note says so, since a translator with only the string could easily make it an
accusation.

## Findings handed back, not repaired

**ADR 0053 still reads *proposed*.** This plan's task 9 says the owner accepted
it on 2026-09-19, option B. The decision file's own status line says *proposed,
2026-09-17*, and `crates/alo-brokerd/tests/the_updates_wait_on_their_decision.rs`
— which is written to start failing the day that line changes — passes today.
Accepting a decision is the owner's act and not a worker's, so nothing was
changed to match. The consequence is concrete and belongs in front of whoever
picks up the broker lane: **the broker plan's task 8 is now blocked on that one
line and on nothing this crate owes.** Its other two blockers are closed —
`alo_egress::Errand::FetchingAnUpdate` exists, and `Staging::approved` exists as
of this change.

**A date in the plan.** Task 9's own status line says it was *written
2026-09-20*; the commit that wrote it is dated 2026-09-19. Left as its author
wrote it.

## Verification

Platform: Windows Server 2022 host, Ubuntu 24.04 under WSL2, run in the
serialized Linux tree the gates use (`/root/alo-trees/this-machine`, built into
`/root/alo-builds/this-machine`) after an `rsync --checksum --no-times` of this
checkout, which is how `tools/kernel-loop/src/gates.rs` runs them. The gate-turn
directory was empty and no Cargo process was running when this began.

The product workspace is `.` throughout.

| Command | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy -p alo-keeping-up -p alo-updating --all-targets -- -D warnings` | clean, zero warnings |
| `cargo test -p alo-keeping-up` | ok — 82 + 9 + 6 + 10 passed, 0 failed |
| `cargo test -p alo-updating` | ok — 60 passed, 0 failed, 5 ignored (the virtual-machine and real-registry tests, `#[ignore]`d as they were before) |
| `cargo test -p alo-saying` | ok — 68 passed, 0 failed (run because a word was added to a crate it collects) |
| `cargo doc -p alo-keeping-up -p alo-updating --no-deps` | clean, zero warnings |

Not run, deliberately: the full workspace suite, the supervisor's own three
gates and the BPF target's two. The supervisor runs all nine after this.

On Windows directly, `cargo fmt --all` fails with *The filename or extension is
too long (os error 206)* — the command line rustfmt is handed exceeds the host
limit for a workspace this size. `cargo fmt -p <crate>` works there, and the
whole-workspace check passes in WSL, which is where the gate runs it. Recorded
here because the next worker on this machine will meet it.

### Evidence, one line per acceptance criterion

| Criterion in the plan | Workspace | Crate | Target | Test |
|---|---|---|---|---|
| a constructor deciding the same instruction from an approved `{from, to}` and the base's `Deployments` read now | `.` | `alo-keeping-up` | `--lib` | `staging::tests::an_approval_and_a_ready_decide_the_same_instruction_element_for_element` |
| a test takes one `Ready`, decides both ways, and asserts the two `arguments()` are identical element for element | `.` | `alo-keeping-up` | `--lib` | `staging::tests::an_approval_and_a_ready_decide_the_same_instruction_element_for_element` |
| refuses: the machine is no longer running `from`, carrying both digests | `.` | `alo-keeping-up` | `--lib` | `staging::tests::an_approval_for_a_build_this_machine_no_longer_runs_is_refused_with_both_builds` |
| refuses: `to` is already staged | `.` | `alo-keeping-up` | `--lib` | `staging::tests::an_approval_for_a_build_already_waiting_is_not_staged_again` |
| refuses: the base reports no running build | `.` | `alo-keeping-up` | `--lib` | `staging::tests::an_approval_on_a_machine_running_no_build_is_not_staged` |
| refuses: `from` and `to` are the same digest | `.` | `alo-keeping-up` | `--lib` | `staging::tests::an_approval_naming_one_build_twice_is_not_an_update` |
| an approval whose digests arrived swapped is refused (the decision above) | `.` | `alo-keeping-up` | `--lib` | `staging::tests::an_approval_whose_two_builds_arrived_swapped_is_refused` |
| nothing here restarts the machine: the approved door never carries `--apply` (the decision above) | `.` | `alo-keeping-up` | `--lib` | `staging::tests::an_approval_stages_for_the_next_restart_and_never_applies_at_once` |
| no two refusals read alike | `.` | `alo-keeping-up` | `--lib` | `staging::tests::no_two_refusals_an_approval_can_meet_read_alike` |
| every new refusal is in the vocabulary with a translator's note | `.` | `alo-keeping-up` | `--lib` | `words::tests::every_word_carries_a_note_and_only_the_offer_to_put_back_has_gaps` |
| no sentence names the machinery | `.` | `alo-keeping-up` | `--lib` | `words::tests::nothing_here_names_the_machinery_hedges_or_calls_an_update_urgent` |
| every new refusal is reachable from a public `said()`, and declared nowhere it cannot be reached | `.` | `alo-updating` | `--test what_a_person_is_told` | `every_sentence_these_crates_can_say_is_in_the_vocabulary_with_a_note` |
| the new refusal reads differently from every other refusal on the road, and says the machine is unchanged | `.` | `alo-updating` | `--test what_a_person_is_told` | `every_refusal_on_the_road_is_said_and_no_two_read_alike` |

Thirteen criteria, twelve tests — the first two rows are the same test, because
*decides the same instruction* and *the two `arguments()` are identical element
for element* are one measurement. Each of the twelve was run on its own with
`-- --exact <name>`, and each reported `test result: ok. 1 passed; 0 failed`.

### The identity test was measured failing

A structural guarantee nothing measures is prose. In the Linux tree only, and
put back immediately afterwards, `Staging::approved` was changed to pass
`WhenItApplies::NowBecauseThePersonAsked` to the shared decision — the smallest
way to make the two doors disagree:

- `an_approval_and_a_ready_decide_the_same_instruction_element_for_element` —
  `FAILED`, panicking at the `arguments()` comparison.
- `an_approval_stages_for_the_next_restart_and_never_applies_at_once` —
  `FAILED`, panicking on the `--apply` assertion.

Both passed again once the file was restored, and the published tree is the
restored one.

## Limitations

- **Nothing has walked through the second door.** It exists for the unit ADR
  0053 describes, and that unit is the broker plan's and is not written. This
  task deliberately builds only the decision: this crate still names no program,
  opens nothing, starts nothing and has gained no dependency.
- **No real machine was updated.** That is task 8's acceptance and task 8 is
  blocked on the installer lane, for the reasons task 7 measured. Nothing here
  changes that, and nothing here should be read as having measured it.
- Whether a build is vouched for is untouched. `Staging::approved` does not ask
  and does not decide; the machine's signature policy answers the question that
  matters, at the moment the base is told, and
  `--enforce-container-sigpolicy` is in the arguments this door writes exactly
  as it is in the other's.

## Proposed shared-document updates

For the integration owner; I have not edited any of the four.

**`CHANGELOG.md`**, under the unreleased heading:

> An update a person approves elsewhere on the machine — through the assistant,
> rather than in Settings — is now prepared in exactly the same way as one they
> choose themselves, and is checked against the machine as it is at that moment
> rather than as it was when they approved. It refuses, each in its own words
> and without changing anything, when the machine is no longer running the
> version the update was found against, when that version is already waiting for
> the next restart, when the machine cannot say which version it is running, and
> when the two versions named are the same one. An update approved this way
> always waits for a restart the person makes.

**`ROADMAP.md`:** no change. This is part of *updates that never interrupt*, and
that line is not complete until a machine can apply one (task 8).

**`docs/autonomy/QUEUE.md`:** the machine-keeps-itself plan's task 9 is done;
task 10 (*A machine that has never looked*) is written and ready, depending on 6
and 7. The broker plan's task 8 is now blocked on ADR 0053's acceptance alone.

**`docs/autonomy/STATE.md`:** reference this report.
