# Every sentence, and the walk from lock to resume to a new desk

**Date:** 2026-09-20.
**Workstream:** `docs/autonomy/v0-5-the-session-and-the-displays-plan.md`, task 7
— *Lock screen, suspend and resume*; *Multi-monitor, scaling, hotplug*;
*Session management*; *Night light and display colour*.
**Machine:** `AGAI01`, checkout `C:\dev\alo-os-3`.
**Status:** ready for integration. Code and vocabulary evidence, run on this
machine; **nothing here is a claim about a lid that has closed, a machine that
has slept, or a screen that has been plugged into anything.** What each of those
is owed is under *What is not claimed*.

## What this task is

Six tasks before this one each ended in sentences a person reads, and each crate
holds its own sentences one at a time. This task asks the two questions none of
them can:

1. **Is every sentence these five crates can say in the machine's one
   vocabulary, with a note a translator can work from — and does any of them
   name `logind`, DRM, EDID or a connector name?**
2. **What does a person actually meet, in order, walking from the moment they
   lock their machine to the moment they dock it at another desk?**

The first is an audit. The second is a walk, and the walk found the thing the
audit could not: **a connector name was reaching a person through a gap.**

## What was found, and what was built because of it

`alo-displays` decided, in task 3, that a screen which says nothing about itself
is **remembered** by the socket it is plugged into. That decision is right and
is not re-opened here. What came with it, and was not decided so much as
inherited, is that the same screen was **named to a person** by the socket's own
string:

> eDP-1 does not say which screen it is, so alo OS remembers it by the socket it
> is plugged into …

`eDP-1` is what the kernel's side of a graphics card calls a connector. It is
printed on nothing, it is in no manual anybody who bought a laptop has read, and
it is the name of something alo OS rented — which `docs/features.md` says a
person never learns. It is also, on most machines, **the only screen there is**,
so this was not an edge case: it was the sentence the ordinary owner of an
ordinary laptop met the first time they opened the screens section of Settings.

No vocabulary audit can see it. Every sentence in `alo-displays` is clean; the
leak is in what gets *put into* `{display}`, which only exists when a real
screen has been reported. That is what a walk is for.

So this task built **`crates/alo-displays/src/plugged_into.rs`**: a socket
becomes a `PluggedInto`, and a `PluggedInto` is a string alo OS declares and
translates.

| The machine's name for it | What a person reads |
|---|---|
| `eDP-1`, `LVDS-1`, `DSI-1`, `DPI-1` | Built-in screen |
| `DP-2` | DisplayPort socket 2 |
| `HDMI-A-1`, `HDMI-B-1`, `HDMI-1` | HDMI socket 1 |
| `DVI-D-1`, `DVI-I-1`, `DVI-A-1`, `DVI-1` | DVI socket 1 |
| `VGA-1` | VGA socket 1 |
| anything else ending in a number | Socket 3 |
| anything else | Another screen |

Seven new strings, each with a translator's note, and the road from a screen's
identity into a sentence is now `Identity::named_in`, which puts a panel's make
and model in as **data** and one of these in as **words alo OS said**
(`alo_strings::Filling::and_said`) — so a German sentence with one of them
inside it is only as translated as the piece inside it, which is the rule
`alo-strings` already carries and which the socket's raw name quietly broke.

## Decisions I made, and why

Nobody was waiting to answer these.

**Fixing it rather than recording it.** The task's constraint says *nothing here
re-decides what the sentences describe*, and I read this as inside that line:
every sentence still describes exactly what it described. What changed is what
fills one gap. The stronger argument is the acceptance itself — *no sentence
names … a connector name* is a very specific thing to have written down, and the
only place in this workstream a connector name exists is `alo_displays::Socket`.
The clause is there because somebody expected this task to find it. Recording it
as a finding and ticking the task would have been ticking a criterion that was
not met.

**Names, not clauses.** *Built-in screen*, not *the built-in screen*. Each of
these goes into a gap where a make and a model would go, and the sentences
around them already treat that gap as a bare proper name — *what was open on it
is now on Dell U2720Q*. A clause with an article reads wrong at the start of a
sentence in English and worse in a language that inflects it. The note on each
one tells a translator it is a name and shows it sitting inside another
sentence.

