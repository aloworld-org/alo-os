# A provider that will not say where it runs is unknown, and unknown never satisfies a policy

**Date:** 2026-09-12
**Workstream:** v0.5, lane B — providers and models, task 3 (*A provider that
will not say where it runs is unknown, and unknown never satisfies a policy*)
**Contributor:** Claude (`C:\dev\alo-os-b`, kernel-loop worker)
**Status:** ready for integration

## What was wrong

`docs/features.md`, v0.5: *a provider that will not say where it runs is
reported as **unknown**, never assumed to be nearby — and unknown never
satisfies a policy naming a region.*

Most of it existed. `alo_models::Region::Unknown` has been a value of its own
since ADR 0021, `Provider` reads an absent `region` as `Unknown` and infers
nothing from the address, `SourcePolicy::InRegion` refused it, and
`alo-egress` already words such a source as *has not said where it runs*.
What was missing:

- **The refusal was not its own.** A rule naming a region refused a silent
  provider as `NotAllowed::OutsideTheRegion`, whose sentence is *{source}
  does not meet that* — and whose name is a claim about where the provider
  runs. The record kept whichever value it was handed, so a provider that had
  said nothing was written down as running outside the region, which is the
  one fact nobody has.
- **No test held either read path to it.** Nothing in `alo-agentd` checked
  what the bound an administrator wrote does with a silent provider, nothing
  checked that the daemon's sentence says *has not said* rather than
  *elsewhere*, and nothing walked a provider from the person's own settings
  file to the refusal.

## What changed

- `crates/alo-models/src/refusing.rs` — `NotAllowed::RegionUnstated { region,
  provider, source }`, a fourth variant. Its sentence names the provider and
  the region and says the provider has not said; it carries no `{source}`
  clause, because that clause would only repeat the sentence. The module doc
  now says why the region rule has two sentences and no third.
- `crates/alo-models/src/source.rs` — `SourcePolicy::refusal` answers
  `RegionUnstated` for a hosted source whose region is `Unknown`, and
  `OutsideTheRegion` for everything else the region rule refuses. The match is
  exhaustive over `InferenceSource`, so a new source shape is a compile error
  rather than a guess.
- `crates/alo-models/src/words.rs` — one string,
  `models.policy.region-unstated`, with its translator's note saying the
  sentence must say *not said* and never *elsewhere*. `EVERY_WORD` grows to
  42 and the refusal list in its tests gains the word.
- `crates/alo-models/tests/what_this_crate_says.rs` — the sentence in German,
  translated whole with the provider's name and the region carried through;
  a translation that drops `{provider}` is refused by the vocabulary check;
  the existing region-refusal test now uses a provider that *said* it runs in
  Singapore, since the silent one has a sentence of its own.
- `crates/alo-choosing/tests/a_provider_that_will_not_say_where_it_runs.rs` —
  new. From a real settings file: a provider added with no region is read
  back as `Unknown`, the file contains neither the word `region` nor
  `unknown`, a bound naming a region refuses it with the machine's whole
  vocabulary saying the provider did not say, a machine with no bound puts
  the question to it, and the clause shown beside an answer names no place
  even though the address ends in `.eu`.
- `crates/alo-agentd/src/describing.rs` — the one place the daemon reads the
  organisation's bound: `[questions] may-go = "in-a-region"` refuses the
  silent provider as `RegionUnstated`, permits a provider that said it runs
  there, and an ordinary machine permits the silent one.
- `crates/alo-agentd/src/doing.rs` — four tests on the daemon's own refusal
  path: an organisation's region rule refuses the silent provider and the
  sentence says *has not said* under *an administrator set that rule*; a
  person's own region rule refuses the same way in the rule's own words; a
  machine with no bound never speaks that sentence; and the indicator's
  wording for such a source names the provider and no place.
- `docs/contracts/machine-description.md`, `docs/contracts/person-settings.md`
  — one sentence each on the `region` field, additive.
- `docs/autonomy/v0-5-lane-b-plan.md` — task 3 marked done. Task 4 was
  already written, so no next task is added.

### User-readable change description

On a machine whose organisation allows inference only inside a named region,
a provider that has not said where it runs is now refused with a sentence
that says exactly that: *this machine is set to use inference in the EU only,
and Somewhere has not said where it runs — unknown does not count as there*.
Before, the same refusal said the provider *does not meet* the rule, in a
value named as if the provider ran outside the region. A personal machine with
no such rule is unchanged, and no region is ever guessed from an address.

## Decisions

- **A new variant rather than a flag on `OutsideTheRegion`.** The two are
  different facts and the record keeps whichever value it is handed. A flag
  would let a reader of the record match the variant and report a silent
  provider as outside the region without ever looking at the flag.
