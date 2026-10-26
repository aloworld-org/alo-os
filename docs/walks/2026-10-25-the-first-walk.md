# The first walk of *a day on alo OS* — 2026-10-25

**Machine:** qemu on the development PC, 4 GB, 4 CPUs, software rendering.
**Image:** built from `main` at `b0eff35c`, which is *A grant can last until the
person removes it (#959)* — so it contains #956, #957, #958, #959 and #960.
**Model:** `gpt-4o` at OpenAI, the owner's own key, entered on the machine.
**Not a certified PC.** Every claim below is about this virtual machine, and
`ROADMAP.md` is right that a PC we can put our hands on is the one that counts.

## How far it got

**Eleven of seventeen steps.** Steps 1–11 of
`docs/walks/a-day-on-alo-os.md` passed, in one sitting, on one machine:

| Step | What was proven |
|---|---|
| 1 | The image boots |
| 2 | The language sheet appears first, English chosen |
| 3 | An account is made — a name, a PIN — and a session starts |
| 4 | The desktop appears with the Dock |
| 5 | **Documents, Downloads and Pictures exist** (#956) |
| 7 | **A folder is given to alo, *until I remove it*** (#957, #959) |
| 8 | **The grant survives closing and reopening Settings** |
| 9 | A provider is added with a real key, tested, saved; `gpt-4o` chosen |
| 10 | `Super`+`A` opens the Bar and takes a question in plain words |
| 11 | **A real model answers, and proposes a change that waits** |

Step 6 (put a file in Documents) was skipped: the terminal was still
installing. Steps 12–17 were not reached.

**What step 11 is worth saying plainly:** a person typed a question in their own
words, a model at a provider answered, and the machine turned that into one
sentence describing a change and **stopped**. Nothing ran. That is law 2 and
*reads answer, changes wait*, working on a booted machine rather than in a test.

## Why it stopped, and it is one line

Asked three times for something to be **read**, alo three times proposed a
**change**:

| asked | proposed |
|---|---|
| what is in my documents folder | install `org.gimp.GIMP` from flathub |
| list my documents folder | install `org.kde.krita` from flathub |
| list my pictures folder | archive `/home/alo/Pictures/` into `/home/alo/Documents/` |

**The cause is not known, and the first answer given here was wrong.**

This report originally said the cause was the text a turn shows:
`Instructions::SHOWN_TO_A_TURN` is `OneExamplePerDoor`, whose first sentence is
*"Answer with one line of JSON and nothing else: no explanation, no code fence,
no second line."*, while the paragraph permitting a model to look first lives in
`SeveralSteps`, which a turn does not show. It is a tidy explanation and it fits
all three observations above.

**It does not survive measurement.** `does_a_model_look_before_it_proposes`
asks a question built to require looking — *the only PDF in a folder, name not
remembered* — and under `OneExamplePerDoor`, the very text accused here,
`gpt-4o` went through the **read** door in **5 of 5** rounds. A model shown that
text looks first perfectly well.

So the three proposals above have some other cause, and what distinguishes the
walk from the measurement is still open. The most visible difference is the
request: the walk asked short, vague things — *"what is in my documents
folder"* — where the exercise is specific about what it wants and about what it
does not know. Whether that is the difference has not been measured.

**What is now known, and was not before:**

- a model shown the current text **does** look before proposing, reliably;
- the fault is therefore not that a turn forbids looking;
- `SeveralSteps`' sentence *"You may send more than one line, one at a time"*
  **can be misread, and was**: a model sent two lines in one response and
  `alo_protocol::FromAnAgent::read` refused it as `MoreThanOneMessage`. The
  text and the protocol agree — the text says a line at a time and the refusal
  means *more than one message where one was expected* — so this is a sentence
  to sharpen rather than a disagreement to resolve. **It was first written up
  here as the two contradicting each other, from seeing the refusal once and
  not reading what it meant.**;
- a measurement of one sample is worthless here: three runs of this exercise
  gave three different pictures before rounds were added.

### What this walk ruled out, and this is the useful part

Three theories were held and dropped, each by a measurement rather than by
argument:

- **"The granted folders never reach the model."** False. The third proposal
  names `/home/alo/Pictures/` **and** `/home/alo/Documents/` — both real, both
  granted. `WhatItMayReach` (#958) works on a real machine.
- **"The endless grant is the problem, so it is today's change."** False. A
  second folder was granted **for an hour** and the model used that path too.
  Timed and endless grants reach it alike.
- **"The screen throws the read's answer away."** False. The shell prints
  everything the agent writes, and `alo-agent` prints an answer when a turn
  answers. The turn genuinely *ended* in a proposal.

## The measurement this produced

`gpt-4o` at a provider, on the fixed ten, scored by the same code:

```
under one-example-per-door: 10 of 10 drove the verbs — "reliably"
under several-steps:        10 of 10 drove the verbs — "reliably"
```

**This proves safety, not benefit, and must not be read as more.** The fixed ten
are single-call exercises; not one asks a model to look something up before
acting, so they cannot tell the two texts apart and both saturate. *A bar that
every candidate maxes out does not choose between them.*

**It runs again** by `crates/alo-driving/tests/against_a_model_at_a_provider.rs`
with `ALO_PROVIDER_INSTRUCTIONS=several-steps`, which is `#[ignore]`d because it
leaves the machine.

## Why the one-line change was not landed, and why it would not have helped

**It would not have fixed this.** See the correction above: the text it would
have replaced is not what stopped the walk. The section is kept because the
guard tests it ran into are worth knowing about.

`SHOWN_TO_A_TURN = SeveralSteps` is one line, and three guard tests refuse it:

- `what_a_turn_shows_is_a_set_that_has_been_measured`
- `what_a_turn_is_shown_is_named_and_measured`
- `the_digest_the_catalogue_decides_by_is_the_one_a_turn_is_shown`

The third is the real one: every grade in the catalogue was earned under the
current text, and switching would leave *the catalogue deciding by a digest
nothing has*. Re-earning them needs local weights and the hardware they were
measured on; this machine has neither (ADR 0095 ships no model). **The change
was made, the guards caught it, and it was reverted rather than the guards
weakened.**

**The next task is therefore not the switch.** It is an exercise that **fails**
under `OneExamplePerDoor` and **passes** under `SeveralSteps` — one that cannot
be answered without looking first. That makes the fixed ten able to decide this,
and turns a walk's finding into something CI holds. Then the re-grade and the
one-line switch are routine.

## Everything else the walk found

Smaller, all real, none of them blocking:

- **A fresh machine installs its applications for 35+ minutes, and says
  nothing.** `Ctrl`+`Alt`+`T` during that time does nothing at all — no "still
  setting up". Three of seven applications were in when the walk reached the
  desktop. The machine's own log says it: *"a chord opens alo's own terminal,
  and this machine has no installed terminal to fall back to"*.
- **The PIN field draws four dots however many digits are typed.** Eight keys,
  four dots, and the account was created — so it is a drawing limit, not a
  storage one. A person cannot see how much they have typed.
- **The Settings list does not refresh when a provider is saved.** It keeps
  saying *"nothing answers questions"* until the window is closed and reopened.
  A person would conclude their key was rejected. This is the same family as the
  fault `a_provider_is_added_in_settings.rs` records from 2026-10-23 — *"A
  person pressed Enter and the machine answered with silence"* — one layer up.
- **The image ships no translations.** `alo-agentd` says
  */usr/share/alo/translations could not be read*, so the machine speaks English
  whatever was chosen on the language sheet. That is worth weighing against the
  promise of all 24 EU languages.
- **Typing goes to the Bar while a task window is open**, not to the task's own
  *"Reply to alo or change the direction…"* field, which is where a person is
  looking.
- **Not a fault:** no local model (`models=0`) is ADR 0095 working as intended —
  the release carries no weights and a person brings their own or adds a
  provider.

## What the walk cost, as a method

It found in one sitting what five green test suites did not: every one of the
faults above passed CI. That is the argument for `docs/walks/` existing, and the
argument for the rule added the same day — *nothing counts as done until a
person can reach it*.
