# Regional formats and timezones per language

**Date:** 2026-09-17
**Workstream:** v0.5 — access and language
**Task:** *Regional formats and timezones per language*
(`docs/autonomy/v0-5-access-and-language-plan.md`, task 6)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written and tested on an **Apple M3 with 8 GB unified memory**,
macOS 26.5.2. Gates in Ubuntu 24.04 aarch64 under Lima, kernel 7.0.0-31-generic.
**Egress:** one — `cargo` fetched the `icu` crates (version 2, `compiled_data`)
from crates.io. Nothing else was downloaded and no CLDR data was edited.
**Status:** done.

## What changed, for somebody outside this repository

- **Dates, times and numbers are written the way the person reads them**, in all
  24 official languages, from CLDR — the same data every other serious system
  uses — rather than from a table somebody here typed.
- **The language and the region are separate choices.** A Portuguese speaker in
  Belgium reads Portuguese and writes dates the Belgian way, and changing one
  does not move the other.
- **The machine does not look up where it is.** The timezone is what the person
  chose; following the network is off until they turn it on, because it is a
  request that leaves the machine.
- **A date written by hand anywhere in this repository now fails a test.**

## What CLDR answers, and what it is asked

| | |
|---|---|
| a date | `17.09.2026` in German, `17/09/2026` in Portuguese |
| a time | the twelve-hour clock where that region uses one — CLDR's answer, not a switch |
| a date and a time together | one format, not two glued |
| a number | `1.234.567` in German, `1 234 567` in French |
| the first day of the week | Monday across Europe, and whatever CLDR says elsewhere |

A test asks all four of every one of the 24 — **Maltese and Irish held to the
same completeness as German**, which is this task's constraint — and another
asserts that German and English really differ, because a rented format that came
back identical everywhere would be a rented format that was not working.

## The two things CLDR has that this cannot reach

The task asks for six things. Four are above. The other two are in CLDR and are
**not** in the rented crate's stable surface, so they are written down as gaps
with an address rather than typed into a table of our own —
`alo_formats::what_cldr_has_that_this_cannot_reach`, held by a test that each
gap names the data and the price of reaching it:

| Wanted | CLDR keeps it in | Why it cannot be reached | What it would take |
|---|---|---|---|
| how money is written | `common/main/<language>.xml` — `currencyFormats`, `currencies` | currency formatting is in `icu_experimental`, which `icu` does not re-export; its API is unstable, and pinning it here pins it for every crate in this workspace | depend on `icu_experimental` and accept the breakage, or read CLDR's JSON at build time — an ADR's weight either way |
| which paper a printer gets | `common/supplemental/supplementalData.xml` — `measurementData`'s `paperSize` | `icu` exposes no measurement data at all | read that one file, or ask the print system, which knows the answer for the printer that is actually there — `alo-printing`'s question |

Neither was guessed at. A hand-written currency table would have been wrong first
in the languages nobody in this repository reads, which is the failure this whole
plan exists to prevent.

## Every date goes through this crate — **and what to do when this test stops you**

**If you are a lane that has just hit `no_crate_writes_a_date_by_hand`, this
section is for you.** The test reads the shipped source of *every* crate, so it
will find your code without you having heard of it.

**The question that decides it is who reads the date.**

- **A person reads it** — it is on a screen, in a sentence, in a panel. Build it
  with `alo_formats::Regionally`, which asks CLDR how that person's language
  writes it. A date built by hand here is English and American for everybody who
  is neither, and that is the whole reason this crate exists.
- **A machine reads it** — a record entry, a catalogue grade, a measurement, a
  file another program parses. That is a **wire format**, this rule does not
  apply, and the fix is to add your crate to `NOT_SHOWN_TO_A_PERSON` in that
  test **with the reason, in the same change**.

**Four crates are excluded already, each named with its reason in the test
itself:**

| Crate | Why it is excluded |
|---|---|
| `alo-record` | what happened on this machine, read back by a later reader and by another machine, which must agree about *when* |
| `alo-keeping` | the same, for what is kept |
| `alo-models` | a catalogue grade states the day it was earned, and two machines compare those days |
| `alo-driving` | a measurement states when it ran, for the same reason |

All four write ISO 8601 on purpose: nobody's regional format, and not meant to
be read as one. The failure message says all of this too, because a lane meets
the failure and not this report.

## How the test works


`tests/no_date_is_written_anywhere_else.rs` reads the shipped source of every
crate and fails on a date built by hand — `%Y-%m-%d`, `%d/%m/%Y` and four more.
It passes today.

Two things it deliberately does not read. **Test code**, because a test may write
`2026-09-17` and mean exactly that. And four crates that write a date **not shown
to a person**: `alo-record`, `alo-keeping`, `alo-models` and `alo-driving` write
ISO 8601 on purpose, so that two machines and a later reader agree about when
something happened. That is a wire format, not a regional one, and the exclusion
is named in the test rather than left as a hole somebody widens later.

## What renting CLDR costs

The `icu` crates with compiled data add **202 compiled artefacts** and about
**15 seconds** to an incremental build on this Mac. That cost lands on every lane
at every gate run, which is why it is measured and written here rather than
mentioned: this workspace rented eleven small crates before today, and this is
the twelfth and by far the largest. The alternative was a table of our own, which
the plan forbids and which would be wrong in Maltese within a year.

## What is not done here

- **Nothing draws.** What a date looks like on a screen is the shell's; this
  says what it says.
- **The person's folder is handed in**, as `alo-appearance`'s and `alo-access`'s
  keeping is: this crate does not know where a home folder lives.
- **No sentences.** This crate declares no words, so `alo-saying`'s list is
  untouched — there is nothing here for a person to read that is not a date.

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima VM
as root, run by the Mac lane's publish script on the tree combined with `main`.
