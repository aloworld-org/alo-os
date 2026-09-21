# Notifications, and do-not-disturb

**Date:** 2026-09-19
**Workstream:** `docs/autonomy/v0-5-the-session-and-the-displays-plan.md`, task 6
**Contributor:** development PC worker, in `C:\dev\alo-os`
**Status:** ready for integration. Not on hardware, and nothing here claims to
be — nothing in this change draws, opens a device or reaches a bus.

## What this is

`docs/features.md`, v0.5: *Notifications, with do-not-disturb.* A notification
is the one thing on a screen that arrives uninvited, and the thing most likely
to put a private sentence in front of somebody else — a stranger at a locked
desk, a room watching a shared screen, everybody who ever watches a recording.
So `crates/alo-notifying` decides three things and deliberately nothing about
how one looks: **who may send one, when one is shown, and what is never said
about one that is not.**

## What changed

**New crate `crates/alo-notifying`**, owned by this plan and named in its
header. Seventeen shipped source files, one per responsibility:

| File | What it holds |
|---|---|
| `src/notification.rs` | A notification: who, a title, a body, its actions — and no public constructor |
| `src/action.rs` | One thing offered: the sender's own name for it, and the words on it |
| `src/picked.rs` | One picked, addressed to the sender and to nobody else |
| `src/sender.rs` | An application, the agent, or alo OS — and an application can never be the agent |
| `src/arriving.rs` | The three doors, the first of which takes `alo_portals::Allowed` |
| `src/refusing.rs` | The five reasons one was never accepted, each naming the program |
| `src/quiet.rs` | Do-not-disturb: whether notifications are held now, and why |
| `src/quiet_hours.rs` | The stretch of the clock a person set aside, which wraps through midnight |
| `src/deciding.rs` | What became of one that arrived: shown, or held with a reason and nothing to draw |
| `src/shown.rs` | What a shell is handed: mark, word, then colour |
| `src/missed.rs` | What a person missed, in this session's memory, until they dismiss it |
| `src/changes.rs` | The two settings, and what alo OS ships |
| `src/keeping.rs` | `notifying.toml` (ADR 0038) |
| `src/unkept.rs`, `src/unreadable.rs` | What a person is told when that file did not read |
| `src/words.rs` | The twenty-three sentences this crate says |
| `src/testing.rs` | Fixtures, `cfg(test)` only |

**Registered** in `Cargo.toml`'s member list and in `alo-saying`
(`Cargo.toml`, and `src/collecting.rs` in all four places: `EVERY_LIST`, the
`declare` call, `ONE_STRING_EACH` and the per-crate count), so
`crates/alo-collected`'s check finds it collected rather than silent.

**The plan** is updated: task 6 marked done, and task 7's acceptance corrected
from *these four crates* to the five it now covers.

## A user-readable change description

> alo OS now has notifications. An application can only send one if you gave it
> permission to, and what it may send is small on purpose: who it is from, a
> line, a message, and at most three things to do. Do-not-disturb holds all of
> them and shows none — when you turn it on, during hours you set aside, and,
> without your having to remember anything, whenever your screen is being shared
> or recorded. If the machine cannot work out whether something is reading your
> screen, it holds them anyway. Nothing appears on the lock screen; whatever
> arrived while you were away is handed to you when you unlock. What you missed
> waits until you dismiss it, on this machine, and is never copied anywhere. The
> agent's own notifications are marked as the agent's and say so in words. And
> nothing an application puts on a notification can approve a change to your
> machine: approving happens where you went to approve something, and nowhere
> else.

## The acceptance, clause by clause

**A notification holds who sent it, a title, a body and its actions, arriving
through the notification portal under the sender's grant.** `Notification` has
four private fields and no public constructor. The only roads to one are in
`arriving`: `from_an_application`, which takes the `alo_portals::Allowed` that
`Request::judged` produced and refuses an `Allowed` for any of the other fifteen
portals; `from_the_agent`; and `from_alo_os`. An application's `Sender` is
`pub(crate)`, so there is no way to name an application in a notification
without a grant having been current when its request was judged.

**Do-not-disturb holds every notification and shows none**, turned on by a
person, by the hours they set aside, or automatically while the screen is shared
or recorded. `Quiet::now` asks `alo_in_use::InUse` **before** it reads a single
setting, and there is no setting anywhere in the crate that could turn that off.

**While locked, task 1's rule applies and nothing is shown.**
`deciding::arrives` hands the notification to `alo_locking::Seat::arrives`
before it looks at anything else. What comes back on a locked seat is
`alo_locking::Arrived::Held`, which carries nothing, so `Became::Held` carries a
`Why` and no notification — there is no branch in this crate that could draw
one, because there is nothing for it to draw.

