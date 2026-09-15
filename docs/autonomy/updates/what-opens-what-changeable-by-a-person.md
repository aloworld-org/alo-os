# What opens what, changeable by a person

**Date:** 2026-09-15
**Workstream:** v0.5 applications, and what they expect —
`docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`, task 4
**Contributor:** Claude (Mac lane worker), for the repository owner
**Status:** ready for integration

## What changed

alo OS can now answer *which application opens this file* — and say why. The
answer comes from what the file really is, read from its bytes, and from the
application the person chose for that kind of file; where they chose nothing,
it is the application that says it opens that kind. A person's choices are kept
in their own settings folder, and a kind of file nothing opens gets a plain
sentence instead of being thrown at a text editor. An application asking the
open-with portal is answered from this and nothing else.

- **`crates/alo-applications`**
  - `what_opens.rs` — `WhatOpensWhat::on(&Installed, &Declared, &Chosen)`;
    `what_opens(file)` reads the kind with `alo_opening::decide` (no name is
    handed over) and `what_opens_a(kind)` answers in order: the person's choice
    if installed, the first installed declaring application, or nothing.
  - `opener.rs` — `Opener` (application, kind, `Because`) and its sentence:
    `ThePersonChoseIt`, `ItDeclaresIt`, `ItDeclaresItAndTheChoiceIsNotInstalled`.
  - `nothing_opens.rs` — `NothingOpens::NoApplication { kind, chosen }`,
    `TheFile(alo_opening::Cannot)` for a program, empty, damaged,
    password-protected or unrecognised file, and `Unreadable`.
  - `chosen.rs` — `Chosen`, the person's choices; `choose`, `forget`,
    `for_kind`, `all`. Its file shape refuses an unknown kind or an identifier
    that is not one.
  - `declared.rs` — `Declared`, what applications declare, in arrival order;
    `declares` and `declares_media_types`.
  - `media_types.rs` — desktop-entry media types to kinds (`text/plain` is both
    kinds of plain text).
  - `spelled.rs` — each kind's name in the file (`pdf`, `word-document`, …).
  - `keeping.rs`, `unkept.rs` — `what-opens-what.toml` under `alo-kept`'s rule
    (ADR 0038): `THE_FILE`, `FORMAT = 1`, `read`, `keep`,
    `put_back_as_shipped`, `at_sign_in`, and the eight `applications.kept.*`
    sentences, each naming the file.
  - `words.rs` — five `applications.opens.*` sentences and the eight kept ones,
    each with a translator's note.
- **`crates/alo-portals`** — `open_with.rs`: `answered(from, path, file,
  &WhatOpensWhat, &Grants, now) -> Result<OpensWith, NotOpened>`.
- **`docs/contracts/person-settings.md`** — the folder's file table gains
  `what-opens-what.toml`, and the file has its own section (keys, format, what is
  written, values, missing file, refused examples, writing).

## Decisions

- **The kind comes from `alo-opening`, not `alo-finding`.** The plan says *the
  way `alo-finding` reads it* — from the bytes, never the extension — and both
  crates do. `alo-finding` calls every Office document a `zip`, so an
  association on its kind would open spreadsheets in a word processor.
  `alo-opening` reads a zip's contents list and tells a Word document from an
  Excel workbook. Both crates are left alone; `alo-applications` depends on
  `alo-opening` and calls its public `decide` with an empty machine and an empty
  name. So the answer is always what the file is, never what this machine can
  do with it, and no name gets a say.
- **The first application to declare a kind answers, not the latest.** This is
  `Installed`'s rule again. If the latest declaration won, installing something
  would be enough to make it what opens every PDF — an application setting an
  association for itself, which *changeable by a person* rules out.
- **A choice of an uninstalled application is kept and reported, not dropped.**
  The declared application answers, and the answer names what the person chose
  and says it is no longer installed. Reinstalling brings the choice back.
- **The file is keyed by alo OS's kind names, not media types.** `text/plain`
  covers two kinds, and a key that means two kinds would make a choice about one
  of them for the other as well. Media types are only read from declarations.
- **A person may choose any installed application for any kind**, even one that
  does not declare it. The choice wins over every declaration.
- **Open-with is judged in this order: the request, the file's grant, the
  kind, the opener's grant.** The file is read only after its grant has allowed
  it, so a refusal never tells an application what kind of file sits at a path
  it was never granted. The test counts reads and finds none. The opener must
  also be granted to the asking application: that is ADR 0040's row, *the file,
  and the application to open it*, and a person's choice does not grant it.
- **"Only a person changes it" is enforced by what exists, and tested.**
  Nothing that answers holds a `&mut Chosen`. Declarations go into `Declared` and
  never into `Chosen`. There are still exactly four application verbs. A source
  test checks that `alo-portals` never calls a mutator, `keeping::`,
  `Command::` or `std::process`, and names the opener to the grants only from
  `WhatOpensWhat`'s answer.
