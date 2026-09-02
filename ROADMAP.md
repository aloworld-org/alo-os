# alo OS — ROADMAP.md

The only order things get built in. Items are checked when they meet the
definition of done in `CLAUDE.md` law 3 — the full path, on real hardware — and
a release is done only when its **exit gate** is fully checked.

Every item here appears in `docs/features.md` with a tier. If it is not there,
it is not built.

---

## v0.01 — it boots and the agent acts

The point is to prove one sentence on real hardware: *an action a person would
take by hand can be proposed by an agent, approved in one click, and afterwards
explained — and the model that proposed it ran on the customer's own machine.*

One hardware target. No installer, no fleet management, no compatibility list.

**Order.** The AI stack comes first because it needs no kernel work, no image
pipeline and no certified hardware — it runs on an ordinary Linux box with a
GPU, and it is useful to `alo-workplace` the day it lands.

- [ ] **Model stack**: catalogue, pull, serve, unload, remove — over the pinned runtime
- [ ] **Agents point at the local model by default**, configured rather than coded
- [ ] **The GPU works on first boot** on the certified machine — drivers and runtime pinned together
- [ ] **Egress indicator**, and no telemetry
- [ ] **`alo-agentd`**: grants, file verbs, application verbs, context on invocation
- [ ] Every execution recorded with its origin, approval and grant
- [ ] **Compositor**: Wayland via Smithay, one display, keyboard and pointer
- [ ] **Sign-in**: alo identity, and a local account that needs no tenant
- [ ] **The agent overlay**: one key, from anywhere
- [ ] Launcher and window management
- [ ] The workspace client runs as an application on the shell
- [ ] **Image**: OCI-built, boots on the certified machine, firmware to sign-in

**Exit gate.** On the certified machine, from a cold boot: sign in, press the
key, ask an agent to do something to a file in a granted folder, approve the
sentence, see it happen, and afterwards ask what it did and get an answer from
the record — with the egress indicator having stayed dark throughout.

---

## v0.5 — a person can work on it all day

Everything that turns a demonstration into a machine somebody uses on a Tuesday.

- [ ] Lock screen, suspend and resume
- [ ] Multi-monitor, scaling, hotplug
- [ ] Recovery and rollback screen
- [ ] Settings: network, display, sound, printers, storage, keyboard
- [ ] **Printing**
- [ ] `.docx`, `.xlsx`, `.pptx` open
- [ ] A web browser for the open web
- [ ] **Application adapters**, and the accessibility fallback for applications without one
- [ ] **System verbs** through the privileged broker
- [ ] **Guided fine-tune**, with the dataset never leaving the machine
- [ ] Full-disk encryption
- [ ] **Atomic updates with rollback**
- [ ] Installer
- [ ] Accessibility: EN 301 549 conformance on the shell
- [ ] **Machines find each other** on a local network, with pairing
- [ ] **One GPU box serves the office** — shared local inference over a pairing
- [ ] A self-hosted workspace on the network is discovered, not configured
- [ ] **Zero inference egress over a working day**, measured and published

**Exit gate.** A person works a full day on alo OS — mail, documents,
a video call, printing something, driving one installed application through its
agent — and does not need another machine. An update lands and rolls back
cleanly.

---

## v1 — an organisation can buy it

- [ ] Fleet enrollment by discovery — the machine asks, an administrator admits it
- [ ] Fleet policy and signed updates, for alo OS machines
- [ ] Files and printers shared between paired machines
- [ ] Cross-machine agent work, under grants made on the target machine
- [ ] Signed images verified before boot; Secure Boot with our key
- [ ] Backup and restore
- [ ] **Adapter SDK published**, with a conformance suite
- [ ] Multi-user on one machine, with per-person grants
- [ ] **Third-party security audit** of `alo-agentd` and the broker, published
- [ ] Compatibility list, grown outward from the certified machine
- [ ] Support and SLA definitions

**Exit gate.** A fifty-seat organisation enrols a fleet, sets policy, receives a
signed update, and has a documented egress guarantee, a published audit and
somebody to call.

---

## Not scheduled

The engine (`alo-engine`) replaces the workspace's rendering incrementally and
has its own roadmap. alo OS Desktop — the non-GPU SKU for the Windows 10 fleet —
follows once the AI SKU has customers, and inherits everything built here.
