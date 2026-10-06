# The loop reads what GitHub sends

**Mac lane, 2026-10-06.** Two defects in `tools/kernel-loop`'s landing path,
both found by using it rather than by reading it, and both of which make a
lane look slower than it is.

## One: the loop could not see a check that had passed

`waited_for` polls `GET /repos/.../commits/<sha>/status` every thirty seconds
and reads the state of each required context. It found none, every time,
because `the_state_of` searched for a pair it built itself:

```
"context":"alo/gates-on-a-runner"      what the loop looked for
"context": "alo/gates-on-a-runner"     what GitHub sends
```

**Measured:** pull request 542's required check passed at 16:52. At 17:44 the
loop was still reporting *has not reported on this commit yet*, with its
sixty-minute deadline approaching, after which it would have announced that
**CI** had been slow. The merge queue was empty and the pull request was
`CLEAN` the whole time.

The sibling helper `text_in` in the same file already finds a key and then
reads its value, tolerating whitespace. `the_state_of` re-implemented that
badly by spelling the pair as a literal. It now finds `"context"`, reads its
value with `text_in`, and compares.

**Why nothing caught it:** every fixture in that module is hand-typed compact
JSON, written by the same hand as the parser in the same hour. Four tests
passed about a shape the server never produces. That has its own entry —
[`a-fixture-i-wrote-agreed-with-the-parser-i-wrote.md`](../../misreadings/a-fixture-i-wrote-agreed-with-the-parser-i-wrote.md).

### And a fourth answer, because three could not say "I could not read that"

`TheChecks` had `AllPassed`, `StillRunning` and `OneFailed`. An unreadable
reply — bad credentials, a renamed repository, a commit GitHub has never heard
of, each of which answers with a `message` and no `statuses` — read as
`StillRunning`, so the loop waited an hour and then blamed CI. There is now
`Unreadable`, and it ends the wait rather than extending it. **The bug and the
thing that hid it are both fixed**: patience was the failure mode, which is
why nothing ever went red.

## Two: on macOS the loop cannot name the machine

`hostname()` tried `ALO_MACHINE`, `COMPUTERNAME`, `HOSTNAME` and
`/etc/hostname`. On macOS the variables are not exported to a non-interactive
shell and the file does not exist, so **every Mac landing fell through to the
`a-machine` placeholder** — which is the exact fault this function's own
documentation warns about: *a branch nobody could place*. Pull request 542
landed on `task/a-machine/feat-shell-a-frame-is-drawn-per-display`.

It now asks the operating system as a fifth source. **A fallback chain ending
in a placeholder reaches the placeholder on whatever platform its author did
not have in front of them, and nothing goes red when it does.**

## What was measured, and what will measure it again

```
FMT=0     cargo fmt
CLIPPY=0  cargo clippy --all-targets -- -D warnings
TEST=0    cargo test     # 186 passed, 0 failed
```

in `tools/kernel-loop`, which is a workspace of its own.

**Four new tests, which run on every build:**

| What it holds | Test |
| --- | --- |
| The shape GitHub actually sends is read | `the_shape_github_actually_sends_is_read` |
| A pretty-printed failure is not waited for | `a_pretty_printed_failure_is_not_waited_for` |
| An unreadable reply is not a wait | `a_reply_that_is_not_a_status_is_told_apart_from_one_still_running` |
| An empty status list still is a wait | `an_empty_status_list_is_a_wait_and_not_a_refusal` |

The first carries a response captured from the live endpoint that day, spacing
and `creator` sub-object included. The last exists so the new guard cannot turn
*CI has not started* into *I cannot read this*.

**The hostname fix is a hand measurement and nothing re-runs it.** On this Mac,
`hostname` answers `Disans-Laptop.local`, which reduces to `disans-laptop-local`
— placeable, and not the placeholder. A portable test would have to assume a
`hostname` binary exists, and a test that skips itself silently is worth less
than this sentence.

## What this does not do

- **It does not make `ALO_MACHINE` unnecessary.** `disans-laptop-local` is
  placeable but not what a person calls this lane. Setting `ALO_MACHINE=mac`
  is still the better answer; this only removes the unplaceable floor.
- **It does not rename pull request 542's branch.** That branch is already
  open and squash-merges; renaming it would cost a pull request to save a
  word in a list.
