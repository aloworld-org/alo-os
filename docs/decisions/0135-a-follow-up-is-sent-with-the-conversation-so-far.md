# ADR 0135 — A follow-up is sent with the conversation so far

**Status:** **accepted, 2026-10-28, by the owner**, who decided that a
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

### Its size, and what happens over it

**8000 bytes of what was said** (`TheConversationSoFar::THE_MOST_SENT`) —
room for a request, a listing and a few answers, and small enough that a
model with a short context still fits the instructions and the verbs ahead of
it. Over the bound, the **oldest** turns are left out **whole**; a turn is
never cut in the middle while a whole one would fit, because a cut sentence
can say something different from the whole one. A single newest turn longer
than the bound on its own is the one exception: it is cut, and the cut is
written where it happened.

Either way it is said, twice: the model is told how many earlier parts were
left out, and the person's task window says in its activity that *the oldest
part of this conversation was not sent to alo with your reply*. Nothing is
shortened silently.

### When it is sent

**Only at the moment the person presses Enter on a follow-up.** The shell reads
the task's conversation then and at no other moment, hands it to the agent
with the new words, and keeps nothing. That is *context is offered, never
watched*: it is the person's own act that offers it, for that turn only.

A reply is **not** sent, and the window says why beneath it, while a change
above is still waiting for an answer (the agent that proposed it holds the
only turn the daemon serves) or while alo has not answered the previous
message yet.

### Where it travels, and that it is seen

On the asking socket as one line **after** the request (`alo-earlier:` and a
JSON object; the contract has the shape). The agent puts it in the question it
asks the daemon, between what the turn may reach and the request, marked as an
earlier exchange that is not a request now. So it leaves the machine **only
where a question does**: through `alo-agentd`, which tells the desktop before
the connection opens (ADR 0134) and writes the record. There is no second
road. `alo-agentd`'s `nothing_leaves_before_the_desktop_is_told` test holds a
follow-up's question to the same guarantee as a first request's.

## The grades, and why none is set aside

ADR 0133 grades every model against the SHA-256 of the instructions text a
turn shows. **That text does not change.** The conversation is placed in the
request half of what a model reads — where the language clause and a turn's
steps already sit — and the digest is of the instructions alone, so every
grade still names the text a turn shows.

What that costs is stated rather than hidden, as it was for the language
clause: **a grade earned on a first request is not proof about a follow-up.**
The real-model check below measures follow-ups on one provider model; no
local model has been measured on one.

## Compatibility

The line comes **after** the request because an `alo-agent` written before
this reads the folders line (if any) and the request, and then never reads its
input again; a later line is ignored, and it answers the follow-up as a first
request. Anywhere before the request it would have been read *as* the request.
A current agent waits half a second at most for the line, so a shell from
before — which sends none — still works. A line after the request that is not
a conversation a shell would write, or one over the bound, is refused in words
and nothing is driven.

## The costs the owner accepted

1. **A protocol addition**: a third line on the asking socket, additive and
   documented in `docs/contracts/asking-socket.md`.
2. **The earlier conversation is sent again with each follow-up.** With a
   provider, that means the conversation leaves the machine once per follow-up,
   not once. The egress indicator shows each departure and the record keeps
   each one.

## What this does not decide

Whether a reply typed **while** alo is still working should be queued or
redirect the work at the next safe step (the design's interaction contract
asks for one of the two). Today it is not sent, and the window says so.
