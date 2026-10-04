# A break that changes nothing is the loudest result

**Concluded**, of a test written that morning and named for exactly this: *the
options the verb offers and the names the file answers are two lists, and this
compares them both ways, so they cannot drift apart.*

**True.** Renaming the constant from `"dark"` to `"night"` — the way a rename
actually happens — left **all 107 tests passing**:

```rust
Offered::called(DARK, words::SCHEME_DARK)   // the offered option
DARK => Some(Scheme::Dark),                 // the answering arm
```

Both sides read `DARK`. Rename it and they move together. **There was one list
wearing the shape of two**, and a set comparison between them could never have
failed. The names are what a model on a machine already sends, and
`docs/contracts/agent-verbs.md` calls a verb's name *a stable identifier, never
reused for a different meaning* — so the rename this test could not see is a
contract change.

## Three outcomes, and the directory had already named two

[A fixture its writer would produce cannot test preservation](a-fixture-its-writer-would-produce-cannot-test-preservation.md)
ends with the cure: *break the thing and watch the **specific** new assertion
fail.* That is right and it is written down. What was missing is that a break
answers in three ways, not two:

```
the named test fails, alone     the test is real.  Proven.
everything fails                proves nothing about this claim
nothing fails                   the test cannot see the thing it is named for
```

All three happened in one afternoon. Letting the same hour through twice failed
exactly `the_same_hour_twice_is_refused_though_both_are_valid_hours`, and nothing
else — the good case. **Removing an option took six tests down**, because
`Verb::checked` refuses a choice of one, so the declaration failed and every test
panicked on its `unwrap`: a general failure, which proved only that the crate
still compiles its own invariants. And the rename failed nothing at all.

**The middle one is the trap**, because red looks like evidence. A break must be
the shape the thing would really drift into — a renamed constant, a dropped match
arm — not a shape that knocks the scaffolding over.

## The cure, where the usual one does not reach

For most claims, a better fixture fixes it: compare against something the
implementation did not produce. **That cannot work here**, because the two sides
are a constant and there is nothing else to compare it to. The only thing that
catches a rename is **the spelling written down a second time, deliberately**:

```rust
assert_eq!(SET_THE_SCHEME, "set_the_scheme");
assert_eq!(DARK, "dark");
```

That is duplication, and it is the point. Every other test reads these through
their constants and moves with them; this one place does not, which is what makes
a rename visible instead of silent. Both a renamed option and a renamed verb now
fail that test and only that test — checked, in both directions, before it was
trusted.

**The question to ask of any test comparing two things:** *where did each side
come from?* If the answer is the same place, the comparison is a mirror. It will
pass for ever and tell you nothing.
