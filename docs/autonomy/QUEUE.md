# Queue — v0.01

Worked in order by `LOOP.md`. Every item names what it implements, so an
iteration can read the reasoning rather than guess at it.

## The honest constraint

**This loop cannot finish v0.01, and no amount of iterations will change that.**
Roughly half of what v0.01 needs ends on hardware and an operating system this
loop does not have:

- **A Wayland compositor, the portals and the bootable image are Linux.** They
  will not compile, let alone run, on the machine the loop runs on today.
- **The exit gate is the certified machine**, verified by somebody sitting in
  front of it, once. No test suite substitutes for that.
- **"Agents point at the local model" belongs to `alo-workplace`.** Different
  repository, and this loop never touches another one.

What the loop *can* finish is the part everything else stands on: the capability
model as working code. Grants, verbs, approvals and the record are pure logic —
portable, exhaustively testable, and the thing that makes `alo-agentd` a service
worth writing rather than a daemon with a permission dialogue. Doing that first
is also the right order: the Linux work becomes an implementation of a settled
model instead of a place where the model gets decided by accident.

Items are **ready** (buildable and testable here), **linux** (needs a Linux
host), or **hardware** (needs the certified machine). The loop takes ready items
and stops when there are none left.

---

## Already built, outside the loop

`crates/alo-models` — read it before starting item 1, because it sets the house
style the rest should match, and two of its decisions constrain later items.
`crates/alo-capability`, `crates/alo-record` and `crates/alo-egress` were built
by the loop and are described in the items below. They depend on each other in
one direction only: `alo-capability` decides and reaches nothing, `alo-egress`
decides about what leaves, and `alo-record` observes both and is reachable from
neither.

| | |
|---|---|
| `catalogue.rs` | What alo OS offers, every licence stated, commercial use answered outright; the CPU costs and defaults from ADR 0007 |
| `runtime.rs` | `ModelRuntime` — what alo OS asks of a runtime, in our words |
| `ollama.rs` | The adapter, and the only file that knows Ollama exists (ADR 0006) |
| `source.rs` | Where a question is answered and what that costs in egress (ADR 0008); the region policy an organisation names |
| `provider.rs` | Providers somebody adds themselves; the key lives in the keyring, never in the settings |

**44 tests, clippy clean against the workspace deny list.** Two patterns later
items must follow:

- **A promise in `docs/` is a test, not a sentence.** "Every model states its
  licence" and "a paired machine is egress too" are tests. Anything an item
  claims should be one.
- **Errors say what to do, not what went wrong.** `provider.rs` is the reference:
  *"give the provider a name — it is what you will see when it answers"*.

---

## Ready

- [x] **1. Grants** — implements ADR 0001 §3 and `docs/contracts/agent-verbs.md`.
  `crates/alo-capability`: `reach.rs` (what a grant covers, what a verb asks
  for), `grant.rs` (one grant and what has to be true of it), `grants.rs` (the
  list, its queries and its refusals), `path.rs` (containment, decided
  lexically). 28 tests, clippy clean. **This is the vocabulary items 2–5 speak**
  — `Grantee`, `Reach`, `Ask`, `Grant`, `Grants`, `GrantId` — and they belong in
  this crate beside it rather than in one of their own.

  Three decisions the next items inherit. **Nothing reads the clock:** every
  question that depends on time takes `now`, so expiry is testable without
  sleeping and the daemon and the settings panel cannot disagree about the
  moment. **Identities are matched exactly** — agent names, application ids and
  paths — because matching loosely matches more than the person picked.
  **Containment is lexical and touches no disk**, so whatever executes a verb
  must resolve symbolic links *before* asking whether a path is granted; the
  reasoning is in `path.rs` and item 6 is where it gets honoured.

  Storage is serde, as with `Providers` — where the list is written and when is
  the daemon's, and it does not exist yet.

- [x] **2. The verb registry** — implements `docs/contracts/agent-verbs.md` and
  law 2. Five files in `crates/alo-capability`: `arg.rs` (what an argument can
  be, and what survives arriving), `sentence.rs` (the approval sentence and
  where its words come from), `verb.rs` (everything that has to be true of a
  verb), `verbs.rs` (the closed list), `call.rs` (a validated call, what it
  would touch, and whether the grants permit it). 37 tests, clippy clean.

  **The registry cannot hold a verb that breaks the contract, and it manages
  that by not being able to receive one:** `Verb::checked` is the only
  constructor and there is no `Deserialize` around it. Law 2 is carried by
  `Takes` being a closed list with no free-text kind in it — a model cannot
  compose something to run because there is no shape for it to arrive in — plus
  a declaration-time refusal of names that announce an interpreter, which is a
  tripwire and is documented as one. A verb's implementation passing an argument
  to a shell is still only stopped by review, and `verb.rs` says so.

  Three items inherit decisions from this. **Every argument is required** — an
  optional one makes the sentence conditional. **The sentence names every
  argument**, or the verb is refused. **Being permitted and being approved are
  separate**: `call.rs` answers the first and deliberately does not answer the
  second, which is item 3's.

