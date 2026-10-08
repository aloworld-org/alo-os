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
- [v0.5] Multi-user on one machine, with per-person grants and no shared agent memory

**Making it yours**

The first thing anybody does with a new machine is make it look like theirs. It
is not a small feature: it is the moment a person decides whether the system is
theirs or the company's, and an operating system that cannot do it feels
unfinished however good the rest is.

**What they change is the surface they work on, not a picture behind it.** alo OS
ships no wallpaper: the canvas plane's own surface *is* the desktop, so making a
machine yours is done to that surface. [ADR
0075](decisions/0075-alo-os-has-no-wallpaper-the-canvass-own-surface-is-the-desktop.md),
accepted 2026-09-28, decides this and says why both places a picture could go are
worse than none.

- [v0.5] **Set the surface's material** — its colour and how it is finished; per display on a multi-monitor desk. *Read from a file or a rotating folder: **deleted rather than completed**, ADR 0075*
- [v0.5] The lock screen shows the same surface and no client's pixels. *An independent lock-screen image: **deleted rather than completed**, ADR 0075 — there is no longer a picture for it to be independent of*
- [v0.5] **Light and dark**, following the time of day if a person wants
- [v0.5] **Accent colour** — four designed hues, each with a value for a light ground and one for a dark, so it reads properly either way. The whole shell follows it, not one button (ADR 0010, as amended by [ADR 0067](decisions/0067-the-agents-colour-is-deep-teal-and-the-colour-it-vacates-is-given-back.md)). *This line said **five** until 2026-09-30. ADR 0010 designed five around a reserved terracotta; ADR 0067 reserved deep teal instead, released terracotta, and — amended 2026-09-27, after the owner was offered a deeper terracotta for light grounds and declined to invent one — settled that **four ship and no slot waits for a fifth**. An accent has to reach 4.5:1 on both grounds and terracotta on cream measures 2.87:1, so four hues that all read is the set, not four fifths of one*
- [v0.5] ★ **Deep teal is not one of them.** It means the agent and nothing else, so it is reserved rather than offered — an accent somebody could set to deep teal would take away the one signal that says the machine is acting on their behalf
- [v0.5] ★ **The agent is never signalled by colour alone** — deep teal always arrives with a mark and a word. A signal carried by hue fails for anybody who cannot distinguish that hue, and EN 301 549 does not allow colour to be the only means of conveying anything
- [v0.5] Text size and scaling, which is an accessibility setting as much as a taste one
- [v0.5] **A fresh machine already looks composed** — the plane ships with a surface of its own, so nobody meets a grey rectangle. *Wallpapers shipped in the image: **deleted rather than completed**, ADR 0075*
- [v1] Cursor size and colour; sounds, including silencing them
- [v0.5] ★ **Ask for it** — "make the surface warmer", "use dark after six" — the same propose-then-approve as any other change, because personalisation is exactly the low-stakes place people first learn to trust the agent
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
- [v0.01] **Full screen** — one window covers the whole display, the Dock and the panel of put-aside windows with it, and gives the screen back when the person leaves it. An application may ask for it and is answered; a person may put a window into it without the application's help. *Added 2026-09-30 by the owner's direction, into the current release. It had never been listed at all: the line above promises **maximise**, which stops above the Dock and leaves it visible, and a client asking to fill the screen was met with silence rather than a refusal — measured the same day, which is what surfaced the omission. The tier is set here before any of it is built, because `CLAUDE.md` binds building to this file*
- [v0.01] **The alo Dock** — a band along an edge of the screen, above the canvas. It shows what you can open and brings what is already open into focus, and it does not move when the canvas does. Labels give way to icons where the short edge demands it ([ADR 0076](decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md))
- [v0.01] **Where the Dock goes** — **the bottom edge by default, and the person may choose bottom, left, right or top.** It works in both orientations rather than being a horizontal bar someone turned sideways. *Restored here on 2026-10-01 to match the owner's reversal of 2026-09-30, recorded in [ADR 0076](decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md): bottom is now the default rather than the only. It had been withdrawn below, and the withdrawal outlived the decision that made it — this file said the promise was gone while the record it cited said the promise was owed. Nothing of it is built: `alo-dock` has no edge field at all, which the ADR's remedy removed. The tier is set here before any of it is built, because `CLAUDE.md` binds building to this file*
- [v0.01] **Reaching the Dock over a full-screen window** — moving the pointer to the screen's edge reveals the covered surface over the content, and **the pointer can travel onto it and click without it disappearing on the way.** The keyboard reaches it too, and dismissing it from there puts focus back in the work rather than nowhere. **No reveal or hide delays:** a timer makes this depend on how fast somebody can move, which is the one thing a person with a tremor or a trackball cannot control. The same reveal serves the panel of put-aside windows at its own edge — one behaviour, one instance per surface — and it is the same behaviour at whichever edge the person put the surface on. *The clause that stood here until 2026-10-01 — `ADR 0076` is untouched by it: an edge is a fact about a surface, never a setting offered to a person — was true of that record as written and was reversed by the owner on 2026-09-30, the day after. The reveal never depended on it: it reveals a covered surface at that surface's own edge, and which edge that is was the only part the reversal changed.* *Added 2026-09-30 by the owner's direction, into the current release, together with the full-screen promise it stands on. It had lived only in [the Dock's design note](design/the-alo-dock.md) and a plan; the rules for it are built and tested in `alo-dock` with nothing yet calling them*
- [v0.01] **The top controls** — a band along the top of the screen holding the
  **active window's controls** and the way **back to the canvas**. Like the Dock
  it is above the canvas and does not move when the canvas does. **It gives way
  to a full-screen window** and is reached the same way the Dock is: by pointer
  or by keyboard, and **it stays while it is being used** rather than vanishing
  under the hand that reached for it. **A person who would rather it never
  hid may keep it visible.** *Added 2026-10-02 by the owner's decision. It had
  been drawn in [the regions note](design/the-regions-a-pointer-can-be-in.md)
  and listed here nowhere, which is the gap that mattered: `CLAUDE.md` binds
  building to this file, so three tasks arbitrated against a surface no lane was
  permitted to build and the pointer classifier could not land in any crate. The
  tier is set here before any of it is built, for the same reason. **The
  top-right corner is the panel's**, by the owner's ruling in that note — the
  top controls' region stops before it, and two surfaces claiming one corner was
  the fault that ruling settled.*
- [v0.5] The dock's size, and whether it hides when a window needs the room

**Restored, not withdrawn.** *The person decides where it goes — bottom, left, right or top, chosen in Settings* was withdrawn by [ADR 0076](decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md) on 2026-09-29 and **the owner reversed that within a day**. It is a promise again, at **[v0.01]**, in the list above. Its **[v0.5]** descendant — *per display, so the dock can sit along the bottom of the laptop and down the side of the external screen* — keeps the tier it had; the reversal moved the edge choice and left per-display placement where it was.

*This paragraph said `withdrawn` for two days after the decision it cited had been reversed. A withdrawal is the one kind of entry that has to be re-read when a record changes, because every other kind describes work that is owed and this kind describes work nobody is allowed to do — so it is the entry a reader trusts to mean stop, and the only one whose staleness silently forbids something the owner has asked for.*
- [v0.5] ★ **Divide the screen** — drag a window to an edge to take half, a corner to take a quarter, or split what is already open with the keyboard. The split holds while you work: resizing one side resizes its neighbour rather than overlapping it
- [v0.5] Remember a split, so returning to a pair of windows restores the arrangement rather than the last position of each
- [v0.5] Splitting works on an external display independently of the laptop's own

**Screens and desks**

*Added 2026-10-08 by the owner's direction, into the current release:* **"I want
all the discussed features of the monitors to be in the next release after this
one so you tasks today is to do all the features we have agreed on"**, following
**"users can name the monitors by any name like 'cat'"**. *The tier is set here
before one line of any of it is written, because `CLAUDE.md` binds building to
this file — the same way **Full screen** was added on 2026-09-30 and
**Notifications** was moved into this release on 2026-10-02. The design is
settled in [ADR 0099](decisions/0099-a-screen-carries-the-name-its-person-gave-it.md)
and the work is ordered in
[`docs/autonomy/more-than-one-display-plan.md`](autonomy/more-than-one-display-plan.md).*

