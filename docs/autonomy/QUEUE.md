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

- [ ] **1. Grants** — implements ADR 0001 §3 and `docs/contracts/agent-verbs.md`.
  A grant is what makes reach possible: enumerated, visible, revocable in one
  action taking effect immediately, and expiring by default. No grant to `/`.
  Model, storage and the queries the shell will ask ("what is granted, to whom,
  until when"). Tests must include: an expired grant grants nothing; a revoked
  one stops immediately; a path outside a grant is refused; a grant is never
  widened by use. **Everything after this depends on its vocabulary — read the
  ADR in full, not the summary here.**

- [ ] **2. The verb registry** — implements `docs/contracts/agent-verbs.md` and
  law 2. A verb is `name`, `purpose`, `effect` (read or change), typed `args`,
  the grant it requires, and how its approval sentence is generated **from the
  validated arguments**. The registry refuses to hold a verb that breaks the
  contract: an argument that reaches an interpreter, a change whose sentence
  cannot be generated, a verb requiring no grant without a written reason.

- [ ] **3. Approvals** — implements ADR 0001 §5. One approval, one execution, of
  exactly those arguments. No "remember this", no duration, no "always allow for
  this application" — durable permission is a grant. Tests: an approval cannot
  be replayed; an approval for one argument set does not authorise another;
  approving nothing runs nothing.

- [ ] **4. The record** — implements ADR 0001 §7. Every execution *and every
  refusal*, with what ran, under whose authority, from which approval, against
  which grant. "Explain what it did" is a query, not a log to grep. Refusals
  matter most: a record keeping only successes cannot answer what a security
  review actually asks. **Also records the inference source** (ADR 0008), so
  "where did that answer come from" is answerable afterwards and not only at the
  moment it appeared.

- [ ] **5. Egress policy** — implements law 1, and now sits on `source.rs` rather
  than starting from nothing. The decision and the indicator event: what an agent
  is about to cause to leave, where to, and the record of it. `SourcePolicy`
  already decides whether a *source* is permitted; this item is the wider
  boundary — any egress an agent causes, not only inference. Enforcement is
  Linux and is a later item.

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
