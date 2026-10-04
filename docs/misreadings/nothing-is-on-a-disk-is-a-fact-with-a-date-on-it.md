# Nothing is on a disk is a fact with a date on it

**Concluded:** *a format bump in `alo-arranging` needs no migration, because no
file of this shape has ever been written to disk.* The crate's own `FORMAT`
doc comment says it three times and reasons from it: *`written` had no
production caller when 1 was defined, none when 2 was, and none when 3 was — so
there is no migration path here and no need of one.*

**True when it was written, and false now.** `crates/alo-desktop/src/main.rs`
calls `alo_arranging::keeping::at_sign_in` at line 294 and
`alo_arranging::keeping::keep` at line 318, neither under a `cfg(test)`, both in
the desktop a person signs into. **Version-3 files are being written to real
disks**, and alo OS 0.0.1 is pushed, signed and public.

So the sentence a reader meets while deciding whether to bump the version is a
claim about the state of the world **in the past**, written in the present
tense, in the one place somebody would rely on it.

**The mechanism.** The reasoning was sound and the fact underneath it moved. A
doc comment records why a decision was taken; it is read as a description of
how things are. Nothing fails when the second drifts from the first — the
comment keeps compiling, and the next change inherits a conclusion whose
premise has quietly expired. This is the same shape as a measurement published
as a standing property, one level further out: there the number went stale,
here the *reason* did.

It is also what made it hard to catch. The comment is unusually careful — it
distinguishes the three versions, says what each one bought, and explains why
the version moved for 3 when it strictly need not have. Care reads as currency.

**The cure.** A claim about what exists outside the repository — a file on a
disk, a caller somewhere else, a machine in a state — is a **measurement**, so
it carries the date it was taken and the command that would take it again.
*Nothing is on a disk to be migrated* becomes *as of 2026-10-02, `written` had
no production caller:* `grep -rn 'keeping::keep' crates --exclude-dir=target`.
Then the next reader runs it rather than believing it.

And where the claim decides something expensive — whether a person's kept file
survives an upgrade — the check belongs in a test rather than in prose: *this
crate writes a file that something reads back* is assertable, and so is *the
version this crate refuses is one nothing wrote.*

*Found 2026-10-04 by the Mac lane, about its own crate, while scoping the format
bump that canvas task 8 needs. Nothing was broken by it: the comment was read
before it was relied on, which is the only reason this entry is a misreading
rather than a person's lost layout.*
