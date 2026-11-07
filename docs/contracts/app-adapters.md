# Contract — application adapters

**Status:** contract, and the one outside developers build against. Additive
changes only; a break requires versioning and a deprecation period.

An adapter turns an installed application into an agent. Blender, DaVinci
Resolve, GIMP, Inkscape, LibreOffice — each becomes `@blender`, `@resolve`,
`@gimp`, sitting alongside the workspace's own agents and obeying the same rules
(`docs/decisions/0001-the-capability-model.md`).

We will not write every adapter, and we should not try. This document is the
contract so that other people can.

## The one rule that matters most

**The adapter exposes typed verbs and generates any script internally, from
validated arguments. The model never authors code that executes.**

Most applications are scripted — Blender and Resolve take Python, GIMP takes
Script-Fu, LibreOffice takes UNO calls. It is therefore very tempting to expose
a single verb that takes a script and runs it. That verb would satisfy the
letter of "no arbitrary command" while destroying its purpose: an agent that can
write a script can do anything the person can do, in one approval, leaving a
record that says nothing useful.

So: the model chooses `resize(width, height)`. The adapter builds whatever the
application needs, from those two validated integers. An adapter that accepts
model-written code will not be accepted.

## The four mechanisms

Pick the best one the application supports.

| Mechanism | Use when | Quality |
|---|---|---|
| **The application's automation API** | It has one | Best — semantic, reliable, verifiable |
| **The accessibility tree (AT-SPI)** | No API exists | Good — a real widget tree, controls activated properly |
| **D-Bus** | The application exposes an interface | Good where present |
| **The screen** — one window's picture read, keys and clicks delivered to that window | Only for a control no better road reaches | Least reliable — it reads pictures, and afterwards nobody can say for certain what was pressed, so every act is marked in the record |

The accessibility tree is the universal floor for an application that describes
itself: readable and operable with no adapter at all, so every such application
gets *some* capability and adapters add *real* capability. **The screen is the
floor beneath that**, for a window that describes nothing.

*Edited 2026-11-06 by [ADR 0147](../decisions/0147-the-screen-road-and-who-reads-the-screen.md),
accepted that day.* This paragraph said screenshot-and-click *must be declared
in the adapter manifest … and is disabled by policy by default*. Now: **the
screen road is carried out by this machine itself, never declared by an
adapter**, under ADR 0147 §1's rules — one window, read at the moment of the act
and never kept, found by what it says, read again after, password fields never
read or sent, and every act marked in the record. alo uses it for an application
only once the person has switched it on for that application; a person's own
typed commands use it at once; an organisation may forbid it on machines it owns
(ADR 0016).

## What an adapter declares

```
adapter:
  application:  how the application is identified on the machine
  mechanism:    api | accessibility | dbus | synthetic
  verbs:        the list below
  grants:       what must be granted for any of it to be reachable
```

And each verb, exactly as in `agent-verbs.md`: `name`, `purpose`, `effect`
(`read` or `change`), typed `args` with purposes, `requires`, and how its
approval `sentence` is generated from the validated arguments.

## How this machine loads one

*Added 2026-09-16 with `crates/alo-adapters`. Additive: nothing above changed.*

An adapter is **declared data**, written as Rust constants — `alo_adapters::Adapter`
— because a verb's words are `alo_strings::Word`s (`agent-verbs.md`, *the words
are the declaration's*), and a `Word` is known when the adapter is built. Every
field is something its author writes and a reviewer reads; none of it is code the
machine runs. `crates/alo-adapters/src/text_editor.rs` is the worked example.

| Field | What it is |
|---|---|
| `name` | What the agent is called by: `text_editor` is `@text_editor`. Lower-case words joined by underscores. |
| `application` | The identifier the machine knows it by. It is also **where every message goes**: an adapter names no destination of its own, so it cannot reach another service. |
| `releases` | The release series it supports — `50` covers `50.1` and `50.2`. At least one. |
| `mechanism` | `api`, `accessibility`, `dbus` or `synthetic`. |
| `words` | Every word its verbs are declared with. A word used and not listed is refused, because it would reach a person as a key. |
| `verbs` | Each with `name`, `purpose`, `effect`, `sends`, typed `args`, `reaches`, `sentence`, `by_hand` and `carried_out`. |

A verb is known on the machine's list as `adapter.verb` — `text_editor.open_document`.

