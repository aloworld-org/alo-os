# The file the two European entries mean

**Date:** 2026-09-11
**Workstream:** v0.01 delivery, lane B — task 15 of
`docs/autonomy/v0-01-lane-b-plan.md`
**Contributor:** Claude Code worker, checkout `C:\dev\alo-os-b`
**Status:** ready for integration

## What this was

ADR 0026 decided, on 2026-09-11, that a catalogue entry may name an artefact its
publisher did not publish — and that naming one costs three statements: whose it
is (`by`), which file exactly (`sha256`), and what a reader needs to know
(`note`). It deliberately named no file for anybody, because choosing one is a
curation act with a name on it rather than a side effect of writing a rule.

Nobody had paid that price. `eurollm-9b-instruct` and `teuken-7b-instruct` — the
two entries the catalogue leads with, in as many words *because nobody else
lists them* — still claimed no quantisation and still stated their publishers'
`bfloat16` releases: 18.30 GB and 14.91 GB, `on_cpu = "slow"`, `min_ram_gb` 24.0
and 21.0. Honest, and not a model anybody runs. This task walks rule 6.

## What changed

### The two entries

`crates/alo-models/data/catalogue.toml`:

| | `eurollm-9b-instruct` | `teuken-7b-instruct` |
|---|---|---|
| `artefact` | `hf.co/bartowski/EuroLLM-9B-Instruct-GGUF:Q4_K_M` | `hf.co/mradermacher/Teuken-7B-instruct-commercial-v0.4-GGUF:Q4_K_M` |
| `quantisation` | `Q4_K_M` | `Q4_K_M` |
| `requantised.by` | `bartowski` | `mradermacher` |
| `requantised.sha256` | `785a3b2883532381704ef74f866f822f179a931801d1ed1cf12e6deeb838806b` | `03fd13daafb6f20c1c5f4b908d163841cdd96803a6f5a7c0c39dd236a2b1630b` |
| `download_bytes` | 18_304_683_360 → **5_582_838_496** | 14_905_484_192 → **5_018_868_512** |
| `min_vram_gb` | 20.5 → **8.0** | 17.0 → **6.5** |
| `min_ram_gb` | 24.0 → **12.0** | 21.0 → **10.0** |
| `on_cpu` | `slow` (unchanged) | `slow` → **`workable`** |
| `drives_verbs` | `not-measured` (unchanged) | `not-measured` (unchanged) |

Every digest and every size is the one the artefact's own repository publishes,
read off the Hugging Face file list on 2026-09-11 — the LFS object id of that
file, which is its `sha256`. `parameters_b` did not move for either entry, so
rule 5's arithmetic is doing real work on the new numbers: 0.61 and 0.67 bytes
per parameter, both four-bit figures for files both entries now name.

### The grounds, which differ for the two, and the Teuken one is a finding

**EuroLLM is a provenance judgement.** Four uploads of a Q4_K_M exist —
`bartowski`, `lmstudio-community`, `QuantFactory` and (Q8 only)
`NikolayKozloff` — and all carry the model's own Apache-2.0, so the licence does
not choose. `bartowski`'s repository states the most complete recipe: the
llama.cpp release it was built with (b4240) and the importance-matrix
calibration dataset, published as a linked gist. The other two full uploads say
"created using llama.cpp" or nothing. That is the ground, and it is written into
the entry because a judgement nobody wrote down reads a year later as a coin
toss.

**Teuken is a licence question, and it nearly went the wrong way.** openGPT-X
publishes the model twice — `-research-v0.4` under `license: other` and
`-commercial-v0.4` under Apache-2.0 — and the requantisers split across the two.
The two names a person reaches for first, `bartowski` (733 downloads) and
`QuantFactory` (544), **both requantised the research release**. This entry's
licence line says Apache-2.0 with commercial use permitted, and `upstream` names
the commercial release (task 9 corrected that). Naming the popular upload would
have put a research-licensed artefact behind a commercial claim: rule 1's harm —
*a licence stated wrongly is worse than a model omitted* — arriving through rule
6's door within a day of that door being opened. So the entry names
`mradermacher`'s commercial set, which states its conversion type and quantize
version, declares itself static (no importance matrix) and carries the model's
own licence.

