# A pause outlived the thing it paused

**Concluded:** *no lane can run the build-loop supervisor, because it publishes
straight to `main` and `SHARED_MAIN.md` keeps those publishers paused.* Said to
the owner twice on 2026-10-05 — once in a status report and once in a written
brief — and about to be carried to them by a second lane as *the thing standing
between three machines and an actual loop*.

**What was true.** The sentence relied on:

> *Existing `kernel-loop` and `dev-loop` publication commands still target
> `main`. Keep those publishers paused until separately adapted and verified for
> this workflow.*

And the adaptation it was waiting for:

```
0274cf31  2026-09-18 11:27:57  docs(workflow): use task branches and
                               serialize gated merges (#1)   ← the pause
14f18940  2026-09-18 20:15:40  feat(loop): the supervisor lands a task through
                               a branch and a pull request (#9)
```

**Eight hours and forty-eight minutes.** Since that evening,
`tools/kernel-loop/src/landing.rs` has opened with *"`main` is protected: nothing
is pushed to it directly"*, and `landed()` pushes `HEAD` to a branch of its own,
opens a pull request, waits for the checks `main` requires, and puts it in the
merge queue. `grep` finds **no** `push origin main` anywhere in the tool. The
condition was met the same day and the sentence stood for seventeen.

**The mechanism.** *A rule about code is a claim about code, and it expires when
the code changes.* Three things kept this one alive:

- **It named its own expiry condition, which made it read as careful.** *Until
  separately adapted and verified* is exactly what a conscientious pause should
  say. A reader who notices that clause reads it as *somebody thought about when
  this ends* and not as *somebody should check whether it has*. The better the
  sentence, the longer it survives.
- **Nothing re-reads a protocol document against the tool it describes.** The
  gates read code. No gate reads prose about code. This is the guard-that-cannot-
  fire pattern at one remove: not a note attached to something that cannot fail,
  but a note attached to **nothing at all**.
- **Its last line invited the mistake.** *Documentation does not change their
  executable behavior* is true and was written to stop somebody assuming a doc
  edit had fixed the tool. Read seventeen days later it does the opposite work:
  it tells you the paragraph is about executable behaviour, which is precisely
  the thing it had stopped being right about.

**And the reader's half, which is mine.** I quoted the line, cited it by file
and line number, and never opened `landing.rs` — one file, whose first sentence
contradicts the claim. I had spent the same session telling another lane that *a
mention is not a call* and writing up a scope that lived in a method rather than
a sentence. **Citing a line precisely is not reading what it describes**, and a
precise citation of a false claim is more persuasive than a vague one, not less.

**The cure.**

**A rule that forbids something because of how the code behaves is checked
against the code before it is quoted, not before it is written.** The writing is
when it is true; the quoting is when it may not be. That is one `grep` — here,
`push origin main` in the tool named — and it is the same cost whether the rule
turns out to hold or not.

**And when a rule names its own expiry, treat that clause as work rather than
as reassurance.** *Until X* means somebody has to notice X. If nothing is
obliged to, write down who, or expect the rule to outlive its reason by as long
as the project lasts. The repository already knows this shape for checks that
cannot fire; this is the same fault in a document that has no compiler.

**What it cost.** Three machines held off a working tool for a day, a
recommendation to the owner to adapt something already adapted, and two reports
that read as measured because they carried a file and a line number.
