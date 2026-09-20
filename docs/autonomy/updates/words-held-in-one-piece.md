# Words held in one piece: an index that holds half of what it did

**Date:** 2026-09-20
**Workstream:** v0.5 — the machine, measured
**Task:** 14, *Words held in one piece: an index in hand the size of its words* —
the last task of that plan.
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** **Apple M3 with 8 GB unified memory**, macOS 26.5.2; built, linted,
tested and **measured** in the Lima VM on that Mac — **Ubuntu 24.04.4 aarch64, 6
CPUs, 4 GB of memory**, kernel 7.0.0-31. Nothing here is ticked *on the machine*.
**Egress:** none.

## The number

Task 13 measured what an index holds and said where the memory goes: a word kept
as its own `String` costs **twenty-four bytes of place in the list** plus an
allocation of its own, which an allocator rounds up to about thirty-two bytes for
a word of seven or eight letters. The letters are the small part.

The words are now **two allocations per file** — every word joined end to end in
one piece, and where each one begins beside it — instead of one per word.

Measured against task 13's own folder, six long letters and three long logs:

| | |
|---|---|
| Held as separate words | **9 548 201 bytes** |
| Held in one piece | **4 724 993 bytes** |
| | **49 per cent** |
| Separate allocations removed | **301 455** |
| Index file | **3 218 102 bytes, unchanged** |

Both numbers come from **one run on one machine**, side by side. Task 13's
numbers were taken on the development PC under WSL, and comparing a number from
one machine with a number from another says as much about the machines as about
the change — so the before is measured here rather than quoted, by
`held_the_old_way` in the same test, in the same loop, over the same entries.

**And both numbers are the index's own count.** Neither can see what the
allocator rounded those 301 455 small requests up to. Task 13 named that as an
estimate rather than measuring it, and so does this: at roughly thirty-two bytes
for a word of seven or eight letters, it is another nine megabytes or so that
went with them. The measured halving is the floor of the saving, not the whole.

Per file, unchanged in shape and much smaller in fact: the letters hold 2 324 672
bytes for 151 455 words; the logs hold 2 400 321 for 150 000.

## How big this plan was, and how big it finished

This closes the plan, so the ratio it is closed with:

| | |
|---|---|
| Tasks when published, 2026-09-13 | **5** |
| Tasks at close, 2026-09-20 | **14** |
| Added after publication | **9**, all of them on 2026-09-13 and 2026-09-14 |

**Nine added to five, and every one within two days of publication.** That is a
third shape, different from both the ones recorded so far. The applications plan
grew five to eleven **over four days**, the work finding adjacent work as it
went; the access-and-language plan did not grow at all, because its subject was
fixed by somebody else's list. This one grew almost twice over, and then stopped
— nothing was added after 2026-09-14, and tasks 13 and 14 sat in the plan for
six days before anybody reached them.

That is a plan **rewritten rather than grown**: the five tasks published on the
first day were a sketch, and the nine that followed within two days were the
real plan being written down once somebody had read the code. It is worth
telling apart from growth-under-work, because it means something different for
an estimate. A plan that doubles in its first two days and then holds steady has
found its size; a plan that gains a task every few days has not. Counting the
remaining tasks is safe for the first and misleading for the second, and the
count alone cannot tell you which one you are looking at — only the dates can.

## What it is

`crates/alo-finding/src/kept_words.rs` — `KeptWords`, one piece and the places:

```rust
pub struct KeptWords {
    joined: Box<str>,      // every word, in order, end to end
    starts: Box<[usize]>,  // where each begins, and where the last ends
}
```

The words arrive sorted, so a search is still a **binary search** —
`KeptWords::says` — and not a scan. Nothing about how a search answers changed;
only what it reads. Every task 9, 11 and 13 test answers exactly as before.

## Three decisions worth the argument

**1. Eight bytes a place, not four.** A `usize` where a `u32` would do costs four
bytes a word — some 600 kB against nine megabytes saved. I wrote the `u32`
version first. It needs a bound on the total length and a branch for a file that
exceeds it, and **that branch cannot be reached, so it cannot be tested** — a
megabyte is three orders of magnitude below what a `u32` holds. My first draft
had that branch silently return *no words at all*: a search that quietly stops
finding things, guarding a case that never happens, untestable by construction.
The honest alternatives were no better — drop some words, or cut one in half.
Eight bytes buys the whole class away.

**2. One representation of nothing.** `none()` and `of(&[])` were two different
shapes — no places versus one place — that behaved identically and compared
unequal. My own test caught it. `Default` is now `of(&[])`, so there is exactly
one way to hold nothing.

**3. The old binary search is gone, not kept beside the new one.**
`wording::says` took `&[String]`; the search moved into `KeptWords`. Leaving both
would be two copies of one rule, which is the fault this crate keeps being bitten
by — and the one `alo-media-server` was built to end elsewhere.

## The public surface, and why it changed the way it did

The acceptance asked that a surface which has to change do so **additively, with
the old way kept and marked deprecated**. One part of that could not be honoured
literally, and it is worth saying why rather than quietly doing something else.

`Contents::words()` returned `&[String]`. **A reference to a slice of `String`
cannot be handed out by something that stores no `Vec<String>`** — keeping that
signature would mean keeping the representation the task exists to remove. So:

- **`Contents::kept() -> Option<&KeptWords>`** is the new way, and hands the
  words over without building anything.
- **`Contents::words()` is kept and deprecated**, and still answers — returning
  `Vec<String>`, at one allocation per word. The name and the meaning survive;
  the type does not.
- The variants keep the **field name** `words`, so `Contents::Read { words }`
  still pattern-matches. Only code that used it *as* a `Vec<String>` had to
  change, which in this repository is nothing outside `alo-finding` — checked,
  not assumed: no crate outside it matches those variants or calls that method.

## The index file is untouched, by construction

`Contents` writes through a separate `Writing` type and reads through `Reading`.
`Reading` still takes `Vec<String>` and converts on the way in, so **an index
written before this change still reads**; `KeptWords` serialises as the list of
words it always was, so a file written after is byte for byte what it was. The
measurement above shows the same 3 218 102 bytes before and after, and the test
that reads an index written in the older shape still passes.

## What it does not do

No `unsafe` — every byte range is taken with `get`, checked like any other slice.
No edit to `alo-files`. Nothing opens a socket, reads a clock or watches a
folder. `MOST_WORDS`, the verb, the list, the record and the words a person reads
are untouched.

## Evidence — tests this task publishes

`crates/alo-finding/src/kept_words.rs`, **7 unit tests**: words come back as they
went in; words that run into each other stay separate (`can`, `candle`,
`candlelight`); a search finds what is kept and refuses what is not; nothing kept
says nothing; every script survives the joining; what it holds is its words'
bytes and one place each; and it is written as a plain list of words.

`crates/alo-finding/tests/an_index_that_fits_in_hand.rs` gains the side-by-side
measurement and asserts, per entry, that the one piece holds **no more** than the
separate words did — so a change that made it worse fails rather than being
reported.

`alo-finding` in full: **79 unit tests and every integration test**, including
task 9's, 11's and 13's, unchanged.
