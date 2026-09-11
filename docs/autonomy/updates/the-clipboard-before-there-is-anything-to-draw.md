# The clipboard, before there is anything to draw

**Date:** 2026-09-11
**Workstream:** v0.01 delivery plan, task 20
**Contributor:** Claude (`C:\dev\alo-os-claude`)
**Status:** ready for integration

## What this closes

`docs/features.md` promises at v0.01: **copy, cut and paste — text, images and
files, across applications.** It was one of the six promises with no crate, no
test and no line anywhere, and until yesterday it was sorted into *work that
needs a machine*. Task 19 read that sorting against the repository and found it
wrong: **a clipboard is a protocol before it is a surface.** This is that
protocol.

`crates/alo-clipboard` is the selection and nothing else. One application owns
it and says which forms it can give; somebody asks for one of those forms; the
broker holds the offer and the way back to the owner, and holds nothing else. No
pixels are claimed and none are tested, no verb is added, and nothing in
`crates/alo-shell` is touched — the `wl_data_device` wiring is the desktop
lane's, and what is settled here is every value and every refusal that lane
would otherwise decide while wiring it.

## What changed

**New crate, `crates/alo-clipboard`** (nine files, one responsibility each):

- `kind.rs` — a form something can be given in: a media type, checked and
  lowercased, with the promise's three families read off it.
- `offer.rs` — what an owner put up: the forms, and copied or cut.
- `offered.rs` — what somebody about to paste holds, and *which* selection it
  names.
- `giving.rs` — the `Gives` port: the only place bytes ever come from.
- `clipboard.rs` — the broker: one selection, and the way back to its owner.
- `pasted.rs` — a completed transfer.
- `refusing.rs` — the five ways a paste does not happen, each with a sentence.
- `words.rs` — eight strings under a new `clipboard` area.
- `testing.rs` — the owners, the counter and the vocabulary its own tests use.

**Two integration tests**, one per half of the acceptance:
`tests/copy_cut_and_paste_across_applications.rs` and
`tests/the_clipboard_is_not_a_turns_context.rs`.

**Registrations**, which are the part a new crate fails the gate on: the
workspace's `Cargo.toml`, `crates/alo-saying`'s manifest and `collecting.rs`
(`EVERY_LIST` 22 → 23, the `declare` call, `ONE_STRING_EACH`, and the count that
holds the machine's vocabulary to the crates' own), and one sentence in
`crates/alo-collected/src/lib.rs` that counts the crates which declare words.

**`docs/autonomy/v0-01-evidence.md`**: the entry for *Copy, cut and paste* now
names what shows it and what is still owed, and a closing section records that
the count of promises with no evidence at all stands at **three**. The audit's
own paragraphs are left as it wrote them.

## What a person gets out of it

Copy and paste works the way a person expects and says so when it does not. What
is pasted is what was copied, in a form the application that copied it said it
could give — never something converted behind their back. When something cannot
be pasted, the machine says which of five things happened rather than doing
nothing: nothing has been copied yet, something else has been copied since, the
application that copied it has gone, it cannot be given in that form, or it did
not hand it over. All five are in the vocabulary a translator works from.

## Decisions this task made, and why

The plan left several things open. Each was decided the way a senior engineer
would and is written into the code beside the argument.

**A form is a media type, and the promise's three are three families of one.**
Text, images and files go through one door: an `Offer` is a list of `Kind`s and
nothing anywhere switches on which of the three is being moved. Three mechanisms
would be three sets of refusals to keep honest, and the fourth thing a client
offers — a font, a spreadsheet range, a colour — would arrive as a fourth path
nobody wrote. `text/uri-list` is checked before the `text/` family because it is
the one media type whose own family says the wrong thing.

**Retirement is ownership, not a flag.** `Clipboard::taken` replaces what is
held, which **drops the previous owner's `Gives`**. A transfer against a stale
handle moves nothing because there is nobody left to ask, rather than because a
check said no. That is the difference between a guarantee and a recommendation,
and it is the same shape `alo-telling` used for suppressing a repeat.

**A selection is named by a serial, never by what it looks like.** Two identical
offers from two applications are two selections. A clipboard that matched a
stale handle against the current offer by resemblance would hand the new owner's
data to somebody who asked the old one — a leak between two applications that
reads to everybody involved exactly like a paste. `tests/…across_applications.rs`
measures exactly that case, with two offers that are identical in every visible
way.

