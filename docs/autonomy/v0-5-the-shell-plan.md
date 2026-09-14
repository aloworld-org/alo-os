# v0.5 — the shell: the half a person looks at

**Workstream:** the fourteen `ROADMAP.md` lines that cannot start without a
screen, beginning with the one v0.01 line still open — **the sign-in screen**
— and then the surfaces that make the agent's promises visible rather than
merely decided.
**Why it exists:** on 2026-09-14 the owner asked what was blocking the
remaining v0.5 work and lifted the rule that reserved `crates/alo-shell` for
the desktop lane, which has been away since before 2026-09-12. Every crate
beneath these surfaces is built, gated and waiting: `alo-greeting` is
*everything the greeter does except drawing*, `alo-approving` is *the sentence
a person approves*, `alo-egress` is *what leaves and the line shown while it
does*, `alo-recounting` is *what a person is told afterwards*. Each of them
says, in its own header, that the screen is somebody else's. This plan is that
somebody.

**Crates this plan owns:** `crates/alo-shell` and `tools/graphics-check`.
Nothing else. Every decision these surfaces render is already made in a crate
this plan **reads and never edits** — `alo-greeting`, `alo-accounts`,
`alo-approving`, `alo-egress`, `alo-indicator`, `alo-recounting`,
`alo-appearance`, `alo-overlay`, `alo-strings`, `alo-saying`. If a surface
needs a decision that is not there, **that is a finding in the report and the
task stays open**; it is never a decision made in a drawing crate.

**If the desktop lane returns**, it pulls `main` like every other lane and
continues from where this got to; nothing here is on a branch and nothing
rewrites what it built. The 133 files it left in `alo-shell` are its work and
this plan adds to them rather than replacing them.

**What this plan may not do:** tick anything *on the machine* — a surface that
has never been seen on a certified machine is `- [x] The code.` and nothing
more; put a sentence on a screen that is not in the vocabulary `alo-saying`
collects (`CLAUDE.md`: hardcoded English is a bug); or name a rented engine
anywhere a person reads. Before writing the next task, `git pull` and read the
plan as published.

**These gates need the graphics libraries** `docs/autonomy/GRAPHICS.md` lists,
in the distribution the gates run in. They are already there — the workspace's
clippy compiles this crate today — and a worker that finds them missing
installs them by that document rather than working around them.

## Tasks

### 1. The sign-in screen

**Status:** **Done, 2026-09-14.** `crates/alo-shell/src/sign_in_*.rs` and
`nested_sign_in.rs`; evidence and decisions in
`docs/autonomy/updates/the-sign-in-screen-drawn-on-the-nested-compositor.md`.
The code only — a certified machine has not seen it, and the fields carry no
label because no crate declares one (a finding in that report).
**Depends on:** nothing.

`ROADMAP.md` v0.01: *boots on one certified machine, firmware to sign-in* —
the last v0.01 line with nothing drawn behind it, and the reason the demo
stops at a console. `alo-greeting`'s own header names what is missing: *a
surface had to authenticate, then knock, then render what came back… a surface
that authenticated and forgot to knock is a screen that takes a correct
password and does nothing at all.* That composition is built and tested. This
draws it.

- **Acceptance:** a surface in `alo-shell` draws a name and a password field
  on the compositor this crate already is, takes keystrokes through the seat it
  already has, and calls `alo_greeting::Greeting` — **never `alo-accounts` or
  `alo-sessiond` directly**, because the order is the composition's and a
  surface that re-implements it is the bug that crate exists to prevent; what
  it shows for each outcome is `Greeting`'s own words through `alo-strings`,
  with a test that every sentence the screen can show is in the vocabulary and
  none is written in this crate; a wrong password says what `alo-greeting`
  says and never how many attempts remain or whether the name exists, and a
  test names that refusal; the password is never drawn, never logged, never in
  a panic message, and is held only as long as the composition needs it — a
  test reads the crate's shipped source for a `Debug` that could print one;
  and when the knock opens a session the surface hands over to it and stops
  drawing, so two things never own the screen at once.
- **Constraint:** nothing here decides who may sign in, what a password is
  worth, or what a session is. No account is created from this screen —
  `alo-setting-up` owns first-run, and *make an account* is its sentence.
  Measured under a nested compositor the way this crate's other tests are;
  a certified machine has never seen it and the report says so.

### 2. The egress indicator, on a screen

**Status:** **Done, 2026-09-14.** `crates/alo-shell/src/egress_status*.rs` and
`nested_egress_status.rs`; evidence and decisions in
`docs/autonomy/updates/the-egress-indicator-drawn-in-the-status-area.md`. The
code only — a certified machine has not seen it, there is no direct-display
submission yet, and the sign-in screen carries no status area (a finding in
that report).
**Depends on:** 1.

`docs/features.md`, ★: *the egress indicator lives in the status area, so
"nothing has left this machine" sits where a person already glances rather
than somewhere they must learn to look.* Law 1's whole promise —
*nothing leaves silently* — has been decided since `alo-egress` and shown to
nobody. `alo-indicator` already says what a screen must show; its
on-the-machine half has been empty since the repository began.

- **Acceptance:** a surface draws `alo_egress::Indicator`'s lines while they
  are live and nothing when it is quiet, in a status area at the far end of
  the dock wherever the dock is; a question answered on this machine draws
  **nothing**, and one answered anywhere else — a provider, or the machine
  down the corridor — draws its sentence, which is the difference the promise
  rests on and is one test with both halves; the line is the agent's name and
  the destination as `alo-egress` words them, never re-worded here; terracotta
  is the colour and it **never arrives alone** — a mark and a word beside it,
  because a signal carried by hue fails for anybody who cannot distinguish it
  and EN 301 549 does not allow colour as the only means (`docs/features.md`,
  ★); and what alo OS does on its own errand is on the same indicator as what
  an agent caused, because *no telemetry* is a promise only somebody who can
  see the machine's unasked traffic is in a position to check.
