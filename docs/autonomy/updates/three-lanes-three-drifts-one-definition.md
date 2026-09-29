# Three lanes, three drifts, one definition — and the kind of drift nobody was looking for

**Date:** 2026-09-29
**Workstream:** the gates themselves (`tools/kernel-loop`)
**Responsible contributor:** the development PC, lane B, with findings from the
third PC and the Mac named where they are theirs
**Status:** a finding and a repair to lane tooling. **No gate changes.**
`tools/kernel-loop/src/gates.rs` is untouched; what changes is what the three
lanes run to reach it.

## What happened

On the night of 2026-09-28 a broken intra-doc link reached `main` through a gate
run that had been reported as nine of nine green. Finding out why turned up the
same fault in all three lanes' gate scripts, in three different directions, and
then a fourth kind of fault that none of the three was looking for.

Every lane had written its own nine commands to match the nine
`gates.rs` defines. **Every one of them had drifted, and no lane's drift was found
by the lane that wrote it.**

## The four kinds

| | |
|---|---|
| **Gates missing** | lane B ran **seven** of the nine — never the BPF target's formatting, never its clippy — plus a `citation` check that is not one of the nine at all. It reported nine. |
| **The command weaker than the gate** | lane B's rustdoc ran `cargo doc --workspace --no-deps` without `RUSTDOCFLAGS="-D warnings"`, which `gates.rs:1028` sets. rustdoc **warned and exited 0** where the gate errs and exits 1. |
| **The command wider than the gate** | the Mac ran `cargo test --workspace --no-fail-fast` where the definition says `cargo nextest run --workspace --profile gates -E …`. Its greens were never false — it ran *more* — but it paid the BPF attach cost on every gate of every branch, 1,400–2,100 s against 150. |
| **A count that cannot see a substitution** | the third PC's check computed `9 of 9` with `grep -c '^        named: "'`. It catches a gate added or removed and **cannot catch one replaced.** Which is exactly the substitution that put `cargo test` where the definition said `cargo nextest`. |

Three of those four are drift in what a script **does**, and comparing commands one
at a time — rather than counting them — finds all three. **The fourth is not a
drift at all**: it is an omission by construction, and no amount of careful command
comparison could ever have closed it. That distinction is the argument for calling
a runner, and it is stronger than *copies drift*.

## Why none of the three found its own

The third PC's observation, and it is the one that makes this a design problem
rather than three careless nights.

Each lane reproduced the definition — and then **built a check on its own
reproduction.** A check built on a copy inherits the copy's blind spots:

| | |
|---|---|
| the third PC | `grep -c` on the number of gate names. Nine against nine, and a substitution is invisible **because counting is the thing that cannot see it.** |
| lane B | nine lines doing nine things, which counted nine and agreed with itself. |
| the Mac | a context name implying the defined command, with nothing tying the name to what ran. |

Three lanes, three verification mechanisms, all three derived from the same
misunderstanding of what there was to verify.

**None of us failed to check. Each of us checked with the instrument that shared
the fault.**

Every drift that was actually found was found from **outside** the reproduction:
one lane's new script measured against another's timing; one lane's CI workflow
forced the question *which binary is deciding this*; and the supervisor's own
refusal found the disk. Not one was found by the check the lane had written for
exactly that purpose.

## The fourth kind, which command comparison cannot find

The third PC's replacement, on its first run, was refused by the supervisor before
a single gate ran:

> `alo-kernel-loop`: this machine needs somebody to clear something, so nothing
> was published: there is less than 12 GiB free on `/root/alo-builds` … A build
> started here would not fail as a build — it fails as a linker that cannot open a
> file, which reads like a broken change and is not one.

**`gates.rs` holds preconditions as well as commands** — room on the filesystem
the build will actually use, a BPF filesystem mounted at `/sys/fs/bpf`, a
toolchain the bridge can reach, and the runner the tests gate names. A copied
command list does not omit those carelessly. **It omits them necessarily, because
it does not know they exist.**

So every gate run that lane had made was taken at a risk the repository's own
runner declines. The runs were real and they passed — a completed run is not made
false by an unmet precondition — but the script had been printing `ROOM … 9.3G` in
its own output for hours and nothing acted on it.

## What all five faults have in common

Each is **a true-looking signal about the wrong subject**:

