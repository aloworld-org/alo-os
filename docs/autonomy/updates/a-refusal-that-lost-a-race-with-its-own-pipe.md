# A refusal that lost a race with its own pipe

**Date:** 2026-09-21
**Workstream:** `docs/autonomy/v0-5-the-session-and-the-displays-plan.md`, task 8
**Contributor:** this development PC's lane, second worker on task 8, one working tree
**Status:** ready for integration. Nothing here was measured on hardware and nothing here draws.

This is a **follow-up report**. The task's own report is
`docs/autonomy/updates/switching-to-another-person-at-a-locked-screen.md`,
written by the first worker on this task, and it is not edited here — README.md
in this directory says a correction is a report of its own. Everything that
report describes is still in the tree exactly as it was left. What follows is
the one thing that stood between it and publication.

## What the gate said

The candidate was refused on the workspace's tests, and on one of them:

```
---- provisioning::tests::a_tool_that_refuses_leaves_what_was_there stdout ----
thread '…' panicked at crates/alo-proxy/src/provisioning.rs:565:9:
assertion `left == right` failed
  left: NotWritten(BrokenPipe)
 right: TheToolRefused(Some(1))
```

**`crates/alo-proxy` is not in task 8's subject and was not edited by it.** It
came into range because the task's change carries `Cargo.lock` — `alo-sleeping`'s
manifest on `main` names two dev-dependencies the committed lock did not have, so
any `cargo` command in a clean checkout dirties the tree, and the first worker
included the correction rather than leave it for the next person to rediscover.
`SHARED_MAIN.md`'s scope table puts a `Cargo.lock` change in the all-nine row.
So the gate read a crate this task never touched, and found something real in it.

## It is a race, not a machine

Reproduced here before anything was changed, in `/root/alo-trees/this-machine`
with `CARGO_TARGET_DIR=/root/alo-builds/this-machine`:

- `cargo test -p alo-proxy --lib a_tool_that_refuses_leaves_what_was_there`
  alone — **passes**, every time.
- `cargo test -p alo-proxy --lib`, the whole binary, three runs — **one of the
  three failed** with exactly the assertion above.

That is the shape of a race, and the shape says where it is.

`TheMachinesCredentials::encrypted` starts `systemd-creds`, writes the password
on its standard input, waits, and then decides. The test's tool is `/bin/false`,
which exits without reading anything, so **the other end of the pipe is gone
while the parent is still writing into it**. Which of two true statements the
caller got back depended on who won:

| who got there first | what the caller was told |
|---|---|
| the write | `TheToolRefused(Some(1))` — the tool ran and would not do it |
| the exit | `NotWritten(BrokenPipe)` — the password could not be handed over |

One machine, one refusal, two answers. Under load the exit wins more often,
which is why the supervisor's whole-workspace run failed twice and a single test
run alone never did.

**And the race has a wrong side.** `NotWritten(BrokenPipe)` sends whoever stands
a machine up looking at a pipe, when what happened is that `systemd-creds`
refused and said so in an exit status. A program that refuses before it reads
takes the pipe with it on the way out, so the broken pipe is a *consequence* of
the refusal and never an independent fault. This is not only a test artefact:
`systemd-creds` exits before reading standard input whenever it dislikes its
arguments, its key or its store, and on a real machine that is the common case.
A caller whose answer depends on scheduler timing has no answer.

## What changed

### `crates/alo-proxy/src/provisioning.rs`

One rule, lifted out of `encrypted` into a function of its own:

```rust
const fn what_it_came_to(
    handed_over: Result<(), NotProvisioned>,
    the_tool_refused: bool,
    with: Option<i32>,
) -> Result<(), NotProvisioned>
```

**The program's own answer outranks the pipe.** A run that finished
unsuccessfully is `TheToolRefused(code)`, whatever became of the hand-over.

The other direction is deliberately **not** symmetrical and is kept as it was: a
program that finished *successfully* while the password never reached it
encrypted something other than the password, and that is `NotWritten(kind)`
rather than a success. A rule that simply preferred the exit status in both
directions would have turned that into a silently wrong credential.

It is a free function rather than a branch inside `encrypted` for the reason
`arguments` already gives in this file — *kept apart from starting the process so
the list is testable as a list on any machine*. Here it buys more than that: the
race cannot be ordered by a test from the outside. The child may exit before the
parent writes or after it, and nothing a test can say decides which. Holding the
rule as a rule is the only way both sides of it are tested rather than one side
and a coin toss.

`NotProvisioned::TheToolRefused`'s own documentation now says it outranks
`NotWritten`, so the precedence is on the type a caller reads and not only at
the site that applies it. The reference to `what_it_came_to` there is
deliberately plain text: the function is private, and a public item linking to a
private one is a rustdoc warning, which is a failed gate.

No public surface moved. No argument, no name, no error variant and no
behaviour a caller can reach was added or removed — `TheToolRefused` and
`NotWritten` both already existed and both already meant this. What changed is
that one of the two is now always the answer to the same event.

### The tests

**`a_refusal_is_the_tools_answer_and_never_the_pipes`** holds the rule directly,
on any host, with no process at all: a refusal with a broken pipe, a refusal
with a clean hand-over, a refusal that was signalled rather than exited, a clean
finish over a hand-over that failed, another over a different failed hand-over,
and a clean finish over one that did not fail. Deterministic, and it runs where
`/bin/false` does not exist.

