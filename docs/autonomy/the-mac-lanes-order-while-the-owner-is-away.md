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
| ★ | asks why a command is waiting | **built, reachable, and drawing nothing a person could read** until #778 on 2026-10-17: the chord put `alo` in the gutter and the sentence it labels went into the gutter's own 80-pixel box, under the word. Now drawn, and **seen under a nested compositor** (step 8c); **still never seen on a machine** |
| ★ | scrolls back through output | **usable** |
| ★ | runs a command touching nothing | **the box exists and works on a kernel** (#729); **nothing calls it** and the surface is unspecified — not usable. *This cell said "the kernel box does not" until 2026-10-17* |
| ★ | scopes one command | **blocked** — a per-command narrowing is `alo-capability`, another lane's |
| ★ | finds a command by what it did | the record can answer it; **nothing can ask it**, and giving it a surface is scope |
| ★ | bisects the machine | **nothing**, and waits on a bracket |
| ★ | reads output that is not a character grid | the first rich view is a test result, #761 — **and its two lines were cropped into the gutter's label box** the same way, until #778. This row said *usable* and the summary was not readable. Now drawn, and **seen under a nested compositor**; two more views are unspecified in the design |
| ★ | hands somebody a session to replay | **nothing**, and waits on rich results |
| | asks what a command will do first | the answer is built (#759); **nothing can ask**, because alo does not know what was typed — **blocked on ADR 0123 being accepted** |

**Where the thirteen stand at the end of 2026-10-17**, each row read rather
than counted by a pattern — a first pass with a crude classifier misread two of
them, which is this document's own lesson about mentions and declarations:

| | how many | which |
|---|---|---|
| **usable** — all four conditions | **4** | opens a terminal and types; reads who is speaking; sees measured apart from worked-out; scrolls back |
| **built and reachable, never seen on a machine** | **2** | asks why a command is waiting (#755); reads output that is not a character grid (#761). **Both were also broken on screen until 2026-10-17** — see the two rows above and step 8c. The count is unchanged because a nested compositor is not a machine |
| waiting on this lane's own earlier steps | **1** | hands somebody a session to replay, behind the rest of rich results |
| **blocked on a decision or another lane** | **6** | decides whether alo may use it; scopes one command; finds a command by what it did; asks what a command will do first; **bisects the machine**, moved here at 2026-10-17 when the bracket landed and did not open it (step 5); **runs a command touching nothing**, moved here the same day when the box turned out to be built and the missing piece turned out to be a surface the design does not specify |
| unowned | **0** | — *this row held* **runs a command touching nothing** *until 2026-10-17, when measuring found the box built (#729, 2026-10-16) and demonstrated on a real kernel. It moved to the row above: what it waits on is a surface, and the design is silent, which makes it the owner's* |

**And what *built and reachable* turned out to mean.** Both rows above said a
promise was built and reachable, and both were — the mechanism ran, a chord
reached it, and **a person would have seen a gutter word beside nothing**. The
sentence and the summary were being drawn into the gutter's own 80-pixel label
box and painted over by the word. Every check of either counted drawn runs and
passed, because both runs existed, one on top of the other.

*So the three conditions this table tracks were all true and the feature did not
work.* That is the strongest argument for the fourth that this lane has found,
and it was found a day early only because somebody drew the thing and looked at
the pixels. Fixed in #778.

**The two in the second row are what tomorrow evening's boot is for.** Both are
built, both are reachable, and nobody has watched either on a machine — which is
the one condition a green suite says nothing about and the reason step 1 of this
order is a boot rather than a task.

*Four of thirteen were usable this morning and four still are. What changed is
that **one promise gained a working view** and **four were found to be blocked
rather than merely unbuilt** — which is worth more than it sounds, because a
lane that believes four blocked things are startable spends four days finding
out.*

*A fifth joined them later the same day, and it is the instructive one: it was
blocked by the step that was supposed to unblock it. The bracket landed, and
`main` then said plainly that nothing brackets a command a person runs. **A
dependency being met is not the same as a task being open**, and the only way to
tell them apart is to measure after the dependency lands rather than to cross it
off.*

## The order, and it is ordered by that bar rather than by size

### 1. See the five answers on a booted machine — **the boot is 2026-10-18, evening**

> **STOP — no image that has ever been built contains a terminal.** Measured
> 2026-10-18, 00:10 local, against every workflow that produces one:
>
> | workflow | last success | commit | has `alo-terminal` |
> |---|---|---|---|
> | `image.yml` | 2026-10-07 | `77db91c6` | **no** |
> | `release.yml` | 2026-10-07 | `08e64019` | **no** |
> | `installer-candidate.yml` | 2026-10-08 | `dd356660` | **no** |
>
> `main` is **208 commits** ahead of the newest of those, and neither
> `crates/alo-terminal/` nor `alo-shell`'s `a_chord_opens_a_terminal.rs` exists
> in any of those trees. The digest `docs/booting.md` tells a person to pull —
> `sha256:fcc732bb…` — is from that era.
>
> **So booting what exists tonight would fail at step 2 and every step after
> it**, and the walker would report that the terminal does not work. It would be
> the most expensive report this repository can produce, and it would be about
> an image rather than about the terminal: the thing they booted never had one.
>
> **What has to happen first:** an image built from a `main` that includes #778,
> and ideally #782. `image.yml` is `workflow_dispatch` and runs only on `main`,
> so it is one deliberate dispatch. **This lane has not run it**: it publishes to
> `ghcr.io` and that is the owner's to start, not a thing to do unprompted at
> midnight.
>
> **What was not measured:** whether an image exists that was built by hand on
> somebody's machine rather than by these three workflows. If one does, check it
> for `alo-terminal` before trusting it — the question is what is *in* the image,
> not when it was made.
>
> **Step 0, and it takes ten seconds.** `image.yml` labels every image it builds
> with the commit it was built from, so the image can be asked directly rather
> than guessed at from a date:
>
> ```text
> podman inspect --format '{{ index .Labels "org.opencontainers.image.revision" }}' \
>     ghcr.io/aloworld-org/alo-os@sha256:…
> ```
>
> Then ask this repository whether that commit had a terminal in it:
>
> ```text
> git cat-file -e <that commit>:crates/alo-terminal/src/lib.rs && echo "it has a terminal"
> git rev-list --count <that commit>..origin/main
> ```
>
> **If the first prints nothing, stop and do not walk.** Every step from 2
> onwards will fail, and what you would be measuring is the age of an image. The
> second number is how far behind the walk's own expectations that image is.
>
> *Asking the artefact rather than the calendar is the whole point: an image
> built this morning from a stale branch is as wrong as one built last week, and
> a date cannot tell them apart.*
>
> **And for the image `docs/booting.md` sends a person to, the answer is already
> written down here.** That page installs the published release by digest, and
> `image/pinned.toml` records what it was built from:
>
> ```text
> version  = "0.0.6"
> revision = "77db91c64e590be259b48a162091425213e28728"
> ```
>
> That is the commit measured above as having no `crates/alo-terminal`. **So
> following `docs/booting.md` tonight installs a machine with no terminal in
> it**, and no registry needs to be asked to know that — the repository records
> it. `booting.md` is not wrong: it describes how a person installs the
> **released** machine, and the terminal is not in a release yet. The walk wants
> something else, and the two have to be told apart before the evening rather
> than during it.


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
| 8 | type `sleep 30`, Enter, then `Ctrl`+`Alt`+`W` | **`waiting for something alo cannot identify`** — a clock is none of the other four, and this is the honest answer rather than a wrong confident one. *The sentence, not the name:* `NobodyCanTell` is what the code calls this answer and **not what the screen says** |
| 9 | scroll up with the wheel, then back down | the transcript moves and stops at both ends |
| 10 | type `printf 'test result: ok. %d passed; %d failed; %d ignored;\n' 23 1 0` | **two lines under the output**: `Tests · 23 passed · 1 failed`, and under it `Source: cargo test output · Not independently verified`. **`program` in the gutter beside both**, not `alo` |
| 11 | look at the gutter beside those two lines again | still `program`. This is the design's *PROGRAM includes parsed rich results*, and it is the one thing in the walk that is easy to get wrong and hard to notice |
| 12 | run `ls` | the two lines **stay**, because the summary is still on the screen above. Not a fault: what is recognised is what the screen holds |
| 13 | run `clear` | the two lines **go**, because the line they were read from is gone |

**This walk was nine steps when the owner set it, and is thirteen now.** Their
own trigger still says *nine steps, each with its pass*, and it was accurate:
at `04310b08`, 2026-10-17 11:47, the table ended at step 9. Steps **10 to 13**
were added by this lane the same afternoon, and all four are about the
recognised test result — the summary, its gutter, and what becomes of it when
more output arrives and when the screen is cleared.

**Said here because the owner is expecting nine.** A longer walk is a longer
evening and a higher bar than the one they set, and *a plan lists what somebody
foresaw; a task is done when it works* is a licence to build the missing work,
not to move somebody's bar without telling them.

**The growth paid for itself, which is the argument for keeping it.** Steps 10
and 11 are where the recognised result's two lines were found cropped into the
gutter's label box — drawn, counted by every test, and unreadable. Step 13 is
where `clear` was found answering *TERM environment variable not set*, because
alo's terminal started every shell with an empty environment. Neither defect is
reachable from the nine.


**Step 6 is the measurement.** Steps 3, 4 and 9 are things believed to work that
nobody has watched either, so they cost nothing to read while the machine is on.
Step 8 is the one most likely to be wrong, because it is the answer a careless
table gets wrong — a first version of that table answered *waiting for another
program* about `sleep`.

**Steps 10 to 13 were added 2026-10-17, after #761.** They are the second of
the two things that are built, reachable and have never been seen. Step 10's
command is **`printf` and not `echo` deliberately**: an echoed command carrying
a summary is counted along with its output, so `echo` would show 46 passed
across two suites. With `printf '…%d passed…' 23 1 0` the echoed line's token
before `passed` is `%d`, which is not a number, so it is refused **by
construction** rather than by luck. `a_test_result_a_program_printed.rs` has the
measurement and a test that fails if that ever stops being true.

**Two steps quoted the wrong words until 2026-10-17, and both would have been
reported as faults that were not.** Step 8 told a walker to look for *nobody can
tell*, which is what the **code** calls the fifth answer; the screen says
`waiting for something alo cannot identify`. Step 10 expected
`Tests · 23 tests passed · 1 test failed`, and #766 corrects the summary to the
design's own `Tests · 23 passed · 1 failed`.

Both came from writing a step against an identifier instead of against the
string a person reads. `crates/alo-citing/tests/the_walk_expects_what_the_machine_says.rs`
now holds every step that quotes a vocabulary sentence to that sentence, so the
next rewording fails a build rather than a walk.

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

### 2. The bracket — **built and wired, 2026-10-17; what it unblocked and what it did not**

**It unblocked one of the three things that waited on it and not the other two.**
See step 5: *bisect the machine* and *your files as they were* both wait on a
decision the bracket did not settle, found by measuring `main` after it landed
rather than by assuming the dependency list was complete.

**Done, and measured on a real filesystem.** `alo_turn::the_bracket_a_turn_takes`
makes the two read-only snapshots and writes the note; `alo-agentd`'s
`the_bracket_this_turn_takes` takes one before the first change a person
approves and closes it as the turn ends. Three tests run as root on a loopback
`btrfs`: `btrfs subvolume list` names the snapshot, `property get -ts` says
`ro=true`, a write into it is refused, and the two sides differ.

**It needed no owner decision after all.** This document said it waited on *what
happens to a turn whose bracket cannot be taken*. ADR 0045's third term already
answers it, in the opposite direction to the guess recorded here: *a machine too
full to snapshot still runs the turn… a verb that refused to run for want of an
undo would be a machine that stopped working to protect a feature.* The taking
therefore answers *why not* rather than failing, and the type is not a `Result`
so no `?` can turn one into the other.

**Two things it did not unblock, and both are somebody else's:**

- **Nothing compares the two sides.** `alo_keeping_up::WhatWasKept`'s fourth
  variant carries the *answer to comparing* them, and ADR 0045 puts the
  comparing in the privileged path with the base's own tools —
  `the-broker-and-the-disk-plan.md`, which the charter gives to the **third PC**.
  So *bisect the machine* and *your files as they were* still wait, on a lane
  rather than on this one.
- **There is no variant for *kept but not yet compared***, which is exactly what
  a machine is now. An empty `Bracket` is not that: `changed: []` says **the turn
  changed nothing**, so handing one back would assert the opposite of what
  happened. `alo-agentd` answers its own three-way `WhatThisTurnKept` instead and
  the mapping waits. **Adding the variant is a change to another lane's published
  enum and is recorded here rather than taken.**

**And one thing about the prescribed build environment**, because it cost an
hour and will cost the next lane the same: `TMPDIR=$HOME/alo-tmp` makes a Unix
socket path exceed `SUN_LEN`, and `alo-agentd`'s failures go **32 → 22** with
`TMPDIR=/tmp` on the same tree. One of the ten is in `serving`, which a bracket
change touches, and reads exactly like *your change broke serving*. The
remaining 22 all need root and all pass under `sudo`.

### 3. What is still owed on rich results

**The first view landed as #761**: a test result, which is the one view the
design specifies. What is left under that task, in the order it is worth doing:

- **the three `Ask alo` actions** the design draws under the summary. The
  agent's road exists (`what_a_person_asked_goes_to_the_agent.rs`) but addresses
  a task window rather than a terminal, so it needs a *who asked* that can be
  either. **That part is this lane's and is buildable.** What is not is **what
  the three actions say** — see below;
- ***a thing on your desk that outlives the scroll*** — the promise's own
  words, and a window rather than a line. A promise of its own;
- **the benchmark and the diff**, which are blocked on the same thing the three
  actions are blocked on — see below. It is one cause, not three.

### 4. One cause blocks three things, and it is the owner's to unblock — measured 2026-10-17

**The design snapshots in this repository record where a component is placed and
never what is inside it.** Measured across every `.xml` in
`docs/design/figma-snapshot/`:

| | |
|---|---|
| component instances with **no** children | **4718** |
| component instances **with** children | **1** — `Choice / Your name`, in `the-first-start-sheets.xml` |

So an `<instance>` is a name, an x, a y, a width and a height. The most
numerous are `Canvas / Window title` (1813), `Terminal / Source region` (451),
`Terminal / Settings choice` (200), `Action / Keep`, `Action / Adjust` and
`Action / Reject` (178 each) and **`Ask alo / Action` (176)**.

**This is not *unknowable*, and an earlier note of mine implied it was.**
Component definitions *are* captured where somebody captured the node holding
them: **56 `<symbol>` nodes across six snapshots** — `Canvas / Window title`,
`Canvas / Icon control / Close` and others are there in full. `ROOTS.md` says
the library lives at node **`9:2` alo OS UI primitives** and that the
components are in the tree whatever the plan tier limits.

**Three of them have simply never been captured**, and all three block work
that is otherwise ready:

| not captured | what it blocks |
|---|---|
| `Ask alo / Action` | the three actions under a rich test result — the mechanism is this lane's and buildable, the wording is not |
| `Terminal / View / Benchmark comparison` | the benchmark rich view |
| `Terminal / View / File diff` | the diff rich view |

**What would unblock all three, in one line:** capture node `9:2` — or those
three components — into `docs/design/figma-snapshot/`, the way the other 56
were. **It needs the Figma connector authorised**, which cannot be done from a
session running unattended; the lane that measured the gutter's type got it by
asking the live file for the label node itself, which is the same road.

*Until then, building any of the three means inventing the inside of a component
the design already has — which is the one thing `CLAUDE.md` forbids outright:
the design is followed, not approximated.*

### 5. Bisect the machine, and *your files as they were* — **the bracket landed and neither opened, measured 2026-10-17**

**This step said both wait on a bracket and on nothing else. That was wrong, and
the bracket landing is what proved it.**

The promise's words are *every **command** in the session is a place you can
return to … your files are as they were **before it ran***. What #767 brackets
is **a changing agent turn** — `TheTurnsBracket` is taken in `alo-agentd`'s
serving loop at an approval — and across `main` at `e7d57ae8` **nothing in
`alo-terminal` or `alo-shell` takes a bracket at all.** A person's own `make` or
`rm` is not a place they can return to, and no amount of scrolling supplies one.

**Why this is the owner's and not a caller this lane may add.**
[ADR 0045](../decisions/0045-what-undoing-rewinds-to.md) term 5 brackets *only a
turn that changes files*, and alo cannot tell whether an opaque program changed
anything — so for a person's command the choice is *before every one* or *none*.
Term 1's window is seven days or **fifty changing turns**, accepted by the owner
immediately after they asked whether the machines would fill up with snapshots;
a shell session reaches fifty commands in an afternoon. Extending the bracket to
the person's commands changes the cost of the one term the owner priced
themselves.

**What would unblock both**, in one line: a decision on whether a command the
person runs in the terminal is bracketed, and under what window — which is an
amendment to ADR 0045 rather than a new decision.

### What that decision costs, measured 2026-10-17 so it is not decided on a guess

**The owner's question when they accepted ADR 0045 was whether the machines
would fill up.** It was the right question and nobody had answered it with a
number. This is the number, taken on a real `btrfs` filesystem on a loop device
in this lane's VM: a 300 MiB project, a 200 MiB build directory, and ten builds
— a working day — with the brackets arranged three ways.

| | held afterwards |
|---|---|
| **no brackets at all**, ten builds | **0 MiB** — each build frees the last one's output |
| **ten brackets**, one before each build *(today's rule)* | **2007 MiB** |
| **fifty brackets**, one before every command, ten builds among them | **2008 MiB** |

**The forty extra brackets cost one mebibyte.** A snapshot holds what is
rewritten underneath it, and a command that changes nothing diverges by nothing,
so it is free. *The cost is the bytes rewritten under a live snapshot, not the
number of snapshots.* Forty `ls`, `cd` and `git status` calls in a row cost
nothing to bracket.

**So the disk argument against bracketing every command does not survive
measurement, and the real cost is somewhere else.** ADR 0045's window is seven
days **or fifty changing turns, whichever ends first**. Bracketing every command
reaches fifty in an afternoon, so the window stops being seven days and becomes
*this afternoon*. **What a person loses is not disk, it is how far back undo
reaches.** That is the trade to decide, and it is a different question from the
one the term was written against.

**Said once, and it does not run again.** This needed root, a loop device and
about four minutes; no gate has those. The rule it establishes — cost follows
bytes rewritten, not snapshot count — is what generalises; the figures are one
workload, and a day spent rewriting a larger tree would hold proportionally
more. Nothing re-measures this, and a reader a month from now should treat the
table as a dated reading rather than a property of the product.

*Recorded rather than guessed, and recorded the minute it was found rather than
at the end of the day, because two tasks in two documents read as startable on
the strength of a dependency that had just been met.*

### 6. Explain a command before it runs — **measured blocked and then decided, 2026-10-17**

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

**So it was written, the same day.**
[ADR 0123](../decisions/0123-alo-learns-what-a-person-typed-from-the-shell-not-the-screen.md)
answers all three, under the rule that a machine unblocks itself rather than
leaving a decision for somebody with less context: OSC 133 for the cycle, a mark
the person's own line editor emits when they ask for it, and a declared length
that makes a truncated command refusable. The third question has no clean answer
and the ADR says so plainly — **nothing stops a program forging a mark**, and
what bounds the harm is that a mark grants nothing: it cannot run anything,
hold a grant, approve a change or relabel the gutter.

**This row stays in the blocked count of four.** The ADR is *proposed*; the
owner rules. What changed is the reason — from *nobody has written the
decision* to *the decision is written and waiting* — and this document warns
two sections below against reading the second as the first.

### 7. Terminal tasks 4 and 5 — **both need a scope decision, measured 2026-10-17**

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

### 8. The choice of whether alo may use the terminal — **blocked, measured 2026-10-17**

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

### 8b. What the walker will actually see — **drawn and looked at, 2026-10-17**

**alo's own terminal was drawn on a compositor for the first time on
2026-10-17** (#775), under a headless parent in this lane's VM, and the frame was
written out and looked at rather than counted. Two of the walk's thirteen steps
described something the walker will not see, and both are now reworded above.

| | |
|---|---|
| what is drawn | the gutter word **`program`** in lower case with its cursor beside it, and **`bash · UTF-8`** along the foot — which are steps 3 and 4, correct and in place |
| what is **not** drawn | **any ground.** The terminal's window and its transcript are both painted `alo_appearance::Token::Cream`, which is `bg/canvas` `#faf7f2` — the same colour as the desktop behind them |
| measured | every pixel sampled across the terminal's area is `(250, 247, 242)`. The only pixels of `bg/cool` `#eef2f4` or `bg/surface` `#ffffff` anywhere in the frame are the dock's, in `(465, 877)-(974, 950)` |

**This is not a bug to fix and must not be filed as one.**
`docs/design/the-terminal-window.md` says so itself, under *Colour*: the
surrounding page's roles are known, but *the transcript's own ground and ink are
not among them, so they are **owed** before the transcript is drawn and are
deliberately absent here rather than guessed.* The `Cream` in the code is a
placeholder standing in for a value the design has not given up, and the two
grounds recorded in that file — `program ground #eef2f4`, `gutter ground
#ffffff` — were measured off a **label** node, not off the window's regions. A
lane that wired those two into the window's two solids would be approximating a
design that says in writing it is silent.

**It is the owner's, and it is the same blocker as step 4**: the transcript's
ground and ink come off the design file, which needs the Figma connector
authorised.

**Why it is written here rather than left to be discovered tomorrow.** A walker
told to look for *a terminal window* who sees text on a bare desktop reports
that the terminal did not open — and that report is the most expensive thing
this repository can produce, which is the walk guard's whole reason for
existing. The guard in `alo-citing` checks that a step quotes words the machine
really says; it cannot check that a step describes something the machine really
draws. **Looking is what found this**, and a pixel count would not have: the
first version of #775's check asserted on area and failed a working terminal.

### 8c. Twelve of the thirteen steps were driven before the boot — **2026-10-17**

**This changes what the boot is for. It does not replace it.** The parent is
`weston --backend=headless` and the renderer is llvmpipe, so nothing below says
a word about a real display, a real keyboard, a real GPU or a machine that
booted. What it removes is the case where the evening of the 18th is the *first*
time anybody finds out.

Driven against a real `bash` under a real compositor by
`ALO_NESTED_SUBMODE=terminal`, and read back off the pixels:

| step | what was read back |
|---|---|
| 2 | a prompt and a cursor — **and no window**, which is why that step is reworded above |
| 3 | `program`, lower case, in the gutter |
| 4 | `bash · UTF-8` along the foot |
| 5 | `cat` waits, printing nothing |
| 6 | **`alo  waiting for the keyboard`** |
| 7 | after `Ctrl`+`D`, that line **goes** |
| 8 | **`alo  waiting for something alo cannot identify`** for `sleep 30` |
| 10 | `Tests · 23 tests passed · 1 test failed`, and under it `Source: cargo test output · Not independently verified` |
| 9 | `seq 1 40`, then wheel notches: the transcript moves, **stops at the oldest line**, comes forward, **stops at the newest**, and the last frame is byte-identical to the first |
| 11 | both of those in the **program's** region, labelled `program` |
| 12 | `ls` runs and the two lines **stay** — what is recognised is what the screen holds |
| 13 | `clear` takes the line away and the two lines **go** with it |

**Step 10's wording in the row above is the old one**, read off a tree that did
not yet have #766. **#766 has since landed**, so a machine built from `main`
now draws `Tests · 23 passed · 1 failed` — the shorter form, which is what the
walk asks for and what that change was for. The row is left as it was read
rather than rewritten to what it would say now: *a reading is a record of what
was on a screen at a moment, and editing one to match a later tree is how a
measurement becomes a claim.*

**One step is still unseen: 1, signing in.** That needs a machine, and it is
what the evening of the 18th is for.

**Step 9 was nearly checked the wrong way**, and the shape is worth knowing
because this document is read by people writing checks. `Server::pointer_axis`
answers whether the scroll was consumed, so the obvious loop sends notches while
it keeps saying yes — and it said yes two hundred times, far past the oldest
line, because with a pointer enabled and nothing focused **the canvas consumes
whatever the terminal does not**. The answer was true about the machine and said
nothing about the transcript. The loop draws a frame after every notch now and
stops when the picture repeats.

**Step 13 failed the first time it was driven, and the cause was not the
summary.** `clear` answered *TERM environment variable not set.* — alo's own
terminal started every shell with an empty environment, so `clear`, `less`,
`vim`, `top` and everything else built on curses refused. **A terminal that
cannot clear its own screen is not a working terminal**, and that had been true
of every terminal alo has ever opened. Fixed in the same change; the shell is
now told `xterm-256color`, and the constant's own doc says why that name. *A
walker would have found this tomorrow, at step 13, with nobody awake to fix
it.*

**Two of the nine were only checkable because they were broken first.** Steps 6,
10 and 11 drew their gutter word and no sentence until #778 — alo's answer and
the result's lines were being cropped into the gutter's own 80-pixel label box
and painted over. Every check of them counted drawn runs and passed, because
both runs existed, one on top of the other. *A count cannot see a thing that was
drawn in the wrong place.*

### 9. Try a command without letting it touch anything — **the box is built; the surface is the owner's, measured 2026-10-17**

**This document listed it as *unowned* and the plan listed it as *blocked on the
kernel boundary*. Both were wrong, and the crate that disproves them was already
in `main`.**

| Asked | Answer |
|---|---|
| the kernel boundary ADR 0013 wants | **landed 2026-09-04** — `alo-bounding`, `alo-bounding-kernel`, `alo-boundaryd` |
| the sealed box of ADR 0064 | **landed 2026-10-16**, #729, *a sealed box runs a command, and the kernel refuses the rest* |
| does it work on a kernel | **yes**, measured on this VM 2026-10-17: 20 tests as the person, and the two needing root pass as root — a command reads the file its box was given and `cat` is refused the file it was not. They **fail rather than skip** without a writable `/sys/fs/bpf` |
| crates depending on `alo-running` | **none** |
| readers of `HowCodeRuns` outside it | **none** — the three levels a person chooses between decide nothing anywhere |
| what the design says about how a person asks | **nothing**. The only match for *box* in `docs/design/the-terminal-window.md` is a label's bounding box, which is the word and not the thing |
| what ADR 0064 says about it | **nothing** — no prefix, no command, no chord |

**So this is the sixth mechanism on this list that nobody can call**, and it is
the one where the mechanism is furthest along: kernel-enforced, recorded,
torn down on every road out, and reachable from nothing.

**Why the caller is not simply built here.** Wiring `HowCodeRuns` to the box
makes an existing promise true and would be this lane's under *work a task needs
is part of that task*. But the promise is *paste the install script from a README
and **run it***, and nothing says how a person does that — so a caller cannot be
written without inventing the surface that calls it, and `CLAUDE.md` sends a
design that is silent to the owner rather than to a lane. Guessing a prefix here
would also collide with the neighbouring promise, *scope one command without
changing a setting*, which **is** specified as a prefix (`--only-here`,
`--offline`): two prefixes confining one command, invented a day apart by the
same lane, is the kind of surface nobody decided.

**What would unblock it**, in one line: how a person asks for a sealed-box run —
and whether that is the same gesture as scoping one command or a different one.

*The lesson this one adds to the four above: **a status can be wrong about a
module while citing the crate that contains it.** Task 9 said the box was not
built and named `alo-running` in the same sentence. The date it carried was the
crate's, not the module's.*

## What this lane cannot finish — corrected 2026-10-17, because the first version was wrong about all three

**The owner asked whether these three were the desktop lane's. They are not, and
checking rather than agreeing changed every row.** The first version of this
table read as *out of reach*; two of the three are reachable and the third
belongs to a different lane than implied.

| | the honest position |
|---|---|
| *run a command touching nothing* | **available to the desktop lane and taken by nobody**, as of 2026-10-17. The owner that day: *development pc is off now so it's tasks can be done by desktop*. Until then it was the development PC's by `kernel-enforcement-plan.md`'s own words: *who can take it — the development PC, inside a KVM guest with a real login*. The desktop lane was told, and answered that it is **not taking the eight plans on a relayed message** — the owner had given it a direct instruction about the application market, and a forwarded sentence does not outrank an instruction the owner typed to that lane. **It was never a capability wall**: this lane's machine has `CONFIG_BPF_LSM=y` and `bpf` in the active LSM list, measured 2026-10-17. So it was ownership all along, the owner has made it available, and **it is still nobody's**. **And then a fifth reading, the same day, which is not an ownership claim at all**: the thing everybody was deciding who should build **had already been built** — #729 landed the sealed box on 2026-10-16 and it passes on a real kernel. What is unowned is not the box but a caller, and what blocks the caller is a surface neither the design file nor ADR 0064 specifies. See step 9. *Four of the five readings of this row were about who should build it; none of them had asked whether it was built* |
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

**And the ownership question turned out to be the wrong question.** Measuring
the promise rather than its lane table found the box built and demonstrated, and
the gap somewhere else entirely — a caller, and a surface the owner has to name.
Whoever that promise belongs to, **nobody can finish it today**, and a day spent
settling who should start would have ended there. *Ask what is built before
asking whose it is.*

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