**`reaches`** is `Over` the path arguments a grant must cover, or
`OnlyItsApplication` for a verb that takes no path. Every adapter verb reaches its
application, and **the grant over the application is asked before anything is
sent**, again at the moment it would be. The written reason `agent-verbs.md` rule 5
asks for, for a verb that reaches only its application, is: *an adapter's verb that
takes no path reaches only its own application, and the grant over that application
is asked before anything is sent to it.* An agent is offered an adapter's verbs only
while it holds a grant over the adapter's application.

**`by_hand`** is how a person does the same thing in the application themselves, in
their own words (ADR 0009). A verb with none is refused. Its verbs are also held to
`docs/by-hand.md` like every other verb.

**`carried_out`**, for `dbus`, is one method call: the object, the interface, the
method, and each parameter from a closed list — a file address made from a path
argument, a literal the author wrote, text from a name or a chosen option, a number
from a count, an empty parameter list, empty platform data. **Nothing else can go
into a message**, and nothing a model sends reaches one except through a value the
capability model validated.

### What is refused when an adapter is loaded

Each refusal is a sentence naming the adapter, the verb and the argument, for the
author to act on:

- **an argument declared as a script, a command or free text**;
- **a parameter the application interprets** — declarable, so a declaration can
  be honest, and never loaded;
- a method whose interface or name says it runs something (`Eval`, `Execute`,
  `RunCommand`, a `Scripting` interface…) — a tripwire, not a boundary; review of
  the declaration is the boundary;
- **an action chosen by an argument**: `org.freedesktop.Application.ActivateAction`,
  `org.gtk.Actions.Activate` and `SetState` must name their action as a literal,
  one verb per action;
- `synthetic` in an adapter's declaration — the screen road is this machine's
  own, carried out under ADR 0147, and an adapter cannot declare it (*edited
  2026-11-06*: this read *`synthetic`, whatever else is true*); `api` and
  `accessibility`, until this machine carries them out — a verb the machine
  cannot carry out is never offered;
- an application that is a person's own (ADR 0043) — an adapter for the terminal is
  a command verb by another road;
- a path argument no grant is required over; an argument that is never sent; a
  parameter from an argument the verb does not take, or of the wrong kind; a
  malformed object, interface or method; no release; no verbs; a verb with no
  by-hand road; a word not declared; a name used twice; and a second adapter for
  one application;
- and everything `alo_capability::Verb::checked` refuses of any verb.

### What a person is told when it does not happen

After a person approves, the application may not be there, may not offer what was
asked (usually another release), may refuse, or may not answer. The first three
are refusals: nothing was done, the person is told which, and the record keeps the
same words. **An application that does not answer is recorded as having run** —
something was sent under the approval — and the person is told that whether it
happened is not known.

## The accessibility fallback, for an application with no adapter

*Added 2026-09-16 with `crates/alo-adapters`. Additive: nothing above changed.*

The fallback is not an adapter and is not declared by anyone: it is two verbs this
machine ships, and they reach an application only when it has **no** adapter. An
application with an adapter is reached through its adapter's verbs alone, because
an adapter is a narrower list someone reviewed, and pressing any control in its
windows would make that list decorative. The name `accessible` is the fallback's,
and an adapter of that name is refused when loaded.

| Verb | Effect | Takes | Sentence |
|---|---|---|---|
| `accessible.read_window` | read | `application` | *read what {application} shows in its windows* |
| `accessible.activate_control` | change | `application`; `kind`, one of `button`, `check_box`, `radio_button`, `switch`, `menu_item`, `tab`, `link`; `name`, one name of up to 200 characters | *press the {kind} named “{name}” in {application}* |

Both require a grant over the application. What an adapter author, or anyone
building on the fallback, can rely on:

- **the tree is read at the moment of the call, for that call.** Nothing is
  subscribed to, no event is registered, and the connection to the tree ends
  with the turn. A control is looked for again when it is pressed, in the window
  as it is then; nothing read earlier is used to find it;
- **an application is identified by its sandbox, never by its own name.** The
  process behind the application's connection is held and its sandbox read, as a
  portal caller is; a program nothing identifies is never a granted application;
- **a password field's contents are never asked for**, and never appear in what
  is read — only that there is a password field, and its name;
- **nothing is found or pressed by position.** No position, size or picture is
  asked for. A control with no name, an area the application draws itself, and a
  part of a window another program shows are each said in words. *Edited
  2026-11-06 by ADR 0147:* **except a person's own `click` and `drag`**, given as
  proportions of the window and recorded as done by position — **never an
  agent's**;
