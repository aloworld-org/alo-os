# A flag I passed narrowed every package

**Concluded:** *`alo-access`, `alo-saying`, `alo-conforming` and `alo-shell`
pass locally.* Written into a pull request body as the local evidence behind a
change to what a screen reader announces.

**What was true.** The command behind that sentence was

```
cargo test -p alo-access -p alo-saying -p alo-conforming -p alo-shell --lib
```

and **`--lib` applies to every package named, not to the last one.** It
restricts each to its library target, so the run executed `src/` unit tests and
**skipped every integration test in `tests/`** — including the walk that then
failed CI, and including the new test the change itself added. *The change's
own test did not run in the verification of that change.*

Measured rather than inferred, on the same checkout:

```
$ cargo test -p alo-access -p alo-saying -p alo-conforming -p alo-shell --lib
  ... | grep -c "^test result"
4                 # one library target per package

$ cargo test -p alo-access | grep -c "^test result"
7                 # the library, and six test binaries beside it
```

**Four result lines were read as four packages passing.** They were four
library targets passing, and `alo-access` alone had six more binaries that
produced no line because they were never built.

CI caught it: `a_sign_in_with_the_screen_off_reads_as_the_table_in_the_report`
failed because a recorded walk table still said `no` and `approve` where the
reader now says `No` and `Approve`.

**The mechanism.** The flag was reached for to say *and the shell's library
tests too* — `alo-shell`'s full suite takes minutes and the change did not need
it — and it silently applied backwards over the three packages that were the
whole point of the run. Nothing in the output says what was not run: a green
`test result: ok` line appears per binary, and binaries that were never built
produce no line at all. **An absent line looks exactly like no tests to run.**
Four green lines were read as four packages passing, and they were four
*library targets* passing.

Three properties made it invisible, and the third is the general one:

- **The narrowing is in the method, not in the sentence.** *Passes locally* is
  a claim about a package; the command made a claim about a target. A reader
  cannot tell those apart from the sentence, and neither could its writer an
  hour later.
- **The subset was the correct subset to doubt.** Unit tests are where a
  vocabulary change would ordinarily show, so the run looked well chosen. It
  was well chosen for the risk that had been *thought of*.
- **A skipped test is silent.** A failing test is a line; a test that never
  ran is nothing. Scope reductions in test commands have no output, so the
  only record of what was measured is the command itself — which is why the
  command has to be quoted rather than summarised. The count above is the
  whole of the evidence that anything was missing, and it had to be asked for.

**The cure, and it is two sentences rather than a practice.**

**Quote the command, not the conclusion.** A pull request that says
`cargo test -p alo-access --lib` has told the reader everything; one that says
*alo-access passes* has told them something that may be false in a way they
cannot check. This repository already forbids *reporting a change as tested on
a subset of the gates*; this entry is what that looks like when the subset is
created by a flag rather than by a choice.

**And for cargo specifically: target flags are global to the invocation.**
`--lib`, `--bins`, `--test <name>` and `--doc` apply to every `-p` in the
command. When part of a run needs narrowing and part does not, that is two
commands. `cargo test -p alo-access` with no target filter runs all seven of its
targets and takes seconds.

**Found by CI, on 2026-10-05, in the same session in which this lane quoted
the subset rule to another lane.** Knowing a rule and holding a command to it
are different acts, and the gap between them is one line of shell.
