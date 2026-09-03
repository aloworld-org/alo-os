# alo OS — features.md

Feature inventory. Three tiers, mapped to the releases in `ROADMAP.md`:
**[v0.01]** = it boots and the agent acts · **[v0.5]** = a person can work on it
all day · **[v1]** = an organisation can buy it — from a fifty-seat firm to one
with thousands of machines, which is a wider bar than it sounds and is where
most of this list comes from. **★** marks differentiators —
things no other operating system offers.

Rule of the file: **nothing gets built that isn't listed here, and nothing gets
listed without a tier.** Additions go through the scope gate — this file, the
current release, and Non-goals below.

---

## The shell — what a person signs into (ADR 0002)

- [v0.01] **The colours come from a source this repository can read, and it is not CSS.** There is no CSS in alo OS and there will not be — the shell is native Rust (ADR 0002). But alo's palette currently *lives* in `alo-workplace`'s `tokens.css`, which a Rust compositor cannot read, so the source moves somewhere language-neutral and **generates** both: constants for this shell, and the custom properties the workspace's web client still needs. This ends CSS's authority over the palette rather than importing it, and nothing below can be drawn in alo's own colours until it exists
- [v0.01] Compositor: Wayland via Smithay, one display, keyboard and pointer
- [v0.01] Sign-in with an alo identity, and a local account that needs no tenant
- [v0.01] ★ The agent overlay: one key, from anywhere, with the current context offered and never harvested
- [v0.01] Launcher and window management: open, focus, close, tile
- [v0.5] Lock screen, suspend and resume
- [v0.5] Multi-monitor, display scaling, hotplug
- [v0.5] Recovery and rollback screen — reachable when the workspace is not
- [v0.5] **Settings, as one place** — network, display, sound, printers, storage, keyboard, accounts, privacy, updates. Not a scattering of dialogues a person has to know the name of
- [v0.5] Accessibility: the AT-SPI tree the agent uses is the one a screen reader uses; EN 301 549 conformance is the same work, not extra work
- [v1] Multi-user on one machine, with per-person grants and no shared agent memory

**Making it yours**

The first thing anybody does with a new machine is change the picture. It is not
a small feature: it is the moment a person decides whether the system is theirs
or the company's, and an operating system that cannot do it feels unfinished
however good the rest is.

- [v0.5] **Set the background** — from a file, a folder that rotates, or a solid colour; per display on a multi-monitor desk
- [v0.5] Set the lock-screen image, independently of the desktop
- [v0.5] **Light and dark**, following the time of day if a person wants
- [v0.5] **Accent colour** — five designed hues, each with a value for a light ground and one for a dark, so it reads properly either way. The whole shell follows it, not one button (ADR 0010)
- [v0.5] ★ **Terracotta is not one of them.** It means the agent and nothing else, so it is reserved rather than offered — an accent somebody could set to terracotta would take away the one signal that says the machine is acting on their behalf
- [v0.5] ★ **The agent is never signalled by colour alone** — terracotta always arrives with a mark and a word. A signal carried by hue fails for anybody who cannot distinguish that hue, and EN 301 549 does not allow colour to be the only means of conveying anything
- [v0.5] Text size and scaling, which is an accessibility setting as much as a taste one
- [v0.5] Wallpapers shipped with the image, so a fresh machine is not grey
- [v1] Cursor size and colour; sounds, including silencing them
- [v1] ★ **Ask for it** — "make the background this photo", "use dark after six" — the same propose-then-approve as any other change, because personalisation is exactly the low-stakes place people first learn to trust the agent
- [v1] Themes as a document, so a machine's look can be set once and applied across a fleet (ADR 0004)

## The ordinary things a desktop must do

Everything above is why alo OS is worth building. **This section is why it is
usable**, and it is where most of the engineering actually is. It is also the
whole product for anybody who declines the agent (ADR 0009) — so the bar is not
"good enough alongside an agent" but "worth choosing with the agent switched
off". An operating
system with a brilliant agent and no working Bluetooth is not a product, and a
feature list that skips copy and paste is not honest about the work.

Nothing here is a differentiator. All of it is required.

**Input and interaction**