**What a person missed is kept in a list on the machine until they dismiss it,
never synced.** `Missed` has no `serde`, no path and no file; the whole crate
opens nothing but the settings file, through `alo-kept`, at a path it is handed.

**An agent's own notifications are marked as the agent's with its mark and
word** (ADR 0010). `Shown::the_agents_mark` and the sentence *the agent* come
before `Shown::colour`, and `Sender::the_agent` refuses a grantee that is an
application.

**An application cannot notify with an action that answers an approval.**
`alo-approving` is a dev-dependency of the tests and **not** a dependency of the
crate; an `Action` is a name and a label; a `Picked` holds the sender and the
name and refuses an action the notification never offered. The test puts a real
proposal on a real approval surface and finds it still waiting after the person
has picked *approve*, *yes* and *allow*.

**Nothing draws, and no notification is read by an agent as context.** The
second is a test that reads every manifest in the workspace and the shipped
source of `alo-agentd`, `alo-turn`, `alo-capability`, `alo-protocol` and
`alo-broker`.

## Decisions this task made, and why

Nobody was available to ask, so these were decided the way a senior engineer
would and are written down rather than left to be discovered.

**1. What a person missed lives in the session's memory and reaches no disk.**
The plan says *kept in a list on the machine until they dismiss them, never
synced*. Writing that list to a file would satisfy the letter and would put the
most private text on the machine — a message's first line, who is trying to
reach somebody, the subject of a calendar entry — into a file that survives the
session, gets backed up, and is one feature request away from being copied. So
the list is in memory, and *never synced* is kept by **there being nothing to
sync** rather than by a switch that is off. The narrowing is real and is stated
in `src/missed.rs` and in the crate's own tests: signing out empties the list as
dismissing does. A later release that wants what-you-missed to survive a reboot
is making a new decision, with this paragraph in front of it.

**2. A machine that cannot tell whether its screen is being read holds
notifications.** `alo_in_use::InUse::read_from` refuses rather than answering an
empty list when the media server cannot be reached, and that refusal had to mean
something here. Showing notifications would be the comfortable reading and the
dangerous one: a private sentence shown into a recording cannot be taken back.
So `Quiet::now` takes the screen as an `Option` and `None` holds, with its own
sentence (`notifying.held.cannot-tell-about-the-screen`) saying plainly what the
machine does not know. This is the twenty-third string and was not asked for.

**3. Three things to do, at most.** A notification is read in the corner of an
eye while somebody is doing something else. A program that wants to put a menu
in front of a person has a window for that. `MOST_THINGS_TO_DO` is in this crate
rather than in the shell because it is a decision about what a notification *is*
— a shell that drew four would be drawing something this crate would never have
accepted.

**4. An action's name means nothing to alo OS, and there is no forbidden-word
list.** *approve* is accepted exactly as *reply* is. A list of reserved names
would be a crate that believed a name could do something; what stops an action
answering an approval is that a `Picked` has one destination and it is the
sender. There is a test that says so, because the absence of a reserved list is
the sort of thing somebody adds back in a hurry.

**5. A body is kept as its sender wrote it, minus anything that would redraw the
line.** A title is refused whole when it is not one readable line. A body is a
paragraph, and refusing a whole message over one stray byte loses the message —
while drawing one unaltered lets a sender write a sentence alo OS never wrote
onto somebody's screen. So control characters (line breaks included) are dropped
from a body and everything else is kept, in whatever language and script it
arrived in. This is the one place alo OS edits somebody else's text, and it is
documented on `Notification::body` where a reader will meet it.

**6. An application is named in a notification by its identifier.** Not by what
it calls itself. `alo_applications::Application` makes the argument at length —
two programs can call themselves *Mail* and no two share an identifier — and a
notification is exactly the surface where a borrowed name would pay. A shell
that knows the person's own application list shows the name beside the
identifier.

**7. A time of day is `alo-appearance`'s.** `alo-displays` spelled its own and
said why: the sun's arithmetic needs an infallible constructor from minutes.
Nothing here does, this crate already reads `alo-appearance` for terracotta, and
a third spelling of *22:00* on one machine would be a third place for the
twenty-fifth hour to be allowed. ADR 0038's concern — that a value in
`notifying.toml` must not be refused in another crate's words — is met by
`FileNotRead`: a stretch that begins and ends at once comes back as
`notifying.kept.not-understood`, naming this file, and a test holds that.

