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
A request is at most **512 bytes** (an `open-link` and a `choose-link` line excepted, below), so that a caller cannot make a window system
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

## Counting what an application sends — added 2026-11-07 (ADR 0148)

**What it gives a person:** while a command they typed makes an application do
something, the indicator shows that the application is sending — *Thunderbird
is sending, during your command* — and the command is told how much it sent.
[ADR 0148](../decisions/0148-the-kernel-counts-what-an-application-sends-during-a-persons-command.md)
is the decision; this is the wire.

**The one act on this door that holds its connection**, because the count lasts
exactly as long as the command, and a connection is the one thing a command
cannot forget to end:

```
→ count <application>
← counting | not-installed | cannot-count
  … the command does its work …
→ done
← sent <bytes> | stopped <bytes>
```

- The first line is `count`, one space, and one application identifier held to
  the shape above. A line beginning `count` that is not exactly that is not a
  request: it is closed with nothing written.
- `counting` means the count has begun; `not-installed` that no desktop entry
  names the application; `cannot-count` that this machine has no counter (the
  boundary was loaded without one), that it is already counting as many
  commands as it can, or that no part of the application could be counted.
  **A caller never reads anything but `counting` as a count having begun.**
- `done` is the only thing a caller says after that. Anything else, or the
  connection closing, ends the count with nothing written back.
- `sent <bytes>` is the total, counted by the kernel at the application's own
  sockets, headers included. `stopped <bytes>` means **the count did not cover
  the whole command**: the desktop ended it after two minutes, or part of the
  application could not be counted. `stopped` may arrive before `done` is said.
- **Who answers is the desktop that draws the indicator**, not the act
  carrier: a desktop with no indicator answers every count `cannot-count`.

Nothing an agent sends reaches this act: `alo-agentd` writes only `open`, `focus`
and `close`, and the answer is the person's own command's to read.

## alo working in an application — added 2026-11-10 (ADR 0143 Q6)

**What it gives a person:** when alo works in an application — presses
*Publish* in an editor, say — what that application sends is on the indicator
while alo works, never unshown. **Nothing asks this yet**: no turn drives an
application until stage 4 of the Demo 2 plan, so a person sees none of it today.
[ADR 0143](../decisions/0143-alo-works-inside-applications-and-may-post-in-the-persons-name.md)
Q6 and the owner's answers of 2026-11-10 are the decision; this is the wire.

Like a count, it holds its connection for as long as alo works:

```
→ work-in-alos-copy <application>        (the default setting)
← counting | cannot-count | no-copy-of-its-own | not-installed
  | could-not-start | cannot-show | showing (turned up, no copy)
→ work-in-their-window <application>     (the setting turned up)
← showing | not-installed | cannot-show | not-chosen (at the default)
  … alo works …
→ done
← sent <bytes> | stopped <bytes> | uncounted
```

- `work-in-alos-copy` starts **alo's own copy** of the application, with a
  profile of its own (`alo_launching::alos_copy`), or finds the one already
  running, and counts **only that copy** — its scope, `app-alocopy-<id>@<n>.scope`,
  and the scope Flatpak moved its process into. **The person's own windows of
  the same application are never counted**, and a person's own `count` no longer
  counts alo's copy either.
- `counting`: the copy runs and is counted; the line *{application}, in the copy
  alo is working in, is sending* is up from the first byte. `cannot-count`: the
  copy runs and this machine cannot count it; the line *alo is working in its own
  copy of {application}, and this machine cannot count what it sends* is up for
  the whole time. A `counting` copy whose groups are not found within two
  seconds, or a part of which cannot be counted, becomes the second line at once.
- `no-copy-of-its-own` is **the owner's answer A**: the application cannot open
  a separate copy for alo — it is not on alo's reviewed list, or it is started
  only over the session bus — so alo may not work in it at this setting, and the
  turn names the Settings choice that lets it use the person's own window, in
  `alo_capability::words::NO_COPY_OF_ITS_OWN`: *choose “alo may work in your
  own window” in Settings → Folders alo can use*.
