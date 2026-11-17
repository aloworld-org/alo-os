# alo answers the file chooser

**Date:** 2026-11-17
**Workstream:** alo's own file chooser, change 3 of 4. The coordinator approved
the verb that day: *a dedicated verb judged against the path grants exactly as
the file verbs are, not by pressing rows. One card at "ask each time", none at
full access.*
**Contributor:** Claude Code helper in `C:\dev\alo-fc-verb`, for the owner
**Status:** **built, not walked.**

## What it gives a person

Say an application's file chooser is up, and the person asks alo *choose
notes.txt from my Documents*. At *ask each time* they approve one sentence,
*choose notes.txt in /var/home/alo/Documents on the file chooser*; at full
access there is no card. The sheet is then answered exactly as their own press
of *Open* or *Save* would answer it.

- **alo may answer only with a file in a folder the person granted**, or, at
  full access, any folder their account can open.
- **A link out of a granted folder is refused**, in the file verbs' own words.
- **The person's own use of the sheet is never narrowed**: the grants decide
  only what alo may answer for them.

## What was built

- **The verb** `choose_on_the_file_chooser(folder, name)`, a change.
  - It is declared in `alo-applications`, with its card sentences and its
    record words.
  - It takes a folder and one name, not a path, because a file chosen to save
    under does not exist yet. `archive_folder` takes its place the same way.
  - The verb helper (aa88f60e0d4b6d554) was told the name and arguments, and
    confirmed nothing collides.
- **The turn** (`alo-turn`'s `choosing_on_the_file_chooser`) does four things
  in order:
  1. Asks the file verbs' three questions about the folder.
  2. Asks where an existing file really leads.
  3. Writes the door's line with the folder's real path.
  4. Words every answer. Every refusal is recorded, and a paired machine is
     refused by name.
- **The door**: the `alo-chooses-file` act in `alo-acting`, the daemon's side in
  `alo-agentd`, and the desktop's side in `alo-shell`
  (`alo_answers_the_file_chooser.rs`).
  - The desktop answers `no-sheet`, or `not-believed` (it uses
    `alo_file_chooser::Chooser::alo_chose`), or answers the sheet as the person
    would.
  - The desktop judges no grant.
- **Contracts and docs**: `agent-verbs.md`, `application-door.md`,
  `daemon-protocol.md`, `docs/by-hand.md`, and a line in `docs/features.md`.
- **Law 9**: a request in the every-try fixed set (version 13). The workflow now
  runs when this verb's files change.

## Judgement

**This makes an existing promise true**: the owner's decision of 2026-11-17
promised that alo can answer the file chooser inside granted folders. The
`docs/features.md` line records it in the owner's and the coordinator's words.

## Tests

- **Seen failing first (law 6)**, locally, once, each against a planted break:
  - the wire: 2 of 3 tests failed against a planted reader;
  - the chooser's answer: it failed when planted to believe anything;
  - the turn: the link test should fail with the link check planted away (see
    the PR for the run).
- **Every refusal goes through a whole turn**:
  - a folder nobody granted;
  - a link out of a granted folder;
  - a folder that is not there;
  - no sheet up;
  - a file the sheet would not take;
  - no desktop;
  - a declined choice.

  Also tested: full access with no folder granted, and one approval leading to
  exactly one door ask.
- **Law 7, the second reader:** not done for this change, and said so here.
- **Law 9** runs in CI (`every-try.yml`, ten tries at both levels by dispatch).
  Its result is on the PR.

## Limits

- **alo learns that a sheet is up only from the person's words.** Nothing tells
  a model "a file chooser is open" unasked. A request that does not mention it
  is answered `no-sheet`, which is recorded and said.
- **Not walked**: change 4.
