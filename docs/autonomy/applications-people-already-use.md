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

## What was found on 2026-10-02, before any of it was built again

**Tasks 2, 4 and 5 all read `ready`, and all three are built.** Measured before
starting task 2, which is the only reason none of them was built twice — and
task 1 above records the same thing happening to it on 2026-09-27, when the
exit gate said *no portal backend service exists* and the backend was written.

```text
image/Containerfile:422   dnf install flatpak
image/Containerfile:424   flatpak remote-add --if-not-exists --system flathub
                          with a signing key, and alo-software refuses a place
                          that does not check signatures before fetching (ADR 0073)
alo-software/asked.rs     install, installed, updates, update, remove, sources, open
                          — every verb the acceptance names
alo-software/shipped.toml seven applications, each by role, identifier, source,
                          version and licence:
                            web-browser  org.mozilla.firefox     <- task 5
                            terminal     app.devsuite.Ptyxis     <- task 4
                            file-manager org.kde.dolphin
                            archives     org.kde.ark
                            text-editor  org.gnome.TextEditor
                            image-viewer ...
alo-shipping.service      runs the walker once at first boot
```

**So the group is not four tasks of work. It is built, and owed a machine.**
Every one of these acceptances ends on a running machine — *installed, listed,
updated and removed through `alo-software`'s own road*, *opens on a real
machine* — and no machine this project owns can walk any of them.

**The terminal was rented rather than written, as task 4 asks**, and it is
`app.devsuite.Ptyxis` rather than the `foot` the task names. That is a decision
the list records and this task does not; whichever is right, **the task naming
one and the list carrying another is the kind of disagreement that is only
visible when somebody reads both**, and it is recorded here rather than
silently resolved by a lane.

*Four statuses in this repository read `ready` today and were built: these
three and task 40 of the delivery plan. In every case the work landed and the
sentence that described its absence was left on the page. A status is a claim
with a date, and this plan now carries dates for that reason.*

## Tasks

### 1. The portal backend, running

**Status:** blocked — on a machine, and on nothing else. **Depends on:** nothing.

**Re-read 2026-10-05 and the status line is right, while three paragraphs below
it were not.** The machine, the binary and the unit are all built, installed and
enabled; each correction below quotes what it replaced. Nothing here needs
writing. What it needs is a machine that boots, and the first image carrying a
compositor was built the same day.

**The binary and the unit both exist, verified on `main` 2026-10-03.** This line
read *in progress — taken 2026-09-28, the machine and the binary; the unit and
the image line are the owner's* until then, and all three had landed:
`crates/alo-portald/src/main.rs` calls `Backend::serve_on`, the crate is a
workspace member, and `image/Containerfile` builds it, installs it to
`/usr/libexec/alo-portald`, installs `alo-portald.service` and enables it. What
is left is a machine to start it on, which is why the word on this line is now
`blocked` — the supervisor reads that word and would otherwise hand this to a
worker a fourth time.

**Corrected 2026-09-27, before anything was built.** This task said *no portal
backend service exists*, taken from the exit gate's refusal. Read rather than
believed: `alo-portals::Backend::serve_on` **owns
`org.freedesktop.portal.Desktop`** and answers four interfaces — Secret, OpenURI,
Settings and NetworkMonitor — reading a person's grants at every request so a
revocation is felt at the next one rather than at a restart, and
`the_portal_backend_answers_on_a_real_bus.rs` proves it on a real bus. **The
backend is written.**

**What is missing is narrower, and two thirds of it has since been built.** The
portals that need a dialog are **deliberately** unregistered: `serving.rs`
answers only those decided without one, so the bus itself tells an application
nothing answers the rest. **FileChooser is among them**, which is what an
application needs to open or save anything — so this task ends with a running
backend that still cannot let Chrome open a file, and that second half is task
6's real content. **That part stands and is a design decision, not a gap:** when
a machine answers Settings, Secret, OpenURI and NetworkMonitor and a file dialog
does nothing, that is this task working.

**What this paragraph used to say, and why it is corrected, 2026-10-05.** It
said *No binary calls `serve_on`, so nothing runs it, and there is no unit to
start it — every other daemon here has both.* Both halves were already false
when the paragraph above this one was written: that paragraph records the binary
and the unit as verified on main on 2026-10-03, and this one denied them one
screen below. **A task that contradicts itself within a page is read in
whichever direction the reader opened it**, and a reader who opened it here
would have written a second `main.rs` and a second unit.

**All four things the backend is built from now have a real implementation.**
`Backend::answering_from` takes a machine, a keyring, the sandboxes and a record.
The keyring is `alo-secrets`' `TheKeyring`, the record is `AnswersFile`,
`Sandboxes::under` reads `/proc` — and the fourth is
`crates/alo-portald/src/this_machine.rs:101`, `impl TheMachine for ThisMachine`,
which answers `grants()` out of `alo_remembering::remembered` on the real file
and carries `applications()`, `appearance()`, `time_of_day()` and `reaching()`
beside it.