- **Which setting it is, is the person's** (added 2026-11-10, additively):
  `in-applications` in `/var/lib/alo/how-code-runs.toml`
  (`how-code-runs-file.md`), chosen on Settings → Folders alo can use and read
  by the desktop at each ask — never taken from the word alo sent. **At the
  default** (`in-alos-copy`, or no answer, or a file that will not read),
  `work-in-their-window` is answered `not-chosen` and nothing is started,
  shown or counted. **Turned up** (`in-their-window`), `work-in-their-window`
  is answered `showing`, and `work-in-alos-copy` still uses alo's copy where
  the application has one and is answered `showing`, in the person's window,
  where it has none.
- `work-in-their-window` counts nothing at all. `showing`: the line *alo is
  working in your {application} window, which may be sending, and alo cannot
  count what* is up for the whole time.
- `cannot-show`: this desktop has nowhere to draw a line, so alo may not work.
  **Only `counting`, `cannot-count` and `showing` let alo begin**
  (`alo_acting::Working::may_work`).
- `done` is the only thing said after that. `sent <bytes>` is what alo's copy
  sent while alo worked in it; `stopped <bytes>` that the desktop ended it first,
  after fifteen minutes, and a turn that works longer asks again; `uncounted`
  that nothing was counted. Closing the connection takes the line down.

## A link the person followed — `open-link`, added 2026-11-12

**What it gives a person:** a link they click in their mail, a document, a
code editor or a terminal application opens in their default browser — or, for
an email address, a new message in their default mail application — and the
window comes up on the canvas, in front, as a window they opened does. The
owner's decision, quoted as chosen: *"Yes, add it (Recommended)"* — a narrow
connection by which the portal service asks the desktop to start the chosen
application **exactly as a person's press does**. Which application, and
whether the link may be opened at all, is decided before this line is written,
by the open-with portal (`portals.md`, `OpenURI`), and recorded there.

```
→ open-link <application> <link>
← acted | not-installed | could-not-ask
```

- **Three words.** The act, one application identifier held to the shape
  above, and one link: `http:`, `https:` or `mailto:`, on one line, with no
  space in it, at most 2048 bytes, and a web link with no name and password in
  it (`alo_applications::Link`). A line beginning `open-link` that is not
  exactly that is not a request and is closed with nothing written.
- **The one act longer than 512 bytes.** An `open-link` line is at most
  **2560 bytes**, because a link alone may be 2048; the door reads up to that
  many before a newline, and every other act's own reader still refuses a line
  longer than 512.
- **`acted` means asked**, as for `open`: the desktop has taken the start and
  sends it off the frame loop. The application is started with the link the way
  a press starts it — `org.freedesktop.Application.Open` with the link over the
  bus where its entry says `DBusActivatable=true`, and otherwise its own
  `Exec` line with `%u` or `%U` filled with the link as one argument. An
  application already running is started again with it, which a browser answers
  by opening a tab: its own single-instance handling. **An entry with nowhere
  to put a link — no `%u` and no `%U` — is not started at all**, rather than
  started without the link, and the desktop's journal says so.
- `not-installed` is permitted and alo OS's own desktop never says it, for the
  reason `open` never does. `could-not-ask` is the machine.
- **The link can never be an option or a command.** It begins with a letter, so
  no program reads it as `--something`; it has no space, so it is one argument;
  and it is never handed to a shell.
- **Who writes it.** The open-with portal service, `alo-portald`, which runs as
  the person — so the door's `SO_PEERCRED` check admits it, and admits nobody
  else. **Nothing an agent sends reaches this act**: `alo-agentd` writes only
  `open`, `focus` and `close`, and the agent is a different login. A link the
  agent wants opened is an agent action and goes through the agent's own verbs.

## The person chooses what opens a link — `choose-link`, added 2026-11-12

**What it gives a person:** when an application asks for them to choose —
`ask` on `OpenURI` — the desktop shows the *Open with* sheet the owner approved
on 2026-11-12 (frame `25:52`, `docs/design/default-apps-and-open-with.md`):
*&lt;App&gt; asked to open &lt;address&gt;*, the installed applications that
open that kind of link with their default already chosen, and *Cancel* and
*Open*. Without `ask` no sheet appears and `open-link` above is all there is.

**The one act on this door that waits on the person**, so it holds its
connection while the sheet is up:

```
→ choose-link <asking-application> <link> <default> [<another> …]
← chosen <application> | cancelled | unanswered | cannot-show
```