- **nothing is guessed.** A press is refused when no control of that kind and name
  is showing, when more than one is, when the window is too large to read whole,
  when the control is greyed out, and when it offers no way to be pressed from
  outside the application. An application that does not answer the press is
  recorded as having run and the person is told whether it was pressed is not
  known, as for an adapter;
- **reading is bounded**: at most 2,000 things, 48 levels deep, and 4,000
  characters of any one text, with the answer saying when not everything was read.

## Rules for adapter authors

1. **Be honest about `effect`.** Anything that modifies a document, a file or
   application state is a `change` and waits for approval. "It only nudges the
   layer" is a change.
2. **Type the arguments narrowly.** `width: u32` beats `value: String`. Narrow
   types are most of the validation.
3. **Never take a script, a command, an expression or a "raw" field.** If a
   capability seems to need one, the verb is too broad — split it.
4. **Generate the sentence from the arguments.** If you cannot describe the
   action from its typed arguments, a person cannot meaningfully approve it.
5. **Fail loudly and specifically.** "The document has no layer named Sky" is a
   good failure. "Error" ends a turn for nothing.
6. **Do not require the application to be visible or focused** unless the verb
   genuinely does. Stealing focus mid-turn is hostile.
7. **Test the refusal paths.** An adapter that has only been tested when it
   works has not been tested.
8. **Be honest about `sends`** — `never`, `always` or `may`: whether carrying
   the verb out sends anything off this machine. *Added 2026-11-07*, in the
   change that built what reads it (`terminal-commands.md`, answer 10):
   `alo-app show` prints *this cannot be undone once it has been sent* beside
   every verb that does not say `never`, because Put back restores files and
   reaches nobody else's computer. A message, a post or an upload is `always`;
   an action whose sending depends on the application is `may`.

## Versioning

An adapter declares the application versions it supports. A verb's name and
meaning are stable once published: adding verbs and adding optional arguments is
additive and always allowed; removing a verb, renaming one, changing its
`effect`, or making an optional argument required is a break, and needs a
version and a deprecation period.

Where an application's own API changes underneath, that belongs in
`docs/quirks.md` with the version, the behaviour and the date — the next author
should inherit the knowledge, not the debugging session.

## Distribution and trust

An adapter is code that runs on a person's machine and drives their
applications, so it is treated as such: adapters are signed, they declare their
mechanism and grants up front, and policy can restrict which adapters may load
on a machine or across a fleet. An unsigned adapter can be permitted explicitly
for development and never by default.

## Amended 2026-11-06 — alo reaches an application's controls and what it announces, as well as its adapter

*The owner's answer of 2026-11-06 to question 4 of
[`terminal-commands.md`](terminal-commands.md), under the promise* Three equal
roads *(`docs/features.md`, #1101): a control a person can use on screen that
alo cannot be asked for is a bug. Additive for adapter authors: nothing an
adapter declares changes meaning, and nothing that loaded before is refused
now.*

**1. The accessibility fallback reaches an application with an adapter too.**
Above, *an application with an adapter is reached through its adapter's verbs
alone*. That sentence is replaced: alo is offered an adapter's verbs **and**
the fallback's verbs over the same application, under the same grant. What the
narrower rule protected — a reviewed list not made decorative — is kept by
saying which is which: the sentence a person approves for a fallback press says
it is *a press of a control named “{name}”*, never in the words of a reviewed
verb, and the record names the road.

**2. Abilities an application announces on the session bus are a road for alo
as well as for the person.** `terminal-commands.md` section 2 names them:
`org.freedesktop.Application.Open`, and each action listed by `org.gtk.Actions`
with no parameter or one of a simple type, asked of the application at the
moment they are wanted and kept nowhere. **They are not adapter declarations**,
and the refusal above of *an action chosen by an argument* still binds every
adapter: a declared verb names its action as a literal. A discovered ability is
a separate road, offered to alo under its own rules — only over an application
the person has let alo use, one generated sentence and one OK per act unless the
person chose full trust, recorded with its road — and the sentence says plainly
that *the application describes this itself, and alo cannot tell what it
changes or whether it sends anything*.

**3. The screen road, amended in place.**
[ADR 0147](../decisions/0147-the-screen-road-and-who-reads-the-screen.md) was
accepted by the owner on 2026-11-06 (*"yes"*), and its edits to this page — the
mechanisms table's last row, the paragraph under it, the refusal of `synthetic`,
and *nothing is found or pressed by position* — are made above, each marked with
the date. They were made before any screen-road code, as the owner's answer 15
required. **Nothing of the screen road is built.**