**`NothingCopied` and `NoLongerOffered` are two sentences.** The acceptance says
pasting with nothing copied must be *nothing to copy from* rather than the thing
before. They are kept apart because the repair is different — copy something,
versus the window you closed — and a handle is what tells them apart: holding one
is proof something was copied, so the *nothing copied* refusal is produced only
by `Clipboard::to_paste`, which is where a paste actually starts.

**Cut is one offer with a flag, and a cut moves once.** The promise names cut, so
it is here. `Taking::Cut` means the owner still holds what it cut and gives it up
when a transfer completes, which is the deferred cut a file manager makes; the
offer is retired at that moment, because a move that happened twice is not a
move. **An editor's cut is a copy here, deliberately**: a text editor deletes the
selection at the moment the key is pressed and puts a plain copy up, so there is
nothing left anywhere to move — and treating it as a cut would make cut text
unpastable a second time, which every person who has pasted one line into two
files would experience as the machine losing their work. A cut whose transfer
*failed* does not retire anything, because nothing moved.

**Everything a client sends is bounded and checked.** An application offering a
selection is not part of alo OS: a media type is length-bounded, ASCII and has a
subtype; an offer of nothing, an offer with one form twice, and an offer of more
forms than any real application makes are each refused by name. None of those is
a sentence a person reads — what has gone wrong is in an application — and none
is a partial repair, because a compositor that quietly deduplicated a client's
offer would be answering *which forms are available* with a list nobody wrote.

**Nothing here quotes what was copied.** No preview, no first line, no byte
count, no history, and a hand-written `Debug` that prints the offer and never the
bytes. A clipboard is where a password manager puts a password for the thirty
seconds between two windows; a sentence with a preview in it would put that into
a notification and into every screen recording taken while it was up.
`docs/features.md` schedules *clipboard history* at v1, so until somebody builds
that deliberately the absence is the feature.

## And the thing it had to show it does not do

**The clipboard is not `alo-context`'s selection.** ADR 0001 §4 offers an agent
the focused window, the highlighted text and the open document at the moment of
invocation and for that turn. What a person has copied is none of the three, and
a turn that read it would be the background reader `CLAUDE.md` calls a bug.

`tests/the_clipboard_is_not_a_turns_context.rs` shows it twice, because the two
halves answer different questions. A whole turn is begun over a real context
while a password manager owns the clipboard, and **the owner is never asked for
anything** — that is today. What stops tomorrow is that **neither crate can name
the other**: the test reads both manifests off the disk and holds `alo-context`
to naming no clipboard and this crate to naming `alo-context` only under
`[dev-dependencies]`. A turn cannot reach a clipboard because there is no
clipboard in scope to reach, and the same test holds this crate to depending on
no daemon, no turn, no record, no grants and nothing that asks a model.

**No verb is added**, per the constraint: the ten verbs in
`docs/contracts/agent-verbs.md` are six about files and four about applications,
and copy and paste is a person moving their own text between their own windows.

## Acceptance, one line at a time

| The plan's acceptance | The test |
|---|---|
| what is pasted is what was copied, in a type the copier offered | `what_is_pasted_is_what_was_copied_in_a_form_the_copier_offered` |
| a type never offered is refused in words rather than converted | `a_form_that_was_never_offered_is_refused_in_words` |
| taking the selection retires the previous owner's offer at once | `taking_the_selection_retires_the_previous_owners_offer_at_once` |
| pasting with nothing copied is *nothing to copy from* | `pasting_when_nothing_has_been_copied_says_nothing_has_been_copied` |
| text, images and files are offered types rather than three mechanisms | `text_images_and_files_are_three_forms_of_one_clipboard` |
| every string is in the vocabulary `alo-saying` collects | `every_string_this_crate_says_is_collected_into_the_machines_vocabulary` |
| a turn reaches nothing here | `a_turn_reaches_nothing_on_the_clipboard` |

Every one of them is measured against the machine's own vocabulary
(`alo_saying::everything_this_machine_can_say`) rather than against this crate's
own list, which is the question a crate's own tests cannot ask: `alo-overlay`
declared nine strings that nothing collected and every one of its own tests
passed.