*What makes these **scope** rather than sub-work, in one sentence: they add
promises to a person. Somebody who can call a monitor `cat` has been offered
something nobody was offered before, so not one line of it is written before this
file carries it with a tier.*

- [v0.01] ★ **A desk comes back when you arrive at it** — the screens in front of you, how they are arranged, how large things are on each, and where your windows were. Plug in at one desk and that desk returns; move to another and that one does. Nothing to choose, and nothing to set up twice. *The engine is built and tested in `alo-displays` — a separate arrangement per set of screens, with `three_sets_of_screens_remembered_apart` proving it — and has never had a caller. This line is what lets it reach a machine*
- [v0.01] ★ **You name your screens, and the name is yours** — any name you like, typed on the screen it belongs to. `cat` is a valid name for a monitor. Two screens may carry the same name, because forbidding that would take a choice away for the machine's convenience. **A name is a label, never a key:** the machine tells screens apart by what they report and which socket they are in, and shows the person's word. A person's own name is never translated, as a screen's make and model are not (ADR 0099)
- [v0.01] **Screens are named, not numbered, by default** — *the laptop*, *the big one on the right*. A number is the one thing nobody can match to the screen in front of them, which is the step people actually fail at
- [v0.01] **alo OS says which desk it thinks you are at, once** — and says nothing on the ordinary morning. *The sentences are written and translated already in `alo-displays`: your screens are arranged the way you last left them; this screen has not been used with this machine before; your screens have changed since you arranged them and your arrangement is still here; the arrangement you made at your other desk is still here for when you are back at it. `notes.rs` holds the rule that there is no note for the ordinary morning, because a system that announces every screen every time is one whose announcements nobody reads*
- [v0.01] ★ **Show me where** — when a screen is new, alo puts it beside the others and says so. If that guess is wrong, a card on the new screen is pushed off the edge towards the screen it actually sits beside, and **that gesture is the arrangement**. No map of grey rectangles to drag. Doing nothing keeps the guess and leaves the card offered where the notes are, so somebody who only wants to work is never stopped and never asked twice
- [v0.01] **Unplug and nothing is lost; plug back in and nothing moved** — the windows on a screen that goes come home visibly, with how many and from where, and an undo. Plugging it back puts them where they were
- [v0.01] **How large things are, per screen, in words** — *smaller*, *just right*, *bigger*, previewed on the screen being changed, with no percentages. **It cannot be offered until a display's size reaches an application's window and not only alo's own dock and panel** — measured 2026-10-08, and offering it before then would be a control that lies
- [v0.5] **Two windows onto one desk** — point both screens at the same part of your desk, or at different parts, and either is ordinary. *The architecture is already this: `more-than-one-display-plan.md` task 7 gives each display its own camera onto one canvas. The gesture is the new promise, and it stays at `[v0.5]` so it cannot delay the lines above*
- [v1] **Ask alo to use a screen by its name** — *put the chat on cat*. Kept at `[v1]` because an agent verb is a public contract surface, typed and enumerated under ADR 0001, and a contract is not changed to make a feature land sooner
- [v0.01] Switching between windows, and between applications
- [v0.5] Drag and drop between applications
- [v0.5] Right-click context menus, wherever a person expects one
- [v0.5] Touchpad gestures: scroll, zoom, swipe between workspaces
- [v0.5] **Keyboard layouts, switched easily** — and dead keys and a compose key that work. "Müller" and "Liège" are test cases in a European product, not edge cases
- [v0.5] Input methods for non-Latin scripts
- [v0.5] Virtual desktops
- [v1] Clipboard history, on the machine and never synced to any server. Sharing one copied item directly between a person's own paired devices is an opt-in: end to end encrypted, gone after about two minutes, never readable by the agent (ADR 0064)

**Capture**

- [v0.5] Screenshots: whole screen, one window, a selected region — to a file or the clipboard
- [v0.5] Annotate a screenshot without opening anything else
- [v2] **Screen recording**, with audio, to a file
- [v0.5] Screen sharing for calls
- [v0.5] ★ **A visible indicator whenever the screen, camera or microphone is in use** — by any application, including ours. Law 1 is about egress; this is the same instinct applied to the room you are sitting in

**Desktop**

- [v0.01] Notifications, with do-not-disturb *This was `[v0.5]` until 2026-10-02, when the owner moved it into the current release so a promise already at `[v0.01]` could be kept. The canvas asks that a recovery which moves a frame **shows** the move — *a frame that relocated itself silently is a person's arrangement edited without them* — and nothing else on this machine can tell a person anything, so the clause was unkeepable while this line sat higher. The tier moved rather than the scope gate being crossed: `CLAUDE.md` binds building to what this file says, so the file is what changed. The do-not-disturb half is already built — `alo-notifying`'s `quiet.rs` holds four reasons, two the person's and two the machine's, with the screen winning first so nothing is shown into a recording.*
- [v0.5] Status area: clock, battery, network, volume, brightness. **It is owed a location.** [ADR 0076](decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md) took *at the far end of the dock, wherever the dock is* off this promise — a clock is not something a person opens or brings into focus, so it is not the Dock's — and handed *where does it go* to [the shell's plan](autonomy/the-smallest-canvas-worth-showing.md). The promise stands; what it is owed first is a place
- [v0.5] ★ The egress indicator sits at the far end of the dock, above it, so "nothing has left this machine" sits where a person already glances rather than somewhere they must learn to look. **It did not move with the rest of the status area:** it has a corner of its own, and drawing it is the one part of this that is built
- [v0.5] A file manager, with trash, and archives that open
- [v0.5] USB drives and external storage that appear when plugged in
- [v0.5] File associations — what opens what, changeable by a person
- [v0.5] A text editor and an image viewer, so a fresh machine is not helpless
- [v0.5] **A terminal.** Law 2 forbids the *agent* running arbitrary commands; it says nothing about a person, and an operating system that does not trust its owner with a shell is a toy
- [v0.5] **Search your own files, without asking anything** — by name, kind, date and contents, in the file manager, indexed on the machine. The agent's *“where is that file?”* is a nicer way to reach this; **it is not the only way**, and a machine whose only search is a conversation is a machine somebody locked out of their own documents
- [v0.5] **What is running, and what it is using** — processes, memory, disk and network in a window. The plain answer to *“why is it slow?”*, for the person who cannot or will not ask
- [v0.5] **What is filling the disk** — shown as sizes you can open up and click through, not a number in Settings

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
- [v1] **Corporate proxy support**, machine-wide and honoured by applications. A great many company networks have no other route out
- [v0.5] Portals: USB devices, global shortcuts an application registers, dynamic launchers, remote desktop
- [v1.1] **Location services**, off by default, per-application, with an indicator when in use
- [v1] Applications contribute to search — one place to look, not one per program
- [v1] Realtime scheduling for audio work, which is what a workstation is often bought for
- [v0.5] Unsandboxed installation as a deliberate, clearly-marked act — never the default, and forbiddable by policy on a managed machine. Brought forward from v1 by ADR 0064: developers need their own tools from the first day

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
- [v1] **Right-to-left ready**, so adding a language later is translation rather than rework, even though no official EU language needs it today
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
- [v0.5] ★ **The grant is a boundary the kernel imposes, not a rule the daemon follows** (ADR 0013). For the length of one turn, everything outside the grant is *unreachable* — Landlock for the files, seccomp for the syscalls, an eBPF programme on the turn's own cgroup for the sockets. A verb that overreached would fail at the syscall, not be talked out of it
- [v0.5] ★ **And the kernel is taught what a turn is** (ADR 0015) — our own programs on the kernel's security hooks, the same ones SELinux and AppArmor use, carrying a grant the daemon wrote into shared memory when the turn began. **No kernel is patched and no fork is kept**: they are loaded at boot and verified by the kernel before they run
- [v0.5] **So the record stops being anybody's account of themselves.** What a turn touched is what the kernel watched it touch — the difference between an audit log and a guarantee
- [v0.5] ★ **And it forgets everything that was not an agent.** A syscall outside a turn is checked and leaves no trace: no log line, no counter, no timestamp. Your editor, your browser and your terminal pass through and are forgotten. **The mechanism that could watch everything is the one place this promise is proved by a test rather than stated**
- [v0.5] ★ **So the record stops being a claim and becomes an observation.** Today it is `alo-agentd`'s honest account of itself, which is an audit log; with the boundary in place, what a turn touched is what the kernel watched it touch. *What did the agent do* is answered by the machine rather than by the program being asked about
- [v0.5] A turn whose boundary cannot be applied **does not run** — a refusal, not a warning, the same rule `alo-egress` already follows when a policy cannot be evaluated
- [v0.5] ★ **System verbs** through the privileged broker: printers, network, updates, storage
- [v1] ★ **Application adapters** — installed applications become agents with typed verbs (`@blender`, `@resolve`, `@gimp`); see `docs/contracts/app-adapters.md`
- [v0.5] The accessibility fallback: any application with no adapter is still readable and operable through its AT-SPI tree
- [v1] ★ Adapter SDK published, with a conformance suite third parties can run
- [v1] Policy: which verbs and adapters are permitted, set per machine or per fleet
- [v1] Screenshot-and-click, marked in the record and disabled by policy by default — last resort only, never the default mechanism

