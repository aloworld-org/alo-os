# What each accessibility setting changes

**Date:** 2026-09-16
**Workstream:** v0.5 — access and language
**Task:** *What each accessibility setting changes*
(`docs/autonomy/v0-5-access-and-language-plan.md`, task 1)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written and tested on an **Apple M3 with 8 GB unified memory**,
macOS 26.5.2. Gates in Ubuntu 24.04 aarch64 under Lima, kernel 7.0.0-31-generic,
as root. Nothing was measured against a model or a screen: this task decides
values.
**Egress:** none.
**Status:** done.

## What changed, for somebody outside this repository

- **A machine now holds, in one place, what each accessibility setting is and
  what it changes** — nine of them: read the screen aloud, magnify, high
  contrast, larger text, reduced motion, sticky keys, slow keys, bounce keys,
  and always show where the keyboard is.
- **Each changes one value another part of the machine reads**, not a sentence
  somebody has to implement twice.
- **There is no accessibility mode.** Turning everything on is nine settings on,
  each still its own value. Nothing is switched off, and nobody is sent to a
  simpler machine.
- **Every one can be turned on before there is an account** — at the sign-in
  screen and during setup. A person who needs the screen read aloud cannot be
  asked to read the screen to turn it on.

## What each setting changes

| Setting | What it changes, as a value |
|---|---|
| read the screen aloud | the rented screen reader runs, on the tree the agent reads |
| magnify what is under the pointer | a magnification, between 1.2× and 20× |
| use strong colours | `HighContrast`'s palette, in place of the shell's |
| make all text larger | `alo_appearance::TextScale` at 150% |
| stop things sliding and fading | nothing animates |
| press keys one at a time | the keyboard's filter: modifiers stay held |
| ignore a key unless it is held | the keyboard's filter: a delay before a key counts |
| ignore the same key twice quickly | the keyboard's filter: a delay before it counts again |
| always show where the keyboard is | focus is drawn whether or not a key was pressed |

The three key settings are **one filter**, because a keyboard applies one. The
keyboard crate the desktop plan is building reads it; until then the values are
still decided here, which is better than a setting that means nothing until
somebody writes the other half.

## High contrast is a palette of its own, at AAA

Not the shell's palette with the contrast raised. The shell's six tokens are a
design, and their quietness is the point of them; a person who cannot read them
is not helped by a slightly darker cream. So there is a second palette, and it
is held by a test to **WCAG 2.2 AAA — 7 to 1 for text — on every pair the shell
draws**, in both light and dark:

| Drawn | On | Light | Dark |
|---|---|---|---|
| ink | a window's ground | 21.00 | 21.00 |
| ink | the dock | 17.94 | 18.42 |
| accent | a window's ground | 8.92 | 14.69 |
| accent | the dock | 7.62 | 12.88 |

The pairs are `alo-shell`'s four roles — ground, dock, ink, accent — read off
its desktop palette rather than a list invented here, so a role added there with
no pair here is a role nobody measured.

**Terracotta keeps its meaning and stops carrying it alone.** ADR 0010 gives the
agent one colour, and at AAA the shell's terracotta cannot be it: on cream it
reaches 3.09 to 1, under half of what text needs — a test asserts that, so if it
ever clears AAA this palette is re-argued rather than kept out of habit. The
accent here is **terracotta's own hue, 12°**, taken down to `#862A13` on a light
ground and up to `#F7CFC5` on a dark one, with the mark and the word beside it
carrying the meaning that the colour cannot carry for somebody who needs this
palette at all.

## Two copies of the settings, and why the second is not a duplicate

ADR 0038: each crate keeps its own settings in the person's folder, so this is
`access.toml` beside `appearance.toml`, written by this crate and nothing else.
It does not know where the folder is — it is handed the path.

The machine keeps a second copy, belonging to no account, which the sign-in
screen and setup read. **A person's own setting does not change what the machine
does at sign-in, and the machine's does not follow anybody home** — there is a
test for each direction. A broken settings file never stops the sign-in screen
from drawing: what could not be read is handed back beside the settings, for
whoever can say so, because a person locked out of their machine by a settings
file has no way to fix it.

## The words

Nine sentences, each saying what the setting *does* rather than naming a
feature — *read the screen aloud*, not *screen reader*, because somebody who has
never used one is looking for the first. Two tests hold them: every setting has
one and no two share one; and none names a disability or calls itself a mode.

## What is not done here, and whose it is

- **The words are collected, and this lane added its own line.** `alo-saying`'s
  list has no owning plan: every lane that adds a crate with words appends its
  own entry in the same commit, which is what *reads and never edits* means
  there — other crates' entries are theirs, this crate's line is this lane's.
  Sentences that exist and are not collected are the state that file exists to
  prevent, so `alo-access` is in it.

  **It takes three counts, not two.** `EVERY_LIST`, `ONE_STRING_EACH`, and the
  `each` array inside `the_machine_says_what_the_crates_say_between_them`, which
  sums every crate's own total. The first two are fixed-size arrays, so two
  lanes adding a crate on the same day each raise a count by one and the merged
  file is short by one — the fix on a rebase is to count the entries and set all
  three to that, never to take one side.
- **Nothing draws.** The magnifier, the focus ring and the high-contrast palette
  reach a screen through the shell plan's later tasks, from the values here.
- **The keyboard reads the filter when it exists**, which is the desktop plan's.

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima
VM as root, run by the Mac lane's publish script on the tree combined with
`main`. As of 2026-09-16 the tree is fully green — the aarch64 failure in
`alo-bounding` that every earlier report of this lane named is fixed — so the
script now publishes on all nine passing with nothing failing, and still accepts
the older eight-plus-one state for a machine that has not taken that fix.
