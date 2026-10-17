# A unit a crate keeps is shipped, or named as one that is not

**Date:** 2026-10-17
**Lane:** Mac
**Branch:** `task/mac/a-unit-a-crate-keeps-is-shipped-or-named`

## What changed, for a person outside this repository

Nothing they can see today. What changed is that three components which exist
and never run on a machine are now **written down as such**, where somebody
deciding what to ship will read it — instead of being invisible.

## How this was found

The owner asked whether people would have to delete the agent's undo snapshots
by hand. They would not: ADR 0045 gives that to a daily timer, with a window the
person sets, a rule that never fills a disk, and one act that forgets
everything. All of it is built.

**And none of it reaches a machine.** The image does not install the timer, or
the service it starts, or the two halves of the forgetting act, and does not
build or copy either binary. So a machine built today would keep every snapshot
for ever — not manual deletion, but a disk filling quietly, which is worse
because nobody notices until it is full.

Then the same question, asked of every crate: **eight unit files live in three
crates and the image ships none of them.**

| crate | units | shipped |
|---|---|---|
| `alo-letting-go` | service, timer, forgetting service, forgetting path | none |
| `alo-brokerd` | service, applying-an-update, going-back | none |
| `alo-looking-once` | service | none |

Nor are their binaries built or copied. Three whole components are built and
never start.

## Why the test that should have caught this could not

`alo-image/tests/the_image_starts_what_it_ships.rs` exists for exactly this
family, and its own header records two units reaching `main` that nothing
started — `alo-compositor.service`, which left an installed machine booting to
no screen, and `alo-portald.service`, whose `ExecStart` named a file the image
never created.

It asks, of every unit **in `image/`**, whether anything starts it and whether
its `ExecStart` has a file. **A unit a crate keeps and the image never copies is
not in `image/` to be asked about.** The fault is in the other direction and
nothing looked that way.

## What this adds, and what it deliberately does not

It does **not** ship the three components. Shipping a privileged service changes
what runs on somebody's machine, and the consistent absence of all three may be
deliberate staging. That is a decision, and a lane that merely noticed is not
the one to make it.

What it adds is the complementary guard: every unit a crate keeps is either
shipped by the image, or **named in `NOT_YET_SHIPPED` with what it is waiting
on**. All eight are named, so it passes today.

**What that buys is that the set cannot grow in silence.** A ninth unit written
in a crate and not shipped fails here until a person says which it is. An
absence nobody listed is what cost two releases in the other direction.

## Four tests, because a list rots in four ways

| what | what it catches |
|---|---|
| every unit is shipped or named | a ninth absence nobody listed |
| nothing named as unshipped is shipping after all | an excuse that stopped being true |
| every name still matches a file a crate keeps | a line about nothing |
| each one says what it waits on | a name nobody can act on |

Two controls, because a walk that read nothing would be green about a
repository it never opened: the walk must find at least the eight that were
there, and the shipped directory must hold at least the twelve units it held.

**Mutation-proved.** A unit planted beside `alo-capability` and not shipped
makes the first test fail and name it: `alo-capability keeps
alo-made-up.service`.

## The decision queue this hands over

Each line in `NOT_YET_SHIPPED` is a question for whoever owns the release:

1. **`alo-letting-go`'s four.** Its window and the person's screen for it both
   ship; the half that does the letting go does not. Ship it now, when nothing
   yet takes a bracket, or with the bracket-taking it exists for? **It must not
   ship later**, because the first machine to take a snapshot with no sweep in
   place is a disk that fills.
2. **`alo-brokerd`'s three.** Is the broker meant to ship in this release?
3. **`alo-looking-once`'s one.** Same question.

## What is still owed

The three decisions above, and then the shipping itself — the recipe needs the
packages built, the binaries copied to `/usr/libexec`, the units copied, and the
timer and the path added to `systemctl enable`. None of that is in this change.