## The AI stack — model choice and deployment configurations

**Four choices at first start, in this order** (owner, 2026-10-06, superseding
the clarification of 2026-09-08 kept below):

1. **On this computer** — the person's selected model and runtime on their PC,
   including models such as Llama where the integration supports them.
   *alo works locally. Availability depends on this PC.*
2. **A machine on your network** — a machine this one has been paired with,
   ADR 0003's one box serving an office.
   *Selected content goes to a machine on your network.*
3. **My provider** — a compatible API selected by the person, such as OpenAI or
   Mistral, using the credentials that provider requires.
   *Use your provider. Selected content leaves this PC; provider charges may
   apply.*
4. **No AI** — no model, no provider and no agent (ADR 0009).
   *Open apps, find files and use the full canvas by hand.*

**All four carry equal size, typography and weight, and none is pre-selected**
(ADR 0014 §4, ADR 0025). **No AI is a complete answer** — not a skip, not a
fallback, and not an invitation to persuade the person later.

**alo's own service is not one of the four.** It appears only after *My
provider* is chosen, as an ordinary provider beside the others: no badge, no
special styling, no automatic selection, no commercially privileged ordering
(ADR 0014). Provider names and ordering shown in the design file are
illustrative and do not establish an integration-support matrix.

**Sending to a machine on your network is still data leaving this PC**, and uses
the same disclosure and recording as any other destination. A local destination
is not by itself a promise that nothing leaves the building; that claim needs an
enforced local-only processing policy, which is a separate thing from where the
machine sits.

These are the primary user-facing choices, not four different privacy policies.
Do not replace them with "prefer local processing" and "keep questions on this
PC", or present those as competing setup choices. Any advanced privacy controls
belong separately in settings. This establishes the model-selection direction,
not acceptance of proposed ADR 0021 or new `ThisMachineOnly` semantics.

Existing release tiers still apply: this is not a claim that alo's hosted
service, paired-machine processing or every listed integration is available in
the current release.

### How a first-start screen is built (owner, 2026-10-07)

Settled against the design file and verified against the refreshed snapshot;
`docs/design/the-first-start.md` carries the measurements and the frame ids.

- [v0.01] ★ **The strip above every setup screen reads `alo OS`.**
  It is identity, not state: no step count, no stage name, and no record of
  which step a person came from. **The duplicate wordmark elsewhere on those
  screens is removed.** 69 of the 77 frames carry it.
- [v0.01] **The running canvas has no strip and must not gain one.**
  A frame has the strip when it is a setup sheet; a frame showing the canvas,
  the Dock and a place name has none. Stamping a setup label on the canvas
  would say setup is still running after it has finished.
- [v0.01] ★ **Setup shows no overall progress counter.**
  No *N of M*, no row of dots, no decorative progress line. The screen's
  heading says what the current task is. Branches walk different numbers of
  screens, so there is no honest denominator — and a build must not derive one
  from the frame inventory.
- [v0.01] **Real installation progress stays beside its own operation.**
  Download, preparation and installation progress, and the error and recovery
  screens around them, are not the counter and are not removed with it.
  *Removing a decoration must not remove the one thing a person waiting on a
  disk write actually needs.*
- [v0.01] ★ **`Optional` sits immediately above the heading, on ten screens.**
  Written as `Optional · <subject>`: the three access choices, file import and
  its two follow-ons, sign-in options, PIN, fingerprint and account recovery.
- [v0.01] **Each optional screen keeps its own way to decline.**
  They are not interchangeable — `Not now`, `Cancel`, `Back`, `Use password
  only`, `Use password for now`, `Done`. A build must not normalise them into
  one.
- [v0.01] ★ **One selection border: 2 logical pixels of navy, plus the word.**
  `#102A43`, with `Selected` beside it, and the former 1.5px access-screen
  border was a leftover. **Keyboard focus stays a separate mark** — an outer
  ring with its own gap — and moving focus never commits a choice. **Teal
  never means selected or focused**; it stays reserved for alo acting
  (ADR 0067). Display scaling is applied once, at the existing conversion
  boundary.
- [v1] **`Not now` leaves an access screen without changing permissions.**
  It must not commit the alternative the screen happens to be displaying, must
  not raise or lower access, and is not the same action as `Back`. Tiered at v1
  because the three levels it has to preserve are the v1 item *running code, at
  the level the person picks*; the design exists now and the rule is recorded
  now so that whoever builds the policy inherits it.

**What this supersedes, kept because the change is the point.** The 2026-09-08
clarification named **three** main model-selection choices — *local models, your
own API provider, and Alo* — with alo's own service as one of the three, and
added that the deployment configurations were *not four competing model-source
buttons*, that *the existing no-agent path remains a separate opt-out*, and that
*paired-machine support is preserved without adding a fourth primary source
category; its detailed placement remains to be designed*.

Every one of those is now decided the other way: alo's service sits inside *My
provider*, the no-agent path **is** the fourth choice rather than an opt-out
beside them, and the paired machine **is** a primary choice and is placed,
second. `crates/alo-setting-up`'s `THE_FOUR` already held this shape and this
file had not caught up — which is how `docs/design/figma-brief.md` came to brief
a designer one choice short, and how the design file came to carry a fourth box
that was not ours.

**alo OS works well with a model on this machine, with a model on a machine on
your network, with a provider you added — and with no model at all.** Those are
four supported configurations, not one real one and three compromises, and
**“works well” is a measured bar in each of them rather than a hope in three.**

**Model choice belongs to the person** (owner clarification, 2026-09-08).
Alo-provided models or services, the person's own local models and runtimes,
and compatible third-party APIs are legitimate choices. Alo ownership is never
a condition of being a valid choice. Model ownership and processing location
are separate: an alo-provided service may be remote, and a third-party model
may run entirely on this machine. Existing organisation policy, capability
validation and the release tiers below still apply; this clarification does not
bring later-release features forward or claim unimplemented integrations work.

Freedom to choose a model is distinct from a verified privacy guarantee. A
loopback address establishes where a service is contacted, not where it performs
inference. Any stronger local-only guarantee must be based on established
protection, not the model's brand. The treatment of unverified services under
`ThisMachineOnly` and the enforcement options in proposed ADR 0021 remain
unaccepted; this clarification neither changes that policy nor narrows the
egress promise. Paired-machine operation and the no-agent choice remain intact.

Each has its own bar, and they are not interchangeable:

- **On this machine** — CPU or GPU, and the model must actually drive the verbs,
  not merely produce sentences (ADR 0007, as corrected).
