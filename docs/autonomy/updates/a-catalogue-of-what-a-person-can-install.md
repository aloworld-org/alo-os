# A catalogue of what a person can install

**Date:** 2026-10-16
**Workstream:** third-PC lane — software a person works in
**Contributor:** Claude Code worker in `C:\dev\alo-os-3`
**Status:** ready for integration. The list and its reader are complete and
tested; **nothing reads it on a machine yet** — the market screen is the next
change, and *What was not built* says so plainly rather than leaving it implied.

## What changed, for a person

Nothing a person can see yet. What changed is that this product now has a
decided answer to a question it could not answer: **what does somebody work in
all day?**

A fresh alo OS machine has seven applications. It can browse the web, read a
document, look at a picture and open an archive. It cannot **make** any of
those things — there is no image editor, nothing that writes a document
somebody else can open, no video, no sound. A machine a person is meant to work
on all day had nothing on it to work in.

This change adds the list of what fills that gap, as data, with eight
applications under eight kinds of work:

| What a person makes | The application | Licence |
|---|---|---|
| pictures | GIMP | GPL-3.0+ AND LGPL-3.0+ |
| painting | Krita | GPL-3.0-only |
| illustration | Inkscape | GPL-2.0-or-later |
| photographs | darktable | GPL-3.0+ |
| page layout | Scribus | GPL-2.0+ |
| video | Kdenlive | GPL-3.0-only |
| audio | Audacity | GPL-3.0-only |
| documents | LibreOffice | MPL-2.0 |

Each is installed the way a person installs anything — from its source, through
the rented tool, arriving granted nothing, updated and removed like anything
else. There is no second road, and nothing knows an application came from this
list once it is installed.

## Where this work came from

The owner asked for it on 2026-10-16, having proposed a specific candidate:
`storytold/photocraft`, a Photoshop alternative written in pure Rust under MIT
or Apache-2.0, with local AI and no account. On paper an unusually good fit for
this product — our language, our licence posture, and *"smart selection runs on
your machine, with no cloud and no account"* is this product's own pitch coming
from somebody else.

It is not in this list, and what disqualified it shaped the file:

- **the repository was created 2026-09-30** — sixteen days before it was
  proposed. 65,243 stars and 2,518 open issues in sixteen days is a launch
  wave, not a track record;
- its own README says *early alpha… not yet a Photoshop replacement for daily
  professional work*, with fifteen tools missing;
- **it is not published to Flathub.** Flathub's search returns 44 results for
  `photocraft` and none of them is it; the control query (`dolphin` must return
  `org.kde.dolphin`) passed, so that is an answer rather than a broken probe.
  Its own Linux instruction is a downloaded `.flatpak` bundle — the
  trust-a-file-from-a-website path this product exists not to have.

So the file is built around the thing that was missing from the candidate: an
entry **must name a source, a release and a licence**, or it is refused. A name
is not an entry. Worth revisiting if it reaches Flathub published by its own
developer and leaves alpha.

Every version in the list was read from Flathub's own appstream answer on
2026-10-16, not from memory.

## What the reader refuses, and why each one is a person's problem

`Catalogue::read` refuses a list unless every entry names an identifier the
rented tool could be handed, a source by a name it could be handed, the
upstream release it was listed at and its licence — and four things more. Each
of these is a sentence a market screen would otherwise tell a person untruly:

- **nothing already shipped.** A catalogue naming Firefox would offer to
  install the browser the person is reading it in. This is the check that makes
  `shipped.toml` and `catalogue.toml` one story rather than two that drift.
  It is why `read` takes the shipped list as a **parameter**: a check only ever
  run against the one list that passes it is not a check that has been run, and
  there is a test that reads the same catalogue against a machine shipping GIMP
  and watches it refuse.
- **no application twice**, under one category or two — a list a person cannot
  count.
- **nothing a person's own** (`alo_capability::A_PERSONS_OWN`, ADR 0043). There
  is no terminal category; a terminal reaches a machine through the shipped
  list, which is where a reader looks for it. Nobody is stopped from installing
  one — `Wanted::by_hand` takes any identifier, and a curated list is not the
  only road.
- **no empty category** — a shelf a market screen would draw with nothing on
  it, which is a defect a person sees.

And **no key the reader does not know**, refused rather than ignored, for the
reason `shipped.toml` gives about itself: a key meaning *recommended* or
*cannot be removed* would be a promise nothing keeps.

## Two decisions worth recording

**There is no English in the data, and that is deliberate.** No name, no
summary, no description — not in the file and not in the type. The name and
summary of an application belong to its **source**, already translated by the
people who publish it, and are fetched when a person browses. An English
summary written into this repository would be a hardcoded string standing in
twenty-four languages' way, and it would go stale the week the upstream
reworded its own. What the list holds is only what the machine has to *decide*
with. The obligation that leaves for the screen is written in the module: a
category is drawn through `crate::words`, never by `Category::named`.

**Every category names a thing a person makes, never a product it replaces.**
These applications are the honest answers to *what do I use instead of
Photoshop*, and that sentence does not belong in this repository: a category
called `photoshop-alternative` would describe our list by another company's
catalogue, would age into a lie the day either program changed, and would tell
a person who had never used that program nothing at all. There is a test, not a
preference — `no_category_is_named_after_somebody_elses_program`.

The first spelling of the last category was `office`, and the test asserting
this had to carve out an exception for that one category's own name. **A rule
that has to argue with itself is not a rule**, so it became `documents`, which
also says where the catalogue picks up from the machine: a fresh machine
*reads* documents (`Role::DocumentViewer`) and cannot write one.

## One thing that was nearly copied

`is_a_release` and `is_a_licence` were private functions in `shipped.rs`, and
the new reader needed both. Copying them would have been two rules that can
disagree — a version the shipped list accepted and the catalogue refused would
be this repository contradicting itself in a way no test of either file alone
could see. They moved to `fields.rs`, and the tests there read **both** decided
lists and assert the count they checked, because a loop that found nothing
passes every assertion inside it.

## Scope

`docs/features.md` already had *"Install applications, sandboxed, from Flathub
or a repository the organisation runs; update and remove them"*, so the
mechanism was in scope. A **curated list** of what this product recommends was
written down nowhere, and the scope rule is that a feature nobody wrote down is
still not built. It is now recorded in the same section, as the owner's
decision of 2026-10-16, with the reason the shape is what it is.

## What was checked

- `cargo fmt --check`, `cargo clippy --workspace --all-targets -D warnings` and
  `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace` on the **final**
  tree, after the last hand edit, because a green check covers only the tree it
  read.
- `cargo test -p alo-software --lib`: **124 passed, 0 failed**, 21 of them new.
  The 21 were listed by name with `--list` rather than inferred from the total,
  because a passing count is not evidence that the thing added ran.
- `cargo test --workspace`.

## What was not built

**No screen, and nothing on a machine reads this list.** That is scope cut, not
depth cut: the list is complete, refused when wrong, and every refusal is
tested. What is missing is the market screen — browse, choose, install, with
the egress indicator visible throughout — and that is the change that finally
exercises launch item 7, *a person installs an application and the agent works
with it*, which has never been run on a booted machine.

**Nothing here has been installed on real hardware.** The eight identifiers are
what Flathub answered for on 2026-10-16; whether all eight install, start and
draw on an alo OS machine is unmeasured. That needs a boot, and it is the
measurement to take when the screen exists, because installing eight
applications by hand would prove the rented tool works and not that a person
can reach them.
