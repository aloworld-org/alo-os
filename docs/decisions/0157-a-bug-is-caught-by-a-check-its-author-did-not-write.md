# ADR 0157 — a bug is caught by a check its author did not write

**Status:** **accepted, 2026-11-15, by the owner**, relayed by the coordinator.
The owner was offered research into catching bugs in code an AI writes, with
three recommended methods. The answer was *"Yes add them also make them laws.
Another law will be to write tests first before coding."* The four laws and
their being laws are the owner's decision. The wording, and the account of
which of them anything checks, is the lane's. This adds laws 6 to 9 to
`CLAUDE.md`. Rule 6 of
[ADR 0156](0156-a-feature-is-done-when-its-walk-passes.md) (analyse before
running) stands beside them.
**Date:** 2026-11-15

## What it changes for the person

**Fewer features that pass every test and still fail on their machine.** A
test that agrees with the code's mistake reads as green, and the person is
the first to find the fault. These laws require each change to bring tests
that were seen to fail first, and that somebody other than the code's author
checked. The tests must also go red when the code is broken on purpose. When
the change is to what alo's model is told, alo has to take the right road
every time it is tried, not once.

## Why

On 2026-11-14 and 15 the VSCodium walk failed three times on `main`, at a
different step each time, after every piece of it had passed its own tests.
The owner asked for research into how other teams catch bugs in code an AI
writes. The sources mostly agreed on one point: **an AI that writes both the
code and its tests writes tests that agree with its own mistakes**, and the
bugs are caught by checks that come from somewhere else. The evidence behind
this is mostly practitioner writing, not formal study. The one study cited
for a figure is *Can LLMs Test Their Own Code?* (ICML 2026). It found that
2.8% to 15.3% of AI programs that passed their own unit tests failed a
property-based test.

## The laws

6. **Tests are written first**, run, and seen to fail for the reason the
   code will fix. That is the owner's own addition.
7. **The author of the code is not the only author of its tests.** A second
   lane or helper writes or reviews them from the promise, without the
   implementation.
8. **A test must be able to fail.** The code a change touches is broken on
   purpose with `cargo-mutants`, and a break no test notices is reported with
   the change.
9. **What the model chooses is tested many times, not once.** A change to a
   model's instructions or verbs passes only if every try, on a fixed set of
   requests, takes the expected road. Research on agents finds that one
   successful run overstates reliability by 20–40% (ReliabilityBench, 2026).

## Which of these anything checks

Under `CLAUDE.md`'s *say whether a measurement was performed once or runs
forever*, honestly, as this is written:

- **Law 6 is a discipline.** Git can show whether a test's commit came before
  the code's. Nothing reads that, and a test can be committed first and still
  have been shaped by code its author had already written.
