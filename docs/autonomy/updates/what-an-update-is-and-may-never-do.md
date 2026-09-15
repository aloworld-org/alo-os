# What an update is, and what it may never do

**Date:** 2026-09-14
**Workstream:** v0.5 the machine keeps itself — task 1 of
`docs/autonomy/v0-5-the-machine-keeps-itself-plan.md` (`ROADMAP.md`: *updates
that never interrupt*; `docs/features.md` v0.5 **Updates that never
interrupt**)
**Contributor:** Claude, as a worker in `C:\dev\alo-os`
**Status:** ready for integration. The task decides and does not act: nothing
here downloads, schedules or applies an update, so there is nothing to measure
on hardware. Doing it is task 2.

## What changed, for a person

Nothing a person can use yet. What exists now is the decision about updates,
written down so it can't drift. An update on alo OS never restarts the machine,
never closes an application and never interrupts what the person is doing. The
person chooses when it applies: at their next restart, or now if they ask. They
can't switch updates off, and the machine never applies one by itself. There is
no "urgent" update that gets to break these rules. Checking for an update shows
on the same indicator as everything else that leaves the machine. The person is
told "An update is ready. It will apply when you choose, and nothing you are
doing will be interrupted until then", and never which build it is or how it is
installed.

## What changed, in the repository

New crate `crates/alo-keeping-up`:

| File | What it holds |
|---|---|
| `src/digest.rs` | `Digest` (whole, lowercase `sha256:`; `Deserialize` goes through the same check) and `NotADigest`, with `said` |
| `src/checking.rs` | `a_check_at(Destination) -> OnItsOwn` for `Errand::CheckingForAnUpdate`; `Offered`, made only by `Offered::heard(&Underway, Digest)`; `NotACheck` |
| `src/standing.rs` | `Running::reported`, `Standing::between`, answering `UpToDate` or `Ready`, `Ready { running, offered }`, `Standing::said` |
| `src/never.rs` | `Disturbance` (three clauses), `Cause` (`AnUpdate`, `ThePerson`), `TheRule` / `THE_RULE`, `Forbidden` |
| `src/when.rs` | `WhenItApplies` (`AtTheNextRestart` default, `NowBecauseThePersonAsked`), `cause_of_the_restart` |
| `src/words.rs` | The eight sentences, each with a translator's note |
| `src/testing.rs` | The unit tests' English strings, from the real list |
| `tests/what_an_update_may_never_do.rs` | One test per acceptance criterion, with the table at its head |

Registered:

- `Cargo.toml`: workspace member. `Cargo.lock` updated to match.
- `crates/alo-saying`: dependency, `EVERY_LIST` (35), the `declare` call,
  `ONE_STRING_EACH` and the count test. `alo-collected` reads the workspace and
  found the crate on its own, and it passes.

## Decisions

- **An update is a difference, not an ordering.** Digests have no order. The
  place updates come from decides which build this machine should run, so
  `Ready` means *a different build is offered*, and that includes an older one.
  Whether an offer can be trusted is a question for its signature (ADR 0036),
  checked where it is fetched and applied in task 2. Guessing at "newer" here
  wouldn't make anything safer.
- **The answer is tied to the errand by type.** `Offered::heard` borrows an
  `alo_egress::Underway`, and only `Indicator::beginning_on_its_own` can make
  one. So no offer can exist unless the check was shown while it ran. It must
  also be the update errand: an `Underway` for signing in or fetching a model
  is refused with `NotACheck`. Otherwise the indicator would be showing the
  wrong reason. `Offered` and `Ready` serialise but deliberately don't
  deserialise, so they can't be read back in through a side door.
- **The rule asks who is causing it, not only what.** The person restarting
  their own machine closes their applications and applies the waiting update,
  and that is the person choosing *when*. So `TheRule::allows(Cause,
  Disturbance)` refuses `AnUpdate` for every clause and allows `ThePerson`.
  `WhenItApplies::cause_of_the_restart` is always `ThePerson`, so no choice
  exists under which an update causes the restart.
- **There is no "urgent", and it is left out rather than guarded.** `Cause` has
  two members. `Ready` has no priority field. `allows` takes nothing else. An
  exhaustive match in the tests trips if a third cause or a fourth disturbance
  is added. `WhenItApplies` refuses "urgent", "critical", "security",
  "required" and "forced", and also "never", "off", "disabled",
  "automatically", "overnight" and "immediately".
