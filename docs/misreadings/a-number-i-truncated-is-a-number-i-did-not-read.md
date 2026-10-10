# A number I truncated is a number I did not read

**Concluded, twice in one day:** *the most-shared frame number is used 11 times*
— and, separately, reasoning about a gate's result from `ATTESTABLE` and an exit
code.

**True:** the script's own last line said **24**, and the gate's verdict was on
a `GATES=` line. Both were computed correctly and both were discarded by a pipe:
`head -40` cut the summary off a sorted list, and `tail -6` cut the verdict off a
gate.

**The mechanism.** A pipe that shortens output is a decision about what matters,
made **before the answer exists**. Nothing errors, so there is no signal that
anything was lost — and in the first case the visible maximum was reported as
the maximum.

**The cure.** When a script computes a summary, print it **first** or `grep` for
it by name. Never bound the output of a verdict: for a gate,
`grep -E 'GATES=|ATTESTABLE|dirty_after'`, never `tail -N`. If a truncated
listing is wanted for reading, print the total separately so the two can
visibly disagree.

## A third time, 2026-10-10, and it is the control that makes this one worth adding

**Concluded:** *the image never names `alo-desktop`, so the desktop binary
cannot be what runs on a machine.* `image/Containerfile` names it four times,
builds it, and installs it to `/usr/bin/alo-desktop`.

```sh
grep -rln "alo-desktop" --exclude-dir=.git --exclude-dir=target . \
  | grep -v "^./crates/" | head -20      # 21 files matched; the Containerfile sorted 21st
```

**What is new is not the pipe. It is that the guard against the other fault was
run, passed, and did not help.**
[a-negative-result-proves-nothing-about-the-search.md](a-negative-result-proves-nothing-about-the-search.md)
says to make a query prove it searched, and that was done: a control term,
`alo-compositor`, was run first and returned `image/Containerfile`, so the tool
worked and the directory was reachable. **A control is a different term in a
different invocation, so it can only prove a search was possible — never that
this answer was whole.**

```
a query that could not run        →  returns nothing   →  a control catches it
a query that ran and was trimmed  →  returns something →  a control cannot
```

A trimmed list is *correct output, cut*: every line true, no error, no warning,
and the surviving lines make the conclusion look **supported** rather than
unexamined. So the two entries' cures do not substitute for each other, and
running one is not evidence about the other.

**Also the cure above, applied in the direction it was not yet written for.** It
says to print a total beside a truncated listing. The same goes for a listing
used to decide an **absence**: `grep -rn … | cat` with `| wc -l` beside it, so a
list of 20 and a count of 21 contradict each other where a reader cannot miss
it. `head` is for reading a sample; it is never for deciding a negative.