- [v0.01] Copy, cut and paste — text, images and files, across applications
- [v0.01] Keyboard shortcuts, and a person can change them
- [v0.01] Window management: move, resize, snap, tile, minimise, maximise, close
- [v0.01] **The dock, and the person decides where it goes** — bottom, left, right or top, chosen in Settings. It works in both orientations rather than being a horizontal bar someone turned sideways: the status area reflows, and labels give way to icons where the short edge demands it
- [v0.5] The dock's size, and whether it hides when a window needs the room
- [v0.5] Per display, so the dock can sit along the bottom of the laptop and down the side of the external screen
- [v0.5] ★ **Divide the screen** — drag a window to an edge to take half, a corner to take a quarter, or split what is already open with the keyboard. The split holds while you work: resizing one side resizes its neighbour rather than overlapping it
- [v0.5] Remember a split, so returning to a pair of windows restores the arrangement rather than the last position of each
- [v0.5] Splitting works on an external display independently of the laptop's own
- [v0.01] Switching between windows, and between applications
- [v0.5] Drag and drop between applications
- [v0.5] Right-click context menus, wherever a person expects one
- [v0.5] Touchpad gestures: scroll, zoom, swipe between workspaces
- [v0.5] **Keyboard layouts, switched easily** — and dead keys and a compose key that work. "Müller" and "Liège" are test cases in a European product, not edge cases
- [v0.5] Input methods for non-Latin scripts
- [v0.5] Virtual desktops
- [v1] Clipboard history, on the machine and never synced anywhere

**Capture**

- [v0.5] Screenshots: whole screen, one window, a selected region — to a file or the clipboard
- [v0.5] Annotate a screenshot without opening anything else
- [v0.5] **Screen recording**, with audio, to a file
- [v0.5] Screen sharing for calls
- [v0.5] ★ **A visible indicator whenever the screen, camera or microphone is in use** — by any application, including ours. Law 1 is about egress; this is the same instinct applied to the room you are sitting in

**Desktop**

- [v0.5] Notifications, with do-not-disturb
- [v0.5] Status area: clock, battery, network, volume, brightness — at the far end of the dock, wherever the dock is
- [v0.5] ★ The egress indicator lives in the status area, so "nothing has left this machine" sits where a person already glances rather than somewhere they must learn to look
- [v0.5] A file manager, with trash, and archives that open
- [v0.5] USB drives and external storage that appear when plugged in
- [v0.5] File associations — what opens what, changeable by a person
- [v0.5] A text editor and an image viewer, so a fresh machine is not helpless
- [v0.5] **A terminal.** Law 2 forbids the *agent* running arbitrary commands; it says nothing about a person, and an operating system that does not trust its owner with a shell is a toy

**Software, and what applications expect (ADR 0005)**

Applications install sandboxed and reach the system through the XDG Desktop
Portal interfaces — the contract every Linux application already speaks. Meeting
it is what makes existing software work on alo OS without knowing we exist, and
each portal request is a grant in the sense of ADR 0001.

- [v0.5] **Install applications**, sandboxed, from Flathub or a repository the organisation runs; update and remove them
- [v0.5] ★ **One list of what has been granted to what** — agents and applications in the same place, revoked the same way
- [v0.5] Portals: file chooser and documents, open-with and default applications, notifications, print, screenshot, screen capture, camera, microphone, clipboard, trash, wallpaper, settings, inhibit (no sleep mid-presentation), network and power-profile monitors
- [v0.5] **Secret storage** — one keyring behind the Secret portal, so applications stop inventing credential storage
- [v0.5] **Session management**: log out, switch user, lock, and reopen what was open
- [v0.5] **Corporate proxy support**, machine-wide and honoured by applications. A great many company networks have no other route out
- [v1] Portals: USB devices, global shortcuts an application registers, dynamic launchers, remote desktop
- [v1] **Location services**, off by default, per-application, with an indicator when in use
- [v1] Applications contribute to search — one place to look, not one per program
- [v1] Realtime scheduling for audio work, which is what a workstation is often bought for
- [v1] Unsandboxed installation as a deliberate, clearly-marked act — never the default, and forbiddable by policy on a managed machine

**Devices and media**

- [v0.5] Audio in and out, with device switching that works mid-call
- [v0.5] Bluetooth: pairing, audio, keyboards, mice
- [v0.5] Camera and microphone
- [v0.5] Media playback, and the codecs people actually have files in
- [v0.5] Power management, battery, sleep on lid close
- [v0.5] Night light and display colour

**Language and access**

- [v0.5] **The shell in the user's language — all 24 official EU languages to begin with**, and any language somebody contributes after that. Bulgarian, Croatian, Czech, Danish, Dutch, English, Estonian, Finnish, French, German, Greek, Hungarian, Irish, Italian, Latvian, Lithuanian, Maltese, Polish, Portuguese, Romanian, Slovak, Slovenian, Spanish and Swedish. Not "English plus the big five": a sovereignty product that cannot speak Maltese or Irish is selling sovereignty to some Europeans and not others, and those are exactly the member states with the least software in their own language
- [v0.5] Regional formats and timezones per language, and a keyboard layout offered with it — choosing Greek and then hunting for a Greek keyboard is the same bug twice
- [v0.5] **Right-to-left ready**, so adding a language later is translation rather than rework, even though no official EU language needs it today
- [v0.5] ★ **The agent answers in the language you asked in** — the shell being translated is table stakes; being able to say "wo ist die Rechnung von Northstar?" and get an answer is the thing a cloud assistant does badly for smaller languages
- [v1] EEA and candidate languages as translations arrive: Norwegian, Icelandic, and the accession languages
- [v1] Community translation, so a language nobody sold us on can still be complete
- [v0.5] Screen reader, magnifier, high contrast, larger text
- [v1] ★ **A published accessibility conformance report against EN 301 549** — the harmonised European standard, and the mandatory technical specification for public-sector ICT procurement across the EU. Procurement asks for the report, not the intention
- [v0.5] Sticky keys, slow keys, and keyboard-only operation of everything
- [v1] Voice control of the shell — which for us is the agent, arriving somewhere it was always going

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