**8. What alo OS ships is *shown*, not *held*.** A machine that arrives silent
is a machine whose owner finds out a week later that nothing was ever reaching
them. What holds without being asked for is the screen being read, and that is
not a setting.

## Limitations, and what is owed elsewhere

- **Nothing here draws.** Where a notification appears, how long it stays, what
  it sounds like and what happens when somebody swipes it away are the shell
  plan's, from the decisions made here.
- **The notification portal is still not registered on the bus.**
  `alo_portals::Portal::answered_on_the_bus` answers `None` for it, and
  `docs/contracts/portals.md` says so. This crate is the model a backend would
  answer from; wiring it to `org.freedesktop.portal.Notification` is the
  applications plan's, and nothing in this change claims it.
- **`docs/contracts/person-settings.md` still describes four kept files** and
  names none of `sleeping.toml` (task 2), `displays.toml` (task 3),
  `leaving.toml` (task 5) or `notifying.toml`. That is now **four** files owed
  to that contract. What its section for this file would have to hold: the name,
  `alo_notifying::keeping`, format `1`, and the two keys `do-not-disturb` and
  `quiet-hours` — the second a table of `begin` and `end`, each `{ hour, minute
  }` — with the refusals under `notifying.kept.`: `not-read`, `not-understood`,
  `not-understood-at`, `another-format`, `unknown-key`, `not-written`,
  `not-expressible`, `not-replaced`. A `tests/the_contract_describes_this_file.rs`
  of the shape `alo-appearance`, `alo-dock`, `alo-applications` and
  `alo-shortcuts` already have would then hold the two together. It is one
  change covering four files and one contributor should make it; this task did
  not, because editing a shared contract for another plan's two files is exactly
  the collision `SHARED_MAIN.md` asks contributors not to have.
- **No hardware.** Nothing in this change touches a device. The nine-gate run
  and real-hardware acceptance remain the integration owner's.

## Verification

Platform: Windows Server 2022 checkout at `C:\dev\alo-os`; formatting through
WSL against `/mnt/c/dev/alo-os`; clippy and tests in the serialized Linux copy
at `/root/alo-trees/this-machine` with
`CARGO_TARGET_DIR=/root/alo-builds/this-machine`, as `tools/kernel-loop`'s
`gates.rs` runs them.

Executed, in the foreground, with the exit code read each time, and **every one
of them re-run on the tree as it now stands** — correction 4 below changed a
file, so nothing here is a result carried over from the first hand-over:

- `cargo fmt --all` — clean, and `cargo fmt --all --check` clean afterwards with
  no file rewritten.
- `cargo clippy --workspace --all-targets -- -D warnings` — exit 0, zero
  warnings. Run over the whole workspace rather than this task's crates, because
  this change adds a workspace member.
- `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS="-D warnings"` — exit 0,
  and `alo_notifying`'s documentation generated. **This is the gate that refused
  the first hand-over** (correction 4), so it was run whole and twice: once from
  cold, in 4m 59s, and once more for an exit code nothing had piped through.
- `cargo test -p alo-notifying` — **109 tests, all passing**: 67 in the crate's
  own source, 38 across its nine test files, and the doctests.
- `cargo test -p alo-saying` (68) and `cargo test -p alo-collected` (19) — all
  passing, which is what holds a new crate's words being collected rather than
  silent.
- Inside `tools/kernel-loop`, which gates natively: `cargo fmt --all --check`,
  `cargo clippy --all-targets -- -D warnings` and `cargo test` — exit 0, 152
  tests passing. Its suite is the one that reads every plan in
  `docs/autonomy/`, and this change edits one.
- Each of the eighteen tests in the handoff's `evidence` block, run on its own
  by its exact name with `--exact --include-ignored`, as `crate::evidence` runs
  them: each reported exactly one test passing.