**What this paragraph used to say, 2026-10-05.** It said *`TheMachine` is
implemented five times and every one is a test double. Nothing reads a real
person's grants.* That stopped being true on **2026-09-28**, in the same `#212`
this task's own correction note above cites — the commit is literally titled *A
real machine for the portal backend, and the process that serves it*. The
sentence outlived its own correction by a week, two paragraphs below it.

*(A detail for whoever greps: `TheMachine` names **two different traits**.
`alo-portals`' is this one; `alo-installer` and `alo-installing` each declare an
unrelated trait of the same name, which is why a count of implementations across
the workspace answers a question nobody asked.)*

So the order inside this task was: the machine, then the binary, then the unit.
**All three exist**, and what is left is a machine to run them on.

Without all of it, a sandboxed application installs and can then do nothing at
all: not open a file, not save one, not be notified.

- **Settled, 2026-10-05, by reading what shipped.** This line said *this would
  be **the first user unit in the image** … where a per-person service is started
  from is a small structural decision to make rather than assume.* It is **not** a
  user unit, and there is no `user/` directory in the image at all.
  `alo-portald.service` is installed to `/usr/lib/systemd/system/` and bound into
  the person's session the way `alo-agentd` already was —
  `BindsTo=user@1000.service`, `WantedBy=user@1000.service`, `User=alo`, and
  `XDG_RUNTIME_DIR` and `DBUS_SESSION_BUS_ADDRESS` naming `/run/user/1000`. The
  decision was made by copying the pattern that was there, which is a fine answer
  and an unrecorded one. **It inherits that pattern's defect too:** the person is
  a number written into the unit, so a second person's session manager pulls up no
  portal — the same clause task 18 and
  `docs/decisions/0088-a-machines-grants-belong-to-a-person.md` name for the
  agent.

- **Acceptance:** a real sandboxed application asks through a real portal for a
  file and for a notification; the request is judged by `alo-portals` against
  what the person granted; a refusal is a sentence the person can read; and the
  **record shows what was asked and what was answered**, because a portal
  request is a grant in the sense of ADR 0001.
- **Constraint:** this crate carries no policy. Every decision stays in
  `alo-portals`; if the service needs to decide something, that is a finding
  about the split rather than a reason to decide it here.

### Proposed to the integration owner: the *Software* refusal names the wrong interface

**Not an edit.** `docs/autonomy/SHARED_MAIN.md` reserves `ROADMAP.md` to the
designated integration owner and asks everybody else to put proposed changes in
their own task report. This is that.

`ROADMAP.md`'s refusal of *Software* rests on one clause:

> there is **no backend service at all**: no binary, and nothing in `crates/` or
> `image/` implementing `org.freedesktop.impl.portal.*`. An application on a real
> machine would have nothing to talk to.

**The clause is literally true and its conclusion is false**, which is why it has
survived a week of people reading it. Measured on main at `0ed1fa6f`:

```
org.freedesktop.impl.portal.*          0 occurrences anywhere in the repository
org.freedesktop.portal.Settings        6
org.freedesktop.portal.Secret          5
org.freedesktop.portal.OpenURI         5
org.freedesktop.portal.NetworkMonitor  5
```

**Those are different interfaces.** `org.freedesktop.impl.portal.*` is the
backend that `xdg-desktop-portal` calls into; `org.freedesktop.portal.*` is the
front door an application talks to. alo OS has no `xdg-desktop-portal` and does
not want one — `alo_portals::Backend::serve_on` **owns
`org.freedesktop.portal.Desktop` itself**. So there is no `impl.portal` backend
and there never will be, and an application on a real machine has exactly the
thing the refusal says it lacks.

Dates, since the sentence outlived its subject:

```
refusal written       2026-09-26   #172
alo-portald added     2026-09-28   #212
in the image          built line 176, installed line 410,
                      unit line 711, enabled line 719
```

**What is proposed, and it is not a tick.** *Software* should still be refused —
task 1's own body says FileChooser and every dialog-needing portal are
deliberately unregistered, so an application still cannot open a file, and that
is the honest reason. The proposal is to **replace the reason with the true
one**: not *nothing to talk to*, but *the portals a dialog is needed for are not
answered yet*. A refusal resting on a false premise is one somebody disproves
and then ticks.

### 2. A runtime in the image, and where applications may come from

**Status:** blocked — on a machine. **Depends on:** 1.

**Built, found 2026-10-02**, and this line read `ready` until 2026-10-03. The
evidence is in *What was found on 2026-10-02* above; the status is what the
supervisor reads, and it disagreed with the finding one screen above it.

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