**HDMI, DisplayPort, DVI and VGA are said out loud, on purpose.** They look like
rented names and they are not: they are printed beside the hole the cable goes
into, on the person's own machine. `alo_saying::rented` already draws that line
— the note on `shortcuts.modifier.super` tells a translator that most keyboards
print a Windows logo on that key, because that is what is under their reader's
thumb. The connector *string* is printed nowhere, which is the whole difference,
and the audit's check is for `hdmi-a`, `dp-`, `edp-` and their kin rather than
for the word *HDMI*.

**The last row is honest rather than clever.** A connector kind alo OS has no
word for, with no number to give, is *Another screen*. Two such screens would
read alike. That is a real cost and it is smaller than the alternative, which is
teaching somebody the word `SVIDEO-1` so that alo OS does not have to admit it
has nothing better. Such a screen is still remembered perfectly: the socket is
still its identity, and only the way of saying *that one* is lost.

**Where the two tests live, and why there was no choice.** They belong to the
workstream rather than to any one crate. Three of the five — `alo-sleeping`,
`alo-leaving` and `alo-notifying` — each hold a test that reads **every manifest
in the workspace** and refuses any dependant but `alo-saying` and `alo-shell`.
Those are real guarantees: a crate that answers an agent must not be able to
hold the machine awake, read what somebody had open yesterday, or listen to
their notifications. So **no crate in this workspace can name all five**, and
the only ways to give the two tests a tidier home were to widen one of those
lists or to put them in `alo-saying`, which this plan's own header says it reads
and never edits. Both were refused. They live in `alo-sleeping`, which is the
one of the three whose own rule does not stop it reaching `alo-locking` and
`alo-displays` — the other two crates the walk goes through — and the audit
therefore reads the **assembled vocabulary by key area** rather than five
crates' `EVERY_WORD` lists.

That is not a way around the question. It is a stricter form of it: an
`EVERY_WORD` is what a crate *claims*, and the vocabulary is what the machine
*says* after `alo-saying` has collected it, which is what a person meets. The
coverage is complete because each of the five already holds its own test that
every one of its keys sorts under its own area, so a sentence outside those five
areas fails in the crate that declared it before it reaches here. And the two
readings are shown to agree at least once: `alo-locking`, `alo-sleeping` and
`alo-displays` are read **both** ways, by name and by area, and the counts must
match.

**The walk goes one step past the plan.** The plan's walk is *lock, suspend with
the lid, resume, unlock, dock to a second display, undock*. The walk here adds
the cable going back in, because *and it comes back* is the whole of what hotplug
has to mean and it is the one sentence — `displays.windows-came-back` — that
undocking alone never produces.

## The walk, sentence by sentence

One person's day and a half, in the order they meet it. The sentences are what
`alo_saying::everything_this_machine_can_say()` really says, in English —
somebody reading a language nobody has written yet reads exactly this, which is
`alo-strings`' promise and the reason a missing line is survivable.

| Step | Moment | What a person reads |
|---|---|---|
| 1 | Anna signs in, and alo OS says what it is holding sleep off for | Deciding what closing the lid does |
| 2 | Anna signs in, and alo OS says what it is holding sleep off for | Locking this machine before it sleeps |
| 3 | She opens the screens section of Settings for the first time | Built-in screen has not been used with this machine before, so alo OS has put it beside your other screens and chosen a size for it from how large it is |
| 4 | She opens the screens section of Settings for the first time | Built-in screen does not say which screen it is, so alo OS remembers it by the socket it is plugged into — another screen plugged into that socket will be set up the same way |
| 5 | Nobody touches the machine, and the agent is still working | This machine is staying awake because the agent is still working on what you asked |
| 6 | She locks the screen and leaves the desk | This machine is locked |
| 7 | She presses the key that opens the agent, at the locked screen | This machine is locked |
| 8 | She opens the lid the next morning, and the machine wakes | This machine is locked |
| 9 | She types a password that is not hers | that name and password do not sign anyone in on this machine — check both and try again |
| 10 | She types her own, and the machine unlocks | The agent stopped working on what you asked, because its time ran out while this machine was asleep. Ask again to start over. |
| 11 | At the office she plugs in the screen she uses there | Dell U2720Q has not been used with this machine before, so alo OS has put it beside your other screens and chosen a size for it from how large it is |
| 12 | She closes the lid with the office screen attached, as she chose | This machine is staying awake with its lid closed because another display is attached and you chose that |
| 13 | At the end of the day she pulls the cable out | Dell U2720Q was unplugged, so what was open on it is now on Built-in screen |
| 14 | At the end of the day she pulls the cable out | Your screens are arranged the way you last left them |
| 15 | She puts it back a moment later | Dell U2720Q is back, so what was open on it before has gone back to it |
| 16 | She puts it back a moment later | Your screens are arranged the way you last left them |

