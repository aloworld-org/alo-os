# EN 301 549, clause by clause

**Date:** 2026-09-17
**Workstream:** v0.5 — access and language
**Task:** [task 4](../v0-5-access-and-language-plan.md) — EN 301 549, clause by
clause
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written on an **Apple M3 with 8 GB unified memory**, macOS 26.5.2;
built, linted and tested in the Lima VM on that Mac — **Ubuntu 24.04 aarch64, 6
CPUs, 3 GB of memory**.
**Egress:** none. **The standard's text was not fetched**, and that matters — see
*What this list is not*.
**Status:** done, and the most important line in it is that nothing here claims
conformance.

## What it is

`crates/alo-conforming`, the fifth in the family after `alo-reconciling`
(promises against evidence), `alo-by-hand` (verbs against their plain way),
`alo-collected` (words against the one vocabulary) and `alo-citing` (citations
against the decisions that exist). Same shape, same reason: **the list is checked
against the repository, never against another list written by hand.**

46 clauses of **EN 301 549 V3.2.1 (2021-03)**, each with its number, what it asks
for in one sentence, and where it stands:

| | Today |
|---|---|
| **Met**, by a named test in this workspace | 12 |
| **Not yet**, because a named task in a named plan | 29 |
| **Not applicable**, because a stated reason | 5 |
| **Read against the standard's own text** by a person | **0** |

## No clause may be met by a sentence

That is the acceptance, and it is held by the type rather than by a rule somebody
has to remember. `Standing::Met` carries a crate, a file and a test's name — there
is no variant that takes prose. `Standing::NotYet` carries a task number and a
plan. `Standing::NotApplicable` carries the reason.

`tests/every_clause_that_is_met_names_a_test_that_exists.rs` then walks this
repository and refuses:

- a clause met by a test **nobody wrote** — including one deleted later by
  somebody who did not know a clause was standing on it;
- a clause waiting on a **plan nobody wrote**;
- a clause waiting on a **task that plan does not have**;
- two clauses under one number, a number that is not one, a requirement that is
  more than one sentence, and a dismissal with no reason.

**What *met* means, exactly:** the test exists and is in the workspace, so the
gate runs it. This test does not run it again — `CLAUDE.md`'s gate runs the whole
suite, and a green gate is what makes every *met* row true. What it catches is a
row standing on a name, which is the failure a conformance file dies of.

## Two corrections I made to my own first pass

**The plan says *the parts of 9 that apply to a native shell's text*. Clause 9 is
the web.** The criteria it means reach a native shell through clause **11.1–11.4**,
where the WCAG success criteria are renumbered for software, and that is where
they are listed. A shell that drew web pages would answer clause 9 as well; this
one does not.

**My first list said 22 clauses were met, and 10 of those were over-claimed.** They
cited tests that show a *setting exists* — that reduced motion can be turned on,
that a key filter validates its delay — as evidence for clauses that ask the shell
to **honour** the setting: pause what is moving, accept a key struck twice, resize
text without losing anything. A setting nothing applies yet is not a clause met;
it is a clause waiting on the shell. They are *not yet* against the desktop plan's
task 6 now, and the honest number is 12.

That correction is the whole value of the exercise, and it is the thing a report
writer would otherwise have inherited as a claim.

## What this list is not

- **Not a conformance claim.** `docs/features.md` puts the report at v1. This is
  what a report will rest on, and the plan's constraint is explicit: no report is
  published and no conformance is claimed.
- **Not the standard's text.** EN 301 549 is ETSI's, CEN's and CENELEC's
  copyright. Every requirement here is one sentence in this repository's own words
  about what the clause asks for, written so a reader can tell which clause is
  meant.
- **Not checked against the published document.** Every clause carries
  `Checked::NotAgainstTheText` and the count is in the test's own output:
  **0 of 46**. The numbers and readings were written from this lane's knowledge of
  the standard with the text not in front of it, and the egress rule is why —
  fetching it was not this task's to do. A clause number that turns out to be
  wrong is therefore **a thing this file expects**, and the shape it fails in is a
  person correcting it rather than a report going out with it inside.

Whoever writes the v1 report reads the standard and moves those noughts. The list
is built so that doing so is a diff, one clause at a time, against a file that
already knows which tests it is standing on.

## For the plans this points at

29 clauses name a task somebody else has not done yet, and the check holds the
task to existing. Most of them are the desktop plan's **task 6** — the shell's own
surfaces — which is now the single largest accessibility dependency in the
repository, with **22** clauses resting on it. The access plan's **task 3**
(keyboard operation of everything) carries **5**, and it is blocked on that same
desktop task.
The documents plan's **task 2** carries one that is easy to miss: converting a
document must carry its accessibility information into the copy, which is a real
requirement on `alo-converting` that nothing tests today.