- **The unstated sentence names the provider, not the source clause.** The
  source clause for a silent provider already reads *by Somewhere, which has
  not said where it runs*; inside this sentence it would say the same thing
  twice. A provider's name is data and is never translated, so the sentence
  is as translated as its own words are, and `Said::is_translated` holds.
- **`ThisMachine`, `AServiceAtThisMachinesAddress` and `PairedMachine` under a
  region rule stay `OutsideTheRegion`.** That is existing behaviour, and
  whether a machine in the building satisfies a region rule is a question for
  the organisation's policy, not this task.
- **The `alo-choosing` journey test is a dev-dependency-only road.**
  `alo-choosing` already has `alo-saying` as a dev-dependency, so the test
  reads the refusal in the vocabulary the whole machine loads without adding
  anything to the manifest.

## Acceptance criteria and evidence

| Criterion | Test |
|---|---|
| A provider saved with no region is `Unknown`, never a default region | `alo-choosing` `a_provider_that_will_not_say_where_it_runs::a_provider_saved_with_no_region_is_unknown_and_never_a_default_region`; `alo-models` lib `source::tests::unknown_is_a_value_of_its_own_and_not_a_region` |
| The bound naming a region refuses `Unknown` with a sentence saying the provider did not say, not that it runs elsewhere | `alo-models` lib `refusing::tests::a_provider_that_has_not_said_where_it_runs_is_refused_as_unknown_not_as_elsewhere`; `alo-models` lib `source::tests::a_provider_that_has_not_said_is_refused_as_unknown_and_never_as_elsewhere`; `alo-choosing` `a_provider_that_will_not_say_where_it_runs::a_bound_naming_a_region_refuses_it_as_unknown_and_says_the_provider_did_not_say` |
| A personal machine with no bound is not affected | `alo-models` lib `source::tests::a_machine_with_no_bound_is_not_affected_by_a_provider_that_has_not_said`; `alo-choosing` `a_provider_that_will_not_say_where_it_runs::a_machine_with_no_bound_puts_the_question_to_the_provider_the_person_chose`; `alo-agentd` lib `doing::tests::a_machine_with_no_bound_does_not_refuse_the_silent_provider` |
| The indicator's wording carries *unknown*, not a guessed place | `alo-agentd` lib `doing::tests::the_indicator_names_no_place_for_a_provider_that_has_not_said_where_it_runs`; `alo-choosing` `a_provider_that_will_not_say_where_it_runs::what_the_person_is_shown_for_such_a_provider_says_unknown_and_names_no_place` |
| The refusal is tested beside the acceptance in `alo-models` and where `alo-agentd` reads the bound | `alo-agentd` lib `describing::tests::a_bound_by_region_read_here_refuses_a_provider_that_has_not_said_as_unknown`; `alo-agentd` lib `doing::tests::a_rule_naming_a_region_refuses_a_provider_that_has_not_said_where_it_runs_as_unknown`; `alo-agentd` lib `doing::tests::a_persons_own_region_rule_refuses_the_silent_provider_as_unknown_too` |
| Unknown is not a region spelled "unknown" | `alo-models` lib `refusing::tests::unknown_is_not_a_region_that_happens_to_be_called_unknown` |
| The sentence is in the vocabulary `alo-saying` collects and translates whole | `alo-models` `what_this_crate_says::a_provider_that_has_not_said_where_it_runs_is_refused_as_unknown_in_the_language_they_read`; `alo-models` `what_this_crate_says::a_translation_that_drops_the_provider_that_never_said_is_refused`; `alo-models` lib `refusing::tests::the_unstated_refusal_is_translated_whole_and_carries_the_names` |

**Constraint held:** nothing infers a region from an address, a TLD or a name.
The `alo-choosing` test's provider lives at `api.example.eu` and is shown with
no place. A provider that does say is taken at its word (`OutsideTheRegion`
for Singapore, permitted for the EU), which is ADR 0021's existing rule.

## Verification

Run in WSL Ubuntu against `/mnt/c/dev/alo-os-b`, target directory
`$HOME/alo-builds/alo-os-b-72aa4fda7f7d8fc1`:

```
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test -p alo-models
cargo test -p alo-choosing
cargo test -p alo-agentd
```

Results are recorded in the section below, filled in from the run before the
handoff was written. The full workspace suite was not run here; the supervisor
runs it before publication.

### Results

See the end of this report.

## Limitations

- Nothing physical: this task is decidable without hardware and touches no
  device.
- `alo-agentd`'s `doing` tests run the daemon's refusal path against a local
  fixture; none of them opens a socket to a real provider, by design.

## Proposed shared-document updates

- `CHANGELOG.md`, unreleased: the user-readable change description above.
- `docs/autonomy/QUEUE.md` / `STATE.md`: lane B task 3 done; task 4 (*Run a
  model we never catalogued*) is next and depends on nothing.
