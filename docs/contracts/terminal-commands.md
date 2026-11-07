# Contract — commands a person types to use their applications

**Status: draft, 2026-11-06. Nothing in it is built, and nothing in it is
accepted.** It is the contract `docs/features.md` says must be written *before*
the commands are built (the promise *Everything on the computer, from the
terminal*, in pull request #1101, which had not landed on `main` when this was
written). Every question the owner has to answer is under *Open questions*, at
the end, each with a recommendation. Until the owner has answered them, nothing
may be built against this page.

## What this gives a person, in one paragraph

From any terminal, a person can ask what applications are installed and what
each one can be asked to do, and then do one of those things by typing it — open
a document in the text editor, list every button and menu an application's
window shows and press one by its name, and later send a message or edit a
picture. This is one of the *three equal roads* (#1101): what can be clicked on
the screen can be typed in the terminal and asked of alo, as a guarantee for
alo OS's own surfaces and as far as each other application describes itself. **A typed command runs at once, because the
person typed it**, with no OK to give, including one that sends something off
the machine. It is written in the person's record, it is undone with **Put
back** where that is possible, and what it sends is shown on the indicator. The
other way in, plain words through `alo-ask`, uses **the same list** of what an
application can do, and waits for the person's OK unless they switched that off.

## What this rests on

Read these before arguing with any of it; this page is what they say, applied.

- **The promise**: `docs/features.md`, *Everything on the computer, from the
  terminal* and *alo in the terminal may act without asking, if the person
  chooses* (#1101).
- **ADR 0001 §6** — an application is reached through four roads, best first:
  its own automation interface, the accessibility tree, D-Bus, and pictures
  with synthetic input as a last resort. And the safety rule: **an adapter
  exposes typed verbs and builds any script itself, from validated arguments.**
- **ADR 0124** — `alo-ask` is the one command for plain words, and it both
  answers and acts.
- **ADR 0143** — alo works inside any installed application; what exists, what
  does not, and the owner's answers of 2026-10-31.
- **ADR 0125** — a command does not report its own egress; and a person's own
  runs are written to a record file of their own.
- **ADR 0113** — the terminal's gutter says who is speaking, decided by where
  the bytes came from.
- **ADR 0127, ADR 0130** — the place kept before a person's own command, and
  `alo-back`.
- `docs/contracts/agent-verbs.md`, `app-adapters.md`, `application-door.md`,
  `record-file.md`, `daemon-protocol.md`, `shell-integration-marks.md`.

**Law 2 binds the agent, not the person.** That is why the person's commands
need no grant and no approval. It is *not* a reason for the commands to accept
a script: the adapter rule above is about what a program on this machine hands
to an application's interpreter, and it holds whoever typed the command (see
*Arguments*).

## 1. The command shape

### One command, for every application

There is one program, **`alo-app`**, and it knows nothing about any particular
application. What it can do with an application comes entirely from the list
described in section 2 — the same list alo itself is offered. Nobody writes a
command per application; an adapter author who declares a verb has, by doing so,
given the person a command.

*The name is a proposal (question 1).* It follows `alo-ask`, `alo-try` and
`alo-back`, and it is free: no file in this repository uses it.

```text
alo-app list                                  what is installed, and what each offers
alo-app show <application>                    everything one application can be asked to do
alo-app show <application> <ability>          one ability: its arguments, and what it changes
alo-app <application> <ability> [arguments]   do it

alo-app <application> controls                every control its windows show now, by kind and name
alo-app <application> press <kind> <name>     press one of them, as a click would
alo-app <application> read                    what its windows show, as text
```

The second group is **the screen's road, in the terminal**: whatever button,
tab, check box or menu item a person could click in a window, they can list and
press by the name it shows. It is not a fallback beneath the first group; it is
the shape that makes *three equal roads* true for every application that
describes its window, and the first group is the richer one beside it for an
application that offers more (section 2).

Examples a person would type:

```text
$ alo-app list
$ alo-app show texteditor
$ alo-app texteditor open_document ~/Notes/today.txt
$ alo-app papers controls
$ alo-app papers press button "Print"
$ alo-app alo press switch "Wi-Fi" --in settings
$ find ~/Pictures -name '*.png' | alo-app loupe open -
```

### Naming the application

An application is named by **its identifier** (`org.gnome.TextEditor`) or by
**its short name**, which is the last part of the identifier in lower case
(`texteditor`, `firefox`, `dolphin`, `loupe`). Never by the name its menu entry
shows (*Text Editor*), because two applications may both call themselves that,
and the one acted on must be the one meant — the rule `agent-verbs.md` already
keeps (*the identifier is approved; the name is only shown*).

When two installed applications share a short name, the short name is refused
and both identifiers are listed, so the person picks. Nothing is guessed.

### Naming the ability

An ability is named exactly as it is declared: `open_document`,
`activate_control`. These are the names the record keeps and the names alo uses,
so one thing has one name everywhere. They are never translated; what *is*
translated is the sentence shown beside them (`agent-verbs.md`, *an option is a
name a model sends and a word a person reads*).

### Arguments

Arguments come in the order the ability declares them, or by name as
`--<argument> <value>`; each is given exactly once. Every argument is required,
as `agent-verbs.md` already says of every verb. An argument is one of the kinds
that contract lists, checked by **the same validation alo's own calls go
through**, so a value alo would refuse is refused here too, in the same words:

| Kind | Typed as | Checked |
|---|---|---|
| path | a path; a relative one is taken from the folder the shell is in | made absolute, with no `..` left in it |
| application | an identifier or short name, as above | installed, and exactly one |
| name | one name | its declared length; no control characters |
| count | a whole number | inside its declared range |
| choice | one option's name | exactly one of the declared options |
| text | *not yet a kind* — ADR 0143 stage 5 adds it | — |

A command that needs to carry a message or a paragraph (an email's body) waits
for the **text** kind. When it arrives, the person may give it as an argument,
or as `-` to read it from the pipe. **A text argument is never handed to any
interpreter by anything on this machine** — the adapter puts it into the
application as text, never into a script — which is what keeps a person's own
email body from becoming a program, whoever typed it.

### Pipes

**A value given as `-` is read from the pipe, one value per line, and each line
is one call.** That is the whole rule, and it is the Unix one: any program that
prints paths one per line can feed `alo-app`, and `alo-app` can feed any program
that reads them.

- **What an ability gives back** is a new, optional part of a verb's
  declaration, proposed here: `gives`, one of the kinds above, or nothing.
  `alo-app` prints what an ability gives one value per line when its output is
  a pipe, and as plain sentences in the person's language when it is a
  terminal. `--plain` and `--json` choose explicitly.
- **One line, one call, one record entry.** Ten pictures piped in are ten calls
  and ten entries, never one call over ten. If one fails, the others still run,
  the failure is said on the error output, and `alo-app` ends with the code of
  the worst thing that happened (below).
- **A value that cannot be read in a sentence is refused on its line** — a
  control character or an escape sequence — exactly as in an argument.

*The features list gives `image holiday.jpg brighten 20 | crop square` as an
example. As a shell reads it, `crop` would be a second program of its own, with
every ability of every application competing for that one name. Under this
contract it is
`alo-app <application> brighten holiday.jpg 20 | alo-app <application> crop - square`
— and no application on a fresh machine offers either ability (section 6).
Short words such as `image` and `mail` are question 3.*

### What it says, and the exit codes

Sentences for a person are in their language, from declared strings. Names and
values for a program are never translated, so a script gives the same result on
a Greek machine as on a German one. The exit code says which of these happened,
in the categories `app-adapters.md` already tells a person apart:

| Code | Meaning | Did anything happen? |
|---|---|---|
| 0 | done | yes |
| 1 | the application refused, and said why | no |
| 64 | the command was not written in this shape | no |
| 65 | a value was refused (wrong kind, out of range, not one of the options) | no |
| 69 | the application is not installed, or does not offer that ability (usually another release of it) | no |
| 75 | the request was sent and the application did not answer | **not known**, and it is recorded as having run |
| 77 | an organisation's policy on this machine forbids it (ADR 0016) | no |

These are the standard `sysexits.h` numbers, so they mean the same thing they
mean to every other Unix program.

### What is a public surface, and how it changes

Stable, and changed only additively; a break needs a new version and a
deprecation period, as `CLAUDE.md`'s *contracts outlive code* requires:

- the name `alo-app` and its forms `list`, `show`, and `<application> <ability>`;
- how an application and an ability are named;
- how arguments are written, and `-` for the pipe;
- the line format of a pipe, and the `--json` shape of `list` and `show`, which
  carries `"format": 1` so a later shape can be told apart;
- the exit codes above;
- the record entry in section 4.

Additive means: new forms, new flags and new fields may appear; an existing one
never changes meaning. **A deprecated spelling keeps working for at least one
release after it is announced**, saying on the error output what replaces it.

**Not stable under this contract: the abilities themselves.** A declared ability
belongs to its adapter and changes under `app-adapters.md`'s *Versioning*: its
name and meaning are fixed once published, and it says which releases of the
application it supports. **A discovered ability (section 2) has no stability at
all**, because it is the application's own and it may change on any update.
`show` says so beside every one, so nobody builds a script on it unknowingly.

## 2. How abilities are discovered

`alo-app list` and `show` read the machine **at the moment the person asks**,
and keep nothing. What an application offers comes from four places, and `show`
marks each ability with where it came from. They are not ranked so that one
replaces another: **the controls are always offered where the tree describes
them**, and the other three add what a press cannot express.

### Declared — an adapter somebody wrote and reviewed

An adapter (`app-adapters.md`) lists typed verbs for one application, over its
own automation interface (`api`) or its D-Bus interface (`dbus`). These are the
best: their effect is declared honestly, their sentence is generated from their
arguments, and they are what alo uses. Today there is one: GNOME Text Editor's
`open_document` and `new_window` (`crates/alo-adapters/src/text_editor.rs`).
A declared ability is richer than a press — it can take a file, a number or a
choice and do one meaningful thing — so where one exists it sits beside the
controls, not in place of them.

### Discovered on D-Bus — what the application publishes about itself

Many applications publish themselves on the session bus under their own
identifier. Where one does, `alo-app` asks it — at the moment of `show`, and
only then — through the **standard** interfaces, never through anything
particular to the application:

- `org.freedesktop.Application` — `Open`, which becomes an ability `open`
  taking paths;
- `org.gtk.Actions` — each action the application lists, with no parameter or
  with one of a simple type (a word, a number, yes or no), becomes an ability of
  that action's name.

**These are offered to the person and not to alo.** `app-adapters.md` refuses an
adapter whose action is chosen by an argument, because alo's list must be one
somebody reviewed; that rule is about the agent, and law 2 does not stretch it
to the person. But nobody has reviewed a discovered ability, so `show` says, in
plain words: *the application describes this itself; alo cannot tell what it
changes or whether it sends anything.* Whether this split is right is
question 4.

Interfaces an application invents for itself are **not** turned into abilities:
their arguments have no types `alo-app` can check, and a person who wants one
already has the shell's own D-Bus tools.

### The accessibility tree — every control a window shows

This is the road that makes the screen and the terminal equal, and it is
generic: it needs nothing written for any application. An application that
describes its window to the accessibility tree — GTK, Qt, LibreOffice,
browsers, Electron started with accessibility on, and alo OS's own surfaces —
gets three abilities, **whether or not it also has an adapter**:

| Form | What it does | Built on |
|---|---|---|
| `controls` | lists every control its windows show at this moment: its kind, the name it shows, and whether it is greyed out, checked or selected | `accessible.read_window`'s walk, printing controls rather than text |
| `press <kind> <name>` | presses one control, exactly as a click would | `accessible.activate_control` |
| `read` | prints what its windows show, as text | `accessible.read_window` |

`--in <window>` narrows any of them to one window by its title, where an
application has several.

**What a person can rely on is the fallback's own promise, unchanged**
(`app-adapters.md`): the tree is read at the moment of the command and kept
nowhere; nothing is found or pressed by position; a password field's contents
are never read; and **nothing is guessed** — a press is refused, and the
person told why, when no control of that kind and name is showing, when more
than one is, when it is greyed out, or when it cannot be pressed from outside
the application.

**Menus are pressed the way a person opens them.** A menu's items are often in
the tree only once the menu is open (unmeasured per toolkit), so
`press menu "File"` then `press menu_item "Save As…"` is the honest road, and
`controls` after the first press lists what opened. `menu` is not yet one of
the fallback's kinds; adding it is additive (question 11).

**What this road cannot yet do, said so it is not mistaken for parity:**

- **type into a field**, choose from a drop-down by typing, or move a slider —
  the fallback presses and reads, and nothing on this machine puts text or a
  value into an application yet (ADR 0143, question 2). A control of those
  kinds is listed by `controls` and marked *cannot be set from here yet*;
- reach anything an application **draws without describing** — a canvas, a
  game, a video — which is listed as *drawn by the application, not described*;
- act **by position**, ever.

**An application with an adapter.** For alo, an adapter's narrower list
replaces the tree, because pressing any control would make a reviewed list
decorative (`app-adapters.md`). For the person, both are offered side by side:
the owner's requirement is that whatever can be clicked can be typed, and law 2
does not bind the person. Whether alo keeps the narrower rule is question 4.

### The window — what every application has

`open`, `focus` and `close`, carried out by the compositor through the door in
`application-door.md`. That door admits the person's own login, so `alo-app` can
use it directly. These work for every installed application with a desktop
entry, whatever else it offers.

### When an application offers nothing

`alo-app show` says it in one sentence, naming what was looked for — never an
empty list, which would look like a fault:

> *Blender can be opened, focused and closed. It publishes nothing alo can use
> on the session bus and describes nothing to the accessibility tree, so alo
> cannot do anything inside it. An adapter would let it.*

And for the applications ADR 0143 names as describing nothing — a game, a 3D or
painting canvas, a remote desktop, a virtual machine — it says *alo cannot work
inside this application* rather than guessing. Pictures with synthetic input
are promised, off by default and switched on per application, and they are not
built; until they are, they are not offered here.

The terminal itself (`app.devsuite.Ptyxis`, or alo's own) is answered *this is
a terminal — type into it directly.*

## 3. Three equal roads, and alo OS's own surfaces

`docs/features.md` (#1101): *whatever a person can do in an application's
window, they can also do by typing a command in the terminal and by asking alo
… For alo's own surfaces this is a guarantee: a control with no command and no
way to ask for it is a bug. For applications alo did not make, it is as far as
each application lets itself be reached.*

### For other applications: as far as each describes itself

The controls road of section 2 is the whole of the mechanism, and it is what
makes parity something alo can promise without writing anything per
application: **every control an application describes is a control `alo-app`
can list and press, and one alo can be asked to press.** Where an application
describes nothing, `show` says so (section 2). There is no promise beyond what
the application publishes, and no pretending: the parity for an application is
exactly the controls `alo-app <application> controls` prints.

### For alo OS's own surfaces: a guarantee, held by a test

alo OS's own surfaces — Settings, the windows' own controls, Places, the Dock,
the status area, the record — are reached under the application name **`alo`**,
with `--in` naming the surface:

```text
$ alo-app alo controls --in settings
$ alo-app alo press button "Put back…" --in record
$ alo-app alo press switch "Wi-Fi" --in settings
```

**This road already half exists, which is why it is the one proposed.** The
shell publishes every surface it draws to the accessibility tree a screen
reader reads, with the role, the name and the state that `alo-access` decides
for each control, and serves the *press* interface on every control a reader
can press (`crates/alo-access/src/tree.rs`, `crates/alo-shell/src/access_bus.rs`).
So a screen reader, the terminal and alo would all reach alo's own surfaces
through **one description** — the rule `alo-access` already states: *if it is
good enough for one it is good enough for the other.* Nothing here invents a
second list of alo's controls.

**The guarantee is kept by a test, not by convention.** Proposed, in the crate
that draws each surface:

1. **Every control drawn is in the tree.** For each of `alo_access::Surface::ALL`,
   the test draws the surface in each of its states, collects every control the
   drawing put on the screen, and collects every node the tree publishes for it.
   A control drawn and not published — or published with no name — fails, naming
   the surface and the control. This is the half that catches a new button
   somebody forgot to describe.
2. **Every control in the tree is reachable by `alo-app`.** The same test hands
   each published control to `alo-app`'s own lookup — kind and name, inside that
   surface — and asserts it is found **exactly once**. Two controls with one
   kind and name in one surface fail, because neither the terminal nor alo
   could tell them apart, and the person would have a control that only the
   screen can reach.
3. **Every pressable control can be pressed from outside.** Each is pressed over
   the bus in the test, and the surface is asserted to have done what a click
   does. A control that cannot be pressed from outside fails.
4. **And alo can be asked for each one.** The agent's road reaches the same
   controls through `accessible.activate_control` (question 12), so the same
   list is the agent's list too; the test asserts the agent's lookup finds what
   `alo-app`'s finds.

Where a surface holds a control that cannot be pressed — a slider, a text
field, a list a person scrolls — the test fails **until** that control's road
exists or the control is named on a short, written list of exceptions in the
test itself, each with the reason and the decision that owns it. An exception
list nobody can grow silently is how *a control with no command is a bug*
becomes a thing a gate refuses.

**Read from the code on 2026-11-06, not run:** `alo_access::Surface` lists nine
surfaces — recovery, sign-in, the desktop, the Dock, the status area, the
approval surface, the record, Settings and the window controls. **Places — the canvas's Places and the World — is not
among them**, so today Places publishes nothing and could not be reached this
way; that is the first thing the test would find. The press interface is served;
the *value* and *text* interfaces are not, so a slider or a field in Settings
could be read and not set. Whether the shell's tree is on the bus of a booted
machine is unmeasured — it needs the accessibility bus, which the image does not
name (section 6).

**Richer commands for alo's own surfaces are additive.** A typed verb such as
setting a Settings value by its key is better than pressing a switch by its
label, and may be declared beside it; the controls road is the floor that the
test guarantees, and a typed verb is never a reason to leave a control out.

## 4. A person's command and alo's request

The two ways in use **one list** of abilities and **one validation**. What
differs is who acted, whether anything waits, and how that is shown and kept.

| | Typed: `alo-app …` | Plain words: `alo-ask "…"` |
|---|---|---|
| Who acted | the person | alo, for the person |
| What it may reach | anything the person can, with no grant | only applications the person has let alo use (ADR 0143, the grant over one application) |
| Which abilities | controls, declared, discovered, window | controls, declared and window — **never discovered** (question 4); controls of an application with an adapter only if question 4 says so |
| Waits for an OK | never | yes, one OK per action — unless the person switched on full trust for the terminal in Settings |
| Where it runs | in the person's own process | in `alo-agentd`, as a turn |
| The gutter in alo's terminal (ADR 0113) | `YOU` for the line, `PROGRAM · alo-app` for its output | `YOU` for the request, `ALO` for the proposal and the answer |
| The record | an entry with no agent in it, in the person's own record | the turn's `ran` entry, naming the agent, the grant and the approval |

**What the screen shows.** The gutter decides who is speaking from where the
bytes came from, and a program cannot write into it — so `alo-app`'s output is
labelled `PROGRAM`, and can never be dressed up as alo's. A plain-words proposal
also shows, under its sentence, the typed command that would do the same thing,
so a person learns the second road from the first. **What the person approves
is still the sentence**, not the command line. In a terminal that is not
alo's own, such as Ptyxis, there is no gutter, and nothing can show the
difference on the screen; the record still can.

**What the record keeps.** A typed command is written by `alo-app` itself, to
the person's own record file — `~/.local/state/alo/commands.jsonl`, the file
`alo-try` already writes — because *a record file has one writer* and the
process that did the work is the one that writes it down (ADR 0125). Proposed
as a new entry, additively, `format` staying `1`:

```json
{"at":"…","happened":{"asked-an-application":{"application":"org.gnome.TextEditor","release":"50.1","ability":"open_document","road":"declared","arguments":{"document":"/home/alo/Notes/today.txt"},"answered":"done","kept_as":null}}}
```

- `road` is `controls`, `declared`, `discovered` or `window`;
- `answered` is `done`, `refused` with the sentence the person saw, or
  `no-answer` — the last recorded as having run, as `app-adapters.md` requires;
- **no `agent` field, and no room for one**: the person acted;
- a command refused before it reached the application keeps the ability and
  why, and **not** the arguments, for `record-file.md`'s reason: what never
  validated is whatever arrived.

Which commands are written down at all is question 5.

### Put back

Before a typed ability that changes something — declared as a change, or
discovered, whose effect nobody knows — `alo-app` asks the service for a place
to return to, with the existing `a-place-to-return-to` request
(`daemon-protocol.md`). It is the person's own process asking, so nothing is
granted or proposed. The entry's `kept_as` then names that place, and the record
window offers **Put back** for it, exactly as it does for alo's changes.

ADR 0127 made the place before a person's command something they switch on per
session, because the shell cannot tell a command that changes things from one
that only reads. `alo-app` can tell, so it can keep a place only before a
change. Whether it does that always, or only in a session that asked, is
question 6.

**What Put back can bring back:** files and folders in the person's home, as
they were before the command — a picture an application edited and saved, a
document it changed and saved.

**What it cannot, said as plainly as the machine must say it:**

- **Anything already sent cannot be undone.** An email, a message, a post, an
  upload: once it has left this machine, nothing on this machine can recall it.
  Put back restores files; it does not reach anybody else's computer. ADR 0110
  §2: *a snapshot … does not recall a push, un-send an email, or undo anything
  that already left.*
- Anything that lives only inside the running application — an unsaved edit, an
  open tab, its window — is not a file, so there is nothing to put back.
- A change outside the person's home is outside the place that was kept.

**Saying so before, not after.** `docs/features.md` promises *say up front
whether it can be undone*. For a declared ability, the adapter's declaration
proposed here gains `sends` — `never`, `always` or `may` — and `show` prints,
beside every ability that does not say `never`, *this cannot be undone once it
has been sent*. For a discovered ability `show` says *alo cannot tell whether
this sends anything*. A typed command still runs at once: the person typed it,
and the warning is where they read before typing — in `show` — not a question
put between them and Enter.

## 5. What leaves the machine

`alo-app` itself sends nothing across the network: everything it says goes to
the application, on this machine, over the session bus, the accessibility bus
or the compositor's door. **The bytes that leave are sent by the application**,
from its own process — the same fact ADR 0143 found for alo (its fact 5), and
it applies to the person's command identically.

The promise says *what it sends fires the indicator*. The indicator today shows
what `alo-agentd` and alo OS themselves send, and nothing an application sends.
So for a typed command this is **not yet true, and nothing in this contract may
be built as though it were**. There are two honest ways to make it true, and it
is question 7:

- **Measured** — for as long as the command runs, the kernel counter that
  already watches a turn's connections is attached to that application's
  sandbox, and the indicator shows *Thunderbird is sending — because of your
  command*. It counts, rather than declares. It also counts anything else that
  application sends in those seconds.
- **Declared** — a line on the indicator, raised for any ability whose `sends`
  is not `never`, saying the application was asked to send. It shows that
  sending was asked for, not what left. ADR 0125 says a command does not report
  its own egress, so the line would be raised by the service that draws the
  indicator, not printed by `alo-app`.

Either way: **counting everything an application does, outside the command, is
not offered**, because that is watching the person, which this product calls a
bug (ADR 0143, the table of settings). And `alo-try --offline` already lets a
person cut a command off from the network with the kernel holding it.

The plain-words road is covered by ADR 0143's own answer (stage 9 of its plan):
by default alo works in a window of its own, and only that window's traffic is
counted.

## 6. What a fresh machine can do with this today

**`alo-app` does not exist.** So today, on a fresh machine, a person can type
none of this. What follows is what it *would* reach on the day it is built, read
from the code and documents on `main` on 2026-11-06. Nothing in this table has
been measured on a booted alo machine; where only one could tell, it says so.

Two facts govern every row:

- **The accessibility tree may not be there at all.** The image names no
  accessibility package (`grep` over `image/` for `at-spi` and `a11y` finds
  nothing), and whether the accessibility bus starts in a signed-in session is
  unmeasured (ADR 0143, fact 10; its plan's stage 1 says *not started*). If it
  does not start, every *read* and *press* below is unavailable, and so is a
  screen reader.
- **Typing into an application is not built**, by any road. So nothing on a
  fresh machine can write an email, put text into a document, or edit a
  picture — whoever asks.

| Application | Window: open, focus, close | Declared | Discovered on D-Bus | Controls: list, press, read |
|---|---|---|---|---|
| Firefox (`org.mozilla.firefox`) | built, not walked on a booted machine | none. Its automation interface runs script, which an adapter may not take | unknown; only a booted machine can say what it publishes | documented yes; unmeasured on alo |
| Dolphin (`org.kde.dolphin`) | as above | none | it answers the shared file-manager interface under a shared name, which a command cannot tie to Dolphin; whatever else it publishes, unknown | documented yes, once Qt sees an assistive technology; unmeasured |
| Ark (`org.kde.ark`) | as above | none | unknown | documented yes; unmeasured |
| Text Editor (`org.gnome.TextEditor`) | as above | **`open_document`, `new_window`** — built and tested, never run on a booted machine | `Open` and its actions, by the GNOME pattern; the list is unmeasured | GTK 4 publishes a tree on Wayland, documented; typing and saving are not abilities |
| Image Viewer (`org.gnome.Loupe`) | as above | none | `Open`, if it publishes itself as GNOME applications do — unmeasured | controls yes, documented; the picture itself is a picture. **No brighten, no crop, by any road** |
| Papers (`org.gnome.Papers`) | as above | none | as Loupe | controls yes; a page's text partly, documented |
| Terminal (`app.devsuite.Ptyxis`) | as above | — | — | answered *type into it directly* |
| **alo OS's own surfaces** (`alo`) | — | none yet | — | the shell publishes nine surfaces with names, states and press (`alo-access`, `alo-shell`), read back over a bus by the shell's own tests (read from the code, not run here); **Places is not one of them**; sliders and fields can be read, not set; never seen on a booted machine's bus |

**And there is no mail application on a fresh machine.** `crates/alo-software/shipped.toml`
names seven, one per role, and none of them sends email. `mail send` therefore
has nothing to reach until the person installs one, and what that application
offers decides what can be typed. The code editor ADR 0143 chose (VSCodium) is
not on that list yet either.

So the honest summary of a fresh machine, once `alo-app` is built: **open,
focus and close any application; open a file in the text editor; probably open
files in the image viewer, the document viewer and the browser; and — only if
the accessibility bus turns out to run — list and press the named buttons, tabs,
switches and menu items of any described window, alo OS's own Settings, Dock,
record and window controls included.** Nothing that types, edits or sends. That
is real parity for what is *pressed* on the screen and none yet for what is
*typed* into it, and the report on this work should say both halves.

What would widen it, in the order it pays: the accessibility bus on the image
(ADR 0143 stage 2), typing into an application (stage 6, waiting on stage 1's
measurement), the text kind (stage 5), and adapters with real verbs for real
applications — LibreOffice, GIMP, a mail client — through their own automation
interfaces, which `app-adapters.md` refuses to load until this machine can
carry one out.

## 7. Open questions for the owner

Each has a recommendation. None is answered by this draft.

**1. What is the command called?**
`alo-app` here; it could be anything. *Recommendation: `alo-app`*, which follows
`alo-ask`, `alo-try` and `alo-back`, and is unused. Not `alo`, which ADR 0124
left unbuilt and which a person might expect to mean plain words.

**2. Is the shape `alo-app <application> <ability> <arguments>` right?**
The alternative is one command per application (`texteditor open_document …`),
which puts every installed application's name on the shell's own list of
commands, where any of them can collide with a program of the same name.
*Recommendation: one command*, with the short words of question 3 as an
optional layer above it.

**3. The features list's examples use `mail` and `image`. Should those short
words exist?**
They would be shell functions in alo's snippet, each meaning *the application
this person chose for that kind of file* — the way `try` is a function that
calls `alo-try`. But `mail` is already a standard Unix program on many systems.
*Recommendation: later, and additively* — define a word only where no command of
that name exists, as `try` is guarded on `alo-try` being there; and correct the
two examples in `docs/features.md` to the contract's form when this is accepted.

**4. Discovered abilities: for the person only, or for alo too?**
*Recommendation: the person only.* The person may use anything the application
publishes, because law 2 does not bind them. alo stays on abilities somebody
reviewed, because `app-adapters.md` refuses an action chosen by an argument for
exactly the reason ADR 0001 exists. If the owner wants alo to use them as well,
that is a change to `app-adapters.md` and ADR 0001 §6, made first.

**5. Which typed commands are written in the record?**
*Recommendation: every change, every discovered ability, and every refusal of
one; not `list`, `show`, or an ability that only reads.* Writing every read a
person makes would be a record of the person rather than of what happened to
the machine, which `agent-verbs.md` avoids for the same reason.

**6. Is a place to return to kept before every typed change, or only in a
session that asked for it?**
ADR 0127 made it something the person switches on per session, because a shell
cannot tell a command that changes things from one that reads. `alo-app` can.
*Recommendation: before every change, always*, because it keeps Put back true
for exactly the commands it can help with, and does not spend the window ADR
0127 was protecting on reads.

**7. How does what an application sends reach the indicator, for a typed
command?**
*Recommendation: measured* — the application's sandbox counted for the seconds
the command runs, with the line naming the command. It is the only form that
shows what actually left, and it watches nothing outside the person's own
command. *Declared* is the fallback for an application whose sandbox cannot be
told apart.

**8. Does the plain-words proposal show the typed command beside its sentence?**
*Recommendation: yes*, below the sentence and clearly not what is being
approved — it is how a person who started with plain words learns that they
can type it next time.

**9. Should `alo-app` refuse to run under the agent's own login?**
If a person has let alo run commands (ADR 0111), alo could type `alo-app` and
reach an application it was never granted. *Recommendation: no refusal* — that
person already let alo run any command, which reaches strictly more — **but**
the entry then names the agent rather than the person, read from the kernel's
answer about who is running it, never from a flag.

**10. Does this contract change `app-adapters.md`?**
It proposes two additive fields on a verb: `gives` (what an ability hands to a
pipe) and `sends` (whether it can send anything off the machine).
*Recommendation: yes, in the same change that first builds `alo-app`*, so that
no field exists before something reads it.

**11. Which kinds of control can be pressed?**
The fallback presses seven kinds — button, check box, radio button, switch,
menu item, tab, link. A menu itself is not one, so `press menu "File"` would
be refused. *Recommendation: add `menu`*, additively, because opening a menu is
how a person reaches every item in it; and treat fields, drop-downs and sliders
as part of the typing work (ADR 0143 question 2), not as presses.

**12. For alo's own surfaces, does alo use the same tree, or verbs of its own?**
The accessibility fallback today reaches only *an application with no adapter*,
by a grant over its identifier, and alo OS's shell is not an installed
application. *Recommendation: the same tree*, reached as `alo` under a grant the
person makes like any other — so the screen reader, the terminal and alo share
one description of every control, and the test in section 3 holds all three at
once. A Settings change alo makes this way is still a change, one sentence and
one OK, unless the person chose full trust.

**13. Should the parity test refuse a control with no road, from the day it lands?**
*Recommendation: yes, with a written exception list in the test* (section 3),
starting with Places and the controls that need typing, each naming the
decision that will close it. A test that only warns is one lanes learn to skip.

## What this page does not decide

How alo types into an application (ADR 0143, question 2); the text kind (ADR
0143 stage 5); the grant over one application and where a person makes it (ADR
0143, question 10); and pictures with synthetic input. Each is its own decision,
and this contract uses whatever they settle.
