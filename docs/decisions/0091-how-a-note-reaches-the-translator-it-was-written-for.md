# ADR 0091 — How a note reaches the translator it was written for

**Status:** accepted, 2026-10-04, by the Mac lane under
`docs/autonomy/a-new-machine-becomes-a-lane.md`'s rule for a blocker that is a
decision. The language half of `docs/autonomy/access-and-language-plan.md` is
this lane's, and the lane that found this said it wanted a decision rather than
a commit from whoever happened to notice.
**Date:** 2026-10-04
**Context:** `CLAUDE.md` — *user-facing strings are externalised from day one,
the first target is all 24 official EU languages, and hardcoded English is a
bug*; `docs/contracts/translations.md`; `crates/alo-strings` (`Word::noting`,
`Word::note`); `crates/alo-saying` (`the-vocabulary.txt`, `src/rented.rs`);
every crate's `src/words.rs`.

## The question in one line

**Every word this system says carries a note written for a translator, several
crates refuse a word that has none — and no artefact a translator can obtain
contains one. Where does the note go?**

## What was measured, 2026-10-04, on `dbb1938d`

```
the-vocabulary.txt        key <TAB> English.      Generated. No notes.
what_this_machine_says()  phrase.key(), phrase.source()  — note() is not read
Word::note() readers      tests asserting one exists; a search index; rented.rs
translations on disk      /usr/share/alo/translations/*.toml, written by a person
```

Three facts, each checked rather than recalled:

1. **A note is mandatory.** Crate after crate asserts `word.note().is_some()`
   for every word it declares — `alo-greeting`, `alo-changing`, `alo-egress`,
   `alo-granted`, `alo-converting` and others. Writing one is enforced at the
   moment of writing.
2. **A note is not exported.** The snapshot is the only generated artefact that
   carries the vocabulary outward, and it carries two columns. Nothing else
   writes a note anywhere a person outside this repository could read it.
3. **The contract says it is what a translator writes from.**
   `docs/contracts/translations.md` — a public surface, and *the only one a
   person outside the organisation that owns the machine types* — says every
   string is declared *with its English and a note to the translator beside
   it*, and later that the rented-name check holds *the notes with it, because
   a note is what a translator writes from.*

**And a production refusal is reasoned from the same belief.**
`alo_saying::rented` refuses a third-party name appearing in a note, in these
words: *a translator working on {key} would meet {name} and write it into every
language alo OS is translated into.* That sentence describes a path that does
not exist. The guard is right to exist and its stated reason is false today —
which is the shape this repository has been finding all day, here in a refusal
rather than in a comment.

So the position is not *the notes are missing a nice-to-have*. It is that
**this repository mandates a thing, contracts that the thing is delivered, and
guards the thing's content — and never delivers it.**

## Why it matters more than it looks

The notes are not decoration; they are the cases where key-and-English is
actively misleading. Three from today alone:

- `applications.called` — *{called} is the name the application gives itself,
  in whatever language it was packaged in, and **is not ours to translate***. A
  translator seeing the English alone translates it.
- `applications.ours.settings` — *unlike every other application name this is
  ours to translate*. The exception is **only** in the note.
- `shortcuts.action.*` — since ADR 0089 these are read aloud as well as drawn,
  so a translator choosing a terse label changes what a screen reader says.

A translator working from two columns gets each of these wrong, in the
direction that looks correct.

## The options

**A — the snapshot grows a third column.**
- *Costs:* `the-vocabulary.txt` exists to catch a published sentence changing
  under a key. Notes change for reasons that have nothing to do with the
  sentence, and every note edit would become a diff in the file whose job is to
  make a sentence's movement visible. It would also make a translator's source
  a file whose header says *GENERATED. Do not edit by hand*, which is true of
  the artefact and wrong as an instruction to the person it is for.

**B — a generated file for translators: key, English, note.**
- *Costs:* a second generated artefact to keep in step, which a test does.
- *Gains:* the snapshot keeps its one job.

**C — admit the notes are for readers of the code.** Drop the contract's
sentence, keep the notes as developer documentation, and leave translators with
key and English.
- *Costs:* it decides that a translator may not be told *this one is not ours
  to translate*, which is `CLAUDE.md`'s law failing in the specific way it
  warns about: the languages served worst are the ones with least software of
  their own, and they are served by whoever has the least context.

**D — the generated artefact is a translation template**: a valid, complete
`.toml` of the shape `docs/contracts/translations.md` already specifies, with
each note as a comment above its key and the English as the value.
- *Costs:* the same as B, plus the template is large — four hundred-odd keys.
- *Gains:* it answers a question nothing answers today, which is **where does a
  translator start.** The contract describes the file's shape and says nothing
  about where a person gets the list of keys to fill. A template is the list,
  in the shape it will be submitted in, with the notes already beside the keys.

## The decision

**D.** `alo-saying` generates a translation template alongside its snapshot:
every key, its English as the value, its note as a comment above it, in the
TOML shape `docs/contracts/translations.md` specifies. A test regenerates it
and fails when it is stale, the way the snapshot's does.

1. **The snapshot is unchanged.** It keeps its one job, which is to make a
   published sentence's movement visible under a key.
2. **The template ships where a translator can get it**, and
   `docs/contracts/translations.md` gains the sentence saying where — the
   contract currently describes the file a translator writes and never says
   where its keys come from.
3. **The rented-name guard's reasoning becomes true** rather than aspirational,
   because the note now does reach the language files it describes.
4. **A note stays mandatory**, and that rule now buys something: the template
   has no blank comments in it.

## What it costs

- **A second generated artefact**, and the discipline that goes with one: it is
  regenerated rather than edited, and a stale one fails a test.
- **Four hundred notes become public**, written in a hurry over weeks by people
  addressing an imagined reader. Several will read as internal. That is a cost
  worth paying once rather than a reason to keep them private, and the first
  pass over them is work this decision creates and does not do.
- **It does not translate anything.** No language arrives because of this. What
  arrives is the possibility of one being translated correctly by somebody who
  was not in the room.

## What would change this decision

A translation workflow that reads the vocabulary directly — a service, a
submission form, anything that is not a person editing TOML. Then the template
is scaffolding for a road nobody takes, and the notes should go wherever that
workflow reads from. The contract's shape is what makes a file the answer
today.