- **On a machine on your network** — a deliberate pairing made on both machines,
  and it is egress: the indicator fires, because *it only went down the
  corridor* is how a guarantee stops meaning anything (ADR 0003).
- **With a provider you added** — including alo's own, which says who and where
  in the same words as anyone else's and gets no exemption (ADR 0008, ADR 0014).
- **With none at all** — the whole machine still works, the agent's surfaces are
  absent rather than greyed out, and every capability an agent has is reachable
  by hand (ADR 0009). A person who never turns it on has a complete computer.

**Nothing moves between them on its own.** A local model that fails does not
become an API call, a provider that runs out does not become a local model, and
a machine with the agent off is never talked into turning it on.

- [v0.01] ★ **It runs on the machine you already own.** No graphics card required: the catalogue carries models that answer comfortably on an ordinary business laptop's CPU, and the system picks one (ADR 0007). This is what puts alo OS on the Windows 10 fleet rather than on a few hundred workstations
- [v0.01] ★ **It works well on a CPU and it works well on a GPU** (ADR 0007, as corrected). Neither is the other's fallback and neither is a “default”: the machine runs a model sized for what it has, and **“works well” is a measured bar in both cases rather than a hope in one of them**. A card buys a larger model and practical fine-tuning — not entry, and not the real version of the product
- [v0.01] ★ **The catalogue says whether a model can drive the verbs, not just whether it will run.** An agent turn asks a model to emit a typed verb call with valid arguments several times over, which is exactly what small models are worst at — sentences they manage, structure they lose. A model that runs beautifully on a laptop and cannot emit a valid call is useless as an agent, and a catalogue that only knew about memory would recommend it
- [v0.01] **And it is measured by us, not claimed by the publisher** — the same honesty already applied to how a model behaves on a CPU
- [v0.01] **A machine is only offered agent work it can actually do.** Where nothing catalogued clears the bar on a given machine, the honest answers are the ones already on offer — a paired machine, or a provider — shown as a choice and never substituted silently
- [v0.01] ★ **The GPU works on first boot**, where there is one — no driver installation, no CUDA archaeology. Acceleration, not an entry price
- [v0.01] ★ **A model runs in one command**, from a curated catalogue of open-weight models with their licences stated
- [v0.01] ★ **The release carries no model, and a person brings their own** — weights they already have, a provider they have an account with, or no model at all, and alo OS chooses none of the three for them. **Nothing is chosen on their behalf, and until somebody chooses, nothing answers** — which is the half of the older promise that survives. *This read **the local model is what the machine arrives ready to run** until 2026-10-06, when the owner decided the release ships no weights: a model we picked was a model picked for them, it was sized for the one certified laptop, and everybody downloaded 4.87 GiB including the people who would never run it. **Out of the box now answers nothing**, and saying so plainly is the point of this rewording; it returns later through our own API rather than through the image, and answering that way means inference leaving the machine — visible, recorded and never silent (Law 1)* (ADR 0095, ADR 0025, ADR 0016, ADR 0014)
- [v0.01] ★ **Add your own provider in Settings** — a name, an address, and a key: Mistral, your own endpoint, or whatever you already pay for. The key goes to the keyring, never into a settings file, so it cannot leak through a backup or a support bundle. You say where the provider runs; nothing is guessed from its address
- [v0.5] An address that is not https is refused rather than warned about, unless it is a service on this machine — "it is only our internal network" is how a key ends up on the wire in clear
- [v0.5] Test a provider before saving it, so a mistyped key is found now rather than in the middle of a question
- [v0.01] ★ **Or not at all** (ADR 0009). Setup's fourth choice, with the same weight as the other three: no model, no provider, no agent. Everything else in this document still works, the agent's surfaces are absent rather than greyed out, and turning it on later is a setting rather than a reinstall
- [v0.01] ★ **A person never learns the name of anything we rented.** They install an application — not a Flatpak. They run a model — not Ollama. The system updates — it does not *pull an image*. Every engine underneath is configured and never patched (`CLAUDE.md`), and the same rule runs all the way to the surface: **if somebody has to learn what a container is to use their own computer, the machinery has become their problem instead of ours**
- [v0.01] **And it is enforced rather than remembered.** No sentence in `alo-strings` — the one place every word a person reads is written — may contain the name of a rented component. A translator handed “the Flatpak could not be installed” has been handed our plumbing to render into twenty-four languages
- [v0.01] **Anything an agent verb can do, a person can do by hand** (ADR 0009). A standing rule rather than a feature, and the check on every verb anybody proposes: a machine without a working agent must lose *convenience* and never *capability*.
- [v0.01] ★ **And it holds however the agent became unavailable** — declined at setup, no model downloaded, offline, the provider down, the key expired, or **the money ran out**. Only the first of those is a choice. If a verb is the only way to do something, that thing is not finished: it has a feature that disappears when somebody's card is declined
- [v0.01] **Running out of credit is its own answer, not an error.** A provider saying *payment required* or *quota exceeded* is reported as what it is — not as a rejected key, which sends somebody to check a key that is correct, and not as a status code. The machine says it once, where it happened, and carries on. **It never spends money somewhere else instead**, which would be the worst reading of ADR 0008's *never a silent fallback*
- [v0.01] **And it never nags.** A machine that cannot reach a model does not follow somebody around asking them to buy credit — that is the greyed-out panel ADR 0009 already refused, in a different disguise
- [v0.01] ★ **Or use an API instead** (ADR 0008). A model may answer on this machine, on a machine on your network, or behind a provider's API — for a laptop too thin to run one, or an organisation that would rather buy inference than operate it
- [v0.01] ★ **Which model answers is the person's setting; the organisation's rule is a bound around it and never a choice inside it** (ADR 0016). Two files and two owners — the bound in the machine's description an administrator writes, the choice in the person's own settings under their home directory, where nothing else about the machine goes. **A choice the bound forbids is refused in words naming the rule, never quietly swapped for a permitted one**: a machine that answered from somewhere else while showing a person the choice they made would break the one promise ADR 0008 exists to keep
- [v1] **alo hosts models in the EU, and you can subscribe to them** (ADR 0014) — for the machine that cannot run one worth using, and for the organisation that would rather buy inference than operate it, from a company subject to European law
- [v1] **It starts as Mistral, resold, and says so.** No inference of our own at first — an alo account in front of Mistral's API, which is French, so the sovereign claim is true from day one rather than from the day we can afford graphics cards. The provenance line names the whole chain: *answered by alo, using Mistral, in France*. **An intermediary that will not say who it forwards to is the thing this product exists to replace**, and we do not get an exception for being the intermediary
- [v0.5] ★ **The same model, whichever place you run it.** Mistral's weights are open, so the model we host and the model on your laptop are the same model — start on a subscription because your machine is slow, buy a better machine later, and move to local inference with no change in the answers. **No other vendor can offer this**: leaving OpenAI or Anthropic always means accepting a different model, because theirs have no local equivalent. Where it runs becomes a question of hardware and money rather than of quality
- [v0.5] ★ **And our own service gets no exemption.** It is one more provider: the same egress indicator fires, the same *answered by alo, in Frankfurt* provenance line is shown, the same policy applies — **a machine set to keep questions in the building refuses ours too** — and the same words appear when the balance runs out. There is no variant for it in the code, no default, no pre-selection, and no quieter indicator because we know where it runs
- [v1] **Selling inference means we profit when questions leave the machine**, which runs against this product's own promise. It is written into ADR 0014 rather than left unsaid, and the defence is structural: our service is configured exactly as a third party is, so there is nowhere for a special case to live
- [v1] **Cancelling is not an expiry.** Signing in to alo OS does not sign anybody in to a paid model, and stopping paying leaves an operating system that works rather than a trial that ended
- [v0.01] ★ **Where the answer came from is said where the answer appears** — "on this machine", "on the studio workstation, on your network", "by alo, in the EU". Not in a settings page somebody would have to go looking for, because a person about to paste a contract into a question is entitled to know where it is going first. **And it never claims more than alo OS can check**: a service you run yourself at a loopback address is *by a service at this machine's address — alo cannot verify where it was processed*, because an address establishes what was contacted and never what did the work (ADR 0021)
- [v0.01] ★ **Never a silent fallback, in either direction.** A local model that fails does not quietly become an API call — failing to answer is recoverable, a person's records leaving the building because a download was corrupt is not. **And a provider that fails does not quietly become a local model**, which would answer somebody from a smaller model wearing the same face and remake a choice they had already made
- [v0.01] ★ **Neither is the other's fallback.** A model on this machine and a provider you added are both ordinary, complete ways to run alo OS — not a good option and a degraded one. Which is in use is a person's decision, never a substitution the system performs when the first disappoints
- [v0.5] A provider that will not say where it runs is reported as **unknown**, never assumed to be nearby — and unknown never satisfies a policy naming a region
- [v1] Policy over where inference may happen: anywhere, in the building, inside a region **the organisation names**, or this machine alone. We ship the mechanism, never a region of our own (ADR 0004, ADR 0008)
- [v0.01] Model lifecycle: pull, list, serve, unload, remove; disk accounted honestly
- [v0.01] ★ **Run a model we never catalogued.** Point alo OS at weights you already have and it runs them. *This was `v0.5` until 2026-10-06, when the owner decided the release carries no weights. **The tier moved rather than the scope gate being crossed**: with nothing shipped, this is how a person gets a local model at all, so a promise of bringing your own that waited for a later release would have been a promise with nothing behind it. The store the runtime serves from moved with it, from `/usr/share/alo/models` to `/var/lib/alo-model/models`, because `/usr` is the read-only half of a bootc machine and weights a person brings have to be able to land somewhere (ADR 0095)* **The catalogue recommends; it does not gate** — its job is stating licences and honest costs so somebody can choose well, never deciding what they may run on hardware they own. A machine where the only models are the ones we approved is a walled garden with a sovereign label on it
- [v0.5] **The machine warns and then gets out of the way.** A model too large for the memory in this laptop is *said so plainly, once* — and then run anyway if that is what somebody asked for. The honest costs the catalogue states are for deciding with, not for refusing with
- [v0.5] **What you bring is yours, including its licence.** We state the licence of everything we offer and gate our own catalogue on it. Weights somebody brings themselves come with their own terms and their own responsibility, and alo OS does not pretend to have checked them
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
- [v1] ★ **Printers, solved.** The agent finds it, sets it up, and fixes it when it stops. The most hated object in computing, and a small feature people tell other people about
- [v0.5] ★ **"I can't open this file."** A `.pages`, a `.heic`, a `.dwg`: the system converts it where it can, and where it cannot says plainly what will open it, instead of shrugging
- [v0.5] ★ **Undo what the agent did.** Every execution is already recorded with its origin and the image already rolls back — together they make *"undo everything the agent did this afternoon"* real. An agent you can reverse is an agent people let do more, and no other system offers it
- [v0.5] **Updates that never interrupt.** Atomic images mean an update can be genuinely invisible and instantly reversible. On the system people are leaving, this is the single most hated behaviour there is
- [v0.5] ★ **"Make this machine like my old one."** Configuration as a document, pointed at a person rather than an administrator: a replacement machine that is actually yours, not a week of rebuilding
- [v1] **A new colleague working on day one** — a managed machine that arrives with the right applications, policy and grants already in place

