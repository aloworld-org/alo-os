# A reader we wrote agreed with the tree we wrote

## What was concluded

`crates/alo-shell/tests/the_tree_a_reader_finds.rs`: *"read back with
`alo-adapters` — the agent's own reader … which asks the questions a screen reader
asks. What is asserted is what crossed the bus."* Its tests were green, and the
tree was taken to be one a screen reader can read.

## What was true

On a booted machine on 2026-11-13, Orca logged *strange signature so* for every
child it asked alo's shell for, and read each of those children as nothing.
`GetChildAtIndex` was replying with two loose arguments, `so`, where
at-spi2-core's `Accessible.xml` and libatspi both require one structure, `(so)`
(`docs/quirks/libatspi-reads-an-accessible-reference-only-as-one-structure.md`).
The test had asked exactly that question, `GetChildAtIndex`, for every node, and
had passed.

## The mechanism

**Both ends were built on the same library, and the library forgives the
difference.** The shell answers with zbus and `alo-adapters` reads with zbus. When
zbus reads a reply into the Rust tuple `(String, OwnedObjectPath)`, it accepts
both a body of `so` and a body of `(so)`. That is how `alo-adapters` read GTK's
correct replies and alo's wrong ones the same way. *That zbus accepts both is
inferred, not measured on its own:* the same `deserialize::<(String,
OwnedObjectPath)>()` call is green against GTK's `(so)` in
`alo-adapters/tests/the_accessibility_fallback_on_a_real_application.rs` and
against alo's `so` in `the_tree_a_reader_finds`. So the question did cross the
bus, but the one thing that was wrong with the answer could not be seen through
that reader.

"What crossed the bus" was true and was not the claim that mattered. The claim
that mattered was "a screen reader can read it", and the reader standing in for
the screen reader was one we wrote, sharing the fault's own library. This is the
constitution's *a check that stands in for the thing*, in a new form. The stand-in
was not a narrower check. It was **a second implementation that agreed with the
first because it was the same implementation underneath.**

It was **invisible**. Nothing in the repository caught it, and it was found by a
person switching the screen reader on.

## The cure

- **Where a surface is read by a rented program, test it with that program's own
  library**: libatspi for a screen reader, the real client library for any
  protocol. A reader we wrote is fine for checking what we meant. It cannot check
  what the other side accepts.
- **Ask what the two ends have in common.** If our producer and our consumer share
  a serialiser, a parser or a schema, any fault in that shared part passes
  between them. Check the wire against the specification's own text, such as the
  signature string in the XML, not against our own decoding of it.
- **Hold the rented library's warnings to silence.** libatspi said exactly what was
  wrong, on stderr. A test that fails on any line there catches the next fault of
  this kind without anyone having to predict it:
  `crates/alo-libatspi-walk/tests/libatspi_reads_alos_tree.rs` does
  that on every gate.
