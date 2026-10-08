# The last commit to touch a file did not break it

**What was concluded.** `main` stopped compiling on Windows:
`alo_installer::A_REDISTRIBUTABLE_LIBRARY` and `alo_installer::TheImports` were
gone from the public root and a Windows-only test still imported them. The Mac
lane reported the cause to the lane that owns the crate as **#571, control
five** — the commit that had landed minutes before the failure — and said it had
checked rather than inferred.

**What was true.** **#559** removed them, four commits and several hours
earlier. It deleted `mod imports;` and the whole `pub use imports::{…}` block.
#571's diff to that file is **three added lines and one modified**, and it
removed nothing. The lane that was told it had broken `main` found this itself,
with one command, and corrected the lane that had told it.

**The mechanism.** Two true facts were checked and their **conjunction** was
reported as a third that had not been:

- the symbols are absent from `origin/main` — true, verified;
- #571 is the last commit to touch that file — true, verified;
- *therefore #571 removed them* — **never checked, and false.**

`git log -- <file>` answers *who touched this file*. The question was *who
removed this symbol*, and the tool for it is `git log -S "<symbol>" -- <file>`,
which names the commit in one line. The two commands look interchangeable and
answer different questions, and the wrong one is the shorter one.

**Recency is what made it feel verified.** A change that landed minutes before a
failure is the most available explanation there is, and availability reads as
evidence. The lane had even written, in the same message, *"I checked rather
than infer the cause"* — which was true of the two facts and not of the claim
they were offered for. **A sentence that names its checking is not evidence that
the checking covered the claim.**

**The cure.**

- For a **missing** symbol, declaration or line, the first command is
  `git log -S "<text>" -- <path>`, never `git log -- <path>`. A removal is a
  content change and only `-S` searches content.
- Before naming another lane's commit as a cause, read **that commit's own
  diff** for the file. #571's diff is four lines and takes ten seconds to read;
  it would have refuted the claim before it was sent.
- Where a conclusion joins two verified facts, say which part was verified and
  which is the inference. The inference is the part that needs the extra
  command.

**Which kind of fault this was: one that something caught.** The gate found the
broken build, and a peer found the misattribution. Nothing shipped. What it cost
was a false accusation, sent to the lane that owns the crate, in the message
telling them `main` was broken — and that lane spending its first minutes
disproving a cause rather than finding one. *Related:*
[`a-negative-result-proves-nothing-about-the-search.md`](a-negative-result-proves-nothing-about-the-search.md),
which is the same shape with the conclusion drawn from an absence instead of
from a coincidence.
