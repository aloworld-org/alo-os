# Switching to another person at a locked screen

**Date:** 2026-09-21
**Workstream:** `docs/autonomy/v0-5-the-session-and-the-displays-plan.md`, task 8
**Contributor:** this development PC's lane, one working tree
**Status:** ready for integration. Not on hardware, and nothing here claims to be.

## The decision, and why it is an ADR rather than a paragraph

The plan left this genuinely open — *either that rule stands and a household is
told plainly to unlock first, or it gains a fifth thing and the reasons for the
other four have to survive it* — and said the argument is written down before
the code is. It is:
`docs/decisions/0061-a-locked-screen-offers-a-road-to-the-greeter.md`, accepted.

**The answer is yes, once.** A locked screen may offer a road to the greeter.

The whole of the argument turns on a distinction task 1 implied and never
wrote down: **what a lock screen may *show* is not the same question as what it
may *offer*.** Every leak task 1 was written against is a fact about the person
behind the lock — a notification's first line, the document they were writing,
the agent's last question, a change waiting to be tapped — and every one of those
reasons survives this change untouched. A lock screen already offers one thing
that is not among its four: a name and a password, the way back in. Nobody has
ever read the rule as forbidding that, because it discloses nothing.

What the record refuses is as load-bearing as what it allows. **Option (B) — the
lock screen shows who else has an account here and the second person picks
themselves — is what most desktops ship and is refused outright.** A list of the
accounts on a machine is a disclosure about people who are not at the desk, made
to a stranger, and ADR 0024 had already decided that this product's greeter asks
for a name typed rather than offering one to pick. That decision is what makes a
road to the greeter safe here and would not make it safe elsewhere.

And the cost of leaving it alone was not neutral. A household sharing one machine
had the first person come back and type their password **in front of** the second
before the second could sign in at all. What a rule like that teaches a household
is to stop locking the screen, or to share the password. A lock nobody uses
protects nothing, and that cost is paid on the same surface the rule exists to
protect.

## What changed

### `crates/alo-locking/src/somebody_else.rs` — the road, new

`Seat::somebody_else(&self, accounts: &Accounts) -> SomebodyElse`. Three answers
and no fourth:

| | |
|---|---|
| `TheGreeter(Standing)` | the screen is the sign-in's, and this is what it stands at |
| `Refused(NotWhileLocked)` | this machine has no account for anybody to sign in to |
| `NoLockScreen` | the seat is open, so there is no lock screen and no road from one |

Three properties are held by the **types**, which is what the plan asked for
(*`alo-locking` holds it as a type rather than as prose*):

- **`SomebodyElse` has no generic parameter.** A `Seat<N>` is generic over what a
  notification is and holds every one that arrived while the machine was locked.
  A value with nowhere to put one cannot leak one, whatever a later change does
  to the shell — and a `compile_fail` example in the module header holds that
  `SomebodyElse<String>` is not a type.
- **The road takes `&self`.** *Not ended, not signed out, not unlocked* — task
  1's clause 8 — is the signature rather than a paragraph. Because it changes
  nothing, a surface may ask it twice: once to decide whether to offer the road
  at all, and again when somebody takes it. That is deliberate, and it is why
  the road is not a field of `LockScreen`: a shell that had to *call* the action
  to find out whether to draw it would be a button that sometimes does nothing.
- **`Standing` is derived, never named.** `Standing::of(accounts)` appears once
  in the crate and the value it produces is the value handed on, exactly as
  `alo_leaving::switching` already did it.

### The one refusal, and it is a real one

A machine whose accounts stand at *make an account* is refused, in the lock
screen's own existing sentence. `Standing::of` answers `MakeAnAccount` when the
store holds no account at all; on a cold machine that is ADR 0024's first-boot
screen and it is right, and on a **locked** machine it would be an offer, made to
whoever is standing at the desk, to create an account on somebody else's machine
and sign in on it — the store having been emptied under a running session by an
administrator, a restore, or a disk that came back wrong.

It is rare. It is also exactly the shape of thing this product is judged on, and
it is the reason this task has a refusal path worth testing rather than an
`if locked { yes }`.

### `LockScreen` is untouched, and that is the point

The road is **not** a field of `alo_locking::LockScreen`. That type is *what is
shown about this session at this moment*; it still has room for the time, the
lock image, the battery and the locked sentence, its `compile_fail` examples
stand, and `docs/contracts/lock-screen-rendering.md` continues to describe its
public view correctly and is not edited by this change.

Keeping the two in separate types is what stops the next person who wants a fifth
thing from arguing that the line has already moved: a field on `LockScreen` is a
disclosure and still needs the argument task 1 made; a road that carries no
session is not.

### `crates/alo-locking/src/words.rs` — one string

`locking.somebody-else`, *Somebody else*, with a translator's note that says what
the road does, that whoever locked the machine stays signed in with everything
still running, that it names nobody and a translation must not name anybody
either, and that a word suggesting the first person is being put out or signed
out is wrong because nothing is ended by taking it. The crate now says two
things; the header of `words.rs` says why the second is not a fifth thing about
this session. Task 7's audit reads the assembled vocabulary by area and picked it
up **without being edited**, which is the shape that file was written in.