- **The digest rule is strict.** Only `sha256`, 64 lowercase hex characters,
  nothing trimmed. This matches `alo-image`'s own pin rule. The crate doesn't
  depend on `alo-image` for it, because that is the installer lane's build
  checker, not a runtime type.
- **Which registry is not decided here.** `a_check_at` takes a `Destination`.
  The plan says the registry comes from installer-plan task 1, which is waiting
  on ADR 0036. This task needs only the type, so it is not blocked, and it
  guesses no hostname.
- **`NotACheck` is in English and has a `Display`.** The only way to trigger it
  is code written wrong, not anything happening on the machine. That follows
  `alo_saying::NotCollected`. `NotADigest` *can* come from a real answer, so a
  person gets a sentence for it: "nothing on this machine has changed". It has
  no `Display`.
- **The restart-now choice says it closes applications.** "Restart now and
  apply it, which closes the applications that are open". The person is
  choosing that, and needs to know before they do.
- **"No clock, no scheduler, no background fetch" is tested.** The test checks
  that the manifest's `[dependencies]` are exactly `alo-egress`, `alo-strings`,
  `serde` and `thiserror`. It also checks that no file under `src/` names
  `SystemTime`, `Instant`, `Duration`, `std::time`, threads, sockets, `std::fs`,
  `std::process`, `std::io` or `async`. That is why the indicator tests, which
  need a moment, are in the integration test file and not in the crate's own
  source.

## Acceptance criteria and evidence

All in `crates/alo-keeping-up/tests/what_an_update_may_never_do.rs`:

| Criterion | Test |
|---|---|
| An update is the digest offered, the digest running, and whether they differ | `an_update_is_a_build_offered_that_differs_from_the_build_running` |
| No clock, no scheduler and no background fetch decided here | `nothing_here_keeps_time_schedules_or_reaches_anything` |
| A check for an update is an `alo_egress::Errand`, on the indicator | `a_check_for_an_update_is_on_the_indicator_while_it_happens` |
| …and an answer heard during any other errand is refused | `an_answer_heard_during_any_other_errand_is_refused` |
| Never restarts the machine | `an_update_never_restarts_the_machine` |
| Never closes an application | `an_update_never_closes_an_application` |
| Never interrupts what a person is doing | `an_update_never_interrupts_what_a_person_is_doing` |
| No variant meaning *urgent* | `there_is_no_urgent_update_that_bypasses_the_rule` |
| No setting that turns updates off; nothing applies one unasked | `the_choice_is_when_and_never_whether` |
| What a person is told is in the vocabulary `alo-saying` collects | `an_update_ready_is_said_in_words_the_machine_collects` |

Unit tests in `src/digest.rs`, `src/checking.rs` and `src/words.rs` cover the
digest refusals (wrong algorithm, too short, capitals, surrounding space, bad
values read back). They also check that no sentence names the machinery
(`bootc`, deployment, digest, image, registry, container, pull, reboot), hedges,
or calls an update urgent, critical or required.

## Verification

Run on 2026-09-14 in WSL Ubuntu on the Windows Server gate machine, with
`CARGO_TARGET_DIR=$HOME/alo-builds/alo-os-88e6ebddb0cab76e`:

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, no warnings |
| `cargo test -p alo-keeping-up` | 10 unit + 10 integration passed |
| `cargo test -p alo-saying` | 63 unit + 4 integration + 1 doctest passed |
| `cargo test -p alo-collected` | 8 + 11 passed |
| `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-keeping-up --no-deps` | exit 0 |

Not run: the full workspace suite, which the supervisor runs. No hardware
measurement applies, because nothing here touches the machine.

## Remaining limitations

- Nothing fetches an offer, reads the running build from the base, or applies
  anything. That is task 2, and it must go through these types.
- The registry and its pinned digest still wait on installer-plan task 1 (ADR
  0036).
- Only English sentences exist so far. Task 5 owns the full walk from "an
  update exists" to "it is applied" to "it is rolled back", as a table.

## Proposed shared-document updates

- **CHANGELOG.md:** "Updates now have written rules that can't drift: an update
  never restarts the machine, never closes an application and never
  interrupts. The person chooses when it applies, and checking for one shows on
  the indicator."
- **ROADMAP.md:** under v0.5 *updates that never interrupt*, note that the
  policy types are in (`crates/alo-keeping-up`), and that staging and applying
  are still to come.
- **QUEUE.md / STATE.md:** the machine-keeps-itself plan, task 1 done; task 2
  (*an update applied, and the same machine afterwards*) is next.
