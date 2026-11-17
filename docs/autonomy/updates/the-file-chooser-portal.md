# The file chooser portal

**Date:** 2026-11-17
**Workstream:** alo's own file chooser (the owner, 2026-11-17: *"Yes, build
it"*). This is change 2 of 4; change 1, the sheet, is #1360.
**Contributor:** Claude Code helper in `C:\dev\alo-file-chooser-portal`, for the owner
**Status:** **built, not walked.** Once this lands, an application on the
machine that asks the file chooser portal is shown alo's sheet. No booted
machine has done it yet.

## What it gives a person

A sandboxed application such as Text Editor asks to open or save a file. alo's
own sheet appears, the person chooses, and **the application is given that one
file and nothing else**, even when its sandbox reaches none of their folders.
Before this, `FileChooser` was not served, so each application drew its own
dialog.

## What was built

- **`alo-portals`** serves `org.freedesktop.portal.FileChooser` version 4. It
  answers `OpenFile` and `SaveFile`, and refuses `SaveFiles` and `directory` as
  *not offered yet*. The answer the desktop gives is held to what a choice can
  be, and anything else is recorded as *not believed*. The choice is recorded,
  then exported, then answered as `uris`.
- **`alo-portald`** asks the desktop at its door (`choose-file`) and hands the
  file over through `org.freedesktop.portal.Documents` (`AddFull` to open,
  `AddNamedFull` to save), as upstream's own chooser does.
- **The image** installs Fedora's `xdg-desktop-portal` for its documents portal
  only, and masks its frontend so `alo-portald` keeps
  `org.freedesktop.portal.Desktop`. The build starts the documents portal.
  [ADR 0159](../../decisions/0159-a-chosen-file-reaches-an-application-through-the-documents-portal.md)
  records the decision: the coordinator chose option A on 2026-11-17.
- **Contracts:** `docs/contracts/portals.md` and
  `docs/contracts/portal-answers-file.md`.

## Judgement

**This makes an existing promise true**: *Portals: file chooser and documents*
(`docs/features.md`), and ADR 0005's *a portal request must be answered by our
file chooser*.

## Tests, and a departure from law 6, stated

- **Law 6 was not kept for this change.** The portal code was written before
  its tests, because it was drafted while #1360 was in review. The tests did
  fail before they passed, but against the finished code, for these reasons:
  - a second reader's assertion about a reason with no name;
  - three standing tests that had not been told FileChooser is now answered.

  So none of them failed first against code that did not yet exist, and this
  report says so rather than letting the change read as test-first.
- **Law 7:** a second reader wrote
  `crates/alo-portals/tests/a_file_is_chosen_on_alos_own_sheet.rs` from the
  contract alone. Its finding 1 added the `not-believed` reason. Its findings 2
  to 4 tightened the contract's wording: a whole path with no `..`, the record
  keeps a path, and a file that is there may be saved over after the sheet
  asks.
- **Measured once, locally**, on a tree with #1360 merged in: every test of
  `alo-portals` and `alo-portald` passed, and so did clippy on both. CI runs
  them on every build from here on.
- **Law 8:** CI's mutation check runs on the PR.
- **Law 9 does not apply**, because nothing a model is told or offered changed.

## Limits, said here

- **Not walked.** A real `/run/user/<uid>/doc` mount and a real Flatpak opening
  an exported file have not been seen. That is change 4, the walk.
- **`filters`, `choices` and `current_filter` are not read**, so every file is
  shown. **A folder and several files are not offered.**
- **alo cannot answer the sheet yet.** That is change 3: alo choosing only
  inside the folders the person granted.

## Proposed for the integration owner

- **CHANGELOG.md:** "Applications now use alo's own file chooser. When an app
  asks you to open or save a file, alo shows one chooser, the same for every
  app, and the app is given only the file you chose."