`crates/alo-sleeping/tests/the_walk_from_lock_to_resume_to_a_new_desk.rs` parses
this table out of this file rather than holding a copy of it, so a sentence that
changes without the table changing fails, and so does a table edited to say
something the machine does not. A later change that moves a sentence publishes
the table again in a follow-up report and points that test's `THE_REPORT` at it;
a published report is never rewritten.

### What the walk shows that no single crate could

- **The sleep itself says nothing, and there is no row for it.** Between step 7
  and step 8 the lid closes, the seat locks, the machine suspends and it wakes.
  Not one sentence. That is what a machine that goes to sleep and comes back
  does, and inventing a row for it would have been the easiest dishonest thing
  in this task. The walk asserts it happened — the seat is locked before
  anything reaches the base, the base was asked to sleep exactly once, and the
  seat that comes back is still locked — and records no sentence, because there
  is none.
- **Steps 6, 7 and 8 are the same sentence three times, deliberately.** Locking,
  reaching for the agent at a locked machine, and waking from a whole night's
  sleep all read *This machine is locked*. `alo-locking` declares **one** string
  and says why: a second sentence naming what was refused would be a second
  thing on a screen whose whole rule is that it shows four. The walk is where
  that decision stops being a paragraph and becomes something a person reads
  three times in a row without learning anything they should not.
- **Step 9 is not this workstream's sentence at all.** A wrong password at the
  lock screen answers with `alo-accounts`' own refusal, carried through the
  greeting unchanged — because unlocking *is* signing in, and a lock screen with
  a refusal of its own would be a second, weaker road. Reading it in sequence is
  the proof: the sentence a person meets at 08:40 at their own locked laptop is
  the same one they met the day they first signed in.
- **Step 10 arrives after the unlock and not before it.** What became of the
  agent's work is decided and written down at the *wake*, and shown at the
  *unlock*. A stranger who opens the lid learns nothing about what anybody asked
  the agent to do.
- **Steps 13 and 14 read as one thought, from two crates' worth of decisions.**
  The cable comes out, the windows go to the screen that remains, and the same
  breath says the screens are back the way she left them — because the set of
  screens she is left with is a set she has used before and `alo-displays`
  remembered its arrangement.
- **Steps 3, 4 and 13 are where the connector name used to be.** *Built-in
  screen*, three times, in three different sentences from two different files.

## What is not claimed

- **No lid, no sleep, no resume, and no screen.** The walk's machine is a
  `Logind` implementation in the test file that counts what it was asked and
  keeps the sentence it was asked with. `alo-sleeping`'s real hold was measured
  against a live `logind` in WSL when task 2 was built; **no sleep and no lid
  have been measured on certified hardware**, `docs/hardware.md` still lists no
  certified machine, and no v0.5 line ticks here.
- **Nothing was drawn.** The lock screen in the walk is an
  `alo_locking::LockScreen` value; the notes are `alo_displays::Note` values.
  The surfaces that show them are the shell plan's later tasks.
- **The session and the turn are real.** Anna's `alo_accounts::Session` exists
  because a password verified; the turn is an `alo_turn::Turning` writing to a
  real record, and the sentence in step 10 is what `Woke::a_turn` really
  produced for a turn whose hour ran out overnight.
- **No claim about `alo-leaving` or `alo-notifying` beyond their words.** The
  audit reads every sentence those two crates say, in the assembled vocabulary.
  The walk does not reach them: logging out, switching user and a notification
  arriving are not on the road from a lock to a new desk, and the one place the
  walk touches a notification is `alo_locking::Seat::arrives`, which it asserts
  holds it and draws nothing. Their own tasks' tests are their evidence.

## Two findings recorded rather than fixed

