# Contract — what a person's shell sends their agent

Status: public surface. The socket and its first line are
[ADR 0109](../decisions/0109-a-persons-words-reach-the-agent-through-a-socket-systemd-opens-as-the-agent.md)
(2026-10-14); the folders line was added on 2026-10-24; the earlier
conversation was added on 2026-10-28 by
[ADR 0138](../decisions/0138-a-follow-up-is-sent-with-the-conversation-so-far.md);
the window offered was added on 2026-11-10, and where the person's folders are
the same day; whether the person chose full access was added on 2026-11-11.
This document was written on 2026-10-28: until then the lines were described only
in code, in two places that held each other to nothing.

## The socket

`/run/alo-asking/socket`, opened by the person's shell, accepted by
`alo-asking.socket` and handed to one `alo-agent` running as the agent's own
user. **One connection is one request and one turn.** Which person asked is the
kernel's answer (`SO_PEERCRED`), never a field.

The shell is `crates/alo-shell`'s `what_a_person_asked_goes_to_the_agent.rs`;
the agent is `crates/alo-agent`'s `what_the_person_sent.rs`.

## What the shell writes

Up to six lines, each ending in a newline, written at once:

| Line | When | Shape |
|---|---|---|
| 1 | only when the person has granted folders | `alo-may-reach:` and the folders, separated by a tab |
| 2 | always | **the request**, in the person's own words, on one line — a line break the person typed is written as a space |
| 3 | always, from a shell that knows this line | `alo-earlier:` and the conversation so far, or nothing after the marker for a first request |
| 4 | always, from a shell that knows this line | `alo-this-window:` and the window the person offered, or nothing after the marker for a request about no window |
| 5 | always, from a shell that knows this line | `alo-their-folders:` and where the person's home and standard folders are, or nothing after the marker where the shell knows no home |
| 6 | always, from a shell that knows this line | `alo-full-access:` and `yes` when the person chose full access, or nothing after the marker at every other level |

The folders are **a hint for the model, never a capability**: every verb is
still decided by `alo-agentd` against the grants it reads itself. So is the
window.

### Where the person's folders are

*Added 2026-11-10, after a boot walk where the model named
`/Users/username/Documents` on a machine whose home is `/var/home/alo`.*

One JSON object on one line after the marker, holding exactly two fields:

```text
alo-their-folders:{"home":"/var/home/anna","folders":{"Documents":"/var/home/anna/Dokumente"}}
```

| Field | Means |
|---|---|
| `home` | The person's home folder, as their session names it: an absolute path |
| `folders` | Each standard folder by what it is for (`Documents`, `Downloads`, `Pictures`) and where it is, under the name it really has — read from `~/.config/user-dirs.dirs`, the record `alo-their-folders` writes when the account is made. Only folders the record names and that exist are listed; at most 16 (`alo_instructing::THE_MOST_FOLDERS`) |

**When.** Read on the person's side **at the moment they send the request**, for that
request only. **It is the machine's layout and never what is in a folder.** Like the
folders granted, it **allows nothing**: the model is told where things are, that it
must not name another computer's layout, and that what it may reach is decided as for
any other path (`alo_instructing::ThePersonsFolders::told`). It reaches a provider
only inside the question.

**Refused, in words, and nothing driven:** a fifth line that does not begin with
`alo-their-folders:`, one that is not the JSON above or has any other field, a home
or a folder that is not an absolute path, and a line over
`alo_instructing::THE_LONGEST_FOLDERS_LINE` bytes. A shell that sends four lines is
answered with no home said, after the fifth is waited for briefly.

### Whether the person chose full access

*Added 2026-11-11, after a boot walk where a person at full access asked what
folders were in the window they offered and was told nothing had been granted.*
Full access has no folder grants
([ADR 0149](../decisions/0149-full-access-is-everything-the-person-can-do.md)), so
line 1 is never sent there, and a model shown no folders was told that every read
would be refused.

```text
alo-full-access:yes
```

**When.** Read on the person's side from `/var/lib/alo/how-code-runs.toml`
(`docs/contracts/how-code-runs-file.md`) **at the moment they send the request**,
by the rule `alo-agentd` keeps: only an explicit full access is `yes`; not chosen
and a file that will not read are the marker alone. The agent cannot read that
file itself (`0700 alo alo`, ADR 0109), which is why the shell says it.

