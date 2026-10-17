# Four things to do about a waiting command

**Date:** 2026-10-17
**Lane:** Mac
**Branch:** `task/mac/a-person-can-ask-why-a-command-is-waiting`

## What changed, for a person outside this repository

A command in the terminal looks stuck. The machine can be asked why — and now
it can also act on the answer: **stop it gently**, **name the program holding it
up**, or **say which connection it is waiting on and where that goes**.

Nothing shows this to anybody yet. What exists is everything underneath a
screen.

## The promise, and the four halves of it

`docs/features.md`: *ask, and get [five answers] — then show the prompt, inspect
the connection, reveal what is blocking it, or stop.* Its own words were that
the four actions **are a surface and need one**.

Each turned out to be two halves: a **mechanism** answerable from `/proc` with no
decision to make, and a **presentation** that is a surface. The mechanisms are
here. The presentation is not, and the mapping from five answers to four actions
is not either — the promise lists both and never pairs them, so deciding that
would be designing rather than building.

| action | mechanism |
|---|---|
| *stop* | `SIGINT` to the foreground group, refusing the shell |
| *reveal what is blocking it* | a lock holder, a pipe's writer, or a child |
| *inspect the connection* | which socket, and where it goes |
| *show the prompt* | already existed — `back_to_the_newest()` |

## Stop sends the gentle signal, by name

`SIGINT` to the **foreground process group** — byte for byte what a terminal
does when a person presses its interrupt key.

**Not `SIGKILL`.** That cannot be caught, so nothing cleans up and a
half-written file stays half-written. The promise this sits inside is *instead
of killing something that was nearly done*; offering a kill as the first step
would answer that promise with the thing it is against. A script gets to run
its trap, an editor gets to ask about unsaved work.

**To the group, not the process.** `cat file | grep thing | less` is three
processes in one foreground group, and signalling only the leader leaves the
rest reading a pipe nobody will write to again — a terminal that looks stopped
and is not.

**It refuses the shell.** When nothing is running the shell is in front, and
interrupting a shell at a prompt is useless at best.

**Mutation-proved**, and the timing is the tell: both behavioural tests pass in
0.06s and fail in **10.05s** when the signal is removed, because they wait out
the full patience before giving up. A test that times out when the machine stops
cooperating is a test that was really watching the machine.

## A control test found a real fault in the blocking reader

`a_pipe_fed_by` named a process about a **busy loop** — one working flat out,
whose standard input happened to be an inherited pipe. Confident, specific, and
wrong.

The three readers each answer *who holds this thing*. That is **not** the same
question as *what is holding this process up*, and nothing joined the two. A
process the kernel has on a processor is blocked by nothing, and that is now
asked before any reader runs.

The test that caught it had one job: assert a negative. Nothing else in the file
could have.

## Two encodings measured before they were written

**`/proc/locks` pairs a holder with its waiter, and the arrow shifts the
fields.**

```text
10: FLOCK  ADVISORY  WRITE 2142693 fd:01:4311 0 EOF
10: -> FLOCK  ADVISORY  WRITE 2142696 fd:01:4311 0 EOF
```

Two lines sharing an index. Parsing by fixed position would read a pid from one
and a lock mode from the other — a number that is not a process, compared
against a process, **quietly never matching**. The parser skips the arrow, and
both real lines are in the tests with a control asserting they do not read as
the same process.

It also finds the holder through the kernel's own pairing rather than by
matching inodes, so it cannot name the holder of a different file that happens
to share an inode on another device.

**`/proc/net/tcp` writes the address backwards and the port forwards.**

```text
 1: 0100007F:AEAB 00000000:0000 0A ... 20249653 1 ...
    ^^^^^^^^ ^^^^                      ^^^^^^^^
    address  port                      the inode, token ten
```

Read the address forwards and it is `1.0.0.127` — a real address somewhere
else. Read the port backwards and it is 43950 — a real port that is not this
one. **Neither would ever look like a bug.** Both wrong orders are controls in
the tests.

And the inode is **token ten** although the header lists eleven columns, because
`tx_queue`/`rx_queue` share one token and so do `tr`/`tm->when`. A reader
counting header columns would be two out and would read the **uid** as an inode
— nothing, matching nothing, for ever.

## It asks about the same descriptor the diagnosis used

`the_connection_being_read_by` asks about standard input, deliberately: that is
what `why_a_command_is_waiting` decided *waiting for the network* from, because
`docs/quirks.md` records that `wchan` cannot tell a terminal read from a socket
read. So the two readings cannot disagree about which socket they mean.

## Measurements: where each one runs again

| what | where it runs again |
|---|---|
| a running command stops, and then nothing is waiting | every build — a real pty, a real `/bin/sh`, a real `sleep` |
| stopping at a prompt is refused, by name | every build |
| each refusal says its own thing | every build |
| a holder and a blocked waiter read from the kernel's real lines | every build |
| a pipe is traced to the process feeding it | every build — a real `sleep \| cat` |
| a child is named to the process waiting for it | every build |
| **nothing is blocking a process that is working** *(the control that found the fault)* | every build |
| the address is little-endian and the port is not | every build |
| IPv6 is four little-endian words | every build |
| a real socket is found from its own descriptor | every build |
| a command whose input is not a socket has no connection *(control)* | every build |

## What is still owed

**A surface.** Something that shows a person the answer and offers them the
acts. That is a screen and not a mechanism, and `docs/features.md` says so in
its own words.

**And the mapping**, which is the owner's: five answers, four actions, and the
promise pairs none of them. Every answer offering all four is one honest
reading; pairing them is another. Guessing would be a release nobody decided.
