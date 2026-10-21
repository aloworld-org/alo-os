# A check whose answer I was carrying in my head

**Found 2026-10-21**, in my own gate report, by the report's own fields
disagreeing. It cost nothing, because it was caught before it was published.
It is written down because the version that is not caught leaves no trace at
all.

## What was concluded

Nothing, in the end — which is the only reason this entry is short. The
sentence I was one step from writing was:

> 327 passed, 0 failed, across 7 binaries.

Which is true. The tree was green. Every figure in it is correct.

## What was true

The script that was supposed to produce that figure printed this:

```
FMT=0  CLIPPY=0  DOC=0  TESTS=0
PASSED=7
SUMPASS=0  SUMFAIL=0
```

`SUMPASS=0` beside `PASSED=7` and `TESTS=0`. The awk that summed the test
lines had the wrong field indices, so it added nothing and said so. The real
figure, measured afterwards by parsing the lines properly:

```sh
grep -oE 'test result: [a-zA-Z]+\. [0-9]+ passed; [0-9]+ failed' "$OUT" \
  | sed -E 's/.*\. ([0-9]+) passed; ([0-9]+) failed/\1 \2/' \
  | awk '{p+=$1; f+=$2} END {printf "passed=%d failed=%d binaries=%d\n", p, f, NR}'
# passed=327 failed=0 binaries=7
```

## The mechanism

**327 was a number I already knew.** It was in my own report from before the
rebase, in a message I had sent an hour earlier. So when the instrument printed
zero, the correction available to me was not *measure it* but *remember it* —
and both routes end at the same true sentence, written in the same words, with
nothing in it to say which one produced it.

That is the whole finding. A check whose expected value you are holding in your
head is a check you can satisfy without running, and **no report distinguishes
*I read 327* from *I remembered 327*.** The usual protections do not reach it:
the number is right, so nothing contradicts it; the tree really is green, so
nothing fails; and the figure is one a careful reader would accept precisely
because it matches what was said before.

It was caught only because three fields had to agree and one of them did not.
Had the script printed the awk total alone, the most likely outcome is not a
reported zero — it is **327, from memory, with the instrument never consulted.**

**This is one step earlier than the neighbouring entry and not the same
mechanism.** `a-test-that-asserted-its-own-premise.md` is a value *observed
once and mistaken for a property*: there is a wrong premise left behind in the
code, and somebody can find it. This is a value *known in advance and mistaken
for a reading*: there is no artefact at all, only a right number with no
provenance. The second is the one no instrument catches, because the instrument
is never consulted — so the two are kept apart deliberately rather than merged.

**And the rebase is what made the stale number dangerous rather than merely
known.** `327` described the pre-rebase tree. It happened to also describe the
rebased one. A figure carried across a rebase is a figure about a different
tree, and it will look right for as long as the two agree — which is exactly
until the moment it matters.

## The cure

1. **When you can predict a check's output, that is the moment it is worth
   least.** Print something alongside it that your memory could not have
   supplied: a sha, a path, a count of files, a timestamp. In this gate that is
   `HEAD=b9dac520 BASE=52835759` — two values I was not carrying, which is what
   makes the rest of the report a reading rather than a recollection.
2. **Never carry a figure across a rebase or a re-gate.** The old number is
   about the old tree. Re-derive it, even when you are confident, *especially*
   when you are confident.
3. **Make a report print more than one field that must agree**, which is
   `CLAUDE.md`'s standing rule and the only thing that worked here. A single
   number cannot be caught being wrong. Three that must be consistent can, and
   this one was.
4. **Treat a disagreement between your own fields as the finding**, not as
   noise to resolve toward the answer you expected. The temptation was to read
   `SUMPASS=0` as obviously the broken field — which it was — and move on with
   the number I already had. Re-deriving it cost one command.
