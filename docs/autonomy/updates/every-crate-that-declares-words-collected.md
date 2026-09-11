# Every crate that declares words, collected — and the one that is not, named

**Date:** 2026-09-11
**Workstream:** v0.01 delivery plan, task 17 (`docs/autonomy/v0-01-delivery-plan.md`)
**Contributor:** Claude (separate checkout, `C:\dev\alo-os-claude`)
**Status:** ready for integration

## What was wrong

`alo-saying` is the one vocabulary every word a person reads is collected into,
and it exists for a reason that is not tidiness: a translation is **checked
against the vocabulary it is loaded into**, so a process holding only its own
strings would read a translator's correct line for another part of the system as
a mistake.

What it collected, though, was a **hand-written list** — a `EVERY_LIST` constant
and one `declare` call per crate, kept beside the crates rather than derived from
them. A crate that declares words and is not on that list compiles, passes
clippy, passes its own tests, ships, and says nothing to anybody in any language:
every sentence in it reaches a person as a bare key.

That is not hypothetical. Task 3 records it happening: `crates/alo-overlay`
declared nine strings and `alo-saying` collected nothing of them. It was found by
a person reading — the same way `docs/features.md`'s six missing promises were
found six times over before `alo-reconciling` made it arithmetic, and the same
way a verb with no by-hand answer would have arrived before `alo-by-hand` did.

## What changed

**`crates/alo-collected`** is the same answer for words. It is a repository check
like `alo-reconciling` and `alo-by-hand`: it says nothing to a person, declares
no strings, has no `src/words.rs`, and `alo-saying` does not collect it — which
the check itself now measures rather than asserts
(`the_check_itself_declares_no_words`).

- `declaring.rs` — which crates declare words, **read out of the workspace**.
  The convention it reads is `src/words.rs` with a `pub fn declare_into`, which
  twenty-three crates already follow. The workspace's own member list is parsed
  by `alo_by_hand::declaring::members_of` rather than by a second copy of that
  parser (see *Decisions*).
- `apart.rs` — the floor under the reason a crate gives for standing outside the
  vocabulary. Forty characters, the same floor `alo-by-hand` and
  `alo-reconciling` put under a sentence, for the same reason.
- `finding.rs` — seven findings, each naming the crate it is about and what to do
  about it, in the English of whoever is adding a crate with the repository open.
- `holding.rs` — `held`, which reads no disk: it is handed the two lists, the
  manifest's text and a way to read a file by its repository-relative path.

**`crates/alo-saying`** gained one additive public surface:
`collecting::DELIBERATELY_APART`, the crates that declare words and are
deliberately **not** collected, each with the reason. One today, and the argument
for it was already in that file's own documentation and in its `Cargo.toml`
comment — what is new is that it is now in a form a check can read, and that it
is held to being a reason rather than a name.

**`docs/contracts/translations.md`** gained the code-side half of *a key names a
string the running system says*: where a crate declares them, that `alo-saying`
collects all of them, that `crates/alo-collected` fails the build in the change
that breaks either half, and that a crate which cannot be collected is named with
its reason. This is where whoever adds the next crate meets the rule — the same
place rule 7 of *adding a verb* does the job for `alo-by-hand`.

### The one that is not collected

`alo-agentd` declares three strings and stands outside the one vocabulary,
because it is Linux and every module in it is compiled out anywhere else: a
vocabulary assembled from it would hold three fewer strings on a host with no
daemon than on a machine with one, so one host would refuse a translation file
the other accepted — the exact failure `alo-saying` exists to prevent, in its
platform-shaped form. It declares its own three on top of the machine's.

So the check takes exceptions, and this is the part worth arguing about: **an
exception with no reason beside it is indistinguishable from the failure the
check exists for.** A name on that list is a standing permission for a crate to
say nothing to anybody in any language; the difference between that and a bug is
entirely the sentence. So the sentence has a floor, and
`an_exception_with_no_reason_is_refused` is what keeps it one. An exception that
is no longer needed — a crate of that name with no words — is a finding too,
because it would still be standing on the day somebody does declare words there,
and then it is silence with permission attached.

