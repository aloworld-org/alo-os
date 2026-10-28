# ADR 0136 — A follow-up is sent with the conversation so far

**Status:** **accepted, 2026-10-28, by the owner**, and amended by the owner
the same day (whole conversations, held replies, follow-ups graded apart). The
owner decided that a
follow-up typed in a task window is sent to alo together with the
conversation so far, and accepted both costs named below.

**Context:** law 1; *context is offered, never watched*; *reads answer,
changes wait*; [ADR 0109](0109-a-persons-words-reach-the-agent-through-a-socket-systemd-opens-as-the-agent.md)
(how a person's words reach the agent);
[ADR 0133](0133-a-question-is-answered-inside-the-envelope.md) (the words a
turn shows, graded by digest); [ADR 0134](0134-the-desktop-is-told-what-alo-agentd-sends.md)
(the desktop is told what leaves); `docs/contracts/asking-socket.md`.

## What was found

`docs/features.md` promises *follow-ups stay in that window*. On 2026-10-28 a
person replied in a finished task window and the window read *alo is working*
for ever with nothing asked. #982 made it honest — the reply stayed and the
window said alo had not received it — and left the question this decides:
`alo-agent` takes **one request per turn and remembers nothing between
turns**, so a reply sent on its own would hand a model *"and the April one
too"* with nothing it could refer to.

## The decision

**A follow-up is sent with the conversation so far.**

### What the conversation is

Each thing the person saw in that window, oldest first:

- what the person asked or replied, in their own words;
- what alo answered, in words;
- the sentence of each change alo proposed, and whether the person
  **allowed** it, **denied** it, or it was **not answered** in the window
  (it lapsed, or was answered somewhere else; the window cannot tell which, so
  neither is claimed).

It **never** includes what a verb read during an earlier turn — a file's
contents, a listing — unless the person's own words or alo's answer contained
it. That is true by construction: the shell builds the conversation from the
turns it draws, and it never holds a turn's steps or its activity as
conversation. A test holds the activity out.

### Its size: whole, and the model's limit is the only one

*Amended by the owner the same day.* The first version of this decision cut a
conversation at 8000 bytes, oldest parts first, and said so. The owner set
that aside: **alo shortens nothing of its own accord.** The only limit is what
the chosen model can read.

- **A conversation is sent whole**, every turn of it.
- **When it is too long for the model**, the person is told in plain words in
  the window — *this conversation is now longer than the model can read; start
  a new one from the Dock to continue* — and nothing is sent in part. That
  sentence is worded where the model's refusal is heard (`alo-asking`), and
  never by dropping the beginning and carrying on.
- **One bound remains, and it is not alo's opinion of a conversation**: the
  daemon refuses any question over 1 MiB (`docs/contracts/daemon-protocol.md`,
  *Bounds*), so the asking socket refuses a line longer than that, and the
  shell checks before sending and says *this conversation is now too long to
  send to alo in one go* rather than letting the daemon refuse it in words
  about envelopes. 1 MiB is roughly a quarter of a million tokens, beyond the
  context of any model alo OS can be pointed at today; the contract records
  the reason.

### When it is sent, and a reply alo cannot take yet

**Only when a reply is sent.** The shell reads the task's conversation then
and at no other moment, hands it to the agent with the new words, and keeps
nothing. That is *context is offered, never watched*: it is the person's own
act that offers it, for that turn only.

**A reply alo cannot take yet is held, not refused** (*amended by the owner
the same day*). While alo is still answering, or while a change above waits
for Allow once or Deny, the reply shows under the conversation marked
*waiting to be sent*. It goes by itself, with the conversation as it then
stands, once alo's answer has arrived and nothing waits for the person — so a
change keeps it held until they answer. Before then they can take it back:
Backspace in the empty reply box puts it back in the box to change or delete.
It cannot go sooner because the daemon serves one turn at a time and the turn
before it is still open. A second reply sent while one is held joins it.

### Where it travels, and that it is seen

On the asking socket as one line **after** the request (`alo-earlier:` and a
JSON object; the contract has the shape). The agent puts it in the question it
asks the daemon, between what the turn may reach and the request, marked as an
earlier exchange that is not a request now. So it leaves the machine **only
where a question does**: through `alo-agentd`, which tells the desktop before
the connection opens (ADR 0134) and writes the record. There is no second
road. `alo-agentd`'s `nothing_leaves_before_the_desktop_is_told` test holds a
follow-up's question to the same guarantee as a first request's.

## The grades: none is set aside, and a follow-up is measured on its own

ADR 0133 grades every model against the SHA-256 of the instructions text a
turn shows. **That text does not change.** The conversation is placed in the
request half of what a model reads — where the language clause and a turn's
steps already sit — and the digest is of the instructions alone, so every
grade still names the text a turn shows.

A grade earned on a first request is not proof about a follow-up, so **the
owner chose to measure follow-ups separately for each model** (*amended the
same day*): a follow-up result is recorded per model and per instructions
digest, apart from the first-request grade, and a model nobody has run a
follow-up on is recorded as *not yet measured* rather than given one. The
measurement and where it is kept are `alo-models`' and `alo-driving`'s; this
decision only says that it exists and is kept apart.

## Compatibility

The line comes **after** the request because an `alo-agent` written before
this reads the folders line (if any) and the request, and then never reads its
input again; a later line is ignored, and it answers the follow-up as a first
request. Anywhere before the request it would have been read *as* the request.
A current agent waits half a second at most for the line, so a shell from
before — which sends none — still works. A line after the request that is not
a conversation a shell would write, or one longer than a question can be, is
refused in words and nothing is driven.

## The costs the owner accepted

1. **A protocol addition**: a third line on the asking socket, additive and
   documented in `docs/contracts/asking-socket.md`.
2. **The earlier conversation is sent again with each follow-up.** With a
   provider, that means the conversation leaves the machine once per follow-up,
   not once — and with no limit of alo's own, a long conversation leaves whole
   each time. The egress indicator shows each departure and the record keeps
   each one.
