# A default nobody chose, or a promise that says so

**Date:** 2026-09-11
**Workstream:** v0.01 delivery plan, task 16
**Contributor:** Claude (checkout `C:\dev\alo-os-claude`)
**Status:** ready for integration. The decision it publishes is **proposed**, and
the owner is the only person who can accept it.

## What this task was

`docs/features.md` promises at v0.01 that **the agents point at the local model by
default — sovereignty is the default configuration, not an option to find**. ADR
0016 settled that *a default is a choice, made by whoever set it*, and refused
one. `docs/autonomy/v0-01-evidence.md` has carried the promise since the audit
was written as one of six with **no evidence at all**, and as the only one of the
six that *cannot get a line without a decision*: no worker may narrow the promise
to fit the code, and no worker may contradict an accepted ADR to make a test
pass.

So the decision was the task, in the shape ADR 0024 already used: the options set
out fairly, a recommendation, and what each would cost. **No code follows it**,
because until it is accepted a worker writing code would be choosing between the
options rather than building one.

## What changed

### The decision

`docs/decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md`.

Four options, and the fourth is recommended:

- **A — unset means local.** `Settings::untouched` resolves to the catalogue's
  model. It is the plainest reading of the sentence a customer reads, and it
  supersedes ADR 0016 — whose reason was never *who* set the default but that a
  default **is** a choice, and that the unanswered state must stay visible. It
  also ends `agentd.nothing-answers-questions`, gives an agent to whoever has not
  yet declined one (ADR 0009), and makes `Catalogue::agent_for_cpu`'s
  recommendation into a setting by nobody's action.
- **B — the image ships the person's choice.** A settings file baked into the
  image under `$XDG_CONFIG_HOME/alo/`. It is us writing in the one file ADR 0016
  keeps for the person, which we have less claim on than an administrator does;
  and it is impossible as stated, because ADR 0024 ships no accounts (so there is
  no home directory at build time) and ADR 0007 makes the right model a function
  of the hardware (so a file baked once is wrong across the fleet). Named
  because it is the seductive one: it is A with the mechanism moved somewhere
  nobody would look.
- **C — setup pre-selects local.** Keeps ADR 0016 literally, since a person does
  press the button, and is persuasion by geometry: ADR 0009 gave the fourth
  choice *the same weight and no persuasion attached*, and a pre-selected control
  makes *not at all* the option you must undo something to take.
- **D — the default is what a machine arrives able to do.** *(recommended)* The
  image carries the pinned runtime and a model sized for the machine, so *on this
  machine* is complete on first boot with no key, no account, no card and no
  network; setup lists the four configurations with the local one first and
  **pre-selects nothing**; and nothing is written into anybody's settings before
  they choose.

### The ledger

`docs/autonomy/v0-01-evidence.md`'s entry for the promise names the decision and
says what is owed underneath it. **The count of promises with no evidence at all
stays at four.** A proposed decision is not evidence that anything was built —
which is precisely what `alo-reconciling` refuses an ADR for — so this entry
closes when a machine arrives with a model on it, and not before. The audit's own
findings above it are left exactly as they were written, for the reason the two
earlier closures gave.

### The check

`crates/alo-reconciling/tests/every_v0_01_promise_is_reconciled.rs` gains two
tests. A ledger entry that cannot answer *shown by* sometimes answers *waiting
on*, and what it waits on is an ADR — the one kind of pointer this crate
deliberately refuses **as evidence**, and therefore the one kind nothing checked
at all. A promise resting on a document nobody wrote reads exactly like an
answer, which is this crate's own first sentence about why the audit exists.

- `a_promise_that_waits_on_a_decision_names_one_that_is_there` — against the real
  ledger on the real disk: every `docs/decisions/…` it names is a file that is
  there, the entry for the local-model default is still owed rather than shown,
  and it still names this decision. Verified to bite: with the ADR moved aside it
  fails with *the ledger sends a reader to a decision that is not in this
  repository*.
- `a_ledger_naming_a_decision_nobody_wrote_is_a_finding` — the refusal beside it,
  against a fixture, with the live decision checked in the same test so the
  refusal is about the ledger rather than about the checker.

## Decisions I made, and why

**The ADR is proposed, not accepted.** ADR 0024 was accepted by a later worker
under a standing delegation *after taking the measurement it said it owed*. There
is no equivalent measurement here that would settle this one: the recommendation
asks for a line of `docs/features.md` to be reworded, and the definition is the
owner's. Accepting it myself would be a worker moving the scope gate.

