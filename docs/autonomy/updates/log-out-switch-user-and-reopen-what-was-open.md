# Log out, switch user — and reopen what was open

**Date:** 2026-09-18
**Workstream:** `docs/autonomy/v0-5-the-session-and-the-displays-plan.md`, task 5
(*Log out, switch user, lock — and reopen what was open*), for
`docs/features.md` v0.5 **Session management: log out, switch user, lock, and
reopen what was open** and `ROADMAP.md`'s v0.5 *Software* line of the same name.
**Contributor:** development PC, one worker, one working tree.
**Status:** **ready for integration.** Focused acceptance run and recorded
below. Nothing here has been on certified hardware, and nothing here needs to
be: no device is opened, no session is ended and no dialogue is drawn.

## What a person gets

Choosing **Log out** now asks every application to close, one at a time, and
gives each the time it needs to save. If any of them will not close — an editor
waiting to be told whether to save, a document with unsaved work — logging out
**stops and names them**, one sentence each, and nothing is killed. The person
can go back to their desktop, or choose *Log out anyway, and lose whatever has
not been saved*, which says in the label what it costs. They are also told when
some applications have already closed, because they have, and no system can put
one back.

Choosing **Switch user** locks the session and hands the screen to the sign-in.
The first person's applications, downloads and approved work keep running; they
come back to their own session with their own password.

And there is one new setting, **off as alo OS ships it**: *Open my applications
again when I sign in.* Turned on, alo OS writes down which applications were
open and on which screen and in which half or quarter of it, and opens them again
at the next sign-in. It never writes down what was in them — no document, no
page, no message, no window title — so what comes back is the shape of a desk
rather than a record of a day's work. Turned off, nothing is written down at all.

## What changed, and where

**New crate: `crates/alo-leaving`** (13 source files, six integration tests and a manifest, all new). It is this plan's, and
the plan's header now says so: task 5 was the one task in that plan with no crate
named for it.

| File | What it holds |
|---|---|
| `src/lib.rs` | The eight clauses, and what this crate never does |
| `src/logging_out.rs` | The walk: `TheApplications`, `Closing`, `LoggingOut`, `asked` |
| `src/refusing.rs` | `WouldNotClose` — the names, what already went, and `even_so` |
| `src/ending.rs` | `MayEnd`, `AfterWhat`: the one value that says a session may end |
| `src/switching.rs` | `Switching`, `asked`: locked first, the screen afterwards |
| `src/open.rs` | `Open`, `WasOpen`: an application, a screen, a split, and no fourth thing |
| `src/split.rs` | `Split`: the whole screen or one of `alo_dividing::Place`'s pieces |
| `src/changes.rs` | `Changes`, `Setting`, `Settings`: one setting, off as shipped |
| `src/keeping.rs` | `leaving.toml` through `alo-kept`; `at_sign_out`, `at_sign_in` |
| `src/restoring.rs` | `Restoring`, `AtSignIn`: what is reopened, which is nothing unless asked |
| `src/unkept.rs` | What a person is told when their own file did not read |
| `src/words.rs` | The twelve sentences this crate says |
| `src/testing.rs` | The person, the machine, the screens and the desk (`cfg(test)`) |

