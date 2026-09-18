# Everything from the keyboard, including the things nobody told you the key for

**Date:** 2026-09-18
**Workstream:** v0.5 — access and language
**Task:** [task 3](../v0-5-access-and-language-plan.md) — keyboard-only
operation of everything
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written on an **Apple M3 with 8 GB unified memory**, macOS 26.5.2;
built, linted and tested in the Lima VM on that Mac — **Ubuntu 24.04 aarch64, 6
CPUs, 3 GB of memory**.
**Egress:** none.
**Status:** done. Its blocker cleared when `alo-keyboards` landed (`2c2510f`).

## The question worth asking

Every one of `alo-shortcuts`' eleven actions already answers to a chord. So *is
there a keyboard road?* is answered **yes** before any of this was written, and a
task that stopped there would have been a tick against a clause and nothing for a
person.

The question this task actually answers is the next one: **can somebody who has
not been told the chord find it?** A person who cannot use a pointer and does not
know that the agent is `Super+A` has no road at all — and a list of chords in a
manual is not a road, it is homework.

So `alo-access::reaching` holds something stronger than clause 11.2.1.1 asks:
**every action has a chord *and* a place the keyboard arrives at by pressing
Tab**, and `the_tab_stop_for` says where each one is. Two of those stops did not
exist, and the accessibility tree gained them in the same change:

| Action | Where the keyboard finds it |
|---|---|
| the agent | **new** — *ask the agent*, on the desktop |
| the launcher | **new** — *open something*, in the dock |
| next/previous window | the list of open windows |
| next/previous application | the dock |
| close, minimise, maximise, snap left, snap right | the window's own controls |

The plan asked for the agent by name — *the agent overlay's one key is not the
only road* — and the answer for all eleven is now the same.

## The focus order is the reading order, and cannot disagree

Not a second list. `focus_order` **is** `tree`'s reading order with everything
that cannot be used taken out, so there is nowhere for the two to drift apart. A
surface where the keyboard visits controls in a different order from the one a
screen reader announces them in is a surface that cannot be built here.

That is EN 301 549's meaningful focus order, held by construction, and it is why
getting the reading order right in task 2 was worth the argument.

## Escape leaves every surface, and never answers yes

`leaving` says what Escape does on each of the eight surfaces, and there is **no
variant for *nothing happens*** — a surface a person cannot leave with one key is
the trap the clause is about.

The one that matters is the approval: **Escape declines.**

- It must never approve. What a person approves is the sentence (ADR 0001), and a
  key pressed to get out of the way is not somebody reading a sentence.
- It must not leave the request unanswered either. Somebody who pressed Escape
  believes they have dealt with it, and a change still waiting behind that belief
  is worse than either answer.

## Sticky, slow and bounce keys, as behaviour rather than as settings

`key_filter` already held what a person set. `filtering` holds what the keyboard
then does, with a test each:

- **Sticky** — `Ctrl`, let go, then `c` is `Ctrl+c`, and the modifier is let go of
  after the key it belonged to. Clause 5.9: nothing needs two keys at once.
- **Slow** — a key counts on the way up, and only if it was held for as long as
  the person asked, so a hand resting on a keyboard types nothing.
- **Bounce** — the same key struck again too soon counts once; a *different* key
  straight after is a person typing, not a tremor.

And a fourth test for all three at once, because that is the argument
`key_filter` makes for one filter rather than three settings: **slow keys decide
whether a press happened, bounce keys whether it counts, sticky keys what it
counts as.** In any other order the keyboard swallows the second half of
everything.

**Time arrives as an argument**, never read from a clock — the rule
`alo-choosing` states about the environment, for the same reason. A test is
therefore a list of presses rather than a person with a stopwatch.

## What this moved in the conformance file

`alo-conforming` went from **12 met to 17**, all five by tests that now exist:

| Clause | Met by |
|---|---|
| 11.2.1.1 keyboard | every action reached by Tab and not only by a chord |
| 11.2.1.2 no keyboard trap | Escape leaves every surface |
| 11.2.4.3 focus order | the focus order is the reading order |
| 5.8 double-strike | bounce keys count one press where a hand shook twice |
| 5.9 simultaneous actions | sticky keys make a chord out of two presses |

**5.7 (key repeat) is deliberately still *not yet*.** Slow keys are not key
repeat: the clause is about the delay before a held key repeats and about turning
that off, and nothing here holds a repeat rate. Calling it met would have been
the over-claim I corrected in the same file yesterday.

## What is still the shell's

Nothing here draws. The tree says what must be reachable and in what order; the
shell honours it. **Focus being visible** is the clearest of these: the setting
exists (`FocusAlwaysVisible`, task 1) and what it changes is declared, but a
person can only see a focus ring somebody drew — so 11.2.4.7 waits on the
desktop plan's task 6, with the other 23.
