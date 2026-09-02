# Queue — v0.01

Worked in order by `LOOP.md`. Every item names what it implements, so an
iteration can read the reasoning rather than guess at it.

## The honest constraint

**This loop cannot finish v0.01, and no amount of iterations will change that.**
Roughly half of what v0.01 needs ends on hardware and an operating system this
loop does not have:

- **A Wayland compositor, the portals and the bootable image are Linux.** They
  will not compile, let alone run, on the machine the loop runs on today.
- **"The GPU works on first boot" and the exit gate are the certified machine.**
  They are verified by somebody sitting in front of it, once, and no test suite
  substitutes for that.
- **"Agents point at the local model" belongs to `alo-workplace`.** Different
  repository, and this loop never touches another one.

What the loop *can* finish is the part everything else stands on: the capability
model as working code. Grants, verbs, approvals and the record are pure logic —
portable, exhaustively testable, and the thing that makes `alo-agentd` a service
worth writing rather than a daemon with a permission dialogue. Doing that first
is also the right order: the Linux work becomes an implementation of a settled
model instead of a place where the model gets decided by accident.

Items are therefore either **ready** (buildable and testable here), **linux**
(needs a Linux host), or **hardware** (needs the certified machine). The loop
takes ready items and stops when there are none left.

---

## Ready

- [ ] **1. Grants** — implements ADR 0001 §3 and `docs/contracts/agent-verbs.md`.
  A grant is what makes reach possible: enumerated, visible, revocable in one
  action taking effect immediately, and expiring by default. No grant to `/`.
  Model, storage and the queries the shell will ask ("what is granted, to whom,
  until when"). Tests must include: an expired grant grants nothing; a revoked
  one stops immediately; a path outside a grant is refused; a grant is never
  widened by use.

- [ ] **2. The verb registry** — implements `docs/contracts/agent-verbs.md` and
  law 2. A verb is `name`, `purpose`, `effect` (read or change), typed `args`,
  the grant it requires, and how its approval sentence is generated **from the
  validated arguments**. The registry refuses to hold a verb that breaks the
  contract: an argument that reaches an interpreter, a change whose sentence
  cannot be generated, a verb requiring no grant without a written reason.

- [ ] **3. Approvals** — implements ADR 0001 §5. One approval, one execution,
  of exactly those arguments. There is no "remember this", no duration, no
  "always allow for this application" — durable permission is a grant, made
  deliberately. Tests must include: an approval cannot be replayed; an approval
  for one argument set does not authorise another; approving nothing runs
  nothing.

- [ ] **4. The record** — implements ADR 0001 §7. Every execution *and every
  refusal*, with what ran, under whose authority, from which approval, against
  which grant. "Explain what it did" is a query, not a log to grep. Refusals
  matter most: a record that keeps only successes cannot answer the question a
  security review actually asks.

- [ ] **5. Egress policy** — implements law 1. The decision and the indicator
  event: what an agent is about to cause to leave, and the record of it. The
  *enforcement* is Linux and is a later item; the policy that decides is
  portable and belongs here, along with the rule that management traffic and
  shared inference are egress like anything else (ADR 0003, ADR 0004).

- [ ] **6. File verbs, the portable half** — the verb definitions, argument
  types, grant checks and sentence generation for list, read, find, rename,
  move, archive. The filesystem calls themselves are trivial; what is worth
  testing is that a path outside a grant never reaches them.

- [ ] **7. Keyboard shortcuts** — the binding model, defaults, user overrides
  and conflict detection. `docs/features.md` promises shortcuts a person can
  change, which means the model must express a conflict rather than silently
  letting the last binding win.

- [ ] **8. Appearance** — the personalisation model from "Making it yours":
  background per display (file, rotating folder, or colour), lock-screen image,
  light/dark with a schedule, accent colour drawn from the design tokens, text
  scaling. Model and storage; the drawing is the compositor's.

- [ ] **9. Strings** — the i18n scaffolding for all 24 official EU languages:
  the catalogue, the lookup, the fallback chain, and a test that a missing
  translation is visible in development rather than silently English. No
  translations yet — the scaffolding is what stops English being hardcoded
  while the shell is written.

---

## Blocked — linux

Not this loop's, on this machine. Listed so the queue is a true picture of
v0.01 rather than only of what is convenient.

- **Compositor** — Wayland via Smithay, one display, keyboard and pointer.
- **Sign-in and the local account**, the agent overlay, the launcher and window
  management, copy and paste, window switching — all of them draw on the
  compositor.
- **File and application verbs, the acting half** — AT-SPI, D-Bus, the portal
  backend (ADR 0005).
- **Egress enforcement** — the policy from item 5, made true at the network
  boundary.
- **The image** — OCI-built, bootable, atomic.
- **The workspace client running as an application on the shell.**

## Blocked — hardware

- **The GPU works on first boot** on the certified machine.
- **The model stack against a real Ollama and a real GPU** — ticked in
  `ROADMAP.md` as built and tested, with this verification explicitly owed.
- **The v0.01 exit gate**, which is one person, one machine, one cold boot.

## Not ours

- **Agents point at the local model by default** — `alo-workplace`, and
  configuration rather than code (`AiConfig` has spoken to an OpenAI-compatible
  endpoint since 2025).
