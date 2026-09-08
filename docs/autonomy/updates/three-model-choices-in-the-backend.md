# The three model choices, in the backend

- Date: 2026-09-08
- Workstream: model selection and configuration (`alo-choosing`, `alo-models`,
  `alo-agentd`)
- Contributor: Claude Code
- Task: Audit the backend configuration and model-selection contracts against
  the owner's three choices, and implement the missing support
- Status: **the second choice is now something a person can configure, persist
  and have honoured.** It cannot be *completed* on this machine, and the
  blocker is named below rather than guessed at.

## The audit

`docs/features.md` names three: **local models**, **your own API provider**, and
**alo**. Measured against the code rather than against intent:

| Choice | The door that answers it | Could a person configure it? |
|---|---|---|
| **1. Local models** | `Answers::Runtime` (Ollama, ADR 0006) and `Answers::Service` (an OpenAI-compatible service on this machine) | **yes** — `[answers] catalogue` / `brought` |
| **2. Your own API provider** | `Answers::Provider` — implemented, tested, with the indicator, the policy and the record | **no** |
| **3. Alo** | the same door, because ADR 0014 makes alo's service **one more provider** with no special case anywhere | **no** |

**The doors were never the problem.** The provider path is complete and has been
for some time; what was missing was the spine between a person's settings file
and it. In three places:

1. `alo_choosing::Chosen` could name a model from one of **this machine's two
   lists** and nothing else. Its own documentation said so: *a provider and a
   machine somebody paired with are two more places ADR 0008 permits, and
   neither is here.*
2. A settings file naming a provider was **refused whole**.
3. `alo_agentd`'s `Looked`/`WhatAnswers` had no provider arm, so the daemon
   could only ever find a runtime on this machine.

So choices 2 and 3 were unreachable by configuration, and choice 3 needs nothing
of its own beyond choice 2 — which is ADR 0014 working as intended, not a gap.

## What was implemented

**A person can now choose a provider, and the daemon acts on it.**

```toml
format = 2

[answers]
provider = { name = "Mistral", model = "mistral-small-latest" }

[[provider]]
name = "Mistral"
endpoint = "https://api.mistral.ai"
region = "the EU"
```

- **`alo_choosing::Picked`** — what a person chose: a model on this machine, or
  a provider and one of its models. `Chosen` is untouched and still means *a
  model from one of this machine's two lists*.
- **`Settings` carries the person's own provider list** (`alo_models::Providers`,
  which already existed and was serde-ready) and refuses a choice naming a
  provider that is not on it — the rule `brought` was already held to.
- **`alo-agentd` resolves a provider choice out of that list** and puts the
  question through `Answers::Provider`, with the permission taken from the
  provider's own source.

### Model ownership, processing location, and verified confinement

Kept apart, deliberately, because the owner's clarification says a loopback
address establishes where a service is *contacted* and not where it processes.

- **Ownership** is who provides the model. It is the person's choice and alo has
  no opinion about it.
- **Processing location** is `Picked::source`, and it is asked **of the person's
  own provider list** — never of the choice alone. A version that could answer
  without the list would make a local choice and a remote one the same value,
  which is the silent local/remote switch nothing may do. `Chosen::source` can
  stay a `const fn` answering `ThisMachine` because for a model on this machine's
  disk there is no other answer.
- **Verified confinement** is not claimed by any of this. ADR 0021 is PROPOSED
  and unaccepted; nothing here changes `ThisMachineOnly`, the indicator, the
  record or any enforcement.

### The credential path: there is nowhere to paste a key

`alo_models::SecretRef` is *where a key lives in the keyring*, never a key. The
settings file has **no `key` field at all**, and the keyring name is derived as
`provider/<the person's own name for it>`.

That is protection by construction rather than by care: the single most reliable
way for a credential to end up in a text file in somebody's home directory is for
the file to have a field called `key`. A file that invents one is refused,
naming the key.

`needs-a-key = false` is the compatible service that takes no credential —
nothing is looked up and nothing is sent.

## The blocker, named rather than guessed at

**There is no keyring on this machine.** `SecretRef` is a handle and nothing
implements the store behind it; `grep` finds the type, its documentation and
nothing else.

So a provider that needs a credential is **chosen, persisted, resolved — and
refused at the moment of asking**, with a sentence saying the provider needs a
key and this alo OS has nowhere to keep one yet. It is never asked without its
key, and **no other place answers in its stead**.

What that leaves working today: **a provider that needs no credential**
(`needs-a-key = false`) goes end to end through the production path. What it
leaves blocked: every hosted API that authenticates, which is nearly all of them,
including alo's own service whenever it exists.

**Completing choices 2 and 3 needs a credential store.** That is its own piece of
work with real decisions in it — where the store is, what happens on a machine
with no desktop secret service, who may read it — and inventing one inside this
task would have been guessing at all three.

**Alo's own service additionally has no endpoint**, and none was invented. When
there is one it is configured exactly like any other provider, which is ADR 0014
holding.

## Test evidence

Existing coverage was reused where it was sufficient; nothing was duplicated.

**`crates/alo-choosing/tests/the_three_choices.rs` — 6 tests, all passing**

