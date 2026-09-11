# The grants a person can see, before there is anywhere to show them

- **Date:** 2026-09-11
- **Workstream:** delivery plan (`docs/autonomy/v0-01-delivery-plan.md`), task 22
- **Contributor:** Claude (autonomous worker, kernel-loop lane)
- **Status:** ready for integration

## What changed, in a sentence a person can read

alo OS now has the list of grants a person would look at — who may reach
what, since when, and for how much longer — derived from the grants the
machine actually keeps, with *revoke* one act away and taking effect
immediately, with an expired grant never shown as live, and with *nothing
granted* said as a sentence rather than shown as an empty list. Nothing draws
it yet; this is the value a settings surface will draw, with its refusals
decided before anybody can see them.

## What changed, by path

- `crates/alo-granted` — new crate, the whole of the task:
  - `src/listing.rs` — `Listing::of(&Grants, now)`, the only door: the rows
    are derived from `alo_capability::Grants::active_at` at one moment, and
    the *nothing granted* state answers with a declared sentence.
  - `src/seen.rs` — `Seen`, one row: handle, agent, reach, when it was made,
    how much is left. No public field, no `From`, no deserialiser, no
    constructor from text, and compile-fail examples that turn adding one
    into a failing build. `Seen::revoke` is `alo_capability::Grants::revoke`
    with the row's own handle, and nothing beside it.
  - `src/revoking.rs` — `Revoked::Now` and `Revoked::AlreadyGone`, each with
    its sentence: the revocation says it has *already stopped*, and the stale
    row says *nothing was changed* and that the list needs re-reading.
  - `src/words.rs` — four strings under a new `granted` area, each with a
    translator's note; the row's `{what}` is `alo-capability`'s own clause,
    carried rather than re-worded.
  - `src/testing.rs`, `tests/the_grants_a_person_can_see.rs` — see
    *Verification* below.
- `crates/alo-saying/Cargo.toml`, `crates/alo-saying/src/collecting.rs` —
  `alo-granted` collected into the machine's one vocabulary (24 crates
  collected, 25 declaring with `alo-agentd` apart), with the one-string-each
  and sum tests extended.
- `crates/alo-collected/src/lib.rs` — the prose count of declaring crates
  updated (twenty-four → twenty-five); the checks themselves are equations
  against the workspace and needed nothing.
- `Cargo.toml` — workspace member added.
- `docs/autonomy/v0-01-evidence.md` — the entry for *Grants: pick a folder,
  see what is granted, revoke it, and it expires*: the still-owed half this
  task closes is closed, and what remains owed (drawing it, in
  `crates/alo-shell`) is named.
- `docs/autonomy/v0-01-delivery-plan.md` — task 22 marked done; task 23
  written (see *What this task found*, below).

## Decisions, and why

**The seam is the value, exactly as tasks 3, 7, 8, 9 and 20 decided theirs.**
No compositor port, no pixels, no panel. Unlike the overlay and the
indicator, this crate does not even take a surface to refuse on — a listing
is a value a surface asks for, not a thing pushed at a screen, so *nowhere to
show it* is not a state this crate can be in.

**Provenance is the type, not the documentation.** The acceptance's *no
constructor from text* is held the way `alo-recounting`'s `Told` and
`alo-indicator`'s `Lamp` hold it: `Listing::of` takes the machine's own
`Grants` — the same value the daemon's `permits` answers from, whether
granted this session or read back through `Grants::remembered` — and `Seen`
cannot be built from parts. The compile-fail examples make a second door a
failing build.

**Revocation is the machine's own, not a second mechanism.** `Seen::revoke`
calls `Grants::revoke` with the handle the row was derived with. Immediacy is
therefore not promised here — it is `alo-capability`'s existing property,
shown on the daemon's own `permits` and on a real verb (`Authorised::read`)
in the integration test. A stale row lands on nothing *by* `alo-capability`'s
own rule that handles are never reused, and the person is told the list was
out of date; the grants after a refused revocation are byte-for-byte the
grants before it.

**An expired grant cannot become a row.** `Seen::of` goes through
`Grant::expires_in`, which answers `None` at and after the end — so *never
shown as live* is not a filter somebody remembered to run. The test pins the
sharp case: the grant still on the stored list (`grants.len() == 1`), the
daemon refusing it, and the listing agreeing with the daemon rather than with
the storage.