- a gate that reports which test failed and never why;
- a skip that is the same colour as a pass;
- a rustdoc *warning* wearing a pass;
- a status posted on a queue commit nobody measured;
- and a linker that cannot open a file, which reads like a broken change.

The last one is the clearest statement of the class, and it is the runner's own
sentence rather than anybody's analysis.

## What was done

**Each lane now runs `alo-kernel-loop gates` instead of its own nine commands.**
There is no list to drift from, no count to agree with itself, and the
preconditions arrive because they belong to the thing being called rather than to
a copy of it.

Two properties of the wrappers are worth keeping and are not the supervisor's job:

**The supervisor is built from the tree being gated.** A gate run by a binary from
another tree is a gate about another tree's definition — the same fault as nine
copies, one level up.

**`ATTESTABLE` is printed only when four things hold at once**: the supervisor's
exit code is zero, the working tree is still clean, the tree sha is unchanged, and
`origin/main` has not moved. The word and the conditions for the word are in one
place, so there is nothing to remember at the moment of posting. That also removes
the last place a hand-typed sha could enter a landing: the run establishes the
base and prints it, and posting copies a line.

**Nothing was weakened to make this pass.** `gates.rs` is unchanged and still
defines nine.

## What it cost, stated rather than left to be inferred

**Lane B's attestations before 2026-09-29 were seven of nine plus a non-gate.**
That includes the statuses posted on #256's queue commit and on `main`'s own
repair. The two BPF gates **pass** on that lane — verified by running them through
the supervisor afterwards — so nothing landed was false in substance. But that can
only be said from having run them, and not from anything the script did at the
time.

## Why a comment was not enough, and this is the part worth keeping

`gates_only` in `tools/kernel-loop/src/main.rs` already said it:

> a short list here without an error would be a run that quietly skipped gates and
> reported success — **which is how one lane ran seven of nine for weeks while its
> own check counted nine and agreed with itself.**

That sentence was written **about lane B's script**. Lane B read past it twice on
the night in question, both times while quoting the function it sits in, because
it was reading that function for a different question.

The comment was in the right file, in the right function, in code being actively
read, and it still did not arrive. **A sentence in a source file is not a check.**
The reason the replacement is the repair and the comment was not is that the
replacement makes the drift impossible rather than documented.


## Three rules, which came out of the repair rather than the faults

The repair produced more than the faults did, because writing the checks
reproduced the disease twice more. These are the usable part.

### A path with one user is a path with one tested case

A **reporting** path has no user until it fires. So does a **decision** path: the
word `ATTESTABLE` had exactly one exerciser — trees that passed — and every
condition inside it was therefore read only by runs that did not need it. Three
of one lane's six conditions were wrong and nothing said so; another lane's
`elif` chain would have reported one failing condition and hidden the very
disagreement that revealed the problem.

The only way to have a reporting path is to fire it on purpose. **Every fix that
stuck tonight was watched refusing first.**

### A harness has a different subject than the script

Watching a check refuse proves its **logic**. It says nothing about whether the
logic is **wired to the subject**.

This was learned by ignoring it. A digest check — meant to catch a script edited
while running, since bash reads from a byte offset and such a run has no single
version of itself — was watched refusing in six injected cases *and* on a real
edit-while-running of a copy. It was still broken in the real script: the path was
resolved after a `cd`, so it measured nothing. **The mechanism was correct and the
integration was never exercised.**

The check that would have caught it is cheap and is now the practice: run the real
script once and read its own first lines — a digest line that is 32 characters, or
a bare one that says the run is not attributable.

### An equality is worth nothing without a liveness check on its operands

The sharpest of the three, and the one that explains why every self-agreeing check
tonight passed.

| | |
|---|---|
| nine names counted against nine gates | both sides derived from the same assumption, so agreement was guaranteed rather than measured |
| nine lines counting nine | the script counted itself |
| an empty digest against an empty digest | `[ "" != "" ]` is false, so the check reported no change while having read nothing |

**Absence reads as agreement.** A comparison proves two things are the same; it
never proves either was measured, and the failure is silent because *same* is the
answer the happy path wants.

So the absence case has to be its own failure. *An empty digest matches an empty
digest, so nothing here was checked* is the general form, and any equality standing
between a run and a published claim needs it.