- **The words.** The act; the identifier of the application that asked, as its
  sandbox named it; one link held to `open-link`'s shape; then the
  applications offered, **the person's default first** and every other
  installed handler of that kind after it in the order they declared it, each
  an identifier held to the shape above and each once — **at least one, at most
  sixteen**. The line is at most **4096 bytes**. A line beginning `choose-link`
  that is not exactly that is not a request and is closed with nothing
  written.
- **Who decides what is offered is the portal**, from the same answers
  Settings → Default apps draws (`alo_applications::WhatOpensWhat`'s
  `what_could_open_links` and `what_opens_links`), and a terminal is never
  among them. The desktop draws what it is sent and offers nothing else.
- `chosen <application>`: the person pressed *Open*, or Enter, on that one,
  which is always one of those offered. **It starts nothing.** The portal
  records the choice and then asks `open-link` for it, so the application is
  started exactly as every other link's is, and a caller never reads `chosen`
  of an application it did not offer as anything but a fault.
- `cancelled`: the person pressed *Cancel* or Escape. Nothing opens.
- `unanswered`: nobody answered while the sheet was up for **five minutes**,
  and the desktop took it down. Nothing opens. Not `cancelled`, because the
  person did not say no.
- `cannot-show`: this desktop cannot ask the person now — no person is signed
  in at it, or four other sheets are already waiting. Nothing opens.
- **One sheet at a time**, in the order they were asked; up to four more wait
  behind it, each with its connection held.
- **Nothing here changes the default.** That is chosen in Settings → Default
  apps and nowhere else; what the person picks on the sheet is for this one
  link.
- **A caller that hangs up takes the sheet down**, with nothing opened.
- Nothing an agent sends reaches this act, for `open-link`'s reason: the
  portal service writes it, as the person, and `alo-agentd` writes only
  `open`, `focus` and `close`.

## The window you were working in — `front`, added 2026-11-07

**What it gives a person:** typing `alo-app front press button "Save"` in a
terminal acts on the window they were just working in, without looking up which
application it belongs to (`terminal-commands.md`, *front*). This is the
compositor's half: which window that was.

```
→ front
← front <window-number> <application> <title> | none | unnamed
```

- `front` is alone on its line. `front` followed by anything is not a request
  and is closed with nothing written.
- *This terminal* is whatever has the person's keyboard when the line arrives,
  and it must be a terminal: alo's own, or a window whose application is one of
  the terminals `alo_capability::A_PERSONS_OWN` lists. Anything else means the
  command was not typed in a terminal — a script a timer started — and the
  answer is `none`.
- The answer is **the most recently focused window that is not that terminal
  and is still open**: the compositor's number for it (stable for its life,
  never reused), the identifier it gave itself, and its title, cleaned of
  control characters, cut to 200 characters, and possibly empty.
- `none` — nothing had the keyboard before this terminal, or what has it is not
  a terminal. `unnamed` — the window before it cannot be named: it gave no
  identifier, or it is alo's own terminal, which is not an application.
  **Another window is never answered instead**: a window that closed is not
  replaced by the one before it.
- **What is remembered to answer this**: who had the keyboard, most recent
  first, at most thirty-two changes, in the compositor's memory for the
  session. It is never written down or sent anywhere, and a window that closes
  drops out of it.

## Where alo's own surfaces are — `alo-tree`, added 2026-11-07

**What it gives a person:** `alo-app alo …` reaches Settings, the Dock and the
record through the tree a screen reader reads, and reaches **alo's** — not a
program that calls itself alo. The shell is not sandboxed, so nothing on the
accessibility bus tells its tree from another unsandboxed program's; the
compositor publishes the tree, so it is the one that can say which it is.

```
→ alo-tree
← at <unique-bus-name> | none
```

- `alo-tree` is alone on its line; anything after it is not a request.
- `at :1.42` names the connection the shell's tree answers as on the
  accessibility bus — always a unique name, which no other program can hold.
- `none` — the shell publishes no tree: no accessibility bus answered when the
  session began.

## alo typing in an application — `type-into` and `save-in`, added 2026-11-13 (ADR 0153)

**What it gives a person:** alo writes an approved text into the application
alo is working in, as key presses, and saves it once it has read the text back
— the keyboard road of [ADR 0153](../decisions/0153-alo-types-by-keys-in-its-own-copy-and-reads-the-text-back.md).
Additive: nothing above changes meaning.