### What the check found about this repository

Nothing wrong, which is the honest answer and is why the measurement matters:
twenty-three crates of this workspace declare words, twenty-two are collected,
one is named apart, and `declaring == collected + apart` on the disk the test runs
on. The list beside `alo-saying` was correct on the day this was written. It is
now correct because something checks it, and the next crate that is not on it
fails the build in the change that adds it.

## Decisions, and why

**A crate rather than a document.** `alo-by-hand` needed `docs/by-hand.md`
because its answers are prose a person writes and another person reads. Here both
lists are already code — `EVERY_LIST` is beside the `declare` calls it must agree
with — so a document would have been a third list to drift. The exception's
reason lives with the list it excepts from, in `alo-saying`, not in a file of its
own.

**The manifest is parsed once.** `alo-collected` depends on `alo-by-hand` for
`members_of`. A second copy of that parser would be a second thing to fix the day
`Cargo.toml` changes shape, and the two checks would then disagree about what
this repository contains — which is the same failure as two lists of crate names,
one floor up. The dependency reads oddly (a words check reaching into a verbs
check), and the alternative considered was a third crate holding nothing but the
workspace walk. That was rejected as churn for two callers; **if a third check
needs the same walk, extracting it is that change's work**, and this note is
where whoever does it will find the argument.

**The two lists are handed in, not reached for.** `alo-collected` does not depend
on `alo-saying` at all except as a dev-dependency of its test. The crate checks
whatever lists it is handed; the test is what hands it the real ones. That keeps
the method testable against fixtures and keeps `alo-saying` free of a dependency
on its own check — the same division `alo-by-hand` uses for the crates that
declare verbs.

**A crate on both lists, and a crate named twice on one, are findings.** Neither
is in the acceptance. Both are cheap, and both are ways the counts can add up
while the workspace does not: with a crate on both lists, whether its words reach
a person depends on which list somebody reads.

**The convention is a rule now.** `src/words.rs` with a `pub fn declare_into` was
a habit twenty-three crates happened to share; this check makes it load-bearing,
so it is written into `docs/contracts/translations.md` where an author deciding
will meet it. What nothing mechanical can catch is a crate that declares words
somewhere else on purpose — a file can always be called something else — which is
why the contract answers the author who is deciding and the check catches the
author who did not know.

## Verification

Gates run from this checkout. **The gate of record is Linux**, because that is
what alo OS is; the Windows results are given with it, and a pre-existing Windows
failure is named below rather than left for the supervisor to find.

Linux — `wsl -d Ubuntu -u root`, `/mnt/c/dev/alo-os-claude`,
`CARGO_TARGET_DIR=/root/target-claude`:

| Command | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean, zero warnings |
| `cargo test --workspace` | 3187 tests, 179 suites, 0 failed |

Windows — `C:\dev\alo-os-claude`:

| Command | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean, **after the two repairs below** |
| `cargo test --workspace` | **six pre-existing failures in `alo-recounting`'s lib tests**, unrelated to this change — see below |

Every acceptance test was also run on its own, by name, on Linux.

### Two repairs the gate demanded, which were not this task's

`cargo clippy --all-targets -- -D warnings` was **already failing on Windows on
`main`** before any of this work, in two places, and both are the same fact: a
question that only Unix can answer.

- `crates/alo-keeping/src/failing.rs` — `NotKept::a_link` is reached only from
  `believing::text_of`, which is `#[cfg(unix)]` because `O_NOFOLLOW` is what
  answers it. On Windows the constructor is dead code. It is now `#[cfg(unix)]`
  itself; the *variant* is deliberately not, because a refusal this machine can
  word is part of what the crate says everywhere.
