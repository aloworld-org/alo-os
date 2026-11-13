# ADR 0151 — alo ships a screen reader, and its voices stay on the machine

**Status:** **accepted, 2026-11-12, by the owner**, in two words relayed by the
coordinator: *"Ship Orca"*. The decision — a screen reader in the image, started
and stopped by a **Screen reader** switch — is the owner's. The package set, the
sizes, the voice table and the configuration below were measured and written by
the lane that built it, and stand until the owner says otherwise.
**Date:** 2026-11-12

## What it changes for the person

Until now a blind person could not use alo OS at all: the shell put its controls
on the accessibility bus, and nothing on the machine read them aloud. After
this, **Settings → Accessibility has a *Screen reader* switch**. Turned on, the
screen is read aloud in the language the person reads, from that moment and at
every sign-in after, until they turn it off. **The voice is made on the
computer, and nothing it says is sent anywhere.**

What it does **not** do yet is speak on the sign-in screen, before anybody has
signed in (*What this does not reach*, below).

## The decision

### 1. Three pinned upstream engines, configured and never patched

| Engine | What it is | Version measured |
|---|---|---|
| **Orca** | the screen reader: reads the accessibility tree and decides what to say | 48.9 |
| **Speech Dispatcher** | the layer Orca speaks through | 0.12.1, with only its eSpeak NG module |
| **eSpeak NG** | the voice: turns text into sound, offline | 1.51.1 |

They are the base's own packages (`fedora-bootc:42`, pinned by digest), added in
one `dnf install` in `image/Containerfile` with `install_weak_deps=False` like
every other. A distribution package is held by **a floor and a start**, not by
an exact release, for the reason the recipe gives for WirePlumber: pinning one
release would pin the base's security updates out with it. The build **starts**
Orca (`orca --version`) and makes eSpeak NG speak a word in each of the 24
languages to a file, because *`test -x` stood in for a converter starting* is
the first fault `CLAUDE.md` names.

**Measured, not assumed, and it mattered at once.** With the four packages and
nothing else, Orca does not start: it imports `dbus` and `python3-dbus` is only
a weak dependency, so `orca --version` ends in *ModuleNotFoundError: No module
named 'dbus'*. The recipe names `python3-dbus`, and the start check is what
would catch the next one.

### 2. Orca is Python, and that is allowed

`CLAUDE.md`: *one language: Rust … pinned engines are the exception, and an
engine is configured, never written in.* Orca is an engine in exactly that
sense — an upstream screen reader that alo installs, starts and stops. **No line
of alo is written in Python**, nothing alo ships imports Orca, and nothing in
this repository changes Orca's code. What alo writes is a unit file and a switch.

### 3. What leaves the machine: nothing, and the unit makes it so

**Speech is made on the machine.** eSpeak NG is a formant synthesiser with no
network code; Speech Dispatcher talks to Orca over a socket in the person's
runtime folder, not over the network (`CommunicationMethod` is left at its
default, `unix_socket`).

**No network voice is loaded, and the build refuses one.** Speech Dispatcher
ships configuration for other voices, and two of them reach a server:
`mary-generic` and `mimic3-generic` fetch speech with `curl` from an HTTP
server. Its own `speechd.conf` loads exactly one module, `espeak-ng`, and the
build fails if any other `AddModule` line is uncommented, so a later base that
loaded another could not ship.

**Orca has one road to the network, found by reading it, and the unit closes
it.** `orca/speech_generator.py`, `_generate_link_file_size`, opens a link's
address with `urllib.request.urlopen` to say how large the file behind it is —
a request to whatever server the link names, made when the person asks where
they are on a link. **That is egress a person did not ask for.** Patching it out
is forbidden; configuring around it is not: `alo-screen-reader.service` says
`RestrictAddressFamilies=AF_UNIX`, so Orca and anything it starts can open no
internet socket at all. The request fails, Orca catches the failure, and the
file's size is simply not said. **That is the cost, in plain words: Orca does
not tell a person how large a linked file is.** The same line is on Speech
Dispatcher's own unit, for the case where it is started by its socket rather
than by Orca.

`crates/alo-image/tests/the_screen_reader_speaks_on_this_machine.rs` holds the
unit, the drop-in and the recipe's checks to this on every build of the gates.
**What it cannot show is a machine measured at its network boundary** with the
reader running; that is a boot walk (below).

### 4. How it is started: by the person's session, as the person

- **`alo-screen-reader.service`, a user unit**, run by the person's own systemd
  manager — never by PID 1 and never as anyone else. Orca is the distribution's
  binary, and `crates/alo-image/tests/a_unit_starts_what_its_manager_may_execute.rs`
  already holds that kind of unit to the user directory.
- **It has no `[Install]` section.** Nothing starts it but the desktop.
- **At sign-in** the desktop reads the person's `access.toml`. Switch on: it
  asks the manager to start the unit. Switch off: it asks it to stop, which is
  nothing on a fresh session and stops a reader left over from an earlier one.
- **When the switch is turned over** in Settings, the desktop is told on the
  next frame and starts or stops it then.
- **A start is watched for two seconds**, so a reader that starts and falls over
  is reported as not started rather than as on. Each answer that is not *done*
  is said in Settings under the switch, in the person's language, and on the
  journal in full. **The switch stays as the person set it** — law 5 — so a
  reader that could not start is tried again at the next sign-in.
- **An agent cannot reach it.** There is no verb for it; it is the person's
  switch, the same as every other on that page.

