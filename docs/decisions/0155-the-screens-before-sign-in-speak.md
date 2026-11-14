# ADR 0155 — the screens before sign-in speak

**Status:** **accepted, 2026-11-14, by the owner**, in two words relayed by the
coordinator. Told that none of the first-start screens (language, name, PIN or
password, the time-zone question) and none of the sign-in screen can be read by
a screen reader — so **a blind person cannot set up an alo machine, or sign in
to one, without somebody sighted beside them** — the owner said: *"Build it"*.
The decision to make them speak is the owner's. The shape below — which login
the reader runs as, how the screen's contents reach it, how it is turned on and
how it hands over — is the lane's, and stands until the owner says otherwise.
**One question is the owner's and is not decided here**: whether the reader is
on when a machine first starts (*The question for the owner*, below).
**Date:** 2026-11-14

## What it changes for the person

ADR 0151 gave alo a screen reader that starts **after** somebody has signed in.
The first thing a new machine shows — choose your language, your region, your
keyboard, your name, your PIN — and the sign-in screen every day after that
were silent, and invisible to a reader. A blind person could not get as far as
a desktop.

After this:

- On the **first-start screens and the sign-in screen**, pressing
  **`Ctrl`+`Alt`+`S`** or the **Accessibility** button turns the screen reader
  on. It says that it is on, then reads where the keyboard is: each choice with
  its name, whether it is selected and its place in the list, each field and
  what it is for, and every sentence the screen says when something goes wrong.
- It speaks **the language chosen on the first screen**, from the moment it is
  chosen.
- **Nothing it says leaves the computer**, exactly as ADR 0151 promises for the
  desktop.
- Turned on during first start, it **stays on in the account that is made**:
  the person's own *Screen reader* switch in Settings starts on, and their
  desktop reads from the first frame. They never have to find the switch twice.
- At sign-in the screen's reader stops and the person's own starts, if their
  switch is on. Two voices never talk at once.

## The decision

### 1. The reader runs as the sign-in login — never as root, never as a person

The machine already has a login for exactly this: **`alo-greeter`, uid 60990**
(`image/usr/lib/sysusers.d/alo.conf`, ADR 0024) — *it signs in nowhere; it is
the screen people sign in at*. Until now it was only the group the compositor
runs in. It becomes the login the pre-sign-in reader runs as, the shape GDM
gives its own `gdm` login.

- **It gets a manager of its own**, `user@60990.service`, started at boot
  because the login *lingers* (`/var/lib/systemd/linger/alo-greeter`, written
  by a `tmpfiles.d` line on every start, since a bootc image's `/var` is the
  machine's and not the image's). That manager gives it what ADR 0151 found
  missing before sign-in: **a session bus, an accessibility bus** (at-spi2's
  launcher, activated by that bus as on every desktop) **and PipeWire with its
  PulseAudio socket** — the user units every person's manager already runs.
- **Sound**: the login is put in the base's `audio` group, which udev gives the
  sound devices. A login with no logind session is given no seat's device
  access, so the group is how it reaches the speakers. **The cost, in plain
  words: the sign-in login can open the microphone as well as the speakers.**
  Nothing it runs records anything — it runs Orca, Speech Dispatcher, eSpeak NG,
  PipeWire and alo's own relay (below) — and GDM's login has the same reach
  through the seat. Restricting its PipeWire to playback is a configuration a
  later change can add; this one does not.
- **Orca, Speech Dispatcher and eSpeak NG are ADR 0151's own**, unchanged, and
  so is the unit that starts Orca: `alo-screen-reader.service` has no
  `[Install]`, so it runs in whichever manager is asked to start it, and here
  the relay below asks. The relay's own unit carries
  `ConditionUser=alo-greeter`, so no person's manager runs it; and the two
  person's units enabled in every manager — the keyring and the folders —
  carry `ConditionUser=!alo-greeter`, so the sign-in login's runs neither.

### 2. The compositor never touches that login's buses — a relay does

The sign-in screen is drawn by `alo-compositor`, which runs **as root with no
capabilities at all** (`CapabilityBoundingSet=` in its unit). That second half
matters: root without `CAP_DAC_OVERRIDE` cannot enter `/run/user/60990`, which
is 0700, and a D-Bus bus admits its own uid and nobody else unless its policy
says otherwise. **Both are the right answer**, and neither is changed:

