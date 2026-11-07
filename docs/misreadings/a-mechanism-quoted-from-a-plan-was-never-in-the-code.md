# A mechanism quoted from a plan was never in the code

*Found 2026-11-06 by the lane building `alo-app`, before any code rested on it.
Nothing caught it: it passed three documents and an owner's decision.*

## What was concluded

`docs/contracts/terminal-commands.md`, section 5, after the owner's answer 7:

> *for as long as the command runs, the kernel counter that already watches a
> turn's connections is attached to that application's sandbox*

ADR 0143's question 6(b) said the same — *the kernel programme that already
watches a turn's sockets is attached to the application's own sandbox* — and
the demo-two plan's stage 9 listed *`alo-bounding`'s socket counting* among the
things it would reuse.

## What was true

There was no counter. `crates/alo-bounding-kernel/src/kernel.rs` says *Nothing
in this file puts anything into a map, a ring buffer, a counter or a log line*,
and `crates/alo-bounding/tests/the_boundary_decides_and_forgets.rs` fails if a
third map appears. The program on `socket_connect` and `socket_sendmsg`
**decides** — allowed or refused — and keeps nothing. Attaching it to an
application's sandbox would have cut the application off from the network, not
counted it. The owner then decided a counter of its own (ADR 0148).

## The mechanism

**A sentence about what the code does was copied from a document about what the
code might do.** *Watches a turn's connections* is true in the ordinary sense —
the program looks at every connection a turn makes — and false in the sense the
sentence needed, which was *keeps a count*. Each document that repeated it cited
the one before, so the claim gained authority at every step and was checked
against the code at none. By the third repetition it was under an owner's
decision, which made it read as settled.

## The cure

Before a document says a mechanism **already** exists, name the file and the
line that is the mechanism, in the document. A claim that cannot be given a line
is a proposal, and is written as one: *a counter would be needed*. When reusing
another document's description of the code, open the code — *watches* and
*counts* are different verbs, and only the code says which one it is.
