# ADR 0147 — The screen road, and who reads the screen

**Status:** **accepted, 2026-11-06, by the owner**, in one word: *"yes"* —
after a simpler explanation of it, relayed by the coordinator. Proposed the same
day by the lane drafting
[`docs/contracts/terminal-commands.md`](../contracts/terminal-commands.md), on
the owner's answers of that day; the decisions are the owner's and the wording
is the lane's. **The amendments in §5 are made in the change that accepts it**
— to ADR 0001 §6 and to `app-adapters.md` — so no document on `main` still
refuses the screen road; the owner's answer 15 required them before any
screen-road code. **Nothing of the screen road is built.**

*Number: 0146 is* a person may switch watching on *on `main`. No branch or pull
request on the remote adds a 0147, checked on 2026-11-06 after a fetch of every
pull request's head. A second branch, `app/alo-studio`, also adds a 0146 of its
own; that collision is not this record's and is named here so it is not lost.*

**Amends:** [ADR 0001](0001-the-capability-model.md) §6, and
[`docs/contracts/app-adapters.md`](../contracts/app-adapters.md)'s refusal of the
`synthetic` mechanism and its rule that *nothing is found or pressed by
position*.

## What it changes for the person

Today, an application that describes nothing about its window to the
accessibility tree — a game, a canvas, many applications built on their own
toolkit — cannot be used from the terminal or by alo at all. After this, **every
window on the machine can be**: alo OS draws every window and carries every key
and click, so it can read what a window shows and press, type and click in it.
It is the least reliable way to do it, so it is used only where nothing better
exists, and every act done this way is marked in the record. **The person
chooses what reads the screen** — alo's own built-in recognition, their own
local model, or an outside service they name — and an outside service is told
plainly to send pictures of the window off the machine.

## Why

The owner, 2026-11-06: *"Not only alo applications but all installed
applications should be able to be fully used by the user via ui, terminal and
ask alo."* `docs/features.md` (#1101) now makes that a guarantee for every
installed application, reached *where an application offers none of them,
through the screen itself*. ADR 0001 §6 already lists this road last — *Poor:
fragile, and unauditable* — and permits it *only where nothing better exists, …
marked as such in the record, and can be disabled by policy*. `app-adapters.md`
went further and refused it *whatever else is true*. The guarantee cannot be
kept while that refusal stands.

## The decision

### 1. The screen road exists, under these rules

ADR 0001 §6's fourth row is read, from this record on, as:

| Mechanism | When | Quality |
|---|---|---|
| The screen: one window's picture read, and keys and clicks delivered to that window | **Only for a control no better road reaches** | Least reliable: it reads pictures, and afterwards nobody can say for certain what was pressed — so every act is marked in the record |

And every use of it keeps all of these, which are the reason it is safe enough
to offer:

- **One window.** The compositor composes the named window alone for its
  picture, so nothing in front of it or behind it is in it, and delivers keys
  and clicks to that window and no other.
- **At the moment of the act, and never kept.** The picture is taken for one
  command or one action and thrown away when it ends. Nothing is subscribed to,
  and no picture is taken between acts. *Context is offered, never watched*
  holds by construction.
- **Found by what it says.** A control is found by the words it shows, as on the
  accessibility road; where two things say the same words, where words cannot
  be read, or where the window is covered or off the screen, the act is refused
  rather than guessed.
- **Read again after.** After every act the window is read again and what
  changed is said; where nothing visibly changed, the act is recorded as having
  run with its outcome *not known*.
- **Password fields are never read and never sent.** A field the toolkit draws
  as hidden characters is cut out of the picture before any reader sees it, and
  text typed into it is never recorded — only the field and the number of
  characters.
- **Marked in the record.** Every entry for an act on this road carries the road
  `screen`, and every surface that shows the record says it was *done by reading
  the screen*.
- **Policy can forbid it** on a machine an organisation owns (ADR 0016), as §6
  already says. On a person's own machine it is never removed from them.

### 2. Who may use it