- A small program of alo's own, **`alo-reading-the-sign-in-screen`**, runs in
  the sign-in login's manager. It is the only thing on that login's
  accessibility bus that speaks for the screen: it serves the tree there with
  the same `access_*` machinery the desktop serves its own with (#1202, #1203,
  #1238), and hands back what a reader pressed.
- **The compositor and the relay meet on one socket**, which the compositor
  binds in a runtime directory that is `0750 root:alo-greeter`. The relay
  connects to it; the compositor reads the kernel's word for who connected
  (`SO_PEERCRED`) and **refuses any uid but 60990** — the same test, by the
  same mechanism, that `alo-sessiond`'s door applies (ADR 0024). The relay in
  turn refuses a socket whose other end is not uid 0.
- **What crosses that socket is data, never code**: the compositor sends the
  tree whole whenever it changes (roles, names, states, a choice's place in its
  list, where the keyboard is) and the language; the relay sends back the path
  of a node a reader pressed. **A press is one the screen already accepts from
  a key** — choose a row, go on, go back — and the compositor answers it
  through the function the key takes. There is no verb, no command and nothing
  a reader could ask the compositor for that a person at the keyboard could not.
- **What never crosses it: anything typed.** A name field says whether it is
  empty; a PIN or password field is a *password field* and says nothing about
  what is in it, not even its length — the rule the password sheet already
  keeps (`the_password_sheet_as_read.rs`). No `Text` interface is served.

So **no privileged code runs as the reader's login**, and the reader's login
can reach nothing the compositor holds but one socket that takes presses.

### 3. How a blind person turns it on, before anything is chosen

- **`Ctrl`+`Alt`+`S`**, which the design proposes
  (`docs/design/the-first-start.md`, frames `375:30435` and `384:26446`: *Press
  Ctrl + Alt + S at any time to turn the screen reader on or off*). On the
  screens before sign-in it collides with nothing — no application is running,
  and the desktop's shortcuts are not served there — and the design's own check
  that it is free in the desktop's shipped set stands
  (`crates/alo-shortcuts/src/defaults.rs`, measured 2026-10-06). **This ADR
  does not register it on the desktop**; that is ADR 0151's open item and its
  own priced `Action`.
- **The Accessibility button**, bottom left on every first-start sheet and on
  the sign-in screen, which has done nothing since it was drawn. **The design
  draws what it opens**: `375:30435`, *Accessibility · Setup options* — Screen
  reader, Text size, More options, *Return to setup* — and `384:26446` with the
  reader on. Until that sheet is built, the button turns the reader on and off
  directly and says so aloud, and the gap is named where it is built rather
  than a different sheet being invented.
- **Orca's own `Super`+`Alt`+`S`** (GNOME's) is not added: the design names one
  chord, and two would be a second promise nobody made.
- **Turning it on is said aloud**: the first words are that it is on, in the
  chosen language, then the heading and where the keyboard is. Turning it off
  is said too, before the voice stops — the design's *Ctrl + Alt + S turns it
  off* is only true if somebody hears that it did.

### 4. What it speaks in

**The language chosen on the first screen, from the press that chose it**, as
the screen already redraws in it (the owner's *from the first minute*,
2026-10-28). The compositor sends the language with the tree; the relay writes
`LC_ALL` for it and starts Orca again, because Orca reads its language once,
when it starts. Before a language is chosen it speaks the language the screen
is drawn in. On the sign-in screen it speaks the machine's language, kept from
the first start. All 24 have a voice (ADR 0151 §5).

### 5. What leaves the machine: nothing

ADR 0151 §3 applies whole: Orca and Speech Dispatcher run under
`RestrictAddressFamilies=AF_UNIX`, so neither can open an internet socket, and
only the eSpeak NG module is loaded. **The relay is held to the same line**: it
needs a Unix socket to the compositor and its own login's buses, and is
allowed nothing else. The build checks that every unit the sign-in login runs
for the reader carries it.

### 6. Handing over at sign-in

- **When a session opens, the sign-in reader stops first.** The compositor
  tells the relay *a person is signing in*; the relay stops Orca and Speech
  Dispatcher in its own manager, and PipeWire there lets go of the sound
  device. Only then does the person's desktop start their own reader — so two
  voices never speak, and the person's PipeWire is not refused a device the
  sign-in login was holding.
- **The person's own switch decides from there**, exactly as ADR 0151 §4 says.
- **Turning it over at the sign-in screen is kept in the machine's copy** of
  the accessibility settings (`alo_access::keeping::THE_MACHINES_FOLDER`,
  written by `alo-access` and nobody else, ADR 0038), so this machine's
  sign-in screen still reads after a restart. The machine's copy and the
  person's are already two files for exactly this reason.
- **A reader turned on during first start is on in the account made there.**
  The compositor may not write in a person's home, so the account's file is
  not written for them; instead **a person who has kept no accessibility
  settings of their own starts with the machine's**
  (`alo_access::keeping::for_this_person`), in their desktop and in Settings.
  That is the design's *accessibility preferences survive setup and carry into
  the account*.
