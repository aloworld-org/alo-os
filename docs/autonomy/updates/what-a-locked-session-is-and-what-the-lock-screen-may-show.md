# What a locked session is, and what the lock screen may show

**Date:** 2026-09-16
**Workstream:** `docs/autonomy/v0-5-the-session-and-the-displays-plan.md`, task 1 —
the first task of the v0.5 session-and-displays plan. Tasks 2, 5 and 6 depend on it.
**Contributor:** Claude Code worker, `C:\dev\alo-os`.
**Status:** ready for integration. The code is complete and passes its gates. Nothing
in it touches the machine, so no hardware measurement is owed for this task. The
lock screen will be seen on hardware once the shell draws it (see *Limitations*).

## What this is

A lock screen is the one surface a stranger at the desk is sure to see. On most
systems it leaks: the names of recent documents in notifications, the agent's last
question, an approval waiting to be tapped. This change adds a new crate,
`crates/alo-locking`, which holds what locking decides as types. None of those leaks
is something a shell could draw by accident or a setting could turn on.

Nothing here draws, reads hardware or talks to `logind`.

## What changed

### A new crate, `crates/alo-locking`

| File | What it is |
|---|---|
| `src/seat.rs` | `Seat<N>` — the person's session, `Open` or `Locked`, holding the session in both. `opened`, `locked`, `arrives`, `lock_screen` |
| `src/locked.rs` | `Locked<N>` — a locked session and the notifications held for it. No accessor shows, counts or names them |
| `src/screen.rs` | `LockScreen` — the time, the lock image, the battery and an egress `Lamp`; that it is locked, as a sentence |
| `src/arriving.rs` | `Arrived<N>` — `ToShow(N)` on an open seat, `Held` on a locked one; no preview variant |
| `src/unlocking.rs` | `Seat::unlocks` and `Unlocking<N>` — signing in again through `alo_greeting::Greeting` |
| `src/the_locked_session.rs` | The door an unlock knocks at: the locked session itself, never the opener |
| `src/summoning.rs` | `Seat::press_the_agents_key` — refused while locked, compositor untouched |
| `src/approving.rs` | `Seat::ask`, `Seat::approve`, `Seat::decline` — refused while locked, turn untouched |
| `src/refusing.rs` | `NotWhileLocked` — whose sentence is the lock screen's own |
| `src/running.rs` | `Running` and `WhileLocked` — applications, approved turns and downloads keep running; one variant |
| `src/battery.rs` | `Battery` — a reading from 0 to 100 percent, charging or not |
| `src/words.rs` | One sentence: `locking.locked`, *This machine is locked* |
| `src/testing.rs` | Fixtures: a real sign-in, a real turn on a real disk |
| `tests/nothing_here_ends_a_session.rs` | Reads the source and manifest for every way to end a session |
| `tests/unlocking_is_signing_in.rs` | Reads the source: one call checks a password, and it is the greeting's |
| `tests/every_sentence_here_is_collected.rs` | The sentence is in the machine's vocabulary and names nothing rented |

### Registered where a new crate has to be

- `Cargo.toml`: workspace member; `Cargo.lock` follows.
- `crates/alo-saying`: dependency, `EVERY_LIST` (45 to 46), the `declare` call,
  `ONE_STRING_EACH` and the vocabulary-size test.

## A user-readable change description

When you lock your alo OS machine, everything you had running keeps running. That
includes applications, downloads and anything you already asked the agent to do.
The lock screen shows the time, your lock-screen picture, the battery, and that the
machine is locked. It does not show your name, your notifications, what the agent
is doing or anything waiting for your approval. There is no setting that shows
notification previews there. Notifications that arrive while the machine is locked
are kept, and you get them in order when you unlock.

At a locked machine, the agent's key does nothing except say the machine is locked.
Changes the agent proposed cannot be approved or declined there. The indicator that
something is leaving the machine still lights while locked, but it never says what
is leaving or where it is going. You unlock by signing in with the same name and
password you sign in with, checked the same way. There is no PIN, and nothing
remembers you. Somebody else's account on the same machine cannot unlock your
session.

