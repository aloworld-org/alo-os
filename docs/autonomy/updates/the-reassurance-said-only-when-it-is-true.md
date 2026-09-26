# The reassurance, said only when it is true

Task 13 of `docs/autonomy/v0-5-the-session-and-the-displays-plan.md`, done
2026-09-26 on the third PC, under
[ADR 0068](../decisions/0068-a-published-sentence-changes-by-getting-a-new-key.md)
as accepted.

Task 12's walk found `displays.the-desk-changed` promising *the arrangement you
made at the other desk is still here for when you are back at it* to somebody who
was back at it — a sentence making a promise about a situation it was not in, wrong
on every return leg of every journey, and read beside
`displays.as-you-left-them` a duplicate of it.

It is split. The plain fact and the reassurance are two sentences now, and the
reassurance is said only when the screens in front of the person are **not** a set
they have arranged — which is when they are actually losing an arrangement.

## The two keys, and the one that retired

| | |
|---|---|
| **retired** | `displays.the-desk-changed` |
| **new** | `displays.woke-to-other-screens` — *Your screens have changed since this machine went to sleep, so alo OS has set up the ones in front of you now* |
| **new** | `displays.the-other-desks-arrangement-is-kept` — *The arrangement you made at your other desk is still here for when you are back at it* |

Two new keys and a retirement rather than an edit, because the remaining half no
longer means what the whole did: a translation of the old key would have gone on
promising something the sentence no longer says. That is ADR 0068 §1 and §3, and
its question — *would a correct translation of the old English still be a correct
translation of the new one?* — answers **no** here, unmistakably: the old sentence
made two claims and the new one makes one.

`Note::TheDeskChanged` keeps its name, because a variant name is Rust rather than a
key and *the desk changed* is still what it means. The new variant is
`Note::TheOtherDesksArrangementIsKept`.

## The condition, and why it is not a second opinion

`Changes::for_screens(&self.arrangement.screens()).is_none()` — the reassurance is
said when the arriving set has no kept arrangement. That is **the same question
that already chose** between restoring a kept arrangement and working one out a few
lines earlier in `resumed_to`, so nothing new decides what counts as a desk of
theirs. Had it needed a second rule, that would have been an argument for leaving
the sentence alone.

## The walk, sentence by sentence

Task 12's report is **not edited** — it is published, and its table was true of the
sentence as it then was. This is the table republished, which is the mechanism that
walk's own test describes for exactly this case, and
`crates/alo-sleeping/tests/the_walk_to_a_desk_that_changed.rs` now reads it here.

| When | What she reads |
|---|---|
| Anna signs in at her own desk, with her own screen beside the laptop | Built-in screen has not been used with this machine before, so alo OS has put it beside your other screens and chosen a size for it from how large it is |
| Anna signs in at her own desk, with her own screen beside the laptop | Built-in screen does not say which screen it is, so alo OS remembers it by the socket it is plugged into — another screen plugged into that socket will be set up the same way |
| Anna signs in at her own desk, with her own screen beside the laptop | Iiyama ProLite XU2793 has not been used with this machine before, so alo OS has put it beside your other screens and chosen a size for it from how large it is |
| she opens the lid at the office, where the screens are not the ones she left | Your screens have changed since this machine went to sleep, so alo OS has set up the ones in front of you now |
| she opens the lid at the office, where the screens are not the ones she left | The arrangement you made at your other desk is still here for when you are back at it |
| she opens the lid at the office, where the screens are not the ones she left | Dell U2720Q has not been used with this machine before, so alo OS has put it beside your other screens and chosen a size for it from how large it is |
| she opens the lid at the office, where the screens are not the ones she left | Iiyama ProLite XU2793 was unplugged, so what was open on it is now on Built-in screen |
| she opens it again at her own desk, where her own screen is back | Your screens have changed since this machine went to sleep, so alo OS has set up the ones in front of you now |
| she opens it again at her own desk, where her own screen is back | Your screens are arranged the way you last left them |
| she opens it again at her own desk, where her own screen is back | Dell U2720Q was unplugged, so what was open on it is now on Built-in screen |
| she opens it again at her own desk, where her own screen is back | Iiyama ProLite XU2793 is back, so what was open on it before has gone back to it |

## Read against the criterion again

**The office morning:** what happened, then that what she arranged elsewhere is
safe, then the new screen placed, then where her open work went. An account, and the
reassurance is where it earns its place — she is at a desk she has never arranged,
and the thing she might worry about is the layout she left behind.

**The morning she comes home:** what happened, then *your screens are arranged the
way you last left them*, then where the office screen's work went, then what came
back to her own. **The wrong promise is gone and nothing replaced it.**

The two remaining sentences on that morning are not a duplicate, and it is worth
saying why rather than asserting it: *your screens have changed since this machine
went to sleep* is about the set differing from the one it slept with, and *your
screens are arranged the way you last left them* is about this set matching one she
made. Two facts, in a sequence — she learns why the screens moved and that they are
where she wants them. The old pairing failed because both sentences claimed the
machine had laid her screens out from something it kept, and one of them pointed at
the wrong desk.

## What this did not do

No other sentence was touched. `displays.as-you-left-them` is exactly as it was and
is doing the work the criterion said it should. Nothing in `crates/alo-shell`,
nothing on the machine, `logind` stays rented (ADR 0011), and both of task 12's
findings are now closed — the ordering by task 14, this by task 13.

## Evidence

    cargo test -p alo-displays                      PASS  (123 unit tests)
    cargo test -p alo-sleeping                      PASS  (the walk's 3 tests)
    cargo test -p alo-saying                        PASS  (the snapshot, 1,478 keys)

The vocabulary snapshot ADR 0068 §amendment two added moved by exactly what this
change did: `displays.the-desk-changed` gone, two keys in its place. Neither needed
a record in `reworded.txt` — adding a sentence is additive and a retired key's
translations are never consulted again — and the snapshot test says so in its
failure message rather than leaving somebody to wonder.

probe_should_be_7=7