**The empty list is a sentence, and it deliberately does not instruct.** The
overlay's `overlay.at-rest.nothing-granted` tells a person what to do,
because the overlay is where doing starts. This list is where checking and
taking away happen, so its sentence reports — *nothing is granted, nothing to
revoke* — and an instruction to go and grant something would read as the
machine asking for reach. The two strings' notes name each other so a
translator can tell them apart.

**Times are values, not strings.** `Seen::granted_at` and `Seen::expires_in`
are exposed as values for whoever displays them, for the reason
`alo-capability`'s `Reach::said` already states: formatting an expiry would
hardcode a calendar as well as a language, and neither decision belongs here.

**No verb was added.** A person looking at their own grants is not an agent
doing something (the task's own constraint, and law 2's list is not where a
person's oversight belongs). Nothing in `crates/alo-shell` was touched.

## Acceptance, criterion by criterion

| Criterion | Where it is shown |
|---|---|
| The list is derived from the machine's own kept grants and nothing else; no constructor from text | `a_kept_list_read_back_off_a_disk_is_the_list_a_person_sees` (and the compile-fail examples on `Seen` and `Listing`) |
| Revoking through it is the revocation `alo-capability` enforces, on the daemon's own `permits` immediately, verb refused afterwards | `revoking_through_the_list_stops_the_daemon_immediately_and_the_verb_is_refused` |
| The refusal beside it: a stale revocation changes nothing and says so | `a_revocation_from_a_stale_list_changes_nothing_and_says_so` |
| An expired grant is never shown as live | `an_expired_grant_is_not_on_the_list_a_person_sees` |
| *Nothing granted* is a sentence rather than an empty list | `nothing_granted_is_a_sentence_a_person_reads` |
| Every string is in the vocabulary `alo-saying` collects | `everything_this_crate_says_is_collected_by_the_machine` |

All six are in `crates/alo-granted/tests/the_grants_a_person_can_see.rs`;
the unit tests beside each file take the same halves apart, including the
translated and the never-declared (`is_a_bug`) paths.

## Verification

Platform: Windows 11 (the checkout's own host; nothing here is
platform-shaped). Commands, from the checkout root:

- `cargo fmt --all` — clean (no diff).
- `cargo clippy --all-targets` with warnings denied — zero warnings.
- `cargo test --workspace` — all suites pass on this host, with the six
  pre-existing Windows-only `alo-recounting` failures already named by the
  task 17 and 18 reports unchanged and deliberately not cut to green.
- `cargo test -p alo-granted` — 26 unit, 6 integration, 3 doctests, 3
  compile-fail doctests.

No physical acceptance is claimed: no pixels are drawn and none are tested,
and *On the machine* does not move.

## What this task found, and the next task

**The person's half of *a change reaches the running daemon* has no owner.**
The daemon's half exists and is tested: `alo-agentd/src/rereading.rs` answers
a knock (`FromAPerson::Granted`, which carries nothing) by re-reading
`/var/lib/alo/grants.toml` whole, so a grant missing from the file is a grant
revoked — that was lane B's *a grant made now reaches the daemon now*. What
does not exist is anything on the **person's side** that composes the acts:
revoke through the list (or grant through the picker), write the list back
with `alo_remembering::kept`, and then knock. Today only test fixtures ever
call `kept` — so a revocation made through this value is immediate in the
`Grants` it was made on and reaches no daemon and no next sign-in until a
surface hand-writes that composition, which is exactly the kind of
order-sensitive glue (a knock sent *before* the write re-reads the old list)
that should be a tested value rather than a habit. It needs no screen, no
decision and no hardware, and it is written into the plan as task 23.

## Remaining limitations

- Nothing draws the list; the drawing is `crates/alo-shell`'s, the desktop
  lane's, and the evidence ledger names it as still owed.
- `Grants::revoke_everything_for` — ADR 0001 §3's *one action for everything
  an agent holds* — exists and is tested in `alo-capability`, and this value
  does not surface it; a surface that wants the one-act button calls it
  directly, and shaping that row was not in this task's acceptance.

## Proposed shared-document updates (for the integration owner)

- `CHANGELOG.md`: "The list of grants a person will see exists as a value:
  who may reach what, since when and for how much longer, derived from the
  grants the machine keeps. Revoking from it takes effect immediately, a
  stale revocation changes nothing and says so, an expired grant is never
  shown as live, and an empty machine says *nothing is granted* in the
  reader's own language."
- `docs/autonomy/QUEUE.md` / `STATE.md`: task 22 done per this report; task
  23 (*a person's change to the grants reaches the file the daemon
  re-reads*) added by the plan.