- **Constraint:** the indicator shows; it never decides. Whether something may
  leave is `alo-egress`'s and is asked before a socket opens, not by this
  surface. No dismissing, no hiding, no setting that turns it off.

### 3. The sentence a person approves

**Status:** ready. **Depends on:** 1.

ADR 0001: an agent proposes and a person approves, and `alo-approving` is
*one approval, and the sentence a person approves: the change an agent
proposed, put in front of somebody exactly as the turn worded it, answered
once, and carried out.* Exactly as the turn worded it — which means the
surface may not improve on it.

- **Acceptance:** a surface draws one proposal at a time — the sentence
  `alo-approving` carries, unedited, unsummarised, unwrapped into a different
  claim — with two answers and no third, and a test that the text drawn is
  byte-for-byte the text the turn wrote; **nothing is preselected and nothing
  proceeds on silence**, with a test that a surface left alone answers
  nothing; the answer goes back through `alo-approving` once and a second
  answer to the same proposal is refused there rather than here; what a person
  declined says nothing about why, because *"no" is the whole answer and a
  system that recorded a reason would be a system that asked for one*; and a
  proposal that arrives while another is open is queued rather than replacing
  it, so nobody answers a question they did not read.
- **Constraint:** no *approve all*, no *remember this*, no timer. Each of the
  three is the mechanism by which approval becomes a formality, and ADR 0001
  is what they would hollow out. If a person wants an agent to do a class of
  thing without asking, that is a **grant**, made deliberately in
  `alo-picking`, and this surface is not a road to one.

### 4. What the machine did, in front of the person

**Status:** ready. **Depends on:** 3.

`docs/features.md`: *afterwards, ask what it did.* `alo-recounting` composes
the account and has every sentence a person reads; ADR 0009 says the plain
way to reach it must exist beside the agent's. This is the window.

- **Acceptance:** a surface draws `alo_recounting`'s account — each entry's
  clause at the head of its line and the machine's own sentence after it, in
  the order the record has them, newest first — and every clause is the
  vocabulary's rather than this crate's, held by a test; what was **refused**
  is drawn as plainly as what ran, because a record that showed only successes
  cannot answer what a security review asks; an entry that names no agent —
  alo OS's own errand — is drawn without inventing one; the window is reachable
  without asking an agent anything, which is ADR 0009, and a test names that
  road; and asking the agent *what did you do?* reaches the same account rather
  than a second telling of it.
- **Constraint:** the surface reads the record and writes nothing to it. No
  filtering that could hide a refusal by default, no search that changes what
  *today* means, and no summary — `alo-recounting`'s own rule is that a
  surface able to word what happened would be a machine with two accounts of
  one moment.

### 5. The ordinary desktop: the dock, the status area, and what is running

**Status:** ready. **Depends on:** 2.

`ROADMAP.md` v0.5: *the ordinary desktop* — the furniture a person expects,
and the frame the other surfaces sit in. `alo-dock` decides the dock's edge
and its per-display placement; `alo-appearance` carries the accent set;
`alo-measuring` answers what is running and what is filling the disk, and has
no window.

- **Acceptance:** a dock is drawn on the edge `alo-dock` names, per display,
  with a status area at its far end holding the clock, battery, network,
  volume and the egress indicator of task 2; the accent is `alo-appearance`'s
  and **terracotta is never offered**, because it means the agent and nothing
  else (`docs/features.md`, ★) — a test refuses a palette that offers it; a
  window drawn from `alo-measuring` shows what is running and what is using
  the machine, and a second shows what is filling the disk as sizes that open
  up, each number the one the kernel gave with nothing derived here; and
  everything drawn follows light and dark as `alo-appearance` decides it,
  never as this crate decides it.
- **Constraint:** the dock is furniture and holds no authority: nothing is
  granted, approved or revoked from it. What the status area shows about the
  machine is measured by the crates that measure, and this surface adds no
  number of its own.

### 6. One place for settings

**Status:** ready. **Depends on:** 5.

`ROADMAP.md` v0.5: *Settings, as one place — network, display, sound,
printers, storage, keyboard, accounts, privacy, updates. Not a scattering of
dialogues a person has to know the name of.* Every setting this OS has is
already decided in a crate and written into the person's own file; none of
them has ever been shown.

- **Acceptance:** one surface holds every setting the machine has today — what
  a person chose at setup (`alo-setting-up`), the model and provider
  (`alo-choosing`), appearance (`alo-appearance`), the dock (`alo-dock`),
  shortcuts (`alo-shortcuts`), the grants a person made (`alo-granted`) and
  the pairings they made (`alo-nearby`) — each section reading and writing the
  same file its crate already owns, with a test per section that the surface
  changes nothing the crate would not; **the grants and pairings sections show
  what has been granted to what in one list and revoke the same way**, which
  is `docs/features.md`'s ★ promise and `alo-granted`'s own shape; and a
  setting the machine does not have yet is **absent**, not greyed out, because
  a dialogue full of disabled controls is a promise nobody made.
- **Constraint:** this surface decides nothing. Every value it writes goes
  through the crate that owns it, so that what a person sets and what the
  machine enforces cannot disagree. Nothing here is a second place to grant:
  making a grant is `alo-picking`, a person standing in a folder and choosing
  it, and revoking is the one action `alo-granted` already enforces.
