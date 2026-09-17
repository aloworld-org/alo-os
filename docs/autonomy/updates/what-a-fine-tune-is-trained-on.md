# What a fine-tune is trained on, and what it may never reach

**Date:** 2026-09-17
**Workstream:** v0.5 — models a person adapts, and the one they subscribe to
**Task:** *What a fine-tune is trained on, and what it may never reach*
(`docs/autonomy/v0-5-models-a-person-adapts-and-subscribes-to-plan.md`, task 1)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written and tested on an **Apple M3 with 8 GB unified memory**,
macOS 26.5.2. Gates in Ubuntu 24.04 aarch64 under Lima, kernel 7.0.0-31-generic.
No model was run: this task decides the input, and the training is task 2's.
**Egress:** none.
**Status:** done.

## What changed, for somebody outside this repository

- **A fine-tune trains on folders a person granted, and nothing wider.** Not a
  typed path, not *all my documents* — the grant they made in the picker.
- **They see the list before anything trains**, and every file that will *not*
  be used is named with its reason. Nothing is skipped silently, because a
  person would otherwise believe their model had read something it never saw.
- **The dataset is written in one working folder, which is removed when the
  fine-tune ends** unless they kept it.
- **This crate cannot send anything anywhere**, and two tests keep it that way.
- **What a model learned from is written down**, so *what has my model learned
  from?* can be answered a year later.
- **[ADR 0047](../../decisions/0047-a-revoked-grant-stops-the-next-fine-tune-and-does-not-unlearn-the-last.md)
  says plainly what revoking the grant afterwards does, and does not do.**

## An adapted model carries the person's documents

This is the decision the rest of the task hangs from, and it is written into the
crate rather than left as a caution.

A fine-tune over somebody's correspondence produces weights the sentences can be
drawn back out of. So **sending an adapted model is sending those documents**,
and calling it *uploading a model* names the file while hiding the act — a person
who would never send their correspondence to a service may well agree to send
"their model".

Therefore:

1. **This crate has no road to the network.** No client, no socket, no door.
   `tests/nothing_here_can_send_anything.rs` reads its own source, and its
   manifest, and fails on any of twelve roads to one.
2. **If a later task sends one, it is an `alo_egress` departure** with the
   indicator lit and a record written, and the sentence a person reads says
   *your documents*, not *your model*.
3. **alo's own service is not an exception** (ADR 0014): our address is not
   privileged, a policy refusing hosted inference refuses ours, and a
   subscription does not make somebody's correspondence less theirs.

**The errand for it does not exist, and that is the finding.**
`alo_egress::Errand` is a closed list — signing in, fetching a model, checking
for an update, three about applications — and none of them is *sending what a
model learned from this person's documents*. This plan may not edit that crate,
so `alo_adapting::leaving` states what that variant would have to say, including
that its indicator should be at least as loud as the one for a model coming
*down*, since this one carries the person's own writing. Whoever adds it will
find that text before they add it.

## Snapshot, not folder — and why

**A fine-tune trains on the list of files taken when it started**, not on
whatever the folder holds while it runs. A file that arrives mid-training is not
learned; one that is deleted is not reached.

The reason is the list the person approved. They were shown *these documents*;
a fine-tune that followed the folder would train on whatever landed afterwards —
a mail that arrived at the wrong moment, a file somebody else dropped into a
shared folder. **A person cannot approve a list that is still changing.**
`Dataset::will_learn_from` is the one road every reader takes, so this is a fact
about the type rather than a rule somebody remembers.

## What a model learned from, written down

`alo_adapting::LearnedFrom` carries the folders, the grantee the grant was made
to, when the list was taken, how many files were learned from and how many were
left out. That is what a record entry carries when the fine-tune runs.

**How it reaches the record is a finding, not a gap this lane could close.**
`alo_record::Entry` is built from an authorised verb call, which is right: a
fine-tune is a change, so it is proposed and waits for one approval (ADR 0001),
and running it writes a record through the machinery that already exists. The
verb belongs in `alo-capability`'s closed list, which this plan may not edit. So
this crate produces exactly what that verb's record entry needs, and whoever
declares the verb — `start_a_fine_tune`, taking the granted folders — wires it.

## Revoking a grant: ADR 0047

Every other revocation in this system is complete and immediate. This one cannot
be, and the machine must not pretend otherwise. The sentence, shown where the
revocation happens:

> **This stops new training. A model that has already learned from these
> documents still has what it learned — delete the adapted model to be rid of
> it.**

What stops, what does not, and the one thing they can do. Deleting the adapted
model is offered in the same place, because a person told that something cannot
be undone must be given the action that can be taken, at that moment.

**No sentence may imply forgetting** — not *forget*, not *unlearn*, not *removes
your documents from the model*. Two tests hold it: one reads every sentence this
crate says, the other requires the translator's note to warn against exactly
those words, because a sentence that is true in English and softened in Latvian
is a person in Latvia believing something false. The ADR also rejects deleting
the adapted model automatically on revocation: it reads as tidy and destroys
something the person built, possibly for an unrelated reason.

## What is not done here

- **No trainer.** The rented fine-tuning stack is task 2's, as is the boundary
  test that runs a whole small fine-tune with no network and finds no departure
  attempted — there is nothing to run inside a boundary until something trains.
- **The verb** that starts a fine-tune, per above.
- **No model was run**, and none will be larger than the small one this machine
  holds (the owner's rule of 2026-09-15).

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima VM
as root, run by the Mac lane's publish script on the tree combined with `main`.
