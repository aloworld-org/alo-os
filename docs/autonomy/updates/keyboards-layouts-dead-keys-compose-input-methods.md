# Keyboards: layouts, dead keys, compose, input methods

**Date:** 2026-09-18
**Workstream:** v0.5 — hands on the desktop
(`docs/autonomy/v0-5-hands-on-the-desktop-plan.md`, task 6)
**Contributor:** Claude Code, in `C:\dev\alo-os-b`
**Status:** ready for integration

## What changed, in a sentence a person outside this repository can read

Choosing a language now chooses the keyboard people of that language actually
type on — all twenty-four of them, Greek and Maltese and Irish included — so
nobody has to hunt through two hundred entries for their own. A second keyboard
is added by naming a language rather than a code, one shortcut switches between
them, and the one in use is shown in a small square in the status area. `ü`,
`è`, `ß`, `ł`, `ő`, `č`, `ġ`, `ħ`, the Irish fadas and the Maltese letters are
typed with a dead key or a compose key **through the tables the system already
ships** — alo OS writes no keyboard data of its own — and a key sequence those
tables do not have types nothing at all rather than a guess. Chinese, Japanese
and Korean are added the same way, by naming the language: the software that
does the work is rented, it is never named at a person, and it does not run at
all on a machine where nobody asked for it. All of it is kept in the person's
own folder, and a file that has been edited by hand and is wrong is refused
whole with the file and the line named, never half-read.

Nothing draws and nothing presses a key. This crate decides; the compositor
will honour it.

## The crate, and what is in it

**`crates/alo-keyboards`**, new, the third of the three the plan owns.

| File | What it is |
|---|---|
| `layout.rs` | A keyboard as the rented data names it: `de`, `us(intl)`, checked once |
| `offering.rs` | **The only keyboard data alo OS owns** — which keyboard is offered with which language, for all 24, and the others its speakers use |
| `rented.rs` | The machine's own list of keyboards (`evdev.lst`), read and never written |
| `keysym.rs` | One key, named the way the rented tables name it |
| `compose.rs` | The rented compose table, read: which sequence writes which letter |
| `typing.rs` | A dead key waiting, and the letter it is put on |
| `compose_key.rs` | Which key composes, out of the rented options |
| `switching.rs` | The one shortcut that switches keyboards |
| `methods.rs` | Input methods for the scripts a keyboard cannot type |
| `these.rs` | A person's keyboards, as a list that cannot be empty |
| `keyboards.rs` | The whole of it, and every change to it |
| `changes.rs`, `keeping.rs`, `unkept.rs` | `keyboards.toml` in the person's folder (ADR 0038), and what they are told when it did not read |
| `refusing.rs`, `words.rs`, `testing.rs` | Why nothing changed, every sentence, and the fixture |

Registered in `crates/alo-saying` (dependency, `EVERY_LIST` 56 → 57, a `declare`
call, `ONE_STRING_EACH` and the count), which is what `crates/alo-collected`
holds every word-declaring crate to, and added to the workspace members.

## The decisions this task made

**Nothing rented is linked; the rented data is read.** `libxkbcommon` has a
compose implementation and a C library to link it from. This crate reads
`/usr/share/X11/xkb/rules/evdev.lst` and
`/usr/share/X11/locale/en_US.UTF-8/Compose` as files instead, for three reasons
that all point the same way: the workspace `forbid`s `unsafe_code` and every
dependency in it today is pure Rust, a linked C library would stop this crate
compiling on the machine half this repository's contributors work from, and the
thing the plan actually asks for is that **the letters come from the rented
tables rather than a table of ours** — which reading the file satisfies exactly
as well as linking the library does. What is not rented, and is now written
down as a consequence, is a reader for two small line-based formats: about
seventy lines in `rented.rs` and `compose.rs`, each refusing whole rather than
part-way. If a later task needs the rest of what `libxkbcommon` does — a key
event turned into a keysym, which is the compositor's question and not this
crate's — that is where an ADR about linking it belongs.

