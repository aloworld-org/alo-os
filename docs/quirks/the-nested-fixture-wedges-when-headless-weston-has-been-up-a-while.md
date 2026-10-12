# The nested fixture wedges when headless weston has been up a while

**Date:** 2026-10-11. **Status:** open, worked around by restarting the parent.

## What happens

Every fixture in `crates/alo-shell/examples` that nests under a Wayland parent
stops making progress. The symptom is not a crash and not an error: the process
sits at its deadline and never reaches the line after its draw loop.

Measured on two different fixtures on one afternoon:

| fixture | what it did | where it was |
|---|---|---|
| `desktop_check` | wrote picture 1 of 64, then nothing for 15 minutes | `poll_schedule_timeout` |
| `a_real_application` | started its clients, mapped nothing, ran past a 12-second deadline to 200 | `poll_schedule_timeout` |

`cat /proc/<pid>/wchan` on the main thread says `poll_schedule_timeout` in both;
the `llvmpipe-*` threads are all in `futex_do_wait`, so software GL is loaded
and idle. The parent is **still alive** — `pgrep weston` finds it, the socket is
still in `$XDG_RUNTIME_DIR` — it has simply stopped delivering whatever the
nested compositor's submit is waiting for.

## The workaround, which is the whole of what is known

Kill the parent **by pid** and start a fresh one. `desktop_check` then wrote all
64 pictures, and `a_real_application` mapped its windows and completed its walk
on the first try after a restart.

`pkill -f "weston --backend=headless"` does **not** work from inside
`limactl shell` — it returns 255 and the process survives. `kill <pid>` does.
That is worth writing down separately because a `pkill` that quietly fails looks
exactly like a parent that was already gone.

## What has not been established

- **Whether it is time or work that wedges it.** The parent that wedged
  `desktop_check` had been up for hours; the one that wedged
  `a_real_application` for under an hour but had served a full 64-picture run.
  Those are two different hypotheses and nothing distinguishes them.
- **Whether a real seat behaves the same.** Only headless has been measured.
- **Whether it is weston's fault at all.** The nested compositor could be
  waiting on a frame callback it is not entitled to, and a parent that stops
  sending one would then be correct. Nobody has read the protocol exchange.

## Why it matters beyond the inconvenience

**A wedged fixture produces no output and a passing one produces output**, so
the failure mode is a run that looks like it is still working. Both of the above
were found by watching a file count stay still, not by anything failing. Any
loop that runs a nested fixture unattended needs a deadline **outside** the
fixture — `timeout` around it — because the fixture's own deadline is inside the
loop that stops advancing.
