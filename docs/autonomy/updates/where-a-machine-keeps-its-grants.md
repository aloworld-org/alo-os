# Where a machine keeps its grants between one sign-in and the next

**Date:** 2026-09-10
**Workstream:** v0.01 lane B — accounts and session entry, task 3
**Contributor:** Claude (checkout `C:\dev\alo-os-b`, lane B)
**Status:** ready for integration

## What was wrong

`docs/features.md` promises for v0.01: *Grants: pick a folder, see what is
granted, revoke it, and it expires*. Since `alo-picking` a person could make
one, and `alo-agentd` honours the ones it is holding — but **nothing kept
them**. `crates/alo-agentd/src/starting.rs` said so in as many words: the
service began with an empty list, because where a machine's grants live was a
question nobody had answered and a list read from a file would have been a list
nothing writes. A folder granted this morning was granted to nobody by the
evening, and *see what is granted* had nothing to show.

## What changed

### `crates/alo-remembering` — the file, and everything it will not do (new)

One small crate. `/var/lib/alo/grants.toml`, written by the person's side of the
machine and read by their daemon.

- `written.rs` — the file's shape, and **every grant made again on the way in**.
  Nothing here deserialises a grant: each table is handed to
  `alo_capability::Grant::checked`, the same road a person's pick takes, so a
  file hand-edited into `folder = "/"`, or into a grant with no end, or into one
  granted to nobody, is refused by the crate that owns those rules. Reading also
  **drops what has expired before the list exists**, rather than answering with
  it and leaving a filter for whoever remembers.
- `keeping.rs` (Unix) — the file on the disk. `O_NOFOLLOW`; owned by root or by
  the login reading it; refused if its group or the world can write it; checked
  on the open file rather than on the path, so what was checked and what is read
  cannot be two files. Written through a `0600` sibling and renamed over, so a
  machine that loses power keeps the grants it had. The folder is **not** made
  here — `/var/lib/alo` is the image's, for `alo-accounts`' reason.
- `refusing.rs` — `NotRemembered`, in English, because its reader is whoever is
  standing a machine up and looking at a file somebody edited. A refusal always
  means *no grants were read*: there is no partial list.

### `crates/alo-capability` — two constructors and one refusal

`Grants` already documented that its handle counter existed "so that ids stay
unique across a restart, since the list is written down and read back". This is
the road that makes that true, and it is deliberately not `serde`:

- `GrantId::numbered` — a handle read back off a disk.
- `Grants::remembered(held, next)` — the list, with the handles it was kept
  under, refusing a handle that appears twice or one that is on the list **and**
  named as the next to hand out (`NotOneList`). Both are lists in which revoking
  the grant a person can see could take away a different one.
- `Grants::next_handle` — what the writer needs so that an expired grant's
  number is not handed to a new grant somebody's stale list still shows.

### `crates/alo-agentd` — read once, before there is a door

`src/main.rs` reads the file after the record and before the boundary and the
socket, and hands `starting::until_stopped` an `alo_capability::Grants` — a
value with no path in it. That is the whole of *nothing an agent can send over
the socket writes a byte of it*: below that line there is nothing to write it
with.

A machine with **no** grants file starts and refuses everything, as before —
first morning is not an error. A file that **is** there and is not believable
stops the service (`NotStarted::NoGrants`), for the reason a description that
will not parse does: whoever can write it says what this machine's agent may
reach, and a daemon that shrugged would make *somebody tampered with your
grants* look exactly like *you have not granted anything yet*.

### `image/usr/lib/tmpfiles.d/alo.conf` — a comment, no directive

`/var/lib/alo` is already `0700 alo alo`; the grants file lives in it and needs
nothing new. The comment now says so, and says why the mode matters twice.

## Decisions I made, and why

**A crate rather than a module in `alo-capability` or `alo-agentd`.** In
`alo-capability` it would give the crate that *decides* a reason to touch a
disk, which its own header forbids. In `alo-agentd` it would be unreachable from
the person's side, and the writer is the person's side.

**`/var/lib/alo/grants.toml`, and not in the session.** The grants have to
outlive the session by definition, which rules out `/run/user/<uid>` and
`/run/alo/<uid>` — both are gone at sign-out. `/var/lib/alo` is the directory
the image already makes `0700` for the person, beside the record: what an agent
did on somebody's machine and what they let it reach, in one place that is
theirs.

**A constant, not a key in the machine description.** `[record].path` is
configurable because ADR 0004 gives retention to whoever manages the machine.
Where the grants live is nobody's policy, and a key for it would be a second
answer somebody can point at a file the daemon is not reading.