- `crates/alo-recounting/tests/what_this_machine_did.rs` — the `NotKept` import
  is used only by the Unix-only half of the test, and is now `#[cfg(unix)]`. This
  error was hidden behind the first until it was fixed.

Confirmed pre-existing by stashing this work and re-running both. They are
repairs rather than scope: a gate that is red for somebody else's reason still
stops this change from being published.

### One thing that is still red on Windows, and is not this task's to fix

`cargo test -p alo-recounting --lib` fails six tests on Windows, on `main`,
without any of this work applied (verified by `git stash -u` and re-running):

```
recounting::tests::a_record_that_is_not_there_is_refused_rather_than_answered_as_nothing
recounting::tests::nothing_that_is_not_this_machines_record_is_read_as_one
recounting::tests::nowhere_to_put_it_refuses_in_words
recounting::tests::one_question_puts_one_account_in_front_of_the_person
recounting::tests::the_answer_is_read_off_the_disk_every_time_it_is_asked
recounting::tests::the_question_put_to_the_surface_is_the_records_own
```

All six pass on Linux. They read a record through `alo_keeping`'s *believed*
door, whose three questions — is it a link, whose is it, who else may write it —
are Unix questions, and on Windows the refusal that comes back is *this platform
does not do that* rather than the one the test names. **This is a real finding
and it has an owner's decision in it**: either those tests are honestly
`#[cfg(unix)]`, or `alo-keeping` says something about a non-Unix host that
`alo-recounting` can match on. Cutting them to green would be narrowing what is
run, which `docs/autonomy/LOOP.md` names as the gate being weakened to pass it,
so nothing here touches them. Task 8's own report is where this belongs as a
follow-up.

## Files

- `crates/alo-collected/Cargo.toml` (new)
- `crates/alo-collected/src/lib.rs` (new)
- `crates/alo-collected/src/declaring.rs` (new)
- `crates/alo-collected/src/apart.rs` (new)
- `crates/alo-collected/src/finding.rs` (new)
- `crates/alo-collected/src/holding.rs` (new)
- `crates/alo-collected/tests/every_crate_that_declares_words_is_collected.rs` (new)
- `crates/alo-saying/src/collecting.rs` — `DELIBERATELY_APART`
- `crates/alo-saying/src/lib.rs` — its re-export
- `crates/alo-keeping/src/failing.rs` — the first gate repair
- `crates/alo-recounting/tests/what_this_machine_did.rs` — the second
- `docs/contracts/translations.md` — where a crate declares its words
- `Cargo.toml`, `Cargo.lock` — the new member
- `docs/autonomy/v0-01-delivery-plan.md` — task 17 marked done, task 18 written

## Limitations

- **It checks that words are collected, never that they are good.** A crate on
  the list whose `declare_into` returns without declaring anything would pass
  this check; `alo-saying`'s own `the_machine_says_what_the_crates_say_between_them`
  is what counts the strings, and the two together are the pair.
- **A crate that declares words outside `src/words.rs`** is invisible to this,
  and nothing mechanical reaches it. The contract is what answers that author.
- **Nothing here is about a translation file.** Whether a language is complete,
  and what a translator sees, is `alo-saying`'s `Loaded` and `Damage`, unchanged.

## Proposed shared-document updates

For the integration owner; nothing here edits those files.

**CHANGELOG.md** — under the unreleased section:

> A crate that declares words and is collected nowhere now fails the build in the
> change that adds it. alo OS keeps one vocabulary so that a translation can be
> checked against the whole machine, and which crates it holds was a list kept by
> hand — a crate left off it shipped with every sentence in it reaching people as
> a bare key, in English and in every language somebody had translated. That
> happened once and was caught by somebody reading. It is now read out of the
> workspace itself, and the one crate deliberately outside the vocabulary is
> named there with the reason it has to be.

**QUEUE.md / STATE.md** — task 17 of the v0.01 delivery plan is done; the plan
names task 18 after it. The Windows-only `alo-recounting` failures above want an
owner.
