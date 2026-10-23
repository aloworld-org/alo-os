# A red verdict needs a baseline as much as a green one

Four instances in one evening, 2026-10-22, across two lanes. Enough to be a
family rather than bad luck.

## What was concluded

Four times, a run was read as a result:

- *"Six suites failed, so the change broke them."*
- *"Six suites failed, so the change broke them"* — again, for a different
  reason, after the first was fixed.
- *"The test passes against the mutant, so it does not catch the bug."*
- *"All settled"*, from a watcher that printed it and exited `0`.

Three of those are red and one is green, which is the point: **colour was not
the variable.**

## What was true

None of the four had measured the thing it named.

| what it said | what had happened |
| --- | --- |
| six suites failed | the disk was at **0 bytes free**, 19 GB of it cargo incremental |
| six suites failed | a **stale cgroup** at `/sys/fs/cgroup/init.scope/home`, left by an earlier run in the same session |
| the test passes against the mutant | `sed` and then `perl` had both **failed to match**, so the "mutant" was the original file |
| all settled | `for n in $ME` under zsh, which does not word-split, so the **loop body never ran** |

The second is the family the gate section already names — *three unrelated tests
assumed they were alone on the machine* — arriving as leftover state from this
lane's own earlier work. The fourth was the other lane's, and it is the same
artefact with the opposite colour.

## The mechanism

**A verdict has two inputs, and only one of them is usually checked.** The
subject — the code, the unit, the tree — and the instrument: the disk, the
shell, the environment, whether the command parsed, whether the loop entered.
A report names the subject. Nothing in the output distinguishes *the subject
failed* from *the instrument never ran*.

**Red is more dangerous than green here, which is the part worth keeping.** A
green run that measured nothing is a known fear; this repository has four
recorded cases of a check standing in for a thing, and the gate section asks
*what would have to be true for this to pass while the product is broken.* But
a **red** run that measured nothing arrives dressed as a finding. It is
*interesting*. It pulls the reader straight into diagnosing the subject, and
the whole time spent diagnosing is time spent not asking whether the
measurement happened. Two of the four above were diagnosed as code faults for
several minutes each, and one of them — a unit's `ExecStartPre` — was written
up as a shipping bug and nearly reported to the owner as one. It was a dangling
`/home` symlink in a container.

**And the field that catches it is never the verdict itself.** The full disk was
caught by *8 errors beside 0 failed suites*, two fields that had to agree and
did not. That is the clause this repository already has: *a single number cannot
be caught being wrong; three that must be consistent can.* Worse, in that same
run the error count was **also** wrong — the pattern was matching test-failure
summaries as compile errors — so two fields were lying and still contradicted
each other enough to show it.

## The cure

**Run the same command against a tree you did not change.** That is the whole
rule, and it costs one run. If the baseline fails identically, the finding is
about the machine. If the baseline is clean, the finding is about the change.
Nothing else separates them, and no amount of reading the failure does.

It settled all four in one command each. `alo-agentd` at the branch head with
nothing applied gave 13 suites and 0 failures; with the change, 13 and 0. That
simultaneously disposed of the "six failures" and established what the change
did — which was the second finding of the evening and the more useful one: *not
one of the 40 tests is sensitive to the setting they were all changed to use.*

**For a mutation specifically: verify the mutation, not the run.**

```bash
cp file file.bak
sed -i '161s|.*|<the mutation>|' file          # line-addressed, not pattern-matched
diff -q file.bak file >/dev/null && { echo "MUTATION_APPLIED=NO"; exit 1; }
```

Pattern-matched substitution is where this goes wrong: `b"\n"` defeated both
`sed` and `perl` here, silently and with exit code 0. A line address cannot miss
quietly. **And check before the run rather than after**, because after the run
there is already a number to interpret.

**For anything whose shape could be empty: make it prove it ran.** A count, a
visit tally, a line nobody could predict. `0 missing` with nothing checked is
not a measurement — that is already written down for file audits, and it is the
same rule. The other lane's fix for the unentered loop was to count the visits
and refuse to conclude on fewer than three.

**And a strange failure on this machine is a full disk until proven otherwise.**
Check `df` before reading a single error.