## The local network — machines that find each other (ADR 0003)

Discovery is open; **use requires a deliberate pairing on both machines**. Being
on the same WiFi confers nothing.

- [v0.5] Machines find each other with zero configuration — no addresses typed, no accounts
- [v1] ★ **One GPU box serves the office**: a machine without a GPU discovers the one with it, and the agents just work. The inference never leaves the building; it moves down the corridor. **It is still egress, and the indicator still fires** (ADR 0003) — *"it only went to the machine down the corridor"* is exactly the kind of exception that quietly ends a guarantee, so shared inference is shown like any other departure and the pairing is what makes it wanted rather than what makes it silent
- [v0.5] Pairing: mutual, deliberate, enumerated, revocable in one action, and expiring — grants, across a machine boundary
- [v0.5] ★ **The whole of it works with no internet at all.** An office that cannot connect still has working AI
- [v0.5] A self-hosted workspace on the network is **discovered, not configured** — no DNS step
- [v2] Files and printers shared between paired alo machines, with no server in the middle
- [v1] Enrollment by discovery: a new machine appears to the fleet and asks; an administrator admits it
- [v0.5] ★ Cross-machine agent work — an agent may **ask** a paired machine, and acts only under a grant made **on that machine, by its person**

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

- [v0.01] ★ **The egress indicator**: every network egress an agent causes, visible at the moment it happens — **except through a service the person themself put on this machine**, which is their own trust boundary and not one alo OS polices. A process they started can forward a question anywhere and no operating system can see past it; what alo OS does instead is refuse to pretend otherwise, so an answer from such a service says it cannot verify where the question was processed rather than claiming it stayed here (ADR 0021)
- [v0.01] ★ **No telemetry.** Not "anonymised telemetry". None — and the policy lives in a Rust service, not a checkbox
- [v0.5] ★ **A working day with the runtime alo OS ships produces zero inference egress**, measured at the network boundary — and we publish the test. **The runtime, specifically**: it opens no socket at all, so the claim is carried by the absence of one. A service somebody else runs opens a loopback socket that this measurement cannot see past, and is not what this line promises
- [v0.5] Full-disk encryption, enrolled at install
- [v1] Signed images, verified before a deployment becomes bootable; Secure Boot with our key
- [v1] Third-party security audit of `alo-agentd` and the broker, published

## The system and the image

- [v0.01] Boots on one certified machine, firmware to sign-in
- [v0.01] Image built as an OCI container image — a **bootable container** (`bootc`) on a rented, unmodified Linux base (ADR 0011), so the operating system *is* the image rather than being installed by one. No third language enters the repository to build it
- [v0.5] ★ Atomic updates with rollback — the previous deployment stays bootable, which a bootc image gives us rather than us building it (ADR 0011)
- [v0.5] ★ **Installed from the machine it replaces** — download one program on the Windows machine alo OS is replacing; it checks the machine, says exactly what will happen, stages a minimal boot environment, and pulls the operating system itself — signed and versioned — from the same registry updates come from. Download, click, reboot, sign in: no USB stick, no ISO burning, no firmware ceremony. Secure Boot is respected through the signed shim, never something a person is told to switch off; anything destructive takes a typed consent naming what is destroyed, and Windows stays bootable until the one named point of no return (ADR 0023)
- [v1] Printing. Unglamorous, and it decides public-sector deals. `alo-printing` is built and gated against a real CUPS — found, set up, printed to, and told what is wrong when it stops — so what is owed is the **On the machine.** half: no paper has come out of anything, and law 3 is what ticks that box (ADR 0078)
- [v0.5] The documents people are actually sent open: `.docx`, `.xlsx`, `.pptx`
- [v0.5] A web browser for the open web — a pinned upstream one, since our own engine is not scheduled
- [v0.5] Installer
- [v1] Fleet enrollment, policy and signed updates — **for alo OS machines only**
- [v1] Backup and restore
- [v1] A compatibility list, grown from the certified machine outward

---

## Designs worth learning from HarmonyOS

Huawei's HarmonyOS does several everyday things well. These are the ones that
fit a PC and fit alo, adopted on 2026-09-22 and not yet built. Each is alo's own
version, not a copy: it runs through the grants, the record and the person's
choice like everything else.

