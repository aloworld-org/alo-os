# A grant made now reaches the daemon now

**Date:** 2026-09-10
**Workstream:** v0.01 lane B — accounts and session entry, task 4
**Contributor:** Claude (checkout `C:\dev\alo-os-b`, lane B)
**Status:** ready for integration

## What was wrong

Task 3 gave the machine somewhere to keep its grants and `alo-agentd` reads them
once, before the socket exists. What it could not do was hear about a grant made
**while it is running**: a person picks a folder at eleven, the file on the disk
says so, and the service holding the turn goes on serving under the list it read
at sign-in. The gap was narrow and honest — the daemon is bound to the person's
session, so the grant applied at the next sign-in — but `docs/features.md`
promises *pick a folder*, and *pick a folder and sign out again* is not that
promise.

## What changed

### The wire: a knock, never a payload

`docs/contracts/daemon-protocol.md` gains one request on the **person's** door
and one answer beside it, both additive, `format` unchanged.

```
{"format":1,"asks":{"granted":{}}}          the person's side, saying what is granted has changed
{"format":1,"tells":{"granted":{"holding":2}}}   read again, and this is how many there are
```

`crates/alo-protocol`: `Asked::Granted {}`, `FromAPerson::Granted`,
`Told::Granted { holding }`, `ToAPerson::Granted { holding }`. The request has
**no field for a grant, a folder, a reach, a duration or a handle**, and neither
`FromAnAgent` nor `ToAnAgent` has any shape for either of them — so an agent
sending it is `NotUnderstood::NotForAnAgent` before anything reaches a turn, and
a `granted` answer can never be written onto an agent's connection.

That absence is the whole design. The only thing this message can cause is the
daemon reading **its own file** again, under `alo-remembering`'s three rules
about who may have written it. Nothing on the wire carries a grant, so a request
that arrived from anywhere at all could not widen anything — and the kernel
already says which door a caller is on.

### `crates/alo-agentd/src/rereading.rs` — the decision, in one file (new)

- `Remembering` — one method, and it **reads**. There is nothing in this crate
  that writes the grants file, which is what keeps *nothing an agent can send
  over the socket writes a byte of it* true after this change: what the socket
  can now reach is a road to reading it.
- `ThePersonsFile` — the only implementation that ships; holds a path and
  nothing else, and the path is `src/main.rs`'s.
- `WhatIsGranted` — the list the service is serving under and the file it came
  from, as one value. It travels everywhere below `src/main.rs`, and
  `read_again` is a method on it.
- `an_agent_knocked` — the refusal on the agent's door, worded and **written
  down** before the file is opened.

The list is **replaced whole**, because that is what the file means:
`alo-remembering` writes what is granted, so a grant that is not in it is one
that was revoked. The one thing carried across is the grant a turn's own
invocation made (ADR 0001 §4) — it is not in the file, it lasts as long as the
turn, and `alo_turn::Turning::ending` gives it back by handle — so it is put back
**under the handle it already has**, and a file that has since handed that handle
to something else is refused rather than merged.

### `crates/alo-agentd/src/holding.rs` — the machine, or the turn holding it (new)

The person's door used to be handed an `Option<&mut Turning>`, because nothing
on it touched anything outside the turn. The knock does: the refusal has to be
written down whether or not an agent happens to be connected, and between turns
the turn is not there to write it through. `Holding` is the machine **or** the
turn, never neither, and it carries the turn's `Questions` with it because those
begin and end together.

### `crates/alo-record` — one new kind of entry

`Happened::GrantsNotReadAgain { why }`, and `Entry::the_grants_were_not_read_again`.
Additive; `format` stays `1`, which `docs/contracts/record-file.md` already
argues for. It has **no agent field**: making, revoking and re-reading a grant
are the person's acts, and a name there would be an authority the record
invented. That document's *every entry names whose authority it was under,
except one* now says *except two*, and says how the two are told apart — only
one of them reached the network.

`alo-recounting` gains the clause a person reads for it, in the vocabulary, with
a note for translators.

### `crates/alo-turn` — two doors onto that entry

`Machine::the_grants_were_not_read_again` and the same on `Turning`. Both take
only the sentence and the moment; neither takes an agent, so no caller can write
somebody's name against something no agent did. The turn's door closes the turn
when the record breaks, exactly as an unwritable verb does.

## Decisions I made, and why

**The name is `granted`, and it is on the person's door.** A grant is made by a
person picking a folder (ADR 0001 §3) and revoked on the same side. An agent that
could ask for the list to be re-read would be an agent choosing the moment its
own reach is recalculated.

**The answer carries a count and no list.** *See what is granted* is a surface,
and the surface is the compositor lane's. A list crossing here would be a second
copy of the person's own file for nobody to check against. The count is what a
shell needs to know the knock arrived and was acted on.

**A file that is not there reads as an empty list; a file that cannot be
believed leaves the grants alone.** The first is the ordinary first morning, or a
person who revoked the last thing they granted — the same answer start-up gives.
The second is the acceptance criterion, and it is the difference between a
machine that says nothing changed and a machine that went silent about what its
agent may reach.

**One sentence to the person, several in the service log.**
`NotReadAgain` is English and names the file, because whoever reads it is
whoever goes and looks at it. What crosses the socket is one translated sentence,
because there is one thing for the person to do about every one of them, which is
nothing: *nothing was widened and nothing was forgotten*.

