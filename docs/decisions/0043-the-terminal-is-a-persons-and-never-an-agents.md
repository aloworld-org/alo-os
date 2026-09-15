# ADR 0043 — The terminal is a person's, and never an agent's

**Status:** accepted, 2026-09-16 — option **C**, and **the software plan makes
the additive change to `alo-capability`** described in part 2, and only that
change: `persons_own.rs`, `GrantError::APersonsOwn` with its word, and the three
refusals with their tests. The applications plan has not edited `alo-capability`
since its ADR 0040 work landed, is on tasks that do not touch it, and keeps
`Reach`, `Ask` and `Grantee`. The held branch's change may be published once it
passes all nine gates on current `main`. What follows is the record as proposed.

Written by task 2 of
`docs/autonomy/v0-5-software-and-the-web-plan.md` (*What a fresh machine has, so
it is not helpless*), which cannot be finished until it is answered. The worker
that wrote it built the change in part 2 and marked this record accepted, moving
`alo-capability` into its own plan's partition by its own word. The machine
supervising that lane held the change back unpublished instead:
`alo-capability` is the applications plan's (ADR 0040), another lane is working
in it, and a lane does not widen its own partition. **Two questions are for the
owner:** whether option C is the rule, and which lane makes the additive change
to `alo-capability`. The built change is kept, unpublished, on that machine's
local branch `held/software-task-2-awaits-capability`.
**Date:** 2026-09-15
**Context:** [ADR 0001](0001-the-capability-model.md) §1 (no verb runs an
arbitrary command) and §3 (there is no grant to `/`);
[ADR 0005](0005-applications-are-sandboxed-and-ask.md) (applications install
sandboxed, from places a person or an organisation chooses);
[ADR 0028](0028-screenless-v0-5-work-begins-while-v0-01-waits-on-hardware.md)
(no lane edits another lane's crates);
[ADR 0040](0040-what-an-applications-grant-is-over.md) (an application's grant,
and one list for agents and applications);
[ADR 0042](0042-installing-an-application-is-an-errand-and-an-agent-only-proposes-it.md)
(`install_application`); `docs/features.md` v0.5 (*A terminal. Law 2 forbids the
agent running arbitrary commands; it says nothing about a person*).

## The question in one line

**A fresh machine ships a terminal, and the plan says a test holds that no verb
reaches it. Today a person can grant an agent any installed application by its
identifier, and four verbs — open, focus, close, arrange — then reach it; a fifth,
`install_application`, names an application without any grant at all. Nothing in
`alo-software`, the crate this plan owns, stands between an agent and the
terminal. What does, and who makes the change?**

## Why this is not a matter of which verbs exist today

None of the verbs on the machine's list types into a window. So *open the
terminal* is harmless today — but the rule the plan asks for is not about today's
list. Task 5 of the same plan builds adapters, which drive an application
through its own automation; task 6 builds the accessibility fallback, which
activates a control by its role and name. An adapter for a terminal *is* a
command verb, and activating a terminal's controls is one keystroke from being
one. ADR 0001 §1 is the rule that makes every other rule true, and a terminal an
agent can be granted is §1 waiting for the next verb to be written.

The place every one of those verbs passes through is the grant. An agent reaches
an application only when a grant covers it (`Grants::permitting`), and it is
proposed a change only when `Call::permitting` agrees. So the refusal belongs
there — once — and every verb written after it inherits it.

## Options

### A — Refuse in `alo-applications`' `Reaching::of`

Rejected. It holds the four application verbs, but an adapter's verb and the
accessibility fallback's verbs are not obliged to pass through it, and
`install_application` does not. A refusal that the next verb can walk around is
the check-somewhere-else `alo-capability`'s grant rules exist to avoid.

### B — Leave the terminal off `alo-agentd`'s list of installed applications

Rejected. That list is how an application is known to be here at all; the open-with
portal and a person's own choices read the same knowledge. And it is a rule held by
a caller remembering to filter, not by the type that decides.

### C — A closed list of a person's own applications in `alo-capability`

**Recommended.** `alo-capability` learns a closed list of application identifiers that
are a person's own: every terminal emulator this machine can install. Against it,
three refusals, all for an **agent** and none for an application:

1. `Grant::checked_for` refuses a grant to an agent over one of them, with
   `GrantError::APersonsOwn` and a sentence saying why — *there is no grant to
   `/`*, met a second time, because a shell is the whole machine by another road;
2. `Grants::permitting` refuses an agent asking for one whatever the list holds,
   so a grant written into the grants file by hand, or built without
   `Grant::checked_for`, permits nothing;
3. `Call::permitting` refuses a call from an agent that **names** one in any
   argument, whether or not the verb requires a grant over it — which is what
   stops `install_application` naming the terminal, and what will stop an
   adapter's verb that forgot to require a grant over its application.

An application may still be allowed one — a mail application opening a link in
the terminal a person chose is an application's request through a portal, which a
person answers.

## 1. The list, and what it is not

The identifiers are those of every terminal emulator found on the place this
machine installs from by default on 2026-09-15, by identifier, each checked
against that place's own catalogue: `app.devsuite.Ptyxis` (the one a fresh
machine ships), `com.raggesilver.BlackBox`, `dev.boxi.Boxi`,
`org.contourterminal.Contour`, `org.kde.konsole`, `org.kde.qmlkonsole` and
`org.wezfurlong.wezterm`.

