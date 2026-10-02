# Applications people already use

**Why this exists.** The exit-gate reconciliation of 2026-09-27 refused three
promises at once: *the ordinary desktop* — no file manager, no editor, no
viewer, **no terminal**; *a web browser* — none pinned in the image; and
*software* — **no portal backend service exists**. So alo OS boots, the shell
draws, the agent works, and **there is nothing to open.**

**What is already built, so nobody builds it twice.** Three crates carry the
models and none of them runs on a machine:

- `alo-portals` — what a portal request is and how it is judged against a
  person's grants ([ADR 0040](../decisions/0040-what-an-applications-grant-is-over.md)).
  It decides, by design, nothing itself.
- `alo-software` — the rented installer asked argument by argument, with no
  shell between the list and the tool, and which places an application may come
  from on this machine and who said so.
- `alo-applications` — installed applications, what opens which media type, what
  a person chose, and what is said when nothing opens it.

**The gap is not a model. It is a service, a runtime, and five protocols.**

---

## Tasks

### 1. The portal backend, running

**Status:** in progress — **taken 2026-09-28**, the machine and the binary; the
unit and the image line are the owner's and are handed over rather than written.
**Depends on:** nothing.

**Corrected 2026-09-27, before anything was built.** This task said *no portal
backend service exists*, taken from the exit gate's refusal. Read rather than
believed: `alo-portals::Backend::serve_on` **owns
`org.freedesktop.portal.Desktop`** and answers four interfaces — Secret, OpenURI,
Settings and NetworkMonitor — reading a person's grants at every request so a
revocation is felt at the next one rather than at a restart, and
`the_portal_backend_answers_on_a_real_bus.rs` proves it on a real bus. **The
backend is written.**

**What is missing is narrower.** No binary calls `serve_on`, so nothing runs it,
and there is no unit to start it — every other daemon here has both
(`alo-convertd`, `alo-agentd`, `alo-boundaryd`). And the portals that need a
dialog are **deliberately** unregistered: `serving.rs` answers only those
decided without one, so the bus itself tells an application nothing answers the
rest. **FileChooser is among them**, which is what an application needs to open
or save anything — so this task ends with a running backend that still cannot
let Chrome open a file, and that second half is task 6's real content.

**And one of the four things the backend is built from has no real
implementation.** `Backend::answering_from` takes a machine, a keyring, the
sandboxes and a record. Three of them exist for a real machine — the keyring is
`alo-secrets`' `TheKeyring`, the record is `AnswersFile`, and `Sandboxes::under`
reads `/proc`. **`TheMachine` is implemented five times and every one is a test
double.** Nothing reads a real person's grants, their *what opens what*, and how
they set the machine to look, which is what `the_machine.rs` says is read at
every request so that a revocation is felt at the next one.

So the order inside this task is: the machine, then the binary, then the unit.
Writing the binary first would mean assembling a backend out of a thing that
does not exist.

Without all of it, a sandboxed application installs and can then do nothing at
all: not open a file, not save one, not be notified.

- **Also owed:** this would be **the first user unit in the image**. Every unit
  there today is a system one, and a portal backend serves one person's session
  bus rather than the machine — so where a per-person service is started from is
  a small structural decision to make rather than assume.

- **Acceptance:** a real sandboxed application asks through a real portal for a
  file and for a notification; the request is judged by `alo-portals` against
  what the person granted; a refusal is a sentence the person can read; and the
  **record shows what was asked and what was answered**, because a portal
  request is a grant in the sense of ADR 0001.
- **Constraint:** this crate carries no policy. Every decision stays in
  `alo-portals`; if the service needs to decide something, that is a finding
  about the split rather than a reason to decide it here.

### 2. A runtime in the image, and where applications may come from

**Status:** ready. **Depends on:** 1.

The image is verifiable and immutable, so software is not installed into it.
Flatpak is how an image-based system installs applications, and `alo-software`
already names the places one may come from and who said so — this puts that in
the shipped image rather than in a model.

- **Acceptance:** on a machine built from the pinned image, an application is
  installed, listed, updated and removed through `alo-software`'s own road; the
  recipe's label moves in the same change, which `pinned.toml` enforces.
- **Constraint:** the places are the ones a person or an organisation approved.
  A default that reaches any remote it likes is not a default, it is an absence.

### 3. The protocols real applications need

**Status:** ready — **and it is shared ground, so coordinate before editing.**
`crates/alo-shell` is where the canvas is being built, and this touches the same
crate. **Depends on:** nothing.

alo-shell speaks five Wayland protocols: compositor, output, seat, shm and
xdg_shell. That is enough for a fixture and not enough for Chrome, VS Code or
anything else people actually use. Missing, and each has a reason:

- **`linux-dmabuf`** — without it every frame is copied through the processor.
  It is also what video needs, so one piece of work pays twice.
- **`wl_data_device`** — copy and paste. An operating system without it is a
  demonstration.
