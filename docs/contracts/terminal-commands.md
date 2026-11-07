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
a document in the text editor, list every control an application's window
shows, and press, type into, set or choose any of them by its name — and so send
a message or edit a picture in whatever application does that. This is one of
the *three equal roads* (#1101): whatever can be done with hands on a window can
be typed in the terminal and asked of alo, **as a guarantee for every installed
application**. Each control is reached through the best road it offers, and a
window that describes nothing is reached through the screen itself — the least
reliable road, marked as such in the record. **A typed command runs at once, because the
person typed it**, with no OK to give, including one that sends something off
the machine. It is written in the person's record, it is undone with **Put
back** where that is possible, and what it sends is shown on the indicator. The
other way in, plain words through `alo-ask`, uses **the same list** of what an
application can do, and waits for the person's OK unless they switched that off.

## What this rests on

Read these before arguing with any of it; this page is what they say, applied.

- **The promise**: `docs/features.md`, *Three equal roads: the screen, the
  terminal, and asking alo*, *Everything on the computer, from the terminal*
  and *alo in the terminal may act without asking, if the person chooses*
  (#1101), as it stood after the owner's third decision of 2026-11-06 — parity
  as a guarantee for every installed application, the screen road where nothing
  better exists.
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

alo-app <application> controls                     every control its windows show now
alo-app <application> read                         what its windows show, as text
alo-app <application> press  <kind> <name>         press a button, tab, switch, menu or menu item
alo-app <application> type   <name> <text | ->     put text into a field
alo-app <application> set    <name> <number>       move a slider or a number box to a value
alo-app <application> choose <name> <option>       pick one option of a drop-down or a list
alo-app <application> key    <keys>                press keys in its window, as a keyboard would
```

The second group is **what a person does with their hands on a window, in the
terminal.** It is the same seven forms for every application ever installed,
and it is the shape that makes *three equal roads* a guarantee: whatever a
person can click, type, slide or pick in a window, they can do by naming it.
**Which road carries a form is decided per control, not per application**, best
first, as ADR 0001 §6 orders them: the accessibility tree where the control is
described there, and **the screen** where it is not (section 2). The person
types the same command either way; `controls` and the record say which road
was used. The first group is the richer one beside it, for an application that
offers more than its controls.

Examples a person would type:

```text
$ alo-app list
$ alo-app show texteditor
$ alo-app texteditor open_document ~/Notes/today.txt
$ alo-app papers controls
$ alo-app papers press button "Print"
$ alo-app texteditor type "Search" "invoice"
$ alo-app alo set "Text size" 125 --in settings
$ alo-app alo choose "Language" "Latviešu" --in settings
$ alo-app texteditor key ctrl+s
$ find ~/Pictures -name '*.png' | alo-app loupe open -
```

**What the forms take.** `<name>` is the name a control shows, or the label
beside a field, exactly as `controls` printed it. `type` puts the text in as
the person would type it, replacing what the field holds, and `--append` adds
to it instead; `-` reads the text from the pipe, **whole**, so an email body
of many lines is one value. `set` takes a number inside the range the control
itself reports and refuses one outside it. `choose` takes one of the options
the control itself lists, matched exactly. `key` takes keys in the common
spelling — `ctrl+s`, `alt+f4`, `escape`, `tab`, `shift+tab` — one press each,
separated by spaces, and is how a shortcut that has no control is reached.
None of the seven ever takes a position: a control is found by what it says,
on both roads.

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
| text | any text, lines included, as an argument or as `-` from the pipe | at most a length this contract will fix; no characters a person cannot see, except line breaks and tabs |

**Text is a kind here before it is a kind for alo.** For alo it waits on ADR
0143 stage 5, which amends ADR 0001 first, because it widens what a model can
express. For the person it is simply what they typed, and `type` cannot be
used without it. Either way **a text value is never handed to any interpreter
by anything on this machine** — it goes into a field as keystrokes or as the
field's text, never into a script — which is what keeps a person's own email
body from becoming a program, whoever typed it.

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

*The features list gives `image holiday.jpg brighten 20 crop square` as an
example: two abilities, one after the other, on one picture. Under this
contract that is two commands, or two lines of a script, because each is one
call and one record entry:
`alo-app <application> brighten holiday.jpg 20 && alo-app <application> crop holiday.jpg square`.
No application on a fresh machine offers either ability by name (section 6);
through the controls forms, an image editor's own *Brightness* slider and
*Crop* button are reached the same way as any other. Short words such as
`image` and `mail`, and a form chaining abilities in one command, are
question 3.*

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
| 76 | on the screen road: the keys or the click were delivered, and reading the window again could not confirm the change | **not known**, and it is recorded as having run |
| 77 | an organisation's policy on this machine forbids it (ADR 0016) | no |

64, 65, 69, 75 and 77 are the standard `sysexits.h` numbers and mean what they
mean to every other Unix program. 76 is that list's *protocol* code, borrowed
for *sent and not confirmed* because no standard number says it; whether to
borrow it or take one outside the list is part of question 10.

### What is a public surface, and how it changes

Stable, and changed only additively; a break needs a new version and a
deprecation period, as `CLAUDE.md`'s *contracts outlive code* requires:

- the name `alo-app` and its forms `list`, `show`, `<application> <ability>`,
  and the seven control forms `controls`, `read`, `press`, `type`, `set`,
  `choose` and `key`, with the kinds of control `press` takes;
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
and keep nothing. What an application offers comes from five places, and `show`
marks each ability with where it came from. They are not ranked so that one
replaces another: **the seven control forms are offered for every
application**, carried by the accessibility tree where a control is described
there and by the screen where it is not, and the other three add what a
control cannot express.

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

The better of the two roads for the seven control forms, and generic: it needs
nothing written for any application. An application that describes its window
to the accessibility tree — GTK, Qt, LibreOffice, browsers, Electron started
with accessibility on, and alo OS's own surfaces — has each described control
carried this way, **whether or not it also has an adapter**:

| Form | Through the tree | Built today |
|---|---|---|
| `controls` | walks the windows and prints each control's kind, the name it shows, and whether it is greyed out, checked, selected, or holds a value | the walk is `accessible.read_window`'s |
| `read` | prints the text the windows show | `accessible.read_window` |
| `press` | the control's own *press* action | `accessible.activate_control` |
| `type` | the field's editable text: its content set, or text inserted at its end | **not built**; whether each toolkit accepts it is unmeasured (ADR 0143 question 2(a)) |
| `set` | the control's value, inside the range it reports | **not built** |
| `choose` | the drop-down or list's own selection of one option | **not built** |
| `key` | — the tree carries no keys; `key` always goes by the screen road's keyboard | — |

`--in <window>` narrows any form to one window by its title, where an
application has several.

**What a person can rely on is the fallback's own promise, unchanged and
extended to the new forms** (`app-adapters.md`): the tree is read at the moment
of the command and kept nowhere; nothing is found by position; a password
field's contents are never read; and **nothing is guessed** — a form is refused,
and the person told why, when no control of that kind and name is showing,
when more than one is, when it is greyed out, when a value is outside its
range or not one of its options, or when the control cannot be reached from
outside the application.

**Menus are pressed the way a person opens them.** A menu's items are often in
the tree only once the menu is open (unmeasured per toolkit), so
`press menu "File"` then `press menu_item "Save As…"` is the honest road, and
`controls` after the first press lists what opened. `menu`, `field`, `slider`,
`drop_down` and `list` are not yet among the fallback's kinds; adding them is
additive (question 10).

**Where the tree runs out, the screen takes over, control by control.** A
control the tree does not describe, a field the tree describes but will not
take text through, and every `key` go by the screen road below — automatically,
said in `controls`, and written in the record.

### The screen — every window, whether it describes itself or not

ADR 0001 §6's last road, and since the owner's decision of 2026-11-06 the one
that makes parity a guarantee for **every** installed application: alo OS draws
every window and carries every key and click, so it can read what a window
shows and act in it for the person. It is the **least reliable** road, so it is
used only for a control nothing better reaches, and the record marks every act
that went this way.

How each form works on it:

- **`controls` and `read`** — the compositor takes **one picture of that one
  window**, at the moment of the command, and text recognition reads the words
  in it and the shapes around them: a word inside a raised box is a button, a
  word beside an empty box is a field, a box with a tick is a check box. What is
  printed is marked *read from the screen — may be wrong*. The picture is
  thrown away when the command ends, and is never another window's: the
  compositor composes that window alone, so nothing in front of it or behind it
  is in the picture.
- **`press`** — the control is found **by what it says**, as on the tree road,
  in that picture; the compositor delivers one click to the centre of what was
  found, **to that window and no other**. The position is the compositor's to
  work out, never the person's to type, and never kept.
- **`type`** — the field is found by its label and clicked to focus it, and the
  text is delivered as keystrokes **to that window alone**, by the compositor
  that already carries every key a keyboard sends. Replacing a field's
  contents is *select all, then type*; `--append` types at the end.
- **`set`** — the slider is focused and moved with the keys toolkits use for it
  (arrows, Page Up, Home, End), reading the value back after each step until it
  shows the number asked for. A slider that shows no number is refused rather
  than dragged to a guess.
- **`choose`** — the drop-down is pressed, the opened list is read, and the
  option is pressed by its words.
- **`key`** — the keys are delivered to that window, one press each.

**After every act, the window is read again**, and the command says what
changed. Where nothing visibly changed, or something else did, it ends with
code 76 and says *alo sent the click and cannot tell whether it worked* —
recorded as having run, as an unanswered request always is.

**What the screen road promises, and what it cannot.** It never acts on a
window but the one named; it never keeps a picture; it never reads a password
field (a field the toolkit draws as dots is recognised as one, and the text
typed into it is never recorded); and it refuses rather than guesses when two
things say the same words, when the words cannot be read, or when the window
is covered or off the screen. **It cannot promise to understand a window**: it
recognises text and familiar shapes, so a control that is only a picture — an
icon with no words — is listed as *an unnamed control* and cannot be named,
and a control the application draws in its own style may be read as the wrong
kind. That is what *least reliable* means, said where the person reads it.

**What reads the screen.** Text recognition is an engine, pinned and
configured, never written here, which must read every one of the 24 languages a machine can
be set to — or, where the person has one, their local model. Which is
question 14. **Nothing here is built**: ADR 0143 found no code that delivers
keys or clicks for anything but a person's own hands, and `app-adapters.md`
refuses the `synthetic` mechanism today; this contract proposes the amendment
(question 15).

**An application with an adapter.** `app-adapters.md` says today that, for
alo, an adapter's narrower list replaces the tree, because pressing any control
would make a reviewed list decorative. For the person, both are offered side by
side. **For alo the guarantee now pulls against that rule** — *alo cannot be
asked for* a control is a bug — so it is question 4.

### The window — what every application has

`open`, `focus` and `close`, carried out by the compositor through the door in
`application-door.md`. That door admits the person's own login, so `alo-app` can
use it directly. These work for every installed application with a desktop
entry, whatever else it offers.

### When an application offers nothing but its window

No application offers *nothing* any more: every window has the screen road.
What `alo-app show` says, in one sentence naming what was looked for, is which
roads there are — never an empty list, which would look like a fault:

> *Blender can be opened, focused and closed. It publishes nothing on the
> session bus and describes nothing to the accessibility tree, so everything
> inside it goes by the screen — the least reliable road. An adapter would do
> better.*

**What the screen road still cannot reach, said rather than hidden.** For the
windows ADR 0143 names as describing nothing — a game, a 3D or painting
canvas, a remote desktop, a virtual machine, a video — `controls` lists what
words it can read and `key` sends keys, but **a stroke on a canvas or a drag
in a 3D view is a place, not a control**, and none of the seven forms takes a
place. Whether the person may click or drag by position is question 16. Until
it is answered, `show` says *keys and named controls only — drawing and
dragging inside this window cannot be done from the terminal.*

The terminal itself (`app.devsuite.Ptyxis`, or alo's own) is answered *this is
a terminal — type into it directly.*

## 3. Three equal roads, for every installed application

`docs/features.md` (#1101): *whatever a person can do in an application's
window, they can also do by typing a command in the terminal and by asking alo
… **This is a guarantee for every installed application, not only alo's own**:
a control a person can use on screen that the terminal cannot reach, or alo
cannot be asked for, is a bug.*

### How the guarantee is carried

By the seven control forms of section 1, and by nothing written per
application. Every control is carried by the best road it has — the tree where
it is described, the screen where it is not — so **there is no window the
terminal cannot reach**, and the only differences between applications are how
reliably and how richly. What `alo-app <application> controls` prints is,
honestly, what parity is for that application today, each line saying which
road carries it.

The one part of a window none of the seven forms reaches is **a place rather
than a control** — a point on a canvas, a drag in a 3D view. That gap is named
in section 2 and is question 16; until it is answered, the guarantee covers
every control and every key, and `show` says where it stops.

### alo OS's own surfaces

alo OS's own surfaces — Settings, the windows' own controls, Places, the Dock,
the status area, the record — are reached under the application name **`alo`**,
with `--in` naming the surface:

```text
$ alo-app alo controls --in settings
$ alo-app alo press button "Put back…" --in record
$ alo-app alo press switch "Wi-Fi" --in settings
$ alo-app alo set "Text size" 125 --in settings
```

**These go by the tree, never by the screen**, because alo draws them and can
describe every one of them exactly. The shell already publishes every surface it
draws to the accessibility tree a screen reader reads, with the role, name and
state `alo-access` decides for each control, and serves *press* on every
control a reader can press (`crates/alo-access/src/tree.rs`,
`crates/alo-shell/src/access_bus.rs`). So a screen reader, the terminal and alo
reach alo's own surfaces through **one description** — the rule `alo-access`
already states: *if it is good enough for one it is good enough for the other.*

**Read from the code on 2026-11-06, not run:** `alo_access::Surface` lists nine
surfaces — recovery, sign-in, the desktop, the Dock, the status area, the
approval surface, the record, Settings and the window controls. **Places — the
canvas's Places and the World — is not among them**, so today Places publishes
nothing to the tree. *Press* is served; *value* and *text* are not, so a slider
or a field in Settings can be read and not set. Whether the shell's tree is on a
booted machine's bus is unmeasured: it needs the accessibility bus, which the
image does not name (section 6).

### The test that holds it — and what no test can hold

The guarantee is a test, not a convention, and it is two tests, because what can
be proved about alo's own surfaces and what can be proved about somebody
else's application are different things.

**1. alo OS's own surfaces — proved, on every build.** In the crate that draws
each surface, for each of `alo_access::Surface::ALL`, in each of its states:

- **every control drawn is in the tree**, with a name and a kind — a control
  drawn and not published fails, naming the surface and the control. This
  catches the new button somebody forgot to describe;
- **every control in the tree is found by `alo-app`'s lookup exactly once** —
  two controls with one kind and name in one surface fail, because neither the
  terminal nor alo could tell them apart;
- **every control can be used from outside, by its own form** — pressed,
  typed into, set or chosen over the bus, and the surface asserted to have
  done what the person's hand would have done;
- **and alo's lookup finds what `alo-app`'s finds** (question 12).

A control with no road fails the build **unless** it is on a short exception
list written in the test itself, each line naming the control, the reason and
the decision that will close it — so *a control with no command is a bug*
becomes a thing the gate refuses, and the list cannot grow without a reviewer
reading why. On the day it lands that list starts with Places and with every
slider and field, because *value* and *text* are not served yet.

**2. Every other application — measured, as far as measuring reaches.** Nobody
can read the source of every application a person may install, so the test for
them proves **the roads**, not the applications:

- **A fixture per kind of window, in CI.** One small application per toolkit —
  GTK 3, GTK 4, Qt 6, an Electron window, and **one that draws everything itself
  and describes nothing** — each with one control of every kind: a button, a
  check box, a field, a slider, a drop-down, a menu, a tab. For each, every one
  of the seven forms is used, by the road `alo-app` chooses for it, and the
  fixture reports back what it received. The self-drawn fixture is what proves
  the screen road; the others prove the tree road and that the choice between
  them is made per control. A fixture that cannot be reached fails the build.
- **A walk over the applications a fresh machine ships, on a booted machine.**
  For each of the seven in `shipped.toml`, `controls` is run on its first
  window, and each control listed is used once in a throwaway profile, with the
  window read again to check it changed. This is a walk in `docs/walks/`, run
  per release, not on every build — **and its report says so**, with the
  machine it ran on, as the gate requires of a measurement.
- **`alo-app <application> coverage`, on the person's own machine** — a
  diagnostic, not a test: how many controls the tree describes, how many only
  the screen can see, and how many are unnamed. It is how a person, or a bug
  report, sees what parity is for an application nobody here has ever tested.

**What no test can prove, said so it is not mistaken for proof:**

- **that every control of every application is reachable.** Applications are
  unbounded and change on every update; a test proves the roads work on the
  windows it was given, and nothing about the next one;
- **that the screen road read a window correctly.** It recognises pictures; a
  test can show it reads the fixture, never that it reads a window drawn in a
  style nobody here has seen;
- **that an act did what the person meant.** A test proves a button named
  *OK* was pressed; it cannot know what that application does when it is;
- **that an unnamed control is the one the person wanted** — which is why an
  icon with no words is listed as unnamed and cannot be named;
- **and that every application's own actions can be undone.** Put back reaches
  files; what an application does with a press is its own (section 4).

### Richer commands are additive

A typed verb — setting a Settings value by its key, an adapter's
`open_document` — is better than a press by label, and may be declared beside
the controls. It is never a reason to leave a control out: the seven forms are
the floor the guarantee rests on.

## 4. A person's command and alo's request

The two ways in use **one list** of abilities and **one validation**. What
differs is who acted, whether anything waits, and how that is shown and kept.

| | Typed: `alo-app …` | Plain words: `alo-ask "…"` |
|---|---|---|
| Who acted | the person | alo, for the person |
| What it may reach | anything the person can, with no grant | only applications the person has let alo use (ADR 0143, the grant over one application) |
| Which abilities | the seven control forms by the tree or the screen, declared, discovered, window | the seven control forms, declared and window — **never discovered** (question 4); the screen road for that application only once the person has switched it on (question 17) |
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

- `road` is `tree`, `screen`, `declared`, `discovered` or `window`. **`screen`
  is ADR 0001 §6's least reliable road, and every reader of the record shows it
  as such** — the record window marks the entry *done by reading the screen*,
  because afterwards nobody can say for certain what was pressed;
- for `type`, the text typed is kept in full, so the record says what went
  where — **except into a password field**, where only the field and the number
  of characters are kept;
- `answered` is `done`, `refused` with the sentence the person saw, or
  `no-answer` — the last recorded as having run, as `app-adapters.md` requires;
- **no `agent` field, and no room for one**: the person acted;
- a command refused before it reached the application keeps the ability and
  why, and **not** the arguments, for `record-file.md`'s reason: what never
  validated is whatever arrived.

Which commands are written down at all is question 5.

### Put back

Before a typed ability that changes something — declared as a change, or
discovered, or any of `press`, `type`, `set`, `choose` and `key`, whose effect
nobody can know in advance — `alo-app` asks the service for a place
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

Three facts govern every row:

- **The accessibility tree may not be there at all.** The image names no
  accessibility package (`grep` over `image/` for `at-spi` and `a11y` finds
  nothing), and whether the accessibility bus starts in a signed-in session is
  unmeasured (ADR 0143, fact 10; its plan's stage 1 says *not started*). If it
  does not start, the tree road is gone everywhere, and so is a screen reader.
- **The screen road is not built.** Nothing takes a picture of one window for a
  command, nothing recognises text in one, and nothing delivers keys or clicks
  except a person's own hands; `app-adapters.md` refuses `synthetic` today.
- **Typing, setting and choosing are not built on either road.** So nothing on a
  fresh machine can write an email, put text into a document, or move a slider —
  whoever asks.

| Application | Window: open, focus, close | Declared | Discovered on D-Bus | Tree: controls, read, press | Screen road |
|---|---|---|---|---|---|
| Firefox (`org.mozilla.firefox`) | built, not walked on a booted machine | none. Its automation interface runs script, which an adapter may not take | unknown; only a booted machine can say | documented yes; a page is as described as its site made it; unmeasured on alo | not built; would be its road for a page that describes nothing |
| Dolphin (`org.kde.dolphin`) | as above | none | the shared file-manager interface under a shared name, which a command cannot tie to Dolphin; the rest unknown | documented yes, once Qt sees an assistive technology; unmeasured | not built |
| Ark (`org.kde.ark`) | as above | none | unknown | documented yes; unmeasured | not built |
| Text Editor (`org.gnome.TextEditor`) | as above | **`open_document`, `new_window`** — built and tested, never run on a booted machine | `Open` and its actions, by the GNOME pattern; unmeasured | GTK 4 publishes a tree on Wayland, documented; whether its text view takes text through the tree is unmeasured | not built; would type into the document if the tree will not |
| Image Viewer (`org.gnome.Loupe`) | as above | none | `Open`, if it publishes itself as GNOME applications do — unmeasured | controls yes, documented; the picture is a picture | not built. **Whether Loupe itself has brightness or crop controls is not known here** — if it has, they are reached as controls; if not, no road brightens a picture in it |
| Papers (`org.gnome.Papers`) | as above | none | as Loupe | controls yes; a page's text partly, documented | not built |
| Terminal (`app.devsuite.Ptyxis`) | as above | — | — | answered *type into it directly* | — |
| **alo OS's own surfaces** (`alo`) | — | none yet | — | nine surfaces published with names, states and *press*, read back over a bus by the shell's own tests (read from the code, not run here); **Places is not one of them**; sliders and fields can be read, not set; never seen on a booted machine's bus | never: alo's own surfaces go by the tree only |

**And there is no mail application on a fresh machine.** `crates/alo-software/shipped.toml`
names seven, one per role, and none of them sends email. `mail send` therefore
has nothing to reach until the person installs one. Once one is installed, its
*To* field, its message body and its *Send* button are reached by `type` and
`press` like any other controls — which is the point of the seven forms. The
code editor ADR 0143 chose (VSCodium) is not on that list yet either.

So the honest summary of a fresh machine, once `alo-app` is built **and before
typing and the screen road are**: open, focus and close any application; open a
file in the text editor; probably open files in the image viewer, the document
viewer and the browser; and — only if the accessibility bus runs — list and
press the named buttons, tabs, switches and menu items of any described window,
alo OS's own Settings, Dock, record and window controls included. **That is
parity for what is pressed and none yet for what is typed, set or chosen, and
none at all for a window that describes nothing.** *Fully used* is true only
when all three are built.

What closes the gap, in the order it pays: the accessibility bus on the image
(ADR 0143 stage 2); `type`, `set` and `choose` on the tree road (stage 6 and the
text kind, stage 5); the screen road, with its own measurement; *value* and
*text* on alo's own surfaces, and Places in the tree; and adapters with real
verbs for applications that offer an automation interface.

## 7. Open questions for the owner

Each has a recommendation. None is answered by this draft. **Settled by the
owner's decisions of 2026-11-06, and so no longer asked:** whether parity covers
applications alo did not make (it does, as a guarantee); whether a window that
describes nothing is reached (by the screen road); and whether typing, sliders
and drop-downs are in scope (they are — *fully used* needs them). The old
question 11 is replaced by the new 11 below.

**1. What is the command called?**
`alo-app` here; it could be anything. *Recommendation: `alo-app`*, which follows
`alo-ask`, `alo-try` and `alo-back`, and is unused. Not `alo`, which ADR 0124
left unbuilt and which a person might expect to mean plain words.

**2. Is the shape `alo-app <application> <form or ability> <arguments>` right?**
The alternative is one command per application (`texteditor open_document …`),
which puts every installed application's name on the shell's own list of
commands, where any of them can collide with a program of the same name.
*Recommendation: one command*, with the short words of question 3 as an
optional layer above it.

**3. The features list's examples use `mail` and `image`, and chain two
abilities in one line. Should either exist?**
Short words would be shell functions in alo's snippet, each meaning *the
application this person chose for that kind of file*, the way `try` calls
`alo-try` — but `mail` is already a standard Unix program on many systems.
*Recommendation: later, and additively* — define a word only where no command of
that name exists; keep one ability per command, so one line stays one record
entry; and correct the two examples in `docs/features.md` to the contract's form
when this is accepted.

**4. Abilities discovered on D-Bus: for the person only, or for alo too?**
*Recommendation: the person only.* Law 2 does not bind the person; alo stays on
abilities somebody reviewed, because `app-adapters.md` refuses an action chosen
by an argument for exactly the reason ADR 0001 exists. **The controls are
different**, and here the guarantee does decide something: alo must be able to
be asked for every control, so `app-adapters.md`'s *an application with an
adapter is reached through its adapter's verbs alone* has to give way — alo
reaches an adapter's verbs **and** the application's controls, under the
person's grant, each press one sentence and one OK. *Recommendation: amend that
sentence of `app-adapters.md`*, keeping what it protected by marking a control
press as less certain than a reviewed verb in the sentence the person approves.

**5. Which typed commands are written in the record?**
*Recommendation: every act on an application — every declared change, every
discovered ability, every `press`, `type`, `set`, `choose` and `key` — and every
refusal of one; not `list`, `show`, `controls`, `read` or a declared read.*
Writing every look a person takes would be a record of the person rather than of
what happened to the machine.

**6. Is a place to return to kept before every typed act, or only in a session
that asked for it?**
Every `press` is now an act whose effect cannot be known, so *before every
change* would mean a snapshot before every click typed. *Recommendation: before
every command that acts, always — one place per command line, not per control —
and measure on a booted machine what a place costs before this is accepted*,
because if places are cheap, Put back is true for every typed act, and if they
are not, ADR 0127's window of fifty is spent in an afternoon.

**7. How does what an application sends reach the indicator, for a typed
command?**
*Recommendation: measured* — the application's sandbox counted for the seconds
the command runs, with the line naming the command. *Declared* is the fallback
for an application whose sandbox cannot be told apart.

**8. Does the plain-words proposal show the typed command beside its sentence?**
*Recommendation: yes*, below the sentence and clearly not what is being
approved.

**9. Should `alo-app` refuse to run under the agent's own login?**
*Recommendation: no refusal* — a person who let alo run commands (ADR 0111)
already let it reach strictly more — **but** the entry then names the agent,
read from the kernel's answer about who is running it, never from a flag.

**10. Does this contract change `app-adapters.md`?**
It proposes two additive fields on a verb, `gives` and `sends`; five new kinds
of control (`menu`, `field`, `slider`, `drop_down`, `list`); fallback verbs for
typing, setting and choosing; and the screen road as a mechanism the machine
carries out. *Recommendation: yes, each in the change that first builds what
reads it.*

**11. On the tree road, how is text put into a field?**
Through the field's own editable text (what a screen reader's dictation uses),
or always as keystrokes to the window (the screen road's way), which every
toolkit accepts. *Recommendation: the field's editable text first, keystrokes
where the field refuses it* — the first can be read back exactly, which is what
lets the record say what went in. ADR 0143 stage 1 measures it.

**12. For alo's own surfaces, does alo use the same tree, or verbs of its own?**
*Recommendation: the same tree*, reached as `alo` under a grant the person makes
like any other, so the screen reader, the terminal and alo share one description
of every control and the test in section 3 holds all three at once. A Settings
change alo makes this way is still one sentence and one OK, unless the person
chose full trust.

**13. Does the parity test refuse a control with no road, from the day it lands?**
*Recommendation: yes, for alo's own surfaces, with a written exception list in
the test* (section 3), starting with Places and every slider and field; **and
for other applications, the fixtures fail the build and the shipped-application
walk runs per release**, its report naming the machine. A test that only warns
is one lanes learn to skip.

**14. What reads the screen?**
A text-recognition engine, pinned and configured like every engine here, or the
person's own local model where they have one. *Recommendation: an engine, always
available, with no model required* — the road must work on a machine with no
model, as *a good computer without the agent* (ADR 0009) asks — and the local
model as an option the person may choose for better reading. The engine needs
an ADR naming it, and has to read all 24 EU languages.

**15. The screen road needs ADR 0001 §6 and `app-adapters.md` amended.**
§6 already permits it as a last resort, marked in the record and able to be
disabled by policy; `app-adapters.md` refuses `synthetic` outright and says
*nothing is found or pressed by position*. *Recommendation: amend both*, to
permit the screen road under the rules in section 2 — one window, found by what
it says, read again after, never kept, marked in the record — before a line of
it is built.

**16. May a person click or drag by position?**
A stroke on a canvas or a drag in a 3D view is a place, not a control, so none
of the seven forms reaches it. *Recommendation: for the person's typed commands,
yes, later* — a `click` and a `drag` in proportions of the window, never in
pixels (*nothing is built to one screen size*), recorded as *by position*; **for
alo, no**, until somebody can say what a position-based act did afterwards.
Until then, `show` says where the terminal stops.

**17. Is the screen road on for alo by default?**
`docs/features.md` and ADR 0143 (its question 11) promise pictures and keystrokes
for alo **off by default, switched on per application** by the person, because
alo may press the wrong thing. The new guarantee says alo can be asked for every
control. *Recommendation: both stand* — the person's typed commands use the
screen road at once, because the person typed them; alo uses it for an
application only once the person has switched it on for that application, and
when it is off, asking alo for such a control answers *I can only do this by
reading the screen of Blender — switch that on in Settings?* That keeps the
guarantee (alo can be asked, and the choice is the person's) without
withdrawing a promised default, which law 5 would call a bug.

## What this page does not decide

The text-recognition engine (question 14); the amendments to ADR 0001 §6 and
`app-adapters.md` (question 15); the text kind for alo (ADR 0143 stage 5); and
the grant over one application and where a person makes it (ADR 0143,
question 10). Each is its own decision, and this contract uses whatever they
settle.