- [v1.1] ★ **Live capsules for work in progress** — a small live pill in the status area for anything under way, **the agent's work first of all**: *building your site, 3 of 5*, *waiting for your approval*. What the agent is doing, visible at a glance without opening anything
- [v1] ★ **A privacy centre with history** — one page answering *who used what, and when*: which applications **and the agent** used the camera, the microphone, files and the network. Built from the record alo OS already keeps, which most systems do not have to draw on
- [v1.1] **Cards from dock icons** — hover over or long-press an application's icon and a small card shows what matters in it and what can be done, without opening it
- [v1.1] **A collection shelf** — gather text, pictures and files from several applications into one shelf, then drop them where they are needed, including on another of the person's paired devices. The agent may fill it when asked, and only then
- [v1.1] **Snap devices together** — drag one paired device's icon onto another's to connect them. A drag is deliberate, and the other device still asks (ADR 0003)
- [v1] **One design language on every screen** — when alo's phone applications come, the same design and the same behaviour on the phone as on the PC, adapting to the size of the screen rather than being redrawn for it
- [v1.1] **An alo typeface** covering all 24 official EU languages, drawn once for every screen
- [v1.1] **Motion that feels physical** — animations that move like real objects and never stutter, within the frame budget the shell already holds itself to

## The interface (ADR 0065)

The v1 interface, decided on 2026-09-22 and not yet built. The v0.5 shell is
its foundation and stays as the familiar option a person can keep (Law 5).

- [v1] ★ **A recording is a document once its words are text** — a person asks
what is in a video or a call and alo turns its speech into text **on the
machine**, then answers from it exactly as it answers from any document. It says
which road the answer came from: **from the words, not the picture**, so an
answer about a slide nobody read aloud is never given confidently. Frames are
sampled and looked at only where the words are not enough, and fewer of them on
a machine with less to spare. What is not promised is understanding *motion* —
what happened after he stood up — because that is not a claim this hardware can
keep, and a promise nobody can keep is worse than a gap
- [v1] ★ **Ask about a recording you never watched, and nothing is read until you
ask** — the question worth answering is *which recording was the one about the
Belgian contract*, across files nobody has opened. Two roads, and the person
picks: **on demand**, where alo transcribes that file when asked, says how long
it will take rather than spinning, and offers to keep the transcript beside it so
the second question is instant; or **a folder the person chose**, worked through
with what it did in the record. **Neither is a background index of somebody's
disk** — context is captured on invocation and nothing reads a person's things
while they are not looking. Transcripts are tiny where video is enormous, which
is what makes the second question and the search across files nearly free
- [v1] **Open it at the moment it was said** — *take me to where she talks about
the deadline* finds the words in the transcript and moves the playhead there.
The **manual path is the same feature without an agent**: the transcript sits
beside the recording, and clicking a line moves the picture to it. Asking **why**
she said it is the model reasoning rather than searching, so it **quotes the
passage and the moment it reasoned from** and the person checks it in one click
- [v1] ★ **The words that came with the recording are read first** — a great
many videos already carry a subtitle or caption track, and a subtitle track is a
transcript with timestamps that somebody else already made. alo reads those
**before it offers to make its own**, which means *open it where she says it* and
reading along both work **with no model at all**: instantly, with nothing to
wait for, and **in No AI**, where there is no model to ask. A person who declined
AI entirely does not thereby lose the ability to search their own recordings.
Where a track exists and a transcription would be better, the person is told
both are possible rather than one being chosen for them; where the track is in
another language it is still a set of moments, and saying so is better than
ignoring it
- [v1] **What the pipeline is asked for before it is built** — three things this
plan needs from whatever decodes a recording, cheap to require now and expensive
to retrofit: seeking that lands on **the moment asked for** rather than the
nearest keyframe; a transcript carrying **timestamps** and not only text; and
**one position control**, which the person's scrubber and alo both drive; plus
**the subtitle and caption tracks a file already carries**, surfaced as text with
their timings rather than only burned into the picture.
[ADR 0009](decisions/0009-a-good-computer-without-the-agent.md)'s
rule is the reason for the third: a second, private way to move the playhead
drifts from the first, and then alo can do something the person cannot
- [v0.01] ★ **Every goal is a canvas** — a Place is an endless surface, and objects **and applications alike open as panels on it**, where the person put them. Arranging is placing, and **nothing is stacked** — no window is buried behind another where the person cannot find it. This line used to add *so nothing is minimised*; the owner's direction of 2026-09-26 withdrew that, and the promise three lines below has recorded the withdrawal ever since while this one went on asserting it. Putting a window aside deliberately is not stacking, and a person asked for both. **Zoom** and **pan** move through it — by pinch, by wheel with a modifier, and by key — and the canvas replaces window switching, tiling and virtual desktops *This was `v1` until 2026-09-30, when the owner put the full canvas experience into the current release. The tier moved rather than the scope gate being crossed: `CLAUDE.md` binds building to what this file says, so the file is what changed*
- [v0.01] ★ **The World, and moving between Places** — zoom out past a Place and every Place the person has is seen at once; zoom into one and it fills the screen. Moving between Places is the same gesture as moving across one, so there is no second way to navigate to learn. **This promise is the World as a level — seeing every Place and navigating between them.** Carrying a frame across is the promise five lines below, which owns it whole; this line used to end by promising that too, and two lines promising one capability is the fault this file keeps finding. *This promise did not exist in this file before 2026-09-30. `docs/decisions/0065` has defined the World since it was written — `World → Place → Object`, and *every Place the person has, seen at once* — and no line here ever carried it, so a decision record described a level of the interface that the only list of what gets built had never heard of. Found while moving the canvas into v0.01 because the owner named World navigation as required for completion and there was nothing to move*
- [v0.01] ★ **Frames, dragged and resized like a design canvas** — a frame shows nothing but its content while the person works, and its name and few controls appear when they point at it, select it or zoom out — the name is also what it is dragged by, so a click inside always belongs to the application. Handles resize it and the application is told its size as it happens, dragging moves it, several can be taken at once, guides and snapping line them up, and each frame carries a name shown when the canvas is far out. Fit the Place to the screen, fill the screen with what is selected, double-click to work inside — each with a keyboard form *This was `v1` until 2026-09-30, when the owner put the full canvas experience into the current release. The tier moved rather than the scope gate being crossed: `CLAUDE.md` binds building to what this file says, so the file is what changed*
- [v0.01] ★ **A frame arrives the shape its work is** — a messaging application opens as a narrow column of conversations, a spreadsheet wide, a video sixteen by nine — declared by the application and remembered per Place once the person changes it *This was `v1` until 2026-09-30, when the owner put the full canvas experience into the current release. The tier moved rather than the scope gate being crossed: `CLAUDE.md` binds building to what this file says, so the file is what changed*
- [v0.01] **Alo working in a window you put aside, and Stop** — a window handed to alo and
  put away still says what is happening to it: **the task it was given**, how far it has got,
  the last thing it did that you approved, and **Stop, always available and never withheld**.
  **Stop cancels the work and not the application** — it acts immediately with no *are you
  sure*, withdraws that run's authority, cancels its pending approvals and queued actions,
  and **leaves the application open, the window put aside, and completed changes kept**,
  because stopping and undoing are different acts. What the row then says is the **actual
  state** and never a hope: *stopping*, *stopped and changes kept*, or *stop requested with
  no confirmation* when the machine cannot confirm. An operation that cannot safely be
  interrupted **says so while cancellation completes** rather than claiming it stopped
  instantly. Stopping **does not revoke alo's access** — revoking is its own act — and
  starting again takes an explicit act by the person. For work spanning several windows,
  Stop targets **the task**, shows its scope before you press it, and updates every window it
  touched. *Added 2026-10-03 by the owner's decision, into the current release, together with
  the Stop design in task 7 of
  [putting a window aside](autonomy/putting-a-window-aside.md). It had been listed here
  nowhere: the nearest lines are* ★ **Give it to alo** *and* **The agent's presence**, *both
  `[v1]`, and both promise much more than this — a whole goal handed over under one capsule, a
  teal edge, a named cursor inside applications. **Those keep their tier.** This line names
  only the surface the ruling designed, so that building it does not quietly put the rest of
  v1 in the current release. **What it carves out of* ★ **Give it to alo** *is one phrase —*
  the person may step in, take over a piece or stop it *— and only as it applies to a window
  put aside. Handing a whole goal over, the capsule, and the plan shown first stay at*
  `[v1]`*, and the teal edge and the named cursor inside applications stay there too. So
  stopping now appears at two tiers on purpose rather than by drift, and the narrower line
  governs the put-aside surface.* `CLAUDE.md` binds building to this file, which is why the line is
  added before the drawing rather than after it*