- [v0.01] ★ **It runs on the machine you already own.** No graphics card required: the catalogue carries models that answer comfortably on an ordinary business laptop's CPU, and the system picks one (ADR 0007). This is what puts alo OS on the Windows 10 fleet rather than on a few hundred workstations
- [v0.01] ★ **The GPU works on first boot**, where there is one — no driver installation, no CUDA archaeology. Acceleration, not an entry price
- [v0.01] ★ **A model runs in one command**, from a curated catalogue of open-weight models with their licences stated
- [v0.01] ★ The agents point at the **local** model by default — sovereignty is the default configuration, not an option to find
- [v0.01] ★ **Add your own provider in Settings** — a name, an address, and a key: Mistral, your own endpoint, or whatever you already pay for. The key goes to the keyring, never into a settings file, so it cannot leak through a backup or a support bundle. You say where the provider runs; nothing is guessed from its address
- [v0.5] An address that is not https is refused rather than warned about, unless it is a service on this machine — "it is only our internal network" is how a key ends up on the wire in clear
- [v0.5] Test a provider before saving it, so a mistyped key is found now rather than in the middle of a question
- [v0.01] ★ **Or not at all** (ADR 0009). Setup's fourth choice, with the same weight as the other three: no model, no provider, no agent. Everything else in this document still works, the agent's surfaces are absent rather than greyed out, and turning it on later is a setting rather than a reinstall
- [v0.01] **Anything an agent verb can do, a person can do by hand** (ADR 0009). A standing rule rather than a feature, and the check on every verb anybody proposes: a machine with the agent switched off must lose *convenience* and never *capability*. It was a consequence of that ADR with no entry here until an audit found it missing
- [v0.01] ★ **Or use an API instead** (ADR 0008). A model may answer on this machine, on a machine on your network, or behind a provider's API — for a laptop too thin to run one, or an organisation that would rather buy inference than operate it
- [v0.01] ★ **Where the answer came from is said where the answer appears** — "on this machine", "on the studio workstation, on your network", "by alo, in the EU". Not in a settings page somebody would have to go looking for, because a person about to paste a contract into a question is entitled to know where it is going first
- [v0.01] ★ **Never a silent fallback.** A local model that fails does not quietly become an API call: failing to answer is recoverable, a person's records leaving the building because a download was corrupt is not
- [v0.5] A provider that will not say where it runs is reported as **unknown**, never assumed to be nearby — and unknown never satisfies a policy naming a region
- [v1] Policy over where inference may happen: anywhere, in the building, inside a region **the organisation names**, or this machine alone. We ship the mechanism, never a region of our own (ADR 0004, ADR 0008)
- [v0.01] Model lifecycle: pull, list, serve, unload, remove; disk accounted honestly
- [v0.5] ★ **Guided fine-tune**: LoRA/QLoRA over a granted folder or a tenant's records, as a flow rather than a toolchain
- [v0.5] ★ The dataset, the adapter and the resulting weights never leave the machine
- [v0.5] Model runtime versioned *with* the drivers it needs, so an upgrade cannot break a working stack
- [v1] Serving more than one person from one workstation
- [v1] Evaluation: compare a fine-tune against the base model on your own questions, before trusting it

## Everyday pain — what people actually complain about

Not a category an operating system usually has. Each of these is a thing
everybody has suffered this month, none of them is solvable without an agent
that can reach the machine, and every one of them demonstrates in fifteen
seconds.

- [v0.5] ★ **"Where is that file?"** Ask in words — *"the contract Anna sent before the summer"* — over granted paths, indexed on the machine and never uploaded. Every cloud assistant can do this if you send it your documents; this one never sends them
- [v0.5] ★ **"Why is it slow?" and "what is filling my disk?"** Every operating system is opaque about itself and everyone has typed these into a search engine. With system verbs and the record, the agent can actually answer
- [v0.5] ★ **Printers, solved.** The agent finds it, sets it up, and fixes it when it stops. The most hated object in computing, and a small feature people tell other people about
- [v0.5] ★ **"I can't open this file."** A `.pages`, a `.heic`, a `.dwg`: the system converts it where it can, and where it cannot says plainly what will open it, instead of shrugging
- [v0.5] ★ **Undo what the agent did.** Every execution is already recorded with its origin and the image already rolls back — together they make *"undo everything the agent did this afternoon"* real. An agent you can reverse is an agent people let do more, and no other system offers it
- [v0.5] **Updates that never interrupt.** Atomic images mean an update can be genuinely invisible and instantly reversible. On the system people are leaving, this is the single most hated behaviour there is
- [v1] ★ **"Make this machine like my old one."** Configuration as a document, pointed at a person rather than an administrator: a replacement machine that is actually yours, not a week of rebuilding
- [v1] **A new colleague working on day one** — a managed machine that arrives with the right applications, policy and grants already in place