**Registered where a new crate has to be**, and nowhere else:
`Cargo.toml` (workspace members), `crates/alo-saying/Cargo.toml` and
`crates/alo-saying/src/collecting.rs` (the one vocabulary — `EVERY_LIST`, the
declaration, the one-key-each list and the count, which is what
`crates/alo-collected`'s check reads).

**`Cargo.lock`** gains this crate — **and one line that was already owed.**
`alo-displays` gained `serde_json` in task 3 or 4 and the lock was not written
with it, so the committed lock did not describe the committed manifests; the
regenerated lock in this change carries both. Worth a glance from the integration
owner, because it means a `--locked` build of `main` would have failed before
this.

**The plan**, `docs/autonomy/v0-5-the-session-and-the-displays-plan.md`: task 5
marked done with what it decided, and the header's crate list. Tasks 6 and 7 were
already named after it, so no new task was written.

**Nothing in `alo-shell`, `alo-locking`, `alo-greeting`, `alo-accounts`,
`alo-sessiond`, `alo-dividing`, `alo-applications` or `alo-appearance`** — all
read, none edited.

## The decisions, and why

The task left several things open. Each was decided the way a senior engineer
would and is written here rather than left for a reader to infer.

**A new crate, called `alo-leaving`.** The plan owns four crates and none of them
is about a session ending: `alo-locking` says at length that it never ends a
session and holds a test to it, and putting a log-out in it would have made that
test a lie. `alo-sessiond` opens sessions and is one of two privileged components
on the machine — the last place a walk over a person's applications belongs.
*Leaving* covers all three doors the task names (logging out, switching user, and
the lock that `alo-locking` already has) and names what the file holds: what you
left open.

**The order applications are asked in: last opened first, one at a time, and all
of them.** One at a time because a save dialogue deserves the screen to itself.
Last first because a stack unwinds — an editor opened from a file manager is
asked while the file manager it came from is still running — and because the
application a person was just in is the one they expect to be asked about first,
rather than after nine dialogues about programs they had forgotten were open.
**All of them, even after one refuses**, because a walk that stopped at the first
refusal would hand a person one dialogue, then another, then another; this comes
back with one list.

**`MayEnd` rather than a function that ends a session.** Ending a session is not
this crate's (the base's session manager does it, ADR 0011), so what a log-out
produces is a value saying it may happen. It has no public constructor and
exactly two roads to one: every application closed, or
`WouldNotClose::even_so`, which **consumes** the list of names. That is how
*never kills one silently* is held: this crate cannot kill anything at all — no
signal, no child, no process, no dependency that could lend it one — and the only
forced end is the session ending over an application that said no, after the
person read its name.

**What is kept is the identifier, not the name.** `alo_applications::Application`
already says why only an identifier is ever approved: the name is written by
whoever packaged the application and two can claim the same one. A file naming
*Mail* would reopen whichever *Mail* answered to it next, so `Open::of` takes an
identifier and the name has no road into a person's folder at all.

**One entry per window, and no window identifier.** Two windows of one
application on two screens are two entries, because the promise is about what was
on which screen. They are not two *named* windows: a compositor's identifier is
meaningless at the next sign-in, and writing one down would be recording that
there were two particular windows rather than two places. A log-out still asks
that application once.

**One file, `leaving.toml`, holding the choice and the list.** It would have been
tidier to put session state somewhere other than a settings file and it would
have been worse: `alo-kept` gives one file one writer, one format number and one
rule (refused whole when wrong, written whole or not at all, read back before it
counts — ADR 0038), and a second file would be all of those decided again. It
also puts the choice and the consequence of the choice in one place a person can
open in an editor.

**The list is written at exactly one moment, and only for somebody who asked.**
`keeping::at_sign_out` takes a `MayEnd`, so the only thing that writes a list is
a log-out that walked the applications; there is no watcher and nothing that
keeps the file up to date as windows come and go, because that would be the
background reader `CLAUDE.md` calls a bug in this product. The choice is read from
the file at the moment of the write, so a person who turned the setting off five
minutes ago is written no list and the list they had is taken out. A person who
never asked is left with **no file**.

The cost is honest: **a session that ends without a log-out — a power cut, a
crash — leaves nothing to reopen.** The alternative is a file that follows a
person around all day, and it is not close.

**The shipped default is off.** *Restored at the next sign-in only if the person
chose it* could have been read as *on unless they turn it off*. It is off,
because the list only exists for a person who asked for it, and a machine nobody
configured should hold no record of what anybody had open.

## Two findings rather than silences

**1. *Switch user* is refused at a locked screen.** Every other desktop offers
*sign in as somebody else* there. Task 1 of this plan decided what a lock screen
may show — the time, the lock image, the battery, and that the machine is locked —
and a road to the sign-in would be a fifth thing on it. That decision is
`alo-locking`'s and not this task's to reopen, so `switching::asked` answers a
locked seat with `alo_locking::NotWhileLocked`, which says the one thing the lock
screen already says and nothing about what is open behind it.

**What it costs:** two people sharing one machine have to have the first unlock
before the second can sign in. **What would fix it:** a decision in `alo-locking`
that a lock screen may carry one control — a road to the sign-in — which is a
change to task 1's `LockScreen` and to the sentence it may say. It is a small
change and it is somebody else's to make; this crate's refusal becomes a
`HandedOver` the day it is made, and `switching::asked` is the one function that
changes.

