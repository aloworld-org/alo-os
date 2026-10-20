# A control that cannot fail, because something else is holding the thing up

**Concluded**, in a test written to prove a new safety rule works: *this control
shows that the rule is doing the refusing.*

The rule is that the disk the installer is running from is never offered to a
person choosing where to install. The control pointed the medium at a different
disk and asserted the stick would then be offered:

```rust
let elsewhere = TheMedium::ThisDisk("/dev/nvme0n1".to_owned());
let shown = what_may_be_offered(&disks, &elsewhere, &strings);
assert!(shown.offered.iter().any(|one| one.disk == "/dev/sdc"));
```

**True.** `crates/alo-installing/src/disks.rs:269` already refuses **any** disk
carrying the installer's label, whatever a caller thinks the medium is:

```rust
if disk.any(&|one| one.label.as_deref() == Some(THIS_INSTALLER)) {
    return Err(Unsuitable::HoldsThisInstaller);
}
```

So the stick is held up by two rules. Moving the medium cannot make it fall,
and the control was asserting a state that cannot exist.

## The mechanism

**It failed loudly, and it could just as easily have passed.**

Had the control been written the other way round — assert the stick *stays*
refused — it would have been **green**, and the conclusion would have been *my
rule works*, when what had been tested was somebody else's rule, written months
earlier, in another file. The new rule could have been deleted entirely and
every test would still have passed.

That is the shape: **a control proves a rule only where that rule is the only
thing holding the outcome up.** Two rules with the same effect are good
engineering and bad evidence, and the second one is invisible from the first
one's test. Nothing about writing the test reveals the overlap — the author is
looking at the code they just wrote.

It is the same family as a check whose scope misses its question, with one
difference worth keeping separate: there, the measurement answers a narrower
question than its sentence claims. Here the measurement is correct, the
sentence is correct, and **the attribution is wrong**. The outcome is real and
something else caused it.

## The cure

**Isolate by finding an input the other rule would accept.** The replacement
uses a disk that is empty and unlabelled — `disks.rs` is happy to offer it — and
then asserts it is refused when, and only when, the new rule is told the
installer is running from it:

```rust
let on_the_empty_one = TheMedium::ThisDisk("/dev/sdb".to_owned());
assert!(!what_may_be_offered(&disks, &on_the_empty_one, &strings)
    .offered.iter().any(|one| one.disk == "/dev/sdb"));
assert!(what_may_be_offered(&disks, &TheMedium::Unlabelled, &strings)
    .offered.iter().any(|one| one.disk == "/dev/sdb"));
```

Both directions on one input, where only the new rule changes between them.

**The question to ask of any control: what else could be producing this
outcome?** If the answer is *another rule with the same effect*, the control is
measuring the pair. The cheapest way to find out is the one that caught this:
**delete the new rule and run the test.** A test that still passes was never
about it.

## And the one beside it, three times in a day

Twice in the same evening — once in this lane and twice in the other, in two
different crates — a function or constant was inserted **above** an existing
one, and the existing item's doc comment silently became the new item's,
leaving the original undocumented. `rustdoc` caught one; clippy caught the
other two as *missing documentation for a constant*.

`inserting-above-an-item-steals-its-doc-comment.md` is already in this
directory and was written before any of the three.

**So the finding is not the trap. It is that a written-down trap was repeated
three times in one day by two lanes who had both read the directory it is in.**
A page cannot reach the moment of an edit. What reached all three edits was a
lint, in under a second, naming the file and the line.

**The cure is the same as the tooling rule elsewhere in this directory**: when a
mistake recurs despite being written down, the fix is a check that runs, not a
better paragraph. These three were already caught by the gate, which is why
they cost minutes — so the note to keep is narrower than *remember this*: it is
**insert below, not above, and let the lint be the thing that remembers.**