## The local network — machines that find each other (ADR 0003)

Discovery is open; **use requires a deliberate pairing on both machines**. Being
on the same WiFi confers nothing.

- [v0.5] Machines find each other with zero configuration — no addresses typed, no accounts
- [v0.5] ★ **One GPU box serves the office**: a machine without a GPU discovers the one with it, and the agents just work. The inference never leaves the building; it moves down the corridor. **It is still egress, and the indicator still fires** (ADR 0003) — *"it only went to the machine down the corridor"* is exactly the kind of exception that quietly ends a guarantee, so shared inference is shown like any other departure and the pairing is what makes it wanted rather than what makes it silent
- [v0.5] Pairing: mutual, deliberate, enumerated, revocable in one action, and expiring — grants, across a machine boundary
- [v0.5] ★ **The whole of it works with no internet at all.** An office that cannot connect still has working AI
- [v0.5] A self-hosted workspace on the network is **discovered, not configured** — no DNS step
- [v1] Files and printers shared between paired alo machines, with no server in the middle
- [v1] Enrollment by discovery: a new machine appears to the fleet and asks; an administrator admits it
- [v1] ★ Cross-machine agent work — an agent may **ask** a paired machine, and acts only under a grant made **on that machine, by its person**

## Identity, fleet and compliance — what a large organisation requires (ADR 0004)

A machine is **personal** — nobody above the person — or **managed**, in which
case the organisation sets policy and holds a recovery key, and the person is
told so at first sign-in. There is no silent enrollment.

- [v1] **Sign in with the organisation's own identity provider** — SAML/OIDC against Entra ID, Okta or Keycloak. Nobody maintains a second set of identities for us
- [v1] **Smartcard and national eID sign-in** — eIDAS and government ID cards; in much of EU public sector this is required, not preferred
- [v1] **Disk-encryption key escrow**, so a machine survives the person leaving
- [v1] **Remote lock and wipe** for a machine that is lost — destructive by design, and recorded
- [v1] **Records export to their SIEM** — Splunk, Sentinel, Elastic, over syslog/OpenTelemetry. A security team needs agent actions in *their* console, not ours
- [v1] **Update rings**: canary then broad, haltable. No organisation updates a fleet at once
- [v1] **An update mirror they host**, for machines that never reach the internet
- [v1] ★ **A private model catalogue** — the organisation curates which models may run, served from inside. No one lets staff pull arbitrary weights
- [v1] ★ **An adapter allowlist**, signed and centrally permitted — an adapter is code that drives your applications
- [v1] ★ **Agent policy by role**: which verbs, adapters and models, per department. A finance team's agent may raise an invoice; an intern's may not
- [v1] ★ **Agent retention policy, centrally set** — what agents remember and for how long. A GDPR question with an actual answer
- [v1] ★ **Inference accounting** — which team used the GPU, and for what. Whoever paid for the workstation asks within a month
- [v1] ★ **Egress attestation**: a signed, printable statement of exactly what left this machine in a period. The artifact an auditor asks for and nobody can currently produce
- [v1] Configuration as a document — image, policy, adapters and settings declared in one file, so an identical machine can be rebuilt
- [v1] Helpdesk assistance as a **session a person starts and can end**, never a capability an administrator holds
- [v1] Certification groundwork: ISO 27001, BSI Grundschutz, ANSSI, Common Criteria. Years and money rather than code, which is why it starts early

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
weights. **No general-purpose distribution** — no package manager for the world and
no attempt to be Ubuntu; software is installed as sandboxed Flatpaks from
repositories other people run (ADR 0005), which is how we have applications
without packaging them. **No third-party device management** — fleet features
exist for alo OS machines; an MDM product is a different company. **No phone or
tablet** — not in v1, possibly never. **No directory service** — we do not
rebuild Active Directory or LDAP, and we do not become the place a company's
identities live; alo identities and pairing are what we offer. **No trusted
network setting**, ever (ADR 0003) — the switch that would turn pairing off is
the vulnerability, not a convenience we have not got round to. **No arbitrary command verb**, ever
(ADR 0001 §1); this one is not a scope decision and is not revisitable without
replacing that ADR.

Every absence here is a sales argument.
