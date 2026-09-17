# v0.5 — models a person adapts, and the one they subscribe to

**Workstream:** two `ROADMAP.md` v0.5 lines about the model, from opposite ends —
★ *Guided fine-tune, with the dataset never leaving the machine* (`docs/features.md`:
*LoRA/QLoRA over a granted folder or a tenant's records, as a flow rather than a
toolchain; the dataset, the adapter and the resulting weights never leave the
machine*); and *alo's own hosted model, and a subscription to it* (ADR 0014: *built
as a provider like any other, with a test that proves it: our address is not
privileged in `alo-egress`, and a policy refusing hosted inference refuses ours. The
account and the billing live outside this repository; what the machine knows is an
address, a key in the keyring, a region, and whether the last request was
accepted*). They belong together because both are **the model becoming more the
person's own** — shaped by their documents on their machine, or bought from us
without us being any more trusted than anybody else.
**Why it exists:** written 2026-09-15 so that no v0.5 line is without a plan.

**The owner's rule for this plan, 2026-09-15:** *test on the small model; once a small
model works, larger models are not run.* Every fine-tune in this plan is run on the
smallest model the catalogue already recommends, with a small dataset, and **no larger
model is downloaded, loaded or trained to find out whether it also works.**

**Crates this plan owns, all new:** `crates/alo-adapting` (a guided fine-tune: what is
trained on, where, with what, and what comes out) and `crates/alo-hosted` (alo's own
service as a provider entry, and the tests that prove it gets no exemption). **It reads
and never edits** `alo-models` and `alo-driving` (what a model is, the catalogue, and how
its grade is measured — an adapted model is measured the same way), `alo-picking`
and `alo-capability` (a dataset is a granted folder, nothing wider), `alo-finding` (what
the index already read), `alo-egress` (the hosted service's departures, and the proof a
fine-tune has none), `alo-choosing` (a provider is a person's choice, ADR 0016),
`alo-secrets` (the key, ADR 0022), `alo-answering` and `alo-telling` (a refused or unpaid
request's words), `alo-measuring` (what a running fine-tune uses), `alo-bounding` (the
kernel boundary a turn runs in), and `alo-saying`. **Nothing in `crates/alo-shell`**, and
nothing in `image/`.

**What this plan may not do:** tick anything *on the machine*; write a trainer, an
optimiser or a kernel (`docs/features.md` non-goals: *no model training toolchain from
us* — a pinned upstream fine-tuning stack is rented, configured and never patched); send
a byte of a dataset, an adapter or a weight off the machine; build alo's account,
billing or hosted infrastructure (outside this repository, ADR 0014); give alo's own
service a code path, default, pre-selection or quieter indicator that no other provider
has; or run a model larger than the small one (above). Before writing the next task,
`git pull` and read the plan as published.

## Tasks

### 1. What a fine-tune is trained on, and what it may never reach

**Status:** **Done, 2026-09-17.** **Depends on:** nothing.

- **Acceptance:** `alo-adapting` holds a fine-tune's input as **one granted folder** (or
  several, each granted) read through the grant a person made with `alo-picking` —
  never a path typed, never *all my documents* — with the files it will use listed and
  counted before anything trains, so a person sees what their model will learn from; a
  file the index could not read or a kind it will not train on is named, not skipped
  silently; **nothing in the dataset, the adapter or the weights may leave the machine**,
  held by a test that runs a whole small fine-tune inside a boundary with no network
  and finds no departure attempted, and by a test that reads the shipped source for any
  upload road; and the dataset is never written anywhere but the fine-tune's own working
  folder, which is removed when it ends unless the person kept it.
- **Constraint:** no trainer here. The input is decided; the training is task 2's rented
  stack.

### 2. A fine-tune run, on the small model, with the rented stack

**Status:** ready. **Depends on:** 1.

