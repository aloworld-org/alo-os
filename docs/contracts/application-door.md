# Contract — what `alo-agentd` asks the compositor about an application

**Status:** contract. Additive changes only; a break requires versioning and a
deprecation period. See `CLAUDE.md`, "Contracts outlive code".

**Every part of this door is built; none of it has been walked on a booted
machine**, and the table says which is which. A page that read as finished
would be the measurement-reported-as-a-property fault in document form.

| | |
|---|---|
| The vocabulary and its tests | **built** (`crates/alo-acting`) |
| The compositor answering the door | **built** (`alo-shell`) |
| The desktop serving it at `/run/alo/<uid>/shell.sock` | **built 2026-10-31** (`alo_shell::the_door_on_the_desktop`); only an example served it before |
| The compositor carrying the acts out | **built**, not yet walked against a real client |
| The daemon offering the verbs and asking | **built** (`alo-agentd`, `alo_turn::acting_on_an_application`) |

*This table said until 2026-10-31 that the daemon half was not built and that
nothing asked the door. Both had stopped being true: the daemon offered all
three acts and knocked — and the desktop a person signs in to served no door,
so every approved act was answered* could-not-ask. The fourth row carries its
own limit — which window to reach is arithmetic over the window list and is
unit-tested, but *reaching* one needs a real compositor and a real client,
which is the nested fixture's road and `examples/a_real_application.rs`'s, and
has not been walked for these three acts.

## How an `open` starts an application — the owner's decision of 2026-10-31

**An approved `open` starts the application exactly as a person's press on the
Dock does** ([ADR 0108](../decisions/0108-the-compositor-carries-an-application-verb-out-and-needs-no-token-to.md),
dated section): over the session bus where its desktop entry says
`DBusActivatable=true`, and from the entry's own `Exec` line otherwise, as a
unit of its own in the person's systemd manager handed alo's display. Until
that day it was bus activation only.

**Nothing on this wire changed.** The request is still `open <application>`
and the answer still one word. What runs is still only what the installed
application's own entry names, found by its identifier — there is no field for
a command to arrive in. The daemon reads the same entry before it asks, so its
record says which road starts the application, and an entry that gives no way
to start is refused by name without the door being asked.

This is what goes across the compositor's door in both directions: the one line
`alo-agentd` writes to have something done to an application's window, and the
one line it is told back. It is the **acting half** of the application verbs —
`docs/contracts/agent-verbs.md` publishes the verbs themselves, and that
document is what a person's agent is written against. This one is for whoever
writes a compositor that is not ours.

Read [ADR 0108](../decisions/0108-the-compositor-carries-an-application-verb-out-and-needs-no-token-to.md)
first: it decides the door's shape and argues the one thing this contract turns
on, which is that no token crosses it. `crates/alo-acting` is this document as
working code, and it names no path — the path is below and belongs here.

## What this door is for, and what it is not

`alo-applications` decides all four application verbs: the declarations, the
typed arguments, the approval sentence a person reads, and whether the thing
asked about is an application this machine has. By the time a line reaches this
door, **the grants have been asked and the person has approved a sentence.**

So the compositor is a hand and never a second judge. Three answers belong to
three different owners, and this door is only the third:

| question | whose answer |
|---|---|
| Which applications exist? | `alo_applications::Installed` |
| May this verb run at all? | the turn, in `alo-agentd` |
| Which windows are there, and what happened? | the compositor |

**A compositor that answered the second question would be a second place
deciding it**, so there is no *not permitted* on this wire and there must never
be one. Every refusal below is a fact about the machine.

## Why no token crosses it

`alo-broker`'s door takes five words and an HMAC over the exact verb, argument,
approval and moment, good once and for sixty seconds. It needs that because it
runs as **root**: a root service must not act merely because a process of the
person's said so.

**There is no privilege gap here.** The compositor runs as the person. Asking it
to close a window is something any of the person's own processes can already do
on any desktop, so a door that admits the person's uid hands out nothing the
person did not already have. A token would be this machine proving something to
itself across a boundary it already controls — ceremony that reads as a
guarantee.

**And the agent cannot reach it.** The person and the agent are two Unix logins
(`daemon-protocol.md`; ADR 0001 §5), and the daemon refuses to start on a
machine where they are one. A door that admits only the person therefore admits
the daemon and refuses the agent, decided by the kernel, with nothing on the
wire taking part in the decision.

## The transport

A **Unix domain socket**, one per signed-in person, beside the daemon's own:

```
/run/alo/<uid>/shell.sock
```

