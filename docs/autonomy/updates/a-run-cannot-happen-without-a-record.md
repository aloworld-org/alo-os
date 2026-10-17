# A run cannot happen without a record

**Date:** 2026-10-16
**Lane:** Mac
**Branch:** `task/mac/a-run-cannot-happen-without-a-record`
**Stacked on:** `task/mac/the-record-says-a-command-ran` (#731)

## What changed, for a person outside this repository

When the agent runs a command on your machine, your record says so. When it
asks to and nothing runs, your record says that too, in a sentence you can
read. Neither can be skipped, and if the record cannot be written the command
does not run.

## The promise, and why it was still only a promise

ADR 0064 keeps three protections on at **every** one of its three levels. The
sealed box landed with the record described as the caller's — true about
ownership, and it meant a caller could forget. #731 gave the record two entries
for a command; nothing wrote one.

`alo_turn::Kept` had already argued the answer, about turns:

> *A turn hands nothing back that it has not written down first. That promise is
> only structural if a turn cannot be made without somewhere to write.*

So `WhereCommandsRun` takes somewhere to write and **there is no constructor
that does not**. A caller cannot reach a run holding nothing to record it in.

## The hole that was in the join, not in either half

`ARun::checked` refuses four things: a level that is not the sealed box, a
relative program, a program that is not there, and a run naming nowhere to
reach. A caller called it **before** the run — so each of those four refusals
happened outside the only road that writes a record, and **a request refused on
the way in left no trace at all.**

`CLAUDE.md`'s gate is *every execution and every refusal leaves a record*. Four
of eight refusals were outside it.

**No test would have caught this**, and that is the part worth keeping: the
checking was right, the recording was right, and the promise fell through the
join. So `AskedToRun` is the unchecked request a caller hands over, the checking
happens inside, and there is one road in. `ARun` stays public because a surface
wants to validate before it asks a person anything; what it no longer is, is the
way in.

## Why not `alo_turn::Kept` itself

Because it lives in `alo-turn`, which has seventeen dependencies — models,
nearby, egress, software, applications, the whole agent world. Reaching through
it for one trait would pull all of that into the sealed box and re-couple
exactly what ADR 0117 separated. `SomewhereToWrite` declares the same shape over
the same two implementations, and **the duplication has a written condition for
ending**: `Kept` names no turn and belongs in `alo-keeping`, which both crates
already depend on. If it moves there, that file becomes a re-export.

Unifying the two sinks is the whole of what the trait does: `Record::keep` takes
an entry by value and cannot fail; `Writing::keep` takes it by reference and
can.

## Where the words live, and the hazard that decided it

The four sentences a person reads when a command did not run are declared in
**`alo-capability`**, not here.

`alo-saying` is the machine's one vocabulary and it ships every crate that
declares a word — which puts that crate inside the process an agent's turn runs
in. **This crate holds the only `exec` in the repository.** A word declared here
would have put that `exec` on a turn's dependency path. `alo-letting-go` guards
the identical hazard with a test of its own, for its one privileged remover, and
`alo-saying` has zero dependency on `alo-running` today.

`alo-capability` already declares the words for the three levels and is already
in that process, which is where these belong: they are about the capability, not
about the mechanism.

**Four sentences for eight refusals**, and the grouping is deliberate: a person
asking *why did nothing happen* needs to know whether the request was unusable,
whether their own choice stopped it, or whether the machine failed — and does
not need eight shades of the third. A surface wanting the detail has the error.

## Four mistakes I made, three caught by the compiler or a test

**`TheBoxWouldNotGoAway` took the refusal road.** A box that will not come away
*after* the command ran would have recorded *a command was not run* about a
command that did. There are three roads out of `ran` now and the middle one is
the easy one to miss; the error carries how the command ended, the run is
written down, and the cleanup failure is what is reported.

**Two functions returned a `Result` whose `Ok` arm nothing can reach.**
`refused` and `put_back` are each reached only from a failure. Both answer with
the refusal itself now.

**I invented a `refused_for_a_test` entry point.** A bad public surface, and
unnecessary: `checked_and_written_down` is the real road's first half, honestly
named, and useful on its own to a surface that wants to tell a person their
request is not a command and have the record say so.

**Two APIs I guessed wrong**, both caught by the compiler: `Said::as_str` (it is
`text()`), and `alo_keeping::NotKept` implementing `Error` — it implements
neither that nor `Display`, being a person-facing type with a `said(strings)`
method, so it is carried by its `Debug` with the reason written beside it.

## The source guard earned its keep, and then collided with a name

The guard holds that exactly two files here may start a program, and checks the
spawning one against the literal `Command::new(helper)`. The helper became a
field, so it failed on the rename — which is the guard noticing that the call it
guards had moved.

Then it failed again, on `lib.rs`, where the only mention is a `pub use`:
**`WhereCommandsRun` contains the substring `Command`.** The matcher now names
the call, `Command::`, which is the shape `alo-bounding`'s guard already records
about `exec` being inside *execution* — this repository's own vocabulary
colliding with a syscall's name. The collision is a control in the test now, so
it cannot come back silently.

*This is the third time in one session that a bare-`Command` matcher tripped on
a name. It is a pattern and not bad luck: the domain word here is command.*

## Measurements: where each one runs again

| what | where it runs again |
|---|---|
| every refusal of a request leaves exactly one entry, reading as a refusal | **every build**, any machine |
| the four sentences are four sentences, not one *(control)* | same |
| a record that will not write is itself the refusal | same |
| a run's account carries what the record asks for | same |
| only two files start a program, and which, and what | same |
| the matcher catches a real spawn and not a name *(control)* | same |
| **a run is written down, with the kernel holding the boundary** | root + BPF LSM; `--profile boundary` after a merge |
| a command is refused outside its bound | same |

**The refusal half runs in CI and the run half cannot.** That took a deliberate
change: the refusal road needs somewhere to write and nothing else, so it is a
free function. As a method it would have carried a `Boundary` it never touches,
and a `Boundary` cannot be made without a kernel holding a BPF LSM — which would
have put one of `CLAUDE.md`'s named capability guarantees behind a machine that
only exists after a merge.

## What is still owed

- **The setting** a person chooses the level in, and **the surface** they
  approve a command on. Nothing asks anybody anything yet, and this crate makes
  neither look built.
- **The snapshot before each run**, which is the second of ADR 0064's three. It
  is unblocked as of ADR 0045 terms 8 and 9 and is not built: nothing creates
  `/var/lib/alo/undo` and nothing takes a bracket.
- **The egress indicator** is `alo-egress`'s, and what this crate owes it is the
  bound — a box with no approved destination cannot connect anywhere, which the
  kernel enforces on `socket_connect`.
