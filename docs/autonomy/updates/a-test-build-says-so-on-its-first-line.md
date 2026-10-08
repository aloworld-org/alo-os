# A test build says so on its first line

[ADR 0096](../../decisions/0096-a-workflow-may-publish-a-candidate-and-a-candidate-announces-itself.md)
control five, which that ADR assigns to this lane. The last thing standing
between the installer chain and a candidate build on real hardware.

## What a person sees

A test build, before anything else on screen:

> This is a test build of alo OS from 2026-10-07. It is not a release

A release says nothing about it. So does every local build, and every test
binary — nothing compiled in is the ordinary case, and the line appears only
where it is true.

## Why one line on screen rather than anywhere else

Every other control a candidate carries lives on the page it came from.
**Deleting a pre-release removes that page and not the file on somebody's
disk.** So a person who downloaded a candidate, kept it, and double-clicks it
next month has no page to consult and no network.

A filename does not reach them — nobody reads a filename, and a zip gets
extracted. The release notes do not reach them — that is the page they left.
One line on screen does, and **the date in it is what lets a kept build say how
old it is with nothing to look up.**

ADR 0046's own first sentence is what is at stake: *the program that
repartitions somebody's only computer.* A person who mistakes a candidate for a
release does not have a bad afternoon.

## It informs and does not refuse

A test build that stopped working after some number of days could bite
mid-walk, and that failure would look exactly like the loader failure this whole
road exists to eliminate — a program that prints nothing and does not start. It
would also be the installer deciding for the person, which the fifth law
forbids: *every protection is a default they can change, not a wall.*

Telling somebody plainly what they are holding restricts nothing. Refusing to
run is a wall.

## The warning survives a broken date

There are **two** sentences, not one:

| | |
|---|---|
| `installer.a-test-build-from` | *This is a test build of alo OS from {day}. It is not a release* |
| `installer.a-test-build` | *This is a test build of alo OS. It is not a release* |

A value the program cannot read as a day — empty, a timestamp, a month that is
not a month, a date in somebody else's order — loses the day and **keeps the
warning**. *It is a test build* is the half that protects somebody; the day is
the improvement.

One sentence with a bad gap would do one of two worse things: put whatever was
compiled in front of a person, or be suppressed and take the warning with it.
Eight malformed values are tested, and each is asserted still to say *not a
release*.

This is the same shape as the Fast Startup reading's own control an hour
earlier: **an instrument that cannot report its own failure reports the
safe-looking answer**, so the failure gets its own road rather than a fallback
inside the happy one.

## Both directions are tested, and the release one against a real build

`ALO_INSTALLER_CANDIDATE_BUILT` is compiled in, so a test binary cannot vary it.
The reading is therefore split the way `FastStartup::read` is: a pure
`what_was_built(Option<&str>)` the tests drive with a day, with a value that is
not a day, and with nothing; and a thin `this_build()` that hands it the
compiled-in value.

**And the release direction is held against a build of the real shape.** A test
binary is not built by the candidate workflow, so this binary is release-shaped,
and `a_release_says_its_own_name_first_and_nothing_about_test_builds` asserts
what a release says *first* — plus that no line anywhere in a run calls itself a
test build, because a warning in the middle would be as wrong and the first
assertion would not catch it.

A flag a test set would prove the branch runs. This proves the branch is **not
taken** on the thing people download.

## The date is written as the machine wrote it

ADR 0096 illustrates the sentence with *7 October 2026*. The date is filled in
as `2026-10-07`, and that is not a deviation: `words.rs` already **instructs
translators** to use this form, in its note on the one existing sentence that
shows a person a date —

> `{newest}` a date as year-month-day, for example 2026-09-28.

Month names in prose would contradict a written instruction one file away, and
would mean translating twelve month names into twenty-four languages for one
sentence. *In the person's own language* governs the sentence; the date is a
filled gap. `Day` already validates the shape and already refuses what is not a
date, so it is reused rather than a second validator written.

The Mac lane found that translator's note while checking the reasoning, and it
is stronger than the three arguments it replaced.

## Also in here

`installer.starting`'s own documentation called itself *The first line*, which
this change makes untrue. It now says it is the first line on a release and
names what comes above it on a test build. A sentence that describes its own
position is a caller of that position.

## What this unblocks

The owner authorised the candidate dispatch on 2026-10-08, to the Mac lane
directly, and **made this change a condition of it**: *"I do not want a build in
public that does not say it is a test build."*

So with this in `main` the dispatch can happen, the testing NUC downloads a
candidate from the page as a person would, and task 4's first metal run — a
failure injected before the point of no return, with Windows still starting
afterwards — becomes possible without spending a release per attempt, which is
what ADR 0096 was for.

One thing to watch on that dispatch, which nobody has evidence about yet: the
workflow's **delete** step has never run. Both earlier dispatches found no
previous candidate, so the fallback was exercised and the deletion was not.
There is a `candidate-*` pre-release on the page now, so this is the first
dispatch where it can happen.