**A line the compose reader cannot read refuses the whole table.** The rule
`alo_kept` states for a person's own file, applied to the machine's: half a
table is the machine choosing the other half, and a person who typed `ő` and got
nothing would have no way to know alo OS had quietly dropped the line. The
refusal names the line. Against the table this machine ships — 5 145 sequence
lines — every line reads.

**The switch is `Alt+Space`, not `Super+Space`.** `Super+Space` is what most
systems use for this, and in alo OS it is the launcher's:
`alo_shortcuts::DEFAULTS` binds it, and **this plan reads `alo-shortcuts` and
never edits it**, so putting the keyboard switch there would have been two
things on one chord — the failure `alo_shortcuts::Clash` exists to catch. There
is no *switch keyboard* action in that crate's closed list and inventing one
here would put two lists of shortcuts on the machine. So the switch is the
rented option `grp:alt_space_toggle`, which nothing in the release binds, held
by a test; `switching.rs` says in its own header that the chord becomes
`alo-shortcuts`' value the moment that crate has an action for it. A person who
binds something else onto `Alt+Space` is **told which shortcut is on it**, in
one sentence naming both — alo OS does not move a shortcut somebody chose, and
does not pretend the switch still works.

**Dutch is offered `us(intl)`, not `nl`.** The Netherlands types on US
keyboards; the `nl` layout exists in the rented data and almost nobody uses it.
`us(intl)` puts the diaereses and accents on dead keys of a keyboard whose
keycaps match the machine a person bought. `be` and `nl` are both offered beside
it, and this is why every language carries *also* entries at all: a language is
not a country — German is `de`, `at` and `ch`; French is `fr`, `be`, `ca` and
`ch(fr)` — so the offer is a starting point with the neighbours in reach, not a
verdict.

**No compose key is set until a person sets one.** Every candidate is somebody's
key already, and the worst of them is Right Alt: that is AltGr, which is how
half of Europe types `€`, `@`, `ł` and `ß`. Taking it by default would break
typing on exactly the keyboards this product exists for. The six choices offered
are all rented options this machine has, checked by a test, and the letters the
plan names are reachable on each language's own layout without a compose key at
all.

**A person's keyboards are a list that cannot be empty** (`these.rs`), rather
than a `Vec` with a rule somebody remembers to check. So *which keyboard am I
typing on* has an answer instead of an `Option`, there is no arithmetic that can
run off the end of it under `indexing_slicing`, and *you cannot remove your last
keyboard* is the type saying no.

**An input method is added by naming a language.** `libpinyin`, `chewing`,
`mozc-jp` and `hangul` are names this crate and the rented framework use to
agree; a person names Japanese. And **nothing starts on a machine that needs
none**: `Keyboards::what_the_session_starts` answers `None` until somebody has
added one, because a framework started for everybody in case somebody needs it
is a process reading every keystroke on machines where nobody asked for it.
A Greek speaker who looks for Greek in that list is refused and **sent to the
Greek keyboard**, in a sentence that names Greek as *Ελληνικά*.

**No sentence this crate can say names a rented part** — not a layout code, not
a keysym, not XKB, not the input-method framework — which is task 7's constraint
met here rather than deferred to it. It is held over the whole vocabulary at
once by `words::tests::nothing_this_crate_says_names_a_rented_part`, rather than
file by file. The one thing a sentence does name is a language, in its own
language. The status-area square is the exception and is not a sentence: `DE`,
`GR`, the same in every language, for the reason `alo_shortcuts::Key::mark` is
not a string either.

## Acceptance, and the test for each