- [x] **3. Approvals** — implements ADR 0001 §5. Four files in
  `crates/alo-capability`: `proposal.rs` (a change put to a person, and what is
  never put to one), `approvals.rs` (the list waiting to be answered, and the
  answering), `approval.rs` (one answer, worth one execution), `authorised.rs`
  (the only type meaning may-run, and both doors into it). 24 tests and two
  doctests, clippy clean.

  A change now travels `Call` → `Proposal` → `Approved` → `Authorised`, and
  each step can only be reached from the one before. **One approval, one
  execution** is carried twice over: the list refuses a proposal answered twice,
  and `Approved::redeem` takes `self`, so a second execution is not a program
  that compiles — which is what the `compile_fail` doctest on `redeem` asserts,
  with a twin that passes so the pair cannot rot into a test of a typo. The
  arguments travel inside the approval and there is no accessor lending them
  out, so an approval for one argument set cannot authorise another.

  Three decisions the next items inherit. **The grants are asked last at the
  moment of execution**, which is where a revoked grant becomes immediate; being
  permitted is checked when the call is made, again when it is proposed, and
  again there. **A read is never proposed and a change never takes the read
  door**, so ADR 0001 §5 is two refusals rather than a convention. **A refusal
  carries what it refused** (`Refused::call`), because item 4 records refusals
  and one that threw away the call could only say that something was stopped.

- [x] **4. The record** — implements ADR 0001 §7. Seven files in
  `crates/alo-record`, a **new crate** rather than more of `alo-capability`:
  `line.rs` (text as the record keeps it), `written.rs` (one argument),
  `what.rs` (what a call was), `happened.rs` (the four things that can happen),
  `entry.rs` (one moment and what happened at it), `record.rs` (the list),
  `explain.rs` (what can be asked). 42 tests, clippy clean.

  **Why a crate of its own**, against what `alo-capability`'s docs previously
  said. The deciding crate must not deserialise — `Call`, `Value` and `Proposal`
  serialise and deliberately do not read back — while a record exists to be read
  back, so it needs its own types either way. And the record needs
  `InferenceSource` from `alo-models`, which would have put an HTTP client and a
  TLS stack behind the crate whose whole value is being small enough to audit.
  `alo-capability` still depends on nothing but serde and thiserror.

  Three decisions the next items inherit. **`Grants::permitting` answers with
  the grant rather than with a yes**, and `permits`/`refusal` are now two halves
  of that one search — so nothing can be permitted by a grant the record cannot
  name, and `Authorised::against` carries it. **A record is evidence, not an
  instruction**: nothing in `alo-record` turns back into a call, and nothing
  takes an entry out. **Two things are never kept** — the question a person
  asked, and the arguments of a call that never validated.

- [ ] **4a. Where the record is written, and what prunes it** — the daemon's,
  cut from item 4 deliberately. `Record` keeps everything and has no `forget`,
  because how long evidence is kept is one decision made in the open rather than
  a method anything holding the list can reach for. That decision, the file it
  is written to, and the appending are `alo-agentd`'s and do not exist yet.

- [x] **5. Egress policy** — implements law 1 §8. Five files in
  `crates/alo-egress`, a **new crate**: `destination.rs` (where something is
  going, and what may be shown as an address), `leaving.rs` (one egress about to
  happen — who, where to, why), `policy.rs` (what an organisation permits, and
  the refusal in words), `departing.rs` (the only type meaning may-leave),
  `indicator.rs` (what is leaving right now). 27 tests and 3 doctests, clippy
  clean.

  **The decision and the indicator are one call**, and that is the whole design:
  `Indicator::beginning` asks the policy and shows the result, and it is the only
  constructor of `Departing` — the token a caller must hold before it opens a
  connection. *Permitted but not shown* is therefore not a state that exists,
  which is the guarantee `CLAUDE.md` names, carried by a type rather than by
  whoever writes the next verb remembering. Ending a departure takes `self`, so
  one connection can never take two lines off the indicator; the `compile_fail`
  doctest asserts it, with a passing twin.

  Three decisions the next items inherit. **The rule is stated once**:
  `EgressPolicy` is made `From<&SourcePolicy>` rather than written down again,
  and a test walks every policy against every source to prove the wider boundary
  and the inference one cannot disagree. **A question answered on this machine is
  not a departure** — `Leaving::asking` refuses it — so law 1's zero-egress claim
  is the absence of a type rather than a counter that happens to read zero.
  **Only a paired machine is in the building**: a host that answers on the same
  wire is outside it, because ADR 0003 refuses "it is only our internal network".

  Enforcement at the network boundary is Linux and is a later item; this crate
  guarantees the ordinary path, not code that never asked.