where `<uid>` is the person's login number. Both names are part of this
contract: `alo` for the directory, which the image makes at boot, and
`shell.sock` for the socket. A caller finds it by that path and by nothing
else — there is no port, no bus name and nothing to discover.

**Three things own three parts of that path, and only one of them is new.**

- `/run/alo` is the **image's**, made at boot through `tmpfiles.d`, `0755` and
  owned by root.
- `/run/alo/<uid>` is the **daemon's**, made when that person's session starts
  and taken away when it ends, `0750`. ADR 0017 gives it to the daemon, and
  `daemon-protocol.md` says so. **The compositor never creates it and never
  sets its mode.** A directory two services both made would have whichever
  mode lost the race, and the mode of the directory a socket lives in is the
  thing that decides who can replace the socket.
- `/run/alo/<uid>/shell.sock` is the **compositor's**, made when it starts and
  removed when it stops.

So a compositor finding that directory missing **serves no door** and says
which directory is missing and who makes it, exactly as the daemon does for
`/run/alo`. A window system is not a thing to make start depending on a
service; it keeps running without the door, and the verbs answer
`could-not-ask`. That is also the honest reading of the situation: there is no
daemon, so no turn can have authorised anything anyway.

**The socket is `0600`, and the difference from the daemon's `0660` is the
whole point.** The daemon's socket is handed to the group the agent is in,
because the agent has to reach it. This one is handed to nobody: only the
person's own uid may connect, and the agent is a different login. A compositor
that widened this mode would undo the only thing protecting the door.

Before binding, the compositor refuses a path that is a symbolic link, is not a
socket, or belongs to somebody else — and refuses the **directory** on the same
three grounds, without following a link to answer about it. Whoever owns the
directory a socket lives in can replace the socket, and the daemon would then
be talking to them.

**Credentials are read from the kernel before a byte is read.** `SO_PEERCRED`
answers with the process, user and group of the peer as the kernel recorded them
when the connection was made. It is not a field in a message and a caller cannot
set it. A connection from any other uid is **closed with nothing written on it**,
which is `daemon-protocol.md`'s rule for a stranger and its reason: an answer
tells whoever knocked that there is an alo OS here.

**A message is a line.** One request, then one answer, terminated by a newline.
A request is at most **512 bytes**, so that a caller cannot make a window system
hold a line open while it reads it.

## The request

Two words separated by one space:

```
<act> <application>
```

`alo-broker`'s line is five words because a root service needs an approval
number, a moment and a proof. This one needs none of them, and the section above
is why.

**Two words and not JSON, because there is nothing to nest.** An act is one of
three words and an application is one identifier held to its own shape, so a
line that parses is a line that is complete: there is no field to forget and no
field to smuggle.

A line that is not *exactly* two words is **not a request**, rather than a
request with something missing. That is `daemon-protocol.md`'s rule one level
up — *a field nobody declared is refused rather than ignored … rather than
having the part it cared about dropped* — and it is cheaper to keep here than to
recover from.

### The acts

The list is closed. A verb name is never text the compositor looks up, which is
law 2 at the one place a caller can reach.

| act | what it means | may start the application |
|---|---|---|
| `open` | bring this application up, starting it if it is not running | yes |
| `focus` | put an existing window of it in front | no |
| `close` | ask it to close | no |

### The application