- **`xdg_decoration`** — whether the application or the shell draws the frame,
  which on a canvas is the shell.
- **`text-input`** / **`zwp_text_input_v3`** — anything not typed in English.
- **`viewporter`** and **`presentation-time`** — scaling, and knowing when a
  frame was shown.

- **Acceptance:** a real unmodified application — not a fixture — runs, is
  keyboard-driven, copies and pastes, and draws through a buffer the processor
  never copies.
- **Constraint:** Smithay provides delegates for all of these. This is wiring
  and measuring, not invention, and anything that turns out to be invention is a
  finding worth writing down.

**The finding that constraint asked for, 2026-10-02: `linux-dmabuf` is the one
that is invention, and the other five were wiring.**

Four are wired or in flight — the clipboard, the frame decoration, the drawn
size, and text a person did not type. Each was a field on `Surfaces`, a line in
its constructor, a handler and a delegate. `linux-dmabuf` cannot be any of
those, and the reason is not a gap in smithay:

```text
DmabufState::create_global(display, formats)   must advertise the formats the
                                               GPU can actually import
DmabufHandler::dmabuf_imported(.., notifier)   must try the import and tell the
                                               client whether it worked
server.rs:115   Surfaces::new(&display.handle())   globals are made here
nested.rs:102   winit::init_from_attributes       the ONLY renderer in the
                                                  workspace is made here, later
Surfaces, Server                                  hold no renderer at all
```

**So the handler has nothing to import into.** Every other protocol's state is
answerable from what `Surfaces` already holds. This one needs a live renderer at
two separate moments — when the global is created, to say which formats are
offered, and at every import, to say whether this buffer worked.

**What must not be done, and is the reason this is written instead of built.**
A `dmabuf_imported` with no renderer can only accept optimistically: tell the
client the buffer is fine and discover at draw time that it is not. That would
advertise a protocol this compositor cannot honour, and **the failure would
arrive as a black window rather than as a refusal the client can handle.** It is
the same shape as a global nobody can use, inverted — not unreachable, but
answering yes without knowing.

**What it actually needs is a decision about where the renderer lives**, which
is architecture rather than wiring:

- `Surfaces` holds the renderer, which inverts the current direction — today the
  renderer is threaded in as a parameter at draw time, and `direct_target.rs`,
  `direct_loop.rs` and `direct_input_loop.rs` all take `&mut GlesRenderer`
  rather than reaching for one; or
- the import is deferred to the first draw, and the notifier is held until then,
  which means a client waits on a round trip it did not expect; or
- the global is created by whoever owns the renderer and the handler lives
  there too, which splits this protocol's state away from the other five.

**None of these is a line of wiring and each changes something another lane
touches.** So it is the owner's, and it is written here rather than decided by
the lane that found it.

*One measurement that is worth keeping on its own: the workspace constructs
exactly **one** renderer, `winit::init_from_attributes` in `nested.rs`, which is
the windowed development backend. The direct path takes a `&mut GlesRenderer`
everywhere and builds none. Whether a machine booting to `alo-compositor` has a
renderer at all is a separate question this measurement raises and does not
answer.*

### 4. A terminal, rented rather than written

**Status:** ready. **Depends on:** 3.

A terminal emulator is forty years of escape sequences, fonts, scrollback and
selection, and it is not the product. `foot` is Wayland-native, small and has no
baggage. **What is ours is the part nobody else has:** a terminal where alo runs
a command and the person sees exactly what ran, approved it, and can undo it.

- **Acceptance:** the terminal is in the image and opens on a real machine; a
  person types in it; **alo runs a command in it through the same road a person
  would**, and the record carries what ran ([ADR 0009](../decisions/0009-a-good-computer-without-the-agent.md)).
- **Constraint:** rented and unpatched (ADR 0011). A terminal we patch is a
  terminal we maintain.

### 5. A browser in the image

**Status:** ready. **Depends on:** 3.

Named in `docs/features.md` and pinned in nothing. It is also how most people
would first judge whether this machine works.

- **Acceptance:** the browser is pinned in the image by digest, opens on a real
  machine, plays nothing it cannot honestly play, and **what leaves the machine
  is on the indicator** like anything else.

### 6. The walk: install something, open it, keep it, remove it

**Status:** blocked — on 1 to 5. **Depends on:** 1, 2, 3, 4, 5.

One walk on a real machine: find an application, install it, open a file with
it, grant it something through a portal, see the grant in the record, update it,
and remove it — with a raster at each step and the exact sentences recorded as a
table the test reads out of the published report.

- **Acceptance:** as above, and the report says which machine it ran on.
- **Constraint:** an application that cannot be installed honestly is **a
  finding and a sentence**, never a special case in the installer.

---

## What this deliberately does not decide

**Windows and Android applications.** `docs/features.md` carries that at v1 and
it wants scoping rather than building: a named, tested list of applications that
work is a promise that can be measured, where a category is not.