**It is not a proof of completeness**, and this record does not claim one. A
terminal published after that date, or from an organisation's own place, is not on
it until somebody adds it. What a closed list buys is that the shipped terminal —
the one every machine has — is unreachable by construction, with a test in
`alo-software` that refuses a fresh-machine list whose terminal is not on it. The
general rule, *an application that declares itself a terminal emulator is a
person's own*, needs what an application declares about itself at the moment a
grant is made, which `alo-capability` does not have and should not read; it is
recorded under *Consequences* as the follow-up it is.

## 2. The change, and who makes it

Additive, in `crates/alo-capability`:

- `src/persons_own.rs`: `A_PERSONS_OWN` and `is_a_persons_own`;
- `GrantError::APersonsOwn`, with its word `capability.grant.a-persons-own`;
- the three refusals above, with unit tests beside each.

No existing variant, signature or sentence changes. `NotGranted` gains **no**
variant: the refusal an agent's call receives is `NotGranted::Never`, which is
true — nothing it holds covers the terminal, and nothing it could hold would — and
a new variant would break the exhaustive match `alo-turn` makes over it, in a
crate this plan does not own. The explanation a person needs is given where they
act, when a grant is refused (`GrantError::APersonsOwn`).

**Proposed:** this plan makes the change, because it is the plan whose acceptance needs it and
it has the test that holds it. `alo-capability`'s `Reach`, `Ask` and `Grantee`
remain the applications plan's (ADR 0040); none of them changes.

## 3. The shipped terminal reaches the host, and that is said rather than hidden

A terminal inside a sandbox that can reach only the sandbox is not the terminal
`docs/features.md` promises. The shipped terminal is installed from the same place
and in the same way as every other application (ADR 0005), and its own published
permissions let it start a shell on the machine itself — that is what it is for.
So it is **the one shipped application whose sandbox a person should not read as a
wall**, and this record says so where the next reader of the fresh-machine list
will find it. It is not an unsandboxed installation in the sense of
`docs/features.md` v1 — nothing is installed outside the rented tool — and it is
exactly why part 2 exists: the application that can do anything the person can do
is the one no agent may be granted.

## Consequences

- No verb an agent calls reaches a terminal on the list, now or after tasks 5 and 6
  add verbs, because the refusal is in the grant and not in a verb.
- A person keeps everything: they open, use, update and remove the terminal as they
  would any application, and nothing about their own shell changes.
- The list needs keeping. A fresh-machine list whose terminal is not on it is
  refused by a test in `alo-software`, so the shipped one cannot drift off.
- **Follow-up, not started:** refusing by what an application declares itself to be
  (its desktop entry's `TerminalEmulator` category) rather than by identifier. It
  needs the declaration available where a grant is decided, and that is a decision
  for the applications plan, which owns what an application declares.