**The knock is the one wrong-door message from an agent that leaves an entry.**
A malformed message, or one meant for the other door by a confused client, is
noise: refused in words, nothing happened. An agent reaching for the person's own
list is not noise. Recording every protocol refusal would be a larger change with
a different argument, and it is not this task's.

**A new `Happened` variant rather than reusing `turned-away`.** `turned-away`
carries an agent, and the person's-door refusal has none. Bending one into the
other would have put a call nobody made, under an authority nobody held, into a
record that people read.

**No entry is written when the grants *are* read again.** Making a grant is not
recorded either — `alo-picking` writes no entry — so recording the re-reading
while the granting goes unrecorded would be a record that answers half a
question. What the record is for (ADR 0001 §7) is what the agent did, and the
refusal is here because it changes what the agent may reach and nothing else
would say so.

## Verification

Windows host, gates run in WSL Ubuntu with `CARGO_TARGET_DIR=$HOME/target-claude`
— the environment `tools/kernel-loop` runs them in.

| Command | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | zero warnings |
| `cargo test --workspace` | exit 0, 165 test-result lines, no failures |
| `cargo doc --workspace --no-deps` (`RUSTDOCFLAGS=-D warnings`) | exit 0, no warnings |

The ten `#[ignore]`d tests are the pre-existing session fixtures
(`a_session_that_really_ended.rs` and its kin); nothing here added one.

### Acceptance, one test each

| The plan says | Test |
|---|---|
| a grant made while the daemon is running is honoured in the same session without a restart, and a revocation takes effect on the next question asked | `alo-agentd`, `lib`, `serving::tests::a_grant_made_while_the_service_runs_reaches_it_and_a_revocation_does_too` — a real socket, an agent's turn open the whole time, the person's side writing the file and knocking exactly as a folder picker would, the read that was refused a moment earlier carried out, and then the same in reverse |
| the request carries no grant, no path and no duration | `alo-protocol`, `lib`, `asked::tests::saying_what_is_granted_changed_cannot_carry_a_grant` — five ways of trying to put one in, all refused by the shape rather than by a check |
| the same request on the agent's door is refused in words and the refusal is written down | `alo-agentd`, `lib`, `serving::tests::the_same_request_on_the_agents_door_is_refused` — over a real socket, with a file that really does grant the folder, so a service that read it at the agent's asking would have answered the read that follows. `doing::tests::only_the_knock_on_the_agents_door_is_written_down` is the entry itself, and that the other wrong-door messages leave none |
| a grants file that has become unbelievable leaves the grants it already had, with the refusal in the record | `alo-agentd`, `lib`, `answering::tests::an_unbelievable_grants_file_leaves_the_grants_alone_and_is_written_down` — the folder is still reachable afterwards, the sentence is not a key, and the entry is `GrantsNotReadAgain` with no agent named |

Refusal and edge paths beyond those: a knock that arrives with a number, a folder
or an agent in it (`person::tests`); an agent's `approve`, `decline`, `waiting`
and `granted` all refused at the door (`agent::tests`); a machine with no grants
file reading as nothing granted; a turn's own grant carried across a replacement
and still revokable by its handle
(`rereading::tests::the_grant_this_turns_invocation_made_is_carried_across`); and
a list that would collide with that handle refused with the grants left as they
were (`rereading::tests::a_list_that_reuses_the_turns_handle_is_refused`).

## Limitations

- **The handle collision is unreachable on this machine today.**
  `crate::serving` begins every turn with `alo_context::Context::at_invocation`,
  which offers no document, so no turn holds a grant of its own and nothing can
  collide with one. It is written and tested because the compositor lane is what
  makes it reachable, and a grant outliving the turn that made it is not a thing
  to discover then.
- **The surface that lists and revokes is still the compositor lane's.** The
  answer here is a count; there is nothing for a shell to draw a list from, and
  that is deliberate.
- **Nothing here has run on a booted alo OS machine.** The socket, the doors and
  the file are exercised for real on the Linux host; delivery-plan task 10 is
  where the image and these meet.

## Proposed shared-document updates

I did not edit `CHANGELOG.md`, `ROADMAP.md`, `docs/autonomy/QUEUE.md` or
`docs/autonomy/STATE.md`. For the integration owner:

**CHANGELOG.md**, under the unreleased section:

> **A folder you grant is reachable straight away.** Until now, picking a folder
> for your agent took effect the next time you signed in — the file on disk said
> so, and the agent service was still working from the list it read when you
> signed in. It now hears that what you granted has changed and reads your list
> again, so the folder you just picked is one your agent can use in the same
> breath, and a folder you just revoked is refused at the very next question it
> asks. Nothing about a grant travels over that connection: your side of the
> machine only says *this changed*, and the service goes and reads your own file.
> If it ever cannot read it, it keeps exactly what you had granted, tells you so,
> and writes it down — it never quietly decides you granted nothing.

**ROADMAP.md / QUEUE.md:** lane B task 4 is done; task 5 (*a person can be told
what their machine did*) is written and ready. No task in
`v0-01-delivery-plan.md` matched this one, so nothing was marked there. The
`Grants` line of `docs/features.md` now has *pick*, *it expires*, *it is kept*
and *it takes effect at once* in code; *see what is granted* and *revoke it*
still wait on the compositor lane's surface.
