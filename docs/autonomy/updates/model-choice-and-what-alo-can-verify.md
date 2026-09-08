# Model choice, and what alo OS can actually verify

- Date: 2026-09-08
- Workstream: kernel enforcement (`alo-asking`, and proposed ADR 0021)
- Contributor: Claude Code, kernel-enforcement workstream
- Task: Align the privacy proposal with the owner's model-choice direction
- Status: **behavioural coverage added and published; the proposal is revised and
  remains PROPOSED.** No enforcement, label, rule or promise was changed.
- Reads: `docs/features.md` (model-choice clarification, `9707ad5`),
  `docs/autonomy/updates/owner-model-choice-direction.md`

## What changed

**One test file, one revised proposal, one plan entry.** Nothing in the running
system behaves differently.

- `crates/alo-asking/tests/a_day_that_only_looks_like_it_never_left.rs` — **new**,
  two behavioural tests through the production `Answers::Service` door.
- `docs/decisions/0021-what-a-service-on-this-machine-vouches-for.md` — revised.
  **Still PROPOSED. Publication is not acceptance.**
- `docs/autonomy/kernel-enforcement-plan.md` — task 9, and the decision entry.

## The separation the owner asked for

The clarification's sentence — *a loopback address establishes where a service is
contacted, not where it performs inference* — names a category error that runs
through this whole subject. The ADR now separates five things:

| | What alo OS knows |
|---|---|
| **Who provides the model** | exactly — the person chose it |
| **Who manages the runtime** | exactly — it is which of the three doors was used |
| **Where processing occurs** | **almost nothing**, once a socket is involved |
| **What alo can verify or enforce** | the address it connected to; the destination a *turn* may reach; what its own components do |
| **What has been permitted** | exactly — `SourcePolicy`, the organisation's rule, the grant, the typed capability |

**Brand is evidence of neither privacy nor its absence, in either direction.** A
third-party model can be genuinely local: llama.cpp serving weights the person
downloaded never touches a network, and none of that depends on who wrote it. An
alo-provided service can be remote, and `docs/features.md` already says our own
hosted service gets no exemption.

`Served::source()` answers *where processing occurs* with a fact about *what alo
can verify*. That is the whole defect, and it is a category error rather than a
vulnerability.

## The behavioural evidence

**Audited before writing.** `alo-asking` already covers the guarantees well —
non-loopback refused, redirects refused, no silent fallback in either direction,
a local day with no egress in the record, `ThisMachineOnly` permitting the local
door. None of that needed repeating and none of it was.

**What was missing** was the case where those same assertions are *true of the
words and false of the world*. So the new file copies every assertion from the
honest case in `a_day_that_never_left.rs` and makes them against a service at
`127.0.0.1` that forwards.

| Test | Kind | What it shows |
|---|---|---|
| `a_service_that_forwards_is_answered_as_though_it_never_left` | **documents a gap** | the answer says *on this machine*, the indicator is quiet, the record's *what left this machine* is empty — **and the far service reports holding the question text** |
| `this_machine_only_still_permits_a_service_that_cannot_be_verified` | **documents a gap** | the strictest rule this machine has permits a question it cannot follow |

Both **pass**, which is the finding. A transparent relay is not a redirect — it
answers in its own voice, so nothing in the HTTP exchange gives it away, and the
existing redirect guarantee does not touch it.

**Gap tests and guarantee tests are labelled as such**, in the file's own module
documentation and in the ADR's test table, because they read the same and mean
opposite things. The day either of these fails is the day somebody made alo OS
say something truthful here, and the file says to come and record what changed.

**Owned fixtures only.** The relay and the far service are listeners this test
owns on ports the operating system chose; the far one binds this machine's own
interface where there is one and loopback otherwise. **No credential, no external
provider, nothing resolved by name, nothing left the machine.**

## Model choice, and the guarantee that is not the same thing

Every option in the ADR leaves **ordinary use of an owner-configured local
service and of a third-party API exactly as it is.** What the revision adds is
the distinction between two different transactions:

- **Ordinary selection** — free and unrestricted. What alo owes is a truthful
  account of what it knows, which today it does not give.
- **An enforceable local-only restriction** — a guarantee, and a guarantee needs
  a mechanism.

Today they are the same setting, decided by an address.

### How a runtime the person owns could qualify — without alo owning it

**Qualification is supervision, not ownership.** A runtime qualifies when alo OS
can observe or constrain its egress — it runs under a unit whose control group
carries a zero-egress rule. Nothing in that mentions who wrote it: llama.cpp,
vLLM or LM Studio would qualify identically if the person asks alo to run them
that way, and **the runtime alo ships would stop qualifying if it were run
outside that supervision**.

