# Count the production callers, not the tests

## What was concluded

> The alo Bar is finished. `Super`+`A` is bound, the composer takes the
> keyboard, `ComposerKey::of` reads a symbol, `a_key_reached_the_composer`
> carries it out, and `the_bar_finds` decides what a request means. Every
> layer is tested and green.

Each clause was true.

## What was true

**A person could not type a single character into the Bar.** Measured on a
booted machine on 2026-10-23: the chord fired, the composer took the keyboard,
twenty-nine characters were typed, and the field stayed empty. The shell's own
counter confirmed every key arrived and that it *could not make sense of 0
events* — read, understood, and handed to nobody.

```
$ grep -rn 'a_key_reached_the_composer' crates/ --include=*.rs | wc -l
18
$ grep -rn 'a_key_reached_the_composer' crates/ --include=*.rs | grep -c '_tests.rs'
18
```

**Eighteen mentions. Eighteen in tests. Zero production callers.** The same
count found two more the same day: `a_key_reached_a_task_window` (eight, all
tests) and, earlier, the provider's model — written into `WhatToSave`, read by
nothing that put it in the provider.

## The mechanism

**A test of a layer cannot see whether anything calls the layer**, and the
better the test, the more completely it hides the question.

The clearest case names itself. `a_chord_opens_the_bar_tests::what_is_typed_
after_the_chord_reaches_the_field` exists for exactly this risk and says so:

> **The next letter typed goes into the field**, which is the whole point: the
> chord is useless if the keyboard arrives somewhere a person cannot type.

and then proves it by calling the handler **directly** — which is the one step
a real keyboard could not reach. The test is honest, well-named, and about a
function. The machine's fault was about a road.

**Coverage cannot report this.** The line is executed, by the test. **Review
cannot either**: every file reads correctly, and the missing thing is not in
any of them. The absence has no location.

`crates/alo-shell/src/a_key_reaches_settings.rs` recorded this same fault being
fixed once before — *every link below this one was finished and tested, and
nothing on a running machine called either* — and the Bar had it anyway,
because that note lived in the file that fixed it rather than anywhere a
reader of the Bar would look.

## The cure

**For anything a person's input reaches, count its production callers before
believing it works.**

```
grep -rn '<handler>' crates/ --include=*.rs | grep -v '_tests.rs\|/tests/' \
  | grep -v 'fn <handler>\|///\|//!'
```

Zero is the finding. It takes one command and it is the only check that sees
this class at all.

**And hold the call site with a test that reads the production source.** A unit
test of an entrance passes with nothing calling the entrance, so the entrance
needs its own assertion:

```rust
assert!(shipped.contains("server.the_bar_is_taking_the_keys()"), …);
```

`crates/alo-shell/tests/a_key_a_person_types_reaches_the_bar.rs` does this, and
`a_request_becomes_a_window.rs` did it first. Carry a **positive control** in
the same scope — a marker the file is certain of — so a read that returns
nothing fails loudly instead of passing clean.

**The question to ask of any finished-looking feature**: not *is this tested*,
but **what calls it on a machine** — and then go and look.

## Related

- `a-check-whose-scope-misses-the-question.md` — the same family: a measurement
  narrower than its label.
- `a-service-that-starts-and-serves-nothing.md` — a thing that runs and is
  reached by nobody, found the same way.
