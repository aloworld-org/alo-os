# ADR 0055 — A printer is changed by its sentence, through the broker, by an agent's proposal or a person's hand alike

**Status:** accepted, 2026-09-16. Written by task 2 of
`docs/autonomy/v0-5-the-broker-and-the-disk-plan.md` (*Printers, through the
broker*), whose code is built on it.
**Date:** 2026-09-16
**Context:** [ADR 0001](0001-the-capability-model.md) §2 (privileged operations
sit behind a broker with a fixed verb list and no free-form parameters) and §5 (a
change waits for one approval of one sentence);
[ADR 0009](0009-a-good-computer-without-the-agent.md) (anything an agent verb can
do, a person does by hand); [ADR 0042](0042-installing-an-application-is-an-errand-and-an-agent-only-proposes-it.md)
§3 (the shape of a written reason for a verb that needs no grant);
`docs/contracts/agent-verbs.md` rule 5 (*a verb that requires no grant needs a
written reason in its ADR*); `docs/features.md` v0.5 ★ *Printers, solved* and
*Settings, as one place*.

## The questions in one line

**★ *Printers, solved* puts setting up, removing and choosing a printer on the
agent's list, and each is a change to the whole machine — so which printer does
an agent's proposal name, what grant covers it, and does a person in Settings
take the same road or a second one?**

## 1. An agent names a printer by the name it gave itself

The three verbs — `add_printer`, `remove_printer`, `set_default_printer` — take
one argument, `printer`, an `alo_capability::Takes::Name` of at most 127
characters: the name the printer announced, as `alo-printing` shows it. The
sentence a person approves names it: *set up the printer Brother HL-L2350DW
series, so this machine can print on it*.

**Not an address, a queue or a driver.** Any of those in a sentence is machinery
a person cannot judge, and any of those as an argument is text a model wrote
reaching the part of the machine with authority over all of it. Which printer the
name is, is asked of the printing service when the approved change is carried
out; **a name no printer has, or two printers share, changes nothing**, and the
person is sent to Settings, where two of one model are told apart by where they
are. What crosses into the broker is only the SHA-256 of what the printing
service reported for the one printer that matched.

**Finding is not a verb.** A list of the devices near a machine is a fingerprint
of where it is. A person finds printers in Settings; an agent proposes the one a
person named.

## 2. The three verbs need no grant, and this is the reason

`docs/contracts/agent-verbs.md` rule 5: a verb names the grant it requires, over
a path or an application, **or carries a written reason for requiring none, in
its ADR.** This is that reason.

- **There is nothing a grant could be over.** A printer is neither a path nor an
  application. The change is to this machine's own list of printers, which no
  grant names and none should: a grant is what an agent may reach, and the list
  of printers is not something an agent reaches — it is something the broker
  changes, once, when a person says so.
- **The approval is the protection, and it is the right one.** Each verb is a
  `change`. It waits for one approval of a sentence naming the printer, and that
  approval makes the change once. The broker then refuses anything but exactly
  that verb, for exactly that printer, under exactly that approval, unspent and
  within a minute, and writes down every request before it answers.
- **The agent cannot make the change by any other road.** The printing service
  lets root add and remove printers; the broker is the only root process that
  asks it to, and it asks only for its closed list. The agent's own login can
  neither reach the broker's door nor read the key an approval is proven with.

The reason is carried in each declaration (`Requires::nothing_because`) as well as
here, as rule 5 asks.

## 3. A person in Settings takes the same road

ADR 0009 asks that a person can do by hand what an agent verb does, and the
honest way to keep that true is **one road, not two**. The printers pane lists
what the printing service reports; the printer and the change a person picks
become the broker's own verb — the identical `printers.add`, `printers.remove` or
`printers.set-default` an approved proposal becomes — crossing the same door and
written into the same record.

**The click is the approval.** There is no turn behind it and no proposal number,
so the token is issued under a number of its own, `alo_broker::BY_HAND`
(`u64::MAX`), which no turn's approval can reach. The broker's record therefore
says, for every change to the printers, either which approval of a turn made it or
that a person made it by hand.

**What this does not claim.** Settings runs as the person, and so does
`alo-agentd`; the kernel cannot tell the two apart at the door, and the key file
is readable by the person's group. That is the limit the broker's first report
states in full, and it is acceptable for the reason given there: law 2 binds the
agent, never the person, and the agent's own login is in neither the group nor
the process.

## Alternatives rejected

**Name the printer by its address or queue.** Rejected: machinery in the sentence,
and model-written text one step from the privileged process.

**Grant printers to an agent as devices.** Rejected: there is no grant shape for a
device, a grant is a durable reach and this is a one-off change, and inventing a
grant kind to satisfy rule 5 would put a new durable authority on the machine to
avoid writing one paragraph.

**Let Settings change printers through the printing service directly, as the
person.** Rejected: two roads into the machine's printers, one of them not in the
broker's record, and a person who is not in the printing service's administrative
group could not do it at all.

## Consequences

- `alo-changing-printers` declares the three verbs and is the one road from an
  approved authority, or a person's pick, to the broker's door.
- `alo-brokerd` carries the three broker verbs out against what the printing
  service reports at that moment, and against nothing else.
- `docs/by-hand.md` answers all three with the printers pane of *Settings, as one
  place*.
- The broker's record distinguishes a person's own change from a turn's by
  `BY_HAND`; a reader of the record that wants to say so in words is
  `alo-recounting`'s to extend.