**The recommendation rewords the definition rather than reinterpreting it in
silence.** Proposed wording is in the ADR. It is **not a narrowing**: the
rewritten line demands a model on the disk of every machine we ship, which the
current line does not, and which is the half nobody has built. What it gives up
is only the claim that a value exists in a settings file before a person has
touched one — a claim this repository has deliberately made unbuildable for two
ADRs and which would not make a single machine more sovereign.

**Ordering is not weight, and that distinction is the whole of what makes both
sentences keepable.** Option D lists the local configuration first because it is
the one that needs nothing added, while pre-selecting nothing and carrying no
persuasive copy. Pre-selection is Option C and is rejected on ADR 0009's grounds.

**What the reading found and nobody had written down:** the promise is not
blocked on a settings key at all. `image/Containerfile` adds two binaries, two
units, two directories and one description to a pinned base and carries **no
model runtime and no weights**, so on the machine this repository builds there is
nothing local to point at. The expensive half of *local by default* is a model on
a disk. That is now in the ledger entry, in the ADR, and in the plan.

**The measurement is named rather than guessed**, in ADR 0024's shape: whether
the weights are carried on the certified image or fetched at setup — since a
machine that fetches at setup is not local by default when it is offline at
setup. The ADR recommends carrying them and records that the number nobody here
has (the smallest catalogued model clearing the verb-driving bar, against what
the update channel can carry) is the first thing the implementation measures,
with `docs/quirks.md` as where the answer goes.

**Why a test at all, on a task whose acceptance is a document.** Not to tick the
handoff: the check is the thing this task made newly necessary. Before it, the
ledger named no decision at all; after it, a promise's entire answer is a pointer
at an ADR, and nothing anywhere would notice if that ADR were renamed or never
written. It declares no strings, says nothing to a person, and is a repository
check like the rest of the crate it lives in.

## Verification

Run in the Ubuntu WSL environment (`CARGO_TARGET_DIR=/root/target-claude`) from
`C:\dev\alo-os-claude`, and on Windows for the crate under change.

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean, zero warnings |
| `cargo test --workspace` | exit 0; 176 test-result lines, none failed |
| `cargo test -p alo-reconciling --test every_v0_01_promise_is_reconciled -- --exact a_promise_that_waits_on_a_decision_names_one_that_is_there` | 1 passed |
| `cargo test -p alo-reconciling --test every_v0_01_promise_is_reconciled -- --exact a_ledger_naming_a_decision_nobody_wrote_is_a_finding` | 1 passed |
| the same first test with the ADR moved aside | **FAILED**, naming the missing decision |

**Not run, and not claimed:** nothing on real hardware, nothing in a VM, and no
image was built. This change adds a document, a ledger entry and two tests that
read files.

Windows note, unrelated to this change: `cargo clippy --all-targets` fails on
Windows in `crates/alo-keeping/src/failing.rs` with `associated function a_link
is never used`, because its only caller is behind `#[cfg(unix)]`. It is
pre-existing, it is not in any file this task touches, and the Linux gate is
clean. Worth a follow-up by whoever owns that crate; I have not touched it.

## Limitations

- **The promise is not closed and is not claimed to be.** It stays a v0.01
  promise with no evidence at all until a machine arrives with a model on it.
- **The decision is proposed.** Until the owner accepts it — or accepts Option A
  and supersedes ADR 0016 — no code may follow, and the ledger says so.
- The check verifies that a decision named by the ledger **exists**. It does not
  and cannot judge whether the argument in it is any good; that is the reader,
  which is the half of this audit no test replaces.

## Proposed shared-document updates

For the integration owner; I have not edited these files.

- **CHANGELOG.md** — *The one v0.01 promise that needed a decision rather than
  code now has one. `docs/decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md`
  sets out what "the agents point at the local model by default" can mean beside
  ADR 0016's refusal of a default nobody chose, and recommends that the default
  is what a machine arrives able to do: a model already on the disk, four choices
  with the local one first, and nothing pre-selected. It is proposed; the owner
  decides. The evidence ledger names it, and a new check keeps the ledger from
  pointing at a decision nobody wrote.*
- **ROADMAP.md** — no line moves. Nothing was built and nothing is ticked.
- **QUEUE.md / STATE.md** — task 16 of `docs/autonomy/v0-01-delivery-plan.md` is
  marked done in the plan, and task 17 (*every crate that declares words,
  collected — and the one that is not, named*) is written there.