**No loader check can see that mistake.** A research requantisation has a name, a
digest and a note like any other; `Requantised::what_is_wrong_with_it` would pass
it. So the rule is one level up and it is a test over the catalogue:
*an entry that borrows a file names a requantisation of the release its own
`upstream` names*, compared by that release's own last path segment. It is
deliberately **not** a refusal in `Catalogue::parse`, because a first-party
artefact is spelled the way the pinned runtime's library spells it
(`mistral:7b-instruct-v0.3-q4_K_M`) and carries no release name to compare — a
loader rule over those would be a rule about a runtime's tagging habits rather
than about a licence. The function has its refusal path shown on fixtures, both
directions plus the two shapes it must not catch.

### What else moved

- **`crates/alo-models/data/catalogue.toml`'s rules.** Rule 4's closing sentence
  no longer says the two entries claim no quantisation. Rule 5 says the fallback
  is now the shape an entry takes on its way in rather than one anything stands
  in, and that this is not a reason to soften it. **Rule 6 gained the two
  sentences a curator needs**: read the licence against the uploader's
  repository as well as the publisher's, and put the grounds in the note.
- **`docs/quirks.md`** gained *The two best-known requantisers of Teuken took the
  release nobody may rely on* — the licence split, and the smaller finding
  beside it: three of the four EuroLLM uploads are within 400 bytes of each
  other in size and share no digit of their digests, because there is no "the"
  Q4_K_M of a model, only somebody's. That is the whole reason rule 6 asks for a
  name and a pin.
- **The carry-or-fetch table** carries both new sizes; the paragraph about those
  two rows now says they moved twice in one day and why; and the channel half no
  longer argues from two `bfloat16` entries that are no longer there. It says
  plainly that *can the stream move this* was answered by choosing a smaller
  file, not by measuring the stream — the honest bound is unchanged.
- **`crates/alo-models/src/requantised.rs`** records, in its module doc, the one
  thing the type cannot check and where the catalogue checks it instead.

### The consequence, stated in the plan before it was built

`Catalogue::to_choose_from_on_cpu(16.0)` offers **seven** again rather than six,
and `crates/alo-driving/tests/from_a_prompt_to_what_a_machine_offers.rs` reads
seven. Teuken returns to that list because it names a file that really is five
gigabytes; EuroLLM stays out of it, because nine billion parameters on a
processor is `slow` whatever they are quantised to — the same class as
`gemma-2-9b-instruct`, which is `slow` for the same reason. Both halves are
asserted. An entry that came back to the list for the wrong reason would be the
four-bit-leftover bug of task 13 wearing a digest.

The **measured** count is untouched at four, which is the number a person's
answer actually turns on.

## Decisions taken here

1. **`bartowski` for EuroLLM, `mradermacher` for Teuken**, on the grounds above.
   The task left this open and it is the substance of it.
2. **The release-agreement rule is a test, not a loader refusal.** Reasoning
   above. A loader rule would have to understand two spelling conventions and
   would refuse legitimate entries for their tags.
3. **EuroLLM stays `on_cpu = "slow"`.** It would have been easy to move it with
   the size, and wrong: `OnCpu` is about the processor's experience, and the
   catalogue already grades a 9.2-billion-parameter four-bit entry as slow.
4. **`teuken-7b-instruct` moves back to `workable`.** That word was true of a
   four-bit download and became false only while the entry could name no such
   file. It is true again, of a file the entry now names.
5. **No grade earned, and none attempted.** Task 9 measured that this lane's box
   — a 5,926 MB WSL guest — cannot hold a 7B model at four bits at any useful
   speed. Nothing in the prompt, the scoring or the runtime's wait moved, and
   neither model was downloaded: this task's deliverable is a named file, and a
   grade waits on a machine with room, which is an owner's decision under
   `docs/autonomy/SHARED_MAIN.md`.

## Two existing tests were repointed, and exactly how

Both were written by earlier tasks in this lane and both asserted, as a property
of the current tree, something that was true only of the change that made it.
Neither was weakened.