Not executed here, and not claimed: the full workspace suite (the supervisor's),
the two BPF-target gates, and anything on certified hardware.

### Four corrections the gates found

This crate was written in a session that ended before its gates were run, and
`cargo test -p alo-notifying` refused it three times; the supervisor's own run
then refused the hand-over a fourth time, on a gate no worker here had run. What
the gates said, and what was done about it — because a report recording only the
last run would be a report of a suite that was green rather than of the work
that made it so:

1. **`src/testing.rs`: the test vocabulary was one crate short.** The sentence
   saying who a notification is from puts an application's own clause inside it
   (`applications.called`, which is `alo-applications`'), and a `Vocabulary`
   holding only this crate's words answered `«applications.called»` — the marker
   for a string nobody declared — in three unit tests. The fixture now declares
   `alo_applications::words::declare_into` beside this crate's, which is the
   shape every other crate's fixture already has. The integration tests never
   showed it: they build their `Strings` from `everything_this_machine_can_say`,
   which is the whole machine's vocabulary.

2. **`src/missed.rs`: a test expected a name the portal does not carry.** It
   asserted that what a person missed reads as *Sent by Mail
   (org.example.Mail)*. It reads as *Sent by org.example.Mail*, and that is
   right: what arrives through the portal is an `alo_portals::Allowed`, which
   carries the identifier a grant was made over and nothing an application wrote
   about itself — decision 6 above, made structural. The assertion now expects
   the identifier, with the reason written beside it; a shell that knows the
   person's own application list is where a name is drawn next to it.

3. **`tests/a_notification_cannot_answer_an_approval.rs`: one assertion claimed
   more than the design does.** It held that an `Action` built elsewhere with
   the same name *and the same label* as one a notification offers cannot be
   picked. An `Action` is its sender's name and the words on it, compared by
   value, so an action identical in both **is** the action the notification
   offers, and `Picked::of` accepted it.

   The choice was between giving an action a per-notification identity — a
   token, or picking by position — and correcting the claim. Identity was
   rejected: a `Picked` is addressed to that notification's own sender in every
   case, and the name it carries is always one that sender itself offered, so
   nothing escapes. What identity would buy is catching a shell that confused
   two notifications whose actions are indistinguishable, at the cost of an API
   where cloning an action makes it unpickable. So the assertion now says what
   holds and is the part worth holding: an action this notification never
   offered is refused, and so is one that differs in its label — a shell that
   relabelled a button would be reporting that the person picked something they
   were never shown.

   The clause the plan asks for is untouched, and it is the test above this one
   that carries it:
   `a_change_that_is_waiting_is_still_waiting_after_every_action_is_picked` puts
   a real proposal on a real approval surface and finds it still waiting after
   *approve*, *yes* and *allow* have each been picked.

4. **`src/action.rs`: a documentation link into a private module.** The
   supervisor refused the hand-over twice at the gate `rustdoc, warnings
   denied`, on one line: `src/action.rs`'s header said that picking an action
   produces a `Picked` addressed back to the sender ``([`crate::picked`])``, and
   `picked` is a private module. `Cargo.toml`'s `[workspace.lints.rustdoc]`
   denies `private_intra_doc_links` for exactly this, and says why: rustdoc
   silently *drops* such a link rather than rendering something false, so the
   reader meets an ordinary sentence and is never told that the thing it
   promised to point at is missing. The whole crate failed to document, and with
   it every crate the workspace had not finished documenting yet.

   The comment beside that lint also names the fix and forbids the other one:
   *the fix is never a wider `pub` — it is a code span naming the file, which is
   where the argument is anyway.* So `picked` stays private, as every module
   here does whose only public thing is re-exported at the root, and the
   parenthetical is now the file in a code span: *addressed back to the sender,
   whose half of the argument is made in `src/picked.rs`*. Three words of prose,
   no API surface widened, and the sentence says more than the dropped link
   would have.

   Nothing else in the crate links to a private item; rustdoc reports all of a
   crate's broken links in one run and reported this one alone, and the gate now
   passes over the whole workspace.

None of the four changed what this crate does for a person: the first was a
fixture, the second and third were assertions describing something other than
the design, and the fourth was a sentence in a header. They are written down
because *the refusal path tested as carefully as the happy path* is worth
nothing if a worker may hand over a suite it never watched finish — and because
the fourth is the one a worker's own `-p` gates cannot catch. `cargo test -p
alo-notifying` never runs rustdoc over the crate, so a link like that is
invisible to every check a worker is told to run and fatal to the one the
supervisor runs next. The cheap habit that would have caught it is `cargo doc -p
<crate> --no-deps` with `RUSTDOCFLAGS="-D warnings"`, which takes seconds
against a warm cache.

## Proposed shared-document updates

For the integration owner; this task edits none of them.

- **`CHANGELOG.md`** — the user-readable description above.
- **`ROADMAP.md`** — v0.5 *Notifications, with do-not-disturb*: the code and its
  tests, not drawn and not on hardware.
- **`docs/autonomy/QUEUE.md`** — task 6 of the session-and-displays plan done;
  task 7 (every sentence, and the walk) is the plan's last and is ready.
  `docs/contracts/person-settings.md` is owed four kept files and wants one
  contributor.
- **`docs/autonomy/STATE.md`** — reference this report.