## Decisions taken, and why

1. **Unlocking goes through `alo_greeting::Greeting` itself, not a copy of it.** A
   greeting authenticates and then knocks. The real door (`alo-sessiond`) would
   rightly refuse a second session with `ALREADY_SIGNED_IN`, so the unlock knocks
   at `TheLockedSession` instead. That door answers `Opened` for the locked
   session's own uid and gives the opener's own refusal for any other uid. The
   password check is untouched: `alo-accounts` decides it, with its one refusal
   and even timing. Only the step after a verified password differs. I considered
   and rejected two alternatives. The first was calling `Accounts::signs_in`
   directly, which is a second composition that could drift. The second was editing
   `alo-greeting` to add an unlock method, which the plan forbids (it reads that
   crate and never edits it).
2. **The greeting is built for the locked session's uid.** Another account's
   correct password is refused by `Session::opened`'s own *not the described
   person* rule, so nobody unlocks somebody else's session. A `SignedIn` whose
   session is not the locked one cannot happen through that greeting. If it did,
   it is refused with `NOT_SIGNED_IN` rather than let in.
3. **The unlock reads the accounts as they are at that moment.** `unlocks` takes
   `Accounts` at the moment of the attempt. A password changed while the machine
   was locked is the one that counts, and a test holds that.
4. **The lock screen does not show whose machine it is.** The acceptance lists
   four things and a name is not one of them. Unlocking takes a name and a
   password, the same as signing in. A test checks that the lock screen's `Debug`
   does not contain the account name.
5. **The egress light is `alo_indicator::Lamp`.** It is dark, or lit for a count,
   and has no line in it. So it can fire while locked without naming the
   destination, the agent or the reason. Nothing in locking touches the
   `Indicator`, so departures behind the lock are recorded and lit exactly as
   anywhere else.
6. **A notification is a generic `N`.** `alo-notifying` (plan task 6) decides what
   a notification is. This crate's promise (held while locked, never on the lock
   screen, handed back in order at an unlock) holds whatever `N` becomes.
   `LockScreen` is not generic, so no notification type can be put into it.
7. **Refusals say only "This machine is locked."** The agent's key and an answer
   to a change are refused with the one sentence the lock screen already shows. A
   distinct sentence would add something to the screen and would reveal that a
   change is waiting.
8. **An overlay that is open when the machine locks is closed** by `Seat::locked`
   through `Summoning::dismissed`. An agent left open on a lock screen would be
   answering whoever is at the desk.
9. **A question that was on screen before the lock must be asked again after
   unlocking.** `alo_approving::Approving` has no withdraw method, and this crate
   may not edit it. The seat refuses every answer while locked, and the rustdoc on
   `approving.rs` says that whatever redraws the desktop asks again with
   `Seat::ask`. The test walks exactly that: asked, locked, refused twice, unlocked,
   asked again, approved once, and the file moved once.
10. **`Running` and `WhileLocked` have a single variant, `KeepsRunning`.** A
    compile-fail example holds that `Paused` does not exist. `Seat::locked` is
    handed nothing it could stop, and a test shows a turn under way proposing
    while locked, with the proposal really waiting.
11. **The source tests avoid `std::process` in the fixture as well.** The
    "nothing ends a session" reader scans every file under `src`, including the
    test fixture, so the fixture names its temporary folders by time and a counter
    instead of the process id. That way the reader needs no exemption.

## Acceptance, clause by clause