The owner's answer 17, 2026-11-06:

- **A person's own typed command** (`alo-app`) uses it at once. Law 2 binds the
  agent, not the person.
- **alo** uses it for an application **only after the person has switched it on
  for that application**, in Settings, with its cost in ADR 0143's words — *alo
  will act on what it sees in pictures of this window, and may press the wrong
  thing.* Off until then, per application, which is the promise ADR 0143 and
  `docs/features.md` already made and this record keeps rather than
  withdraws. Until it is on, alo asked for such a control says it can only do
  this by reading the screen of that application, and offers the switch.
- Every act alo takes on this road is still one sentence and one OK, unless the
  person chose full trust.

### 3. By position: the person only

The owner's answer 16, 2026-11-06: **a person may click and drag by position**,
because a stroke on a canvas or a drag in a 3D view is a place and not a named
control. A position is given **as proportions of the window, never in pixels**,
and each such act is recorded as *done by position*. **alo never clicks or drags
by position**, on any road and at any setting.

So `app-adapters.md`'s *nothing is found or pressed by position* stays true of
every act alo takes, and of every named act a person takes; the one exception is
a person's own `click` and `drag`, named as such.

### 4. Who reads the screen: the person chooses

The owner, 2026-11-06: *"User chooses in settings, it can be alo services, local
model, third-parties apis."*

| Choice | What leaves the machine | Said beside it in Settings |
|---|---|---|
| **alo's own recognition** — built in, pinned and configured like every engine here, working with no network and no model | nothing | *Reads the screen on this machine. Nothing is sent.* |
| **the person's local model** | nothing | *Reads the screen with your model, on this machine. Nothing is sent.* |
| **a third-party service the person names** | **a picture of the window, every time the screen is read** | *A picture of the window alo is reading is sent to {provider} each time. Password fields are cut out first.* |

- **Before anybody has chosen, alo's own built-in offline recognition is used** —
  *the owner's decision of 2026-11-06*: it is the only one of the three
  that works on every machine, with or without a model, and sends nothing — so
  it is the one that decides nothing for the person that they could not see
  (ADR 0025).
- **A third-party reader is egress, and law 1 holds for it at every setting**:
  each picture sent is on the indicator as it happens and in the record
  afterwards, sent by the service that already sends questions to a provider,
  never by the command that asked.
- **Whichever reader is chosen must say which of the 24 languages a machine can
  be set to it cannot read**, and a window in one of them says so rather than
  being read badly in silence.
- An organisation may name which readers are permitted on machines it owns
  (ADR 0016); it never chooses one for a person's own machine.

### 5. What `app-adapters.md` becomes

**Made in the change that accepted this record, 2026-11-06** — earlier than the
proposal said (*the change that first builds the screen road*), so that no
document on `main` refuses what this record permits. ADR 0001 §6 carries a dated
amendment pointing here, and `app-adapters.md` was edited in place:

- *`synthetic`, whatever else is true* is removed from the list of refusals;
  `synthetic` is carried out by the machine itself under §1 above, and is not a
  mechanism an adapter author declares;
- *nothing is found or pressed by position* gains: *except a person's own
  `click` and `drag`, by proportions of the window, recorded as done by
  position; never an agent's*;
- the four-mechanisms table's last row is replaced by §1's.

## What this costs, written here because it is the part to argue with

- **It will sometimes press the wrong thing.** Reading pictures is a guess about
  pixels, however good the reader. The record says which acts were guesses;
  Put back reaches the files those acts changed and nothing else.
- **A third-party reader sees the person's windows.** It is the person's choice,
  said in plain words, and on the indicator each time — but a person who chooses
  it is sending pictures of whatever alo was reading.
- **A text-recognition engine is a new pinned component**, in every language
  this product ships in. Which one is not decided here; it is named in the change
  that builds it, under *engines are configured, never patched*.

## What this does not decide

The recognition engine itself; the Settings surface the three choices appear
on, which follows the design; and the per-application switch's place in the
grants list, which follows ADR 0143's question 10.