- [v0.01] **The canvas is where they left it** — arrange windows across several Places,
  move their cameras, end the session, and a fresh start puts them back. **Place identity,
  frame identity and geometry, each window's presentation state and each Place's camera**
  all survive, and a window that was minimised, compacted or full screen comes back with its
  **normal geometry** intact rather than the shape it was last seen in. Frames stay reachable
  when displays or scaling change. *Added 2026-10-03 by the owner's decision, into the
  current release. It had never been listed here at all — the promise lived only in task 9 of
  [the smallest canvas worth showing](autonomy/the-smallest-canvas-worth-showing.md), which
  read `Done` while nothing wrote a file or read one back, so a person's layout did not in
  fact survive a restart. `CLAUDE.md` binds building to this file, which is why the line is
  added before the work rather than after it. **This is canvas layout persistence and not
  monitor arrangement persistence** — `alo-displays` keeps the physical screen layout and is
  a different thing with a confusingly similar name.*
- [v0.01] ★ **Compact, and minimised — two things, and the person picks** — *compacting* a frame, by dragging it small enough or with one key, leaves a **live tile on the canvas** that still shows what matters (the last messages, the track playing, the build at four of six) and can be acted on without growing back. *Minimising* puts a **preview in the panel at the edge of the screen**, which stays where it is while the canvas moves, and restoring returns the window to its own place on the canvas at the size it had. This line used to say *nothing is swallowed into a bar*, refusing the second outright; the owner's direction of 2026-09-26 keeps both, because a tile that stays visible and a thing put away for later are two different wishes and a person has both in a day. What the older sentence was protecting still holds: **compacting never becomes minimising by accident**, and nothing is put away without the person saying so. *This was `[v1]` until 2026-09-30, when the owner moved it into the current release so the panel could be built now. The tier moved rather than the scope gate being crossed: `CLAUDE.md` binds building to what this file says, so the file is what changed*
- [v0.01] **Tidy this canvas** — alignment and distribution for the person who wants them, and the same asked of alo as a proposal shown before anything moves *This was `v1` until 2026-09-30, and it moved because leaving it there had become incoherent rather than because anything new was decided. The frames promise four lines above moved to v0.01 the same day and already says **several can be taken at once, guides and snapping line them up**. So multi-select and snapping were put in the current release while alignment and distribution, the adjacent operation on the same selection, stayed at v1: **one operation at two tiers, made so by the lane that moved the other half.** The laptop lane separately reports the owner naming multi-select, drag preview and arrange-together as v0.01 — recorded as reaching this file through that lane rather than from the owner, so it can be corrected if arrange-together was narrower than either lane read. **The hard part is known and is not the alignment arithmetic:** `alo-put-aside`'s proposing module already shows a placement before anything moves; what does not exist is a proposal that carries a **set** rather than one frame*
- [v0.01] ★ **A frame can be dragged out of one Place and into another**, and the work goes with it — **through the World by pointer, or by keyboard with *Move to Place***, so neither road is the only road. **Restoring a minimised window is not this**: it returns to the Place it was already on, and the view travels there. Nothing is relocated by a restore. *This was `v1.1` until 2026-09-30, when the owner put the full canvas experience into the current release and ruled that cross-Place movement includes both navigation and window transfer. The two were briefly conflated while this was being written — cross-Place **restoration** and a frame **transferred** between Places read as one sentence at a glance and are two acts, and the owner settled that both are in v0.01 and that a restore relocates nothing*
- [v1] ★ **A frame simplifies as it shrinks** — application, then compact form, then its name and what it is doing, chosen by how large it is on screen. Zoomed out a person reads *three new from Anna* and *the build at four of six*, never a wall of unreadable miniatures
- [v1] ★ **Zones that mean something** — name a region *drafting*, *waiting on Anna*, *done*, and dragging a frame into it does what the name says; a zone can be handed to alo whole
- [v0.01] ★ **A Place remembers time** — drag the ribbon and the canvas is as it was on Tuesday, from the snapshots undo already takes *Moved into the current release on 2026-10-04, when the owner ruled that the complete canvas is one milestone — [ADR 0086](decisions/0086-the-complete-canvas-is-one-current-milestone.md). The tier moved rather than the scope gate being crossed: `CLAUDE.md` binds building to what this file says, so the file is what changed*
- [v0.01] **Every screen is a view onto the canvas** — two displays are two viewports at their own zoom, not two desktops; on a small screen, focus shows one frame at a time *Moved into the current release on 2026-10-04, when the owner ruled that the complete canvas is one milestone — [ADR 0086](decisions/0086-the-complete-canvas-is-one-current-milestone.md). The tier moved rather than the scope gate being crossed: `CLAUDE.md` binds building to what this file says, so the file is what changed*
- [v0.01] **The habits people arrive with still work** — the keys that cycle windows, close one and switch desktops become cycle frames, remove from canvas, and move between Places *This was `v1` until 2026-09-30, when the owner put the full canvas experience into the current release. The tier moved rather than the scope gate being crossed: `CLAUDE.md` binds building to what this file says, so the file is what changed*
- [v0.01] **A panel out of view costs nothing** — it is a still picture until it is reached, so a Place holding forty things is not forty programs running *Moved into the current release on 2026-10-04, when the owner ruled that the complete canvas is one milestone — [ADR 0086](decisions/0086-the-complete-canvas-is-one-current-milestone.md). The tier moved rather than the scope gate being crossed: `CLAUDE.md` binds building to what this file says, so the file is what changed*
- [v0.01] **Every canvas also answers as a list** — its panels in order, by keyboard and to a screen reader, because a surface that needs a touchpad excludes people (EN 301 549) *Moved into the current release on 2026-10-04, when the owner ruled that the complete canvas is one milestone — [ADR 0086](decisions/0086-the-complete-canvas-is-one-current-milestone.md). The tier moved rather than the scope gate being crossed: `CLAUDE.md` binds building to what this file says, so the file is what changed*
- [v1] ★ **The alo Bar** — ask, find, open, create or hand over, from one place. **It works with no model at all**: applications open, files are found, settings change, arithmetic is exact, commands run
- [v0.01] ★ **Give it to alo** — anything selected can be done by hand or handed over, and **a whole goal can be handed over**: alo shows its plan, works under one capsule, and returns only the decisions that must be the person's. The person may step in, take over a piece or stop it *The stopping clause of this line is carved out at `[v0.01]` for one surface only, by the owner's decision of 2026-10-03: a window put aside shows what alo is doing in it and offers Stop, which is promised in its own line above. **Nothing else here moved.** A whole goal handed over, the capsule and the plan shown first remain `[v1]`, and a reader finding stopping at two tiers should take the narrower line for the put-aside surface and this one for everything else. The carve-out is recorded on both lines because the Tidy-this-canvas promise below shows what happens when it is recorded on neither — one operation at two tiers, and a lane left to work out which governs.* *Moved into the current release on 2026-10-04, when the owner ruled that the complete canvas is one milestone — [ADR 0086](decisions/0086-the-complete-canvas-is-one-current-milestone.md). The tier moved rather than the scope gate being crossed: `CLAUDE.md` binds building to what this file says, so the file is what changed*
- [v0.01] **When the machine moves a window, the person is told** — a frame the machine had to move, because a fixed control came to cover its name, says so: what moved, and **where it was**, so the person can put it back. *Added 2026-10-04 by [ADR 0086](decisions/0086-the-complete-canvas-is-one-current-milestone.md), which names recovery notices as part of the complete canvas. It is **not** the notifications portal and not calm notifications — both stay where they are, because they are the ordinary notification system and the canvas needs one sentence about one frame. The record already exists: `Recovery::BroughtBack` carries where the frame was, and nothing says it*
- [v1] ★ **History** — what happened, why, and undo. Agent actions come from the kernel-watched record; a person's own work is shown from file versions, never from watching them
- [v1] **Content is the interface** — an open object fills the screen and tools appear when something is selected
- [v1] **No dock by default** — the alo key, the bottom edge or a swipe reveals the alo Edge. Its replacement must be found by somebody who has never seen it within thirty seconds, tested with people; a person may pin a dock
- [v0.5] **Which system the machine starts by default** — changed from alo OS's settings or from Windows, kept in one place both can reach (the EFI system partition), so both sides always show the same answer (ADR 0066). alo OS changes it through a verb on the broker's list, never by a person writing under /boot
- [v1] ★ **The privacy symbol is always visible and can never be hidden** — *private*, *local activity*, *data leaving*, *camera or microphone on*
- [v1] **Notifications are decisions** — *requires you*, *working*, *finished*, and *finished* only for work the person walked away from
- [v0.01] **Every application lives in a Place**, including the ones with ordinary windows, so there is one world rather than a modern half and an old half *This was `v1` until 2026-09-30, when the owner put the full canvas experience into the current release. The tier moved rather than the scope gate being crossed: `CLAUDE.md` binds building to what this file says, so the file is what changed*

