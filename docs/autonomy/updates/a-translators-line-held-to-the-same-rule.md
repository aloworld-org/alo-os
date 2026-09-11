# A translator's line, held to the rule the English is held to

**Date:** 2026-09-11. **Workstream:** v0.01 delivery plan, task 21.
**Contributor:** Claude (agent lane).

## What changed

`docs/features.md` promises at v0.01 that **a person never learns the name of
anything we rented**, and that it is **enforced rather than remembered**.
`crates/alo-saying/src/rented.rs` enforces it for the English: every sentence,
note and key the workspace declares is walked against `EVERYTHING_WE_RENT` in
CI. What it could not reach — and what `docs/autonomy/v0-01-evidence.md`
recorded as the still-owed half — is a translation: a file a person outside
the organisation types, arriving in the image, read when a process starts.
*Das Flatpak konnte nicht installiert werden* would have reached a screen in
somebody's own language, past a rule written to stop exactly that sentence in
this one.

Now the same question is asked of every translated line at the one moment this
repository holds the file: when a machine loads it.

- **`crates/alo-saying/src/translated.rs`** (new) —
  `what_a_translation_would_teach` walks a translation's lines against the
  same `EVERYTHING_WE_RENT` list with the same matcher the English check uses
  (`rented::names`, made crate-visible so the two checks cannot drift apart).
  Each finding is a `Taught`: the file, the key, the language, what was found
  and what alo OS rents it for, quoted back on one line. The refusal is worded
  the way the English refusal is — *what a person reading it would have to
  learn* — never as a banned word, because whoever fixes it is looking for
  another sentence to write.
- **`crates/alo-saying/src/loading.rs`** — the check runs in `Loaded::one`,
  before the vocabulary check, so a line wrong in both ways is refused for the
  reason that matters more. A line that names a rented component is taken out;
  the rest of the file loads. The `without` helper was generalised from taking
  a `Wrongs` to taking the keys to remove, so both checks share it.
- **`crates/alo-saying/src/damage.rs`** — `Damage` gained a third kind,
  `taught_of()`, counted and reported beside files that gave nothing and lines
  left out. It travels separately from `LeftOut` because the reader's job is
  different: a dropped gap is fixed by putting the gap back; this is fixed by
  finding a sentence that does not teach anybody a component's name.
- **`docs/contracts/translations.md`** — the paragraph that said *a
  translation is held to the same rule and nothing here can check it* is no
  longer true and now says what actually happens: checked at load, the line
  left out, the rest of the file shown, the refusal naming the file, the key
  and the language. The line-left-out list gained the new cause.
- **`docs/autonomy/v0-01-evidence.md`** — the entry for *a person never learns
  the name of anything we rented* no longer names the translator's line as
  owed; what remains owed is stated (a surface composing a sentence of its
  own is prevented by construction, not verified mechanically). The
  enforcement entry gained the new file.
- **Repair found by the gates, not part of the task:** the reconciling gate
  was red on clean HEAD because commit `ed3e4bb` accepted ADR 0025 and
  reworded the local-model promise in `docs/features.md` without moving the
  ledger's entry or the test anchored to the old wording. The ledger entry
  now sits under *the local model is what the machine arrives ready to run*,
  updated to say the decision is accepted and what is still owed (the model
  runtime and weights the image does not carry, and the decision's open
  measurement), and
  `crates/alo-reconciling/tests/every_v0_01_promise_is_reconciled.rs` anchors
  to the reworded promise — its assertions unchanged: the entry must be owed,
  not shown, and must still name ADR 0025's file.

User-readable change description: **a translation that names something alo OS
runs on — Ollama, Flatpak, systemd and the rest of the rented list — loses
that line and only that line when the machine loads it. The rest of the
language is shown, and the service log says which file, which string and which
language, in a sentence that explains what a reader would have been made to
learn rather than listing a forbidden word.**

## Decisions

- **The rest of the file is kept, and the rule is deliberately no harsher than
  a dropped gap's.** A translation is somebody's donated work, and
  `docs/contracts/translations.md` already settles what one wrong line costs:
  that line and nothing else. Refusing a whole language over one sentence
  would keep one promise by breaking another — and it would turn a person's
  language off on every machine at once in the release that shipped the file.
  The line is left out, the English shows in its place (marked as English, as
  every untranslated string is), and the refusal is reported in `Damage`.
- **The check runs before the vocabulary check.** A line that both names a
  rented thing and drops a gap is refused as a rented-name line, because that
  is the reason that matters more; the vocabulary check then never sees it.