**`a_tool_that_refuses_answers_the_same_way_every_time`** holds it through the
real process, sixty-four rounds against `/bin/false`, each asserting the same
refusal, that nothing was written, that nothing is left beside the credential
and that the store is still empty. Rounds rather than one because the race is
the subject; one round is the coin toss that got this candidate refused.

`a_tool_that_refuses_leaves_what_was_there`, the test the gate failed on, is
unchanged. It was right, and it is what found this.

**The regression is shown rather than asserted.** With the ordering put back the
way it was — in the Linux copy only, and restored from the checkout afterwards —
the rule test failed **three runs out of three** and the sixty-four-round test
failed **two out of three**, both with `left: NotWritten(BrokenPipe)` against
`right: TheToolRefused(Some(1))`. Counts and commands are under *Verification*.
A regression test that has never been seen to fail is a description.

## Decisions taken here, and why

- **Fix the code, not the test.** A test naming a refusal that only sometimes
  arrives usually means the refusal was meant to be one thing; deleting it or
  loosening it to accept either answer would have kept the ambiguity and hidden
  it. The assertion is the promise: one refusal, one answer.
- **Precedence by exit status, not by suppressing the write error.** The
  hand-over's error is still what is returned when the program *succeeded*. The
  smallest change that removes the ambiguity is a precedence, not a deletion.
- **A named function rather than a reordered pair of statements.** Two swapped
  lines inside `encrypted` would have fixed the machine and left nothing that
  says why, and would have been testable only through the race it exists to
  settle. `CLAUDE.md`'s *the refusal path tested as carefully as the happy path*
  is not reachable from outside this one.
- **Scope.** Nothing else in `alo-proxy` was touched, and the crates that depend
  on it were read for callers of this surface rather than rebuilt: the only
  external uses of `TheMachinesCredentials::written`'s refusal are
  `alo-brokerd`'s, which assert `is_err()` and not a variant, so no dependent's
  expectation moved. `alo-brokerd`'s proxy test was run anyway.

## Task 8 itself

Nothing in the first worker's change needed correcting; its crates were re-gated
here unchanged and pass. The plan,
`docs/autonomy/v0-5-the-session-and-the-displays-plan.md`, already carries task 8
as **Done, 2026-09-21** and already carries task 9 — *The four files this plan
keeps, in the contract that describes them* — written because the plan named none
after it. Both were the first worker's and both are correct; the plan is listed
among this change's files because it is in the change being published, not
because it was edited again.

## Verification

Run in `/root/alo-trees/this-machine`, a content-synchronised copy of this
checkout, with `CARGO_TARGET_DIR=/root/alo-builds/this-machine` and
`RUSTFLAGS="-C link-arg=-fuse-ld=mold"` — this machine's one build cache,
serialised, per `SHARED_MAIN.md`. Platform: Ubuntu under WSL 2 on this
development PC. Each was run in the foreground and its exit code read.

| command | result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy -p alo-proxy -p alo-locking -p alo-leaving --all-targets -- -D warnings` | clean |
| `cargo test -p alo-proxy` | 118 + 4 + 4 + 1 pass, **five consecutive runs** |
| `cargo test -p alo-locking` | 34 + 4 + 2 + 3 + 2 + 6 pass |
| `cargo test -p alo-leaving` | 41 + 2 + 5 + 3 + 5 + 3 + 6 + 2 pass |
| `cargo test -p alo-brokerd --test the_proxy_set_is_the_proxy_handed_over` | 11 pass |
| `cargo doc -p alo-proxy -p alo-locking -p alo-leaving --no-deps` | clean |

Before the fix, for the reproduction: `cargo test -p alo-proxy --lib` failed
once in three runs on `a_tool_that_refuses_leaves_what_was_there`, and the same
test run alone passed every time.

After writing the fix, with the old ordering put back in the Linux copy only:
`cargo test -p alo-proxy --lib` — `a_refusal_is_the_tools_answer_and_never_the_pipes`
failed 3/3 and `a_tool_that_refuses_answers_the_same_way_every_time` failed 2/3.
The copy was then restored from this checkout by content-synchronised copy and
re-gated; the table above is that tree.

**The whole workspace suite was deliberately not run here**, per this lane's
standing instruction; the supervisor runs it.

**Not measured:** nothing on hardware, nothing drawn, no egress. `alo-proxy`'s
`what_this_machine_writes_it_can_read_back` did run against this machine's real
`/usr/bin/systemd-creds` and passed, which is a measurement of that tool and not
of a certified workstation.

## Limitations

- The race is settled at the point where the answer is chosen. It is still true
  that a program can refuse before reading; what is no longer true is that a
  caller cannot tell.
- `Cargo.lock`'s two lines remain part of this change and remain not this task's
  work, for the reason the first worker's report gives. They are what put a
  latent fault in an unrelated crate in front of this task, which is an argument
  for landing lock corrections on their own rather than against either worker.
- The same pattern — write to a child's standard input, then read its status —
  exists nowhere else in this workspace's shipped code. Every other
  `stdin.take()` under `crates/` is in a test harness driving a process it also
  reads. Checked by reading them, not by a lint.

## Proposed shared-document updates

For the integration owner; not edited here.

- **CHANGELOG.md**, beside the task 8 entry: *a proxy password that
  `systemd-creds` refuses to encrypt is reported as a refusal by the tool, and
  no longer sometimes as a broken pipe depending on which of the two got there
  first.*
- **QUEUE.md / STATE.md**: task 8 of the session-and-displays plan is finished;
  this report and
  `docs/autonomy/updates/switching-to-another-person-at-a-locked-screen.md` are
  its two reports. No new work is owed by it.
