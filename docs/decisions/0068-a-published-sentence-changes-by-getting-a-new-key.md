# ADR 0068 — A published sentence changes by getting a new key, never by being edited under the old one

**Status:** proposed, 2026-09-26.

Written by task 13 of `docs/autonomy/v0-5-the-session-and-the-displays-plan.md`,
which cannot be built until it is answered: it has to change a sentence a person
reads, and nothing in this repository says whether that is allowed or how.

## The decision in one line

The key of a user-facing string is a public surface, because every translation is
keyed to it — so **a sentence whose meaning changes gets a new key and the old key
is retired**, and only a change that leaves the meaning alone may be made under the
key it already has.

## What forced the question

Session task 12 walked one person from her own desk to the office and back, and
recorded what she reads
(`docs/autonomy/updates/every-sentence-at-a-desk-that-changed.md`). On the return
leg `displays.the-desk-changed` says:

> Your screens have changed since this machine went to sleep, so alo OS has set up
> the ones in front of you now — **the arrangement you made at the other desk is
> still here for when you are back at it**

She is back at it. The clause promises that something is waiting for a moment that
has arrived, to a person standing in it. It is apt on the outbound leg — a laptop
closed at home and opened at the office, which is the case task 11 wrote it for —
and inapt on **every return leg of every journey**.

Fixing that means either editing a sentence that is already published or splitting
it in two. There is no rule for either. `docs/contracts/` has twenty-odd contracts
and none of them is about the vocabulary; `alo-strings` states no rule; no ADR
mentions one. Three earlier commits have changed a `Word`'s text with nothing to
consult.

## Why this is a rule and not a judgement

**A translation is keyed, and a key says nothing about what it once meant.**
`alo-saying` assembles every crate's words into one vocabulary and a translation
supplies text per key. So editing the English under a published key does not
invalidate a translation — it leaves it **rendering perfectly and saying something
the product no longer says.** Nothing fails, nothing warns, and the machine tells
somebody a thing that is not true in their own language while telling an English
reader the truth.

Three things make that worse than an ordinary bug.

- **The first target is all 24 official EU languages** (`CLAUDE.md`), plus any
  language somebody contributes. The more translations exist, the more copies of
  the old meaning are in the field.
- **The person who meets it is the person least able to report it.** An English
  reader sees the corrected sentence. The reader of a contributed translation sees
  the stale one and has no way to know the English moved underneath it — and, by
  the standing rule that the languages nobody else serves are the ones this
  product is for, they are exactly who must not be told something false.
- **It is the defect this repository keeps catching, one level along.** A check
  standing next to the thing: the key resolves, the string renders, the machinery
  is satisfied, and the sentence is wrong. `docs/quirks.md` records the same shape
  three times over — a guest filesystem's free space standing in for the drive
  under it, `test -x` standing in for *the converter runs*, a `$?` read through a
  shell that expanded it first.

And `CLAUDE.md` already decides the general case: *Contracts outlive code … they
change additively, and a break requires versioning and deprecation.* A string key
is not on its list of public surfaces, which is why this is being written down
rather than assumed — but a third party translating against our keys is building
against ours exactly as an adapter author is.

## The decision

1. **A key's meaning is fixed once it is published.** A sentence whose meaning
   changes — narrowed, widened, split, or conditioned on something new — is a
   **new key**. The old key is retired in the same change.
2. **An edit under the existing key is allowed only where the meaning is
   unchanged**: a typo, grammar, punctuation, a clearer word for the same fact, or
   the translator's note (which is instruction to a translator, not a claim to a
   person). If a translator would have written the same sentence in their language
   before and after, it is the same meaning.
3. **Splitting a sentence is two new keys and one retirement**, because the
   remaining half no longer means what the whole did. It does not matter that the
   text of one half is a substring of the original.
4. **Retiring a key means deleting it.** Keys are not persisted — nothing in
   `docs/contracts/` names one, nothing writes one to disk, and `Word::key` is
   read at the moment a sentence is said. So a translation of a deleted key is
   simply never consulted, which is the harmless outcome; keeping the key alive
   with a marker would keep the stale meaning reachable, which is the harmful one.
5. **A new key is not a licence to reword the neighbours.** The change that splits
   a sentence changes that sentence and nothing else. A sentence somebody wants
   phrased differently is a finding with its own argument, not a passenger on this
   one.

## What this does not decide

- **Who may change a sentence.** Ownership is unchanged: the crate that declares a
  word owns it, and a crate reaching into another to coordinate phrasing would put
  a second author on a sentence that already has one. This ADR says what a change
  looks like, never whose it is.
- **Whether translations are tracked.** Nothing here adds a manifest of which keys
  are translated into what, or a report of which ones a change orphans. That may be
  worth having and is not needed to stop the silent lie — a retired key orphans its
  translations loudly, by their never being used again.
- **Anything about the sentence that forced this.** Whether
  `displays.the-desk-changed`'s reassurance belongs in that sentence is session
  task 13's argument to make under whichever rule is accepted here.

## Consequences

**A sentence that turns out wrong costs translator work.** That is the point: the
work is where it can be seen, rather than a translation quietly meaning the old
thing. The alternative is not cheaper, it is only quieter.

**Key churn is visible in review.** A change that adds a key and deletes one is
legible in a diff in a way that an edited string literal is not, which is the
second reason to prefer it — the reviewer who should be asking *did the meaning
move?* is asked it by the shape of the change.

**`alo-saying`'s count moves.** Every crate's word list is fixed-length and
`collecting.rs` keeps four hand-kept lists; a split moves those counts. That is
noise the software plan's task 10 is already for and is not an argument against
this.

## Alternatives rejected

**Edit strings freely; treat translations as best-effort.** What happens today,
for want of a rule. It is rejected on one ground: it makes the machine say
something false in a language the author of the change cannot read, with nothing
failing. For a product whose case is that it serves the languages nobody else
serves, a silent mistranslation is not a smaller fault than a visible one, it is a
larger one.

**Version the vocabulary**, so a translation declares which version it was made
against. Heavier, and it records that a lie may exist rather than preventing one:
a translation made against version 1 still renders under version 2 unless
something refuses it, and refusing it means a person reads English instead — a
worse outcome than the honest new key, and a version number is not needed to reach
it.

**Keep the old key, marked deprecated, alongside the new one.** Rejected for the
reason a retired grant is removed rather than flagged: a stale meaning that is
still reachable is still reachable, and the marker is read by whoever maintains the
code rather than by the machine that says the sentence.
