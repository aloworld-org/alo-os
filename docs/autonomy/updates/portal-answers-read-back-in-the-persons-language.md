# What applications were answered reads back in the person's language

**Date:** 2026-09-15
**Workstream:** v0.5, applications and what they expect
(`docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`, task 10)
**Contributor:** Claude worker on the Mac checkout, for the supervisor to gate
and publish.
**Status:** ready for integration.

## What changed

The portal answers file (`/var/lib/alo/portal-answers.jsonl`) holds
identities, never sentences. Until now nothing turned what it holds back into
words: `Outcome::said` words an answer only while the backend holds the
decision, and `KeptAnswer` and `ReadBack` — what survives the backend
stopping — said nothing. They now do.

- **`KeptAnswer::said(&Strings) -> AnswerSaid`**
  (`crates/alo-portals/src/kept_answer_said.rs`): the moment, handed back as a
  `SystemTime`, and three sentences. *Asked by org.gnome.Fractal*, or *Asked by
  an application that could not be named*. What the portal lets an application
  do (`Portal::said`, task 1's sentence). And what it was answered with.
- **What it was answered with** (`crates/alo-portals/src/kept_outcome_said.rs`)
  is said with **the keys `Outcome::said` uses, filled the same way**. For what
  the backend answered, those are this crate's words. For `never-granted` and
  `lapsed` they are `alo-capability`'s `APPLICATION_NEVER_GRANTED` and
  `APPLICATION_LAPSED`, and for a kind nothing opens they are
  `alo-applications`' `NOTHING_OPENS` and `NOTHING_OPENS_CHOICE_NOT_INSTALLED`.
  `NotARequest` and `Unanswered` use their own `said`.
- **`ReadBack::said(&Strings) -> ReadBackSaid`**
  (`crates/alo-portals/src/read_back_said.rs`) gives one of two sentences:
  *Nothing has been removed from what applications on this machine were
  answered*, or *… does not go all the way back — older answers were removed
  under how long this machine keeps its record*. Beside it come `since` as a
  moment, the rule `under` said by `alo_keeping::Keeping::said`, every answer
  said, and the count of lines that did not read. `ReadBack::is_whole` is new.
- **Words** (`crates/alo-portals/src/words.rs`): eight new words and one
  plural, all under `portals.read-back.*`, each with a translator's note.
  - Sentences: `asked-by`, `not-a-kind-anything-opens`, `file-not-read`,
    `whole`, `shortened`.
  - Clauses: `nobody-named`, `what-it-asked-for`, `that-kind-of-file`. These
    are listed in the new `alo_portals::CLAUSES` so the *begins a sentence*
    test knows them.
  - The plural: `lines-not-read`.

  `declare_into` now also declares the plural, and `WordsError` gains
  `Counting`. `alo-saying` already collects the crate, so nothing needed
  registering.
- **Contract:** `docs/contracts/portal-answers-file.md` gains *Reading it to a
  person*, and its owner and *held by* lines name the new modules and test.

**Change description (for `CHANGELOG.md`):** *What applications asked for and
were told can now be read back in the person's own language after the portal
backend has stopped. Each answer says which application asked, or that none
could be named, what the portal lets it do, and what it was told, in the same
words it was told at the time. The list says whether it still reaches back to
its first answer or was shortened under the machine's record rule, and from
when. Lines that could not be read are counted beside the rest rather than
silently dropped.*

## Decisions

1. **Reuse wherever the file holds enough to fill the same sentence; a clause
   where it does not; a new sentence only where the sentence itself depends on
   what was not kept.** The file keeps no path, no kind of file, and not which
   of `alo-opening`'s findings made a file unopenable.
   - A portal over a facility loses nothing. What was asked for is the portal's
     facility, and a grant covering a facility was over that facility alone. So
     `never-granted` and `lapsed` on a facility portal read back **byte for
     byte** as the backend said them. A unit test and the acceptance test both
     compare against `Outcome::said`.
   - A portal over a path keeps `alo-capability`'s sentence with *what it asked
     for* in `{wanted}` and `{reach}`. A kind nothing opens keeps
     `alo-applications`' sentence with *that kind of file* in `{what}`.
   - `the-file` and `unreadable` get two sentences of this crate's, because
     `alo-opening` picks a different sentence for each finding, and inventing
     which one would be a guess.

   The other option was to extend the file to keep the path and the kind. I
   rejected it. It is a format change to a public surface, and it would put
   file names into a machine-wide record. Lines written before the change would
   still need these clauses anyway.
2. **Three sentences and a moment, not one composed sentence.** A portal's
   sentence (*Lets an application …*) is a whole sentence, and gluing it inside
   another would leave translators unable to reorder it. The moment is never
   written into a sentence, for `alo_keeping::Head::since`'s reason: how a date
   is written belongs to the region, not to the language. Whatever draws the
   list places the three and formats the moment.
3. **A line with no `application` whose outcome implies one** (for example a
   hand-written `secret-handed-over` with no application) still reads. Its
   `{application}` gap is filled with *an application that could not be named*,
   so no gap is ever left empty. I considered tightening `KeptAnswer::read` to
   refuse such lines. That would change task 8's reading rules, which is beyond
   this task, and the backend never writes such a line.
4. **Two present-tense phrasings remain as `alo-applications` wrote them.**
   *{chosen}, which you chose for it, is no longer installed* was true when the
   backend answered and may not be when the file is read. Reuse was the
   acceptance, and that crate is not this plan's to edit.
5. **`not_read` is `None` only when every line read.** One or more gives the
   plural sentence (*One line …* / *2 lines …*). The line numbers stay on
   `ReadBack::unreadable`.

## Acceptance criteria and where each is held

All in
`crates/alo-portals/tests/what_applications_were_answered_reads_back_in_the_persons_language.rs`:

| Criterion | Test |
|---|---|
| `KeptAnswer` worded through `alo-strings`: the application or *an application that could not be named*, the portal's sentence, what it was answered with | `the_file::an_answer_is_said_as_who_the_portal_and_what_it_was_told` |
| …in the language the person reads | `the_file::an_answer_is_said_in_the_language_the_person_reads` (German preferred; identifier untouched; `is_translated`) |
| Reusing the words `Outcome::said` uses | `the_file::an_answer_reads_back_in_the_words_it_was_answered_with` (kept with `AnswersFile`, read back, compared sentence for sentence, including all three refusals of the grants) |
| Every `KeptOutcome` kind worded, no key shown in place of a sentence | `the_file::every_kind_of_kept_outcome_is_a_sentence`: 28 kinds, each from a named application and from nobody, each on a facility portal and a path portal, read off a real file. An exhaustive match fails to compile when a kind is added. |
| `ReadBack` says whole or shortened, and from when, the moment a moment | `the_file::a_whole_file_and_a_shortened_one_say_so_and_from_when` (also shortened twice, and `forever` after) |
| Lines that did not read said as a count beside what did, never dropped | `the_file::lines_that_did_not_read_are_counted_beside_what_did` (a non-identifier line, a torn last line, one and two, and none) |
| New strings declared in `alo_portals::words` and collected by `alo-saying` | `every_string_read_back_is_collected`, plus the unit tests in `words.rs` |
| Contract | `the_contract_says_how_the_file_is_read_to_a_person` |

## Verification

Run on the Mac checkout, with every gate inside the Lima VM `alo` (Ubuntu,
`limactl shell alo sudo`), where `CARGO_TARGET_DIR=/root/alo-builds/alo-os-a96435785b5e7d7d`
is set as the supervisor sets it:

- `cargo fmt --all -- --check`: clean.
- `cargo clippy --all-targets -- -D warnings` (workspace): exit 0, no warnings.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-portals --no-deps`: exit 0.
- `cargo test -p alo-portals`: every target passed. That is 44 unit tests,
  8 in the new test file, the 7 existing integration files and the doc-test.
- `cargo test -p alo-saying`: passed (63 + 4 + 1). I ran it because it collects
  the new words, though the crate itself was not edited.

Not run, as instructed: the full workspace suite, which the supervisor runs.

### The first handoff was refused, and why

The supervisor refused the first handoff before building anything. The Linux
VM that runs the gates had less than 12 GiB free on `/`, and the gates keep
that much in reserve. The refusal concerned the machine, not the code. A second
worker took over on 2026-09-15 and found no fault in the change. They reran every
gate above inside that VM (`limactl shell alo`), using this checkout's own
build directory. All passed with the tree unchanged, and every evidence test
was listed by name. When they checked, the VM's `/` still had 9.7 GiB free, and
nearly all of the used space was this checkout's own build directory (42 GiB).
Nothing shared was deleted. Space has to be made on that disk before the
supervisor will gate this again. The code cannot fix that.
Nothing here touches hardware or a bus beyond what the existing portal tests
already start.

## Limitations

- Nothing draws the list, starts the backend in a session, or decides who may
  read the file. All three were excluded by the task.
- The file has no path or kind, so the read-back of a refused file request
  cannot say which file. That is by design (decision 1).
- **Whose file it is remains undecided.** The backend runs on a session bus, but
  `THE_ANSWERS` is one machine-wide path. I wrote this into the plan as task 11.

## Proposed updates to shared documents

- **CHANGELOG.md:** the change description above.
- **QUEUE.md / STATE.md:** v0.5 applications plan task 10 is done (this
  report); task 11, *Whose record of applications' answers it is, decided
  before anything shows it*, is written and ready.
- **ROADMAP.md:** no change.
