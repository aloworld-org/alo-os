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
| ★ | You decide whether alo may use it | **built as types and stored nowhere** — the three levels of ADR 0064 exist in `alo-capability`; measured 2026-10-17, no crate outside it reads them and the machinery that keeps a person's grants cannot express a level. A person can neither see nor change either choice |
| ★ | You can tell who is speaking | **built** — `you`, `alo`, `system`, `program` in the gutter |
| ★ | alo says what it measured apart from what it worked out | **built** — the same gutter's `system` against `alo` |
| ★ | Why is it waiting? | **built and drawn** — `Ctrl`+`Alt`+`W` draws `alo  waiting for the keyboard`, and `waiting for something alo cannot identify` for a clock. This row said *no surface* until 2026-10-17; the surface was there and **drew nothing a person could read**, which #778 fixed |
| ★ | Scrollback you can move along | **built** — back, forward, and it stops at both ends |
| ★ | Try a command without letting it touch anything | **the box is built** — #729, 2026-10-16, and it passes on a real kernel. This row said it did not exist. **Nothing calls it**, and how a person asks is a surface the design does not specify |
| ★ | Scope one command without changing a setting | **nothing** |
| ★ | Find a command by what it did | **nothing** |
| ★ | Bisect the machine, not the repository | **nothing** |
| ★ | Output that is not stuck in a character grid | **the first rich view is built and drawn** — a recognised test result, #761, and its two lines were cropped into the gutter's label box until #778. This row said *nothing*. Two more views are unspecified in the design |
| ★ | Hand somebody a session they can replay | **nothing** — the commands wait on ADR 0123, what each changed on ADR 0045 |
| | Explain a command before it runs | **the answer half is built** — `WhatACommandWillDo`; the asking is decided by ADR 0123, **accepted 2026-10-18**, and is buildable |

**Four of these rows were wrong on 2026-10-17 and all four in the same
direction** — they understated what was built, because nobody had drawn the
thing and looked. *Why is it waiting* and *output that is not stuck in a
character grid* both had a surface that drew its gutter word and no sentence,
and the box for *try a command without letting it touch anything* had been built
a day before the row saying it did not exist. **A row that reads worse than the
truth is as expensive as one that reads better**: it sends a lane to build what
is already there, and it hid two defects that a person would have met on the
first boot.

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

## Where the eleven stand, 2026-10-18

**Two are done and nine are blocked.** Task 11 was added on 2026-10-18 from the
owner's three rulings on
[ADR 0124](../decisions/0124-one-command-asks-alo-from-the-terminal-and-asking-means-acting.md)
and **written up as startable, which it is not** — corrected within the hour,
below, and left visible because the reason is a sixth cause this plan did not
have: **law 1 has no road out of a command.** The nine are blocked by one of six
causes, five of which are another lane's or the owner's:

| | task | blocked by |
|---|---|---|
| 1 | *why is it waiting* reaches a person | **done**, #755 |
| 2 | the source qualifier | **done**, by #761 |
| 6 | rich results | **the first view is done**, #761; the other two need a design component captured |
| 3 | explain a command before it runs | **built**, #805 — the owner chose the agent's road on 2026-10-18 |
| 7 | hand somebody a session to replay | **the comparer** — the third PC's; shell integration stopped being its other blocker on 2026-10-18 |
| 4 | find a command by what it did | **the verb is built**, #812 — the owner chose the surface on 2026-10-18; the file manager's half is a second surface |
| 5 | scope one command | **`--only-here` is built**, #818 — and `--offline` was already true of a boxed run. It never depended on `alo-capability` |
| 8 | bisect the machine | **the comparer** — the third PC's |
| 10 | *your files as they were* | **the comparer** |
| 9 | try a command touching nothing | **the kernel boundary** — the kernel plan's |
| 11 | ask alo where the person chose, not only here | **both blocks gone.** ADR 0125 settled law 1 out of a process; the owner settled the trade, and the setting and its Settings row are built (#820, #822). **What is left is law 1's egress sequence gaining a caller that is not a turn**, and then a request and a sender — see task 11, where *one request on the socket* was measured and found to understate it |

**So the five causes are:** shell integration (2 tasks), the privileged comparer
(3), a design component nobody has captured (part of 1), `alo-capability`'s
ownership (1), the kernel boundary (1), and one scope decision (1).

*Three of the five are named in ADRs as belonging to another lane or to the
owner. None of the eight is waiting on work this lane could do and has not.*

## Tasks

### 1. The answer to *why is it waiting* reaches a person

**Status:** done, 2026-10-17 — **and it drew nothing a person could read until
#778 the same day.** Report:
[`updates/the-answer-reaches-a-person.md`](updates/the-answer-reaches-a-person.md).
**Depends on:** nothing.

**What *done* did not cover.** The chord was bound, the five answers were built,
every test passed, and pressing `Ctrl`+`Alt`+`W` put `alo` in the gutter with
**nothing beside it**: the sentence was being drawn inside the gutter's own
80-pixel label box and painted over by the word. Three of this plan's four
conditions held and the feature did not work. It was found by drawing the
terminal on a compositor and reading the pixels back, not by a test — every
check counted drawn runs and passed, because both runs existed, one on top of
the other.

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

**Status, 2026-10-18 evening: the input half is built and the asking is one
decision away.** The owner chose the key that day — the shell's `Ctrl`+`X`
family — and `Ctrl`+`X` `Ctrl`+`A` is the pair, measured unbound in the bash a
person actually gets while `Ctrl`+`X` `Ctrl`+`E` is `edit-and-execute-command`.

**What is built and proven:**

| | |
|---|---|
| the reader | `crates/alo-terminal/src/what_the_shell_says_about_the_prompt.rs` — a second `vte` parser, term 7, `alacritty_terminal` untouched |
| the wiring | `AShellOnAPseudoterminal` feeds it the same bytes it hands the grid and drains through `what_the_shell_said()` |
| proven with a real shell | three tests start `/bin/sh` on a real pseudoterminal; the seam was mutated and cutting it fails exactly those |
| the display | `WhatAloSaid::explains()` and `WhatACommandWillDo` were already built (#759) |
| the snippet | `crates/alo-terminal/shell-integration/alo.bash`, bound to `Ctrl`+`X` `Ctrl`+`A`, held to **a real interactive `bash`** — three tests source it, press the key, and assert what alo reads back |

**So the input half is end to end and proven on a real shell.** A person with
the snippet installed presses the key and `rm -rf /important` reaches alo
**without being run**. That is three of the bar's four: the mechanism works,
something calls it, and a person can reach it by pressing a key. *Nobody has
seen it on a machine, because nothing draws it yet — which is the half below.*

**Two faults the real-shell tests caught that a reading would not have.** The
snippet first declared `${#line}`, which counts **characters**; `echo café` is
nine characters and ten bytes, so the reader's own length check refused it. It
would have worked perfectly for every ASCII user. And removing the `bind -x`
line fails both key tests while leaving the cycle-mark test green, which is the
shape of *declared is not dispatched*.

**What is left is one question: where does the explanation come from?** The
answer must carry **plain words and the three marks** — destructive, needs
elevated rights, leaves the machine — and the marks are what make the road a
choice rather than a detail.

- **Through the agent daemon**, reusing `TheAgentsDoor`, which already writes a
  line to a socket and posts an answer back. It holds the one live indicator and
  the record. **But it makes a person's request for an explanation an agent
  turn**, with grants and approval around what is a *read* — and
  [ADR 0125](../decisions/0125-a-command-does-not-report-an-egress-it-asks-a-service-that-can-show-one.md)
  marked that as the part most likely to be got wrong. It also posts back a
  plain `String`, so the marks would have to be parsed out of prose.
- **The shell asks a model directly** through `alo-asking`. It stays a read,
  *reads answer, changes wait*, and the compositor **does** hold the indicator
  so the egress is lawful. But it puts model-asking — settings, runtime,
  the key, the record — inside the compositor, and a model call is seconds in a
  process that must not block a frame.

**This lane's recommendation is the second**, because an explanation is a read
and a read should not acquire an agent's machinery. *Marked as a recommendation:
it is the owner's, and `CLAUDE.md`'s rule is that when the answer is not obvious
it is scope.*

**What that decision also settles** and should be answered with it: whether the
explanation says **where it was answered** — on this machine, on this network,
or with a provider. `docs/features.md` makes that a permanent area in the task
window, and a sentence drawn in the terminal is the same promise in a smaller
space.

**Depends on:**
[ADR 0123](../decisions/0123-alo-learns-what-a-person-typed-from-the-shell-not-the-screen.md),
proposed 2026-10-17 by this lane under the rule that a machine unblocks itself
rather than waiting for somebody with less context, and **accepted by the owner
on 2026-10-18**. It was *outside this plan* until somebody wrote it; that
phrasing was the blocker, not the decision. **This task is now buildable** — the
terms it rests on are 1 to 7, which are the accepted half.

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

**Status:** **the verb is built — #812, 2026-10-18.** **Depends on:** nothing;
the scope decision it waited on was made by the owner the same day.

The owner was asked where a person goes to ask *which command made this file*
— the record window, a question to alo, or the file manager — and answered
*"B and C"*: a question to alo, and the file manager later. So the agent needed
a verb, and `docs/contracts/agent-verbs.md` had none.

`what_touched_this_file` is that verb: a **read** over a **granted file**,
declared in `alo-record`'s `src/verbs.rs`, carried out by `alo-turn`'s
`asking_the_record`, offered by `the_verbs_a_whole_machine_offers`, and in the
published contract. The grant is required because the verb is the *agent's*
road to the record rather than the person's — a person reads the record in the
record window, which shows all of it — and an agent that could ask about any
path would learn whether a path appears in the record by asking about a guess.

**It never answers across a pairing**, enforced in `Arriving::reading` rather
than documented: the catalogue read crosses because the catalogue is identical
on every alo machine, and the record is the opposite.

**Two sentences no shape of the answer can omit**, which is the feature's own
*where nothing connects them it says so*: *this is what alo did, commands you
ran yourself are not in it*, and, where nothing names the file, *nothing in the
record names it* instead of an empty list. The first is load-bearing — see the
finding below, which was already in this section and is why.

**What is still owed under this task:**

- **the file manager's *what made this?***, which is the owner's *C* and a
  second surface. Not started;
- **a shortened record answers narrowly and invisibly.** Entries older than the
  keeping rule are gone, and nothing records that a shortening removed
  something matching, so the answer cannot say so. Stated in the contract and
  beside `Done::left_something_out` rather than left to be discovered.
  **Closing it is a change to `alo-keeping`, which is not this lane's**, and it
  is reported here the minute it was found;
- **nothing a person can press yet.** The verb is reachable by the agent, which
  is what *a question to alo* means — a person asks alo and alo calls it. That
  road exists. What has not been watched is a person doing it on a machine,
  which is the fourth condition and the one a green suite says nothing about.

*The paragraph below is the section as it stood while this was blocked, kept
because its finding is still true and is the reason for the first of the two
sentences above.*

**Status, as it read until 2026-10-18:** **blocked on a scope decision.**
**Depends on:** which surface asks the question — outside this plan.

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

### Tasks 5 and 9 are one surface, and three of the owner's five wait on one answer

*Found 2026-10-18 while costing task 9's surface, after the owner decided it.*

**`try`, `--only-here` and `--offline` are the same thing with different
restrictions.** `docs/features.md` draws all three as a **prefix** on a command
a person runs, and says why in its own words: *a process cannot credibly
restrict itself — the kernel has to, which is why no other terminal offers this
as a prefix.* So they are one surface asking the kernel for three different
boundaries, not two tasks.

**And all three run through `alo_running::WhereCommandsRun`, which records every
run before it runs.** `new(boundary, boxes_live_in, record)` takes the record,
and `ran` opens with `checked_and_written_down`. That is ADR 0001 §7 —
*everything executed is a record with an origin* — as working code, and it is
not optional.

**So the question is who writes that record**, and it is the same question
[ADR 0125](../decisions/0125-a-command-does-not-report-an-egress-it-asks-a-service-that-can-show-one.md)
asks about an egress. A command has the code to append one
(`alo_keeping::Writing`) and not the permission, because the file is under
`/var/lib`. Either a prefix command runs with privileges, or it asks a service
that has them — which is exactly what that ADR proposes, and it is **proposed and
not accepted**.

| the owner's decision | waits on |
|---|---|
| **1** the explanation comes through the agent | nothing — built |
| **2** a question to alo asks the record | an addition to `docs/contracts/agent-verbs.md` |
| **3** `--only-here` scopes one command | **ADR 0125** |
| **4** `try` runs a command in a box | **ADR 0125** |
| **5** a stand-in when the chosen model cannot be reached | **ADR 0125** |

**Three of five wait on one answer.** Accepting or rejecting that ADR is worth
more than any of them individually, and it is why this is written down rather
than built around.

**What task 5 said was its blocker is not one.** It read *depends on
`alo-capability`, which is another lane's*, and under it that giving a grant a
level *needs a third `Row` variant*. **The level is ADR 0111's and belongs to
order item 8, not here.** A per-command narrowing needs neither: `Grant` already
carries a `Reach` and an expiry, `Reach::Folder` is one of four, and
`Reach::File`'s own comment names this exact pattern — *a grant with a very
short life rather than a special case (ADR 0001 §4)*. **The mechanism is there;
what is missing is the record's permission.**

### 5. Scope one command without changing a setting

**Status:** **`--only-here` is built — #818, 2026-10-18. `--offline` was
already true and needs no code.** **Depends on:** nothing. It never depended on
`alo-capability`.

One command run under a narrower bound than the person's session has, without
touching what they chose for everything else. `try` is the word (#817) and these
are its narrowings: `try --only-here ./build.sh` may work in the folder the
person is standing in and nowhere else.

**`--offline` was already the behaviour of a boxed run, and nothing had said
so.** `alo_bounding_map::Bounds` holds where a turn may connect to and *begins
as nowhere* — default-deny. Measured on 2026-10-18, the same `curl` to an IP
address with one difference between the runs:

| | |
|---|---|
| unboxed | `http=301`, exit 0 |
| in a box | `http=000`, exit 7 — could not connect |

So a flag for it would be a flag that does nothing. **What the promise asks
beyond this is *and the indicator shows that it did not***, which is the
compositor's surface and is not reachable from a command — the same gap ADR 0125
settles for an egress that happens, pointing the other way. That half is still
owed and is the owner's road rather than this one's.

**And `--only-here` is a loosening**, which the flag's name does not suggest:
`try` alone grants the command and nothing of the person's, and this adds their
folder. It is also **stricter than the promise says** — the promise is *may not
write outside this folder*, and a place in `alo-bounding` is read and write
together, so what it gives is *may not touch outside this folder*.

### What this status said until 2026-10-18, and why all three parts were wrong

> **Status:** **blocked.** **Depends on:** `alo-capability`, which is another
> lane's — and ADR 0111 leaves whose change it is to the integration owner, in
> its own words, *and that question was never answered*.

Kept because the reasoning is the instructive part, and because the owner was
asked for a permission on the strength of it.

| it said | measured while building it |
|---|---|
| depends on `alo-capability` | **it does not.** A person running their own command needs no grant — law 2 binds the agent, never the person — so the narrowing is `alo-running`'s bound. **Nothing in `alo-capability` was touched** |
| `alo-capability` is another lane's | it is **this lane's**, through `applications-and-what-they-expect-plan.md` in `a-new-machine-becomes-a-lane.md`'s crate table. The real objection in ADR 0111 is different and better: *a lane does not widen its own partition* |
| ADR 0111 leaves whose change it is | **ADR 0111 is *the person chooses whether their agent may use the terminal*** — a different promise, row two of the thirteen. Its `alo-capability` item is the terminal grant. The two were conflated |

**The owner was asked to approve widening `alo-capability` and said yes on
2026-10-19. The permission was not needed and was not used**, which is recorded
here so that a later reader does not take this task as the precedent for one.

*This is the same shape as task 4's correction the same day: there, the status
listed *a verb the agent calls* among roads that cannot be taken, under* there is
no such verb *— a true statement about `main` and not a reason. Here, *depends on
`alo-capability`* was a true statement about a different ADR's work. **A status
that names a dependency is making a claim, and a claim is worth measuring before
a lane spends a day on it or an owner spends a decision.***

### 6. Output that is not stuck in a character grid

**Status:** **the first rich view is built (#761), and its two lines were
cropped into the gutter's label box until #778** on 2026-10-17 — the same fault
as task 1 and found the same way. **Depends on:** nothing in
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

**Status:** **blocked on the comparer, and on keeping what it reads.**
*This said two causes on 2026-10-17 and one of them went away on 2026-10-18.*
**Depends on:** the privileged comparer — and, separately, somewhere to keep a
person's own commands, which the bullets below separate from knowing them.
Shell integration was the other cause and is built: ADR 0123 was accepted, the
reader landed in #798 and the snippet in #800. Neither is in this plan,
and task 6 is no longer the obstacle. The commands come from that decision;
*what each one changed* is
[ADR 0045](../decisions/0045-what-undoing-rewinds-to.md)'s bracket.

A transcript somebody else can step through. Task 6 was the recorded
dependency and it is built, so this read as the next startable task. It is not,
and the promise says why: *not a transcript — the commands **and what each one
changed**, replayed on their machine.* Both halves are blocked, and each by
something another task here is already blocked by:

- **the commands — no longer blocked, 2026-10-18.** This read *alo does not know
  what a person typed, task 3's blocker*. It does now: the reader is in `main`
  (#798) and the snippet a person installs is #800, so a shell with it sourced
  sends the typed line and `alo-terminal` reads it back. *What stood here is kept
  below, because the measurement is still true of the **record** and that is a
  second gap this task has:* `alo-terminal` does not depend on `alo-record`
  (**0** mentions in its manifest), no terminal file in `alo-shell` names
  `alo_record` (**0**), and `Happened::RanACommand` **always carries an
  `agent`** — it is a command an agent ran, under whose authority, by
  construction. **A person's own typed command is still recorded nowhere**, so
  knowing it and keeping it are different, and this task needs both.
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

### 11. Ask alo where the person chose, and not only this machine

**Blocked twice, and both blocks are the finding.** One is law 1, below. The
other was found by running the command rather than reading it, and it is not a
missing feature at all: **what it does today is answer somewhere the person did
not choose.** *This was written up as startable on
2026-10-18 and corrected the same hour. The first draft is quoted because the
mistake is worth more than the correction:* **nothing in this task waits on
anybody — this one is a caller that does not exist for doors that do.** Two of
the five duties below cannot be discharged by any command, and I listed them as
duties without asking whether a road to them exists.

### What a person gets today is worse than a missing feature — run, not read, 2026-10-18

**It answers somewhere they did not choose, and does not say so.** A settings
file naming a provider exactly as `docs/contracts/person-settings.md` documents
it:

```toml
[answers]
provider = { name = "Mistral", model = "mistral-small-latest" }
```

```text
$ alo-ask may the tenant sublet
"May the tenant sublet" is a bit of a colloquialism, but more commonly…
on this machine
```

The only model installed on that machine is `smollm2:1.7b`. **So a person who
asked for `mistral-small-latest` from Mistral was answered by a 1.7B local
model and was never told their choice was not used.** *on this machine* is the
only hint, and it reads as confirmation.

[ADR 0008](../decisions/0008-where-inference-happens.md) is explicit: *a hosted
API is a supported first-class option and **never a silent fallback***.

**The sharpest evidence is a contrast.** `alo_choosing::what_answers_here` is
the function that answers *what answers questions on this machine*.
`alo-desktop` calls it. **`alo-ask` never does** — grepped across `crates/`, the
only callers are `alo-choosing` itself and `alo-desktop`.

**With no local weights, or several, it refuses instead** with *choose a model
for this question to be answered by* — also false, since the person did choose.
*That path is **read** in `main.rs` and `the_only_one`'s two arms and was not
run; the one above was run.*

**No existing sentence can carry the honest refusal**, checked rather than
assumed: `models.policy.not-this-machine` says *this machine is set to answer
only on itself*, which would be a **second falsehood** — the restriction is a
constant inside `alo-ask`, and `THE_STRICTEST_RULE`'s own doc says it *is not a
reading of the organisation's policy and does not pretend to be one*.
`answering.wrong.sent-somewhere-else` is about an HTTP redirect.

**And the fix is the owner's, because refusing takes something away.** A person
with a provider configured and one local model installed gets answers today;
refusing would stop that, and law 5 says *the person chooses; alo never takes
the choice away*. So this is a trade between ADR 0008's *never silent* and law
5's *never withdraw*, and the three options are: refuse and say so; answer
locally but say plainly that the chosen provider was not used; or leave it until
the provider road exists. **The second takes nothing away and is this lane's
recommendation**, and it still needs a sentence nobody has written.

*The crate documented this behaviour as correct — "nothing is silent about
typing a command named for the place it asks" — and that argument was corrected
in #793 before this was measured: the command is called `alo-ask` and names no
place.*

### The transport, and what it is missing

**What a person gets today**, measured 2026-10-18: `alo-ask` holds
`SourcePolicy::ThisMachineOnly` and calls `Asking::to_this_machine` and nothing
else. A person whose settings name a provider is answered by **nothing at all**,
and since
[ADR 0095](../decisions/0095-the-release-carries-no-model-and-a-person-brings-their-own.md)
ships no weights, that is the ordinary state of a fresh machine rather than an
edge case. `docs/features.md` promises *a provider they have an account with*,
and [ADR 0008](../decisions/0008-where-inference-happens.md) says outright *it
assumed everybody runs a model. Most people will not.*

**This makes an existing promise true rather than adding one**, which is the
sentence `CLAUDE.md` asks for: the three places are ADR 0008's and already
published, and the owner ruled on 2026-10-18 that all three may answer and never
local only. No line of `docs/features.md` gains a capability here; one stops
being false on a machine without weights.

**The road is built. The caller is not** — and since ADR 0125 was accepted on
2026-10-18, **the caller is `alo-agentd` rather than `alo-ask`.** That is the
single most important thing on this page to read before building it: the table
below said *by whom* and meant the command, and five duties later it turned out
the command could discharge only two of them.

`alo_choosing::Settings::provider`, `alo_choosing::what_answers_here` and
`alo_asking::Asking::to_a_provider` all exist, and `alo-agentd` already drives
the last of them — `doing.rs` has a test that reads the person's settings, asks
a real keyring for their key, and opens a connection *to the address they wrote
down and to nothing else*.

| owed | by whom, after ADR 0125 |
|---|---|
| resolve the address and register it before the boundary | **the daemon.** `to_a_provider`'s own doc: *resolved and registered by the caller before any boundary was entered* (ADR 0020) |
| the key out of the keyring | **the daemon**, which already does it. `docs/features.md`: *the key goes to the keyring, never into a settings file* |
| show the egress **at the moment it happens** | **the daemon**, which holds the one live indicator. A command cannot: see the section below |
| keep the entry | **the daemon.** ADR 0125's own line: *that service shows it **and keeps the record*** |
| **never fall back** | **both, differently.** The daemon does not try somewhere else; and what `alo-ask` does when the chosen place is unreachable is now the person's setting — `[when-mine-is-unavailable]`, built 2026-10-19 |

**So what `alo-ask` must become is much smaller than this page first said**: it
sends the person's question to their own daemon socket when their settings name
a provider, and shows what comes back. `FromAPerson` has fourteen variants and
not one of them is *ask this*.

### And *one request on the socket* understates it — measured 2026-10-19

*That sentence was written an hour before this one, in the same change, and it
is the claim this page has just finished warning about: **a status that names a
dependency is making a claim.** So it was measured.*

The request and the command's half really are small. **The work is that law 1's
sequence has no caller that is not a turn.**

| piece | state |
|---|---|
| the protocol request, and `alo-ask` sending it | not written, and small |
| the daemon reading the person's settings and reaching their provider | **mostly built** — `alo-agentd`'s `doing.rs` has a test that asks a real keyring for their key and connects to the address they wrote down and nothing else |
| **a non-turn caller's road to law 1's sequence** | **does not exist, and is the task** |

**What that sequence is**, in `alo-turn`'s `asking.rs`: take the boundary and
the indicator together; impose a boundary permitting the one resolved address;
put the question *inside* it, through a closure that is run once because one
permission is one attempt; write `Entry::left` **before** the line comes off the
indicator; then end the departure. Its refusals — no boundary, a boundary that
ran nothing, held back, left and unanswered — are five more entries, spread
across six interlocking `Turning` methods that also set the turn's `closed` flag
and remember a lost thread.

`Machine::bounding_and_indicator` is `pub(crate)`, so none of it is reachable
from `alo-agentd` today.

**So the shape of the work is: put that sequence on `Machine`, where the
indicator and the record already live, and have `Turning` delegate to it** —
one copy, two callers. **Not two copies.** `put_off_this_machine`'s own doc
names the failure mode: a call that compiles, with the right types, while
nobody sees anything. Two copies of law 1's dance is how one of them quietly
stops showing.

*Nothing here changes what ADR 0125 decided. It changes the estimate, and it
names the file the next person should open first.*

### A second sequence has to be shared too — measured 2026-10-19, by starting it

**The egress sequence is not the only thing a person's road must share.**
Measured by building the protocol half and then reading what the daemon would
have to do next:

`alo-agentd`'s `doing.rs` prepares a provider question in a **deliberate
order**, and its own comments say why each step is where it is:

1. **the organisation's rule first** — *a question the organisation's bound
   refuses is refused before the keyring is opened, so a policy refusal never
   reaches for the person's credential*;
2. **then which way out of this machine** — *a question with no road to go on
   never reaches for the person's credential*, and a road that could not be
   decided is refused rather than quietly going straight out, because *on a
   company network that would be a machine sending its company's traffic
   around the company's own rule with nobody told*;
3. **then the key**, from the keyring, by the name `SecretRef` holds;
4. then `Hosted::taking`, and the ask.

That is eighty-odd lines with **four distinct refusals**, each a different thing
for a person to go and do. A person's road needs the same four in the same
order — so it must **share** them, for the same reason the egress sequence is
one copy: two orderings of *rule, road, key* is how one of them one day reaches
for a credential to answer a question a rule had already refused.

**So the task is two extractions and then three small pieces**, in this order:

| | |
|---|---|
| 1 | law 1's egress sequence — **done**, #825 |
| 2 | `doing.rs`'s provider preparation, shared rather than copied — **done**, `alo-agentd/src/a_provider_ready_to_be_asked.rs` |
| 3 | the protocol message, the daemon's handler, `alo-ask` sending it — **the remainder**, and the whole of it |

*The protocol message was written on 2026-10-19 and **discarded unlanded**: a
message nothing sends is the fault this plan keeps finding, and the honest place
to stop was before it. It is rewritten with its sender and its handler, together,
or not at all.*

**The ordering is now a test and not a comment.** Five tests in that file, two of
them written by mutating the order and watching: asking the key first fails
*a rule refuses before the keyring is opened* and *no road out refuses before the
keyring is opened*; asking the road before the rule fails *a rule refuses before
a road is asked for*, and nothing else. The instrument is
`WhoseKeyring::Nobodys`, whose every lookup is `NotStored::Unavailable` — so a
credential reached for out of turn shows up as **the wrong refusal** rather than
as nothing at all, which is the only way a question about ordering can be asked
of code that refuses either way.

**The refusal paths are the work, not the happy path.** A provider configured
with no key, a key the provider rejects, an address that will not resolve, an
`http` address that is not on this machine (`docs/features.md` refuses rather
than warns), and a settings file naming a provider on a machine with no network.
Each is a different sentence to a person and none of them may be *answered
locally instead*.

### Why law 1 cannot be satisfied from a command — decided, not blocking

*This heading read **What blocks it** until 2026-10-19. It is not a blocker any
more: the owner accepted ADR 0125 and the answer is that the command asks the
daemon. The measurement is kept because it is the whole argument for that
answer, and because the next person-facing command that wants to cause an
egress will meet the same wall and should find out why here.*

**Measured 2026-10-18, after the table above was written.** Two of those five
duties are law 1 itself — *visible at the moment it happens and afterwards in a
record* — and a command can discharge neither:

| asked | answer |
|---|---|
| what is `alo_egress::Indicator` | a `Vec<Shown>` and a counter, **in process memory** |
| who turns one into something a person sees | `alo-indicator` makes a `Drawn` from one, and `alo-shell` draws it on an output |
| is there a road for another process to report an egress | **no.** `docs/contracts/native-egress-indicator.md`: *no protocol, agent capability, D-Bus interface or stored format changes … this document describes only the drawing surface* |
| is there a road to the record file | **yes — the code and the permission both.** `alo_keeping::Writing` implements `SomewhereToWrite` and appends one line at a time. *This cell has now been wrong twice in one day.* It read “no road a command may use”, which would have sent somebody to build an API that exists; then “the permission, no”, which is false on the shipped image: `d /var/lib/alo 0700 alo alo` and `u alo 1000 "alo OS" /home/alo /bin/bash` — **`alo` is the person's login**, and `alo-agentd` and `alo-desktop` run as it, so the service and the person are one account. **What a run actually needs settled is which file it appends to**, because `docs/contracts/record-file.md` says a record file has one writer — and that contract's own answer to a new writer is a new file (ADR 0125) |

So an `alo-ask` that reached a provider would create its own `Indicator`, show
the egress to itself, and drop it when the process exited. **That is a law 1
violation wearing the API that exists to prevent one** — the call compiles, the
type is right, and no person sees anything.

**This is larger than this task.** Law 1 is satisfiable today only inside the
compositor and the agent daemon. `alo-ask` is the first person-facing command in
this product that would cause egress, and the first to find that out. Every
later one meets the same wall: *the command that fetches a model, the command
that tests a provider, anything a person types that leaves this machine.*

**What was needed was a decision, not a caller:** how a process that is neither
the compositor nor the agent daemon reports an egress and keeps an entry. **The
answer is that it does not** — it asks one that can.

**That ADR is now accepted — [ADR 0125](../decisions/0125-a-command-does-not-report-an-egress-it-asks-a-service-that-can-show-one.md),
by the owner on 2026-10-18 — and writing it changed the answer.** It needs **no
new public surface on the indicator**, because the precedent already exists and
is load-bearing: `alo-agent` is a short-lived process that needs a model, names
`Indicator` zero times across 7 files and 1070 lines, does not depend on
`alo-egress` at all, and **asks the daemon over a socket** instead. The daemon
holds the one live indicator (`alo-agentd/src/starting.rs:250`, in
`pub fn until_stopped`, whose doc calls it *the single one law 1's surface is
drawn from*), and its socket is already one per signed-in person.

**And a *report your egress* interface turns out to be refused by design rather
than merely missing.** `alo_egress::Indicator::beginning`: *the two are one call
because they must not be two … the `Departing` that comes back is what a caller
must hold to open the connection, and it is the only thing that means so.*
Permission to open the connection **is** what the indicator hands back, so
reporting afterwards would split the permission from the showing — the one
failure the type was shaped to refuse. *`native-egress-indicator.md` saying it
adds no protocol was read here as a gap. It is a decision, and ADR 0125 agrees
with it.*

*Nothing in the local road is affected: it causes no egress, which is why
`alo-ask` works today and why the gap went unnoticed until something had to
leave.*

**What this task must not do:** choose a place on anybody's behalf, or widen
`THE_STRICTEST_RULE` for the local road. The rule exists so this command cannot
be where a policy gets loosened; the provider road needs its own policy from the
person's settings, not a relaxation of that constant.

*Added 2026-10-18 by the Mac lane, from ADR 0124's consequence that `alo-ask`
grows a road it does not have. Two false sentences in `alo-ask`'s own crate
documentation were corrected in the same change — they claimed where a record
lives is undecided, and that ADR 0008 permits one door — because a justification
left in place is how a gap stops being read as one.*

## What this plan is waiting on, in one place

- ~~**Whether a command the person runs is bracketed**~~ — **answered
  2026-10-19 by the owner**, who accepted
  [ADR 0127](../decisions/0127-a-persons-own-commands-are-bracketed-in-a-session-they-asked-for.md):
  **in a session the person asked for, every command they run is bracketed**, and
  an ordinary session takes no snapshots. *Every command* was not taken because
  alo cannot tell a changing command from a reading one, so it would snapshot
  `ls`, and ADR 0045's fifty-turn window would be spent before lunch. The starred
  promise in `docs/features.md` is narrowed to match, in the same change.
  **Task 10 is now *decided*, not buildable** — the lane said buildable and
  measured it an hour later. The comparer is genuinely not needed (a bracket
  snapshots the whole home, so *your files as they were* is a whole-home rewind),
  but **nothing in this product can put a home back**: the repository's only two
  `btrfs` invocations are `subvolume snapshot -r` and `subvolume delete
  --commit-after`, in 122 crates. And the hard part is *when* — a home cannot be
  swapped out from under the person living in it. ADR 0127 carries the three
  shapes and they are the owner's. **Both answered the same day**: a Settings row
  under Terminal, and *a folder now, the next sign-in next, never a root helper*
  — so task 10 **is** buildable, in that shape, and it is this lane's next task.
  Tasks 7 and 8 are unaffected and still the comparer's.
  **Tasks 7 and 8 are not**: both need what each command *changed*, which is the
  comparer's, and the comparer is the third PC's (ADR 0045, and `alo-agentd`
  records the gap in its own words). *This bullet said "a bracket" until the
  bracket arrived and the tasks stayed shut; it then said "a decision" until the
  decision arrived and two of the three stayed shut anyway.*
- **How a person asks for a sealed-box run** — task 9. **Not the box**: that is
  built (#729, 2026-10-16) and passes on a real kernel. What is missing is a
  caller and the surface it would be called from, and the design file and ADR
  0064 are both silent on the surface. *This bullet said "the kernel box" and
  pointed at two ADRs that had already been built against.*
- **How a process that is not the compositor reports an egress and keeps an
  entry** — task 11, and every later command that leaves this machine. Law 1 is
  satisfiable today only inside the compositor and the agent daemon:
  `alo_egress::Indicator` is a `Vec` in process memory and
  `docs/contracts/native-egress-indicator.md` adds *no protocol, agent
  capability, D-Bus interface or stored format*. A public surface, so an ADR
  comes before a caller. *Found 2026-10-18, in the hour after task 11 was
  written up as startable.*
- ~~**Who the indicator and the record name when the person asks**~~ —
  **answered 2026-10-19 by the owner, and task 11 is unblocked.**
  [ADR 0126](../decisions/0126-a-persons-own-question-is-a-third-authority-on-the-indicator-and-in-the-record.md)
  is accepted with the lane's proposal **rejected**: a person's own question
  shows **no** indicator line, because they chose the provider in Settings. Law
  1's body scopes itself to *every network egress an agent causes*, and
  `docs/features.md` made the promise in that same scope — so nothing is
  withdrawn and nothing has to be built. **What stays is the organisation's
  rule**, which is ADR 0016 rather than law 1 and is already built and already
  asked first (#826). What follows from it: the daemon answers the person
  **without a turn and without an indicator**, which was the whole difficulty.
  **ADR 0125's route stands** — the lane first wrote that `alo-ask` could ask
  the provider directly and then measured it: the organisation's rule is read by
  1710 lines inside `alo-agentd`, which nothing depends on, and a proxy that
  asks for a password is a systemd credential only that unit may read. Five
  pieces are left and they are listed in the ADR.
- **The owner, on the pairing of five answers to four actions** — `docs/features.md`
  records it as theirs. Task 1 does not need it; offering the actions does.
