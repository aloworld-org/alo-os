# A person can be told what their machine did

**Date:** 2026-09-10
**Workstream:** v0.01 lane B — accounts and session entry (`docs/autonomy/v0-01-lane-b-plan.md`, task 5)
**Contributor:** Claude Code worker, checkout `C:\dev\alo-os-b`
**Status:** ready for integration

## What was actually missing

`docs/features.md` promises, for v0.01: *A record of what the agent did, in
words*. `alo-record` writes it, `alo-keeping` puts it on a disk and shortens it,
and `alo-recounting` turns one entry into a clause a person reads. The plan says
nothing read the file back, and that was half right in a way worth writing down,
because the half that was wrong is where this task's work went.

`alo_recounting::Recounting::kept_at` did read the file. What it could not do
was be **called on a running machine**: it takes a path, the record lives at
`[record].path` in `docs/contracts/machine-description.md`, and nothing on the
person's side of the machine had ever opened that file. So every caller in this
repository was a test that already knew the answer, and *what did my machine do
today* had no door at all on the machine it was about.

Two more things were missing underneath that, and both are in the plan's
acceptance rather than in its heading:

- **The record was read without being believed.** `Reading::at` is
  `fs::read_to_string`. A record behind a symbolic link, one owned by somebody
  else, or one anybody signed in could write was read and shown as this
  machine's own account of itself — on the one surface whose whole worth is that
  what it shows is evidence.
- **An account was unbounded.** `Account` held every entry that answered the
  question. On a machine that has been working for a year that is a year, handed
  to a compositor in one value, and the machine with the longest record is the
  one whose owner has the most reason to be asking.

## What changed

### `crates/alo-recounting/src/where_it_is.rs` — where the record is (new)

`where_the_record_is(&Path)` reads `[record].path` out of the machine
description, and `Recounting::on_this_machine()` / `Recounting::described_at()`
are the doors that use it. `NotSaid` is why it could not: no description there,
what is there is not one this alo OS can read, or the machine would not read it.

### `crates/alo-keeping/src/believing.rs` — who may have written it (new)

`alo-accounts`' three questions, asked of the **open file** so that what was
checked and what is read cannot be two files: not a symbolic link
(`O_NOFOLLOW`), owned by root or by the login reading it, and not writable by
the group or the world. `Reading::believed_at` is the door,
`NotKept::ALink`/`SomebodyElses`/`WritableByOthers` are the refusals, and each
has a sentence in `alo-keeping`'s own list. `Recounting::about` reads through
it, so every account this repository can produce has been through those three
rules.

### `crates/alo-recounting/src/bounding.rs` — how much at once (new)

`AtMost`, which is not optional: `Recounting::about` and `Recounting::show` take
one, and there is no door beside them that answers with everything.
`Account::how_many_answered` and `Account::is_all_that_answered` say what was
left out, and `Account::said` adds a sentence for it.

### The rest

`Account` gained the bound and the count; `NotRecounted` gained `Nowhere`;
`alo-recounting`'s word list gained four strings (one remark, three refusals) and
`alo-keeping`'s gained three. `alo-saying` needed no change — it collects
`alo_recounting::declare_into`, and the counts it holds are sums rather than
literals.

## Decisions, and why

**The record's path is read out of the machine description rather than being a
constant.** `alo-remembering` argues the opposite for the grants and is right
about them: nobody's policy says where a machine's grants live, so a second copy
of the answer is a second place to point wrong. The record is not that.
`[record].path` is typed by whoever stands the machine up, `ADR 0004` gives
retention to the organisation, and a constant here would be a person's account
of their own machine reading a file the daemon is not writing to.

**So this is a third reader of the machine description, and it is answerable for
one key.** `alo-agentd`'s `describing.rs` decides whether a description is
believed; `alo-image`'s `description.rs` reads what the image is answerable for
and says in as many words why there cannot be one reader. The same argument
reaches further than that file: the daemon compiles to nothing off Linux, and a
surface that linked it to ask where a file is would be a person's shell linking
the privileged service. What this reader does differently from both is **ignore
a key it has never heard of** — they are answerable for the whole description and
deny unknown fields; a reader that refused to say where the record is because
the description grew a section about something else would take away somebody's
account of their own machine over a policy that has nothing to do with it. It
still refuses a **format number** it does not know (`2`, and `1` is read), because
what `path` means is fixed by the shape the file is in, and reading it anyway
would be reading a file by guessing.

