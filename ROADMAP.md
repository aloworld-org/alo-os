# alo OS — ROADMAP.md

The only order things get built in. Items are checked when they meet the
definition of done in `CLAUDE.md` law 3 — the full path, on real hardware — and
a release is done only when its **exit gate** is fully checked.

**A tick means done, not written.** Where something is built and tested but has
not yet met law 3 on real hardware, the item says so in its own words rather
than being ticked optimistically. A roadmap whose ticks are aspirations is worse
than no roadmap, because somebody plans around it.

Every item here appears in `docs/features.md` with a tier. If it is not there,
it is not built.

## Three states, because two are not enough

Almost every item below is one capability spanning two halves: a crate that
decides, and a screen or daemon that shows and runs it. The crates are being
built first, deliberately — they need no compositor and no certified machine.
So an item is routinely *half done* for months, and a box that can only be empty
or ticked has no way to say that.

It said the wrong thing instead. For eight commits this file did not move while
eight crates and several hundred tests landed, and the two commits before that
moved it only by **adding** unticked lines. Read literally, it reported that
nothing had happened. That is a reporting failure, not a dishonest tick, but
somebody planning from it is misled either way.

So a line is written in one of three states, and there is no fourth:

- `- [ ]` **Not started.** Nothing exists.
- `- [ ]` **· Built: … · Owed: …** — part done. The clause names the crate that
  exists *and* the thing still missing. It stays an empty box: half a capability
  is not a capability, and a person planning around this file must never read
  the annotation as delivery.
- `- [x]` **Done** — law 3 on real hardware, or the line states in its own words
  what remains owed on the certified machine.

**The Built/Owed clause is a claim about code, so it names the crate.** A clause
that cannot name one is decoration, and decoration here is how a roadmap starts
lying slowly.

---

## v0.01 — it boots and the agent acts

The point is to prove one sentence on real hardware: *an action a person would
take by hand can be proposed by an agent, approved in one click, and afterwards
explained — and the model that proposed it ran on the customer's own machine.*

One hardware target. No installer, no fleet management, no compatibility list.

**Order.** The AI stack comes first because it needs no kernel work, no image
pipeline and no certified hardware — it runs on an ordinary Linux box with a
GPU, and it is useful to `alo-workplace` the day it lands.

- [x] **Model stack**: catalogue, pull, serve, unload, remove — over the pinned
      runtime (ADR 0006). `crates/alo-models`: the catalogue with its licence
      gate, `ModelRuntime`, and the Ollama adapter. 22 tests, nine of them
      against a real socket. **Not yet run against a real Ollama or a GPU** —
      law 3's "on real hardware" is owed on the certified machine.
- [ ] **Agents point at the local model by default**, configured rather than coded
      · Built: `alo-models` — where an answer may come from, and the policy that
      keeps it in the building, on the machine, or in a region, both now said in
      the language the person reads rather than in English · Owed: something
      that points, which is `alo-agentd`
- [ ] **Add your own provider in Settings** — name, address, key to the keyring;
      the region stated rather than guessed; https required off this machine
      · Built: `alo-models` — the provider, the key held as a keyring handle and
      never in the record, testing it before it is saved, and every refusal about
      any of the three readable in the reader's own language · Owed: the Settings
      panel to type it into
- [ ] **The GPU works on first boot** on the certified machine — drivers and runtime pinned together
- [ ] **Egress indicator**, and no telemetry
      · Built: `alo-egress` — what counts as leaving, and the line said about it
      while it happens, now in the language the person reads rather than in
      English; the policy still decides before a socket opens and without
      needing a vocabulary to do it · Owed: the indicator itself, which is a
      compositor surface
- [ ] **`alo-agentd`**: grants, file verbs, application verbs, context on invocation
      · Built: `alo-capability` (the verbs, the grants, the approvals, every
      refusal of theirs said in the language the person reads, and — since 9g —
      the sentence a person approves, carried as what names it and the values
      that fill it rather than as words in whichever language the verb was
      declared in), `alo-files` (the six file verbs, declared from the words
      a translator is handed) and `alo-applications` (all four application
      verbs — open, focus, ask-to-close and arrange — the list of what is
      installed they are checked against, and the rule that an ungranted
      application refuses identically whether or not it is here). Since 11a an
      argument that offers a choice offers a name a model sends beside a word a
      person reads, so an option cannot reach an approval sentence as
      untranslated English, and a sentence holding a word nobody has translated
      says the line is not translated · Owed: the daemon itself, the acting half
      of the application verbs — which is Wayland and D-Bus, and is the whole of
      what makes any of these move a window — and the context an agent is given
      when invoked