**TOML with an explicit shape, not `serde` over `Grants`.** Two reasons. The
file is one a person may open, and `{"secs_since_epoch":…}` in the file that
says what their agent may reach is not readable; and a `Deserialize` road into
`Grants` would be a list nothing validated, where re-making each grant through
`Grant::checked` re-applies every rule the crate has. Moments are whole seconds,
truncated **downwards**, so a grant read back ends at or before the moment it
was made to end — the same side `alo-capability` puts the boundary on.

**What is written is what is granted.** Expired grants are not kept in the file;
the `next` key is what stops their handles being reused.

**The daemon refuses to start on an unbelievable file, rather than serving
empty.** Argued above. It is the same choice `alo-agentd` already makes about
its description, its record and its boundary.

## What this does not do, said plainly

A grant made **while the daemon is running** reaches the daemon when it next
starts, which on alo OS is the next sign-in (the service is bound to the
person's session). Carrying one to a running daemon needs a request on the
person's door, which is `alo-protocol`'s surface — that is **task 4** in
`docs/autonomy/v0-01-lane-b-plan.md`, written in this change, and this task is
what makes it worth building: until now there was nothing to carry.

The surface that lists and revokes grants is still the compositor lane's, as the
plan says.

## Verification

Windows host, gates run in WSL Ubuntu with `CARGO_TARGET_DIR=$HOME/target-claude`
— the environment `tools/kernel-loop` runs them in.

| Command | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | zero warnings |
| `cargo test --workspace` | exit 0, 161 test-result lines, no failures |
| `cargo doc --workspace --no-deps` (`RUSTDOCFLAGS=-D warnings`) | exit 0, no warnings |

### Acceptance, one test each

| The plan says | Test |
|---|---|
| a grant a person made survives a restart of the daemon and is honoured afterwards | `alo-agentd`, `lib`, `starting::tests::a_grant_made_before_a_restart_is_honoured_after_one` — a grant kept on a disk, a service handed only what was read back, and the verb the machine before it refused is carried out and written down. The person's-pick half of the same claim is `alo-remembering`'s `a_grant_a_person_made_survives_a_restart_and_is_honoured_afterwards`, where the grant comes from `alo_picking::Picker` over a real folder. |
| a revoked grant does not come back | `alo-remembering`, `the_grants_a_machine_keeps`, `a_revoked_grant_does_not_come_back` |
| an expired one is gone when it is read | `alo-remembering`, `the_grants_a_machine_keeps`, `an_expired_grant_is_gone_when_the_list_is_read` — `len` is zero, not one behind a filter |
| the file is the person's alone, and a store somebody else could write is refused in words | `alo-remembering`, `the_grants_a_machine_keeps`, `a_file_somebody_else_could_write_is_refused_in_words` — plus the link, and `keeping::tests::only_roots_grants_or_our_own_are_believed` for the branch a test cannot chown its way to |
| nothing an agent can send over the socket writes a byte of it | `alo-agentd`, `lib`, `starting::tests::nothing_an_agent_says_writes_a_byte_of_the_grants` — four requests over a real socket, the file compared byte for byte, and the directory checked for anything written beside it |

Refusal paths beyond those: a file from another format, a key nobody declared, a
grant over nothing or over two things, a grant that ends before it begins, one
handle on two grants, a handle that would be handed out again, a missing folder
refused rather than made, and a stale staging file.

## Limitations

- The live hand-over is task 4, described above.
- Ownership by a *third* login is tested as a rule (`believed`) rather than on a
  real file: a test cannot chown without root. The same shape as
  `alo-accounts`', and for the same reason.
- Nothing here has run on a booted alo OS machine; the file's directory and mode
  are the image's, and delivery-plan task 10 is where the two meet.

## Proposed shared-document updates

I did not edit `CHANGELOG.md`, `ROADMAP.md`, `docs/autonomy/QUEUE.md` or
`docs/autonomy/STATE.md`. For the integration owner:

**CHANGELOG.md**, under the unreleased section:

> **Your machine remembers what you granted.** Until now a folder you granted
> was forgotten the moment the agent service stopped, so *see what is granted*
> had nothing to show the next morning. Grants are now kept in a file that is
> yours alone, they still expire exactly when they were always going to, a grant
> you revoked does not come back, and a grant that has run out is gone rather
> than hidden. If that file is ever one somebody else could write, the agent
> service refuses to run and says so instead of quietly acting as though you had
> granted nothing.

**ROADMAP.md / QUEUE.md:** lane B task 3 is done; task 4 (*a grant made now
reaches the daemon now*) is written and ready. No task in
`v0-01-delivery-plan.md` matched this one, so nothing was marked there. The
`Grants` line of `docs/features.md` now has *pick*, *it expires* and *it is
kept* in code; *see what is granted* and *revoke it* still wait on the
compositor lane's surface.