- [x] **5a. The record of what left** — implements law 1's second half in
  `alo-record`. One new file, `departed.rs` (the only door from an egress into
  the record), plus three new `Happened` variants and the query behind them. 12
  new tests and 3 new doctests, two of them `compile_fail`; 218 tests and 8
  doctests across the workspace, clippy clean.

  **The decision, which the item asked for before it asked for code.** Neither
  of the two options as written: `Answered` did not become provenance beside a
  departure, it became `AnsweredHere` and stopped having a source at all. A
  question answered somewhere else *is* the departure it caused, so it is
  `Happened::Left` and nothing beside it; a question answered here is
  `Happened::AnsweredHere`, which has nowhere to name and no field to name it
  in. Nothing is lost — `Destination` says everything `InferenceSource` said,
  and `alo-egress` already maps one to the other in one place — and two things
  are gained. `caused_egress` became a variant rather than a calculation, so no
  two readers can work it out differently. And the record can no longer
  contradict itself: saying *the answer came from a provider* now requires a
  `Departing`, which only the indicator makes.

  Three decisions the next items inherit. **An egress entry is unreachable
  except from a `Departing`, and a held-back one except from a `NotPermitted`**
  — both `pub(crate)`-constructed by the indicator, asserted by `compile_fail`
  doctests that were checked to fail on the privacy and not on a typo.
  **`Why` now deserialises**, additively, like `InferenceSource` gained
  `Serialize` in item 4; `Leaving` still does not. **`alo-record` depends on
  `alo-egress` and no longer on `alo-models`** except in tests — the observer
  reaches the decider, never the reverse.

- [ ] **6. File verbs, the portable half** — the verb definitions, argument
  types, grant checks and sentence generation for list, read, find, rename,
  move, archive. The filesystem calls are trivial; what is worth testing is that
  a path outside a grant never reaches them.

- [ ] **7. Keyboard shortcuts** — the binding model, defaults, user overrides and
  conflict detection. `docs/features.md` promises shortcuts a person can change,
  so the model must express a conflict rather than letting the last binding win.

- [ ] **8. Appearance** — the personalisation model from "Making it yours":
  background per display (file, rotating folder, or colour), lock-screen image,
  light/dark with a schedule, accent colour drawn from the design tokens, text
  scaling. Model and storage; the drawing is the compositor's.

- [ ] **9. Strings** — i18n scaffolding for the 24 official EU languages to begin
  with, and any language contributed after that (ADR-free, `CLAUDE.md`): the
  catalogue, the lookup, the fallback chain, and a test that a missing
  translation is visible in development rather than silently English. No
  translations yet — the scaffolding is what stops English being hardcoded while
  the shell is written.

- [ ] **10. Test a provider before saving it** — promised at v0.5 in
  `docs/features.md` and the one loose end in `provider.rs`. A mistyped key
  should be found when it is typed, not in the middle of a question. Reuse the
  stub-server pattern from `ollama.rs`'s tests; do not add an HTTP client, ureq
  is already here.

---

## Blocked — linux

Not this loop's, on this machine. Listed so the queue is a true picture of v0.01
rather than only of what is convenient.

- **Compositor** — Wayland via Smithay, one display, keyboard and pointer.
- **Sign-in and the local account**, the agent overlay, the launcher and window
  management, copy and paste, window switching — all draw on the compositor.
- **File and application verbs, the acting half** — AT-SPI, D-Bus, the portal
  backend (ADR 0005).
- **Egress enforcement** — item 5's policy, made true at the network boundary.
- **The image** — OCI-built, bootable, atomic.
- **The workspace client running as an application on the shell.**

## Blocked — hardware

- **The model stack against a real Ollama.** Ticked in `ROADMAP.md` as built and
  tested, with this verification owed. A CPU-only run would close most of it and
  needs no GPU — but it needs Ollama installed, which is a person's decision to
  make rather than a loop's.
- **"The GPU works on first boot"**, which needs a machine that has one.
- **The v0.01 exit gate** — one person, one machine, one cold boot.

## Not ours

- **Agents point at the local model by default** — `alo-workplace`, and
  configuration rather than code (`AiConfig` has spoken to an OpenAI-compatible
  endpoint since 2025).
