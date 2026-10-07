# The release carries no model, and a person brings their own

The owner, 2026-10-06: *"the release should not have models, the users will
download their own models or use our apis or work without models"*, and
*"users can download their own weights on the machine, use third party apis or
use our apis (future) or use without apis or model"*, and *"alo os when the
users install them with our apis that will work out of the box"*.

Written up as
[ADR 0095](../../decisions/0095-the-release-carries-no-model-and-a-person-brings-their-own.md).

## What a person gets

Four roads, and alo OS chooses none of them.

| road | when |
|---|---|
| weights they already have | now |
| a third party's API | now |
| **our own API** | **future**, and marked so |
| no model at all | now, and said honestly |

**Out of the box, a machine now answers nothing.** That is a real reduction and
it is not dressed up: *sovereignty is what a machine can do out of the box* was
true of a machine carrying 4.87 GiB of weights and is not true of this one. What
survives is the stronger half — the machine lets you run weights you own, and
the catalogue recommends without gating.

## Why the weights went

ADR 0025 put them on the image on purpose, and its argument was sound: a machine
that fetches at setup has not arrived ready when it is offline at setup. That
argument is not called wrong. What changed is which half of ADR 0025 governs —
**nobody chooses for the person** — and shipping one model was the one place
where we did. Not as a setting they could change, but as 4.87 GiB they received
whether it suited them or not, sized for the one 16 GB laptop the hardware
document certifies first, downloaded by everybody including the people who would
never run it.

## The store had to move, and this is the part that nearly shipped wrong

The model runtime was pointed at /usr/share/alo/models. That is **right** for
weights that arrive with the machine — they are ours, nobody is meant to change
them, and a read-only place is the correct place for that. It is **impossible**
for weights a person brings, because /usr is the read-only half of a bootc
machine.

Taking the weights out and leaving that line would have left every sentence
about bringing your own weights reading correctly while the thing itself could
not happen. The store is now /var/lib/alo-model/models, inside the service's own
state directory, which systemd makes before the process starts, owns, and keeps
at 0700. A new rule asks whether the store is somewhere the machine may write,
so moving it to a second read-only place fails rather than passing a comparison
against whatever was written down.

## About 1,300 lines of rules removed, because they could no longer fire

| | before | after |
|---|---|---|
| the weights reader | 822 lines | 150 |
| the module answering which model a machine arrives with | 217 lines | gone |
| the checker | 3,162 lines | 2,667 |
| the integration test | 692 lines | 562 |
| the recipe | 833 lines | 661 |

Every rule removed held the image **to carrying** a model: pinned by content,
checked before anything read it, pruned to the manifest that named it, matched
against the catalogue's recommendation for the certified laptop, and held to a
licence we could redistribute under. With nothing carried there is nothing to
pin, verify, prune or license. They were deleted rather than left standing
because this repository has already written down what an inert guard costs — a
check that cannot fail reads as diligence and gets quoted as evidence, and is
neither.

**Two rules replaced them.** The image carries no weights, caught by where a
copy lands rather than by a stage's name, so weights put back under a different
name or in the store's new place are caught too — 4.87 GiB returns in one COPY
line and nobody reviewing a recipe notices a layer getting bigger. And the store
is writable, above.

## Two records were annotated rather than rewritten

The disk measurement of 2026-09-12 stays exactly as taken, with a note that the
4.5 GB of model inside its 9.6 GB is no longer there. **Rewriting a measurement
to match a later decision is how a record stops being one.**

The 17 GB and 24 GB refusal floors are deliberately unchanged. The image is
smaller so both could be lower — but a refusal threshold is a decision about
somebody's only computer, neither number was ever only the image's size, and
lowering one to let more machines through is a thing to do on purpose with a
measurement behind it rather than as a side effect of another change.

## Five documents pointed at a file this change deleted

The citation check caught four and a fifth was found beside them. All five sit
in records of work that was done, so each keeps what it measured and gains a
sentence saying the module was removed and by what. The dead path stops being
backticked, because that checker's own rule is that prose naming a file is not a
pointer — and a file that no longer exists is not one.

The quirks entry needed the most care. **The quirk is still true** of the
runtime's library — it is why a grade is earned against an artefact rather than
a name — so it now reads as something a person's own weights meet rather than
something the image does.

## What Law 1 says about the road back

Answering out of the box returns later through our own API. That has a
consequence worth writing down before anybody is surprised by it:

> **answering that way means inference leaving the machine.**

Law 1 is not weakened by it — it is the case Law 1 was written for: visible at
the moment it happens and afterwards in a record, with the indicator a feature
rather than a diagnostic. But *with a local model a working day produces zero
inference egress* now belongs to the bring-your-own-weights road specifically,
and must not be printed beside a machine answering through our API. Law 1's own
wording was already conditional, so the constitution needed no edit.

## What is owed

**No surface.** The words exist and nothing draws the screen that offers the
four roads. And whether the runtime *serves* a file from outside its own store
has not been measured on a machine: the door measures the file and the grade
travels, but a booted machine answering from weights brought that way is not
shown by anything here. Until it is, bringing your own is mechanism rather than
experience.

## A consequence that is a benefit and was not the reason

Without the weights the image should be small enough for a hosted runner, so the
image workflow becomes usable and a release stops depending on whichever machine
has 70 GB of free disk. **That is unmeasured until it runs**, and it is written
in the ADR as a consequence rather than as the argument.

## Measured

323 tests in the image crate, all passing, including that the recipe this
repository ships lands no weights and that the store it points the runtime at is
one the machine can write. No image was built and no hardware was involved.
