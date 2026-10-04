# A check fixed in one place leaves its copy

**Concluded**, after making a guard precise and watching it pass: *that check is
fixed.*

**True.** The flaw was in three checks, not one, and fixing the first taught me
nothing about the other two:

```
the gate guard        refused a commit because "clearest instance" appeared on a
                      line without the word Concluded. The phrase was inside the
                      **Concluded** block, two lines below its header, which is
                      how every entry here quotes the claim it retracts.
                      FIXED — made block-aware, and broken once to prove it
                      still caught the asserted form.

the post-merge check  the same grep -v 'Concluded', per line, left as it was.
                      After the merge it printed
                      "WARNING: it appears outside a Concluded block on main".
                      False. Nothing was wrong with main.

the note read-back    counted "the close button drawn on every window" per line
                      in Rust source. A trailing backslash in a string literal
                      removes the newline AND the next line's indentation, so
                      the sentence exists while no single line holds it.
                      Reported close 0, minimise 1, maximise 0. All three were
                      there.
```

**The mechanism.** A fix is made where the failure was felt. The other copies
are not failing *at that moment*, so nothing points at them — and a check only
announces itself when it fires, which is exactly when it is too late to be
treated as a copy. **Two checks that must agree, with nothing comparing them**,
is the same shape as two lists of names with nothing comparing them; it is just
harder to see, because both copies are mine and were written minutes apart.

The cost is specific: a verifier that errs toward refusing is safe, and **a
verifier that errs toward a false alarm about already-merged work is not** — it
invites exactly the wrong action, which is to go and change something correct.

**The cure, in order of how much it buys:**

1. **Do not write the check twice.** If it is worth guarding before a commit, it
   is the same function afterwards. A pattern copied is a pattern that will
   diverge.
2. **When a check is fixed, grep for its siblings before moving on** — the
   phrase it searches for, or the flag that was wrong. The fix is not finished
   at the first site.
3. **Never search wrapped text per line.** Markdown bold wraps, Rust string
   literals wrap with a trailing backslash that also eats the indentation, and
   prose wraps everywhere. Join first, then search. This repository already
   says so about `**`; it is the same rule for every other wrapped thing.

**This is the second repeat recorded in one day**, after
[the reason was above the line I judged](the-reason-was-above-the-line-i-judged.md),
and the pattern between them is worth more than either: **both cures were
written as sentences, and neither fired.** The one that now works is a script.
A rule I have to remember at the right moment is a rule that works on the days I
did not need it.

## The guard written for this entry failed the same way

The commit that carries this file runs a check over every entry: a `**` must
balance within its block, never within a line. It refused two files — this one,
and `a-rule-i-remember-is-not-a-rule-the-repository-has.md`, **which was already
merged and is the entry about that very rule.**

Both were correct. The check counted the asterisks inside an inline code span —
the ones written while *explaining* what balances — as though they were bold
markers. **A check that does not know the shape of what it reads** is the fourth
instance of that in one day, and it arrived attached to the entry describing it.

The fix is the same shape as the cure above: strip the fenced blocks and the
inline code spans first, then count. A rule about markdown has to know where the
markdown stops.