**Nothing believes the description, and that is deliberate.** It is root's, in
`/etc`, and the daemon believes it before it will serve anything. What must be
believed before it is read is the record, which is where somebody is about to be
shown what a file says happened on their machine. A machine whose `/etc`
somebody else can write has lost this argument several steps earlier.

**`Reading::at` was left exactly as it was, and believing is a second door.**
The obvious move is to make every read believe, and it is wrong in one direction:
the daemon opens its own record at start-up, holds it open and shortens the file
it is already appending to. A service that stopped writing down what its agent
did because a mode bit changed underneath it would go quiet about exactly the
afternoon somebody would want to read. A reader being shown an account is the
other way round — nothing is lost by refusing, and what is at stake is a person
believing a file somebody else wrote.

**On a host that is not Unix, a record is not read at all.** There is no way to
ask who owns a file or who may write it, so `believing.rs` refuses rather than
pretending to have asked, and the refusal carries the standard library's own
word for a thing this platform does not do rather than a sentence invented in
this crate. alo OS is Linux; a believed read that did not believe would be a
worse answer than no answer.

**The bound keeps the newest entries and tells them oldest first.** *What has my
machine been doing* is a question about the recent past. Keeping the first two
hundred instead would answer *what did this machine do when it was new*, and
would go on answering that for the rest of the machine's life — a bound that
quietly stops the account moving is worse than no account, because it looks like
one. `AtMost::ONE_SITTING` is 200, named here rather than in every shell so two
surfaces cannot disagree about how much of a record is a page.

**`AtMost::entries(0)` is `None`.** An account bounded to nothing is an empty
list, and an empty list must never be able to mean *nothing happened* by
accident. That is this crate's one mistake, and a bound of zero would
manufacture it.

**`Recounting::about` and `show` changed shape rather than gaining a bounded
twin.** Two doors, one of them unbounded, is a door the compositor lane can pick
by mistake. Every call site in the existing suite was updated; nothing about
what those tests measure changed.

## Acceptance, and where each is measured

Every test below is in a file this change adds or edits, and each was run on its
own with `--exact` before this report was written.

| Criterion | Test |
|---|---|
| Read back off the real record file, as `Told` values, oldest first and bounded | `alo-recounting` `what_this_machine_did` `what_this_machine_did_is_read_off_the_record_it_says_it_keeps` |
| *This record does not go all the way back* carried into the account | `alo-recounting` `what_this_machine_did` `a_record_that_was_shortened_says_so_in_the_account_rather_than_being_read_as_a_quiet_day` |
| A record that cannot be believed is refused in words, never an empty day | `alo-recounting` `what_this_machine_did` `a_record_that_cannot_be_believed_is_refused_in_words_and_never_answered_as_an_empty_day` |
| A question about a span answers only that span | `alo-recounting` `what_this_machine_did` `a_question_about_a_span_is_answered_about_that_span_alone` |
| Nothing an agent can send over the socket reaches any of it | `alo-recounting` `what_this_machine_did` `nothing_an_agent_can_send_over_the_socket_reaches_the_record` |

The last one is two absences rather than a check, and it is worth saying which:
there is no request for it — `alo-protocol` is the closed list of what a client
can put on the wire, and a line asking for a record is not understood on either
door — and there is no code path for it, because `alo-agentd` does not depend on
`alo-recounting` at all and so nothing behind either door can make an account to
hand back. That is the same shape as `alo-remembering`'s file being unreachable,
measured the same way.

Beside them, the refusal paths each have a test of their own: every branch of the
ownership rule (`alo-keeping` `lib` `believing::tests::only_roots_record_or_our_own_is_believed`),
a link and a writable mode against real files on a real disk, a machine that
cannot say where its record is, a description from a newer alo OS, a description
with no `[record]` in it, and a bound that left something out saying so.

## Verification

Ubuntu under WSL, `CARGO_TARGET_DIR=$HOME/target-claude`, from
`C:\dev\alo-os-b`. Every gate `tools/kernel-loop/src/gates.rs` names was run,
in its order, and all nine passed:

- `cargo fmt --all --check` — clean.
- `cargo clippy --workspace --all-targets -- -D warnings` — zero warnings.
- `cargo test --workspace` — exit 0, 166 `test result: ok` lines, no failures.
- `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings` and
  `cargo test` in `tools/kernel-loop` — clean; 48 tests pass.
