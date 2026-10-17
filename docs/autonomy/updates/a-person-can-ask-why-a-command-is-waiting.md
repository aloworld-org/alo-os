# A person can ask why a command is waiting

**Date:** 2026-10-17
**Lane:** Mac
**Branch:** `task/mac/a-person-can-ask-why-a-command-is-waiting`

## What changed, for a person outside this repository

A command in the terminal looks stuck. The machine can now be asked why, and
answers from what it can actually see: waiting for you to type, waiting for a
network, waiting for another program or a lock, still working, or nobody can
tell with the evidence there is.

The point is the one `docs/features.md` makes: **instead of killing something
that was nearly done.**

## What was missing was only the joining

Three pieces were already built, each tested against real processes, on
different days:

- `the_command_in_front` knew **which** process to ask about — the one the
  terminal has in front, not the shell;
- `what_can_be_found_out_about` knew how to gather the evidence from three
  unprivileged reads under `/proc`;
- `why_it_is_waiting` knew how to read that evidence into one of five answers,
  with a measured table and a control for each case.

**Nothing called them together.** So the promise's *ask* had no ask, and the
feature was three correct halves of a question nobody could put.

This is one method. It is the whole of what was missing, and it is worth saying
plainly because it is the fault this repository keeps finding: **the pieces were
right and the seam did not exist.**

## Why it is about the command and not the shell

A shell sits reading its own terminal whenever a person is at a prompt. Ask
about the shell and the answer is *waiting for the keyboard* every time —
**including while a command is running**, which is the one case anybody asks in.

So the method asks about the process the terminal has in front, and answers
*nothing* when that is the shell itself, because then nobody is waiting for
anything and the person is simply at a prompt.

**Mutation-proved.** Changed to ask about the shell — the exact fault the
existing documentation warns about — and both new tests fail.

## What it returns, and why it is not just the answer

`WhatIsWaiting` carries the process as well as the answer. The promise does not
stop at the answer: *then show the prompt, inspect the connection, reveal what
is blocking it, or stop.* Every one of those four acts on something, and this is
the something.

What the process is *called* is not carried, because
`what_this_process_is_called` already answers that and most callers would not
read it.

## Measurements: where each one runs again

| what | where it runs again |
|---|---|
| a command reading the terminal answers *waiting for the keyboard* | every build — a real pseudoterminal, a real `/bin/sh`, a real `cat` |
| when the command finishes, nothing is waiting again *(control)* | every build |
| the answer is about the command and not the shell | the mutation above; the control test is what holds it |

The first test waits for `cat` to reach the front rather than sleeping a fixed
time, and fails loudly if it never does — so a green is a green about a command
that really was in front.

The second is the control that matters: a reader that answered *waiting for the
keyboard* about the shell would pass the first test and fail this one, because
the shell is always reading its terminal.

## What is still owed, and which half of it is a decision

`docs/features.md` lists four actions: *show the prompt*, *inspect the
connection*, *reveal what is blocking it*, *stop*. Its own words are that they
**are a surface and need one**.

Each is two halves:

- a **mechanism**, answerable from `/proc` with no decision to make — which
  socket the command is blocked on, which process holds the lock, sending a
  signal to the command in front;
- a **presentation** — where the answer and the offers appear, which is a
  surface and is not a lane's to invent.

The mechanisms are buildable now and are the next piece. **The mapping is not
mine either**: the promise lists four actions and five answers and does not say
which pairs with which, so deciding that would be designing rather than
building.

**No surface asks yet.** That is why the answer is public API of `alo-terminal`
rather than something a screen depends on.
