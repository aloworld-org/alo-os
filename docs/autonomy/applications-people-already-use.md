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

### 1. The portal backend, running

**Status:** ready. **Depends on:** nothing.

A D-Bus service answering the `org.freedesktop.portal.*` interfaces and routing
every request to `alo-portals`, which already knows how to judge one. Without
it a sandboxed application installs and can then do nothing at all: not open a
file, not save one, not be notified.

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