- **`crates/alo-models/tests/sizes_an_entry_can_point_at.rs`** (task 13). Its
  `the_two_entries_that_name_no_artefact_state_their_publishers_own_release`
  compared both entries against their publishers' safetensors manifests, which
  is no longer what either states. That test is **removed** and its job moved to
  the new file, which compares them against the uploads they now name — the same
  kind of check, against the evidence that is now relevant. Everything else in
  the file stands unchanged: the rule-5 refusal path (eight refusals, including
  the four-bit-leftover case these two entries were), the rules-text test, and
  *correcting a size moved no grade*. Its stale-figure list grew the two
  `bfloat16` sizes, so a table row carrying any of the four now fails.
- **`crates/alo-models/tests/whose_requantisation_this_catalogue_vouches_for.rs`**
  (task 14). Its
  `no_entry_was_completed_and_no_grade_moved_in_the_change_that_decided_this`
  asserted that **no** entry names a third party's artefact, which was the point
  of that change and is not a standing invariant. It is replaced by two tests
  asking what survives any particular choice: every borrowed file in the shipped
  catalogue states all three things, is not attributed to its own publisher, and
  **carries no grade unless the entry is on the list of models this repository
  has actually run `alo-driving` against**; and the two European entries still
  claim no grade. The whole fixture half of that file — every way a borrowed file
  can be named without being vouched for, in front of the loader — is untouched.

## Verification

Platform: Windows 11, `cargo 1.97.1`, from `C:\dev\alo-os-b`. All executed, in
this order, after the change was complete.

```
cargo fmt --all                                        # clean
cargo fmt --all --check                                # clean
cargo clippy --workspace --all-targets -- -D warnings  # clean, zero warnings
cargo test -p alo-models                               # 200 passed, 0 failed
cargo test -p alo-driving                              # 30 passed, 0 failed, 1 ignored
cargo doc -p alo-models --no-deps                      # no warnings
```

The one `ignored` in `alo-driving` is the pre-existing measurement fixture that
needs a runtime holding weights; it is unrelated to this change and was ignored
before it.

**Not run:** the full workspace suite, on the instruction in this task's brief —
the supervisor runs it. Workspace-wide `clippy --all-targets` *was* run, because
it is minutes rather than an hour and it is what catches a crate that forgot to
be registered.

**Not measured, and said so rather than implied:** neither model was downloaded
or run. No grade moved. The digests and sizes are what Hugging Face's own API
reported for those repositories on 2026-09-11; nothing here has verified them by
hashing a downloaded file, which is task 16's subject and is stated as such.

### Evidence, one line per acceptance criterion

| Criterion | Workspace | Crate | Target | Test |
|---|---|---|---|---|
| Each entry names an upload with `by`, `sha256` and `note` | `.` | `alo-models` | `--test the_file_the_two_european_entries_mean` | `each_european_entry_names_the_upload_it_means_with_a_pin_behind_it` |
| Every figure rule 5 binds moves with the artefact | `.` | `alo-models` | `--test the_file_the_two_european_entries_mean` | `the_figures_that_travel_with_the_artefact_are_that_artefacts_own` |
| The licence is read against the uploader's repository too | `.` | `alo-models` | `--test the_file_the_two_european_entries_mean` | `no_entry_that_borrows_a_file_names_a_requantisation_of_another_release` |
| …and that check refuses what it exists to refuse | `.` | `alo-models` | `--test the_file_the_two_european_entries_mean` | `a_requantisation_of_the_wrong_release_is_caught` |
| No grade may be earned here | `.` | `alo-models` | `--test the_file_the_two_european_entries_mean` | `naming_a_file_earned_no_grade` |
| The carry-or-fetch table agrees | `.` | `alo-models` | `--test the_file_the_two_european_entries_mean` | `the_carry_or_fetch_table_carries_both_artefacts` |
| The stated consequence: seven, and EuroLLM not among them | `.` | `alo-models` | `--test the_file_the_two_european_entries_mean` | `the_shorter_list_a_laptop_is_offered_has_teuken_back_in_it_and_not_eurollm` |
| …and the same seven where a person is told it | `.` | `alo-driving` | `--test from_a_prompt_to_what_a_machine_offers` | `the_catalogue_we_ship_now_refuses_for_the_reason_a_measurement_gave_it` |
| A borrowed file is stated in full and brings no grade | `.` | `alo-models` | `--test whose_requantisation_this_catalogue_vouches_for` | `every_borrowed_file_the_catalogue_ships_is_stated_and_brings_no_grade_with_it` |
| Neither European entry gained a grade with its file | `.` | `alo-models` | `--test whose_requantisation_this_catalogue_vouches_for` | `neither_european_entry_gained_a_grade_when_it_gained_a_file` |
| The rule a curator reads is in the catalogue's own rules | `.` | `alo-models` | `--test whose_requantisation_this_catalogue_vouches_for` | `the_rule_is_written_into_the_catalogues_own_rules` |
| Rule 5's refusals still refuse, with the two entries moved on | `.` | `alo-models` | `--test sizes_an_entry_can_point_at` | `a_size_that_belongs_to_no_artefact_the_entry_names_is_refused` |
| No stale size survives in the measurement's table | `.` | `alo-models` | `--test sizes_an_entry_can_point_at` | `the_carry_or_fetch_table_carries_the_two_corrected_sizes` |
| Rule 4's pairing, on the entry it was written from | `.` | `alo-models` | `--test candidates_the_box_can_hold` | `the_rule_a_quantisation_is_paired_with_its_artefact_is_in_the_catalogue_s_own_rules` |