- **Not added:** a row in the contract's *A Settings surface* table. That table
  is held, call for call, by `alo-choosing`'s
  `tests/one_persons_folder_from_sign_in_to_the_next_change.rs`, which this plan
  may not edit. The file's own section names every call a surface needs.

## Acceptance

| Criterion | Test |
|---|---|
| The kind comes from the file's bytes, never its extension | `alo-applications` `what_opens_what::the_kind_is_read_from_the_files_bytes_and_never_from_its_name` |
| Answered from the person's choice or the application's declaration, saying which | `alo-applications` `what_opens_what::the_answer_says_whether_the_person_chose_it_or_the_application_declared_it` |
| A person's choice is written to their settings, read back, and wins over any declaration | `alo-applications` `what_opens_what::a_persons_choice_is_written_read_back_and_wins_over_every_declaration` |
| A kind nothing opens is a sentence in the vocabulary, not a text editor | `alo-applications` `what_opens_what::a_kind_nothing_opens_is_a_sentence_and_never_a_text_editor` |
| Open-with is answered from this and nowhere else | `alo-portals` `open_with_is_answered_from_what_opens_what::an_open_with_request_is_answered_from_what_opens_what_and_nowhere_else` |
| Refusal: file not granted, refused before it is read | `alo-portals` `open_with_is_answered_from_what_opens_what::a_request_not_granted_the_file_is_refused_before_the_file_is_read` |
| Refusal: the opener is not granted | `alo-portals` `open_with_is_answered_from_what_opens_what::the_application_that_opens_it_must_be_granted_too` |
| Constraint: no association is set by an application | `alo-applications` `what_opens_what::no_application_sets_what_opens_what_on_its_own_behalf`; `alo-portals` `open_with_is_answered_from_what_opens_what::nothing_in_the_portal_sets_an_association_or_picks_an_opener_itself` |
| A broken file is refused whole and not written over | `alo-applications` `what_opens_what::a_file_of_choices_that_does_not_read_is_refused_whole_and_never_written_over` |
| The contract describes the file | `alo-applications` `the_contract_describes_this_file` (six tests) |

## Verification

Run on the `alo` Lima VM (aarch64 Linux) from this checkout, with
`CARGO_TARGET_DIR=$HOME/alo-builds/alo-os-main`:

- `cargo fmt --all` — clean.
- `cargo clippy -p alo-applications -p alo-portals --all-targets -- -D warnings`
  — exit 0.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p alo-applications -p alo-portals`
  — exit 0.
- `cargo test -p alo-applications -p alo-portals` — all pass: 60 unit tests in
  `alo-applications`, and in its integration tests 6 in `what_opens_what`, 6 in
  `the_contract_describes_this_file`, 6 in `from_a_call_to_a_window` and 7 in
  `what_this_crate_says`; 10 unit tests in `alo-portals`, 9 in
  `a_portal_request_is_a_grant` and 5 in
  `open_with_is_answered_from_what_opens_what`; doctests pass.
- `cargo test -p alo-saying` — passes (the machine's one vocabulary still
  collects).
- `cargo check --tests -p alo-granted -p alo-secrets -p alo-driving -p alo-by-hand -p alo-instructing -p alo-choosing`
  — exit 0.

Not run: the full workspace suite (the supervisor runs it). No hardware
acceptance applies, because nothing here touches the machine: no process is
started, and the only disk access is the person's own settings file in a
temporary folder.

## Limitations

- **Nothing reads desktop entries yet.** `Declared::declares_media_types` takes
  what an entry lists, but walking `/usr/share/applications` and the sandbox's
  exports is the acting half's job (task 5, or whatever builds `Installed`).
- **The kinds are `alo-opening`'s eighteen.** HTML, audio, video and other
  kinds that code does not recognise are opened by nothing until that crate
  learns them. Each new kind is then a compile error in `spelled.rs` and a row
  in `media_types.rs`.
- **Open-with serves nothing over D-Bus yet.** Task 5 serves `OpenURI` from
  `open_with::answered` and must check that the file handle it was passed is the
  path it asks about.
- **No Settings surface draws the choices.** A shell reads them with
  `keeping::at_sign_in` and changes them with `Chosen::choose` and
  `keeping::keep`.

## Proposed updates to shared documents

- **CHANGELOG.md:** "alo OS answers which application opens a file from what
  the file really is, from the application you chose for that kind or, where
  you chose none, the one that says it opens it — and tells you which. Your
  choices are kept in `what-opens-what.toml` in your own settings folder. A kind
  nothing opens is said plainly rather than handed to a text editor, and the
  open-with portal answers applications from the same choices."
- **QUEUE.md / STATE.md:** v0.5 applications plan task 4 done; task 5 (the
  portal backend over D-Bus) is next, and every task it depends on is done.
- **ROADMAP.md:** none.
- **Follow-up for the owner of `alo-choosing`:** add
  `alo_applications::keeping` to the *A Settings surface* table and to
  `one_persons_folder_from_sign_in_to_the_next_change.rs` together, when a
  Settings section for what opens what is built.
