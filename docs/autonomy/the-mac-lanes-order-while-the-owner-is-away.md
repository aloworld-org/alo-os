# The terminal, finished and usable — what that takes

**Written:** 2026-10-17, at the owner's request before a period away.
**The goal, in the owner's words:** *all the tasks of the terminal are done, and
all the features we talked about work end-to-end and users can use it.*

**So "done" here does not mean the change landed.** It means a person sitting at
an alo OS machine can do the thing. That is a higher bar than a green gate, and
it is the bar this document keeps: every row below says **what a person does**,
and what is still between them and doing it.

## What that bar rules out, and it is the thing to read first

A terminal feature is usable when four things are true at once:

1. **the mechanism works** — tested, measured, with its refusal path;
2. **something calls it** — the fault this repository has found in five shapes,
   most recently six terminal mechanisms gated green and visible to nobody;
3. **a person can reach it** — a chord, a control, something they could find;
4. **somebody has seen it on a machine.**

The fourth is the one a green suite says nothing about. **A machine does boot** —
one booted from `06c1444f` on 2026-10-13 and installed all seven applications,
read by the desktop lane on the running machine — so this is reachable rather
than hypothetical. What has **not** happened is anybody opening alo's own
terminal on a booted machine and using it.

*`docs/autonomy/evidence-a-person-can-work-on-it-all-day.md` says "Nothing has
booted the image" in three places and has been wrong since the evening of
2026-10-13. It is not this lane's document to rewrite, and it is named here
because a reader starting from it would conclude this goal is unreachable.*

## The thirteen, against that bar

★ marks a differentiator. **Usable** means all four conditions above.

