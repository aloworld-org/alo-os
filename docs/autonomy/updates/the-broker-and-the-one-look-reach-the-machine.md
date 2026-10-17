# The broker and the one look reach the machine

**Date:** 2026-10-17
**Lane:** Mac
**Branch:** `task/mac/the-broker-and-the-one-look-reach-the-machine`

## What changed, for a person outside this repository

Two things their machine could not do, it can now.

**It checks for updates by itself.** Before this, nothing on any alo OS machine
had ever looked. The checking was built; the one thing that would have looked
*at startup* was not installed. From where a person sits that is
indistinguishable from a machine with no updates — including missing security
fixes, with nothing to tell them.

**The agent can change the machine.** Setting up a printer, joining a network,
mounting a drive, applying an update. Ask for a printer before this and nothing
happened, because the component that carries out a change to the whole machine
was built and never started.

## How these were found

The owner asked whether people would have to delete the agent's undo snapshots
by hand. Checking that led to a daily sweep that was built and not installed,
and then to the same question asked of every crate: **eight unit files lived in
three crates and the image shipped none of them.** #737 made that visible and
named what each waits on; the owner chose to ship two of the three.

## What shipping meant

- both packages built, so four binaries exist to copy;
- `alo-brokerd`, `alo-applying-an-update`, `alo-going-back` and
  `alo-looking-once` copied to `/usr/libexec`;
- four unit files moved into `image/usr/lib/systemd/system/`;
- `alo-brokerd.service` and `alo-looking-once.service` enabled.

**Two units are deliberately not enabled.** *Apply an update* and *go back*
declare no `[Install]` because the broker starts them, which is how
`alo-convertd.service` is already handled.

`alo-brokerd`'s `[[bin]]` names one binary and the other two are
**auto-discovered from `src/bin/`** — the arrangement the Containerfile already
warns about for `alo-software`, where a package left out of the build made four
daemons simply absent and failed two hundred lines later with *no such file or
directory*.

## A unit lives in one place, and which guard watches it follows it there

A shipped unit lives in the image and the crate's own tests read **that** copy,
which is the shape `alo-sessiond` already had. Two copies of a unit is two units
free to disagree, and the one a machine runs would be the one no test read.

So the guards hand over cleanly:

- a unit **in a crate** is held by `every_unit_a_crate_keeps_reaches_the_machine`
  — shipped, or named as deliberately not;
- a unit **in the image** is held by `the_image_starts_what_it_ships` — enabled
  if it asks to be, and its `ExecStart` really copied there.

**Mutation-proved.** Deleting one `COPY` line makes the second say exactly what
is wrong: *alo-going-back.service starts /usr/libexec/alo-going-back, and
image/Containerfile never puts a file there* — which is the `alo-portald` fault
that test was written for, catching mine.

## Four things I got wrong, and what each cost

**`alo-looking-once` embeds its own unit at compile time.** `include_str!`, which
I had not looked for. Moving the file broke the build outright. Its doc said *the
installation is not this crate's* and described a lane that would one day
install it — and that day never came, which is precisely why no machine had ever
looked. The include now reads the shipped copy, and the doc says what changed
and why there is one copy rather than two.

**I generalised one crate's convention to another without checking.**
`alo-sessiond` reads its shipped unit, so I assumed every crate could. `alo-brokerd`
could and did; this one had its own arrangement.

**Pointing `here()` at the image broke three tests** that read the crate's own
*source* through it. `here()` and `shipped()` were always two different
questions and now have two names.

**I ran a gate's tests under `sudo` and poisoned the target directory.**
Root-owned fingerprints, and the next build as the person could not write its
own. My own notes warn about mixing root and non-root cargo in one cache.

## A control of mine was brittle, and this change is what showed it

#737 asserted the walk found **at least eight units**. Shipping four made it
fail on exactly the change it should have been indifferent to.

**A number you have to edit whenever the thing it measures legitimately moves is
not a control; it is a second copy of the answer.** It now counts the crates it
walked — which is what makes *found nothing* impossible — and each named unit is
separately held to existing by its own test, where that lower bound belongs.

## Measurements: where each one runs again

| what | where it runs again |
|---|---|
| every shipped unit that asks to be wanted is enabled | every build |
| every shipped unit's `ExecStart` has a file the image puts there | every build |
| the four remaining crate-kept units are named as unshipped | every build |
| `alo-brokerd`'s own tests read the shipped units | every build |
| `alo-looking-once`'s decision is the shipped unit, at compile time | every build |
| `alo-brokerd`'s 53 tests pass | **as root** — 18 of them need it |

**`alo-brokerd` has 18 tests that fail as the person and pass as root**:
*is not root's, so the broker did not write it*. Pre-existing, measured against
a branch touching no file of that crate, and CI runs the gates as root.

## What is still owed

`alo-letting-go`'s four units, still named as unshipped. The ordering constraint
stands and is worth repeating: **that sweep must not ship later than the
bracket-taking.** The first machine to take a snapshot with no sweep in place is
a disk that fills quietly.
