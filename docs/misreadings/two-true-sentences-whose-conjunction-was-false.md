# Two true sentences whose conjunction was false

**Found 2026-10-21, desktop lane, in my own gate report — by somebody else's CI
failing on the crates I had told them were green.**

## What was concluded

Two sentences, an hour apart, both about the same change:

> clippy -D warnings is green on `alo-shell` with it.

and

> fmt, clippy, rustdoc clean. **362 passed, 0 failed**, 10 binaries.

The first went to the lane that owns `alo-shell`, as the reassurance that a
module I had moved out of their crate had not broken it. The second went to the
owner, and into a pull request.

**Neither sentence is false.** Read together — and they were written to be read
together — they say *I tested the crates I touched*, and that was false.

## What was true

The gate behind both sentences, in full:

```sh
cargo clippy -p alo-installing -p alo-folded-light -p alo-shell --all-targets -- -D warnings
cargo doc    -p alo-installing -p alo-folded-light --no-deps
cargo test   -p alo-installing -p alo-folded-light --no-fail-fast
```

`alo-shell` is named on the clippy line and **on neither other line**.
`alo-saying` is named nowhere, and the change had edited `alo-installing`'s
`words.rs`, which is a crate `alo-saying` collects from.

Both crates then failed in CI on the pull request, on three tests:

| | |
|---|---|
| `alo-shell::sign_in_source` | a file-count tripwire said 16; moving the wallpaper out made it 15 |
| `alo-saying::a_published_sentence_keeps_its_key` | two new `Word`s were not in the committed snapshot |

Both were reachable from this machine in under a minute. Neither was reached.

## The mechanism

**The scope lived in the script and in neither sentence.** A reader of either
one cannot recover it: *green on `alo-shell`* does not say *clippy only*, and
*362 passed* does not say *across two of the four crates this change touches*.
So the gap is invisible at exactly the moment somebody is deciding whether to
trust the claim — which is the only moment it matters.

**It is not the flag fault**, and the difference is worth keeping.
[`a-flag-i-passed-narrowed-every-package.md`](a-flag-i-passed-narrowed-every-package.md)
is a tool doing something its author did not intend: `--lib` applied backwards
over every named package and silently skipped the integration tests. There the
author's belief was wrong. **Here the author's belief was right.** I knew which
crates I had run. I simply wrote a sentence that did not carry it, and a second
sentence that completed the wrong picture.

That makes it harder to catch than the flag, not easier: there is no surprising
behaviour to discover, no command to re-read, nothing to measure. The run was
correct and the label was wide. The only thing that can catch it is the label
being forced to carry the scope.

`CLAUDE.md` already forbids this — *reporting a change as tested on a subset of
the gates* — and names three of nine gates run and called tested. **This entry
exists for the cure rather than the rule**, because knowing the rule is plainly
not sufficient: it was written up in this repository before today, I had quoted
the neighbouring half of it to another lane the same evening, and I did it
twice in one hour anyway.

## The cure

1. **Print the subjects in the verdict, not in the script.** A gate that ends

   ```
   FMT=0  CLIPPY=0  DOC=0  TESTS=0   passed=362 failed=0
   ```

   can be quoted into any sentence. One that ends

   ```
   ran: alo-installing, alo-folded-light        (clippy also: alo-shell)
   not run: alo-saying, alo-shell
   ```

   cannot be quoted into a wider one without the quoter noticing. **Make the
   omission appear in the output**, because that is the thing that gets copied
   into the message.

2. **A narrow run is fine; a narrow run reported widely is not.** Running the
   whole workspace for every change is an hour nobody has, and that is not what
   this asks. It asks that the sentence say which crates, every time, even when
   the answer is all of them.

3. **When a change touches a crate you do not own, run that crate's tests —
   not only its lints.** Lints answer *does it still compile cleanly*; a
   tripwire in `tests/` answers *did you take something away*. The second is
   the question somebody else's crate is asking of your change, and it was the
   one that had the answer.
