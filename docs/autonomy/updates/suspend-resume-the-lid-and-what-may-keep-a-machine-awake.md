# Suspend, resume, the lid, and what may keep a machine awake

**Date:** 2026-09-17
**Workstream:** `docs/autonomy/v0-5-the-session-and-the-displays-plan.md`, task 2.
It covers `docs/features.md` v0.5 *Power management, battery, sleep on lid close*
and the inhibit portal's *no sleep mid-presentation*. It depends on task 1, which
was published on 2026-09-16.
**Contributor:** Claude Code worker, `C:\dev\alo-os`.
**Status:** ready for integration. The code is complete and passes its gates. The
one piece of real-machine behaviour a test can take safely is measured: a `logind`
hold is taken, listed and released. A real suspend, a real resume and a real lid
switch are owed on certified hardware and are not claimed (see *What is owed on
hardware*).

## What this is

A laptop that wakes up unlocked, or will not sleep with nobody able to say why,
has failed at its most basic job. This change adds `crates/alo-sleeping`. The
mechanism stays `logind`'s, rented and configured (ADR 0011). This crate owns
two things: the order in which things happen, and who may hold the machine
awake. Both are written as types.

## What changed

### A new crate, `crates/alo-sleeping`

| File | What it is |
|---|---|
| `src/deciding.rs` | `asked(Why, …) -> Decided`: whether the machine sleeps, decided before any call. `Going`, `StaysAwake` |
| `src/going.rs` | `Going::carried_out`: lock the seat, make a `LockedFirst`, then `Logind::sleep`. `Slept`, `Asleep`, `NotAsleep` |
| `src/waking.rs` | `Asleep::woke` gives a seat that is still locked. `Woke::a_turn` gives an `AfterSleep`: the turn carries on or is stopped, and either way it is recorded |
| `src/lid.rs` | `Lid` (the person's choice), `Displays`, `LidClosed` |
| `src/keeper.rs` | `Keeper`: the closed list of three, each with its sentence |
| `src/holding.rs` | `Holding`: the three doors in, a `logind` hold behind each entry, and `as_it_stands` |
| `src/refusing.rs` | `NotKeptAwake`: why a hold was not taken |
| `src/session_holds.rs` | `TheLidIsOurs` and `UntilLocked`, the two holds alo OS keeps for a signed-in session |
| `src/logind.rs` | The `Logind` trait (`hold`, `sleep`), `Inhibit`, `LockedFirst`, `NotHeld`, `NotSlept` |
| `src/machine.rs` | `TheMachinesLogind`: `Inhibit` and `Suspend` over the system bus (Linux only) |
| `src/changes.rs`, `src/keeping.rs`, `src/unkept.rs` | The person's two settings in `sleeping.toml`, kept by `alo-kept` (ADR 0038) |
| `src/words.rs` | 22 sentences, each with a translator's note |
| `src/testing.rs` | A machine that counts its holds, a real sign-in, and a real turn with a record |
| `tests/an_agent_cannot_keep_this_machine_awake.rs` | Reads every crate's manifest. No crate that answers an agent may depend on this one |
| `tests/the_machine_holds_and_lets_go.rs` | A real `logind` hold, found in `ListInhibitors` and gone once it is dropped |
| `tests/every_sentence_here_is_collected.rs` | Checks that the sentences are collected and that none names anything rented |

### One new kind of record entry, additive

A turn that the machine slept through has to be written down. It must not be
silently lost. The record had no kind of entry for this, so this change adds one
and keeps it small:

- `crates/alo-record`: `Happened::SleptThrough { agent, stopped: Option<Line> }`,
  tagged `slept-through`, and `Entry::slept_through`. `was_stopped` and
  `why_stopped` answer for a stopped turn. The entry is never egress, and
  `format` stays `1`.
- `crates/alo-recounting`: `Outcome::CarriedOnAfterSleep` and
  `Outcome::StoppedBySleep`, each with a clause.
- `crates/alo-turn`: `Turning::slept_through`, a door that uses the turn's own
  agent and is written the way `a_pairing_was_kept` is.
- `docs/contracts/record-file.md`: the new tag and its fields.

### Registered where a new crate has to be

- `Cargo.toml`: workspace member. `Cargo.lock` follows.
- `crates/alo-saying`: dependency, `EVERY_LIST` (48 to 49), the `declare` call,
  `ONE_STRING_EACH`, and the vocabulary-size test.

## A user-readable change description

Your alo OS machine now always locks before it goes to sleep. It locks whether
you close the lid, choose Sleep, or it falls asleep on its own. When it wakes,
you see the lock screen and sign in the same way as always. If the machine
fails to go to sleep, it stays locked.

Closing the lid puts the machine to sleep. The one exception: if a monitor is
plugged in and you have chosen that it should stay awake. With nothing plugged
in, closing the lid always sleeps the machine, so it does not keep running in a
bag.

Only three things can stop the machine from going to sleep on its own: your own
setting to keep it awake, an application you have allowed to keep it awake (a
presentation, for example), and work you asked the agent for that is still
running. Whenever the machine stays awake, it tells you which one is holding it.
None of them can stop a sleep you asked for yourself. An application's
permission ends as soon as you take it back. The agent cannot keep the machine
awake just by asking. Only work you already started can, and only until that
work ends.

If the machine sleeps while the agent is working for you, the work carries on
when the machine wakes, as long as its time has not run out. If its time ran out
while the machine was asleep, you are told the work stopped and that you can ask
again. Either way, the record of what the agent did shows it.

## Decisions taken, and why

1. **A keeper holds off only sleep the machine starts on its own. It never holds
   off a sleep the person asks for.** The plan lists who may keep a machine awake,
   not what they may override. An application able to refuse a closed lid or a
   Sleep chosen from the menu would be the application deciding for the person,
   and a laptop left running in a bag because a video was paused is a failure
   everybody knows. So `Why::LidClosed` and `Why::YouAsked` both sleep, and
   `Going::overriding` names what they overrode. `Why::NobodyIsUsingIt` is held
   off and names every keeper. *No sleep mid-presentation* works this way: a
   presentation holds off idle sleep.
2. **Locking first is a type, not a convention.** `Logind::sleep` takes a
   `LockedFirst`. Its constructor is private (a compile-fail example holds this,
   E0603), and `going::locked_first` makes one only from a locked seat. There is
   no way to call `sleep` without first locking.
3. **A sleep started somewhere else is covered too.** `UntilLocked` is a `sleep`
   delay inhibitor held for the whole session. `before_sleeping` locks the seat
   and only then drops the delay. `TheLidIsOurs` is a `handle-lid-switch` block,
   so `logind`'s own `HandleLidSwitchDocked=ignore` default cannot answer the
   person's lid question for them. Both are `logind`'s mechanisms, configured,
   and no daemon of our own is involved.
4. **Each keeper is a real `idle` inhibitor.** The list in `Holding` and the
   machine's own `systemd-inhibit --list` name the same things, and the `why` in
   that list is the person's own sentence in their language. A hold that the
   machine refuses is not added to the list, so the two lists cannot disagree.
5. **An application's hold is judged again at every decision.** It takes an
   `alo_portals::Allowed` for `Portal::Inhibit` only; an `Allowed` for the camera
   is refused. The grants are asked again when the hold is taken and in every
   `as_it_stands`. A revoked or expired grant therefore lets go at the next
   question, and its `logind` hold is dropped in the same call. A grant made to
   an agent over `Facility::Sleep` is not an application's grant, and the portal
   judges it as nothing granted.
6. **A turn holds for its own length, read off the turn.** `Holding::a_turn`
   takes a `&Turning`. A turn exists only from a person's invocation, and its
   length is the person's bound. There is no door that takes a grantee, a verb or
   a duration (a compile-fail example holds this, E0308). No crate that answers
   an agent (`alo-agentd`, `alo-turn`, `alo-capability`, `alo-protocol`,
   `alo-broker`) depends on `alo-sleeping`; only `alo-saying` and, later,
   `alo-shell` may.
7. **At a wake, a turn carries on if it still has time; otherwise it is
   stopped.** A turn's length is already how long the person allowed the agent to
   work. Sleeping inside that length takes nothing from them. Carrying work past
   it would continue into time nobody granted. Both outcomes are recorded before
   anything is handed back. A turn whose record cannot be written is ended, and
   `AfterSleep::NotRecorded` is the answer, which is `alo-turn`'s own rule for
   evidence that could not be kept.
8. **A new record kind rather than squeezing into an old one.** No existing
   `Happened` described this, and reusing `never-put-anywhere` or `turned-away`
   would have made the record say something false. The addition follows
   `record-file.md`'s *a new kind of `happened` is additive*. It touched three
   crates this plan does not own (`alo-record`, `alo-recounting`, `alo-turn`),
   each in the pattern those crates already use.
9. **What an application says about itself is never shown.** The inhibit
   portal's `reason` is text the application wrote, so a keeper is named only by
   the identifier the sandbox vouches for.
10. **`Suspend` is called with `interactive = false`.** A sleep the person asked
    for must not stop to ask for a password, and a policy that refuses it answers
    `NotSlept` with the seat still locked.
11. **Two settings, and no more:** `lid` and `keep-awake`. How long *idle* is,
    battery thresholds and power profiles belong to the devices plan, as the
    plan's constraint says. Which displays are attached is handed in as
    `Displays`, until `alo-displays` (task 3) exists.

## Acceptance, clause by clause

| Clause | Held by |
|---|---|
| A session is locked before the machine sleeps; a test walks suspend then resume and finds the lock in place first | `waking::tests::suspend_then_resume_lands_on_the_lock_and_the_turn_carries_on`; `going::tests::the_seat_is_locked_before_the_machine_sleeps`; `going::tests::a_machine_that_did_not_sleep_stays_locked`; `going::tests::only_a_locked_seat_proves_it_was_locked_first`; `session_holds::tests::a_sleep_started_elsewhere_waits_for_the_lock`; compile-fail example on `LockedFirst` |
| Closing the lid sleeps unless a display is attached and the person chose otherwise; kept at a path this crate is handed | `lid::tests::a_closed_lid_sleeps_unless_a_display_is_attached_and_the_person_chose_otherwise`; `deciding::tests::a_closed_lid_follows_the_persons_choice`; `keeping::tests::a_choice_is_kept_and_a_wrong_file_is_refused_whole`; `unkept::tests::*` |
| What may keep the machine awake is a closed list, and each is named when it blocks sleep | `deciding::tests::every_keeper_holds_off_idle_sleep_and_is_named`; `keeper::tests::every_keeper_says_why_the_machine_is_awake`; `deciding::tests::a_keeper_does_not_hold_off_a_sleep_the_person_asked_for`; `holding::tests::a_revoked_grant_lets_go_of_the_machine_at_once`; `…a_grant_revoked_after_judging_is_refused_at_the_hold`; `…an_expired_grant_is_gone`; `…another_portals_permission_does_not_keep_the_machine_awake`; `…a_hold_the_machine_refused_is_not_held`; `the_machine_holds_and_lets_go::a_hold_is_listed_while_held_and_gone_when_let_go` |
| An agent cannot inhibit sleep by asking; only an approved turn holds it, for its own length | `an_agent_cannot_keep_this_machine_awake::no_crate_that_answers_an_agent_reaches_sleep` (and its reader shown refusing); `holding::tests::a_turn_holds_for_its_own_length`; `holding::tests::a_grant_to_an_agent_is_not_an_applications`; compile-fail example on `Holding::a_turn` |
| A turn running when the machine slept is resumed or refused with a sentence, and recorded | `waking::tests::a_turn_whose_time_ran_out_while_asleep_is_stopped_and_recorded`; `waking::tests::suspend_then_resume_lands_on_the_lock_and_the_turn_carries_on`; `alo-record` `entry::tests::a_turn_the_machine_slept_through_is_recorded_carried_on_or_stopped`; `alo-recounting` `told::tests::a_turn_the_machine_slept_through_reads_back_carried_on_or_stopped` |

## Verification

Run on 2026-09-17 on this Windows Server machine, in WSL Ubuntu (root, systemd
running, `logind` present), with
`CARGO_TARGET_DIR=$HOME/alo-builds/alo-os-88e6ebddb0cab76e`:

- `cargo fmt --all`: clean.
- `cargo clippy -p alo-sleeping -p alo-record -p alo-recounting -p alo-turn -p alo-saying --all-targets -- -D warnings`: clean.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-sleeping -p alo-record -p alo-recounting --no-deps`: clean.
- `cargo test -p alo-sleeping`: 37 unit tests, 2 + 2 + 1 integration tests and
  2 compile-fail doctests, all passed. Run with `--nocapture`, the `logind` test
  printed no skip. It took a real `idle` hold, found it in `ListInhibitors`
  under *alo OS* with the person's sentence and this process's pid, and found it
  gone once dropped.
- Both compile-fail examples were checked by compiling them as a scratch test
  (removed afterwards). They fail with E0603 and E0308, as their documentation
  says.
- `cargo test -p alo-record -p alo-recounting -p alo-saying`: all passed (79,
  57 and 63 unit tests, plus integration tests and doctests).
- `cargo test -p alo-turn -p alo-collected`: all passed (113 unit tests in
  `alo-turn`, plus integration tests; 8 + 11 in `alo-collected`).

The full workspace suite was not run here. The supervisor runs it.

## What is owed on hardware

- **A real suspend and resume** on the certified laptop, showing that the lock
  screen is up before the panel goes dark and when it comes back. No test here
  calls `Suspend`, because that would put the shared gate machine to sleep.
- **A real lid close**, with and without an external display, with
  `TheLidIsOurs` held. This checks that `logind` hands the lid to the session and
  that the shell routes the switch into `asked(Why::LidClosed, …)`.
- **A sleep started elsewhere** (the power key, or `systemctl suspend` in the
  person's terminal) waiting on `UntilLocked` until the seat is locked.

None of these is ticked. Per the plan, the ROADMAP line gets `- [x] The code.`
and nothing more.

## Proposed updates to the shared documents

- **CHANGELOG.md:** the user-readable description above, under v0.5 → *Lock
  screen, suspend and resume* (and *Power management, battery, sleep on lid
  close*). Also, under the record: *the record now says when the agent's work
  was interrupted by the machine sleeping, and whether it carried on*.
- **ROADMAP.md:** v0.5 *Lock screen, suspend and resume*, add `- [x] The code.`
  for *suspend, resume, the lid, and what may keep a machine awake*. The machine
  boxes stay unticked.
- **QUEUE.md / STATE.md:** session-and-displays plan task 2 is done. Task 7 now
  waits on 3, 4, 5 and 6.

## Limitations

- **Nothing is wired yet.** No shell holds a `Holding`, takes the two session
  holds, listens for `PrepareForSleep`, or routes the lid switch and the idle
  timer into `asked`. That wiring belongs to the shell plan. Until it lands, these
  rules are enforced only in code that uses this crate.
- **The inhibit portal is not served on the bus yet.** `Portal::answered_on_the_bus`
  still answers `None` for `Inhibit`. Serving it (a request handle whose closing
  calls `Holding::lets_go`) belongs to the applications plan's backend. This
  crate takes the `Allowed` that backend will produce.
- **`Displays` is a value handed in.** `alo-displays` (task 3) will supply it.
- **`alo-portals` has no words for a request refused over `Facility::Sleep`
  beyond its general refusal**, which is what `NotKeptAwake::NotAllowed` shows.
  That is adequate, and noted in case the portal plan wants a sentence specific
  to sleep.
- **`alo-recounting` has no owner.** No plan's *Crates this plan owns* names it,
  and in the seven days to 2026-09-17 twelve commits on `main` added to it, from
  at least five workstreams: the broker (7687e34), the machine keeping itself
  (d665ab5, bec518d), the local network (ac67c9d, ee9327f, 5eade90, fc4f33d),
  grants (56b6958) and the record's own work. This task makes a thirteenth. Every
  one of those changes adds sentences, but nothing says additions are all it may
  take. It needs an owning plan, or a note in the plans that it is shared and
  takes additions only, before two lanes change the same sentence.
