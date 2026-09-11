# What a person is asked at setup, before there is anywhere to ask it

**Date:** 2026-09-11
**Workstream:** v0.01 lane B — accounts and session entry
(`docs/autonomy/v0-01-lane-b-plan.md`, task 11)
**Contributor:** Claude Code worker, checkout `C:\dev\alo-os-b`
**Status:** ready for integration. Not committed and not pushed; the supervisor
gates and publishes.

## What this is

ADR 0009 gave setup a fourth choice — *no model, no provider, no agent* — with
the same weight as the other three and no persuasion attached. ADR 0025 settled
that the local one is listed first because it is what the machine can already
do, and that **nothing is pre-selected**. Neither had anywhere to happen: there
was no setup flow in this repository at all, which the evidence ledger records
against three separate promises.

This is that flow **as a value**, in the shape `alo-approving` and `alo-overlay`
took their surfaces: decided without drawing one, so that the drawing is the
compositor lane's and the rules are testable now. No pixels, and nothing in
`crates/alo-shell`.

## What changed

### `crates/alo-setting-up` — new

| File | What it is |
|---|---|
| `src/offered.rs` | `Offered` and `THE_FOUR`: the four configurations, the local one first, each with a name and one line saying what it is |
| `src/answering.rs` | `Answer`: the four as they are answered, each carrying what that configuration needs |
| `src/setting_up.rs` | `SettingUp`: setup in front of somebody — what is selected, and the one answer it gets |
| `src/agent.rs` | `TheAgent`: whether this machine's agent surfaces exist at all, afterwards |
| `src/refusing.rs` | `NotSetUp`: every way an answer goes nowhere, and the sentence for each |
| `src/nudging.rs` | The check that nothing setup says leans on anybody |
| `src/words.rs` | Fourteen strings: the question, four names, four lines, five refusals |
| `src/testing.rs` | `cfg(test)` fixtures |
| `tests/what_a_person_is_asked_at_setup.rs` | End to end, against the machine's one vocabulary and a real file on a real disk |
| `tests/nothing_here_writes_anywhere_else.rs` | Read off the manifest and the source: one writer, no wire, no verb |

### `crates/alo-choosing` — additive

`src/setup.rs` (new) holds `Setup`, the bit that says the question was put and
answered. `Settings` carries it and `Settings::of` takes it; `Choosing::setting_up`
is the one door that sets it. The file grew a `[setup]` section, `THE_FORMAT`
went to `3`, and `1` and `2` are still read exactly as they always were. One new
refusal, `NotSet::SetupNeedsANewerShape`, and one new string for it.

### Elsewhere

`crates/alo-saying` collects the new crate's words (26 lists now).
`crates/alo-overlay`'s two test fixtures pass the new argument.
`docs/contracts/person-settings.md` gained `[setup]` and format 3, additively.
The plan marks task 11 done.

## The decisions this task left open, and what was chosen

**The four are `docs/features.md`'s own four.** *On this machine*, *on a machine
on your network*, *with a provider you added*, *with none at all*. ADR 0025 says
"the four configurations" without listing them; the definition lists exactly
these, holds them apart on purpose, and ADR 0014 puts alo's own service inside
the third with no special case. So `Offered` has four variants and no fifth for
alo, and its absence is ADR 0014 being kept rather than an oversight — there is
a test that says so.

**A machine on this network is offered and cannot yet be answered.** This
machine keeps no list of paired machines anywhere; `alo-choosing` says so in as
many words and `alo-asking` refuses it too. Two honest options: list three and
say nothing, or list four and refuse the one that cannot happen. The second was
chosen. Dropping it would be alo OS quietly narrowing what it offers to what it
has built, and the refusal — *this computer has not been paired with another
one* — is a true sentence about the machine rather than a fault in it. It is
also the shape the pairing work will fill in without moving a line of the offer.

**The setup answer is a bit in the person's own settings, not a store of its
own.** The acceptance says *choosing writes through `alo-choosing`'s own shapes
into the person's own file and nowhere else*, so a second file was never
available. What had to be decided was what that bit records. It records **one
thing: that the question was put and answered** — not which of the four, because
*which* is `[answers]`, and a second copy is a second thing that can disagree.
The consequence is that no rule anywhere has to keep the two in step, and the
three states the acceptance asks for fall out of two fields:

| `[setup]` | `[answers]` | The machine |
|---|---|---|
| absent | absent | nobody has been asked |
| absent | present | configured by hand, never taken through setup |
| `answered = true` | absent | ADR 0009's fourth choice — **a finished setup** |
| `answered = true` | present | a source chosen at setup |

**The format number moved to 3, and that cost was taken deliberately.** The
alternative — a new key with no number behind it — means an older alo OS meets
`[setup]`, refuses it under `deny_unknown_fields`, and sends the person hunting
for a typo. That is exactly the failure `THE_FORMAT` exists for, and the
provider key set the precedent one shape earlier. `PROVIDERS_ARRIVED_IN` is now
named rather than spelled `THE_FORMAT`, because the old check — *anything older
than the newest* — would have started refusing format 2 files for having the key
format 2 was introduced for.

**Setup is asked once.** ADR 0025 says *setup asks once*; answering twice is
refused and the first answer stands, so a stale screen cannot overwrite what
somebody chose. Everything setup decides remains changeable through the doors
`alo-choosing` already had, which is ADR 0009's *turning it on later is a
setting, not a reinstall*.

**A provider answer is two writes, and that is on purpose.** A choice naming a
provider is a reference into the person's own list, so the provider is added
first and chosen second. Each write is whole or not at all. A refusal between
them leaves a provider on their list and an unanswered setup — both true of the
machine, and both acted on. One door taking a provider and a choice together
would put the list's rule and setup's in one call, and a person who mistyped an
address would be told their *setup* failed rather than their *provider*. The
case that would have left half an answer on the disk — a provider with an empty
model — is refused **before** the provider is added, and there is a test for it.

**`Answer::chosen` returns a `Result` rather than an `Option`.** A provider
answered with no model would otherwise read as *chose nothing* and write ADR
0009's fourth choice into the settings of somebody who had just picked a
provider. That is the one confusion in this crate that would cost a person their
answer, so it is a type distinction rather than a comment.

**The no-persuasion rule is asked of this crate's strings, not the machine's.**
*The money ran out* is a sentence alo OS has to be able to say (ADR 0009,
`docs/features.md`) and `alo-answering` says it. A check over the whole
vocabulary would either fire on that or be watered down until it fired on
nothing. The rule is about the moment a person is choosing between four
configurations, so it is asked of the strings read at that moment — and of
their **translator notes** as well, because a translator told which of the four
is the sensible one writes that into twenty-four files no test in this
repository can read. That is `alo_saying::rented`'s argument, for a different
rule about the same vocabulary.

## Acceptance, line by line

- **The four choices are one enumerated value, ordered with the local one first
  and nothing selected until a person selects it.** `Offered` and `THE_FOUR`;
  `SettingUp::at` takes a path and has no constructor that takes a selection, so
  *nothing is pre-selected* is a property of the type. Pressing on without
  choosing is `NotSetUp::NothingSelected`, not a default quietly taken.
- **A setup that has not been answered is distinguishable from one answered *not
  at all*, and the second is a finished setup rather than a skipped one.**
  `alo_choosing::Setup`, read back off a real file through the door `alo-agentd`
  reads settings through. The two settings values are `assert_ne!`-different.
- **Choosing writes through `alo-choosing`'s own shapes into the person's own
  file and nowhere else.** Every answer goes out through `Choosing`; the crate
  has no path, no serialiser and no second store, which is read off the manifest
  and the source rather than promised.
- **Declining writes the same way and leaves a machine whose agent surfaces are
  absent rather than greyed out.** `Choosing::setting_up(None)`, and
  `TheAgent::Absent`.
- **Every string a person reads is in the vocabulary `alo-saying` collects, with
  a test that none of them asks anybody to buy anything or nudges toward a
  source.** `alo-saying`'s list is 26 crates; `nudging` is the second half.

## Verification

Run on the Ubuntu WSL guest, from the checkout, **in the checkout's own
`target/`** — the default, which is the directory the supervisor gates in:

| Command | Result |
|---|---|
| `cargo fmt --all` | clean; no file changed |
| `cargo clippy --all-targets --workspace -- -D warnings` | **exit 0**, zero warnings |
| `cargo test --workspace` | **exit 0**, 204 targets, **3550 passed, 0 failed** |

Per-criterion runs are in the handoff's `evidence` block, each executed on its
own with `--exact`, each reporting exactly one test passing.

### Why the first handoff was refused, and what the fix was

**Nothing was wrong with the code.** The supervisor's gates reported *no method
named `setup` found for `&Settings`* and *no method named `setting_up` found for
`Choosing`*, twice — and both methods are present, public, and exercised by
tests that pass. The compile error was `alo-setting-up` being built against a
**stale `alo-choosing` rlib from before `Setup` existed**: cargo judged
`alo-choosing` fresh and did not rebuild it, so the new crate linked against the
old library. The refusal log shows it plainly — `alo-choosing` is absent from
the list of crates that run recompiled.

The cause is mtime resolution across `/mnt/c`: a source file edited on the
Windows host can carry a timestamp that does not appear newer to the guest than
the artefact built from its predecessor. **The first worker did not see it
because they gated in a private `CARGO_TARGET_DIR=/root/target-claude`** — a
fresh target directory has nothing stale in it, so every crate was rebuilt and
the suite was genuinely green. The staleness lived only in the checkout's own
`target/`, which is the one the supervisor uses. A green run in a private target
directory is therefore not evidence about the gate that will actually be run.

So this pass was gated in the default `target/` with no override, and
`alo-choosing` recompiled before `alo-setting-up` linked against it. No source
change was needed to make the gates pass, and none was made for that purpose —
the smallest correct fix was to build where the supervisor builds. Anyone gating
this checkout in the guest should either use the default `target/` or run
`find crates tools -name '*.rs' -exec touch {} +` first; a run that skips both
can be reporting an earlier tree.

## Remaining limitations

- **There are no pixels.** The flow is a value; the screen is the compositor
  lane's, as picking's and the grants list's are. What a shell must not be free
  to decide differently — what the four are, what each says, that none is
  chosen, and what an answer does — is fixed here.
- **A machine on this network cannot be answered** until this machine keeps a
  list of paired ones (ADR 0003). Offered, refused in words, tested.
- **Setup is not yet put to anybody.** Nothing calls `SettingUp` at first
  sign-in; wiring it into session entry is work neither this task nor its
  constraint covers, and it belongs beside the surface that draws it.
- **`TheAgent` is read rather than enforced.** It answers whether the agent's
  surfaces should exist; nothing in `crates/alo-shell` reads it yet, because
  nothing there draws them yet.

## Proposed shared-document updates

Not made here — `docs/autonomy/SHARED_MAIN.md` gives these four to the
integration owner.

**`CHANGELOG.md`**

> - **Setup asks once, and offers four ways to run this machine.** A person is
>   asked where their questions should be answered: on this machine, on a
>   machine on their network, with a provider they add, or not at all. The local
>   one is listed first because it is the one that needs nothing added, and
>   **nothing is chosen for anybody** — pressing on without choosing does not
>   choose. Saying *not at all* is an answer rather than a skip: the machine
>   records that it asked, never asks again, and the agent's surfaces are absent
>   rather than greyed out. Nothing setup says asks anybody to buy anything, and
>   that is a test rather than a habit.
> - **A person's settings can now say that setup was answered.** Settings files
>   written before this keep working unchanged.

**`docs/autonomy/QUEUE.md`** — lane B task 11 done; task 12 (candidates the
measuring box can hold) is next and was already written.

**`docs/autonomy/STATE.md`** — reference this report.

**`docs/autonomy/v0-01-evidence.md`** — the three promises the ledger carries
against *there is no setup flow* now have a flow with no surface. That is a
change in what is owed rather than a promise met, and it is the integration
owner's to record: what exists is the decision, the refusals and the writing;
what does not exist is anything a person can see.

**`ROADMAP.md`** — no change proposed.