- **Acceptance:** a LoRA (or QLoRA where the machine's memory needs it) adapter is trained
  on **the smallest model the catalogue recommends**, by a pinned upstream fine-tuning
  stack, over a small dataset from a granted folder, on the CPU where there is no graphics
  card and on the GPU where there is one; how long it took, the memory it used and the
  machine it ran on are recorded in the report with the command that reproduces them; a
  run that the machine cannot finish in a stated time is stopped and **said so plainly
  once** (the catalogue's rule for models too large, applied to training), never left
  running forever; and the run is a use of the machine `alo-measuring` names while it
  runs. **No larger model is tried.**
- **Constraint:** the stack is rented and unmodified. Where it cannot do something this
  task needs, that is a finding, not a patch.

### 3. The adapted model is measured before it is trusted

**Status:** ready. **Depends on:** 2.

- **Acceptance:** an adapter is served by the pinned runtime beside the base model it came
  from, and is **graded by `alo-driving`'s exercises in the words a turn shows** exactly as
  the base was (ADR 0037) — an adapter that drives the verbs worse than its base is
  reported as worse, with both numbers, and is not offered as the agent; it appears in the
  catalogue as *yours, from these files, on this date*, never as one of ours; and removing
  it removes its weights and nothing of the base.
- **Constraint:** nothing is graded on a larger model. The bar is the same bar; an adapter
  earns no allowance for being the person's own.

### 4. The flow: a person, not a toolchain

**Status:** ready. **Depends on:** 1, 2, 3.

*As a flow rather than a toolchain.*

- **Acceptance:** the whole road is a sequence of decisions a person makes in words —
  which granted folder, what they want the model to be better at, start, what it cost,
  keep it or not — held as a type with one sentence per step in the vocabulary; **no step
  names LoRA, QLoRA, a rank, a learning rate, an epoch or a checkpoint** (`docs/features.md`:
  *a person never learns the name of anything we rented*), with advanced values available
  only as a documented file for people who want them; and an agent may **propose** a
  fine-tune through a verb a person approves, never start one, because training on a
  person's documents is the most personal act this machine has.
- **Constraint:** nothing draws. The window is the shell's.

### 5. alo's own service, as a provider like any other

**Status:** ready. **Depends on:** nothing.

- **Acceptance:** `alo-hosted` is **data, not code**: alo's service as a provider entry —
  an address, where it runs as the service says it runs, the chain it discloses (*answered
  by alo, using Mistral, in France*, ADR 0014), and nothing else — read through the same
  provider types every other provider uses, with a key kept in the keyring (ADR 0022);
  tests prove each clause of ADR 0014: **the address is not privileged in `alo-egress`**
  (a departure to it fires the indicator in the same words as any provider's); **a policy
  refusing hosted inference refuses ours** in the same words; **there is no default, no
  pre-selection and no variant** for it — a test reads the shipped source for any
  identifier, constant or branch naming it outside this crate's data and fails on one;
  *payment required* and *quota exceeded* from it are reported through `alo-telling` as
  what they are, once, and never fall back anywhere (ADR 0008); and cancelling leaves a
  machine that works (*cancelling is not an expiry*).
- **Constraint:** the service is not built here and is not claimed to run. Every test runs
  against a local stand-in speaking the same API; the report says so. What the machine
  knows is an address, a key, a region, and whether the last request was accepted —
  nothing about an account.

### 6. Every sentence, and the walk from a folder to a better model

**Status:** ready. **Depends on:** 4, 5.

- **Acceptance:** every sentence these crates can say is in the vocabulary with a
  translator's note; one walk — grant a folder, see what will be learned from, fine-tune
  the small model, see its grade beside the base, keep it; then add alo's service,
  ask a question, run out of credit — produces the exact sequence a person meets, recorded
  as a table and held by one test; no sentence names LoRA, a fine-tuning library, the
  runtime or Mistral's API by its tooling (the disclosed chain names Mistral as the model's
  maker, which is provenance, not plumbing).
- **Constraint:** nothing here re-decides what the sentences describe.