Each was run on its own as well as in the crate suites.

`crates/alo-models/tests/the_carry_or_fetch_measurement.rs` was **not** touched
and is therefore not evidence of anything this change wrote — the handoff format
refuses a test whose file is not part of the change, and it is right to. It was
run on its own anyway
(`the_smallest_entry_that_clears_the_bar_is_read_off_the_catalogue`, ok) because
it is the test that fails when the measurement's table stops agreeing with the
catalogue, and both sizes moved.

## Limitations

- **A pin nothing checks.** Both `sha256` figures are stated and no code in this
  repository compares either with a file. ADR 0026 said verifying it at fetch
  time belonged with the weights work; the weights work is blocked on a grade
  that does not exist, so the pin would sit unread indefinitely. That is written
  up as **task 16** in the lane plan, including the part nobody here has asked
  the runtime: whether the digest Ollama exposes for a model pulled from
  `hf.co/...` is the `sha256` of that GGUF file. If it is not, the finding is the
  deliverable.
- **Neither entry can be graded on this box.** Unchanged by this task, and the
  reason is hardware: 16 GB to the runtime and more than four cores, which is an
  owner's decision about a machine or about `C:\Users\SBW\.wslconfig`.
- **The catalogue still offers no local agent.** Seven entries to choose from,
  four measured, none clearing the bar. Task 10 remains blocked for exactly the
  reason task 8 recorded.
- **A pin ages.** If `bartowski` or `mradermacher` re-uploads a better file, the
  entry becomes stale rather than wrong and a curator has to notice. ADR 0026
  chose that direction deliberately; nothing here improves on it.

## Proposed shared-document updates

I did not edit `CHANGELOG.md`, `ROADMAP.md`, `docs/autonomy/QUEUE.md` or
`docs/autonomy/STATE.md`.

**CHANGELOG.md**, under the current unreleased section:

> The catalogue's two European models — EuroLLM 9B Instruct and Teuken 7B
> Instruct — now name the quantised file each one means. Neither publisher ships
> one, so each entry names somebody else's, states who made it, pins it by its
> digest and says what a reader needs to know about it. Choosing Teuken's turned
> on a licence: its publisher releases the model twice, for research and for
> commercial use, and the two best-known quantisations are of the research
> release, which this catalogue may not put behind a commercial claim. Both
> models are now sizes an ordinary machine could hold, and neither has been
> measured driving the verbs — that still waits on a machine with room.

**QUEUE.md / STATE.md:** lane B task 15 done, task 16 (*The pin an entry states,
and what the machine actually got*) written and ready. No task in
`docs/autonomy/v0-01-delivery-plan.md` matched this one — its task 31 is the
weights-aboard work, which is lane B's task 10 and still blocked — so nothing
was marked there.

**ROADMAP.md:** no exit gate moves. `docs/features.md`'s *a curated catalogue of
open-weight models with their licences stated* is closer to true for two entries
that could previously state neither a file nor a size anybody could use, and the
v0.01 model promises are unchanged.
