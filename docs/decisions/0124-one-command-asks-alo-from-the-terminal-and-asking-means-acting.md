# ADR 0124 — One command asks alo from the terminal, and asking means acting

**Status:** **accepted, 2026-10-18, by the owner**, ruling on a disagreement
between the design and the code that the Mac lane found while writing
[ADR 0123](0123-alo-learns-what-a-person-typed-from-the-shell-not-the-screen.md).
The owner's words: *we should have only one … and also it should be the one for
acting and answering.*

## The question

`docs/design/the-terminal-window.md` draws a person reaching alo from the
command line:

```text
you      launch ❯ alo "run this project"
alo      Found one project: ~/Projects/launch…
         Proposed command: npm run dev
system   Grant: read this project; ask before…
alo      Run this command? [y/N]
```

The built command is `alo-ask`, and it does none of that. So three documents
disagreed: the design draws a command that **acts**, the code has one that only
**answers**, and `docs/features.md` names neither.

## What was measured, 2026-10-18

| Asked | Answer |
|---|---|
| what `alo-ask` depends on | `alo-asking`, `alo-models`, `alo-answering` — a model, and nothing else |
| does it reach the agent, the broker, a proposal or an approval | **zero mentions** of any of them in its `main.rs` |
| is there an `alo` binary | **no.** `alo-ask` is the only one a person could type |
| files naming `alo-ask` | **151**, across plans, contracts and decisions |

So the command wearing the name *Ask alo* was the one thing in the product that
could not act, while **Ask alo acts everywhere else** — the dock's opens a task
window with Activity, actions, results, permissions and Stop.

## The decision

**1. One command, not two.** The lane proposed keeping `alo-ask` as an
answer-only road beside a new acting one, on the ground that *this cannot act*
is a guarantee worth having. **The owner ruled against it.** Two commands make a
person learn which of them is the real one, and the answer-only road is the one
nobody wants when they discover the other exists.

**2. Asking means acting.** The one command both answers a question and does
what it is asked: it proposes, it says what grant the work needs, and it waits
for an approval before anything happens. *Ask alo* means the same thing in the
terminal as it does in the dock and the composer, which is the whole point of
the ruling.

**3. The name is not the question, and the existing one stands.** The owner: *or
any name, I do not care about the name.* So `alo-ask` keeps its name, for a
reason better than the 151 files it would cost to change: **the mismatch was
behavioural, not lexical.** `alo-ask` read wrongly only because it could not do
what *Ask alo* does. Once it acts, the name is simply correct.

## The departure this records

`CLAUDE.md`: *the design is followed, not approximated. Where code and design
differ the design is right until an ADR says otherwise, and a deliberate
departure is written down where the next person looks.*

**This is that ADR, and this is that departure.** The design's sample types
`alo`; the command is `alo-ask`. Everything else in the sample — the proposal,
the grant, the approval before anything runs — is what the command becomes, not
what it gives up. `docs/design/the-terminal-window.md` carries a note pointing
here so the next person reading that transcript is not left to wonder.

## What this does not decide

- **Whether the person can tell, before pressing Enter, that a line will act
  rather than answer.** They cannot, and under term 2 above they do not need to:
  nothing happens without an approval, so a question misread as a request costs
  a sentence rather than an action. Said here because it is the obvious worry
  and the answer is structural.
- **What the follow-up prompt is.** ADR 0113 says `alo ❯` means the
  conversation. Whether this command opens one, and what owns the line while it
  does, is that decision's and not this one's.

## Consequences

- **`docs/features.md` gains the promise**, because this is a surface a person
  reaches and scope is gated. *Drafted by the lane for the owner, 2026-10-18.*
- **`alo-ask` grows a road to the agent it does not have**: today it names the
  agent, the broker, a proposal and an approval zero times. That is the work,
  and it is most of it.
- **ADR 0043 and ADR 0111 are untouched.** *The terminal is a person's and never
  an agent's* survives: the person types the command, so nothing here lets alo
  reach a terminal it was not invited into.
- **But ADR 0111's choice still governs what happens after the approval.**
  *Whether it may run commands in one is the part you decide* — so an approval
  on *Run this command? [y/N]* is one person approving one command, which is the
  charter's *reads answer, changes wait*, and it is **not** the standing setting
  that lets alo use a terminal freely. *This reading is the lane's and is marked:
  if the owner means the standing setting to gate the single approval too, say
  so and it is written in.*

## The test that decides whether this works

A person types the command with a request, is shown a proposal and the grant it
needs, approves it, and the thing happens — **on a machine**, not in a fixture.
The mechanism working, something calling it, a person reaching it and somebody
having seen it are four facts, and a green suite speaks to three.
