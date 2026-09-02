# alo OS — features.md

Feature inventory. Three tiers, mapped to the releases in `ROADMAP.md`:
**[v0.01]** = it boots and the agent acts · **[v0.5]** = a person can work on it
all day · **[v1]** = an organisation can buy it. **★** marks differentiators —
things no other operating system offers.

Rule of the file: **nothing gets built that isn't listed here, and nothing gets
listed without a tier.** Additions go through the scope gate — this file, the
current release, and Non-goals below.

---

## The shell — what a person signs into (ADR 0002)

- [v0.01] Compositor: Wayland via Smithay, one display, keyboard and pointer
- [v0.01] Sign-in with an alo identity, and a local account that needs no tenant
- [v0.01] ★ The agent overlay: one key, from anywhere, with the current context offered and never harvested
- [v0.01] Launcher and window management: open, focus, close, tile
- [v0.5] Lock screen, suspend and resume
- [v0.5] Multi-monitor, display scaling, hotplug
- [v0.5] Recovery and rollback screen — reachable when the workspace is not
- [v0.5] Settings: network, display, sound, printers, storage, keyboard layout
- [v0.5] Accessibility: the AT-SPI tree the agent uses is the one a screen reader uses; EN 301 549 conformance is the same work, not extra work
- [v1] Themes and appearance, driven by the shared design tokens
- [v1] Multi-user on one machine, with per-person grants and no shared agent memory

## `alo-agentd` — the agent's reach into the machine (ADR 0001)

- [v0.01] ★ **File verbs**: list, read, find, rename, move, archive — over granted paths only
- [v0.01] ★ **Application verbs**: open, focus, arrange, close
- [v0.01] ★ **Context on invocation**: focused window, selection, open document — offered, never watched
- [v0.01] ★ Grants: pick a folder, see what is granted, revoke it, and it expires
- [v0.01] Every execution recorded with its origin, approval and grant
- [v0.5] ★ **System verbs** through the privileged broker: printers, network, updates, storage
- [v0.5] ★ **Application adapters** — installed applications become agents with typed verbs (`@blender`, `@resolve`, `@gimp`); see `docs/contracts/app-adapters.md`
- [v0.5] The accessibility fallback: any application with no adapter is still readable and operable through its AT-SPI tree
- [v1] ★ Adapter SDK published, with a conformance suite third parties can run
- [v1] Policy: which verbs and adapters are permitted, set per machine or per fleet
- [v1] Screenshot-and-click, marked in the record and disabled by policy by default — last resort only, never the default mechanism

## The AI stack — models on your own hardware

- [v0.01] ★ **The GPU works on first boot.** No driver installation, no CUDA archaeology. This is the promise the SKU exists for.
- [v0.01] ★ **A model runs in one command**, from a curated catalogue of open-weight models with their licences stated
- [v0.01] ★ The agents point at the **local** model by default — sovereignty is the default configuration, not an option to find
- [v0.01] Model lifecycle: pull, list, serve, unload, remove; disk accounted honestly
- [v0.5] ★ **Guided fine-tune**: LoRA/QLoRA over a granted folder or a tenant's records, as a flow rather than a toolchain
- [v0.5] ★ The dataset, the adapter and the resulting weights never leave the machine
- [v0.5] Model runtime versioned *with* the drivers it needs, so an upgrade cannot break a working stack
- [v1] Serving more than one person from one workstation
- [v1] Evaluation: compare a fine-tune against the base model on your own questions, before trusting it

## The local network — machines that find each other (ADR 0003)

Discovery is open; **use requires a deliberate pairing on both machines**. Being
on the same WiFi confers nothing.

- [v0.5] Machines find each other with zero configuration — no addresses typed, no accounts
- [v0.5] ★ **One GPU box serves the office**: a machine without a GPU discovers the one with it, and the agents just work. The inference never leaves the building; it moves down the corridor.
- [v0.5] Pairing: mutual, deliberate, enumerated, revocable in one action, and expiring — grants, across a machine boundary
- [v0.5] ★ **The whole of it works with no internet at all.** An office that cannot connect still has working AI
- [v0.5] A self-hosted workspace on the network is **discovered, not configured** — no DNS step
- [v1] Files and printers shared between paired alo machines, with no server in the middle
- [v1] Enrollment by discovery: a new machine appears to the fleet and asks; an administrator admits it
- [v1] ★ Cross-machine agent work — an agent may **ask** a paired machine, and acts only under a grant made **on that machine, by its person**

## Sovereignty, as testable claims

- [v0.01] ★ **The egress indicator**: every network egress an agent causes, visible at the moment it happens
- [v0.01] ★ **No telemetry.** Not "anonymised telemetry". None — and the policy lives in a Rust service, not a checkbox
- [v0.5] ★ **A working day with a local model produces zero inference egress**, measured at the network boundary — and we publish the test
- [v0.5] Full-disk encryption, enrolled at install
- [v1] Signed images, verified before a deployment becomes bootable; Secure Boot with our key
- [v1] Third-party security audit of `alo-agentd` and the broker, published

## The system and the image

- [v0.01] Boots on one certified machine, firmware to sign-in
- [v0.01] Image built as an OCI container image; no third language enters the repository to build it
- [v0.5] ★ Atomic updates with rollback — the previous deployment stays bootable
- [v0.5] Printing. Unglamorous, and it decides public-sector deals
- [v0.5] The documents people are actually sent open: `.docx`, `.xlsx`, `.pptx`
- [v0.5] A web browser for the open web — a pinned upstream one, since our own engine is not scheduled
- [v0.5] Installer
- [v1] Fleet enrollment, policy and signed updates — **for alo OS machines only**
- [v1] Backup and restore
- [v1] A compatibility list, grown from the certified machine outward

---

## Non-goals

**No kernel.** Linux, unmodified — hardware support is where OS projects die and
we do not fight that battle. **No inference kernels** — we do not compete with
llama.cpp or vLLM. **No model training from scratch** — we serve and adapt open
weights. **No general-purpose distribution** — no package manager for the world,
no attempt to be Ubuntu. **No third-party device management** — fleet features
exist for alo OS machines; an MDM product is a different company. **No phone or
tablet** — not in v1, possibly never. **No directory service** — we do not
rebuild Active Directory or LDAP, and we do not become the place a company's
identities live; alo identities and pairing are what we offer. **No trusted
network setting**, ever (ADR 0003) — the switch that would turn pairing off is
the vulnerability, not a convenience we have not got round to. **No arbitrary command verb**, ever
(ADR 0001 §1); this one is not a scope decision and is not revisitable without
replacing that ADR.

Every absence here is a sales argument.
