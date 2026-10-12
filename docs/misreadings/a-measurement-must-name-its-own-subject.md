# A measurement must name its own subject

**Found 2026-10-11 and 2026-10-12, Mac lane and Desktop lane.** Five faults over two evenings, two of them mine
and one of them the Desktop lane's, with the same shape: a measurement ran, was
clean, and was not of the thing being asked about. The sentence is the Desktop
lane's and it is the one that generalises.

## What was concluded

Three claims, each of which I or another lane acted on.

1. **"The window-edge press is yours."** The Desktop lane asked whose a piece of
   work was — the owner had told them to ask rather than guess, because they had
   once built a duplicate of a fix of mine. I answered that my branch named
   `window_control` zero times and `what_this_edge_is_pointing_at` zero times,
   told them not to wait for my other branch, and told them not to bother
   sending me the diff first.
2. **"Nine of nine on this tree."** Not published, because the run refused for
   an unrelated reason — but it was about to be, and about the wrong checkout.
3. **(Desktop's own)** that a file was a journal rather than a kernel, from
   counting line prefixes.

## What was true

**1.** `gh pr list --state open` shows PR **#614**,
`task/mac/a-press-on-the-edge-acts`, titled *a press on a window's edge reaches
the mechanism that does it*, open, with `a_press_on_an_edge.rs` and
`a_press_on_an_edge_tests.rs` among its files — a real button press at
`pointer.rs:230` and all three performers reused. **The work was mine and
already built.** My grep was of `git diff origin/main...HEAD` on the branch in
my hand, which was a different branch of mine. The result was true. It answered
*does my current branch touch this*, and the question was *is this mine*.

The Desktop lane had written their own version, gated it clean, and discarded it
on the correction. Nothing was pushed, so the only cost was their evening — but
the duplicate the owner's instruction existed to prevent was caused by the
answer to the question that instruction produced.

**2.** `~/dev/alo-vm/gates.sh` had `REPO=/Users/…/dev/alo-os` written into it
with no override. This lane now holds several worktrees. Run from one of them it
gated the **main checkout** and printed:

```
GATES-ABOUT: head <sha> tree <sha> branch <name>
```

Every number real, internally consistent, and about a tree the caller had never
touched.

## The mechanism

**A scope that lives in the method and not in the output cannot be checked by
the reader.** All three measurements were correct about their actual subject.
None of them said what that subject was, so neither the author nor a second
reader could see the mismatch without re-deriving the whole thing.

This is worse than a measurement that is simply wrong. `CLAUDE.md` already
names the distinction: *a verdict whose own fields disagree announces itself; one
that agrees with itself while being wrong is invisible.* A clean grep, a clean
gate and a plausible line count all agree with themselves.

And it is not fixed by care. All three were taken deliberately, by somebody
trying to be rigorous — I added the second grep term *to show the search had been
thorough*, which is the same move as the
[stale negative result](a-negative-result-proves-nothing-about-the-search.md)
and made the same contribution: confidence and nothing else.

## The fourth one, which is the sharpest

**The fix for fault 1 contained a fresh instance of fault 1.**

Having been corrected about #614, I told the Desktop lane to start their
5,514-line removal of the `window_control*` family whenever they liked, because
**0 of #614's 22 filenames match the 35 files the removal deletes.** They
confirmed that, and confirmed something stronger I had not claimed: #614's
*content* names none of the family's types either — `WindowControlLayout`,
`WindowControlPointerEvent`, `route_presented_window_control_pointer`,
`window_control_input`, `control_press`, `WindowControlRoute`,
`window_control_snapshot`, every one 0 in its diff, against a control token
returning 10 and a nonsense token returning 0.

Both measurements were right. **A deletion is not confined to the files it
deletes**: removing 35 files means changing every file that uses them, and **14
of #614's 22 files are on that list** — including `crates/alo-shell/src/pointer.rs`,
which is where #614's press hooks in at line 230.

Measured twice by two lanes with different methods, agreeing on the number that
decides and differing on the aggregate, which is worth recording as the useful
shape:

| | Desktop lane | Mac lane |
|---|---|---|
| files the removal must modify | 108, of which 37 production | 83 |
| intersection with #614 | **14 of 22** | **14 of 22**, same list |

The aggregates differ because they are answers to slightly different questions —
*files under `crates/alo-shell/src/`* is 35 and *files anywhere named
`window_control*`* is 40 — and neither total was load-bearing. The intersection
was, and two methods reached it independently.

**And one of those aggregates was itself an instance.** The Desktop lane's first
count of files-to-modify was 113, from `grep -rl "window_control\|WindowControl"`.
That matched prose: comments naming the retired chrome in files with no code
depending on it, and a comment does not have to change when the family goes.
Excluding comment lines gives 108. They caught it before sending.

So: **#614 lands first**, which is what the plan said and now has a number
behind it. My *start whenever you are ready* was generous and was built on a
filename measurement that was true and not sufficient.

## The fifth, whose cost was caution rather than confidence

Worth its own section because a reader who only meets the overconfident shape
will not recognise this one.

Scoping the removal's third part, the Desktop lane asked whether it reached
`pointer.rs` — the file that decides whether their change can land beside #614.
The command was `grep -c "pointer.rs"` and it printed **1**, so part 3 looked
blocked.

**The dot is a wildcard.** The pattern matched `reader_pointer.rs`. The true
answer is 0, confirmed with `grep -Fxc` and a control on a path that *is* in the
list. They caught it before reporting and before waiting.

Had they not, the consequence would have been **waiting for no reason** rather
than acting for a bad one. Every other entry here cost somebody confidence they
had not earned; this one would have cost a night of work nobody needed to lose.
It is also the most mechanical of the five and the easiest to repeat: a filename
in a `grep` pattern contains a dot, and a dot is a wildcard in every regular
expression anybody will reach for.

In the terms of this directory — the
pattern was right about *a* question, and the question it answered was *does any
path contain `pointer`, then any character, then `rs`*.

## The cure

**Make the subject one of the things the output states.** Not a convention to
remember — a field in the result.

- `gates.sh` now takes `ALO_GATE_REPO` and prints `GATES-ABOUT: checkout <path>
  head … tree … branch …`. The override is the small half; printing the path is
  the half that makes a wrong subject visible without rerunning anything.
- **To answer *is this mine*, list your open work, not your current diff.** A
  branch diff answers a different question; this lane had three open pull
  requests of its own at the time, among five open in all.

  **And `--author @me` is not the filter.** It returns every open pull request
  on the repository, because `CLAUDE.md` requires that *the author of every
  commit is the repository's owner* — so authorship cannot tell two lanes apart
  and never could. Measured 2026-10-11: five open, three `task/mac/*` and two
  `task/third-pc/*`, all five returned by `--author @me`. **The lane is the
  branch prefix:**

  ```
  gh pr list --state open --json number,headRefName     --jq '.[] | select(.headRefName | startswith("task/mac/"))'
  ```

  This correction is the entry eating its own tail: the first draft of this cure
  named `--author @me`, which is a measurement whose subject is *the
  repository* wearing the label *mine*. Caught by checking it before committing
  the file that exists to warn about exactly that.
- **To answer *can this land beside mine*, measure the files a change
  modifies, not the files it names.** A deletion's blast radius is its
  dependents. `git diff --name-only <base> <branch>` intersected with *every file
  whose code references the thing being deleted* is the comparison; filenames
  alone understate it by however much the thing is used.
- **A filename in a `grep` pattern needs `-F`, or its dots escaped.** `grep -c
  "pointer.rs"` matches `reader_pointer.rs`. Use `grep -F` for a substring or
  `grep -Fxc` for a whole line, and put a control beside it: a path known to be
  in the list must return 1, and a nonsense one must return 0.
- When a peer asks an ownership question, say which artefact the answer came
  from. *My current branch's diff* and *my open pull requests* are different
  claims, and naming which lets them catch it in one line — which is what
  happened here, in the other direction, when they verified #614 themselves
  rather than taking the correction on trust.

See also [a name is not a fact about the thing](a-name-is-not-a-fact-about-the-thing.md),
which is the same family one level down: there the label was wrong, here the
label was missing.
