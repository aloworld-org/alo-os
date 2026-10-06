# I watched the client and called it the job

**Found 2026-10-06, Mac lane, while gating
`more-than-one-display-plan.md` task 4.**

## What I concluded

That the test suite had finished. I had armed a watch that said, in as many
words, *tell me when the suite ends*:

```
until ! pgrep -f "cargo test -p alo-shell" >/dev/null 2>&1; do sleep 15; done
```

It fired. I was one step from reading a verdict out of a log and handing the
work over.

## What was true

The suite was still running, and had another twenty minutes to go.

Gates on this machine run **inside a Lima virtual machine**, reached with
`limactl shell alo sudo bash -lc '…'`. So there are two processes with that
command line in them: the **host-side client**, which is a pipe, and the
**guest-side cargo**, which is the job. `pgrep` on the Mac can only ever see
the first. The host ran low on memory and killed the client; the guest kept
compiling, oblivious.

The watch did exactly what it was written to do. What it was written to do
was not what its description said.

## The mechanism

**A proxy that is reliable in the ordinary case and silent about the case
that breaks it.** For a local build, host process and job are the same thing,
and this check is correct — which is why it reads as correct. The two come
apart only when something kills the client without killing the job, and that
is precisely the situation in which somebody is anxiously waiting for an
answer.

This is `CLAUDE.md`'s *a check that stands in for the thing is not the thing*,
with the extra sting that **the stand-in disagreeing with the thing is the
event the watch was for**. The question that catches it is the one that file
already asks: *what would have to be true for this to pass while the product
is broken?* Here: the client dies early. Which it then did, within the hour.

It is also a second instance of a fault I had just written up — see
[`a-long-build-and-an-idle-tree-look-the-same.md`](a-long-build-and-an-idle-tree-look-the-same.md).
Both come from the same blind spot: **I kept reasoning about the build as
something happening on this Mac**, when every byte of it happens in a guest I
can only reach through a pipe.

## The cure

**Watch an artefact the job itself writes, not a process somebody else can
kill.** The job now ends with `echo "SUITE-EXIT=$?"` into a log on the shared
mount, and the watch waits for that string. A log line is written by the thing
that did the work; a process table entry is written by whoever launched it.

Two corollaries worth keeping:

- **Run it detached in the guest** (`setsid nohup … &`, output redirected
  inside the VM), so a dead host client cannot orphan or kill a forty-minute
  build. The first run lost twenty-five minutes of linking to exactly that.
- **A terminal marker must cover failure too.** `SUITE-EXIT=` matches whatever
  the exit code was. A watch that waited for `test result: ok` would hang
  forever on a compile error, and silence would look identical to *still
  building*.

## Related

- [`a-long-build-and-an-idle-tree-look-the-same.md`](a-long-build-and-an-idle-tree-look-the-same.md)
  — the same hour, the same blind spot about where the build lives.
