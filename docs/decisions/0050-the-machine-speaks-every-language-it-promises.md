# 0050 — The machine speaks every language it promises, on the machine

**Status:** accepted
**Date:** 2026-09-17
**Context:** [ADR 0011](0011-the-base-is-rented-and-the-image-is-a-container.md)
(what this machine runs is rented, configured and never patched),
[ADR 0008](0008-where-inference-happens.md) and its rule that there is *never a
silent fallback*, `docs/features.md`'s accessibility line (*the AT-SPI tree the
agent uses is the one a screen reader uses; EN 301 549 conformance is the same
work, not extra work*) and its language line (*all 24 official EU languages to
begin with … a sovereignty product that cannot speak Maltese or Irish is selling
sovereignty to some Europeans and not others*), and
`docs/autonomy/v0-5-access-and-language-plan.md`.

## What was found

The image pins no speech engine. Nothing in this repository decides how a person
who does not read the screen hears it, and `image/Containerfile` names no engine
that could. The finding came from the lane that built fine-tuning, which had to
pin its own stack and noticed the neighbouring gap.

This is not a missing feature; it is a missing decision. A screen reader that
cannot speak is not a screen reader, so accessibility conformance is unreachable
until something is chosen — and whatever gets chosen in a hurry, at the moment a
task needs it, will be chosen for how good it sounds in English.

## The decision

**alo OS pins `espeak-ng`, reached through `speech-dispatcher`, and it speaks
every language this product promises.**

Both are rented and unmodified (ADR 0011). `speech-dispatcher` is the seam the
screen reader already talks to, so the engine behind it can be replaced without
anything above it changing.

**Speech never leaves the machine.** There is no cloud voice, not as a default
and not as an option a person can switch on: what a blind person reads is
everything they read, and sending it to a service would be the most intimate
egress this system could perform. A voice that needs a network is not a voice
this machine has.

**Coverage decides the engine, not how pleasant it sounds.** `espeak-ng` speaks
Irish and Maltese; the better-sounding neural engines mostly do not, and the
ones that do cover them worst. Choosing the engine that sounds best in English
and degrades in Maltese would be exactly the failure `docs/features.md` names —
selling sovereignty to some Europeans and not others. So the baseline is the
engine with the widest coverage, and it is the same engine for everybody.

**A better voice may be added per language, never per market.** Where a
higher-quality voice exists for a language, it may be pinned alongside, provided
it runs on the machine and provided no promised language is left worse off than
the baseline. A language with no better voice keeps the baseline and is not
told it is missing something.

**A language this machine cannot speak is said, once, plainly** (ADR 0008).
Not a silence, not a fallback to English read in an English voice — a sentence
naming the language and what a person can do. English arriving unannounced in
place of someone's own language is the failure this rule exists to prevent.

## What was rejected

**A neural engine as the baseline** (Piper and its kin). It sounds markedly
better and is the obvious choice if the first test is done in English. Its voice
coverage is the problem: the promise is 24 languages including the two with the
least software written for them, and an engine that covers 20 of them well is
an engine that makes this product's central claim false. Revisit when coverage
is complete, not when it is good enough for the languages we happen to test in.

**A hosted voice.** Better than either, and disqualified by law 1. It would also
be the one egress a person could not audit by reading their screen, because the
whole point is that they are not reading it.

**Leaving it open until a task needs it.** That is how it would have been
decided at the worst moment, by whoever was closest to a deadline.

## Consequences

- `image/Containerfile` pins `speech-dispatcher` and `espeak-ng` by version,
  as the document converter is pinned.
- The access-and-language plan owns the crate work; this decision does not
  write it, and no lane should pick a different engine in passing.
- The 24-language completeness test that `alo-formats` already applies to dates
  and numbers extends to speech: a language that cannot be spoken fails it, in
  the same way and for the same reason.
- No source patch to either engine without an ADR (ADR 0011). If a language is
  pronounced badly, that is a finding and a rented-engine issue, not a patch.
