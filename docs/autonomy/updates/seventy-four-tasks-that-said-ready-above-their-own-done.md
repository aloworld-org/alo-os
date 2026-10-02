# Seventy-four tasks that said *ready* above their own *Done*

**2026-09-27.** Every plan under `docs/autonomy/` was read for one
disagreement: a task whose `**Status:**` line does not say done, above a
`**Done, <date>.**` marker in its own body. **74 tasks across six plans** had
it. Each of those 74 status words now reads `done`. No other word of any plan
changed.

## What was wrong

A task in a plan says whether it is finished twice, in two places, and nothing
made them agree:

```
### 1. What an update is, and what it may never do

**Status:** ready. **Depends on:** nothing.

**Done, 2026-09-14.** `crates/alo-keeping-up`: `Digest` is a whole, lowercase
`sha256:` image digest, refused otherwise and read back through the same check;
```

The second line says the work is free to take. The fourth says it was finished
thirteen days ago, and then spends a paragraph describing the code that exists.
A reader who trusts the status line hands the task to somebody; a reader who
trusts the marker does not. The repository had already met this and written a
rule about it — `SHARED_MAIN.md`, *when surveying the plans for free work, key
on the `**Done,` marker and not on the status word* — after the mistake
"nearly handed a lane thirteen completed tasks on the same day this rule was
written".

That rule is good advice and it was doing the wrong job. It told every reader
to route around 74 wrong sentences rather than correcting them, and a rule that
asks every future reader to remember something is more expensive, every time,
than the 74 one-word edits it was standing in for.

Where they were:

| plan | tasks |
|---|---|
| `v0-01-delivery-plan.md` | 32 |
| `kernel-enforcement-plan.md` | 14 |
| `accounts-and-session-entry-plan.md` | 14 |
| `v0-5-the-machine-keeps-itself-plan.md` | 9 |
| `v0-5-documents-and-paper-plan.md` | 4 |
| `providers-and-models-plan.md` | 1 |

**Forty-six of the seventy-four are v0.01's** — a release that shipped, in two
plans still describing most of their finished work as available. The nine in
`v0-5-the-machine-keeps-itself-plan.md` and four in
`v0-5-documents-and-paper-plan.md` are the ones the v0.5 exit gate's
reconciliation named on 2026-09-26 as *two plans, fourteen tasks, reading
`ready` above their own published reports*. Reading every plan rather than the
two the gate happened to point at found five times as many.

## What was changed, and what was not

Only the word. `**Status:** ready. **Depends on:** 3.` became
`**Status:** done. **Depends on:** 3.` — the dependency list, the reasoning
after it, the acceptance criteria and every `**Done,` paragraph are untouched.
The diff is 74 insertions and 74 deletions, and every one of the 148 lines is a
`**Status:**` line. Two of them carried a sentence rather than a word —
`v0-01-delivery-plan.md` tasks 4 and 5 read *lane B's — scheduled in
`accounts-and-session-entry-plan.md` … Lane B's finishing handoff marks it done here*, which
lane B did on 2026-09-10 — and those now read *done — lane B's, scheduled in
…*, keeping the sentence.

**This did not decide that any work is finished.** The decision was made when
each task's own report was published and its `**Done,` paragraph written; what
this change did was copy it into the line that contradicted it. That is the
whole of why it is safe to do to another lane's plan: the evidence is on the
screen, one line below the line being changed, and `SHARED_MAIN.md` already
named that marker as the one to trust.

**One seventy-fifth status line was changed the other way.**
`v0-5-the-installer-plan.md` task 20 — a download that stops arriving — read
`ready` while being held by the owner's instruction until task 4 lands, with the
work so far off-repository on the development PC. It has no `**Done,` marker, so
none of the above applies to it; what applies is the rule that a held task says
`blocked` or `scheduled` **on the status line itself**, because that is the word
a surveying machine reads. Left as it was, the next lane looking for free work
would have taken it. It now reads `blocked — on task 4`.

Two tasks were looked at and **not** changed, because their status words are
right: `v0-5-devices-and-media-plan.md` tasks 1 and 4 are `scheduled`, carried
to v2 by the owner's decisions of 2026-09-26 and 2026-09-27. Reports exist about
what was learnt; the tasks are deliberately not done, and the status line says
so, which is the rule working.

## The finding this leaves

A status word and a `**Done,` marker in one section can disagree, and nothing
notices. 74 of them did, for months, in a repository with a citation check, a
palette check, a vocabulary snapshot and a reconciliation crate. It is
mechanically detectable in four lines of any language, which is the signature of
something that should never have been a rule.

`crates/alo-reconciling` is being extended to hold the v0.5 exit gate to the
definition and to the plans, and this is one of the things it will check: a task
whose status does not say done, above a `**Done,` marker of its own, is a
finding. After that change, the 74 cannot come back one at a time.

## The gate

`cargo fmt`, `clippy -D warnings`, the workspace tests, rustdoc and the
supervisor's three, all on this machine. Several tests read these plans —
`alo-opening`'s *converting waits on its decision*, `alo-keeping-up`'s *undoing
is decided before it is built*, `alo-citing`'s citation walk — so a status word
changed carelessly would have shown there.