| The plan's acceptance | Workspace · crate · target · test |
|---|---|
| `alo-keyboards` offers, for each of the 24 languages, the layout people of that language actually type on — **a test per language names the layout** | `.` `alo-keyboards` `a_keyboard_for_every_language` `greek_is_typed_on_the_greek_keyboard` (and 23 more, one per language, named in that file) |
| …and **fails if a language has none**, so choosing Greek never means hunting | `.` `alo-keyboards` `a_keyboard_for_every_language` `every_one_of_the_twenty_four_languages_has_a_keyboard` |
| …and every keyboard offered is one this machine actually has | `.` `alo-keyboards` `a_keyboard_for_every_language` `every_keyboard_offered_beside_the_first_is_one_this_machine_has` |
| **refusal:** a keyboard the rented data does not name cannot be added | `.` `alo-keyboards` `a_keyboard_for_every_language` `a_keyboard_this_machine_does_not_have_cannot_be_added` |
| switching layouts is one shortcut and is shown in the status area | `.` `alo-keyboards` `switching_is_one_shortcut` `switching_is_one_shortcut_and_the_keyboard_in_use_is_shown` |
| …and the shortcut is a rented option this machine has, bound by nothing else | `.` `alo-keyboards` `switching_is_one_shortcut` `the_shortcut_is_a_rented_option_this_machine_has` |
| **refusal:** a person who has taken that chord is told what is on it, and their shortcut is not moved | `.` `alo-keyboards` `switching_is_one_shortcut` `a_person_who_took_the_chord_is_told_what_is_on_it` |
| dead keys produce `ü`, `è`, `ß`, `ł`, `ő`, `č`, `ġ`, `ħ`, **each typed in a test through the rented compose tables** | `.` `alo-keyboards` `dead_keys_and_compose_through_the_rented_tables` `every_letter_the_plan_names_is_typed_with_a_dead_key` |
| …and the compose key produces each of them, `ß` among them | `.` `alo-keyboards` `dead_keys_and_compose_through_the_rented_tables` `every_letter_the_plan_names_is_typed_with_the_compose_key` |
| …and the Irish letters | `.` `alo-keyboards` `dead_keys_and_compose_through_the_rented_tables` `the_irish_letters_are_typed` |
| …and the Maltese letters | `.` `alo-keyboards` `dead_keys_and_compose_through_the_rented_tables` `the_maltese_letters_are_typed` |
| **refusal:** a sequence the rented table does not have writes nothing — not a guess, not a table of ours | `.` `alo-keyboards` `dead_keys_and_compose_through_the_rented_tables` `a_sequence_the_rented_table_does_not_have_writes_nothing` |
| **refusal:** a table that is not one is refused whole, with the line named, in the person's own language | `.` `alo-keyboards` `dead_keys_and_compose_through_the_rented_tables` `a_table_that_is_not_one_is_refused_whole_and_said` |
| input methods for non-Latin scripts are a rented framework started with the session, and a person adds one without knowing its name | `.` `alo-keyboards` `an_input_method_is_added_without_knowing_its_name` `a_person_names_a_language_and_never_an_engine` |
| …and nothing is started on a machine that needs none | `.` `alo-keyboards` `an_input_method_is_added_without_knowing_its_name` `nothing_is_started_on_a_machine_that_needs_no_input_method` |
| **refusal:** a language typed on a keyboard is refused and sent to its keyboard, named in its own language | `.` `alo-keyboards` `an_input_method_is_added_without_knowing_its_name` `a_language_typed_on_a_keyboard_is_sent_to_its_keyboard` |
| layouts are kept by this crate in the person's folder (ADR 0038) | `.` `alo-keyboards` `keyboards_are_kept_in_the_persons_folder` `keyboards_are_kept_in_the_persons_own_folder_and_read_back` |
| …and only the difference is written | `.` `alo-keyboards` `keyboards_are_kept_in_the_persons_folder` `a_person_who_changed_nothing_writes_nothing_but_the_format` |
| **refusal:** a hand-edited file that is wrong is refused whole and named | `.` `alo-keyboards` `keyboards_are_kept_in_the_persons_folder` `a_file_that_is_wrong_is_refused_whole_and_said` |
| **refusal:** a file that did not read is not written over by the next change | `.` `alo-keyboards` `keyboards_are_kept_in_the_persons_folder` `a_file_that_did_not_read_is_kept_and_not_written_over` |
| **Constraint:** no layout or compose data is written here | `.` `alo-keyboards` `alo-keyboards` (lib) `offering::tests::every_offered_layout_is_a_layout`, with every name checked against the machine's own data by `a_keyboard_for_every_language` |
| every string is declared, and none names a rented part | `.` `alo-keyboards` `alo-keyboards` (lib) `words::tests::nothing_this_crate_says_names_a_rented_part` |
| the crate's words are collected into the machine's one vocabulary | `.` `alo-collected` `every_crate_that_declares_words_is_collected` `every_crate_that_declares_words_is_collected_or_named_apart` |