| Test | What it proves |
|---|---|
| `a_provider_the_person_added_is_chosen_and_persists` | **selection and persistence** — provider, model, endpoint and the region *whoever added it stated* survive a real round trip through a real file |
| `a_credential_is_referred_to_and_never_written_down` | **credential handling** — the derived `provider/Mistral` reference, `needs-a-key = false` giving none, and a pasted key refused because the file has no field for one |
| `choosing_a_provider_nobody_added_refuses_the_file` | **refusal** — a choice into a list that does not resolve; nothing in the file is used |
| `a_provider_choice_is_not_a_local_one` | **no fallback** — with brought weights of the *same name* present, the choice is still not local and does not resolve to them |
| `a_provider_is_never_mistaken_for_this_machine` | the source is the provider's own `Hosted { region }`; `ThisMachineOnly` **refuses** it, and permits the same person's local choice — so the refusal is about the place, not the rule refusing everything |
| `settings_written_before_providers_still_read` | **compatibility** — a `format = 1` file reads exactly as it did |

**`crates/alo-agentd` — 2 new tests, in the daemon's own selection path**

- `a_provider_the_person_chose_is_what_answers` — the provider reaches the daemon
  resolved, with its keyring reference and not a key.
- `a_provider_choice_does_not_fall_back_to_this_machine` — it is neither
  `OnThisMachine` nor `Nothing`. `look` used to end in *find a runtime here*, and
  a provider choice reaching that would have been answered by whatever happened to
  be running locally.

**Updated rather than deleted:** `a_provider_is_refused_naming_the_two_lists_this_machine_has`
recorded the old behaviour and now records the new one —
`a_provider_chosen_in_the_older_shape_is_refused_as_the_older_shape`. The half of
the old rule that survives is that a `format = 1` file does not get a provider.

**Whole-workspace gates:** `cargo fmt --check`, `clippy --workspace --all-targets
-D warnings`, `cargo test --workspace`, `RUSTDOCFLAGS="-D warnings" cargo doc`,
the supervisor's own tests, and the BPF target's fmt and clippy on the pinned
nightly. Kernel tests took the shared lock from `a77d10e`.

### One residual, found by a test I wrote

A settings file with a pasted `key = "sk-live-…"` is refused — and the TOML
parse error quotes the offending line, credential included, into the `NotSet`
value. **It never reaches the person**: `choosing.settings.not-understood` is
filled with the path and nothing else, which the test asserts. But a `Debug` of
that refusal would contain it. Nothing renders or logs it today. Closing it means
not carrying the raw parse error, which is a change to how every settings refusal
reports and belongs in its own task.

## Release scope

**Reachable now:** local models through the runtime alo ships and through an
OpenAI-compatible service on this machine; a provider that needs no credential;
the no-agent opt-out, which is a person who has chosen nothing and is unchanged.

**Configured but not completable:** a provider that needs a credential — chosen,
persisted, refused at the moment of asking, pending a keyring.

**Typed and not reachable:** a **paired machine**. `InferenceSource::PairedMachine`
exists and every door refuses it — *nothing anywhere reaches a machine on this
network yet*. Preserved as the owner asks, and **not** made a fourth primary
choice.

**Not claimed:** compatibility is with the OpenAI-compatible shape and with
Ollama through its adapter. **No claim that every model or API is supported.**
No release tier moved and nothing was brought forward.

## What is preserved

- **No silent fallback**, now at three layers: `alo-asking`'s existing tests, the
  configuration layer, and the daemon's selection.
- **Provider and region policy, and typed capability checks** — untouched. The
  region is still *stated* and never inferred from an address.
- **Credentials** — through `SecretRef` only; the file cannot hold one.
- **ADR 0020** — request-scoped destinations and original-hostname TLS
  verification, entire.
- **The indicator and the record** — untouched. A provider question is shown and
  recorded exactly as it always was.
- **No alo account or subscription** is required by anything here. Local models
  and third-party APIs need nothing of alo's.
- **ADR 0021 remains PROPOSED.** Nothing here approves new `ThisMachineOnly`
  semantics, kernel changes, or a weaker guarantee.

## Handoff to the desktop worker

**No compositor or UI file was touched.** What the backend now offers a
settings panel:

- `alo_choosing::Settings::chosen()` answers `Option<&Picked>` — **one of the
  three**, or nobody has chosen. `Picked::provider()` and
  `Picked::on_this_machine()` say which.
- `Settings::providers()` is the person's own list to show and edit;
  `Providers::add` refuses a duplicate name.
- `Settings::provider()` resolves the chosen one.
- Writing a provider requires `format = 2`. A panel that writes `1` with a
  provider in it will be refused, and told which number it needs.
- **The panel must not write a `key`.** There is no field, and the keyring name
  is derived. A panel that collects a key has nowhere to put it until there is a
  store, which is the blocker above.
- The three choices are **model sources**. Please do not render them as privacy
  levels, and do not add a fourth for the paired machine — it is preserved and
  unreachable, and its placement is undecided.

## Proposed integration updates

Not made here — the integration worker owns these four files.

**CHANGELOG.md** — one user-visible line is warranted: *a person can choose an
API provider they added, alongside a model on their own machine.* With the
caveat that a provider needing a credential cannot be asked yet.

**ROADMAP.md** — no tick. Nothing here completes a release item, and the
credential store is not scheduled.

**docs/autonomy/QUEUE.md** — one item worth scheduling: **a credential store**,
which is what completes the second and third choices. It has real decisions in
it and should not be started by guessing.

**docs/autonomy/STATE.md** — three facts. The person's settings can name a
provider since `format = 2`, with `1` still read unchanged, and the daemon
resolves and asks one. The settings file has **no field a credential can go
in**, and a provider that needs one is refused at the moment of asking because
there is no keyring on this machine — that is the named blocker. And ADR 0021 is
untouched by any of it.
