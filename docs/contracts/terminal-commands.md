# Contract — commands a person types to use their applications

**Status: contract, accepted 2026-11-06.** Additive changes only; a break
requires versioning and a deprecation period (`CLAUDE.md`, *contracts outlive
code*). **The owner answered every question it put on 2026-11-06**; the answers
are in section 7, with the date and the owner's own words where they gave them,
and the body of this page is in line with them.
[ADR 0147](../decisions/0147-the-screen-road-and-who-reads-the-screen.md), the
screen road and who reads the screen, was accepted the same day (*"yes"*), and
ADR 0001 §6 and `app-adapters.md` were amended in the same change. It is the
contract `docs/features.md` said must be written *before* the commands are
built (*Everything on the computer, from the terminal*, #1101).

**What is built, 2026-11-07** (`crates/alo-app`, stage 2 of the plan) — and
**none of it has been walked on a booted alo machine**:

| | |
|---|---|
| `list`, `show <application>`, `show <application> <ability>`, with `--json` carrying `"format": 1` | built |
| `<application> <ability> [values]` for declared abilities, by order or `--<argument>`, through alo's own validation | built |
| `controls`, `read`, `press <kind> <name>` through the accessibility tree | built |
| `open`, `focus`, `close` through the desktop's door | built |
| naming: identifier, short name, reserved words, a shared short name refused | built |
| `--in`, `front`, `alo` | built |
| `-` from the pipe, one call and one entry per line | built |
| the record entry, the count of what the application sends (ADR 0148), one place to return to per command line | built |
| exit codes 0, 1, 64, 65, 69, 75 | built; **76 and 77 are produced by nothing yet** — 76 is the screen road's, and no policy reaches a typed command |
| `type`, `set`, `choose`, `key`, `click`, `drag`, the screen road, discovered abilities, `coverage`, the parity test | **not built** |

Accepted is a promise about what will be built; the table is what a person can
type today (section 6 says what each row reaches).

**Three things this page did not settle, settled by the build and put to the
owner** on 2026-11-07, by the coordinator: when a count cannot be held
the command **runs and says so** rather than refusing; `--plain` is read as
*plain sentences wherever the output goes*; and `open`, `focus` and `close`
refuse `--in` and `front`, because they act on the whole application and the
person meant one window.

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
other way in, plain words through `alo` (with `alo-ask` kept as its alias), uses **the same list** of what an
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
- **ADR 0124** — `alo-ask` — being renamed `alo`, with `alo-ask` kept as an alias — is the one command for plain words, and it both
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

*The name is the owner's choice of 2026-11-06 (answer 1).* It follows
`alo-ask`, `alo-try` and `alo-back`, and was free: no file in this repository
used it.

**Plain words are the equal road beside it.** The owner, 2026-11-06: *"Also the
users will be able to use alo (natural language in the terminal instead of
commands)."* A person who would rather say what they want types it to
`alo` (ADR 0124's command, built as `alo-ask`, which stays as an alias of `alo`), and alo does it in the application under its own
rules — through the same list of abilities and controls `alo-app` uses. Neither
is the real road with the other as a shortcut (answer 2).

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
alo-app <application> click  <across> <down>       click at a place in its window, as proportions of it
alo-app <application> drag   <across> <down> <across> <down>   drag from one place in its window to another
```

The second group is **what a person does with their hands on a window, in the
terminal.** It is the same seven control forms, and the two position forms
`click` and `drag`, for every application ever installed,
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
None of those seven takes a position: a control is found by what it says, on
both roads.

**`click` and `drag` are the two that do, and they are the person's alone**
(the owner's answer of 2026-11-06, answer 16). A place is given **as proportions
of the window** — `click 0.5 0.25` is half-way across and a quarter of the way
down — **never in pixels**, so the same command means the same place on any
screen and at any scale (*nothing is built to one screen size*). Each number is
from 0 to 1, both ends included, and a value outside that is refused. They are
how a stroke on a canvas or a drag in a 3D view is reached, which no named
control can be. **Every click and drag is recorded as *done by position***,
because afterwards nobody can say what was at that place. They always go by the
screen road, to that window only. **alo never clicks or drags by position**, on
any road and at any setting: `alo` is never offered either form.

### Naming the application

An application is named by **its identifier** (`org.gnome.TextEditor`) or by
**its short name**, which is the last part of the identifier in lower case
(`texteditor`, `firefox`, `dolphin`, `loupe`), less a version number it begins
with (below). Never by the name its menu entry
shows (*Text Editor*), because two applications may both call themselves that,
and the one acted on must be the one meant — the rule `agent-verbs.md` already
keeps (*the identifier is approved; the name is only shown*).

When two installed applications share a short name, the short name is refused
and both identifiers are listed, so the person picks. Nothing is guessed.

**A version number the last part begins with is left off.** *Amended
2026-11-08, additively.* Not every identifier is reverse-DNS: LibreOffice's
desktop entries are `libreoffice26.2-writer`, `libreoffice26.2-calc` and so on,
whose last dot is inside the version number, so the rule above gave them
`2-writer` and `2-calc` — found walking a booted machine that day. So where the
last part begins with digits and a hyphen and something follows, the digits
and the hyphen are left off: **`writer`**, `calc`, `base`, `startcenter`. It
cannot touch a reverse-DNS identifier, because no element of one may begin with
a digit (the D-Bus naming rule application identifiers follow); a hyphen
elsewhere (`org.gnome.font-viewer` → `font-viewer`) and digits with no hyphen
after them read as before.

- **The name the rule gave before still reaches the application** (`2-writer`),
  so a script written against it keeps working. It is not deprecated, and it
  counts in the clash rule like the new one: an application *answers to* both.
- `list` and `show` offer the new one, so `short_name` in `--json` reads
  `writer` where it read `2-writer`. Its meaning — the short name that reaches
  the application — is unchanged.
- **The one thing that can stop working is a clash the new name creates**, and
  it is refused rather than guessed: on a machine with both
  `org.example.Writer` and `libreoffice26.2-writer`, `writer` reached the first
  alone and is now refused, naming both. Two LibreOffice versions installed
  together are refused at `writer` the same way, and each one's old name still
  reaches only it. Every identifier still reaches its application.

### `front` — the window the person was just working in

*Added 2026-11-06, approved by the owner: "Yes add it".* A person working in a
window should not have to look up which application it belongs to:

```text
$ alo-app front press button "Save"
Pressed Save in Text Editor, today.txt
```

**What `front` means.** While the person types, the terminal they are typing in
is the window in front, so `front` cannot mean *the window in front now*. It
means **the window that was in front just before focus moved to this
terminal** — the most recently focused window that is not the terminal the
command was typed in. It is asked of the compositor once, at the moment the
command starts, and that one window is the subject of the whole command.
**Nothing answers that question yet**: the compositor's door
(`application-door.md`) takes acts on a named application and has no *which
window was in front before this one*. It is an additive act on that door, made
in the change that builds `front`, and the door already admits only the
person's own login.

- **There is none** — nothing had focus before the terminal, or the window in
  front when the command starts is not a terminal at all (a script started by a
  timer, say), so *before this terminal* means nothing: refused, in a sentence:
  *There is no window you were working in before this terminal.*
- **That window has since closed**: refused, in a sentence — *The window you were
  working in before this terminal (Text Editor, today.txt) has closed.* **Another
  window is never chosen instead**, not the one before it and not another window
  of the same application: the person meant one window, and a guess would act on
  a different one with the same confidence.
- **A window the compositor cannot name** — one that has no application
  identifier this machine knows — is refused the same way, rather than acted on
  as *something*.

**The application is the one that owns that window**, identified as every
application here is: by the identifier its sandbox gives it, never by its
title (`app-adapters.md`, *an application is identified by its sandbox*). Every
form then works exactly as if that identifier had been typed, narrowed to that
one window as `--in` would narrow it. **The record keeps the real identifier and
the window, never the word `front`**, so a reader a year later sees what was
acted on, not how it was named.

**The output names the window it acted on** — the application's name and the
window's title — on every form, not only on success: *Pressed Save in Text
Editor, today.txt*, or *Text Editor, today.txt, shows no button named Save.* A
person reads which window it was in the same line that says what happened.

**Focus comes back to the terminal.** An act on the screen road may bring the
application's window forward. After each act, `alo-app` asks the compositor to
return focus to the terminal it was typed in, so a second `alo-app front …`
typed next still means the same window. Where focus cannot be returned, the
command says so, because the next `front` would be refused.

**Reserved words.** A name in the application's place is read in this order:
first the reserved words — `list`, `show`, `front` and `alo` — then a full
identifier, then a short name. So **an application whose short name is a
reserved word is not reached by that short name**, and `show` and `list` say
so beside it: *reach this application as `com.example.Front`*. **Its full
identifier always reaches it**, because no identifier can be a reserved word —
an identifier has at least two parts separated by dots. A new reserved word is
a change to this contract and is added only where no installed application's
short name is known to need it; adding one is otherwise a break, versioned as
any other.

**For alo.** alo may use `front`, **and only at the moment the person asks.**
The window the person was working in is context, and `CLAUDE.md` is plain about
context: *offered, never watched* — it reaches an agent only at the moment of
invocation, and only for that turn. So when a person types to `alo` in a
terminal, the window that was in front before that terminal is taken once, at
that moment, and handed to that turn as its context, exactly as the focused
window is when alo is asked from the Dock. It is never asked for again during
the turn, never kept after it, and alo cannot ask for *the window in front*
whenever it likes — there is still no verb that lists open windows
(`agent-verbs.md`, *the application verbs*). Acting on it still needs the grant
over that application and the person's OK, unless they chose full trust; the
proposal names the window by its application and title, as the output does.

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
`image` and `mail` come later; a command line does one ability (answer 3).*

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
for *sent and not confirmed* because no standard number says it (answer 10).

### What is a public surface, and how it changes

Stable, and changed only additively; a break needs a new version and a
deprecation period, as `CLAUDE.md`'s *contracts outlive code* requires:

- the name `alo-app` and its forms `list`, `show`, `<application> <ability>`,
  and the seven control forms `controls`, `read`, `press`, `type`, `set`,
  `choose` and `key`, with the kinds of control `press` takes, and the two
  position forms `click` and `drag` with their proportions;
- how an application and an ability are named, the reserved words `list`,
  `show`, `front` and `alo`, and what `front` means;
- how arguments are written, and `-` for the pipe;
- the line format of a pipe, and the `--json` shape of `list` and `show`, which
  carries `"format": 1` so a later shape can be told apart;
- the exit codes above;
- the record entry in section 4.

**As built on 2026-11-07, these are the shapes** (`crates/alo-app`):

- on a pipe, `list` gives one identifier per line; `show <application>` its
  declared abilities' names; `show <application> <ability>` that ability's
  arguments' names; `controls` one control per line as
  `<kind>⇥<name>⇥<states>`, where `<kind>` is a kind `press` takes or `field` or
  `password_field`, and `<states>` is any of `greyed-out`, `on` and `off`
  separated by commas; `read` each piece of text a window shows — and where a
  window shows none, nothing on the output and a sentence saying so on the
  error stream, naming the window (added 2026-11-12, additively: nothing that
  was printed before is printed differently); and an act
  gives back nothing, because no ability declares `gives` yet;
- `--json` for `list` is `{"format":1,"applications":[…]}`, each with
  `identifier`, `name`, `typed_as`, `short_name` (`null` where it does not reach
  it), `terminal`, `abilities` and `reaches` — what its own package lets it
  reach without asking, `{"home": "none"|"read"|"read-and-change",
  "starts_programs_outside": true|false}`, read from the package and its
  overrides on the machine, and `null` where no package was read, which is
  *not known* and never *sealed* (added 2026-11-12, additively, ADR 0143 fact
  4; the sentence form says the same in words and says nothing where there is
  nothing to say); for `show` it is the same object with
  `"format":1` and `roads` — `window`, `tree` (`described`, `not-open` or
  `no-tree`), `declared` (each ability's `ability`, `purpose`, `effect`, `sends`
  and `arguments`), `discovered` (`not-asked`) and `screen` (`not-built`);
- `--plain` gives plain sentences in the person's language wherever the output
  goes, and `--json` is offered for `list` and `show` only;
- `ALO_APP_DOORS_IN` names the folder this person's two doors are in, in place
  of `/run/alo/<uid>`, which is how a test runs the real binary against doors
  of its own. It moves nothing a person does not already have: both doors admit
  only their own login, decided by the kernel.

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

**These are offered to the person and to alo both** (the owner's answer of
2026-11-06, answer 4). The person uses them at once; alo uses them under its
own rules — over an application the person has let it use, one sentence and one
OK per act, or none if the person chose full trust. Nobody has reviewed a
discovered ability, so `show`, and the sentence alo puts to the person, say in
plain words: *the application describes this itself; alo cannot tell what it
changes or whether it sends anything.* `app-adapters.md` is amended in this
change to permit it (*Amended 2026-11-06* there).

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
additive (answer 10).

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

**Who may use it** (the owner's answer of 2026-11-06, answer 17): **the
person's typed commands use it at once.** alo uses it for an application **only
after the person has switched it on for that application**, in Settings, with
its cost said in ADR 0143's words — *alo will act on what it sees in pictures of
this window, and may press the wrong thing.* Until then, asked for a control
only the screen can reach, alo says *I can only do this by reading the screen of
Blender* and offers the switch. That keeps ADR 0143's promise of *off by
default, switched on per application* rather than withdrawing it.

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

**What reads the screen — the person chooses, in Settings.** The owner,
2026-11-06: *"User chooses in settings, it can be alo services, local model,
third-parties apis."* Three choices (answer 14):

| Choice | What it is | What it costs the person |
|---|---|---|
| **alo's own recognition** | text recognition built into alo OS, pinned and configured like every engine here, working with no network and no model | nothing leaves; may read less well than a model |
| **their local model** | a model on this machine that the person has set up | nothing leaves; needs a model that can read pictures |
| **a third-party service they name** | a provider the person chooses and signs in to | **a picture of the window is sent to that provider every time the screen is read** — said in those plain words beside the option before it is chosen, shown on the indicator as it happens, and written in the record, as law 1 requires |

**Before anybody has chosen, alo's own built-in offline recognition is used** — *the
owner's decision of 2026-11-06*: it is the only choice that works
on every machine, with or without a model, and sends nothing. Whichever is
chosen, it must read every one of the 24 languages a machine can be set to, or
say which it cannot. **A password field is never sent anywhere**: it is cut out
of the picture before any reader, built-in or not, sees it.

**Nothing here is built**: ADR 0143 found no code that delivers keys or clicks
for anything but a person's own hands. ADR 0001 §6 and `app-adapters.md` were
amended **before** any of it is built (answer 15), by
[ADR 0147](../decisions/0147-the-screen-road-and-who-reads-the-screen.md),
accepted 2026-11-06, so no document now refuses it — and the code in
`crates/alo-adapters` still refuses an adapter that *declares* `synthetic`,
which stays right: the screen road is the machine's own, never an adapter's.

**An application with an adapter.** `app-adapters.md` said that, for alo, an
adapter's narrower list replaces the tree, because pressing any control would
make a reviewed list decorative. The guarantee — *alo cannot be asked for* a
control is a bug — and the owner's answer 4 reverse that: **alo reaches an
adapter's verbs and the application's controls both**, and the sentence it puts
to the person says when an act is a press rather than a reviewed verb.
`app-adapters.md` is amended in this change to say so. For the person, both were
always offered side by side.

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
in a 3D view is a place, not a control**, and none of the seven control forms
takes a place. The person reaches it with `click` and `drag`, by proportions of
the window, recorded as done by position (answer 16); `show` says so for such a
window — *named controls and keys, and places by position: alo cannot do the
last for you.*

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

The one part of a window no named control reaches is **a place rather than a
control** — a point on a canvas, a drag in a 3D view. The person reaches it with
`click` and `drag` (answer 16). **alo does not**, because alo never acts by
position; asked for such a thing, alo says it cannot do it by position and the
person can. That is the one place the three roads are not equal, and it is the
owner's decision rather than a gap.

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

#### Settings is described as it is drawn, and its old names keep working (added 2026-11-12)

*Additive. Nothing that was accepted before is refused for a reason it was not
refused before, and nothing printed before is printed differently.*

**Until 2026-11-12, `--in settings` reached a fixed list**: a window called
*settings*, a list called *what can be changed*, and one switch for each of the
nine access settings, named *read the screen aloud*, *magnify what is under the
pointer*, *use strong colours that are easier to read*, *make all text larger*,
*stop things sliding and fading*, *press keys one at a time instead of holding
them together*, *ignore a key unless it is held for a moment*, *ignore the same
key pressed twice quickly* and *always show where the keyboard is* (in the
person's own language). On a running machine that list was never on the screen,
so no command reached it; and only two of the nine switches are drawn on any
Settings page.

**From 2026-11-12, Settings is described by the page it is showing** — the one
description a screen reader, the terminal and alo share. While Settings is open,
`controls --in settings` lists what is drawn: the side column's pages as `tab`,
the page's choices as `radio_button` (a Default apps row, a part of a segmented
choice, an entry of an open list), its switches as `switch`, and what opens
something as `button`. `press` reaches each of them by the name drawn on it —
`alo-app alo press radio_button "Firefox" --in settings` chooses what opens web
links. `tab` and `radio_button` were already kinds `press` takes.

**The old names still work, and say what they will become:**

| The name before | What it reaches now |
|---|---|
| *settings* (the window) | Settings' own window, named *Settings* as its side column is headed; `--in settings` is unchanged |
| *what can be changed* (the list) | the side column's list of pages, under the same name |
| *stop things sliding and fading* (a switch) | the **Reduced motion** switch on Accessibility. `press switch "stop things sliding and fading" --in settings` presses it, and the command says on the error output what it is called now and that the old name will stop working in a later release |
| *read the screen aloud* (a switch) | the **Screen reader** switch on Accessibility, with the same notice on the error output |
| each of the other seven switches | nothing, because no page draws them yet. The command is refused with a sentence saying that setting is not on any page of Settings on this computer yet, and nothing is pressed (exit as any other refusal) |

**Keyed on the setting, not on the words.** The shell marks the row that *is* a
setting with that setting's old name, and lists on Settings' own node each
setting no page draws. When a page draws a row for one of the seven, its old
name reaches that row with the notice above, with no change to this contract.

**Deprecation.** The old names are announced as deprecated on this date. Under
*A deprecated spelling keeps working for at least one release after it is
announced*, none of them is removed before the release after the first one that
carries this note.

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
- **and alo's lookup finds what `alo-app`'s finds** (answer 12).

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

| | Typed: `alo-app …` | Plain words: `alo "…"` (alias `alo-ask`) |
|---|---|---|
| Who acted | the person | alo, for the person |
| What it may reach | anything the person can, with no grant | only applications the person has let alo use (ADR 0143, the grant over one application) |
| Which abilities | the seven control forms by the tree or the screen, `click` and `drag`, declared, discovered, window | the seven control forms, declared, discovered and window (answer 4); the screen road for an application only once the person has switched it on for that application (answer 17); **never `click` or `drag`** (answer 16) |
| Waits for an OK | never | yes, one OK per action — unless the person switched on full trust for the terminal in Settings |
| Where it runs | in the person's own process | in `alo-agentd`, as a turn |
| The gutter in alo's terminal (ADR 0113) | `YOU` for the line, `PROGRAM · alo-app` for its output | `YOU` for the request, `ALO` for the proposal and the answer |
| The record | an entry with no agent in it, in the person's own record | the turn's `ran` entry, naming the agent, the grant and the approval |

**What the screen shows.** The gutter decides who is speaking from where the
bytes came from, and a program cannot write into it — so `alo-app`'s output is
labelled `PROGRAM`, and can never be dressed up as alo's. **A plain-words
proposal shows the plain sentence only** — not the typed command that would do
the same thing; the owner declined that on 2026-11-06 (answer 8). What the
person approves is the sentence. In a terminal that is not
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

**What is written down at all — changes and refusals; looking is not recorded**
(the owner's answer of 2026-11-06, answer 5). Every act on an application and
every refusal of one is an entry; `list`, `show`, `controls`, `read` and a
declared read are not.

**When alo runs `alo-app`** — which a person who let alo run commands (ADR 0111)
makes possible — **the entry is alo's, under alo's rules** (answer 9): it names
the agent, read from the kernel's answer about which login is running the
command, never from a flag, and the grant over the application and the OK apply
exactly as on alo's own road.

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
that only reads. `alo-app` can tell, so it keeps a place only before a command
that acts. **One place per command line, and its cost is measured on a booted
machine before this is built** (the owner's answer of 2026-11-06, answer 6).

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
be built as though it were**. **It is made true by counting** (the owner's
answer of 2026-11-06, answer 7): for as long as the command runs, a kernel
counter is attached to that application's sandbox, and the indicator shows
*Thunderbird is sending, during your command*. It counts, rather than declares.
*Corrected 2026-11-07:* this said the counter was the one *that already watches
a turn's connections*. There was none — the boundary decides and keeps nothing
(ADR 0015) — so the owner decided a counter of its own, bytes and nothing else,
in [ADR 0148](../decisions/0148-the-kernel-counts-what-an-application-sends-during-a-persons-command.md),
and `application-door.md`'s `count` act is how `alo-app` asks for it. It also counts
anything else that application sends in those seconds, and the line says
*during your command* rather than claiming every byte was the command's. The
line is raised by the service that draws the indicator, not printed by
`alo-app`, because ADR 0125 says a command does not report its own egress.

**And a picture of a window sent to a third-party reader** (section 2, answer
14) is egress `alo-app` itself causes: it goes through the service that already
sends questions to a provider, on the indicator and in the record, exactly as a
question to a provider does.

**Counting everything an application does, outside the command, is
not offered**, because that is watching the person, which this product calls a
bug (ADR 0143, the table of settings). And `alo-try --offline` already lets a
person cut a command off from the network with the kernel holding it.

The plain-words road is covered by ADR 0143's own answer (stage 9 of its plan):
by default alo works in a window of its own, and only that window's traffic is
counted.

## 6. What a fresh machine can do with this today

**`alo-app` is built as of 2026-11-07 and shipped in the image** (`/usr/bin/alo-app`),
with the forms the table at the top of this page lists. What follows is what it
reaches, read from the code and documents of 2026-11-06 and unchanged by
building it. **Nothing in this table has been measured on a booted alo machine**;
where only one could tell, it says so. *Until 2026-11-07 this paragraph began
"`alo-app` does not exist".*

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

## 7. The owner's answers, 2026-11-06

Every question the draft of this contract put to the owner was answered on
2026-11-06, relayed by the coordinator. Each is closed here with the answer;
where the owner gave words, they are quoted. Where the owner took the draft's
recommendation without comment, it says so. **Answered is not built**: nothing
on this page exists yet.

**1. The command's name — `alo-app`.** Closed.

**2. One command for every application — yes.** The owner added: *"Also the
users will be able to use alo (natural language in the terminal instead of
commands)."* So **plain words in the terminal are an equal road beside
`alo-app`, not a fallback beneath it**: a person types what they want in their
own words and alo does it, under alo's rules. That road is ADR 0124's command,
already built as `alo-ask`. **The owner decided on 2026-11-06 that it is
spelled `alo`, with `alo-ask` kept as an alias**, so one command answers to both
names and nothing that types `alo-ask` breaks. The rename is another change's,
not this contract's; this page names `alo`. Closed.

**3. Short words and chaining — short words later; one ability per line.**
`mail`, `image` and the like may come later, additively, only where no command
of that name exists. A command line does one thing, so one line stays one record
entry. The two examples in `docs/features.md` are written in the contract's form
when this contract is accepted. Closed.

**4. Abilities an application announces on D-Bus — the person and alo both.**
alo uses them under its own rules: one sentence and one OK per act, or none if
the person chose full trust. And alo reaches an application's controls as well
as its adapter's verbs. `app-adapters.md` is amended in this change to say both
(*Amended 2026-11-06* there). Closed.

**5. What the record keeps — changes and refusals; looking is not recorded.**
Every act on an application (every declared change, every discovered ability,
every `press`, `type`, `set`, `choose`, `key`, `click` and `drag`) and every
refusal of one is written. `list`, `show`, `controls`, `read` and a declared
read are not, because a record of what a person looked at is a record of the
person. Closed.

**6. A place to return to — one per command line, and its cost is measured on a
booted machine first.** The measurement comes before this part is built: if a
place is cheap, Put back is true for every typed act; if it is not, ADR 0127's
window of fifty would be spent in an afternoon, and the owner hears that before
anything is built on it. Closed, with the measurement owed.

**7. What an application sends, for a typed command — counted.** The
application's sandbox is counted for the seconds the command runs, and the
indicator's line names the command. Nothing outside those seconds is counted.
Closed.

**8. The typed command under alo's proposal — no.** The owner declined it:
**alo's proposal shows the plain sentence only.** The paragraph in section 4
that said otherwise is removed. Closed.

**9. When the agent runs `alo-app` — allowed, and recorded as alo's, under alo's
own rules.** The entry names the agent, from the kernel's answer about which
login is running it, never from a flag, and everything alo's road requires — the
grant over the application, the OK unless full trust — applies as if alo had
asked through its own road. Closed.

**10. Changes to `app-adapters.md` — the recommendation, taken.** Each lands in
the change that first builds what reads it: `gives` and `sends` on a verb; the
kinds `menu`, `field`, `slider`, `drop_down` and `list`; fallback verbs for
typing, setting and choosing; the screen road as a mechanism (after ADR 0147);
and exit code 76 borrowed for *sent and not confirmed*. The parts answer 4
decided are amended now. Closed.

**11. Text into a field on the tree road — the recommendation, taken.** The
field's own editable text first, because it can be read back exactly and so
the record can say what went in; keystrokes where the field refuses it. ADR
0143 stage 1 measures which toolkits accept which. Closed.

**12. alo's own surfaces — the recommendation, taken.** alo reaches them
through the same tree the screen reader and the terminal use, as `alo`, under a
grant the person makes like any other. Closed.

**13. The parity test — it fails the build from day one, with the written
exception list.** For alo's own surfaces, as section 3 describes; for other
applications, the per-toolkit fixtures fail the build and the walk over the
shipped applications runs per release. Closed.

**14. What reads the screen — the person chooses in Settings.** The owner's
words: *"User chooses in settings, it can be alo services, local model,
third-parties apis."* So there are three choices, and section 2 now carries
them:

- **alo's own text recognition**, built in and working with no network and no
  model;
- **the person's local model**, where they have one;
- **a third-party service the person names** — which **sends a picture of the
  window off this machine** each time the screen is read, said in plain words
  beside the option before it is chosen, shown on the indicator as it happens
  and written in the record, as law 1 requires.

**Before anybody has chosen, alo's built-in offline recognition is used — the
owner's decision of 2026-11-06**, confirming this lane's proposal: it is the only one of the
three that works on every machine and sends nothing. **A password field is
never sent anywhere**, whichever is chosen: it is cut out of the picture before
any reader sees it. Closed.

**15. The amendments before any screen-road code — the recommendation, taken.**
ADR 0001 §6 and `app-adapters.md` are amended before a line of the screen road
is written. The text is
[ADR 0147](../decisions/0147-the-screen-road-and-who-reads-the-screen.md),
**accepted by the owner on 2026-11-06** — *"yes"* — and both amendments are
made in the change that accepted it. Closed.

**16. Clicking and dragging by position — for the person now; never for alo.**
`click` and `drag` are forms of `alo-app` from the start (section 1), with the
position given **as proportions of the window, never in pixels**, and recorded
as *done by position*. **alo never clicks or drags by position**, on any road
and at any setting. Closed.

**17. The screen road for alo — off until the person switches it on, per
application.** The person's typed commands use the screen road at once. alo uses
it for an application only after the person has switched it on for that
application; until then, asked for such a control, alo says it can only do this
by reading the screen of that application, and offers the switch. **This keeps
ADR 0143's promise** — pictures and keystrokes for alo, off by default and
switched on per application — rather than withdrawing it. Closed.

## What this page does not decide

The recogniser itself, which ADR 0147 leaves to the change that builds it; the
text kind for alo (ADR 0143 stage 5); and the grant over one application and
where a person makes it (ADR 0143, question 10). Each is its own decision, and
this contract uses whatever they settle.