**It allows nothing.** `alo-agentd` reads the level again at each read, proposal
and approval. At `yes` the model is told it may reach what the person's account
can open, and **no folder is named** (`alo_instructing::WhatItMayReach::told`).

**Refused, in words, and nothing driven:** a sixth line that does not begin with
`alo-full-access:`, or that has anything after the marker but `yes`. A shell
that sends five lines is answered as below full access, after the sixth is waited
for briefly.

### The window offered

*Added 2026-11-10, for* Help with this window *in a window's panel
(`docs/design/the-window-controls.md`, `11:146`).*

One JSON object on one line after the marker, holding exactly two text fields:

```text
alo-this-window:{"title":"The next chapter","application":"org.gnome.TextEditor"}
```

| Field | Means |
|---|---|
| `title` | What the window's application called it, on one line, at most 1,000 characters (`alo_instructing::THE_LONGEST_TITLE`); empty where it named none |
| `application` | The application's identifier as the window gave it — what a verb's `application` argument takes; empty where it gave none or one longer than 255 characters |

**When.** Only for a request the person sent from the composer that *Help with
this window* opened, and read **at the moment they sent it** — never on a
timer, a focus change or a keystroke. Every other request sends the marker
alone. **Offering a window allows nothing**: the agent tells the model which
window the request means, that the title is the application's own words and
not a request, and that what may be done in it is decided as for any other
window (`alo_instructing::TheWindowOffered::told`). It reaches a provider only
inside the question, as the conversation does.

**Refused, in words, and nothing driven:** a fourth line that does not begin
with `alo-this-window:`, one that is not the JSON above or has any other field,
a window named by neither field, and a line over
`alo_instructing::THE_LONGEST_WINDOW_LINE` bytes.

### The conversation so far

One JSON object on one line after the marker, holding the turns and nothing
else:

```text
alo-earlier:{"turns":[{"person":"list my invoices"},{"alo":"march.pdf and april.pdf"},{"proposed":"Move march.pdf to Paid","answer":"allowed"}]}
```

| Field | Means |
|---|---|
| `turns` | **Everything** said in that task window before the follow-up, oldest first. Each is exactly one of `{"person": …}`, `{"alo": …}`, or `{"proposed": …, "answer": …}` |
| `answer` | `allowed`, `denied`, or `not-answered` — the last for a change that lapsed or was answered somewhere else |

It holds what the person saw in the window and nothing else: never what a
verb read during an earlier turn, unless the person's words or alo's answer
contained it. **Nothing is left out of it**: alo puts no limit of its own on a
conversation (ADR 0138); the model's is the only one, and a conversation too
long for the model is refused and said to be, never shortened.

**The line's bound, and its reason.** A line is at most **1 MiB**
(`alo_instructing::THE_LONGEST_LINE`), and the agent stops reading at that
bound rather than after it. The number is not chosen to fit a model: it is the
daemon's own bound on a request (`alo_protocol::LONGEST`, *Bounds* in
`docs/contracts/daemon-protocol.md`), and the conversation can only ever reach
a model inside one such request, so a longer line could never be asked. 1 MiB
is roughly a quarter of a million tokens, beyond the context of every model
alo OS can be pointed at today — so in practice the model's limit is met
first, and is the one a person hears about. The shell checks before sending
(`TheConversationSoFar::fits_in_a_question`, counted as JSON with room for the
instructions) and says the conversation is too long to send rather than send
something the daemon would refuse.

**When.** Only when the person sends a reply in a task window, built at that
moment. A reply sent while alo is still answering, or while a change waits for
Allow once or Deny, is held in the window and sent by itself once alo has
answered and nothing waits — built then, from the conversation as it then
stands. A shell never sends it on a timer, a focus change or a keystroke.

## What the agent does with it

- Lines 1 and 2 as before.
- Line 3, on a socket, is waited for **for half a second at most**, so a shell
  from before this line still works. On a pipe or a terminal it is not read.
- Line 4 is waited for the same way, and **only after a line 3 arrived**, so a
  shell from before both waits once. A shell from before line 4 is a request
  about no window. The window is told to the model after the conversation and
  before the request.
- Lines 5 and 6 are waited for the same way, each only after the one before it
  arrived. A shell from before line 6 is answered as below full access.