- **Only the translated text is read.** The English check reads the sentence,
  the note and the key because all three are this repository's to write. In a
  translation the only text the translator wrote is the line; the keys are the
  vocabulary's, already held to the rule where they are declared, and a key
  nothing declares never reaches a screen — the vocabulary check leaves it out.
- **`Taught` has no public constructor.** Like `Asked`, `Told` and `Drawn`
  before it, it is made by the walk and from nothing else, so nothing can
  report a leak that was not found in a real translation.
- **No new user-facing string.** The refusal travels in `Damage` and keeps its
  English, for `failing.rs`'s standing reason: it is read by whoever built the
  image or contributed the file, in a service log, and this is the crate that
  runs when the vocabulary has not loaded. The constraint allowed a new string
  only if the decision needed one; it does not.
- **A machine that loads a file whose every line was rented ends up speaking
  that language with nothing translated** — the same state as a file somebody
  just started — rather than refusing the language. The damage says why.

## Verification

Platform: Windows 11 (this checkout), `cargo` from the workspace root.

- `cargo test -p alo-saying` — 62 unit tests, 4 integration tests, 1 doctest,
  all passing (9 new: 7 in `translated.rs`, 3 new in `loading.rs`, 1 new in
  `damage.rs`, minus the reworded existing ones).
- `cargo fmt --all` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean (see limitations for the
  pre-existing Windows state of other crates).
- `cargo test --workspace --no-fail-fast` — everything passes except
  `alo-recounting`, whose Windows-only failures pre-exist this change: 16
  failing tests across four targets (6 in the lib, 5 in
  `afterwards_ask_what_it_did`, 3 in `what_this_machine_did`, 2 in a fourth),
  verified identical on a clean checkout of HEAD with this change stashed.
  Earlier reports counted six; the accurate current count on this platform is
  16, and this change touches nothing `alo-recounting` depends on.
- The reconciling gate (`cargo test -p alo-reconciling`) failed **on clean
  HEAD** before this change: commit `ed3e4bb` accepted ADR 0025 and reworded
  the promise in `docs/features.md` without moving the ledger entry or the
  test anchored to the old wording. Repaired here, because a gate red for
  somebody else's reason still stops publication (task 17's precedent): the
  ledger entry now sits under the reworded promise with the same
  what-is-owed content, and the test in
  `crates/alo-reconciling/tests/every_v0_01_promise_is_reconciled.rs` anchors
  to the new wording with its assertions unchanged.

Refusal paths tested beside the legitimate ones:

- a translated line naming a rented component is found, with the key, the
  language, the file and the same *would have to learn* argument
  (`translated::tests::a_translated_line_naming_a_rented_component_is_found`);
- every name on the English list is caught in a translation
  (`every_name_the_english_is_held_to_is_caught_in_a_translation`);
- capitalisation does not matter; a name inside a longer word is not a leak;
  one line naming two things is two answers; a clean or empty translation
  teaches nothing (four further tests in `translated.rs`);
- at load, the line is left out and the rest of the language shows, with the
  fallback marked as English and never containing the name
  (`loading::tests::a_line_naming_a_rented_component_is_left_out_and_the_rest_shows`);
- a rented name arriving **only** in the translation — the English beside it
  proven clean in the same test — is still refused
  (`loading::tests::a_rented_name_that_arrives_only_in_the_translation_is_refused`);
- a real translation of real keys from the machine's own vocabulary, written
  to a real disk and loaded whole, loses nothing
  (`loading::tests::a_real_translation_with_nothing_rented_loses_nothing`);
- the third kind of damage is counted and reported beside the other two
  (`damage::tests::a_line_that_taught_a_rented_name_is_reported_beside_the_other_two`).

## Limitations

- The check runs when a translation loads, which is when a process starts. A
  file replaced on a running machine is not re-read until the next start —
  the same window every other property of a translation already has, and on
  an image-based system (`/usr` is the image, ADR 0011) not one a person can
  ordinarily reach.
- The pre-existing Windows-only `alo-recounting` test failures remain (16 as
  of this run; earlier reports counted six), deliberately not cut to green
  here — they are another crate's platform issue and this change touches
  nothing it depends on.
- What is still owed on the promise is written in
  `docs/autonomy/v0-01-evidence.md`: nothing verifies mechanically that no
  surface composes a sentence of its own; today that is held by construction
  in each surface crate.

## Proposed shared-document updates

For the integration owner (this report does not edit shared documents):

- `CHANGELOG.md`: "A translation that names a rented component — Ollama,
  Flatpak and the rest — now loses that line when the machine loads it, with
  the rest of the language kept and the refusal logged naming the file, the
  key and the language. Previously the rule was written down for translators
  and checked nowhere."

## Status

Ready for integration.
