# One component gains CAP_CHOWN, and it is a new one

**Date:** 2026-10-17
**Lane:** Mac
**Branch:** `task/mac/one-component-gains-cap-chown`
**Decision:** ADR 0119

## What this is

ADR 0118 accepted more than one person on a machine and left one thing as *the
next record*, with instructions: *one component must gain it, narrowly, with a
test holding it to that. Which one is the next record, and it should be written
against a measurement rather than a preference.*

This is that record. It is a decision and no code.

## What it decides

A single new helper, `alo-giving-a-folder`, gains `CAP_CHOWN` and nothing else,
makes a person's directory under `/var/lib/alo/undo` and gives it to them, and
is held to that by a test over its own source. **Neither `alo-compositor` nor
`alo-sessiond` gains any capability.**

## The measurements it is written against

**`CAP_CHOWN` is necessary.** With every capability except that one, root cannot
give a directory away — *Operation not permitted*, owner unchanged at `0`.

**And it is sufficient for the whole job**, which is the measurement that makes
*narrowly* concrete rather than aspirational. With only `cap_chown` in the
bounding set: `mkdir` in a `0711` root folder, `chmod 0700`, and `chown` to
another account all succeed. Owner `65534`, mode `700`. The first two need no
capability because the helper owns what it has just made.

**So the grant is one capability and not a set.** There is no second capability
left to argue about, which is a fact rather than a judgement.

**And the candidates differ by two orders of magnitude:**

| candidate | lines of Rust | direct dependencies |
|---|---|---|
| `alo-compositor` (from `alo-shell`) | 113,587 | 51 |
| `alo-sessiond` | 2,160 | 5 |
| a new helper | ~100 | 1–2 |

The *~100* is estimated from `alo-enter-the-box`, written the day before for the
same shape of job, which is **62 lines**.

## The argument in one paragraph

The tempting candidate is the worst one. The account-creating write already
happens inside `alo-compositor`, so putting the folder beside it saves a
process — and `alo-compositor` is the screen a person types a password into, it
parses input devices and fonts, and it is fifty-two times the size of the next
candidate. `alo-sessiond` is better and still wrong, and **its own test already
answered this ADR before it was written**: the failure message in
`this_crate_cannot_do_what_its_unit_forbids.rs` says *adding the capability is a
decision and not a fix*. This is that decision, and it says no.

## What I checked before committing it

Every citation, because an ADR reference is the one kind of pointer a reader
believes without opening — `docs/decisions/README.md`'s own words.

- **Four ADR paths and four file paths**, each confirmed to exist.
- **Two quoted phrases**, confirmed against the real text with whitespace
  normalised, because this repository's documents are hard-wrapped and a plain
  `grep` answers zero for a phrase that is present.
- **`alo-citing` reads this file**, mutation-proved: a four-digit number put
  where a decision's number goes, with no decision behind it, makes the check
  fail and name the line. So the citations are held by a gate and not only by my
  own reading.

  *This sentence spelled that number out at first, and the gate rejected the
  report for citing a decision that does not exist — correctly, because the check
  cannot tell a citation from a sentence about one, and an exemption for prose
  would be a hole in it. Describing the mutation instead of writing it is the
  cheaper side of that trade.*

**And I got the filename wrong on the way in.** I guessed ADR 0118's name from
its commit subject and cited a file that does not exist; the real name is
`0118-more-than-one-person-on-a-machine.md`. Caught before quoting it, and it is
the second time today that guessing an ADR's filename from its title has been
wrong.

## What is still owed

**The helper itself**, which is now unblocked: the decision names its capability,
its two arguments, its refusal, and the test that holds it narrow.

**Two callers, not one widened.** ADR 0118's term is that adding a person is a
different road with different authority, and that `makes_an_account`'s refusal
once an account exists stays — it is what stops a surface making a second
*first* account. So the helper is started from the first-account road and from
ADR 0118's later route, and it is idempotent so that two callers are safe.

**`alo-letting-go`'s four units** are untouched by this and still unshipped.
They need `CAP_SYS_ADMIN` to remove a read-only snapshot, which is a different
grant to a different unit, and the ordering constraint stands: that sweep must
not ship later than the bracket-taking.