## alo's visual language

Adopted by the owner on 2026-09-22 and not yet built. One rule runs through all
of it, so a person can read at a glance what is real, what is proposed, and who
is acting: **a ghost is proposed, solid is real, and deep teal is the agent** —
deep teal always with its mark and its word, never by colour alone.

- [v1] ★ **Ghost previews** — before the agent changes anything, a translucent ghost of the result appears in place: the files with their new names, the draft, the edited design. Accepting makes it solid; declining fades it away
- [v1] **The agent's presence** — a thin deep teal edge on the window the agent is working in, and its named cursor inside applications: *alo, for Disan*. Where the agent is, always, without a pop-up
- [v1] **The trust dial** — ADR 0064's three levels for running code as one dial in the agent's capsule, set per project
- [v1.1] ★ **The time ribbon** — a ribbon at the bottom edge; dragging it back fades the whole desktop into the past, with what changed glowing, and letting go restores what the person picks
- [v1] **The approval stack** — what the agent wants to do arrives as cards to accept or decline, each showing a real before and after: marked-up text, two versions of a design side by side, a difference in code
- [v1.1] ★ **Where the data went, on a map** — a small map on which a line is drawn to wherever anything leaves the machine for: *Paris*, *Frankfurt*. On most days nothing moves, and that stillness is the point
- [v1.1] **Explain mode** — hold one key and point at anything, and a card says what it is, what it does and what it can reach. For everyone arriving from another system
- [v1] **One bar for everything** — search, settings, commands and the agent are one bar, typed or spoken, and every action says exactly what it will do before it runs
- [v1] **Project spaces** — moving between projects rather than applications; the dock becomes the project's tools, with its people, files and the agent's memory of it
- [v1] **Calm notifications** — delivered in batches at times the person chooses, summarised by the agent, with a capsule saying how many are waiting

## What makes a person want it

Adopted by the owner on 2026-09-22 and not yet built. Each builds on something
alo OS already owns — the record, undo, the egress indicator, the model on the
machine — and each is a choice the person makes (Law 5), never a thing done to
them.

- [v1] ★ **Projects instead of applications** — open *the client's website* and its mail, meetings, tasks, design, code and the agent's memory of it are one place, not six windows. The agent's one project memory made visible
- [v1] ★ **Windows applications, and Android applications**, run through a compatibility layer, sandboxed and asking for what they need like any application — the largest single reason a person cannot leave Windows, removed. macOS applications are not promised: Apple's licence forbids it
- [v1] ★ **A throwaway box for anything downloaded** — run it in a box that closing erases without trace, built on the same sealed box ADR 0064 gives the agent
- [v1.1] ★ **Scroll the machine back in time** — a timeline to drag back to *Tuesday, 10:32*, see files and settings exactly as they were, and bring back only what is wanted. Built on the snapshots undo already takes
- [v1.1] ★ **Replay what the agent did** — every file it touched and every connection it made, step by step like a recording, with any single step undoable
- [v1.1] ★ **The machine works while the person sleeps** — tasks queued at night run on the machine's own model while it is charging, and the morning brings *here is what I did — approve or undo*. No paid cloud; the person's own hardware
- [v1.1] ★ **Live translation of anything, on the machine** — any window, document, subtitle or call, between the 24 official EU languages, with nothing sent anywhere. It works on a plane
- [v1] **"Why did my computer do that?"** — *why is it slow*, *why did the network drop at three*, *what changed since yesterday*, answered with evidence from the record rather than a guess
- [v1] ★ **Leave Windows in one evening, and come back if you want** — files, bookmarks, network passwords and the wallpaper brought across; the agent shows where each thing now lives; thirty days in which going back to Windows is one click, on the road *remove alo OS* already provides
- [v1] ★ **A monthly privacy receipt** — *this month, nothing you wrote went to an AI company; this much went to system updates; here is every destination*, signed so it can be shown to someone else
- [v1] **Settings in one sentence** — *warmer screen after eight, silence during meetings* becomes a set of rules the person can read and change, never hidden behaviour
- [v1.1] **Remember everything, if the person wants it** — a searchable memory of what was on screen, off by default, kept only on the machine and encrypted, with applications that can be excluded, and never readable by the agent without a grant. It is what *context offered, never watched* allows a person to choose, and it records other people's words too, which the switch says plainly
- [v1.1] **The machine suggests a workspace**, it never rearranges one — *switch to your photo editing space?* when a camera is plugged in, and nothing moves unless the person says so
- [v1.1] **Self-healing by going back** — when something breaks after a change, the machine says what happened and offers yesterday's working system in one click, on the rollback the image already has. Never a silent fix

## The person chooses (ADR 0064)

Law 5: on their own machine a person decides, and every protection is a default
they can change with its cost said plainly. These are decided and not yet built.

- [v1] ★ **Running code, at the level the person picks** — a sealed box the kernel locks down (the default), asking each time, or full trust. At every level each run is recorded and a snapshot taken first, so it can be undone
- [v1] Update checking can be turned off, saying that security fixes stop arriving while it is off
- [v1] *Trust devices on this network*, off by default, saying that it also trusts guests' phones and every device on the network
- [v1] A provider address that is not https can be allowed after a plain warning that the key crosses the network readable
- [v1] A **details** view that shows each rented component's own name as data, for the person who wants to know; plain words stay the default everywhere
- [v1] Screenshot-and-click switched on by the person for their own machine
- [v1] At full trust, a turn whose kernel boundary cannot be applied may run if the person chooses, told that the kernel is not watching it, and the record says so
- [v0.5] **Fast Startup: the installer asks.** Turning it off is recommended when a disk is shared, and both answers are safe because alo OS never mounts the Windows partition read-write

## Non-goals

**No kernel.** Linux, unmodified — hardware support is where OS projects die and
we do not fight that battle. **No inference kernels** — we do not compete with
llama.cpp or vLLM. **No model training toolchain from us** — we serve and adapt open weights, and building a trainer is somebody else's product. **That is a decision about what we build, never a limit on what the machine allows**: a person with the hardware runs whatever training they like on it, with the tools they choose, because it is their machine and their electricity. **No general-purpose distribution** — no package manager for the world and
no attempt to be Ubuntu; software is installed as sandboxed Flatpaks from
repositories other people run (ADR 0005), which is how we have applications
without packaging them. **No third-party device management** — fleet features
exist for alo OS machines; an MDM product is a different company. **No phone or
tablet** — not in v1, possibly never. **No directory service** — we do not
rebuild Active Directory or LDAP, and we do not become the place a company's
identities live; alo identities and pairing are what we offer. **No trusted-network
setting by default** (ADR 0003) — pairing stays how machines trust each other; a
person may opt in on their own network, told that it also trusts every device on
it (ADR 0064). **No code runs unless the person chose it** (ADR 0064, replacing
ADR 0001 §1): the verb list stays typed and closed, and running code is the
person's grant — a sealed box by default, asking each time, or full trust.

Every absence here is a sales argument.
