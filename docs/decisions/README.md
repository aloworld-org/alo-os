# The decisions, and how this repository points at them

`CLAUDE.md` makes what is here binding: *read the ADR before proposing an
alternative*. Everything else in this repository therefore points at these
files constantly — a crate's rustdoc, a finding's own sentence, a promise's
evidence — and an ADR reference is the one kind of pointer a reader believes
without opening, because the number looks like a fact.

`crates/alo-citing` is what makes every one of those pointers land. It reads
this repository's own Rust and Markdown rather than a list kept beside them, so
a citation added anywhere is one it sees, and it fails in the change that adds a
pointer at a decision nobody wrote. What follows is the convention it reads,
which is therefore a rule rather than a habit.

## Writing a decision

- **The filename is four digits, a hyphen, a description and `.md`** —
  `0016-the-organisation-bounds-and-the-person-chooses.md`. The number is the
  next one unused; two files claiming one number is refused, because a citation
  of that number would resolve to whichever file a reader opened first and both
  of them would look like the answer.
- **The file records what its status is**, on a line beginning `**Status:**`,
  saying which of *accepted*, *proposed*, *superseded*, *rejected* or
  *withdrawn* it is. This is checked. An ADR nobody recorded as accepted or
  proposed is one every reader will read as settled — the citation resolves, the
  file opens, and the weight of a decision is taken by a recommendation still
  waiting on the owner.
- **A decision is never renamed on its own.** A rename leaves every link to it
  reading exactly as it did before, which is how
  `0019-a-runtime-is-found-not-configured.md` spent five days pointing at a file
  that had not existed since `0006-the-pinned-model-runtime.md` was renamed.
  That link is the reason this check exists in the shape it does.

## Citing a decision

- **By number: `ADR` and four digits** — `ADR 0001`, and `ADR 0001 §3` where a
  section is meant. `ADR-0015` and the plural `ADRs 0022, 0023` are read too,
  and a list continues, so every number in `ADRs 0001, 0008, 0016 and 0024` is
  checked rather than only the first.
- **By filename**, either as `docs/decisions/0024-what-a-person-signs-in-at.md`
  from anywhere, or as the bare filename a link inside another decision uses.
  Both are checked against what is actually in this directory.
- **A section is not checked, deliberately.** `ADR 0019 §3` is a numbered
  heading, `ADR 0020 §2` is not one at all and `ADR 0004 §policy` is not a
  number. The check judges the pointer, never the argument; whether an ADR says
  what a citation claims is a reader's job and nothing mechanical reaches it.

### Another repository's decisions

`alo-workplace` has ADRs of its own, and this repository cites four of them.
**Name the repository on the same line, before the number**, the way
`docs/contracts/agent-verbs.md` does with `alo-workplace` ADR 0047. A bare
number is read as this repository's and is refused when nothing here answers for
it, which is exactly how a reader would read it too — and if the name wraps onto
the line above, the check refuses it, because so would a reader skimming for
which repository they are meant to open.

### A pointer in an example

A citation of a decision nobody wrote is refused wherever it is written,
including inside a test that is *about* a dead pointer — and that is deliberate,
because a reader who finds one in a fixture cannot tell it from one in a
sentence. Where a test needs a pointer that must not resolve, it assembles it
rather than writing it out; `crates/alo-citing/tests/` and
`crates/alo-reconciling/tests/` both do, and each says why beside it.

## What is not here

No decision is summarised in this file. A list of what each ADR says would be a
second place to keep them up to date and the first place to read something that
is no longer true. `docs/features.md` is what gets built, `ROADMAP.md` is the
order, and the decisions are their own argument.
