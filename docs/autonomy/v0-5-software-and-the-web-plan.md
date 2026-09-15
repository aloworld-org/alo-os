# v0.5 — software and the web: installing applications, the browser, the proxy, and adapters

**Workstream:** four `ROADMAP.md` v0.5 lines — the half of *Software* that is not
portals or grants (*install sandboxed applications, update and remove them;
corporate proxy support*); *A web browser for the open web*; ★ *Application
adapters, and the accessibility fallback for applications without one*; and the
fresh-machine half of *The ordinary desktop* (*a text editor and an image viewer, so
a fresh machine is not helpless*, and *a terminal*). They belong together because
they are **the software a person did not write and alo OS did not write either**,
arriving on the machine and being reachable — by the person, and through adapters by
the agent.
**Why it exists:** written 2026-09-15 so that no v0.5 line is without a plan. The
applications plan decided what an application may be granted; this plan is how the
application gets there.

**Crates this plan owns, all new:** `crates/alo-software` (installing, updating and
removing a sandboxed application, from which sources, and the applications a fresh
machine ships), `crates/alo-proxy` (one machine-wide proxy, honoured by applications),
and `crates/alo-adapters` (loading an adapter against
`docs/contracts/app-adapters.md`, and the accessibility fallback). **It reads and
never edits** `alo-portals`, `alo-granted`, `alo-applications` and `alo-secrets` (the
applications plan's — an installed application arrives with no grants), `alo-capability`
(an adapter's verbs are verbs; if registering them needs a change there, that is a
finding and a decision, not an edit), `alo-egress` (installing and updating are
errands that leave; the indicator fires), `alo-keeping-up` (the machine-keeps-itself
plan's — an application update is not a system update), `alo-access` (the access
plan's accessibility tree is the fallback's tree), `image/` and `alo-image` (the
installer plan's — the fresh-machine applications are named there by the installer
lane, with this plan's list), and `alo-saying`. **Nothing in `crates/alo-shell`.**

**What this plan may not do:** tick anything *on the machine*; build a package
manager, a sandbox, a browser engine or a terminal emulator (Flatpak, its portals, the
browser and the terminal are rented, configured and never patched — ADR 0005, ADR
0011, and `docs/features.md`: *a pinned upstream one, since our own engine is not
scheduled*); name any of them to a person (*they install an application — not a
Flatpak*); accept an adapter that takes model-written code (the contract's one rule);
or install anything unsandboxed (v1, and deliberate). Before writing the next task,
`git pull` and read the plan as published.

## Tasks

### 1. Installing, updating and removing an application

**Status:** **Done, 2026-09-15.** Built in `crates/alo-software` on
`docs/decisions/0042-installing-an-application-is-an-errand-and-an-agent-only-proposes-it.md`;
the report is `docs/autonomy/updates/installing-updating-and-removing-an-application.md`.
`alo_egress::Errand` gained installing, checking for application updates and
updating (ADR 0042 part 1, the change named there); removing reaches no network
and puts nothing on the indicator (part 2); `install_application` is a change
needing no grant, with its reason (part 3). Reading an organisation's list of
permitted places out of the machine description is task 8. What is not yet
shown is the rented tool on a machine: no machine this task ran on has it, and
the report says what the on-machine acceptance is. **Depends on:** nothing.

- **Acceptance:** `alo-software` installs an application from the sources a person or
  an organisation enabled — Flathub, or a repository the organisation runs — through
  the rented tool, and **the application arrives with no grants**, held by a test that
  reads the one list after installing and finds nothing new; installing, updating and
  removing are errands that leave the machine and appear on the egress indicator
  (`alo-egress`), named as what they are; an update is **offered, never applied while
  the application runs**, and an application update never restarts the machine;
  removing an application ends its grants in the one list in the same act, held by a
  test; the signature of every source is verified by the rented tool and a source that
  fails verification is refused in words, never installed from; and an agent may
  propose installing an application through a verb a person approves, never install
  one by itself.
- **Constraint:** no package manager of ours. Sources are the rented tool's
  configuration; an organisation's source list is a bound in the machine's description
  (ADR 0016), never a person's setting it overrides silently.

### 2. What a fresh machine has, so it is not helpless

**Status:** **Done, 2026-09-15.** The decided list is
`crates/alo-software/shipped.toml`, read and held by `alo_software::Shipped` —
Firefox, Dolphin with Ark for archives, GNOME Text Editor, Loupe, Papers and
Ptyxis, each at its identifier, release and licence on the place a fresh machine
installs from — and the installer plan reads that same file
(`alo_software::shipped::WHERE_IT_IS`) rather than a copy in `image/`. Each goes
through task 1's two steps, so a person updates and removes any of them, the
browser included. The terminal is a person's own: on
`docs/decisions/0043-the-terminal-is-a-persons-and-never-an-agents.md` (accepted
2026-09-16, option C) this plan made **that ADR's additive change to
`alo-capability` and nothing else in that crate** — `persons_own.rs`,
`GrantError::APersonsOwn` with its word, and the three refusals — taken from the
held branch `held/software-task-2-awaits-capability` and rebased onto current
`main`. The rented tool on a machine is still outstanding, as it is for task 1;
the report says what that acceptance is. The report is
`docs/autonomy/updates/what-a-fresh-machine-has.md`. **Depends on:** 1.

- **Acceptance:** a decided list — a web browser, a file manager with trash and archives
  that open, a text editor, an image viewer, a document viewer, a terminal — each a
  **pinned upstream application**
  named by its source identifier and version, chosen for licence, maintenance and
  accessibility with the reasons in the report, installed the same way a person
  installs one (task 1) so that updating and removing it is no different; a person
  may remove any of them, including the browser; **the terminal is a person's and not
  an agent's** — a test holds that no verb reaches it (ADR 0001 §1: no arbitrary
  command); and the list is handed to the installer plan as data it reads, not
  edited into `image/` here.
- **Constraint:** no application is patched or rebranded. Which browser is a decision
  with its alternatives recorded in the report; it is not a partnership.

### 3. The web browser, and what it may take from the machine

**Status:** ready. **Depends on:** 1, 2.

- **Acceptance:** the browser from task 2 opens web addresses from any application
  through the open-with portal; **it is a sandboxed application like any other**, with
  no grants beyond what a person gives it through the portals; its downloads go to the
  folder a person chose through the file chooser portal; the machine-wide proxy (task 4)
  reaches it; and nothing alo OS ships sets its home page, search engine or telemetry —
  a test reads the shipped configuration and finds the upstream's own defaults, with
  its telemetry **off** where the upstream allows a policy to turn it off, and the
  report names what could not be turned off.
- **Constraint:** no engine of our own and no fork. *No telemetry* (v0.01) is alo OS's
  promise; a rented browser's own behaviour is stated honestly, not claimed.

### 4. One proxy, machine-wide, honoured

**Status:** ready. **Depends on:** nothing.

*A great many company networks have no other route out.*

- **Acceptance:** `alo-proxy` holds one proxy setting — none, a manual address per
  scheme with exceptions, or an automatic configuration address — kept machine-wide as
  a bound an organisation sets or a person sets on a personal machine (ADR 0016); it
  reaches **every road out alo OS itself uses** — installing, updates, providers — held
  by a test per road, and is published to applications through the environment and the
  proxy portal they already read; a proxy's password goes to the keyring (ADR 0022),
  never to a file; **the egress indicator still names the real destination**, not the
  proxy, because *it went to the proxy* is not what a person needs to know; and an
  automatic configuration script is fetched and evaluated **without running it in any
  process that holds a grant**.
- **Constraint:** no proxy server of ours. A policy that cannot be evaluated refuses,
  as `alo-egress` already does.

### 5. An adapter, loaded against the contract

**Status:** ready. **Depends on:** 1.

★ *Installed applications become agents with typed verbs (`@blender`, `@resolve`,
`@gimp`).*

- **Acceptance:** `alo-adapters` loads an adapter as declared data — its application,
  its typed verbs with validated arguments, and its mechanism (automation API,
  accessibility tree, D-Bus) — **and refuses any adapter whose verb takes a script, a
  command or free text that becomes code**, with a test that submits one and finds it
  refused; each adapter verb is approved and recorded like any verb (ADR 0001) and has a
  by-hand road (ADR 0009) or is refused for lacking one; one reference adapter for an
  application task 2 ships is built end to end as the proof, with the application's
  own automation interface; and **if registering adapter verbs needs `alo-capability`
  to change, the task stops at a decision record** naming the change and who makes it,
  the way ADR 0040 did.
- **Constraint:** the contract is additive and published; nothing here changes it
  without versioning. No adapter uses screenshots and synthetic input (v1, last resort,
  policy-disabled).

### 6. The accessibility fallback, for applications without an adapter

**Status:** ready. **Depends on:** 5.

- **Acceptance:** an application with no adapter is readable and operable through its
  accessibility tree — the agent may **read** what a window shows and **activate** a
  control by its role and name, through two typed verbs, each proposed as a sentence
  that names the control and the application and approved like any change; the tree is
  read at invocation for that turn only, never watched; a password field's contents are
  **never** read, held by a test against a real application's password field; and what
  the fallback cannot do — a control with no name, a canvas — is said in words, never
  guessed at with coordinates.
- **Constraint:** no screenshot, no synthetic pointer event, no coordinates. The tree
  is the rented AT-SPI's.

### 7. Every sentence, and the walk from nothing to a working application

**Status:** ready. **Depends on:** 1, 2, 3, 4, 5, 6.

- **Acceptance:** every sentence these crates can say is in the vocabulary with a
  translator's note; one walk — set a proxy, install an application, open a web link
  in the browser, ask the agent to use the reference adapter, remove the application —
  produces the exact sequence a person meets, recorded as a table and held by one test;
  no sentence names Flatpak, Flathub by its tooling, AT-SPI, D-Bus or a browser engine.
- **Constraint:** nothing here re-decides what the sentences describe.

### 8. An organisation's permitted places, read from the machine's description

**Status:** ready. **Depends on:** 1.

*The organisation bounds; the person chooses* (ADR 0016), for where applications come from.

- **Acceptance:** `/etc/alo/agentd.toml` carries an optional section naming the places an
  organisation permits applications to come from, documented in
  `docs/contracts/machine-description.md` with the `format` rule `[questions]` follows (an
  older service must refuse a description whose software bound it cannot enforce, never
  ignore it); `alo-agentd` reads it into `alo_software::Bound` with who set it decided by
  who owns the file, exactly as `[questions]`; its absence is `Bound::Nobodys` and never
  a permissive list; and a test installs from a place the file does not name and finds it
  refused in the words naming the organisation, beside the same install on a machine with
  no section.
- **Constraint:** nothing here edits what `alo-software` decides; a section that does not
  hold stops the service, as `[questions]` does.