| Clause | Held by |
|---|---|
| A locked session keeps running — applications, approved turns, downloads | `running::tests::everything_running_keeps_running_while_locked`; `approving::tests::a_turn_under_way_carries_on_while_locked` |
| Only the time, the lock image, the battery and that it is locked | `screen::tests::the_lock_screen_shows_the_time_the_image_the_battery_and_that_it_is_locked`; `…the_lock_screen_does_not_name_whose_machine_it_is`; `…each_display_has_the_lock_image_appearance_gives_it`; `…an_unlocked_seat_has_no_lock_screen`; compile-fail examples on `LockScreen` |
| A notification arriving while locked is held, never drawn, no preview variant | `screen::tests::a_notification_arriving_while_locked_changes_nothing_on_the_lock_screen`; `unlocking::tests::the_right_password_unlocks_and_hands_back_what_was_held`; `seat::tests::an_open_seat_shows_what_arrives`; compile-fail examples on `Arrived` and `screen` |
| The agent overlay cannot be summoned while locked | `summoning::tests::the_agents_key_asks_nothing_while_locked`; `seat::tests::an_overlay_open_when_the_machine_locks_is_closed`; `summoning::tests::the_agents_key_works_again_once_the_person_is_back` |
| A waiting proposal cannot be answered from the lock screen | `approving::tests::a_waiting_change_cannot_be_answered_from_the_lock_screen`; `approving::tests::a_change_proposed_while_locked_is_not_shown`; `refusing::tests::a_refusal_says_only_that_the_machine_is_locked` |
| The egress indicator still fires while locked, without naming what left | `screen::tests::the_egress_light_is_lit_while_locked_without_naming_what_left` |
| Unlocking is the same composition as signing in, and no weaker road | `tests/unlocking_is_signing_in.rs` (both tests); `unlocking::tests::a_wrong_password_or_an_unknown_name_leaves_the_seat_locked`; `…somebody_elses_correct_password_does_not_unlock_this_session`; `…a_password_changed_while_locked_is_the_one_that_counts`; `the_locked_session::tests::only_the_locked_sessions_own_number_is_let_back_in` |
| A locked session is not a signed-out one; `alo-locking` cannot end one | `tests/nothing_here_ends_a_session.rs` (all three, including the reader shown refusing); `seat::tests::a_locked_seat_is_still_the_same_session` |

## Verification

Executed on 2026-09-16, on this Windows Server machine, in WSL Ubuntu with
`CARGO_TARGET_DIR=$HOME/alo-builds/alo-os-88e6ebddb0cab76e`:

- `cargo fmt --all` — clean.
- `cargo clippy -p alo-locking -p alo-saying --all-targets -- -D warnings` — clean.
- `cargo test -p alo-locking`: 28 unit tests, 2 + 3 + 2 integration tests and 5
  doctests (compile-fail examples) all passed.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-locking --no-deps` — clean.
- `cargo test -p alo-saying -p alo-collected`: 63 + 4 + 1, and 8 + 11 passed. The
  vocabulary count and the workspace's "every crate that declares words is
  collected" check both include the new crate.

The full workspace suite was not run here. The supervisor runs it.

## What is owed on hardware

Nothing for this task. It is decisions as types and touches no device, so there is
no machine behaviour to measure. The lock screen as a person sees it (drawn,
covering every display, surviving a compositor restart) belongs to the shell plan's
later tasks. Locking before sleep is this plan's task 2. Neither is claimed here.

## Proposed updates to the shared documents

- **CHANGELOG.md:** the user-readable description above, under v0.5 → *Lock screen,
  suspend and resume*.
- **ROADMAP.md:** v0.5 *Lock screen, suspend and resume*, add `- [x] The code.`
  for *what a locked session is and what the lock screen may show*. The machine
  box stays unticked.
- **QUEUE.md / STATE.md:** the session-and-displays plan task 1 is done. Tasks 2,
  5 and 6 no longer wait on it.

## Limitations

- **Nothing is wired yet.** No shell holds a `Seat` today. `alo-shell` must route
  the agent key, approval answers, notifications and the lock screen through it.
  Until then these rules are enforced only in code that uses this crate. That wiring
  belongs to the shell plan, not this crate.
- **`alo-capturing` has its own lock flag** (`Session::behind_the_lock_screen`).
  Proposed follow-up: build it from `Seat::is_locked`, so the machine has one answer
  to "is it locked". Not done here because this plan does not own that crate.
- **No limit on unlock attempts beyond what signing in has.** Unlocking is
  deliberately exactly as strong as signing in. If a retry delay is wanted, it
  belongs in `alo-accounts` or `alo-greeting` so both screens get it, and not here.
- **The battery is a value handed in.** No crate reads the power supply yet. The
  devices plan will provide one, and its sentence is its own.