**2. `docs/contracts/person-settings.md` is now three kept files behind.** It
says *since 2026-09-15 there are four*, and names `appearance.toml`, `dock.toml`,
`shortcuts.toml` and `what-opens-what.toml`. It names neither `sleeping.toml`
(task 2), `displays.toml` (task 3) nor `leaving.toml` (this task). This task did
not pay that debt, for the reason tasks 3 and 4 gave: the contract's four
sections each have a `tests/the_contract_describes_this_file.rs` holding the
prose to the crate, adding a fifth section without one would be prose that can
drift, and rewriting the shared table is the owed lane's work rather than three
tasks each doing a third of it.

What the section for this file would have to say, so whoever pays it does not
have to read the crate:

- `leaving.toml`, kept by `alo_leaving::keeping`, `format = 1`.
- Keys besides `format`: **`reopen`** (a boolean — whether applications open
  again at the next sign-in; absent means the person has not chosen, and what
  alo OS ships is not to) and **`was-open`** (an array of tables, oldest first,
  each with exactly `application` — an identifier — `on` — the name the shell
  knows a screen by — and `split`, one of `the-whole-screen`, `left-half`,
  `right-half`, `top-half`, `bottom-half`, `top-left-quarter`,
  `top-right-quarter`, `bottom-left-quarter`, `bottom-right-quarter`, `part`).
- A `was-open` table with any other key in it refuses the **whole file**, and
  that is the clause worth writing down for a person: there is no key for a
  title, a document or an address, and there will not be one.
- `was-open` is honoured only when `reopen` is true; a list in a file whose owner
  did not ask for one is ignored, and taken out at the next log-out.
- A file that did not read reopens nothing and keeps the release's settings, with
  the refusal beside it — `leaving.kept.*`, naming the file and, where a key was
  what was wrong, the key.

## Verification

Run on this development PC (Windows Server 2022 host, gates in WSL Ubuntu, one
build cache at `/root/alo-builds/this-machine` from the serialised source copy at
`/root/alo-trees/this-machine`, as `docs/autonomy/SHARED_MAIN.md` requires).
Every command below was run in the foreground and its exit code read.

| Command | Result |
|---|---|
| `cargo fmt --all` then `cargo fmt --all --check` (on the checkout) | exit 0 |
| `cargo clippy --all-targets -- -D warnings` (**whole workspace**) | exit 0, 1m 13s |
| `cargo test -p alo-leaving -p alo-saying -p alo-collected` | exit 0 — 40 unit, 24 integration, 2 doc (both `compile_fail`), and the vocabulary and collection checks |
| `cargo doc -p alo-leaving -p alo-saying --no-deps` | exit 0, no warnings |

**Not run by this worker, deliberately:** `cargo test --workspace`. It takes
most of an hour on this machine, the supervisor runs it after every task, and two
finished tasks have already died at a deadline waiting on it. A cross-crate break
it finds comes back here with the error in hand.

**Not measured, and not claimed:** nothing in this task has run on certified
hardware. There is nothing in it that could be: no device is opened, no session
is ended, no process is signalled and nothing is drawn. The log-out a person
actually meets needs the shell's dialogue and the base's session manager, which
are the shell plan's later tasks.

### The acceptance criteria, and the test behind each

| Clause of the plan | Test |
|---|---|
| Logging out ends applications in an order that lets each save | `tests/logging_out_asks_every_application_and_names_what_stayed.rs::every_application_is_asked_last_first_and_then_the_session_may_end` |
| …names any that refused to close | `…::whatever_would_not_close_is_named_and_nothing_is_killed` |
| …and never kills one silently | `tests/nothing_here_reaches_into_an_application.rs::nothing_in_this_crate_can_reach_a_process` (and `it_depends_on_nothing_that_could_kill_a_process`, `it_opens_nothing_but_the_one_file_it_keeps`) |
| *Switch user* locks this session and hands the screen to the sign-in | `tests/switching_user_locks_this_session_and_hands_over_the_screen.rs::switching_user_locks_this_session_and_hands_the_screen_to_the_sign_in` |
| What was open is applications and places, never contents, titles or URLs; kept in the person's folder; restored only if chosen | `tests/what_was_open_is_applications_and_places.rs::a_person_who_asked_for_it_signs_in_to_what_they_left`, `…::nothing_in_this_crate_names_what_was_in_a_window`, `…::a_window_with_anything_else_in_it_refuses_the_whole_file`, `…::a_person_who_did_not_ask_has_nothing_written_down` |
| The agent never reads this list | `tests/the_agent_never_reads_what_was_open.rs::no_crate_that_answers_an_agent_reaches_what_was_open` and `…::nothing_in_the_agents_daemon_names_the_file` |
| Constraint: no application's own session files are read or written | `tests/nothing_here_reaches_into_an_application.rs::it_opens_nothing_but_the_one_file_it_keeps` |
| Every sentence reaches the one vocabulary, naming nothing we rent | `tests/every_sentence_here_is_collected.rs` (both tests) |