**Five of the six below are now wired — measured on `main` 2026-10-03, and
what is left of this task is one protocol and a machine.** Counted by delegate
macro, which is what makes smithay answer a protocol rather than a mention of
its name: `delegate_dmabuf`, `delegate_data_device`, `delegate_xdg_decoration`,
`delegate_text_input_manager` and `delegate_viewporter` are all present in
`crates/alo-shell/src`. **`delegate_presentation` is not, anywhere in
`crates/`.** The list below is kept as written, with the reasons intact, because
the reasons are why each was wanted and they outlive the wiring.

**A caution for whoever finishes it.** `alo-shell` has its own `presentation`
module and the word appears throughout it, all of it about native drawing
targets and none of it about `wp_presentation`. Searching for the word finds the
wrong thing; searching for the delegate finds the right one. This file has now
had five status lines that disagreed with the code, so the habit this plan most
needs is measuring before building.

The original statement, kept because it is what the task was written against:
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

**The finding that constraint asked for, 2026-10-02 — and what came of it.**
It was recorded as the one item that was invention rather than wiring, the
question was put to the owner, the owner ruled the same day, and it is **built**.
Two of the facts it was argued from were wrong, and both are corrected below
rather than quietly dropped, because another lane reading the old version would
skip work that is now done.

Four of the six were wired or in flight — the clipboard, the frame decoration,
the drawn size, and text a person did not type. Each was a field on `Surfaces`,
a line in its constructor, a handler and a delegate. `linux-dmabuf` needed more
than that, and the part that was genuinely not wiring was real:

```text
DmabufState::create_global(display, formats)   must advertise the formats the
                                               renderer can actually import
DmabufHandler::dmabuf_imported(.., notifier)   must try the import and tell the
                                               client whether it worked
Surfaces, Server                               hold no renderer at all
```

**So the handler had nothing to import into, and that much was right.** Every
other protocol's state is answerable from what `Surfaces` already holds. This
one needs a renderer at two separate moments — when the global is created, to
say which formats are offered, and at every import, to say whether this buffer
worked. That is a question about where the renderer lives, which is
architecture, so it was written here instead of decided by the lane that found
it.

**The owner's ruling, 2026-10-02.** The renderer is owned by the graphics
backend and kept alive in its state, reachable through a narrow interface for
capability discovery and buffer-import validation. It is initialised before
DMA-BUF is advertised. A successful import is reported only after a real one,
and failure is never discovered at draw time. No upstream fork, and no
unconditional acceptance.

**What that is, in this crate.** `crate::direct_target::ScenePainter` gained two
methods and nothing else — `importable_formats` and `validate_import` — so the
renderer stays where it was and answers two questions through a door the width
of those questions. `Surfaces` holds `DmabufState` from birth but **no global**;
the global is created in the graphics backend's own setup, one line after the
renderer is built, from the renderer's own format set. `dmabuf_imported` decides
nothing: it holds the offer in `crate::buffers_clients_hand_over`, and
`direct_loop::run_with_input` drains it between dispatching clients and drawing,
which is where both the server and the renderer are in hand.

**Two corrections to the facts above, both named rather than deleted:**

1. **"`linux-dmabuf` is the one that is invention."** It was a decision, not an
   invention. Once the decision existed the work was wiring after all — about
   290 lines, no new dependency, no fork, nothing patched.
2. **"The workspace constructs exactly one renderer, `winit::init_from_attributes`
   in `nested.rs`, and the direct path builds none."** Wrong, and wrong by
   construction: the grep behind it looked for `GlesRenderer::new` and
   `winit::init`, which cannot find a renderer that is neither. The native path
   builds a **`PixmanRenderer`**, in `software_scanout::SoftwarePainter::new`,
   called from `direct_desktop` and `direct_sign_in`. A machine booting to
   `alo-compositor` does have a renderer; the earlier measurement raised that
   question and then answered it incorrectly.

**And the reason that second error mattered.** `crate::software_scanout`'s own
header said importing is the half of a renderer it does not have, so the
conclusion drawn here was that DMA-BUF had nowhere to land. **`PixmanRenderer`
implements `ImportDma` and `ImportDmaWl`.** The import is therefore safe code on
a renderer this workspace already builds — no `unsafe`, which the root
`Cargo.toml` forbids outright, and no fork. That header is corrected in the same
change.

*Still open and not claimed by this: a client window is imported here but not
yet **composed**. `software_scanout::paint` still refuses every client surface by
name. Importing a buffer and drawing a session from it are different halves, and
only the first is done.*

### 4. A terminal, rented rather than written

**Status:** blocked — on a machine. **Depends on:** 3.

**Built, found 2026-10-02**, and this line read `ready` until 2026-10-03. The
evidence is in *What was found on 2026-10-02* above; the status is what the
supervisor reads, and it disagreed with the finding one screen above it.

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

**Status:** blocked — on a machine. **Depends on:** 3.

**Built, found 2026-10-02**, and this line read `ready` until 2026-10-03. The
evidence is in *What was found on 2026-10-02* above; the status is what the
supervisor reads, and it disagreed with the finding one screen above it.

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
