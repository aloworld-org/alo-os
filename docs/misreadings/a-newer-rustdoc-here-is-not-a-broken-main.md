# A newer rustdoc here is not a broken main

**Found 2026-10-30, the walking lane, in its own report to the owner.**

## What was concluded

> Main itself has this docs fault (a link that could mean two things, from #1020).

## What was true

- CI pins `nightly-2026-06-01` (*rustc 1.98.0-nightly*), and its rustdoc
  accepted the link. #1028's queue run on main passed the rustdoc gate.
- This machine's stable *rustc 1.98.1* refused it:
  ``[`crate::a_question_nobody_is_showing`]`` names both a module and a function.
- Main was green. Only the toolchain on this machine disagreed.

## The mechanism

A local gate that fails on a file the change did not touch reads as *main is
broken*, and `git log` names a plausible culprit. Nothing in the failure says
which compiler produced it. A green gate here is evidence about main; a red one
may be evidence about this machine.

## The cure

Before saying main is broken, read the toolchain line from main's last gate log
(`installed - rustc …`) and compare it with `rustc --version` where it failed.
If they differ, say *this toolchain refuses it*, not *main has it*. #1029 made
the link explicit anyway, so the next pin moving does not break it.
