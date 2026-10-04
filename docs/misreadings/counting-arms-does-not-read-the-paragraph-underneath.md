# Counting arms does not read the paragraph underneath

**Concluded, and landed on main before it was caught:** *a third `MayAskIts`
variant is necessary.*

**True:** `crates/alo-nearby/src/permitting.rs` has a section headed **Two arms,
and what is deliberately not a third**, which says *there is no arm for run a
verb on that machine, **and there never will be one here***, with ADR 0003 as
the reason. The absence was the design.

**The mechanism.** Reading the enum found two arms and none for doing work,
which is true. **A gap and a deliberate omission look identical to a count** —
the difference is in prose immediately below, and counting does not reach it.

**The cure.** When an enum, a list or a table looks incomplete, read the file's
header and the paragraph beneath the declaration before concluding something is
missing. In this repository that prose is usually load-bearing and often says
why the obvious addition is wrong.
