# A test that asserted its own premise

**Concluded:** *this test cannot pass vacuously, because it checks its own
premise first* — written into the test's doc comment as the reason it was
trustworthy, and it was the reason it could not land.

The test proved that a search over `/sys/class/input` returns the
**lowest-numbered** keyboard rather than whichever one the directory happened
to list first. It could not choose the directory order, because that belongs to
the filesystem. So it asserted it:

```rust
// The premise, read off the filesystem this is running on.
assert_ne!(
    first, "event1",
    "this filesystem returns the lowest-numbered device first, so this test \
     cannot tell a sorted search from an unsorted one. Pick other names: …"
);
```

**What was true.** Not *the order was wrong* — the order was **never asserted**.
It was observed once, on one filesystem, and mistaken for a property. The test
passed, and it passed for a reason that had nothing to do with what it was
testing: APFS happened to return what the assertion wanted.

The premise is a property of the filesystem, not of the change under test, and
the two filesystems in this project's path disagree:

| where | what `read_dir` returns first for `event1 event2 event3` | result |
|---|---|---|
| macOS, APFS — where it was written | `event3` | premise holds, test is adversarial, **passes** |
| the hosted runner's filesystem | `event1` | premise fails, **the assertion fires** |

Measured from the merge queue's own log on 2026-10-21:

```
=== these failed and are NOT declared:
  alo-installing the_keyboard_a_person_has::tests::
    the_lowest_numbered_keyboard_is_chosen_whatever_the_directory_order
```

Nothing was wrong with the code. The sort was correct, had always been
correct, and the test that failed was the one written to prove it.

## The mechanism

**A test that cannot control an input can only accept it or assert it, and
both are wrong.** This is the fork, and both tines are in this directory
already:

- *Accept it* and the test may pass without exercising anything — the fault
  this directory keeps finding, and the one this test was written to avoid.
  Its own comment records the earlier attempt: a fixture of `event9 event2
  event11`, where `read_dir` returned the already-lowest `event2` first, so
  **removing the sort did not fail the test**.
- *Assert it* and the test passes on one machine and fails on another, for a
  reason that is not about the change.

Choosing the second tine felt like rigour. It converted *a check that cannot
fail* into *a check that fails on a correct answer*, which is louder and
therefore looks safer. It is not safer: a vacuous pass costs a missed bug,
while a false failure **stopped the merge queue**, and stopping the queue is
not a private cost.

**What made it expensive was where it failed rather than that it failed.** The
pull request's own head status was green — those gates really did pass on that
tree, on a machine where the premise held. The queue builds its *own* commit,
and that build failed. The two are different claims, and CLAUDE.md's *a gate
states which subject it answered* says so; this is an instance of it. Then the
workflow died **before** posting the required `alo/gates-on-a-runner` status,
so the queue sat at `AWAITING_CHECKS` waiting for a status that would never
arrive:

```
pos 1 AWAITING_CHECKS PR #897
pos 2 QUEUED          PR #899
```

Measured, because the first draft of this entry guessed and said *forty
minutes*:

| | |
|---|---|
| `#897` enqueued | 09:39:02Z |
| its queue build **failed** | 10:03:29Z |
| dequeued by hand | 10:29:10Z |
| **`#897` held position 1** | **50 minutes, 26 of them after its build had already failed** |
| `#899` enqueued behind it | 10:20:20Z |
| **`#899` blocked** | **9 minutes**, then merged at 10:44:35Z |

`#899` belonged to another lane and was green. **A test asserting a premise
about a filesystem blocked work that had nothing to do with it**, and the pull
request it was on read `CLEAN` throughout — which is why nothing surfaced it
until somebody read the queue commit's log rather than the pull request's
status.

The 26 minutes are the part worth keeping. A failed queue build does not
dequeue itself when the failure happened before the required status was posted:
there is no status to be red, only one that is absent, and absent reads as
*still going*.

## The cure

**Move the property to where the test can supply the input.** Order-independence
belongs to a pure function over a list of names, not to a directory walk:

```rust
fn the_event_numbers_in_order(names: impl Iterator<Item = String>) -> Vec<u32>
```

Then the adversarial order is an argument, on every filesystem, with no premise
to assert — and each case can start with a name that is not the lowest:

```rust
assert_eq!(named(&["event3", "event1", "event2"]), vec![1, 2, 3]);
assert_eq!(named(&["event11", "event2", "event9"]), vec![2, 9, 11]);
```

**Keep the end-to-end test, and say in its comment that it does not prove the
ordering by itself.** An honest weak test beside a strong one is worth more than
one test pretending to be both — on a filesystem listing `event1` first, the
walk would pass unsorted, and the comment now says so rather than implying
coverage it has not got.

**Prove the strong one.** Deleting `events.sort_unstable()` takes the suite from
`9 passed` to `7 passed; 2 failed` after a confirmed rebuild of the crate, and
restoring it returns `9 passed`. Under that mutation the end-to-end test also
failed here — which is exactly the filesystem accident its comment warns about,
and is not evidence that it would fail anywhere else.

**The general rule, which is shorter than the example:** when a test cannot
choose an input, the input is in the wrong place. Asserting a premise about the
environment to rescue a weak test does not rescue it; it relocates the failure
onto whichever machine disagrees, and the machine that disagrees is usually the
one that decides whether the change lands. It also covers the case this one did
not hit: a test that *can* control an input and takes it from the environment
anyway.

## The same fault in another lane, the same week

A peer lane found its own version while reviewing this one, and the pair is
worth one entry rather than two because the shared mechanism is sharper than
either example:

**both tests only ever asked whether something failed, never which thing.**

- This one asked whether the entries came back, not in what order.
- Theirs asked whether a screen opened, not **why it had not** — a four-case
  `NoScreen` was collapsed into an `Option` at the seam, so the case that says
  *"is there and will not open for writing: … this program is almost certainly
  not running as root"* arrived as a bare nothing. A person reading *no screen*
  would have gone looking for a missing framebuffer and changed the wrong
  thing.

**A test that checks a boolean where the code produces four cases is not a weak
test, it is a test of a different function**, and it will keep passing for as
long as the collapse lives in the code it is testing. Neither test could fail
on the fault it was written to cover, and neither author found their own: each
was found by the other reading it.
