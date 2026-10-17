# The terminal in full

**Workstream:** every promise `docs/features.md` makes about alo OS's own
terminal, and the wiring that makes each one reach a person. Thirteen lines,
eleven of them marked **★** — things no other operating system offers.

**Why it exists:** the owner asked on 2026-10-17 to *finish all the wow features
of the terminal, and the terminal should be wired fully*. The work had been
arriving one mechanism at a time, each gated and green, and **six of them have
never been seen by anybody**: the five answers to *why is it waiting*, and the
four things to do about it, were built, tested against real processes and real
kernels, and reach no screen. A promise that works and cannot be reached is the
fault this repository has now found four times in four shapes, and this plan
closes it for one feature at a time rather than adding a fifth.

**Crates this plan owns:** `alo-terminal`, which no plan claimed. It edits
`alo-shell`, which this lane already holds through
[`more-than-one-display-plan.md`](more-than-one-display-plan.md) — *"Crates this
plan owns: `alo-shell`"* — so the right to edit is not being taken from anybody.
The desktop lane confirmed on 2026-10-17 that it has nothing open there.

**What this plan does not do.** It does not design. What the terminal looks like
is settled in the design file and recorded in
[`docs/design/the-terminal-window.md`](../design/the-terminal-window.md), which
carries the window's one size, the transcript's protected region, the 96-wide
gutter and the note beneath. Where that file is silent, the owner decides and the
decision is written there — not here, and not in a diff.

## What is already true, measured 2026-10-17

Checked by reading `docs/features.md`'s thirteen lines against the code rather
than against their own status notes, because a status note is a measurement with
a date on it:

| | promise | state |
|---|---|---|
| | A terminal | **reaches a person** — a chord opens it, the compositor draws it, typing arrives |
| ★ | You decide whether alo may use it | **built** — the three levels of ADR 0064 |
| ★ | You can tell who is speaking | **built** — `you`, `alo`, `system`, `program` in the gutter |
| ★ | alo says what it measured apart from what it worked out | **built** — the same gutter's `system` against `alo` |
| ★ | Why is it waiting? | **five answers and four mechanisms built; no surface** |
| ★ | Scrollback you can move along | **built** — back, forward, and it stops at both ends |
| ★ | Try a command without letting it touch anything | the record and the runner exist; **the kernel box does not** |
| ★ | Scope one command without changing a setting | **nothing** |
| ★ | Find a command by what it did | **nothing** |
| ★ | Bisect the machine, not the repository | **nothing** |
| ★ | Output that is not stuck in a character grid | **nothing** |
| ★ | Hand somebody a session they can replay | **nothing** |
| | Explain a command before it runs | **nothing** |

Two smaller gaps inside promises otherwise kept: the source qualifier
`· cargo test` beside a gutter label, and *choose one, and your files are as they
were*, which is the snapshot half and waits on a bracket.

## Rules

- **One task at a time, and each one reaches a person.** No task in this plan
  ends with a mechanism nobody can call. Where a promise is two halves — a
  mechanism and a surface — the task is both, in one change, because this plan
  exists to stop the halves shipping apart.
- **The design decides what it looks like.** Where it is silent the owner
  decides; record it in `docs/design/the-terminal-window.md`.
- **Lower case for quiet labels**, the owner's decision of 2026-10-17, recorded
  in `docs/design/the-first-start.md`.
- **A new task goes at the end**, numbered in order.

## Tasks

### 1. The answer to *why is it waiting* reaches a person

**Status:** done, 2026-10-17. Report:
[`updates/the-answer-reaches-a-person.md`](updates/the-answer-reaches-a-person.md).
**Depends on:** nothing.

**What it also found.** `alo_shell::the_road_every_action_takes` was cited by
`alo_shortcuts::Action::Launcher`'s own documentation as the thing naming it the
single unreached action — and **did not exist**, so nothing held any of the
twenty-two actions to having a road. Built in the same change, and it took three
attempts: a handler names its own action, and the comment above an arm names the
handler the arm calls, so both of the first two guards passed the mutation they
were written for.

The five answers and all four mechanisms are built. What is missing is the
asking and the saying: a person asks, alo analyses the command in front, and alo
replies with why it is waiting.

**The owner said what this is**, on 2026-10-17: *"Analysis the command and see
why it's not working and then reply the user on why it is waiting."* So the
**answer** is the deliverable. The four actions stay built and unoffered; the
pairing of answers to actions is recorded in `docs/features.md` as the owner's to
settle and is not inferred here.