**The refusals are tested as carefully as the answers**, and several of them are
tested by what did *not* happen: the owner's counter outlives the owner, so a
test can say *nobody was asked* rather than *nothing came back*. That is what
makes *refused rather than converted* and *a stale transfer moves nothing*
measurements instead of assertions.

## Verification

Windows 11, from `C:\dev\alo-os-claude`:

- `cargo fmt --all` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean, whole workspace.
- `cargo test --workspace` — everything passes **except six pre-existing
  Windows-only failures in `alo-recounting`**, which this change does not touch
  and does not cause. They are the same six named in
  `docs/autonomy/updates/every-crate-that-declares-words-collected.md` and
  `docs/autonomy/updates/every-decision-this-repository-points-at.md`, and they
  are deliberately not cut to green here: repairing another crate's
  platform-shaped failure inside a task about the clipboard would bury the
  repair where nobody looking for it would find it.
- `cargo test -p alo-clipboard` — 45 unit tests, 8 integration tests, 4
  doctests, all passing.
- `cargo test -p alo-saying`, `-p alo-collected`, `-p alo-reconciling`,
  `-p alo-citing` — the four registration gates, all passing with the new crate
  in place.

**Nothing was measured on hardware and nothing here needs to be.** No *On the
machine* box moves.

## Limitations, said plainly

- **Nothing draws it and nothing wires it.** No two applications on a real
  machine have moved anything through this. `smithay` 0.7 carries the protocol
  and `crates/alo-shell` enables the feature that holds it; that work is the
  desktop lane's and the evidence ledger says so under the promise.
- **Images and files are offered forms measured against a fixture.** The shape
  is the same for all three by construction, but no drawing program and no file
  manager has offered one.
- **A session holds no clipboard yet**, for the same reason `alo-telling`'s
  memory is held by nobody: there is no session object on this machine. Whoever
  builds one holds both.
- **`Gives::give` is synchronous.** A real compositor reads a pipe without
  blocking its loop; what is settled here is the decision around that read.
  Turning it into a descriptor and a read is the adapter's, and no rule in this
  crate moves when it happens.

## Proposed shared-document updates

I do not edit `CHANGELOG.md`, `ROADMAP.md`, `docs/autonomy/QUEUE.md` or
`docs/autonomy/STATE.md`. Proposed:

**CHANGELOG.md**, under Unreleased → Added:

> **Copy, cut and paste.** alo OS now has a clipboard: one application offers
> what it copied in the forms it can give, another pastes it in one of them, and
> what comes out is what was copied — never something converted on the way.
> Pasting what is not there says which of five things happened rather than doing
> nothing, in the language the person reads. What a person copies is theirs: the
> machine keeps no history of it, quotes it in no message, and no agent can
> reach it.

**QUEUE.md:** task 20 of the v0.01 delivery plan is done; task 21 is written in
the plan.

**STATE.md:** reference this report, and record that
`docs/autonomy/v0-01-evidence.md` now stands at three v0.01 promises with no
evidence at all, none of which this lane can close.

## Files touched

- `Cargo.toml`
- `Cargo.lock`
- `crates/alo-clipboard/Cargo.toml`
- `crates/alo-clipboard/src/lib.rs`
- `crates/alo-clipboard/src/kind.rs`
- `crates/alo-clipboard/src/offer.rs`
- `crates/alo-clipboard/src/offered.rs`
- `crates/alo-clipboard/src/giving.rs`
- `crates/alo-clipboard/src/clipboard.rs`
- `crates/alo-clipboard/src/pasted.rs`
- `crates/alo-clipboard/src/refusing.rs`
- `crates/alo-clipboard/src/words.rs`
- `crates/alo-clipboard/src/testing.rs`
- `crates/alo-clipboard/tests/copy_cut_and_paste_across_applications.rs`
- `crates/alo-clipboard/tests/the_clipboard_is_not_a_turns_context.rs`
- `crates/alo-saying/Cargo.toml`
- `crates/alo-saying/src/collecting.rs`
- `crates/alo-collected/src/lib.rs`
- `docs/autonomy/v0-01-evidence.md`
- `docs/autonomy/v0-01-delivery-plan.md`
- `docs/autonomy/updates/the-clipboard-before-there-is-anything-to-draw.md`