### `crates/alo-leaving/src/switching.rs` — task 5's finding, closed

Task 5 recorded the refusal as a finding rather than a silence and named the
change that would fix it: *a change to what a lock screen may show*. That change
has now been made, so leaving the finding in place would have left prose in the
repository contradicting an accepted ADR.

`switching::asked` now locks first — which leaves a seat that is **already**
locked exactly as it is, holding everything it held — and then asks
`Seat::somebody_else`. Both roads into *switch user* therefore get
`alo-locking`'s answer, and this crate makes no lock-screen decision of its own,
which is what its header always claimed. One ordering property is now stronger
rather than weaker: the empty-store case is refused on the **desktop** road too,
where before it would have locked the session and handed over a greeter standing
at *make an account*.

`Switching::NotWhileLocked` keeps its name and its shape. It is still returned
only for a seat that is locked — after the lock, every seat is — and it still
carries the lock screen's own sentence. Its meaning narrowed from *the machine
was already locked* to *the machine has no account for anybody to sign in to*.

## Decisions I made that the task left open

- **An ADR rather than a paragraph.** The plan made this conditional on the
  answer, and the answer is yes.
- **`SomebodyElse` carries no session, rather than carrying the seat back.**
  `alo_leaving::Switching` and `alo_locking::Unlocking` both hand the seat back,
  and I copied neither. Handing it back would have meant `SomebodyElse<N>`, a
  generic parameter over what a notification is, on the one value that crosses
  from a locked session towards a greeter. `&self` and no parameter is a stronger
  statement of the same guarantee and is cheaper to review.
- **Three answers rather than two.** An open seat gets `NoLockScreen` rather
  than a refusal, because *not while locked* said of a seat that is not locked is
  a sentence that is simply false, and because a caller that gets it has looked
  at the wrong screen rather than been refused.
- **The word is *Somebody else*, not *Switch user*.** *Switch user* is what the
  act is called from a desktop, where there is a user to switch away from. At a
  lock screen the person reading it is not the user and is not switching
  anything; they are asking whether somebody else may use this machine.
- **`alo-leaving` was touched, and `alo-sessiond` and `alo-greeting` were not.**
  The plan's constraint names the two that may not be; `alo-leaving` is this
  plan's own crate and holds the prose this decision falsified. The change to it
  is delegation and documentation, adds no dependency, and its closed dependency
  list test passes unchanged.
- **`docs/contracts/lock-screen-rendering.md` was left alone.** It describes what
  the shell's nested lock surface does today. Editing it would have claimed shell
  behaviour that does not exist yet; it gains the road in the change that draws
  one, which is the shell plan's, and the ADR says so under *what this record
  does not decide*.

### `Cargo.lock`, which is not this task's work and is in this change anyway

The first build in this tree rewrote `Cargo.lock`, adding `alo-appearance` and
`alo-displays` to `alo-sleeping`'s recorded dependencies. **Both are already in
`crates/alo-sleeping/Cargo.toml` on `main`**, as task 7's dev-dependencies; the
lock was committed without them, so any `cargo` command in a clean checkout
leaves the tree dirty. No dependency is added or changed by this task.

It is included rather than reverted because reverting it would leave the next
worker's checkout dirty for a reason they would have to rediscover. It does mean
this change touches a workspace-wide file, so `docs/autonomy/SHARED_MAIN.md`'s
scope table puts it in the *all nine gates* row.

## Verification

Run from `/root/alo-trees/this-machine`, the Linux copy of this checkout that
`docs/autonomy/SHARED_MAIN.md` names, with `CARGO_TARGET_DIR=/root/alo-builds/this-machine`.