- [ ] Every execution recorded with its origin, approval and grant
      · Built: `alo-record` — the record, including refusals, which are written
      down in the same words the person was shown rather than in a second
      rendering of their own; since 9g that covers what *ran* as well as what
      did not, so the sentence in the record is the sentence somebody approved
      — and `alo-keeping`, which puts it on a disk so it outlives the session:
      one line per thing that happened, synced as it happens, a retention rule
      that cannot be set to keep nothing, and a shortened record that says so
      permanently in the first line so an absence is never read as an innocence
      · Owed: queue 4b — the path it is written to and the timer that shortens
      it, both `alo-agentd`'s, and the daemon does not exist
- [ ] **Compositor**: Wayland via Smithay, one display, keyboard and pointer
- [ ] **Sign-in**: alo identity, and a local account that needs no tenant
- [ ] **The agent overlay**: one key, from anywhere
- [ ] Launcher and window management: move, resize, snap, tile
- [ ] **The dock on any edge** — bottom, left, right or top, the person's choice,
      built for both orientations rather than one rotated
      · Built: nothing — the commit that added this line added no code · Owed: all of it
- [ ] **AI can be declined entirely** — setup's fourth choice, and a system that
      is complete without it (ADR 0009)
      · Built: the decision (ADR 0009), not code · Owed: all of it
- [ ] Copy, cut and paste across applications; switching between windows
- [ ] Keyboard shortcuts a person can change
      · Built: `alo-shortcuts` — the shortcuts, rebindable, nothing quietly
      taking one away, and every row and key of the panel said in the language
      the person reads · Owed: a shell to press them in
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
- [ ] **Settings, as one place**: network, display, sound, printers, storage,
      keyboard, accounts, privacy, updates
- [ ] **Making it yours**: background from a file, folder or colour and per
      display; lock-screen image; light and dark; an accent from the five
      designed hues, terracotta reserved (ADR 0010); text scaling; wallpapers
      shipped in the image
      · Built: `alo-appearance` — background per display, light and dark, text
      scaling, and the accent set as working code: five hues, each value
      measured against the ground it is drawn on, terracotta unreachable rather
      than refused; and every word of it readable in the reader's own language
      rather than in English · Owed: the Settings panel, the wallpapers
      themselves, and the mark and word that must appear wherever the agent's
      colour does
- [ ] **The ordinary desktop**: notifications, status area, file manager, trash,
      archives, USB storage, file associations, a text editor, an image viewer,
      a terminal
- [ ] **Capture**: screenshots, annotation, screen recording with audio, screen
      sharing — and an indicator whenever screen, camera or microphone is in use
- [ ] ★ **Divide the screen**: halves and quarters by drag or keyboard, splits
      that hold while you work and are remembered, per display
- [ ] **Input**: drag and drop, context menus, gestures, virtual desktops,
      keyboard layouts with dead keys and a compose key, input methods
- [ ] **Software**: install sandboxed applications, update and remove them; the
      XDG portal backend against our own shell; one grant list covering agents
      and applications alike; secret storage; session management; corporate
      proxy support
- [ ] **Devices**: audio with mid-call switching, Bluetooth, camera, microphone,
      media playback, power management, night light
- [ ] **Language**: the shell in all 24 official EU languages, with regional
      formats, timezones and a keyboard layout offered alongside each; RTL-ready
      even though no official EU language needs it yet
      · Built: `alo-strings` — every sentence named, translation checked against
      what the system says, the 24 languages listed each in its own language,
      English unable to hide, and a sentence that counts something counted with
      its reader's own plural rules, read from CLDR rather than recalled; and
      six crates whose own English has moved onto it — `alo-files` (every file
      refusal, the six verbs, and the sentence a person approves before a file
      is renamed, moved or archived), `alo-shortcuts` (every row of the
      shortcuts panel, and every key named the way the reader's own keyboard
      prints it rather than the way an English one does), `alo-appearance`
      (the eleven colour names a person picks from, each with the note a
      translator needs where the word does not travel, and every refusal about a
      value they chose), `alo-capability` (every way the capability model
      says no — the grant that could not be made, the argument that did not
      survive the boundary, the change nobody was asked about, and the grants
      themselves, which now refuse with a value that is worded in the reader's
      language wherever it is shown or written down), `alo-models` (where an
      answer is about to come from, which is read before somebody decides
      whether to send a document at all; the rule an organisation set and why it
      refused; and every refusal about a provider, a key or the runtime) and
      `alo-egress` (the line a person reads while something is leaving their
      machine — the visible half of law 1 — the place it names, and every
      refusal the egress policy makes)
      — and, since 9g, **the sentence a person approves is one string rather
      than two renderings of it**: a verb is declared from the words a
      translator is handed, a call carries what names its sentence and the
      values that fill it, and the screen, the approval and the record all ask
      the reader's own vocabulary for the words. Since 11a that holds for what
      goes *into* the sentence as well: an option a verb offers is a word
      somebody translates rather than the identifier a model sent, and **a
      sentence is only as translated as its least translated piece**, so a
      finished sentence with an unfinished word in it cannot pass for a
      translated line
      · Owed: a shell to translate, and every translation — there are none yet