- **Law 7 is a discipline.** Nothing can tell who wrote a test.
- **Law 8 is a check** (#1274, `.github/workflows/mutants.yml`). It runs on
  every pull request and by hand. It breaks the code the pull request changes
  and lists each break no test noticed. Test files are not broken. Neither are
  the packages whose tests the gate leaves out: today `alo-bounding`,
  `alo-agentd`, `alo-boundaryd` and `alo-bounding-kernel`. **It is not a
  required check**; that is the owner's setting.
  - *The run that showed it working:* 43686192150, on #1269, was red. It
    found two breaks in `crates/alo-walking/src/writing_in_vscodium.rs` that
    no test noticed. #1281 added the test, and its own run, 43700667911, was
    green.
- **Law 9 is a check** (#1273, `.github/workflows/every-try.yml`). It runs on
  a pull request that changes what the model is told or offered, and by hand.
  It tries every request in the set five times against the real model, and
  carries nothing out on any machine. It spends the provider key, so it does
  not run on every pull request. **It is not a required check**; that is the
  owner's setting.
  - *The runs that showed it working*, with gpt-4o-mini: 43682925362 and
    43683523750 were red. In both, writing in VSCodium was 0 of 5 and
    publishing was 0 of 5. #1275 and #1278 were measured with it before they
    landed: publishing reached 5 of 5 twice and VSCodium 5 and then 4. It is
    still red today, because installing an application is 0 of 5.
  - *Since 2026-11-17 it holds a change to its own requests* (the owner:
    *"a change that adds or alters something the AI chooses is blocked from
    merging until its OWN practice requests pass at least 9 out of 10 tries.
    Other requests' slips don't block it."*). It runs on every pull request
    and queue branch; which requests are a change's own is read off the
    declared file `crates/alo-every-try/what-the-model-is-offered.txt`, and a
    change that leaves that file alone asks the model nothing. Its own are
    tried ten times at each level; the others three times, reported and not
    counted. Whether it is required is still the owner's setting.
  - **A temporary departure from law 9, decided 2026-11-20** (the owner,
    *"Go ahead"*, on the coordinator's proposal). Law 9 says a change passes
    only if **every** try takes the road. The check holds a change's own
    requests to **9 of 10** for now, because at ten tries a request that
    nearly always takes the road still fails by chance in many runs, and a
    gate that fails good changes by chance is one lanes learn to route
    around. Three parts:
    1. **10 of 10 is the target.**
    2. **A ratchet**: a request that has once taken the road on every try at
       a level is recorded in `crates/alo-every-try/what-each-request-has-reached.txt`,
       raised only from a measured run (`cargo run -p alo-every-try -- reached
       <measured>`), never lowered (the check refuses a change whose record
       drops an entry for a request still in the set), and an own request so
       recorded must take the road on every try there.
    3. **Then the law as written**: once every request is reliably at ten,
       twenty tries per own request with all twenty required. That is one
       line, `THE_BAR` in `crates/alo-every-try/src/the_bar.rs`, from
       `TheBar::of(10, 9)` to `TheBar::of(20, 20)`.
    Since 2026-11-20 every request is a change's own when the change alters
    any verb as the model reads it, because every turn is shown every verb
    offered, and `every-try` is a required check.
  - **Compare with main, for a change that owns the whole set** (the owner,
    2026-11-20: *"Compare with main"*). A change to what every turn is told,
    or to any verb as the model reads it, owns every request, and the whole
    set wobbles from run to run: two whole-set runs at ten tries of one
    model-facing tree (CI runs 44779963562 and 44793910551) disagreed by two
    or more tries on 6 of 50 request-and-level pairs, and "type into a search
    box without saving" went from 9 to 1 with nothing changed. Held to the bar
    absolutely, such a change fails by chance. So it is tried, whole, beside
    its base, the same number of times, and blocks only where a request is
    clearly worse than its base (a drop of `THE_CLEARLY_WORSE_DROP`, two, in
    every ten), falls below the bar where its base met it, or breaks the
    ratchet; a request below the bar on its base may stay there, no worse.
    **Flag, then confirm** (approved the same day): a fault on the first ten
    is a flag, the flagged request is tried `THE_CONFIRMING_TRIES`, ten, more
    on both sides, and the verdict is on the twenty — a drop of four, a floor
    of eighteen where the base had eighteen, twenty where the record holds it.
    Both settings are in `crates/alo-every-try/src/the_bar.rs`;
    `crates/alo-every-try/src/compared_with_its_base.rs` is the rule.
  - **An own request already weak on its base is compared with it too**
    (the coordinator, 2026-11-20, keeping the owner's two decisions
    consistent). A change's own request below the floor on its twenty tries
    is tried on the base as many times; if the base met the floor, the
    change must too, and if the base was below it, the request may stay
    below, no worse by four of twenty. The ratchet still blocks outright.
    Why: #1412 makes the window read say whether a slider can be moved, so
    it owns every request that reads a window, among them the search box,
    weak on main and fixed separately (#1413). Held to the floor absolutely,
    a fix was blocked for a fault it did not cause — the deadlock the owner
    chose "Compare with main" to end.

The figures above are single runs on 2026-11-15. The two workflows are what
measure them again.

## What this does not change

The gate in `CLAUDE.md` and ADR 0156's six rules stand whole. These laws are
about who writes a check and whether it can fail. They do not replace any
check that already runs.
