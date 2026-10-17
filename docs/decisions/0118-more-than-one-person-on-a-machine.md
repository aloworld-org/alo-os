# ADR 0118 — More than one person can have an account on a machine

**Status:** **accepted, 2026-10-16, by the owner**, in these words: *"Multi-person
can create accounts"* and *"we need multi-person"*.

**Reverses a deferral.** [ADR
0024](0024-what-a-person-signs-in-at.md) put more than one person at v1 and
said *nothing here forecloses it: the store already holds accounts rather than
an account.* That was true and remains true — this record is the deferral
ending, not a design being undone.

## What is already right, and must not be rebuilt

- **`alo_accounts::Accounts` holds accounts, plural.** `signs_in` finds a
  person by name; `name_of` answers by number; the file format is a list.
- **`/var/lib/alo-undo` is `0711`** ([ADR
  0045](0045-what-undoing-rewinds-to.md) term 8) *precisely* so that listing
  cannot tell one person who else uses the machine. That mode exists for a
  case the sign-in screen cannot yet reach.
- **The design already draws *Switch person*.** It is on the returning
  sign-in screen in `176-10686`, and it is drawn today on a machine that has
  nowhere to switch to.

So the ground is laid. What is missing is not a model but a **route**.

## What a person is on this machine, which is more than a row

Measured on `main`, 2026-10-16. This is the part that makes the work larger
than removing a refusal:

```
image/Containerfile:773            systemd-sysusers          ← at image build
image/usr/lib/sysusers.d/alo.conf  declares alo, uid 1000
image/etc/alo/agentd.toml          "nothing else may disagree with them"
alo-compositor.service             Environment=ALO_PERSON=1000
```

A second person therefore needs **five** things that do not exist:

1. **A Unix user made while the machine is running.** A line in `/etc/passwd`,
   a group, and a home directory. Today that happens once, when the image is
   built, and `systemd-sysusers` writes the line while
   `usr/lib/tmpfiles.d/alo.conf` makes the home — because sysusers does not.
2. **A number that is allocated rather than named.** Every component reads
   `1000` from one file that says nothing may disagree with it.
3. **A greeter that greets more than one person.** `ALO_PERSON=1000` is passed
   in its unit; `Greeting::person` is that number and `whose_name` answers for
   it alone — deliberately, because *a surface that could name any account
   would be an enumeration door*.
4. **The privilege to do any of it.** Creating a user and handing them a home
   needs capabilities **no unit in `image/` grants**. Both lanes have now
   measured this independently: `alo-sessiond` and `alo-compositor` are each
   `User=root` with an empty `CapabilityBoundingSet`, and root without
   `CAP_CHOWN` cannot give a directory away.
5. **A decision about whose agent runs.** `agentd.toml` describes *the
   machine's person*. [ADR 0017](0017-the-agents-door-is-ours-and-not-in-the-session.md)
   already says a machine *may have several people on it and one person's door
   is not another's* — so the door is anticipated; which doors exist on a
   two-person machine is not yet decided.

## The decision

**A machine may have more than one account, and a person already signed in may
make another.**

Three terms follow from the measurement, and each is a thing to build rather
than a thing to assume:

- **The first account keeps its path.** `Greeting::makes_an_account` refuses
  once any account exists, and that refusal stays: it is what stops a surface
  making a second *first* account. Adding a person is a **different** road
  with a **different** authority, not this one relaxed.
- **Adding a person is authorised by a person already on the machine**, at a
  screen they reach after signing in. Not at the sign-in screen: a door that
  could add an account before anybody authenticated would be a door that adds
  an account for anybody.
- **The number is the machine's to allocate, never the surface's.** ADR 0024's
  reasoning holds unchanged — *a surface that could choose it could make an
  account that authenticates and can never sign in.*

## What this does not decide

- **Whose agent runs, and how many.** ADR 0017's question. A second person
  without an agent is a usable machine; a second agent is a separate decision.
- **Which component holds the capability.** Nothing in the image grants
  `CAP_CHOWN` today, and the honest answer is that **one component must gain
  it**, narrowly, with a test holding it to that. Which one is the next
  record, and it should be written against a measurement rather than a
  preference.
- **Whether a person can be removed.** Making one is this; unmaking one has
  its own refusals and its own data question, and is not smuggled in here.
- **The look of any of it.** The desktop lane's, and *Switch person* is drawn
  already.

## What this costs, stated plainly

This is days rather than hours, and it reverses a deferral the owner made with
the facts in front of them. **It is not in Sunday's release**, and saying so
here is the point of writing it down: a launch that slipped because nobody
named the cost is worse than one that shipped with one account and said so.

The one-account machine that ships on Sunday is not a prototype of this. Every
term above adds a road beside what exists rather than changing it.

## How this is known to be done

On a machine: a person signs in, adds a second person, signs out, and the
second person signs in with their own name and password, reaches their own
desktop, and **cannot see the first person's files or undo history**. The last
clause is the one that is easy to leave untested and is the reason `0711` was
chosen before anybody could reach it.
