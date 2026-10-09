# A budget spent proving the tool was missing

**Found 2026-10-09, Mac lane, the minute the Dock loop finally had work to do.**

## What I concluded

That the loop had done twelve rounds of work on the Dock. Its own log said so,
in the words I had written for it:

```
[18:08:15Z] WORK ROUND 12 LOOP-EXIT=0
[18:08:15Z] DOCK-LOOP-DONE after 12 work rounds
```

Twelve rounds, every one reporting exit zero, and the loop retired itself as
finished.

## What was true

**Nothing was attempted at all.** Every one of the twelve rounds died in the
same second it began, and the loop's own supervisor said so plainly two lines
above the verdict I read:

```
the worker cannot run on this machine — 2 in a row died before they could
attempt anything. No task was given up and none was marked: the plan is
exactly as it was. Last: the worker `claude` could not be started:
No such file or directory (os error 2).
```

The cause was one line I wrote myself:

```sh
export PATH="$HOME/.cargo/bin:/usr/local/bin:/usr/bin:/bin"
```

`claude` is at `/opt/homebrew/bin/claude`, which is not on that list. My
interactive shell finds it; the detached loop never could. All twelve rounds
were spent in under a second, and the budget was gone at the exact moment task
50 became available to work on.

## The mechanism

**Two separate faults, and only together do they produce a silent lie.**

The first is ordinary: *a detached process does not inherit the PATH I read
`which` from.* I checked the binary existed by typing `which claude` in a shell
that had my full environment, and then ran the loop in one that did not.

The second is the one worth the entry: **`LOOP-EXIT=$?` measured the launcher,
not the work.** `cargo run` succeeded at every round — it compiled, started,
reported the worker could not be launched, and exited zero, *correctly*,
because running the supervisor is what it was asked to do. So the field I had
put in the log to tell me whether the round worked could only ever tell me
whether `cargo` worked.

This is the third time this family has cost me a day's reading in two weeks —
`$?` belonging to the `grep` at the end of a pipe rather than to clippy, and
`pgrep` watching the host-side client rather than the guest-side job
(`i-watched-the-client-and-called-it-the-job.md`). Each time the proxy is
correct in the ordinary case, which is why it reads as correct.

**And the damage was not the wrong verdict; it was the budget.** A loop that
cannot work consumed the whole allowance it had to work with, proving twelve
times over that a file was missing. The supervisor was scrupulously honest the
entire time. The number I had taught myself to read was the one that was not.

## The cure

Three things, and the third is the general one.

- **Check the tool before spending the budget.** A pre-flight that refuses to
  start costs nothing and converts twelve wasted rounds into one clear line:

  ```sh
  WORKER=/opt/homebrew/bin/claude
  if [ ! -x "$WORKER" ]; then
    say "REFUSING TO START: no worker at $WORKER. Nothing attempted, no round spent."
    exit 1
  fi
  ```

- **Name the tool absolutely in anything detached.** An absolute path cannot be
  broken by an environment, and `which` in an interactive shell is not evidence
  about a `nohup`'d one.

- **A round's verdict must come from the round's work, not from what started
  it.** If a log line says a round worked, the value in it has to be produced by
  the thing that does the work. Where the supervisor already prints an honest
  sentence, *read that sentence* rather than putting a second number beside it
  that agrees with itself and knows nothing.

**A resource consumed is a kind of failure a verdict cannot show.** Exit codes
answer *did it work*; they never answer *did it get a turn*. Where something is
rationed — rounds, gate slots, a queue's attention — ration-spent and
work-attempted are two different counts, and a loop that reports only the first
will one day report twelve of them for nothing.

## Related

- [`i-watched-the-client-and-called-it-the-job.md`](i-watched-the-client-and-called-it-the-job.md)
  — the same family and the same lane: a signal that could not observe the
  thing its name claimed. There the proxy was a process, here it is an exit
  code.
- [`a-formatter-that-writes-is-not-a-check.md`](a-formatter-that-writes-is-not-a-check.md)
  — *a field that comes from a command that mutates is a log line and not a
  verdict*. `LOOP-EXIT` is the same error one step further out: a field that
  comes from a command that **launches**.
- [`a-verdict-computed-is-not-a-verdict-returned.md`](a-verdict-computed-is-not-a-verdict-returned.md)
  — the verdict existed and was not the one read.
