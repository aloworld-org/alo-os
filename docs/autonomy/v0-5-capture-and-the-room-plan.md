# v0.5 — capture, and the room you are sitting in

**Workstream:** one `ROADMAP.md` v0.5 line, and its star — *Capture: screenshots,
annotation, screen recording with audio, screen sharing — and an indicator
whenever screen, camera or microphone is in use*. `docs/features.md` puts the
point of it exactly: ★ *a visible indicator whenever the screen, camera or
microphone is in use — by any application, including ours. Law 1 is about egress;
this is the same instinct applied to the room you are sitting in.*
**Why it exists:** written 2026-09-15 so that no v0.5 line is without a plan.

**Crates this plan owns, all new:** `crates/alo-capturing` (what a capture is —
the screen, a window, a region, once or continuously, with or without sound — and
where it goes), and `crates/alo-in-use` (**what is watching or listening right
now**, and the line shown while it is). **It reads and never edits**
`alo-portals` and `alo-capability` (a screenshot, a screen capture, the camera and
the microphone are the facilities ADR 0040 made grantable to applications),
`alo-granted`, `alo-clipboard` (a screenshot to the clipboard is a clipboard
payload), `alo-egress` and `alo-indicator` (the egress indicator is the model this
one follows, and they must not be confused), `alo-record`, `alo-displays` and
`alo-dividing` (a window or a region is on a display), and `alo-saying`. **Nothing
in `crates/alo-shell`**: the selection, the annotation tools and the indicator are
drawn by the shell plan's later tasks.

**What this plan may not do:** tick anything *on the machine*; write a screen
grabber, an encoder or a media server of our own — PipeWire, the portal's screen
cast and a pinned encoder are rented, configured and never patched (ADR 0011); make
the in-use indicator dismissable, hideable or configurable off; or let an agent
capture the screen by any road other than a verb a person approves. Before writing
the next task, `git pull` and read the plan as published.

## Tasks

### 1. What is watching or listening, right now

**Status:** ready. **Depends on:** nothing.

★ The indicator comes first, before any capture exists, so that nothing can be
built that captures without it.

- **Acceptance:** `alo-in-use` holds, as a live value, **every current use of the
  screen, the camera and the microphone**, each naming who is using it — an
  application by its name, an agent's turn, or alo OS itself — read from the media
  server's own record of open streams rather than from what applications say they
  are doing, held by a test that opens a stream through the rented server with no
  portal involved and finds it listed; the line shown while something is in use is
  a sentence in the vocabulary naming what and who (*the camera, by the video-call
  application*); **alo OS's own captures appear on it like anybody else's**, with a
  test; the indicator is **mark, word and position, never colour alone** (ADR 0010),
  and it is not terracotta unless an agent is the one using it; and there is no
  variant that hides a use, no allow-list of trusted applications, and no setting
  that turns the indicator off.
- **Constraint:** it shows; it never decides. Whether an application may use the
  camera is `alo-portals`' grant. The in-use indicator and the egress indicator are
  two lines, because *the microphone is on* and *something left this machine* are
  different warnings, and a test holds that neither is drawn as the other.

### 2. A screenshot: the screen, a window, a region — to a file or the clipboard

**Status:** ready. **Depends on:** 1.

- **Acceptance:** `alo-capturing` takes the whole screen, one window or a selected
  region through the rented screen-capture mechanism, and writes it **to a file in a
  folder the person chose, or to the clipboard**, never both unasked; the file name
  carries the date and nothing about what was on screen; a screenshot is a use of the
  screen and appears on task 1's indicator for its moment, held by a test; **a
  window from another person's session, or the lock screen, cannot be captured**;
  and an application asking for a screenshot through the portal is judged by
  `alo-portals` against its grant before anything is taken.
- **Constraint:** nothing is uploaded, shared or sent anywhere by taking a
  screenshot. No automatic cloud folder, no *share* step that is on by default.

### 3. Annotation, without opening anything else

**Status:** ready. **Depends on:** 2.

- **Acceptance:** a screenshot can be marked — arrows, rectangles, freehand, text,
  and a blur for what should not be shown — as a closed list of marks this crate
  holds as values; **blur is destructive in the saved file**, held by a test that
  reads the saved pixels and cannot recover what was blurred, because a blur that
  was a layer over the original would leak a password the first time somebody
  opened the file elsewhere; the original is kept unmarked until the person saves;
  and every tool has a name in the vocabulary.
- **Constraint:** nothing draws here; the marks are data the shell renders. No
  *enhance with the agent* button.

### 4. Screen recording, with audio, to a file

**Status:** ready. **Depends on:** 1, 2.

- **Acceptance:** a recording of the screen, a window or a region, with the
  microphone, the machine's sound, both or neither, chosen before it starts; it is
  written to a file in a folder the person chose, in a format a person's other
  machines can play, encoded by a pinned, rented encoder; **while it records, task
  1's indicator shows the screen and each audio source in use**, held by a test;
  stopping is one action, available from the indicator itself; and a recording that
  fails mid-way keeps what it had, says so, and never silently discards it.
- **Constraint:** no encoder is written here and no codec with licensing terms
  the image cannot carry is shipped; which encoder and format is decided with the
  devices plan's codec decision and named in the report.

### 5. Sharing the screen in a call

**Status:** ready. **Depends on:** 1, 4.

- **Acceptance:** a call application asking to share the screen receives exactly
  what the person picked — the whole screen, one window, or nothing — through the
  screen-cast portal, judged by its grant (`alo-portals`); the picking is the
  person's act and **cannot be pre-answered** by a setting such as *always allow
  this application*; while sharing, task 1's indicator names the application and
  what it can see; **notifications are not shown on a shared screen** while it is
  shared; and ending the share is one action on the indicator.
- **Constraint:** the network half of a call is the application's, not ours. alo
  OS decides what it may see and shows that it is seeing it.

### 6. An agent and the screen

**Status:** ready. **Depends on:** 1, 2.

The agent overlay offers *context on invocation* (v0.01) and never harvests. A
screenshot is the most harvest-shaped thing a machine has.

- **Acceptance:** an agent may see the screen **only** through a verb whose proposal
  says *a picture of your screen* and is approved like any other change, and the
  image goes to the turn and nowhere else — never kept, never indexed, never in the
  record beyond *a picture of the screen was taken*; while the turn holds it, task
  1's indicator shows the screen in use **by the agent**, in terracotta with its mark
  and word; a test reads the shipped source of `alo-agentd`'s crates for any capture
  that is not reached through that verb; and whether that verb exists at all in v0.5
  is recorded as a finding if `docs/contracts/agent-verbs.md` does not list it —
  it is then not added here.
- **Constraint:** no continuous watching, no *screen awareness*, no background
  capture of any kind for an agent. `docs/features.md` v1's *screenshot-and-click*
  is last resort, policy-disabled and not this.

### 7. Every sentence, and the walk through a call

**Status:** ready. **Depends on:** 1, 2, 3, 4, 5, 6.

- **Acceptance:** every sentence these crates can say is in the vocabulary with a
  translator's note; one walk — take a region screenshot, blur a field, save; join a
  call and share a window; record thirty seconds with the microphone — produces the
  exact sequence of indicator lines and sentences a person meets, recorded as a
  table and held by one test; no sentence names PipeWire, a portal, an encoder or a
  codec.
- **Constraint:** nothing here re-decides what the sentences describe.