## Verification

Run from `C:\dev\alo-os-b` inside WSL Ubuntu (`CARGO_TARGET_DIR` this
checkout's own), 2026-09-18, all exit 0:

- `cargo fmt --all -- --check`
- `cargo clippy -p alo-keyboards -p alo-saying -p alo-collected --all-targets -- -D warnings`
- `cargo test -p alo-keyboards` — 62 + 27 + 5 + 8 + 5 + 6 unit and integration
  tests, 1 doctest, 0 failed
- `cargo test -p alo-saying` — 63 + 4 + 1, 0 failed
- `cargo test -p alo-collected` — 8 + 11, 0 failed
- `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-keyboards --no-deps`

The whole-workspace suite is the supervisor's, as `docs/autonomy/SHARED_MAIN.md`
sets out; it was not run here.

**Not verified, and it is the half that matters most for this task: nobody has
typed on a certified machine.** Every letter above was composed against the
rented table on this machine, by this crate's own state machine, with no
compositor, no `libinput` and no keyboard. That a German keyboard's dead key
sends `dead_diaeresis` at all, that `Alt+Space` reaches the keyboard
configuration rather than an application that grabbed it, and that an
input-method framework started with a session actually types Japanese into a
sandboxed application are all *on the machine* and stay untickable here.

## Limitations

- **The compose table read is the UTF-8 one.** `en_US.UTF-8/Compose` is the
  table every other locale's file includes, and it is what `Compose::read`
  opens. A locale-specific table that adds to it (there are a handful) is not
  followed, because no `include` directive is read — the reader refuses a line
  it cannot read rather than pretending to follow one. Enough for every letter
  the plan names; a later task that needs a locale's own additions adds
  `include` to `compose.rs` and a test per included file.
- **Layouts are decided, not applied.** Nothing here writes a keyboard
  configuration or tells a compositor about one. `Layout`, `switching::THE_OPTION`
  and `ComposeKey::option` are the three values such a writer needs, and it is
  the compositor's file to write.
- **A person cannot yet reorder their keyboards or choose the switch's chord.**
  Adding, removing and cycling are here; ordering is a settings surface's
  question and the chord is `alo-shortcuts`' the moment it has the action.

## Proposed shared-document updates

For the integration owner; not edited here.

- **`CHANGELOG.md`** — under v0.5: *Keyboards. Choosing a language now chooses
  the keyboard people of that language type on, for all 24 — a second one is
  added by naming a language, one shortcut switches between them, and the one in
  use is shown in the status area. Dead keys and a compose key type `ü`, `ß`,
  `ł`, `ő`, `ġ`, `ħ` and the rest through the tables the system ships rather than
  any table of alo OS's own, and a sequence those tables do not have types
  nothing. Chinese, Japanese and Korean are added by naming the language, and the
  software that types them runs only on machines where somebody asked for it.
  Keyboards are kept in the person's own folder, and a file that is wrong is
  refused whole with the line named.*
- **`ROADMAP.md`** — v0.5 *Input: … keyboard layouts with dead keys and a
  compose key, input methods*: the **code** half of the keyboard clause may be
  ticked, naming `crates/alo-keyboards`. **On the machine** stays untouched: see
  *Verification* above. Drag and drop, context menus, gestures and virtual
  desktops are the same line's other clauses and are tasks 3, 4 and 5.
- **`docs/autonomy/QUEUE.md`** — task 6 of the hands-on-the-desktop plan is
  done; task 7 (*every sentence, and the walk through a working morning*) now
  waits only on tasks 3 and 5, task 2 being blocked on the session plan.
- **`docs/autonomy/STATE.md`** — reference this report.