- A conversation is put to the model **inside the question the turn asks the
  daemon**, between what the turn may reach and the request, marked as an
  earlier exchange that is not a request now. It reaches a provider only as that
  question does — through `alo-agentd`, under the egress indicator, into the
  record (`docs/contracts/daemon-protocol.md`).
- **Refused, in words, and nothing driven:** a third line that does not begin
  with `alo-earlier:`, one that is not the JSON above, a turn or answer of a
  kind not listed here, and a line over the bound. A refusal is written
  to the agent's error output; the shell sees the connection close with nothing
  answered and tells the person the request stopped without finishing.

## What the agent writes back

*Written down 2026-11-06, when something other than a person began reading it;
the words themselves are unchanged.*

Words a person reads, ending when the agent shuts its sending side. **When the
request left a change waiting, the reply is exactly two lines**:

```text
a change is waiting for you to answer, as number 3:
  Rename march.pdf to march-final.pdf
```

`alo_protocol::what_the_agent_says_back` is the one place that spells it, and
both ends use it.

- **It counts only as the first line.** An answer in words or a refusal carries
  words that are not the agent's own — a model can be steered into writing
  anything — so the agent puts a space before any such reply that begins with
  this line, and a reader that trims the start of a reply before looking at its
  first line has undone that.
- **It is a narrowing, never a permission.** A reader that answers the change
  must also find the same number *and* the same sentence among what the daemon
  says is waiting (`waiting` on the person's door), and answers nothing
  otherwise.

### `alo` holds the turn when it is typed at a terminal

`alo` (and `alo-ask`, the same command) closes its sending side as soon as it
has written, as above, **only when its input is not a terminal** — a script or a
pipe. A change such a request leaves is not asked about: there is nobody to ask,
the reply is followed by a sentence saying alo can only ask for the OK in a
terminal, and the change lapses with the turn.

**Typed at a terminal**, it writes the request and an `alo-earlier:` line and
an `alo-this-window:` line with nothing after either, and **keeps its sending
side open**, exactly as a shell does
while a card is on the screen: the agent holds the turn, `alo` reads the reply
and answers the change on the person's door before it closes.

- **With *Let alo act without asking* switched on** in Settings → Terminal
  (`docs/contracts/how-code-runs-file.md`, `in-the-terminal`), it approves the
  change without asking.
- **Otherwise it asks *Run this command? [y/N]*** (ADR 0124), reading one key:
  `y` sends one `approve` of that change's number; any other key — `n`, Enter,
  Ctrl-C, Ctrl-D — sends one `decline`, which the daemon records. No key while
  the question stands (the service's `lapses_in`), or a terminal that went
  away, sends nothing.
- Closing before the reply is read — a Ctrl-C while alo is still working — is a
  Stop, and the change lapses unanswered.

## Versioning

**Additive only.** A new line goes after the last one a current agent reads,
never before the request, because an older agent reads the request from the
position it knows and never reads past it. That is what happens to line 3
today: an agent written before 2026-10-28 reads lines 1 and 2, answers the
follow-up as a first request, and never sees the conversation. Line 4 is the
same: an agent written before 2026-11-10 reads three lines, answers a request
about a window as a request about none, and drops the unread line with
anything else said after the request.

## Stop

*Added 2026-10-29, [ADR 0139](../decisions/0139-stop-reaches-alo-and-cancels-what-it-sent.md).*

**A shell that shuts its sending side while the turn is running asks the agent
to stop.** The shell writes nothing more after the request in any case, so the
half-close is the whole message. The agent then asks the daemon for nothing
more, shuts its own sending side to the daemon (which cancels a question on its
way to a provider; `docs/contracts/daemon-protocol.md`, *Stop*), and exits.
**The agent's side closing is the shell's only evidence that alo stopped**, and
the only moment a window may say so.

- **A caller that shuts its sending side as soon as it has written** — `alo`
  does when it is not typed at a terminal — has said *that was all*, and is never read as stopping: the agent sees
  that half-close already there where the third line could have been.
- **After a proposal**, the shell holding the connection open for the person's
  answer stops the turn the same way; the change on the card lapses with the
  turn and is never answered by a Stop.
- **An agent written before this** reads the half-close as the person having
  answered, and ends the turn too.