The honest form of the guarantee is *this question cannot leave, because the
thing answering it cannot reach the network* — never *because of who wrote it*.

**It is blocked**: it needs the cgroup egress mechanism, a second programme type
ADR 0018's loader does not have, a way to place a person's runtime under
supervision, and somewhere else for model downloads to happen. **Until it exists
no configuration qualifies, alo's own included** — and offering it before then is
exactly what the owner's clarification forbids.

## The unresolved decision, not taken

> **When *This machine only* is selected and the configured service's processing
> location cannot be verified, what should happen?**

Four options, with consequences, in the ADR: **D1** permit and label, **D2**
refuse unless verifiable, **D3** permit with a one-time acknowledgement, **D4**
two settings — *prefer this machine* and *only verifiably this machine*.

Recommended: **D1 now, with the setting's own wording corrected to match what it
does, and D4 when supervision exists.** D2 today would break every current user
of an unsupervised local service — which is all of them — with no upside, since
nothing is verifiable yet. D3 teaches people to click through a dialog.

**This is a recommendation and not an implementation.** Today's behaviour is
unchanged and is now asserted in a test, so that whoever changes it has to come
and say what they changed it to. As the owner's direction says: **a label is a
disclosure, not evidence that a service cannot forward a question.**

## Release scope, stated so nothing reads as a claim

**Implemented and reachable now:** the runtime alo OS ships (Ollama, ADR 0006,
through `Answers::Runtime`); an OpenAI-compatible service the person runs on this
machine (`Answers::Service`, loopback enforced); an OpenAI-compatible hosted
provider (`Answers::Provider`, with indicator, policy and record); and **no model
at all**, which stays supported.

**Typed but not reachable:** a **paired machine**. `InferenceSource::PairedMachine`
exists and every door refuses it — `Miswired::NoPathToAPairedMachine`, *nothing
anywhere reaches a machine on this network yet*. The configuration is preserved
as the owner asks and **it is not an integration that works today**.

**Not claimed:** compatibility is with the OpenAI-compatible shape and with
Ollama through its adapter. **No claim that every model or every API is
supported.** Nothing here brings a later-release feature forward.

## What is preserved

Checked rather than asserted — the whole suite passes:

- **No silent fallback** between local, paired-machine and API configurations —
  `a_local_model_that_fails_never_becomes_an_api_call`,
  `a_question_bound_for_a_provider_never_reaches_the_model_on_this_machine`.
- **Provider and region policy, and typed capability checks** — untouched.
- **Keyring credential handling** — untouched; the new tests use no credential.
- **ADR 0020** — request-scoped destinations and original-hostname TLS
  verification, entire.
- **The indicator and the record** — untouched, and the new tests assert their
  current behaviour rather than changing it.
- **Paired-machine operation and the no-agent choice** — preserved as
  configurations, with the paired-machine one honestly described as unreachable.
- **No account, subscription or alo-owned model is required** by anything here.

## Verification

Ubuntu on WSL2, kernel 6.18.33.2, own `CARGO_TARGET_DIR`, readiness checked
first, **kernel tests taking the shared lock from `a77d10e`**: `cargo fmt --all
--check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test
--workspace`; the supervisor's own tests; `RUSTDOCFLAGS="-D warnings" cargo doc
--workspace --no-deps`; and the BPF target's `fmt --check` and `clippy --release
--target bpfel-unknown-none -Z build-std=core` on the pinned
`nightly-2026-06-01`. Published through `alo-kernel-loop publish`, which runs all
of it and each named acceptance test on its own before staging anything.

**No gate exemption was taken. No test was written to certify prose.**

**WSL is development evidence and never certified-hardware acceptance.** No
*On the machine* box is affected. **The privacy gap is not closed** — it is open
under every option in the ADR, and this work makes it visible rather than
smaller.

## Proposed shared-document updates

Not made here — the desktop integration worker owns these four files.

**CHANGELOG.md** — nothing user-visible. A test and a proposal.

**ROADMAP.md** — no tick, no tier moved, nothing brought forward.

**docs/autonomy/QUEUE.md** — no new item. ADR 0021's three questions belong to
whoever schedules decisions.

**docs/autonomy/STATE.md** — three facts. ADR 0021 is revised to separate model
ownership, runtime management, processing location, what alo can verify and what
has been permitted, and **remains PROPOSED with the `ThisMachineOnly` question
presented as four options rather than answered**. A behavioural test now
documents, through the production service door, that a forwarding service is
answered as though it never left and that `ThisMachineOnly` permits it. And a
future local-only guarantee would be qualified by **supervision rather than
ownership**, which is blocked on a mechanism that does not exist.
