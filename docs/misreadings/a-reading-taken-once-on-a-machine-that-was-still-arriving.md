# A reading taken once, on a machine that was still arriving

**Invisible until a boot**: every test of it was green, and it was found by a
person on a booted machine finding no application at all (2026-10-31).

## What was concluded

`crates/alo-desktop/src/what_this_machine_offers.rs`, until this entry:
*"This runs once, before the desktop stands up, and answers. There is no
partial state to report."* And `alo_dock::pinning`'s owner clause was honoured
to the letter: a failed or unfinished scan is never written down.

## What was true

The scan was neither failed nor unfinished. It **succeeded**, at 669 s, on a
machine where `alo-shipping` had installed nothing yet — Firefox arrived at
1429 s. `flatpak list` truthfully said *nothing*, the reading became
`Completed([])`, and a Dock of Settings and the market was written into the
person's `dock.toml`, where `ThePins::Chosen` will keep it on every later boot.
The log said so in as many words: *the Dock's default roles resolve to
Completed([])*.

## The mechanism

**"Finished" was asked of the scan, when the question was whether the
machine had finished.** A reading is complete about the moment it was taken;
whether it is complete about the machine depends on what else is still writing
to it. The owner's clause — *an unfinished scan is not proof that an
application is absent* — was implemented at the level it was written, the scan,
and the machine one level up was still arriving. Every test handed the producer
a finished machine, because a fixture is a finished machine by construction.

## The cure

When a reading is kept, ask **what is still changing the thing it reads**, and
make that a state of the answer rather than a comment. Here it is
`Discovery::SoFar` until `alo_software::the_applications_arrived` is marked,
and the reading is taken again when the directories change. A fixture for such
a reader includes the moment *before* the writer finished.

**A machine that booted an image before this fix keeps its wrong `dock.toml`.**
The fix stops new ones being written; it does not repair one, because a
`Chosen` list is the person's and cannot be told apart from one they made.