**Not in the record**, which matches *Reduced motion* and every other switch on
the Accessibility page: the record is what alo did and what the person
approved, and a setting the person turns over by hand is neither.

### 5. Which languages have a voice: all 24

Measured 2026-11-12 in the pinned base, `espeak-ng --voices` with eSpeak NG
1.51.1, and **each one made to speak** a word to a file (every one returned 0
and wrote 23–30 KB):

| | | | |
|---|---|---|---|
| Bulgarian ✓ | Croatian ✓ | Czech ✓ | Danish ✓ |
| Dutch ✓ | English ✓ | Estonian ✓ | Finnish ✓ |
| French ✓ | German ✓ | Greek ✓ | Hungarian ✓ |
| Irish ✓ | Italian ✓ | Latvian ✓ | Lithuanian ✓ |
| Maltese ✓ | Polish ✓ | Portuguese ✓ | Romanian ✓ |
| Slovak ✓ | Slovenian ✓ | Spanish ✓ | Swedish ✓ |

**All 24 have a voice. None of them is a good one.** eSpeak NG is intelligible
and robotic in every language; it is what most Linux screen-reader users hear,
and it is small, offline and covers all 24, which no neural voice set does
today. A better voice is a later decision with its own size and its own
measurement. `alo_access::voices::A_VOICE_FOR_EACH` is the table in code, and the
recipe's loop is what makes each one speak on every image build.

### 6. Where the sound goes: a PulseAudio server on PipeWire

Speech Dispatcher as Fedora builds it has three outputs — ALSA, libao and
PulseAudio, **no PipeWire** — and its default is PulseAudio. **The image had no
PulseAudio server**: measured with the recipe's own package list,
`pipewire-pulseaudio` was not installed. So the recipe adds it (447 KB). Its
socket is enabled in every person's manager by the package itself, measured with
`systemctl --global is-enabled pipewire-pulse.socket`.

**What else this changes, said plainly:** every application that plays sound
through PulseAudio now has somewhere to play it — which is every Flatpak
application, since Flatpak gives applications PulseAudio's socket. Before this,
by the same measurement, none of them could have made a sound. That makes the
existing promise *media playback* true rather than adding one.

### 7. The accessibility bus, named rather than inherited

`at-spi2-core` was in the image only because something else depended on it. It
is the bus Orca reads and the shell serves its tree on, so the recipe now names
it and checks that its launcher is there and that the session bus can start it
(`org.a11y.Bus`'s activation file). It is started by D-Bus when the shell first
asks for it, as at-spi2 is on every desktop.

### 8. Size

**About 83 MB installed**, measured 2026-11-12 by installing the recipe's own
package list in the pinned base and then these packages on top: 23 packages,
82,356,759 bytes. The largest are Speech Dispatcher (29.7 MB), Orca (19.6 MB),
eSpeak NG (12.7 MB) and liblouis's braille tables (12.7 MB, which Orca needs and
which will matter when braille does). `pipewire-pulseaudio` adds 0.45 MB. **What
it adds to the image a person downloads is compressed and is measured only by
building the image**, which this change did not do.

## What this does not reach

- **Found on the first booted machine, 2026-11-13: Orca does not start at all
  on this base.** Orca 48.9 reads an X display it is never given, and
  at-spi2-core 2.56.8's keyboard asks X for its modifiers and crashes when there
  is none; both are fixed in Fedora 43's engines, and no configuration of the
  pinned Fedora 42 base starts Orca without an X server. The roads, and what was
  measured against each, are in
  `docs/quirks/orca-on-wayland-needs-an-x-display-twice.md`; choosing one is the
  owner's. Until then the switch says *did not start* when the reader falls, and
  the image build stops at the check that starts Orca as the session does.

- **The sign-in screen does not speak.** The shell serves no accessibility tree
  before sign-in, and the sign-in screen runs with no session bus, no
  accessibility bus and no sound for a reader to use. Speaking there needs all
  three for an unprivileged sign-in user — the shape GDM gives its own greeter —
  and that is its own piece of work. **The *Accessibility* button on the sign-in
  screen therefore still does nothing**; a switch there that turned on a reader
  which could not speak would tell a blind person something false at the one
  place they cannot check it.
- **Orca's own keyboard commands may not work over alo's surfaces.** Orca 48
  hears the keyboard through libatspi's `Atspi.Device`, which on Wayland needs
  the compositor to pass key presses to the accessibility bus. alo's compositor
  does not do that yet. What should work without it is the reader following the
  keyboard and speaking what is focused and what changes; Orca's commands — read
  the whole window, where am I, stop speaking — may not. **This fits the code
  and is not confirmed**: a booted machine says which.
- **`Ctrl`+`Alt`+`S`**, which the design proposes for turning the reader on and
  off from anywhere (`docs/design/the-first-start.md`), is not built. A new chord
  is a new `Action` and is priced in `alo-shortcuts`; it is a separate change.
- **Braille** is not offered. Orca supports it and liblouis is installed with
  it; whether alo offers it is a decision nobody has taken.

## What only a booted machine can confirm

1. Turning the switch on makes the machine speak, in the language the person
   reads, through the speakers PipeWire chose.
2. Orca finds the shell's tree and the applications', and reads focus as the
   keyboard moves.
3. Turning it off stops speech at once, and a sign-in with it on starts it.
4. With the reader on and a browser open, the network boundary shows nothing
   leaving on the reader's account (law 1's measurement).
5. Which of Orca's own keyboard commands work.