```
→ type-into <application> <bytes>
→ <exactly that many bytes: the text, UTF-8>
← typed | not-in-front | not-alos-copy | no-keyboard | cannot-type <code point>
→ save-in <application>
← asked-to-save | not-in-front | not-alos-copy | no-keyboard
→ may-type-into <application>
← may | not-in-front | not-alos-copy | no-keyboard
```

- `may-type-into` makes **every check below and sends nothing** — for typing
  that goes through the accessibility tree rather than the keyboard, which is
  held to the same rule about whose window it is. Added 2026-11-13, additively.

- **Only to the window that holds the keyboard.** Both are carried out only when
  the window that has the keyboard now is a window of `<application>`;
  otherwise `not-in-front`, and nothing is sent to anybody. The compositor
  never moves the keyboard to carry them out.
- **alo's own copy at the default.** At the default `in-applications` setting
  (`how-code-runs-file.md`) that window must also be **alo's own copy** of the
  application — its process in the copy's scope or the scope Flatpak moved it
  into — or the answer is `not-alos-copy`. Turned up, the person's own window
  is accepted too. As for `work-in-…`, the setting is read by the desktop at
  each ask and never taken from the line.
- **To that client alone.** The text goes to the keyboard of the client that
  owns that window and no other: a layout of its own holding every character
  of the text, then each key pressed and released, then the person's own layout
  again, in one run on the compositor's thread, so no key of the person's lands
  between them. A new line is Return and a tab is Tab. `save-in` is `Ctrl+S`
  delivered the same way.
- `cannot-type <code point>`: the text holds a character no key can carry — a
  control character other than a new line or a tab — named as `U+…`. Nothing
  is typed.
- `no-keyboard`: this desktop has no keyboard on its seat to deliver through.
- **The length is the caller's to say and this door's to believe.** The text
  has no limit (ADR 0001's amendment of 2026-11-13); the bytes are read across
  frames as they arrive, and a caller that stops sending is dropped as any
  caller part-way through a line is. Bytes that are not UTF-8 are a caller with
  a bug: the connection is closed with nothing written.
- **This door decides nothing about permission**, as for every act: whether
  alo may type, into which control, and whether the text was approved, were the
  turn's to decide before the line was written. The door checks only where the
  keyboard is, at the moment of delivering.

## alo opening a file in an application — `open-file`, added 2026-11-14

**What it gives a person:** *open hello.txt in VSCodium*, once they approved
it, opens that file in VSCodium — in alo's own copy at the default, so what alo
types into it next is typed there, or in their own window where they chose
*alo may work in your own window*. The owner's decision, quoted as chosen:
*"Yes, add it"*. `alo-agentd` writes it for an approved
`open_file_in_application` (`agent-verbs.md`), after the grants, the installed
list and the file's real place have all answered.

```
→ open-file <application> <path>
← opened-in-alos-copy | opened-in-their-window | not-installed
  | takes-no-file | no-copy-of-its-own | could-not-start
```

- **The path is everything after the identifier**, so a file name with a space
  in it is one path. It is absolute — on this machine it begins with `/`, so it
  is never read as an option — and holds no control character. The line is at
  most **4096 bytes**, the most the door reads of any line. A line beginning
  `open-file` that is not exactly an identifier and such a path is not a
  request and is closed with nothing written.
- **Where it opens is read by the desktop, never sent.** At the person's
  default (`in-applications` in `how-code-runs-file.md`) it opens in alo's
  own copy: the copy is started with the file, or, already running, is handed
  it by a start over the copy's own profile. An application with no copy of its
  own is answered `no-copy-of-its-own`, and nothing is opened. Where the person
  chose their own window, it is started the way their press starts it.
- **What runs is the installed entry's own `Exec` line**, with the file where
  its `%f`, `%F`, `%u` or `%U` stands — inside Flatpak's own
  `--file-forwarding … @@ %F @@`, which is how the sandbox is handed that one
  file — or `org.freedesktop.Application.Open` with its `file://` address over
  the bus. **An entry with nowhere to put a file is answered `takes-no-file`**
  and is not started without it.
- **The answer is where, not only whether**, so the record says which: an
  opening is a change that ran, and the person may want to know whose window it
  is in.
- **Unmeasured on a booted machine.** The arguments that hand a file to a
  running copy are each application's own documentation, as alo's copy's
  arguments are (`alo_launching::alos_copy`).

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