One application identifier, held to `alo_applications::Application`'s own
shape: surrounding whitespace is trimmed, and what is left must be non-empty
and contain no control character, no whitespace, no `/` and no `\`. There is no
length limit of its own — the 512-byte line is the only bound.

It is **not** held to the D-Bus well-known bus name format, although the
identifier of an application the compositor starts over the bus is one.
`docs/quirks.md` has the measurement and the reason: a desktop entry's
identifier is the file's own stem, and on an ordinary machine 14 of 15 of them
are not bus names.

Shell punctuation is therefore admitted, and it is not an injection: every
consumer hands the identifier over as one element of an argument vector or as a
bus name, and neither composes a shell line. *There is no field for a command to
arrive in* is this repository's rule and it holds here.

## The answer

One word.

| answer | what it means |
|---|---|
| `acted` | **this machine asked**; see below |
| `no-window` | that application has no window here |
| `not-installed` | that application is not installed here |
| `it-did-not` | the application was asked and has not done it |
| `could-not-ask` | this machine could not ask the application |

**None of these means *not permitted*.** That answer is given long before, at the
daemon, and a door that could give it would be a second place deciding it.

**And `acted` means *asked*, never *done*.** Not a weakening — it is what
`agent-verbs.md` already promises a person about the third act:
*`close_application` asks; nothing kills anything … the application is asked, it
may put up its own save your changes?, and the person answers that.* The
compositor's own `request_window_close` says the same one layer down: *success
means one XDG close event was queued … it does not mean the application received
it or closed.* An application that declines is `it-did-not` where the compositor
can tell, and where it cannot, the record is the person's own: a window that is
still there.

**Which refusals an act can honestly meet.** One act is constrained and two are
not:

- **`open` never answers `no-window`**, because having no window is the ordinary
  reason to open something rather than a reason not to. A compositor that
  answered it has reported a refusal where it should have started an
  application.
- **`open` on an application that is already running brings it up and starts
  nothing.** It is the same thing a press on a Dock icon does with one window
  open; an `open` that started a second copy would be a verb doing something
  nobody approved a sentence for.
- **`focus` never starts anything.** That is `open`'s, and a `focus` that
  started an application would carry out a different sentence from the one a
  person read.
- **`not-installed` is a word alo OS's own compositor never says**, and that is
  honest rather than a gap. Which applications exist is
  `alo_applications::Installed`'s answer and the compositor has no business
  holding it — *three answers, three owners*, above. An `open` for something not
  installed is asked for, the bus finds no name to start, and the person sees
  nothing open: the same outcome by a road that keeps one question in one place.
  The word stays in the vocabulary because a compositor that *does* know may say
  it, which is why `could_answer` permits it rather than requiring it.
- **`focus` and `close` may answer any of the four**, and are deliberately not
  narrowed. *No window* is the usual answer, but a compositor may honestly know
  more than that — that the application is not installed at all, or that a
  window was asked and would not go — and a contract that forbade the better
  answer would get the vaguer one.
- `it-did-not` is told apart from `could-not-ask` because an application
  entitled to ask the person *are you sure* is an application that has not
  closed, and that is not a failure.
- `could-not-ask` is the only one worth reading a log about. The other three are
  ordinary answers about an application, and a machine that logged them as
  faults would bury the one that is.

`alo_acting::answer::could_answer` is these four paragraphs as working code, and
it is what a compositor is held to rather than this prose.

**A refusal is a kind and not a sentence**, and that is deliberate. What a person
reads about an application verb is `alo-applications`' own sentence, rendered in
their language before the verb ever reached a window. A sentence here would be a
second wording of something already said, in whatever language the compositor
happened to think in. One vocabulary, one place.

**An unknown answer is never read as success.** A caller that receives a word it
does not know treats it as a compositor newer or older than itself, and never as
*it worked*. The asymmetry is the point: a door whose unknown answers were
optimistic would report work that never happened.

## What is not here

**`arrange`.** `docs/features.md` promises four application verbs and this door
carries three. `open`, `focus` and `close` are a window named and a thing done;
`arrange` takes a **where** — a side, a half, a whole screen — and where a window
goes on a screen is `alo-arranging`'s answer and the design's, not this door's.

It is also blocked on something real rather than merely unscheduled: the
compositor's single-window half was removed on 2026-09-26 so that there would be
*exactly one layout decider*, and what replaced it divides **between** two
windows (`native-window-dividing.md`). The chord that put one window on half a
display has no answer any more. Reinstating it or respelling `arrange`'s options
is a decision and not an addition to this contract.

**A list to pick from.** A person pressing a Dock icon for an application with
three windows is offered one, because they are standing there to choose. An
agent is not, so no act ever opens a picker — and what happens instead is
decided rather than left to the compositor:

- **`open` and `focus` bring forward the window the person used most recently**
  — the one they last looked at, which is the only choice among the windows that
  is about the person rather than about the list's order.
- **`close` asks every window of that application, oldest first.** *close Geary*
  means Geary and not one of its windows, and a person who approved *ask Geary
  to close* did not approve leaving two of its three open. Oldest first so that
  a second run of the same act asks the same windows in the same order, and so
  that an application putting up *save your changes?* puts it up for the window
  the person opened first. A window that cannot be asked does not stop the
  others: `acted` if any was asked, and `it-did-not` if none could be — there
  *were* windows when the list was made, so `no-window` would say something
  false.
- **A window belonging to no application is reached by no act.** A client may
  map a toplevel having set neither a class nor a title, so
  `alo_dock::Window::app` is an `Option` by the owner's direction — *do not
  substitute a fabricated app identity* — and two such windows must not collapse
  into one application an act could name.

**A session.** One line is one act. There is no connection that stays
authorised, because nothing about this door is an authorisation in the first
place — the authority was the turn's and it was spent before the line was
written.
