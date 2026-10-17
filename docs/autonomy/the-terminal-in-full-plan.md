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
| ★ | Hand somebody a session they can replay | **nothing** — the commands wait on ADR 0123, what each changed on ADR 0045 |
| | Explain a command before it runs | **the answer half is built** — `WhatACommandWillDo`; the asking waits on ADR 0123 |

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

## Where the ten stand, 2026-10-17

**Two are done and eight are blocked. No task in this plan is startable**, and
each is blocked by one of five causes — four of which are another lane's or the
owner's:

| | task | blocked by |
|---|---|---|
| 1 | *why is it waiting* reaches a person | **done**, #755 |
| 2 | the source qualifier | **done**, by #761 |
| 6 | rich results | **the first view is done**, #761; the other two need a design component captured |
| 3 | explain a command before it runs | **shell integration** — alo does not know what was typed |
| 7 | hand somebody a session to replay | **shell integration**, and **the comparer** |
| 4 | find a command by what it did | **a scope decision** — which surface asks |
| 5 | scope one command | **`alo-capability`**, another lane's |
| 8 | bisect the machine | **the comparer** — the third PC's |
| 10 | *your files as they were* | **the comparer** |
| 9 | try a command touching nothing | **the kernel boundary** — the kernel plan's |

**So the five causes are:** shell integration (2 tasks), the privileged comparer
(3), a design component nobody has captured (part of 1), `alo-capability`'s
ownership (1), the kernel boundary (1), and one scope decision (1).

*Three of the five are named in ADRs as belonging to another lane or to the
owner. None of the eight is waiting on work this lane could do and has not.*

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

**Status:** **done, by #761** — the same change that built the first rich view,
because the qualifier *is* that view's provenance line and not a second thing
beside it.

**This task was declared startable in the order document before anybody read
it**, hours after the same document was corrected twice for the same mistake.
What it asks for is a caption inside the transcript region reading *Source:
cargo test output · Not independently verified*; what #761 draws, under the
summary and inside the region, is `Source: {what} · Not independently
verified` with `{what}` filled by *cargo test output*. The same sentence, in
the same place, from the same frame of the design.

*So the dependency this plan recorded was right, the correction to it was
right, and the thing neither noticed is that satisfying task 6 satisfies this
one outright. A task whose whole content is one sentence of another task's
output is a task to check off rather than to schedule.*

**Depends on:** task 6, and **this plan had the dependency the wrong way round
when it was written.**

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