**A dock per display is still not met, and it is now unblocked.**
`docs/features.md`, under *Making it yours*: *Per display, so the dock can sit
along the bottom of the laptop and down the side of the external screen.* Task 3
recorded that it is **not met** — `alo_dock::Dock` holds one edge for the whole
machine — and could not fix it, because `alo-dock` belonged to
`v0-5-where-a-persons-settings-are-kept-plan.md` while that plan still had
unfinished tasks. Every one of its seven tasks is now marked done, so the crate
is released. This task did not take it: a per-screen edge is a change to a kept
file's shape and to another plan's crate, which is a task rather than a passing
fix, and folding it into the workstream's audit would have put two subjects in
one change. **It is not written as task 8 of this plan either**, and that is
deliberate: this plan's header says it *reads and never edits* `alo-dock`, so
taking it needs that paragraph changed and the settings plan's matching claim
dropped in the same change, and inventing a double claim on a crate is the one
thing `tools/kernel-loop/src/who_owns.rs` calls an error in itself. It belongs in
`docs/autonomy/QUEUE.md` as its own item, for whoever holds both plans' headers
at the time.

**`docs/contracts/person-settings.md` still describes four kept files.** It
names none of `sleeping.toml` (task 2), `displays.toml` (tasks 3 and 4),
`leaving.toml` (task 5) or `notifying.toml` (task 6). Tasks 4, 5 and 6 each
recorded this; it is now five files owed to that contract, and this change adds
seven keys to `displays.toml`'s vocabulary without touching the file's shape.
It is not this task's to pay — the contract belongs to
`v0-5-where-a-persons-settings-are-kept-plan.md` — and it is repeated here so
that the count is right when somebody does pay it.

## What changed

| Path | What it is |
|---|---|
| `crates/alo-displays/src/plugged_into.rs` (new) | What a screen that says nothing about itself is called, and the parse from a connector name to one of seven words |
| `crates/alo-displays/src/words.rs` | The seven new strings with their notes; five notes corrected, because `{display}` is no longer always untranslated text; the gap list gains `number` |
| `crates/alo-displays/src/identity.rs` | `Identity::as_a_person_reads_it` replaced by `Identity::named_in`, which is the only road from a screen into a sentence |
| `crates/alo-displays/src/notes.rs`, `src/coming_and_going.rs`, `src/arrangement.rs` | The five places that named a screen, through the new road |
| `crates/alo-displays/src/lib.rs` | The new module in the crate's own table of what is here |
| `crates/alo-sleeping/tests/every_sentence_this_workstream_says.rs` (new) | The audit: every sentence of the five collected, noted, and naming nothing of the machine's — sentence, note and key |
| `crates/alo-sleeping/tests/the_walk_from_lock_to_resume_to_a_new_desk.rs` (new) | The walk, held to the table above |
| `crates/alo-sleeping/Cargo.toml` | Two dev-dependencies for those tests, with the reason the walk lives here |
| `docs/autonomy/v0-5-the-session-and-the-displays-plan.md` | Task 7 marked done; the header's *reads and never edits* unchanged |

### A user-readable change description

> A screen that does not say what make and model it is — which is nearly every
> laptop's own screen — is now called *Built-in screen*, *HDMI socket 1* or
> *DisplayPort socket 2* wherever alo OS names it, instead of by the code its
> graphics driver uses. Every sentence alo OS says about locking, sleeping,
> screens, logging out and notifications has been checked against the one list
> it translates from: each has a note for whoever translates it, and none of
> them names any part of the machinery underneath. The exact sequence a person
> meets — from locking their laptop in the evening to plugging in a second
> screen at the office the next day — is written down and held by a test, so it
> cannot change by accident.

## Verification

Run on `AGAI01`. The Windows checkout at `C:\dev\alo-os-3` cannot build the
tests: `ring`, reached through `alo-saying`'s dependency on `alo-asking`, needs a
C compiler and this host has no `gcc.exe`. So every command below was run in WSL
2 Ubuntu against the serialized source copy at `/root/alo-trees/this-machine`
with `CARGO_TARGET_DIR=/root/alo-builds/this-machine`, which is the one build
cache this machine shares. No other Cargo process was running.

Every line below was run in the foreground and its exit code read.