**Where it appears is derived from the design, not invented.** The gutter's own
annotation in the design file reads *"SYSTEM means an attributed observation,
with scope and time. ALO is interpretation."* The waiting answer **is**
interpretation of observations — `why_it_is_waiting` reads three unprivileged
files and reads a reason off them — so it is a line in the transcript carrying
the `alo` label. No new region, no new chrome, and the design's own vocabulary
covers it.

**Acceptance:** a person opens a terminal, runs something that waits, asks, and
reads *waiting for the network* with `alo` beside it. On a real machine, against
a real pseudoterminal.

### 2. The source qualifier beside the gutter label

**Status:** blocked. **Depends on:** task 6, and **this plan had the dependency
the wrong way round when it was written.**

The qualifier is a caption inside the transcript region — the design's own
example is `Source: cargo test output · Not independently verified` — and
`docs/design/the-terminal-window.md` says what it is for:

> **The qualifier is not dropped, it is owed.** The design puts the source detail
> inside the region as a caption … rather than in the gutter. **That belongs with
> rich results, which are not built**, so it is recorded here rather than guessed
> at.

**It is provenance for a rich result**, not a second label. *Source: cargo test
output* and *not independently verified* are statements about where a parsed
table came from and how much to trust it — so there is nothing for it to qualify
until something is parsed. Building it first would be inventing a caption for
output that has none.

*This plan listed it as depending on nothing, and task 6 as depending on it. That
was backwards, and reading the design before starting it is what found that —
which is the order `CLAUDE.md` asks for: the design is followed, not
approximated.*

### 3. Explain a command before it runs

**Status:** not started. **Depends on:** nothing in this plan.

A person types a command and asks what it will do before running it. The model is
asked; `alo-asking`, `alo-answering` and `alo-instructing` exist and a turn
already carries a question to one. **This is the one line here not marked ★**, and
it is the cheapest of the remaining six.

### 4. Find a command by what it did

**Status:** not started. **Depends on:** nothing in this plan.

*The command that changed my git config* rather than a substring of what was
typed. `alo-record` already holds what every run did; this is a question put to
that record and a surface to ask it from.

### 5. Scope one command without changing a setting

**Status:** not started. **Depends on:** nothing in this plan.

One command run under a narrower grant than the session has, without touching
what the person chose for everything else. `alo-capability` has grants and
`ADR 0064` has the levels; this is a per-command narrowing and the words for it.

### 6. Output that is not stuck in a character grid

**Status:** not started. **Depends on:** nothing in this plan. **Task 2 depends
on this**, not the other way round.

Rich results — a table a person can sort, a file name they can open — parsed from
what a program printed. The design already reserves the room: the gutter was
measured against a node named *Rich test result / untrusted program data*, and
the annotation says *PROGRAM includes parsed rich results*. **Untrusted is the
load-bearing word**: this parses what a stranger's program printed, so the task
is as much about what it refuses to render as what it renders.

### 7. Hand somebody a session they can replay

**Status:** not started. **Depends on:** task 6, for the same reason task 2
does.

A transcript somebody else can step through — which needs the transcript to be
more than cells, so it follows rich results rather than preceding them.

### 8. Bisect the machine, not the repository

**Status:** blocked. **Depends on:** a bracket, which is `alo-turn`'s and
unbuilt.

*It worked yesterday* becomes a question the machine can answer, by stepping
back through snapshots. There are no snapshots: nothing in the workspace takes
one, measured 2026-10-17. **This cannot start until a bracket exists**, and that
is not this plan's to build.

### 9. Try a command without letting it touch anything

**Status:** blocked. **Depends on:** the kernel boundary, ADR 0013 and ADR 0015.

The sealed box of ADR 0064. `alo-running` is the runner and the record, built
2026-10-17; the box the kernel locks down is not built, so a machine at the
default level runs nothing. **Honest behaviour for a default whose enforcement
does not exist**, and not something a surface can paper over.

### 10. *Choose one, and your files are as they were*

**Status:** blocked. **Depends on:** a bracket, as task 8 does.

The snapshot half of *scrollback you can move along*. The scrolling is built; what
is missing is putting the files back, which no amount of scrolling can supply.

## What this plan is waiting on, in one place

- **A bracket** — tasks 8 and 10. `alo-turn`'s under ADR 0045; nothing takes one;
  the layout it would write was moved somewhere a turn can reach it on
  2026-10-17, which was the first thing in the way.
- **The kernel box** — task 9. ADR 0013's and ADR 0015's.
- **The owner, on the pairing of five answers to four actions** — `docs/features.md`
  records it as theirs. Task 1 does not need it; offering the actions does.
