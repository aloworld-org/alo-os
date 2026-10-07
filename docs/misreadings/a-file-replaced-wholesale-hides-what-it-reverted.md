# A file replaced wholesale hides what it reverted

**Concluded:** *a diff tells the truth about what a change adds and can stay
silent about what it takes back.* On 2026-10-07 a branch was built by copying
seven files out of an older branch. One of them was
`crates/alo-saying/the-vocabulary.txt` — the published list of every sentence a
person can be shown — and the older branch predated a merge that had added a
sentence to it. Copying the file **removed that sentence**, and the diff showed
two additions and no removal.

```
+installer.will.give-alo-os          alo OS gets {area} GB of that space…
+installer.will.restart-keeping-windows   This computer restarts once…
```

That is the whole of what the diff said. What it did not say was that
`installing.not-the-space-the-installer-made` — merged earlier the same day, in
the change that taught the boot environment to refuse any partition but the one
the installer labelled — was no longer in the file.

**The mechanism, and it is not about this file.** A diff compares the branch's
file against the branch's base. Both sides were old. The sentence had entered on
`main` *between* the base and now, so from the diff's point of view it had never
existed. **A file taken wholesale from an older branch carries that branch's
answer to every question the file answers**, including every question somebody
else has since answered differently — and a wholesale replacement shows its new
contents, never its lineage.

**Why it is worth an entry rather than a shrug.** Three habits that are each
good made it invisible:

- **reading the diff.** The diff was complete and correct about what it
  described. Nothing in it was wrong;
- **splitting a branch by file.** `git checkout <other-branch> -- <path>` is the
  normal way to take part of a branch, and it is what produced this;
- **a generated file being left to its generator.** Nobody hand-edits a snapshot
  — which is right, and means nobody reads one either.

**What caught it** was the test that regenerates the snapshot and compares. It
fired on CI, not here, because this lane runs clippy, rustdoc and `fmt` locally
and lets CI own the tests. The snapshot test is in a crate the local run did not
touch.

**What to do about it**, narrow enough to follow:

> **Never carry a generated or shared file across branches. Regenerate it where
> it lands.**

For a snapshot, run its own documented command on the branch after rebasing. For
anything else a tool owns — a lock file, a checksum list, a vocabulary — take
the *inputs* from the other branch and let the tool produce the output again.
The question to ask of any file being copied between branches is **who else
writes to this**, and a file with more than one writer is one to regenerate
rather than move.

**The general form, which is the part worth keeping.** A diff answers *what is
different between these two states*. It is routinely read as *what this change
does*, and those agree only when the base is current. The moment a file is
replaced rather than edited, the two separate — and nothing in the diff's own
output distinguishes the cases. That is the same shape as the rest of this
directory: a check whose answer is read as the answer to a question it was not
asked.

**Found by** the third PC on 2026-10-07, splitting a branch so that the
arithmetic for alo OS beside Windows could land without the refusal it was not
yet time to lift. Caught by `crates/alo-saying`'s own snapshot test on CI, about
twenty minutes after the branch was pushed.