| Command | Result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy -p alo-locking -p alo-leaving --all-targets -- -D warnings` | exit 0, no warnings |
| `cargo test -p alo-locking` | exit 0 — 34 + 4 + 2 + 3 + 2 unit/integration, 6 doctests, all passed |
| `cargo test -p alo-leaving` | exit 0 — 41 + 2 + 5 + 3 + 5 + 3 + 6 + 2, all passed |
| `cargo doc -p alo-locking -p alo-leaving --no-deps` | exit 0, no warnings |
| `cargo test -p alo-saying` | exit 0 — the new string collects into the machine's one vocabulary |
| `cargo test -p alo-sleeping --test every_sentence_this_workstream_says --test the_walk_from_lock_to_resume_to_a_new_desk` | exit 0 — task 7's audit and walk, unedited |
| `cargo test -p alo-citing` | exit 0 — every pointer at ADR 0061 resolves |

`cargo fmt --all` and `cargo clippy` **cannot be run on the Windows side of this
machine**, and neither fact is about this change: `cargo fmt --all` fails with
`os error 206` (the command line exceeds Windows' limit once every target file is
named), and `ring`'s build script needs a `gcc` this host does not have. Both run
normally in the Linux copy, which is where the gates run.

The full workspace suite was **not** run here, deliberately: it takes the better
part of an hour on this machine, the supervisor runs it after this hand-over
regardless, and the crates this change can reach were run individually above.

### The acceptance criteria, one line each

All in the product workspace (`.`).

- *the decision is recorded as an ADR* —
  `docs/decisions/0061-a-locked-screen-offers-a-road-to-the-greeter.md`, accepted,
  and `alo-citing` `every_decision_this_repository_points_at` passes over it.
- *`alo-locking` holds it as a type* — `alo-locking`, unit tests of
  `src/somebody_else.rs`:
  `somebody_else::tests::a_locked_screen_hands_the_screen_to_the_greeter`.
- *the refusal path tested as carefully as the road* — `alo-locking`,
  `somebody_else::tests::a_machine_with_no_account_on_it_is_refused` and
  `somebody_else::tests::an_open_seat_has_no_lock_screen_to_ask_it_of`.
- *a locked seat produces an `alo_greeting::Standing` and nothing else about the
  session it came from* — `alo-locking`,
  `somebody_else::tests::nothing_the_road_hands_back_names_the_session_it_came_from`
  and, as a property of the shipped code, test target
  `a_road_to_the_greeter_and_no_second_one`,
  `the_road_carries_nothing_of_the_locked_session`.
- *a test reads this crate's own source for any second road* — `alo-locking`,
  test target `a_road_to_the_greeter_and_no_second_one`,
  `the_road_to_the_greeter_is_one_file_and_one_call`, with
  `the_reader_refuses_a_road_that_carries_the_session` showing the reader
  refusing what it exists to catch.
- *the session that was locked is not ended, not signed out and not unlocked* —
  `alo-locking`,
  `somebody_else::tests::the_locked_session_is_untouched_and_still_holds_what_it_held`,
  and task 1's `nothing_here_ends_a_session` test target still passes over the
  new file.
- *it may not make switching a thing an agent can ask for* — `alo-locking`, test
  target `a_road_to_the_greeter_and_no_second_one`,
  `no_crate_an_agent_is_answered_in_can_reach_a_seat`.
- *no second road to a password* — `alo-locking`, test target
  `unlocking_is_signing_in`, `the_only_road_through_the_lock_is_the_greeting`
  and `nothing_here_is_a_weaker_key_to_the_lock`, both of which read the new
  file because they read every file under `src`.
- *the string joins task 7's audit* — `alo-locking`, test target
  `every_sentence_here_is_collected`,
  `every_sentence_this_crate_can_say_is_in_the_machines_vocabulary`; and
  `alo-sleeping`, test target `every_sentence_this_workstream_says`,
  `what_the_crates_declare_and_what_the_machine_says_are_the_same_list`.
- *switch user means one thing at a desk and at a lock screen* — `alo-leaving`,
  unit tests of `src/switching.rs`:
  `switching::tests::an_already_locked_machine_is_handed_over_as_it_is` and
  `switching::tests::a_machine_with_no_account_on_it_is_refused`.

## What is not done, and is not claimed

- **Nothing draws.** No shell surface offers this road yet. A household on a
  booted alo OS machine cannot use it until the shell plan draws it, and this
  report does not claim otherwise.
- **Nothing was measured on hardware.** No screen has locked on a certified
  machine and no second person has signed in at one. Nothing in this change
  opens a device, reaches a bus or ends a session.
- **Whether a machine may hold two sessions at once** is `alo-sessiond`'s and the
  shell's, untouched here. It was already the condition of *switch user* from a
  desktop before this change, so this adds no new requirement on the machine.
- **`docs/contracts/lock-screen-rendering.md`** gains the road in the change that
  draws it.

## Proposed changelog entry

> **Somebody else, at a locked screen.** Two people sharing a machine no longer
> need the first to come back and type their password before the second can sign
> in: a locked screen may offer a road to the sign-in. It shows nothing about the
> session behind the lock — not who is signed in, not what is waiting, not what
> was open — and it is deliberately **not** a list of who has an account here,
> because that would be a disclosure about people who are not at the desk. What a
> lock screen may *show* is unchanged: the time, the lock image, the battery, and
> that the machine is locked. The argument is
> `docs/decisions/0061-a-locked-screen-offers-a-road-to-the-greeter.md`.

## Proposed queue and roadmap updates

- Mark task 8 of `docs/autonomy/v0-5-the-session-and-the-displays-plan.md`
  **Done, 2026-09-21** — done in this change, in the plan file itself.
- Task 9 is written in the same plan and is **ready**: *The four files this plan
  keeps, in the contract that describes them.* It pays a debt three of this
  plan's tasks named and none paid — `docs/contracts/person-settings.md` still
  says there are four kept files and names none of `sleeping.toml`,
  `displays.toml`, `leaving.toml` or `notifying.toml`.
- No `ROADMAP.md` line moves. *Session management: log out, switch user, lock,
  and reopen what was open* is closer to complete in code and still has nothing
  drawn.
