# A guard that cannot fire is a comment

**Concluded:** *this is written down where the next person will be stopped by
it, so the next person will be stopped by it.* Two notes in `alo-shell` and
`alo-dividing` say almost the same sentence about almost the same kind of
change, and only one of them can stop anybody.

```
alo-dividing/src/keyboard.rs   "Written out rather than caught by a wildcard,
                                because the next action added should make this
                                file refuse to compile until somebody has
                                decided which it is."

alo-shell/src/display_lifecycle.rs
                               "The display is numbered 1 because there is only
                                ever one; the moment a second output is
                                advertised, the number comes from whatever
                                advertises it and this constant goes."
```

**The first fired on 2026-10-04 and the second cannot.** Adding
`Action::GoBackOnThisPlace` broke `alo-dividing`'s exhaustive match, the build
stopped, and a decision was written into the arm — exactly as its note
predicted, without anybody remembering the note existed. `THE_DISPLAY` is a
`const` with one reader; a second output appearing would not disturb it, and
nothing in the repository makes a second output appear. It has been correct and
inert since the day it was written.

**Both notes are true and only one is a guard.** The difference is not care,
length or placement — the `display_lifecycle.rs` one is the more carefully
argued of the two. It is **what the compiler is obliged to check**: a match that
must stay exhaustive is a claim re-tested on every build, and a constant beside
a sentence about the future is a claim nobody will test until the future
arrives and somebody happens to read that file.

**The mechanism.** Writing the condition down *feels* like arming it, because
the hard part — noticing the assumption and naming when it expires — is the part
the author did. The remaining step is mechanical and invisible when skipped:
attaching the sentence to something that fails. Nothing goes wrong meanwhile.
The note keeps compiling, keeps reading as diligence, and is the first thing
quoted when somebody asks whether the assumption was known about — *yes, it is
documented* — which is true and is not the question.

**It is this directory's standing subject one level out.** A check that stands
in for the thing is a test that passes for the wrong reason. This is a check
that was never a test: there is no green here to interrogate, which is why
*what would have to be true for this to pass while the product is broken* does
not catch it. The question that does is narrower and should be asked of every
note that names a future change:

> **What would have to run for this sentence to stop anybody — and does that
> thing run?**

For `alo-dividing`: a build, on any change that adds an action. It runs on every
commit. For `display_lifecycle.rs`: a reader opening the file while adding
multi-output support. It runs when somebody is already doing the thing the note
is about, which is the one moment they do not need telling.

**Found by the laptop lane**, reading the two notes side by side after the first
one fired: *that one cannot fire by itself, because nothing makes a second
output appear.* Recorded by the Mac lane, whose task 9 the inert note is about —
*every screen is a view onto the canvas* needs two viewports, and the compositor
advertises one output across eighteen files that say so in prose.

**What to do about it is not "add a test for every note".** Most notes about the
future are correctly just notes. The ones worth arming are those naming a change
somebody will one day make *to this file's own assumptions*, and the cheapest
arming is usually already available: an exhaustive match, a declared array
length, a count asserted against a list. Each of those is a sentence the
compiler or a test has to re-read. A `const` with a paragraph beside it is a
sentence nobody has to read at all.