### The refusal paths, tested beside the legitimate ones

- A **locked** machine logs nobody out, and nothing is asked to close:
  `logging_out::tests::a_locked_machine_asks_nothing_to_close` and
  `tests/…::a_locked_machine_logs_nobody_out`, which also holds that the refusal
  says only *This machine is locked* and names nothing that is open.
- A locked machine does not switch user:
  `switching::tests::an_already_locked_machine_is_refused`.
- A person who does not insist ends nothing:
  `tests/…::a_person_who_does_not_insist_ends_nothing`.
- Somebody else's correct password does not unlock a switched-away session:
  `tests/…::the_first_person_comes_back_to_their_own_session`.
- An identifier no grant could name is refused:
  `open::tests::an_identifier_no_grant_could_name_is_refused`.
- A split nobody declares is refused rather than read as *part*:
  `split::tests::a_split_nobody_declares_is_refused`.
- A file with a key that is not this file's is refused whole, and a change is not
  written over it: `keeping::tests::a_wrong_file_is_refused_whole_and_not_written_over`.
- A window carrying a title, a document, an address or a window number refuses the
  whole file: `tests/…::a_window_with_anything_else_in_it_refuses_the_whole_file`.
- A file that did not read reopens nothing:
  `restoring::tests::a_file_that_did_not_read_reopens_nothing`.
- A list nobody asked for is ignored:
  `restoring::tests::a_list_nobody_asked_for_is_ignored`.
- Every reason a file did not read, and every reason a change was not written,
  says a declared sentence with the file named: `unkept.rs`'s three tests.

## Limitations

- **Nothing draws.** The dialogue naming what would not close, the *log out
  anyway* button and the greeter the screen is handed to are the shell's, from
  these decisions. So is the wiring that hands this crate the person's folder.
- **Nothing ends a session.** `MayEnd` is handed over; the base's session manager
  is what stops the session, and that composition is not written yet.
- **A session that ends without a log-out reopens nothing** — by decision, above.
- **`alo-dock` holds one edge for the machine**, so a reopened window's dock edge
  is the machine's. That is the same finding task 3 recorded and it is
  `alo-dock`'s to resolve; nothing in this crate reads the dock.
- **No record entry.** A log-out is the person's own act rather than an agent's
  verb, so nothing here writes to `alo-record`. If a later task wants *when did I
  last log out* in the record, that is an additive entry and a decision of its own.

## Proposed changes to the shared documents

For the integration owner. This worker edited none of them.

**`CHANGELOG.md`**, under v0.5:

> **Log out, switch user, and reopen what was open.** Logging out now asks every
> application to close, one at a time and last-opened first, and stops and names
> any that will not — nothing is ever killed silently, and *log out anyway* says
> in the label that unsaved work will be lost. *Switch user* locks the session
> before the sign-in appears, and the first person comes back to their own
> session with their own password. A new setting, off as alo OS ships it, opens
> your applications again when you sign in: what is written down is which
> applications were open and on which screen and in which part of it, and never
> what was in them — no document, no page, no window title. Nothing is written
> down at all for somebody who has not asked for it.

**`docs/autonomy/QUEUE.md`:** task 5 of the session-and-displays plan is done;
tasks 6 (notifications and do-not-disturb) and 7 (every sentence, and the walk)
are ready and both depend on work that has landed. The queue item for
`docs/contracts/person-settings.md` is now **three** kept files behind
(`sleeping.toml`, `displays.toml`, `leaving.toml`), and the section this task
would have added is written out above so whoever takes it need not read the crate.

**`docs/autonomy/STATE.md`:** reference this report. Worth recording beside it
that the lock screen's shape is now blocking one product behaviour every other
desktop has (*sign in as somebody else* from a locked screen), with the exact
change named in *Two findings* above.

**`ROADMAP.md`:** the v0.5 *Session management* line has its decisions and its
tests; what it still needs before it can be ticked is the shell's dialogue and
the session manager that ends a session, both in the shell plan.

**`docs/autonomy/a-new-machine-becomes-a-lane.md`** names this plan's crates as
`alo-locking`, `alo-sleeping`, `alo-displays`, `alo-notifying`. It should name
`alo-leaving` too; the plan's own header now does.