- **And a person's own choice is never overridden.** The moment they keep any
  accessibility setting of their own, theirs is what is read — so turning the
  reader on at the sign-in screen does not change the switch of anybody who
  has set theirs.

### 7. When it cannot speak

Each of these is said **on the screen**, beside the Accessibility button, in the
person's language, and in full on the journal for whoever maintains the
machine. Nothing pretends the reader is on. The sentences are ADR 0151's own
(`alo_shell::TheScreenReaderAnswered`) wherever one fits, and a new one only
where none does:

| What happened | What the screen says |
|---|---|
| The sign-in login's manager, its bus or the relay is not there | *did not start* |
| Orca or its unit is missing | *not on this computer* |
| Orca, or its speech module, fails at the start | *did not start* |
| No sound device | a new sentence: *on, with nothing to play it on* |
| The reader falls over later, at any time | *did not start* — the desktop's late-failure watch (#1215), the same rule |

**The choice stays as the person made it** (law 5): a reader that could not
start is tried again the next time the screen comes up.

## What is built, and what is not yet

*Added as the stages landed; the decision above is unchanged by it.*

- **Built** (2026-11-14, #1244 and the change after it): the screens' tree,
  the door and what crosses it, the relay, `Ctrl`+`Alt`+`S` and the
  *Accessibility* button turning the reader over, the language following the
  first screen, the hand-over at sign-in, the machine's copy and the new
  account starting from it, and the image's login, linger, home, sound group
  and units.
- **Not yet**: the sentences in §7 are said **on the journal only** — the
  screen does not yet draw them beside the button, so a sighted helper sees
  nothing when the reader fails; the *no sound device* case is not told apart
  from *did not start*; turning the reader **off** is not said aloud before
  the voice stops; a reader's own press (Orca's *activate*) does not reach the
  screen; and the design's *Accessibility · Setup options* sheet is not built.

## The question for the owner

**Is the reader on when a machine first starts?** Not decided here.

- **On**: a blind person hears the first screen with nobody's help. Everybody
  else hears it too, at a moment the machine is new and its volume is whatever
  the factory left — and a sighted person has to find out how to make a talking
  machine stop.
- **Off, with the way to turn it on said aloud** — the shape macOS and Windows
  use at setup: after a pause with no key pressed, the first screen says one
  sentence, *to hear the screen read aloud, press Control, Alt and S*. **The
  lane's recommendation.** It reaches a blind person with nobody's help and
  costs a sighted person one sentence.
- **Off and silent**: what alo does today. A blind person needs to know the
  shortcut before they start.

The second is a new promise — a spoken sentence while the reader is off — so it
is the owner's to make, and none of the three is built until it is answered.
Until then the reader is off as a machine comes, which is what it already is.

**Answered 2026-11-14 by the owner, relayed by the coordinator: *"Off, with
one spoken hint"*.** On a new machine's first screen, after a short pause with
no key pressed, the machine says once, in the first screen's language: *"Press
Control, Alt and S to hear the screen read aloud."* As built
(`crates/alo-shell/src/the_spoken_hint.rs`, the relay's `says_how_to_hear_it`):

- the pause is **ten seconds**, a named constant (`THE_PAUSE`), with nothing
  pressed — **a key or a click** cancels it for good;
- **only on the first-start sheets**: a machine that already has an account
  shows the sign-in screen and never says it;
- **never while the reader is reading**, and **once a boot**: the relay that
  says it runs for the whole boot and says it at most once, so a sign-in
  screen started again does not repeat it;
- it is said by **eSpeak NG alone**, as the sign-in login, in the voice for
  the screen's language (English where eSpeak has none); **Orca is not
  started** for it;
- **a machine with nothing to play it on says nothing**: the voice program
  fails, that is said on the journal, and nothing else changes.

## What this does not reach

- **Text size and More options on the Accessibility sheet** (`375:30435`); a
  larger-text or high-contrast sign-in screen is its own work.
- **Braille**, for the reason ADR 0151 gives.
- **`Ctrl`+`Alt`+`S` on the desktop**, ADR 0151's open item.

## What only a booted machine can confirm

1. The sign-in login's manager comes up at boot, with its session bus, its
   accessibility bus and its PipeWire.
2. PipeWire running as that login, with no logind session, opens the speakers
   through the `audio` group. WirePlumber's seat handling is the part that
   **fits the reading and is not confirmed**.
3. SELinux lets that manager run Orca and the relay, and lets the relay connect
   to the compositor's socket.
4. Orca reads the first-start sheets as the keyboard moves, in the chosen
   language, and stops at sign-in before the person's reader starts.
5. With the reader on at the sign-in screen, the network boundary shows nothing
   leaving (law 1's measurement).
