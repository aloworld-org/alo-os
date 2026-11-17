# The file chooser sheet

**Date:** 2026-11-17
**Workstream:** the file chooser the owner decided on 2026-11-17 (*"Yes, build
it"*): one chooser, the same for every application. Change 1 of 4.
**Contributor:** Claude Code helper in `C:\dev\alo-file-chooser`, for the owner
**Status:** **built and parked.** The sheet is drawn, keyed, read aloud and
answered at the desktop's door. **Nothing reaches it yet**: the portal service
does not ask it until it serves `org.freedesktop.portal.FileChooser`, which is
change 2. Until then an application's file dialog is the application's own, as
it was. Not walked on a booted machine.

## What it gives a person, once change 2 lands

An application that asks to open or save a file shows alo's own sheet: *Text
Editor asked to open a file*, the folder they are in, its folders and files, and
*Cancel* and *Open*. When saving, it also shows a name to save under, and
*Save*. They can go anywhere their own account can read, up to the root. A name
that is already there is not replaced on the first press. Every key works
without a pointer, and a screen reader hears every word drawn.

## Why it is two changes and not one

Registering `FileChooser` on the bus before the sheet existed would make GTK
applications stop using their own dialog and get a refusal instead, so opening a
file would get **worse**. So the sheet comes first, reachable by nothing, and the
portal comes next and makes it reachable. The coordinator agreed this order on
2026-11-17.

## What was built

- `alo-acting`'s `choose-file` act (`choosing_a_file.rs`): one line in, one
  answer out, with folders, names and paths percent-encoded so a space cannot
  split them. The contract is `docs/contracts/application-door.md`, *The person
  chooses a file*.
- A new crate, `alo-file-chooser`: the chooser as a decision, with no drawing.
  It covers the rows (the folder above, then folders, then files, with hidden
  names counted and not shown), every key, a click and a reader's press, when a
  name may be saved, and the warning before replacing. Whether a name is taken
  is asked of the disk, not of the rows shown, so a hidden file or one past the
  thousand shown is not replaced without the warning.
- `alo-shell`: the door hands `choose-file` over with its connection held
  (`the_application_door.rs`), and `the_file_chooser.rs` holds one sheet up and
  four waiting. The sheet is drawn (`the_file_chooser_drawn.rs`), takes its keys
  and presses (`a_key_reaches_the_file_chooser.rs`), and is read aloud
  (`the_file_chooser_as_read.rs`). Its 17 sentences each carry a note for
  translators (`the_file_choosers_words.rs`).
- `docs/design/the-file-chooser.md`: **the design is silent.** No frame in the
  snapshot draws a file chooser (checked by searching every layer name). **The
  live Figma file was not checked.** So the sheet follows *Open with*, and each
  reading is listed there.

## Judgement

**This makes an existing promise true**: `docs/features.md` lists *Portals: file
chooser and documents*, ADR 0005 says *a portal request must be answered by our
file chooser*, and the owner decided to build it on 2026-11-17.

## Tests, and who wrote them

- **Seen failing first (law 6).** The wire's tests were run against a planted
  wrong reader and encoder and failed 4 of 5 for that reason. The chooser's
  tests were run with its *is this name taken* check planted to answer *no*.
  Both runs were local, once, and each report is in this change's PR.
- **A second reader (law 7)** wrote
  `crates/alo-acting/tests/choose_file_from_the_contract.rs` and
  `crates/alo-file-chooser/tests/the_file_chooser_from_its_promise.rs` from the
  design note and the contract alone, without reading the code. Their first
  finding was real and changed the code: the chooser could not see a hidden file
  it was about to replace. `Folders::holds` was added for it. The author added
  that one method to the reader's test double, and nothing else in their files.
- **Mutation (law 8)** runs in CI on this change. What it finds is reported on
  the PR.
- **Law 9 does not apply**: nothing a model is told or offered changed.

## Limits, said here rather than discovered

- **Nothing reaches it** until change 2.
- **Home and End** move to the ends in the chooser, but the seat's key reading
  (`SignInKey`) has no Home or End, so on a real keyboard they do nothing.
- **The name in the reader's tree is its description**, not an editable text
  interface. A reader hears it; typing goes through the seat as for any key.
- **A folder on a slow network mount is listed on the desktop's own thread**,
  so it could stall a frame while it answers.
- **No filters** (*only pictures*) and no choosing of a folder or of several
  files. Those are the portal's options, and change 2 says which it answers.

## Proposed for the integration owner

- **CHANGELOG.md:** "alo OS has its own file chooser, the same for every
  application: browse your folders, open a file or save under a name, with the
  keyboard or a screen reader. Built, not yet shown: applications are given it
  in the next change."