**Status:** **the answer is built (#759); the asking waits on a decision now
written.** **Depends on:**
[ADR 0123](../decisions/0123-alo-learns-what-a-person-typed-from-the-shell-not-the-screen.md),
proposed 2026-10-17 by this lane under the rule that a machine unblocks itself
rather than waiting for somebody with less context. It was *outside this plan*
until somebody wrote it; that phrasing was the blocker, not the decision.

**alo does not know what a person has typed**, measured 2026-10-17: nothing
keeps the typed line, `the_command_in_front` is a process already running, and
the grid is `alacritty_terminal` with a `VoidListener`. Reading it off the
screen would decide what the *person* typed from what a *program* printed;
keeping our own copy of forwarded bytes diverges the first time somebody
presses Up, silently. Shell integration is the real road and is a new
mechanism, a published contract, and a decision about a sequence any program
can forge.

**`alo_terminal::WhatACommandWillDo` is built** — the plain words, the three
marks and a disclaimer no shape of the type can omit. **No chord was bound**,
because one bound to a surface that cannot work is what
`alo-shell`'s `tests/the_road_every_action_takes.rs` refuses.

*This task was called the cheapest of the remaining six. It priced the asking
and assumed the input was free.*

A person types a command and asks what it will do before running it. The model is
asked; `alo-asking`, `alo-answering` and `alo-instructing` exist and a turn
already carries a question to one. **This is the one line here not marked ★**, and
it is the cheapest of the remaining six.

### 4. Find a command by what it did

**Status:** **blocked on a scope decision.** **Depends on:** which surface asks
the question — outside this plan.

The record can answer it: an entry carries `What`, whose `effect` says whether
a call **changed** or only **answered**, and `What::touched(path)` already
matches on recorded arguments rather than on text. What is missing is anywhere
to ask from. The record window refuses by design (`record_keys.rs`: *no
letters… nothing that hides a line*); no verb exists for the record in
`docs/contracts/agent-verbs.md`; and `alo-ask` asks the **model**, not the
record. Each road is a new promise.

*One finding on the way: `RanACommand` records `allowed` — the folders the
kernel let it reach. That is **permitted, not changed**, so for a raw command
the record cannot say what it changed. For a verb it can.*

*The command that changed my git config* rather than a substring of what was
typed. `alo-record` already holds what every run did; this is a question put to
that record and a surface to ask it from.

### 5. Scope one command without changing a setting

**Status:** **blocked.** **Depends on:** `alo-capability`, which is another
lane's — and ADR 0111 leaves whose change it is to the integration owner, in
its own words, *and that question was never answered*.

One command run under a narrower grant than the session has, without touching
what the person chose for everything else. `alo-capability` has grants and
`ADR 0064` has the levels; this is a per-command narrowing and the words for it.

### 6. Output that is not stuck in a character grid

**Status:** **the first rich view is built (#761).** **Depends on:** nothing in
this plan. **Task 2 depends on this**, not the other way round — and task 2 is
therefore startable now.

**A test result is the one rich view the design specifies.** The frame `Rich
test result / untrusted program data` in `485-28999.xml` gives four children at
measured positions, and every figure is now in
`docs/design/the-terminal-window.md`. The benchmark and diff views are
`<instance>` elements whose components are **in no snapshot in this
repository**, so their insides cannot be checked against anything and were not
built.

Three of the design's own sentences became tests: *formatted output is not
system evidence* (every accessor is named `it_said_…`), *text reveals original
bytes* (a result never replaces what the program printed), and *PROGRAM
includes parsed rich results* (the lines extend the program's gutter region,
not alo's).

**What is still owed under this task:** the three `Ask alo` actions, which are
the agent's road; *a thing on your desk that outlives the scroll*, which is a
window rather than a line; and the two unspecified views.

Rich results — a table a person can sort, a file name they can open — parsed from
what a program printed. The design already reserves the room: the gutter was
measured against a node named *Rich test result / untrusted program data*, and
the annotation says *PROGRAM includes parsed rich results*. **Untrusted is the
load-bearing word**: this parses what a stranger's program printed, so the task
is as much about what it refuses to render as what it renders.

### 7. Hand somebody a session they can replay

**Status:** **blocked on two causes already known, measured 2026-10-17.**
**Depends on:** shell integration — which is no longer only a gap, it is written
as [ADR 0123](../decisions/0123-alo-learns-what-a-person-typed-from-the-shell-not-the-screen.md),
proposed the same day — and the privileged comparer. Neither is in this plan,
and task 6 is no longer the obstacle. The commands come from that decision;
*what each one changed* is
[ADR 0045](../decisions/0045-what-undoing-rewinds-to.md)'s bracket.

A transcript somebody else can step through. Task 6 was the recorded
dependency and it is built, so this read as the next startable task. It is not,
and the promise says why: *not a transcript — the commands **and what each one
changed**, replayed on their machine.* Both halves are blocked, and each by
something another task here is already blocked by:

- **the commands.** alo does not know what a person typed — task 3's blocker.
  Measured again from the other side: `alo-terminal` does not depend on
  `alo-record` (**0** mentions in its manifest), no terminal file in `alo-shell`
  names `alo_record` (**0**), and `Happened::RanACommand` **always carries an
  `agent`** — it is a command an agent ran, under whose authority, by
  construction. A person's own typed command is recorded nowhere.
- **what each one changed.** The two sides of a bracket, compared. The bracket
  is taken as of 2026-10-17, and **nothing compares the two sides**: ADR 0045
  puts the comparing in the privileged path with the base's own tools, which
  `the-broker-and-the-disk-plan.md` gives to the third PC.

*The status said "not started" because its only recorded dependency had been
satisfied. That is the fourth time in this plan that a status true about what
somebody foresaw read as a statement about what is possible — tasks 3, 4 and 5
each said* Depends on: nothing in this plan *and were blocked outside it.*

### 8. Bisect the machine, not the repository

**Status:** blocked, and **no longer on the bracket**. **Depends on:** the same
decision task 10 does — see below. The bracket landed in #767 on 2026-10-17, so
the dependency this task recorded is met; what it now waits on is *what gets
bracketed*, because walking snapshots to find when something broke needs
snapshots of the person's own working day, not only of the turns an agent
changed something in.

*It worked yesterday* becomes a question the machine can answer, by stepping
back through snapshots. There are no snapshots: nothing in the workspace takes
one, measured 2026-10-17. **This cannot start until a bracket exists**, and that
is not this plan's to build.

### 9. Try a command without letting it touch anything

**Status:** **the box is built and demonstrated; nothing calls it, and the
surface is the owner's.** **Depends on:** a decision on how a person asks for a
sealed-box run.

**This task's status was wrong on the day it was written.** It said *the box the
kernel locks down is not built* while citing `alo-running`, the crate that
contains it: #729, *a sealed box runs a command, and the kernel refuses the
rest*, landed 2026-10-16. The kernel boundary it named as a dependency landed
2026-09-04. *A claim about a module, written from the date of the crate around
it.*

**Measured 2026-10-17 on this lane's VM.** `alo_running::a_run_inside_a_box`
runs ADR 0064's box in seven steps — ask the boundary it is still there, make a
control group, put its id and places into the boundary's map, start
`alo-enter-the-box`, wait, remove the entry, remove the group — with steps 6 and
7 on every road out. 20 tests pass as the person; the two that need root pass as
root, one reading the file the box was given and one refused the file it was
not. They fail rather than skip without a writable `/sys/fs/bpf`.

**What is owed, and it is the fault this plan exists to stop, a sixth time:**

| | |
|---|---|
| crates depending on `alo-running` | **none** |
| readers of `alo_capability::HowCodeRuns` outside it | **none**, so the level a person chooses decides nothing |
| what the design says about how a person asks | **nothing** — the only match for *box* in `docs/design/the-terminal-window.md` is a label's bounding box |
| what ADR 0064 says about it | **nothing** — no prefix, command or chord |

So a caller cannot be built without inventing the surface it is called from, and
`CLAUDE.md` sends a design that is silent to the owner. **One sentence on the
scope judgement**: wiring the level to the box *makes an existing promise true*
and would be this lane's, but *how a person asks* is a surface the design does
not specify, which is the owner's — so the task stops here rather than guessing
a prefix.

### 10. *Choose one, and your files are as they were*

**Status:** blocked on a decision, **measured 2026-10-17 after the bracket
landed**. **Depends on:** whether a command the *person* runs is bracketed.

The snapshot half of *scrollback you can move along*. The scrolling is built; what
is missing is putting the files back, which no amount of scrolling can supply.

**The bracket landed (#767) and did not unblock this**, which is worth stating
because this plan said a bracket was all it needed. The promise's own words are
*every **command** in the session is a place you can return to … your files are
as they were **before it ran***, and what landed brackets **a changing agent
turn**: `TheTurnsBracket` is taken in `alo-agentd`'s serving loop at an approval,
and measured across `main` at `e7d57ae8` **nothing in `alo-terminal` or
`alo-shell` takes one**. A person's own `make`, `git commit` or `rm` is not a
place they can return to.

**And the gap is a decision rather than a missing caller**, which is why it is
not simply built here under *work a task needs is part of that task*:

- [ADR 0045](../decisions/0045-what-undoing-rewinds-to.md) term 5 is **only a
  turn that changes files is bracketed**, and alo cannot tell whether an opaque
  program changed anything — so for a person's command the choice is *snapshot
  before every one* or *none*, and the filter the term relies on does not exist;
- term 1's window — seven days or **fifty changing turns** — was sized for agent
  turns and accepted by the owner *after they asked whether the machines would
  fill up*. A shell session reaches fifty commands in an afternoon;
- so extending the bracket to the person's commands changes the cost of the one
  term the owner priced personally. **That makes it theirs**, not a caller this
  task may add quietly.

## What this plan is waiting on, in one place

- **Whether a command the person runs is bracketed** — tasks 8 and 10. **The
  bracket itself is built and landed** (#767, 2026-10-17): `alo-turn` takes one
  either side of a changing agent turn, measured on a real `btrfs` filesystem.
  What neither task can start without is a decision on bracketing the person's
  own commands, which ADR 0045 does not make and whose disk cost is the question
  the owner asked when they accepted it. *This bullet said "a bracket" until the
  bracket arrived and the tasks stayed shut.*
- **How a person asks for a sealed-box run** — task 9. **Not the box**: that is
  built (#729, 2026-10-16) and passes on a real kernel. What is missing is a
  caller and the surface it would be called from, and the design file and ADR
  0064 are both silent on the surface. *This bullet said "the kernel box" and
  pointed at two ADRs that had already been built against.*
- **The owner, on the pairing of five answers to four actions** — `docs/features.md`
  records it as theirs. Task 1 does not need it; offering the actions does.