- `RUSTDOCFLAGS=-D warnings cargo doc --workspace --no-deps` — clean.
- `cargo fmt --all --check` and
  `cargo clippy --release --target bpfel-unknown-none -Z build-std=core -- -D warnings`
  in `crates/alo-bounding-kernel` — clean.

`alo-keeping`: 64 unit and 22 integration tests. `alo-recounting`: 54 unit, 5 +
6 integration, 4 doctests.

**One flake, written down rather than left out.** `cargo test --workspace` was
run three times. The first and third were clean (exit 0, 166 `test result: ok`,
none failed); the second failed one test,
`alo-secrets` `connections_come_and_go` `each_keyring_is_a_connection_of_its_own_and_giving_it_up_closes_it`,
which counts connections on a session bus and saw six where it wanted seven. It
passes on its own and passed in both clean runs, and nothing in this change
touches secrets, keyrings or D-Bus. It is the shared-system class
`docs/autonomy/SHARED_MAIN.md` warns about — a test that counts something
global while another worker may be on the same machine — and it is named here so
that whoever meets it next has seen it before.

**Not measured here:** nothing on a screen, and nothing on real hardware. This
task is a model, its constraint is *no new surface*, and the surface that shows
an account is the compositor lane's. The ownership branch of the believing rule
is walked as a function rather than by chowning a file, for `alo-remembering`'s
reason: a test cannot give a file away without root.

## Limitations

- **A believable file is not a believed record.** The three rules are about the
  file's place — owner, mode, not a link — and all three are satisfied by a
  record written whole by whoever owns the file. Task 6, written into the plan in
  this change, is the narrow honest next step: a record whose entries and whose
  beginning disagree is reported as one that is not what it says it is.
- **The format numbers are written down twice.** `where_it_is.rs` knows `2` and
  `1`; `alo-agentd`'s `describing.rs` is where they are decided. A later alo OS
  that raises the description's format has to raise it here too, and until it
  does this reader refuses rather than guesses — which is the safe direction, and
  is the reason it is a constant a reviewer can see rather than a check somebody
  has to remember.
- **`AtMost::ONE_SITTING` is a number nobody has looked at on a screen.** 200 is
  a judgement, and the compositor lane may want a different one; it is a value a
  caller passes, so changing it costs nothing.

## Files

New: `crates/alo-keeping/src/believing.rs`,
`crates/alo-recounting/src/bounding.rs`,
`crates/alo-recounting/src/where_it_is.rs`,
`crates/alo-recounting/tests/what_this_machine_did.rs`, and this report.

Edited: `crates/alo-keeping/{Cargo.toml,src/failing.rs,src/lib.rs,src/reading.rs,src/testing.rs,src/words.rs,tests/what_this_crate_says.rs}`,
`crates/alo-recounting/{Cargo.toml,src/account.rs,src/lib.rs,src/recounting.rs,src/refusing.rs,src/testing.rs,src/words.rs,tests/afterwards_ask_what_it_did.rs}`,
`Cargo.lock` and `docs/autonomy/v0-01-lane-b-plan.md`.

## Proposed shared-document updates

Not made here — `CHANGELOG.md`, `ROADMAP.md`, `docs/autonomy/QUEUE.md` and
`docs/autonomy/STATE.md` are the integration owner's (`docs/autonomy/SHARED_MAIN.md`).

**Proposed `CHANGELOG.md` entry, under v0.01:**

> **Ask what your machine did.** A person's side of an alo OS machine can now
> read the record their machine keeps, off the file the machine says it keeps it
> in, and be answered in words: what the agent did, what it was refused, what
> left the machine, and when. The record is checked before it is read — it must
> be this machine's own file, not a link, and not one anybody else could have
> written — and a record that fails that is refused in a sentence rather than
> shown as a day on which nothing happened. An answer holds the most recent part
> of what was asked about and says so when there is more.

**Queue/roadmap:** lane B's task 5 is done; task 6 (*the account a person asks
for is the one their machine kept*) is written into
`docs/autonomy/v0-01-lane-b-plan.md` and is ready. Nothing in
`docs/autonomy/v0-01-delivery-plan.md` matched this task, so nothing was marked
there.
