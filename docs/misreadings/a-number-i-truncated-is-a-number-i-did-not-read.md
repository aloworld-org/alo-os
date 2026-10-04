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