- [ ] ★ The agent answers in the language it was asked in
- [ ] **Access**: screen reader, magnifier, high contrast, keyboard-only
      operation of everything
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
- [ ] ★ **"Where is that file?"** — local retrieval over granted paths, nothing uploaded
- [ ] ★ **"Why is it slow?"** and **"what is filling my disk?"**
- [ ] ★ **Printers, solved** — found, set up, and fixed when they stop
- [ ] ★ **"I can't open this file"** — converted, or plainly explained
- [ ] ★ **Undo what the agent did**
- [ ] Updates that never interrupt
- [ ] **Machines find each other** on a local network, with pairing
- [ ] **One GPU box serves the office** — shared local inference over a pairing
- [ ] A self-hosted workspace on the network is discovered, not configured
- [ ] **Zero inference egress over a working day**, measured and published

**Exit gate.** A person works a full day on alo OS — mail, documents, a video
call, printing something, driving one installed application through its agent —
and does not need another machine. An update lands and rolls back cleanly. And
the four questions everybody asks a computer are answerable out loud: where is
that file, why is it slow, what is filling my disk, and undo what the agent just
did.

---

## v1 — an organisation can buy it

Not only a fifty-seat firm. v1 is the release a security team, a compliance
officer and a procurement department can all say yes to, which is a much wider
bar and is where most of this list comes from. **ADR 0004** settles what a
managed machine means and what its person is told.

- [ ] **Sign in with the organisation's identity provider** (SAML/OIDC)
- [ ] **Smartcard and national eID sign-in**
- [ ] **Disk-encryption key escrow**; remote lock and wipe
- [ ] **Records export to their SIEM**
- [ ] **Update rings**, and an update mirror they host
- [ ] **Private model catalogue** and **signed adapter allowlist**
- [ ] **Agent policy by role**; agent retention policy
- [ ] **Inference accounting**
- [ ] **Egress attestation** — signed, printable, per period
- [ ] ★ **"Make this machine like my old one"** — configuration as a document, for a person
- [ ] A new colleague working on day one
- [ ] Configuration as a document; helpdesk assistance as a session a person ends
- [ ] Certification groundwork: ISO 27001, BSI Grundschutz, ANSSI
- [ ] **Published EN 301 549 accessibility conformance report**
- [ ] Remaining portals: USB, application-registered shortcuts, remote desktop;
      location services off by default; applications contribute to search
- [ ] Fleet enrollment by discovery — the machine asks, an administrator admits it
- [ ] Fleet policy and signed updates, for alo OS machines
- [ ] Files and printers shared between paired machines
- [ ] Cross-machine agent work, under grants made on the target machine
- [ ] Signed images verified before boot; Secure Boot with our key
- [ ] Backup and restore
- [ ] **Adapter SDK published**, with a conformance suite
- [ ] Multi-user on one machine, with per-person grants
- [ ] ★ Ask for appearance changes — "use dark after six" — under the same
      propose-then-approve as anything else; themes as a document for a fleet
- [ ] **Third-party security audit** of `alo-agentd` and the broker, published
- [ ] Compatibility list, grown outward from the certified machine
- [ ] Support and SLA definitions

**Exit gate.** An organisation enrols a fleet against its own identity
provider, sets policy by role, receives a staged signed update, recovers a
machine whose owner has left, sees agent actions in its own SIEM, and hands its
auditor an egress attestation — with a published third-party audit and somebody
to call. And a person signing in to one of those machines can say, in ten
seconds, who else has power over it.

---

## Not scheduled

The engine (`alo-engine`) replaces the workspace's rendering incrementally and
has its own roadmap. alo OS Desktop — the non-GPU SKU for the Windows 10 fleet —
follows once the AI SKU has customers, and inherits everything built here.