| | what a person does | state |
|---|---|---|
| | opens a terminal and types | **usable** — a chord opens it, the compositor draws it, typing reaches the shell |
| ★ | decides whether alo may use it | built as types; **nothing stores either choice**, so not reachable — and **blocked**, not merely unbuilt |
| ★ | reads who is speaking | **usable** — `you`, `alo`, `system`, `program` in the gutter |
| ★ | sees measured apart from worked-out | **usable** — the same gutter's `system` against `alo` |
| ★ | asks why a command is waiting | **built and reachable** as of #755, `Ctrl`+`Alt`+`W`. **Never seen on a machine** |
| ★ | scrolls back through output | **usable** |
| ★ | runs a command touching nothing | the runner and record exist; the kernel box does not — **not usable** |
| ★ | scopes one command | **blocked** — a per-command narrowing is `alo-capability`, another lane's |
| ★ | finds a command by what it did | the record can answer it; **nothing can ask it**, and giving it a surface is scope |
| ★ | bisects the machine | **nothing**, and waits on a bracket |
| ★ | reads output that is not a character grid | **the first rich view is usable** — a test result, #761. Two more are unspecified in the design |
| ★ | hands somebody a session to replay | **nothing**, and waits on rich results |
| | asks what a command will do first | the answer is built (#759); **nothing can ask**, because alo does not know what was typed — **blocked** |

**Where the thirteen stand at the end of 2026-10-17**, each row read rather
than counted by a pattern — a first pass with a crude classifier misread two of
them, which is this document's own lesson about mentions and declarations:

| | how many | which |
|---|---|---|
| **usable** — all four conditions | **4** | opens a terminal and types; reads who is speaking; sees measured apart from worked-out; scrolls back |
| **built and reachable, never seen on a machine** | **2** | asks why a command is waiting (#755); reads output that is not a character grid (#761) |
| waiting on this lane's own earlier steps | **2** | bisects the machine, behind a bracket; hands somebody a session to replay, behind the rest of rich results |
| **blocked on a decision or another lane** | **4** | decides whether alo may use it; scopes one command; finds a command by what it did; asks what a command will do first |
| unowned | **1** | runs a command touching nothing |

**The two in the second row are what tomorrow evening's boot is for.** Both are
built, both are reachable, and nobody has watched either on a machine — which is
the one condition a green suite says nothing about and the reason step 1 of this
order is a boot rather than a task.

*Four of thirteen were usable this morning and four still are. What changed is
that **one promise gained a working view** and **four were found to be blocked
rather than merely unbuilt** — which is worth more than it sounds, because a
lane that believes four blocked things are startable spends four days finding
out.*

## The order, and it is ordered by that bar rather than by size

### 1. See the five answers on a booted machine — **the boot is 2026-10-18, evening**

**The smallest step to the largest change in confidence.** #755 made *why is it
waiting* reachable; nothing has watched a person use it. One boot, one terminal,
one `cat`, one chord — and either it says *waiting for the keyboard* on a screen
or it does not, and the second is a finding worth more than the next three tasks.

**The owner said on 2026-10-17 that the boot is tomorrow evening.** So this is
scheduled rather than a thing to ask for, and the only question left is whether
the walk is written down before the machine is on — because an unprepared boot is
a boot spent finding out what to look at.

**The walk, in order, so none of it is worked out at the keyboard.** Each step
says what counts as a pass, and each is a thing a person does rather than a
command a lane runs:

| | do this | a pass is |
|---|---|---|
| 1 | sign in | a desktop, and the sign-in screen gone |
| 2 | press `Ctrl`+`Alt`+`T` | a terminal window, with a prompt |
| 3 | read the gutter beside the output | the word `program`, lowercase, in its own column |
| 4 | read the band under the transcript | the shell's name and `UTF-8` — e.g. `bash · UTF-8` |
| 5 | type `cat` and press Enter | nothing happens, which is the point: it is waiting |
| 6 | press `Ctrl`+`Alt`+`W` | **a line saying *waiting for the keyboard*, with `alo` beside it in the gutter** |
| 7 | press `Ctrl`+`D` to end `cat`, then `Ctrl`+`Alt`+`W` again | the line **goes**, because there is nothing waiting now |
| 8 | type `sleep 30`, Enter, then `Ctrl`+`Alt`+`W` | *nobody can tell* — a clock is none of the five, and this is the honest answer rather than a wrong confident one |
| 9 | scroll up with the wheel, then back down | the transcript moves and stops at both ends |

**Step 6 is the measurement.** Steps 3, 4 and 9 are things believed to work that
nobody has watched either, so they cost nothing to read while the machine is on.
Step 8 is the one most likely to be wrong, because it is the answer a careless
table gets wrong — a first version of that table answered *waiting for another
program* about `sleep`.

**Any step failing is worth more than the rest of this document.** What to send
back: which step, what was on the screen, and `journalctl -b 0 -u alo-compositor`
— because a drawing fault and a logic fault look identical from a chair, and the
journal separates them.

**And the chord may not be bound yet**: #755 is gating at the time of writing. If
it has not landed, **walk step 6** does nothing and that is correct rather than a
fault — check `main` first and skip to walk step 7.

*Numbered twice in this document on purpose, and so said twice: the walk above
has steps and so does the order below. A bare "step 6" could mean either, and one
cross-reference in this file was already stale for exactly that reason.*

### 2. The bracket, if it is still nobody's — **what three blocked tasks wait on**

**The source qualifier was step 2 for about an hour, and it was already done.**
Task 2 of the plan asks for a caption inside the transcript reading *Source:
cargo test output · Not independently verified*; #761 draws exactly that
sentence, in exactly that place, from the same frame of the design. Satisfying
task 6 satisfied it outright, and nobody read it before promoting it. *A task
whose whole content is one sentence of another task's output is a task to check
off rather than to schedule* — and this is the fourth time in one day that this
document promoted something it had not read, which is the fault its own
headline lesson is about.

**So the bracket is next, and it is next because three things wait on it**:
*bisect the machine*, *your files as they were*, and the *what each one
changed* half of a replayable session. ADR 0045 puts it in `alo-turn` and #752
settled what the agent may do with one — *take a bracket, and neither alter nor
destroy one*.

**#751 removed the first thing in the way** and **#752 recorded the measurement
it rests on**, both landed today.

**Who owns it: this lane, measured rather than assumed.** ADR 0045 says
`crates/alo-turn` is *lane A's*, which is retired naming, and the charter lists
`alo-turn` only for `the-local-network-plan`'s **pairing** parts. So the
charter's own method 2 was applied — every recent landing in `crates/alo-turn`
resolved to its branch — and **six of six are `task/mac/…`**: #750, #682, #679,
#670, #667, #662. The bracket is not pairing, and it is this lane's.

**Everything it needs is in place, and this is the first step all day that is
unambiguously startable:**

| | |
|---|---|
| the decision | ADR 0045's terms, amended 2026-10-16, plus #752 — *the agent can take a bracket and can neither alter nor destroy one* |
| the privilege | **none needed.** `btrfs subvolume snapshot -r` of a subvolume the person owns, as the person with no `sudo`, **exits 0** — measured twice, 2026-09-21 and 2026-10-16 |
| the folder | **made at first sign-in**, by `alo-giving-a-folder`, this lane's own work of 2026-10-17 (ADR 0120 moved it, ADR 0121 wired the caller). ADR 0045's amendment recorded that *nothing creates it at all*; that is no longer true |
| the layout | `alo-kept-undo` — `THE_TURN`, `BEFORE`, `AFTER`, `BOTH_SNAPSHOTS`, `Theirs`, `TheTurn`. **#751 moved it out of `alo-letting-go` for exactly this**, because nothing a turn runs inside may ship that crate |
| the remover | `alo-letting-go`, and it is measured on a real loopback `btrfs` in `tests/on_a_real_btrfs_machine.rs` |
| how to invoke it | the base's own program, as the remover does: one `Command::new(THE_PROGRAM)`, with a source-reading test asserting there is exactly one |

**So both ends are built and the middle is absent**, in ADR 0045's own words.

**The two things to settle before writing code, and neither is large:**

1. **Where a turn begins.** The caller is what makes this real rather than a
   third uncalled module — and `turning.rs` has no `pub fn began`, so the entry
   point has to be found rather than assumed. *This is the only reason the
   bracket was not started on 2026-10-17: the taker is an afternoon, and a taker
   nothing calls is what this lane declined to land twice that day.*
2. **What happens to a turn whose bracket cannot be taken.** ADR 0045 does not
   say. ADR 0015 says of its own boundary that *a turn whose boundary cannot be
   applied does not run*, and `CLAUDE.md` forbids a silent fallback — so the
   candidate answer is that the turn refuses and the person is told, but it is a
   decision and not an inference.

### 3. What is still owed on rich results

**The first view landed as #761**: a test result, which is the one view the
design specifies. What is left under that task, in the order it is worth doing:

- **the three `Ask alo` actions** the design draws under the summary. They are
  the agent's road, which exists (`what_a_person_asked_goes_to_the_agent.rs`)
  but addresses a task window rather than a terminal, so this needs a *who
  asked* that can be either;
- ***a thing on your desk that outlives the scroll*** — the promise's own
  words, and a window rather than a line. A promise of its own;
- **the benchmark and the diff**, which are **not specified anywhere**:
  `479:29079` and `479:29140` are instances of components no snapshot in this
  repository contains. These cannot be built from what is written down, and
  that is a question for the owner rather than a gap to fill by invention.

### 4. Then bisect the machine, and *your files as they were*

Both wait on a bracket and on nothing else, so **taking the bracket step above
makes these two this lane's** — which is why the corrected table below no longer lists them as
unfinishable. If the bracket turns out to be another lane's, these go with it.

### 5. Explain a command before it runs — **blocked on shell integration, measured 2026-10-17**

**This was step 2 of this order for part of 2026-10-17, on the plan's word that
it was the cheapest of the remaining six.** #759 built the answer and found the
asking impossible. The plan priced the asking and assumed the input was free.

**alo does not know what a person has typed, and the three roads to it are each
a decision:**

| | |
|---|---|
| anything keeping the typed line | **nothing** — `a_person_typed` writes the bytes to the pseudoterminal and keeps none |
| `the_command_in_front` | a process id, so `/proc` describes something **already running** — not a command before it runs |
| shell integration | **none**; the grid is `alacritty_terminal` with a `VoidListener`, which that module already calls *nothing is listening* |

- **read it off the screen** — decides what the *person* typed from what a
  *program* printed, which is `nothing_reads_the_bytes_to_decide_who_spoke`'s
  fault one storey up, and then sends the result to a model;
- **keep our own copy of the forwarded bytes** — diverges the first time
  somebody presses Up, and diverges **silently**, because a shell's line editor
  changes the line without telling anyone;
- **shell integration** — the real road, and how every other terminal answers
  it. A new mechanism, a published contract, and a decision about a sequence
  **any program can forge**, which is ADR 0113's question again.

**What landed anyway, and what deliberately did not.** #759 landed the
explanation value, the second kind of `alo` remark, and the guard test that
`what_alo_said.rs` had been citing while it did not exist. It did **not** land
the action or the chord written for it: a chord bound to a surface that cannot
work teaches a person a road and then takes it away, which
`tests/the_road_every_action_takes.rs` exists to refuse.

**What would unblock it**, in one sentence: a decision on shell integration —
the sequence, its contract, and what stops a program forging it.

### 6. Terminal tasks 4 and 5 — **both need a scope decision, measured 2026-10-17**

**The plan says of each: *Depends on: nothing in this plan.* That is true, and I
read it as *unblocked*, which it does not say.** Both depend on something
outside the plan, and measuring is what found out. *A scope that lives in the
method and not in the sentence is a scope the reader cannot check* — and here the
scope lived in four words, *in this plan*.

**Task 4, *find a command by what it did*.** The question is answerable: a record
entry carries `What`, whose `effect` says whether a call **changed** or only
**answered**, and `What::touched(path)` already exists and matches on recorded
arguments rather than on text. What is missing is **anywhere to ask it from**:

| where a question could arrive | why it cannot |
|---|---|
| the record window | **its own doctrine forbids this.** `record_keys.rs`: *no letters, so there is nowhere to type a search* and *nothing that hides a line, so no key could leave a refusal out*. A narrowed view is both of those |
| a verb the agent calls | **there is no such verb**, and the published list in `docs/contracts/agent-verbs.md` has none for the record. Adding one changes a public contract and `alo-capability`, which is another lane's |
| `alo-ask` | it asks **the model**, not the record — a different `Asking` type with the same name, which is how this looked reachable at first |

So giving it a surface is a new window, a new verb, or a new command — and each
of those is a promise, not the making-true of one. **That is scope**, and
`CLAUDE.md` is explicit that when the answer is not obvious it is scope.

*One real finding on the way: `RanACommand` records `allowed`, "the folders the
kernel let it reach", which is **permitted and not changed**. Answering *what
changed my git config* from it would answer *what could have*, which is this
repository's oldest fault family wearing a new coat. For a raw command the
record cannot say what it changed; for a verb it can.*

**Task 5, *scope one command without changing a setting*.** A per-command
narrowing of a grant, which is `alo-capability` — the same crate, and the same
unanswered ownership question, as the step below.

**What would unblock task 4**, in one sentence: a decision on which surface asks
the record a narrowed question, given that the one window that shows the record
refuses to narrow by design.

### 7. The choice of whether alo may use the terminal — **blocked, measured 2026-10-17**

**This was step 2 of this order for part of 2026-10-17, described as "the
shortest distance from *built* to *usable* of anything on the list after #755".
That was wrong, and measuring it rather than believing it is what found out.**
It is last, and it is not startable.

Row two of the thirteen: the two choices of ADR 0111 — whether alo may **open** a
terminal, and how code it writes **runs** — are built as types and a person can
neither see nor change them. Three measurements against `main` at `e236c8d1`:

| what was measured | answer |
|---|---|
| crates mentioning `HowCodeRuns` | **two** — `alo-capability`, which defines it, and `alo-running`, the sealed box. **Zero in `alo-shell`** |
| crates mentioning `ATerminalForTheAgent` | **one** — `alo-capability` alone, six files, none outside it |
| `alo_changing::Row` variants | **two** — `Grant(Seen)` and `Pairing(SeenPairing)`, and `Changing::granted` takes a `Granting` and a `Chosen`, which is a **folder** grant |

**So nothing stores either choice, and the machinery that stores a person's
grants cannot express a level.** Giving it one needs a third `Row` variant, in
`alo-changing`, or a new kind of grant in `alo-capability`. Both are another
lane's crate: `alo-changing` belongs to `where-a-persons-settings-are-kept-plan`
and `alo-capability` to `applications-and-what-they-expect-plan`.

**And ADR 0111 flags that exact ownership question as unanswered**, in its own
words: a change to `alo-capability` here is *a question for the integration
owner*, because ADR 0043 held a built change back on the ground that *`alo-capability`
is the applications plan's, and a lane does not widen its own partition* — *and
that question was never answered*. So this is not a question this lane may settle
by investigating; the charter's *write the ADR yourself* rule has two exceptions
and **what needs the owner personally** is one of them.

**What is left that is this lane's: showing, not choosing.** A Settings section
could display both choices, the three levels, the plain-words cost of full trust
and the three protections that stay on at every level, all from `alo-capability`'s
own words, in `alo-shell` alone. It was not built, for a reason with a precedent:
ADR 0038 records that the shell deliberately *does not draw a control that changes*
a setting it cannot keep, and a row showing `HowCodeRuns::default()` would be
showing **a default dressed as a person's choice** — nobody has chosen anything,
because there is nowhere for a choice to go. *Absent is stronger than defaulted*,
and features.md already says of the other half that **the default is that they
have not chosen**.

**What would unblock it**, in one sentence for whoever takes it: a decision on
where a capability *level* is kept, given that grants are folders and this is not
one.

## What this lane cannot finish — corrected 2026-10-17, because the first version was wrong about all three

**The owner asked whether these three were the desktop lane's. They are not, and
checking rather than agreeing changed every row.** The first version of this
table read as *out of reach*; two of the three are reachable and the third
belongs to a different lane than implied.

| | the honest position |
|---|---|
| *run a command touching nothing* | **available to the desktop lane and taken by nobody**, as of 2026-10-17. The owner that day: *development pc is off now so it's tasks can be done by desktop*. Until then it was the development PC's by `kernel-enforcement-plan.md`'s own words: *who can take it — the development PC, inside a KVM guest with a real login*. The desktop lane was told, and answered that it is **not taking the eight plans on a relayed message** — the owner had given it a direct instruction about the application market, and a forwarded sentence does not outrank an instruction the owner typed to that lane. **It was never a capability wall**: this lane's machine has `CONFIG_BPF_LSM=y` and `bpf` in the active LSM list, measured 2026-10-17. So it was ownership all along, the owner has made it available, and **it is still nobody's** |
| *bisect the machine*, *your files as they were* | **downstream of this lane's own bracket step**, not out of reach — the step headed *the bracket, if it is still nobody's*. **This cell cited "step 5" until 2026-10-17 and the bracket had become step 6**, because a step was inserted above it and the number here was not followed; the cross-references in this file now name steps instead of numbering them, which is why. If this lane takes the bracket, both of these become its to build — so listing them as unfinishable **contradicted the bracket step in the order above**, and that was an inconsistency in this document rather than a fact about the work. *This cell said "four headings above it" until the order was renumbered, at which point it was three — a positional count is the same trap as a step number and is gone for the same reason.* |
| *ask what a command will do first* | **buildable and testable here.** The promise is *plain words for what it does, with anything destructive, anything needing elevated rights and anything leaving the machine marked* — it does not say which model, and a **provider key is present on this machine** (mode 600, measured 2026-10-17). The first version said *a model on this machine is an environment this lane cannot test in*, which was true of a **local** model and false of the promise |

**Why all three were wrong in the same direction.** Each was written from what
this lane had recently found hard rather than from the document that governs it:
the kernel plan names its lane, the order four headings up already claimed the
bracket, and `docs/features.md` never asked for a local model. **A blocker
written from memory is a blocker nobody checked**, and this table is the fourth
thing today that a dated reading corrected — after `alo-turn`'s *nothing reaches
it*, the evidence document's *nothing has booted*, and the plan's inverted
dependency between the source qualifier and rich results.

**So one of the thirteen is not this lane's**, and it is now unowned rather than
held: the owner made it available on 2026-10-17 and no lane has picked it up.
The other two are this lane's, in the order above.

*The row above is the fourth reading of this one promise in a day — out of
reach, then the development PC's, then the desktop lane's, then nobody's. Only
the owner's message was new information; the rest came from this lane, and the
first was wrong. A row that moves this often is a row to cite rather than
remember.*

**And the third reading was this lane's own fault, of the exact kind it had just
written a paragraph against.** The commit that said *the desktop lane's* also
added a note to the lane charter explaining why the charter's eight rows must
**not** be rewritten to say *the desktop lane*: a row asserting an owner for
work nobody has started is worse than an empty cell, because the next lane reads
it and concludes the work is spoken for. **Then this row did that very thing**,
one table away, in the same change. *A machine being available and a machine
having taken something are different facts* — and knowing that in one paragraph
did not stop it being got wrong in the next. It was caught because the desktop
lane said plainly that it had not started, not by any check here.

## Standing, and the owner's

None of these blocks the order above.

| | what is needed |
|---|---|
| Pair five answers to four actions | `docs/features.md` records it as the owner's in as many words. #755 did not need it; offering the four actions does |
| Ship `alo-letting-go`'s four units | the owner's. This lane's advice is **ship**: measured on real `btrfs`, the units load and start, and #749 means a unit shipping with nothing to start it now fails the build |
| 32 tests only a person can start | the owner's; a gap between what the gate promises and what it enforces |

## How to check this document rather than believe it

Every row points at a plan, a pull request or a document that is the truth about
it. **This file is a reading taken on 2026-10-17 and will go stale** — exactly as
`alo-turn`'s *nothing reaches it* did three days after it was written, and as the
evidence document's *nothing has booted* did within one. A reader who finds this
disagreeing with a plan should believe the plan.

An index nobody can date is an index nobody can check, so this one is dated.
