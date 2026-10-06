# A long build and an idle tree look the same

**Found 2026-10-06, Mac lane, while building `more-than-one-display-plan.md`
task 4.**

## What I concluded

That the checkout was mine to edit. I started `cargo test -p alo-shell`, it
exceeded its foreground timeout and moved to the background, and I carried on
editing source files in the same tree for the next twenty minutes.

## What was true

The run was still going. `CLAUDE.md` forbids *editing a checkout while a gate
owns it* in as many words, and the cost it names is the one I paid: the
verdict describes a tree that no longer exists. Every test result that run
produced had to be discarded.

It also breaks **one Cargo operation per machine**, which I came within one
keystroke of breaking a second time by starting the re-run before the first
had exited.

## Why it happened

**A Cargo run holds the tree without holding anything I can see.** A gate that
has locked a worktree announces itself; a backgrounded `cargo test` is a line
of output from twenty minutes ago. The files are writable, the editor says
nothing, and every tool answers normally. Nothing in the loop between *I have
an idea* and *the file is changed* passes a point where the question *is
anything building right now?* is asked.

The forbidden-list entry begins *deliberately not touching the tree was said
twice in one evening by the lane that then touched it* — and that is the
shape. It is not a rule anybody disagrees with. It is a rule whose trigger
condition is invisible at the moment it binds.

## The cure

**Before the first edit after starting a build, check that the build has
ended** — not *remember that one is running*, which is the thing that failed.
A `pgrep -f cargo` costs nothing and answers the only question that matters.

The stronger version, which this lane does not have: an edit-time hook that
refuses while a Cargo process is alive in the tree. `CLAUDE.md` already
recommends one ("a lock plus a pre-edit hook, not resolve") for the gate case,
and the case here is the same hazard with a shorter rope.

**And when it has happened: throw the verdict away rather than reading it.**
A suite that ran against a tree that no longer exists is not weak evidence, it
is evidence about something else. Reading it as *mostly still true* is how a
stale green reaches a report.

## Related

- [`a-pause-outlived-the-thing-it-paused.md`](a-pause-outlived-the-thing-it-paused.md)
  — the same family: a condition that was true when written and invisible when
  it stopped being.