| Gate | Command | Result |
|---|---|---|
| Formatting | `cargo fmt --all` then `cargo fmt --all --check` | exit 0, clean |
| Clippy, warnings denied | `cargo clippy -p alo-displays -p alo-sleeping -p alo-locking -p alo-saying -p alo-collected --all-targets -- -D warnings` | exit 0, no warning |
| Tests | `cargo test -p alo-displays` | exit 0 — 110 lib + 20 integration + 2 doc = **132 passed, 0 failed** |
| Tests | `cargo test -p alo-sleeping` | exit 0 — 37 lib + 19 integration = **56 passed, 0 failed**, of which this task's are 8 + 4 |
| Tests | `cargo test -p alo-locking -p alo-saying -p alo-collected` | exit 0 — **127 passed, 0 failed** |
| Tests | `cargo test -p alo-citing -p alo-conforming -p alo-reconciling` | exit 0 — **75 passed, 0 failed** (the citation check a `docs/` change reaches) |
| Tests | `cargo test` in `tools/kernel-loop` | exit 0 — **152 passed, 0 failed** (the plan checks, which read every plan and so read this task's edit to one) |
| Rustdoc, warnings denied | `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-displays -p alo-sleeping --no-deps` | exit 0, no warning |

The exact counts are in the handoff's evidence block, one line per acceptance
criterion, each run on its own.

**The full workspace suite was not run here**, deliberately: it takes the better
part of an hour on this machine and the supervisor runs it on the committed tree
regardless. The crates touched and the crates that read them were run.

## Acceptance, against the plan

| The plan asks | Where it is |
|---|---|
| every sentence these five crates can say is in the vocabulary | `alo-sleeping`, `every_sentence_this_workstream_says`, `the_five_crates_of_this_plan_are_collected` and `what_the_crates_declare_and_what_the_machine_says_are_the_same_list` |
| … with a translator's note | the same file, `every_sentence_carries_a_note_a_translator_can_work_from`, and `every_sentence_comes_out_whole` beside it |
| one walk — lock, suspend with the lid, resume, unlock, dock, undock — produces the exact sequence a person meets, recorded as a table and held by one test | `alo-sleeping`, `the_walk_from_lock_to_resume_to_a_new_desk`, `the_walk_from_lock_to_resume_to_a_new_desk_reads_as_the_table`, against the table above |
| … that fails if a sentence changes without the table | the same file, `only_the_walks_own_table_is_read_and_a_changed_sentence_is_a_difference`, shown refusing a reworded sentence, a reordered walk, a table under another heading and no table at all |
| no sentence names `logind`, DRM, EDID, a connector name or any other part of the machinery | `no_sentence_or_note_names_the_machinery` and `no_connector_name_is_in_anything_a_person_reads` over the assembled vocabulary; `no_sentence_on_the_walk_names_the_machinery` over the sequence; `alo-displays`' `plugged_into::tests::no_connector_name_reaches_what_a_person_reads` over every socket a machine could report |
| *nothing here re-decides what the sentences describe* (the constraint) | no verb, no argument, no road, no ADR and no kept file's shape changed. `alo-displays` keeps remembering a screen by its socket exactly as task 3 decided; what changed is the word a person reads for that socket, which is what the acceptance above required |

## Remaining limitations

- Nothing here has met a lid, a suspend, a monitor or a certified machine.
- Two screens on unrecognised connector kinds with no number would read alike as
  *Another screen*. Stated in the word's own note and above.
- `docs/contracts/person-settings.md` is still four files behind; see the
  finding.
- The surfaces that draw any of this are the shell plan's.

## Proposed updates to the shared documents

For the integration owner; this report does not edit them.

**`CHANGELOG.md`**, under Unreleased:

> - A screen that does not say what make and model it is — nearly every laptop's
>   own screen — is now called *Built-in screen*, *HDMI socket 1* or
>   *DisplayPort socket 2* wherever alo OS names it, rather than by the code its
>   graphics driver uses. Every sentence alo OS says about locking, sleeping,
>   screens, logging out and notifications is in one list with a note for
>   whoever translates it, and none of them names any part of the machinery
>   underneath. The exact sequence a person meets, from locking their laptop in
>   the evening to plugging in a second screen at the office the next day, is
>   written down and held by a test.

**`docs/autonomy/QUEUE.md`**: `v0-5-the-session-and-the-displays-plan.md` task 7
is done. Task 8 — *Switching to another person at a locked screen* — is written
in the plan and ready, from the finding task 5 recorded. And a new item, owned by
nobody yet: **a dock per display**, which `docs/features.md` promises, task 3
recorded as not met, and which is now unblocked because every task of
`v0-5-where-a-persons-settings-are-kept-plan.md` is done; taking it means moving
`alo-dock` between the two plans' headers in one change.

**`ROADMAP.md`**: no v0.5 line ticks here. *Lock screen, suspend and resume* and
*Multi-monitor, scaling, hotplug* still owe their evidence on certified
hardware.

**`docs/autonomy/STATE.md`**: reference this report.
