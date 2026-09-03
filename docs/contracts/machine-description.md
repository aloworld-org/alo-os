# Contract — what a machine says about itself

**Status:** contract. Additive changes only; a break requires versioning and a
deprecation period. See `CLAUDE.md`, "Contracts outlive code".

This is the file `alo-agentd` is told what machine it is running on by. It is
written by whoever installs a machine, or by the configuration system of an
organisation that manages one (ADR 0004), and it is read once when the service
starts. Nothing writes it back, and there is no setting in it that alo OS
changes on somebody's behalf.

`docs/contracts/daemon-protocol.md` is what a client says to this service;
`docs/contracts/record-file.md` is what the service writes down. This is the
third of the three and it is the only one a person types.

## Where it is

```
/etc/alo/agentd.toml
```

The service reads that path and no other. It reads it **as it is**: the last
part of the path may not be a symbolic link, and what is opened is what was
looked at rather than whatever the name pointed at a moment later.

## The shape

TOML, because this one is typed by a person — the record and the protocol are
JSON because a program writes them and a program reads them. A comment is how
the person after you finds out why a number is what it is, and TOML has them.

```toml
format = 1

[logins]
person = 1000
agent = 989
group = 989

[agent]
name = "alo"
turn-seconds = 900
proposal-seconds = 300

[record]
path = "/var/lib/alo/record"
keeping = { for-days = 90 }
```

**Every key is required.** There are no defaults, and a key left out is a
machine that does not start rather than a machine running under a number nobody
chose. That is item 23's rule about `alo_models::Driving::NotMeasured`, applied
to the file that says what a machine is: *nobody decided this* must never be
read as *probably fine*.

**A key this service does not know is refused**, naming the line it is on. A
newer alo OS may add keys; what says so is `format`, and a typo is not an
addition.

## `format`

| Field | Meaning |
|---|---|
| `format` | Which shape this description is in. Required. `1` today. |

A description that says anything but `1` is **refused rather than guessed at**,
which is the rule `docs/contracts/record-file.md` states about a record from a
newer alo OS. It is answered before any other value in the file, so a
description written for a later alo OS is refused as one rather than as
whichever of its keys this service happened not to know.

## `[logins]`

| Field | Meaning |
|---|---|
| `person` | The signed-in person, whom `alo-agentd` runs as. |
| `agent` | The agent, which is a login of its own. |
| `group` | The group both are in, which is how the agent reaches the socket. |

These are what decide which of the protocol's two doors a connection is on.
Nothing on the wire says who a caller is — the kernel does, through
`SO_PEERCRED` — so these three numbers are the whole of the division between an
agent that proposes a change and a person who approves it.

Three refusals follow from that, and each of them means no socket at all rather
than a socket with one door:

- **`person` and `agent` may not be the same login.** On such a machine the side
  that proposes a change would also be the side that approves it, and ADR 0001
  §5 would be a sentence in a contract with nothing underneath it.
- **`agent` may not be `0`.** An agent running as root holds authority the
  person does not (ADR 0001 §2).
- **`4294967295` is not a user or a group.** It is what a Unix call answers with
  when there is no user, and it is what a script that could not look one up
  leaves in a file.

`group` grants nothing on its own: being in it means being able to knock. Which
door opens is `person` and `agent`.

## `[agent]`

| Field | Meaning |
|---|---|
| `name` | What this machine's agent is called, exactly as its grants name it. |
| `turn-seconds` | How long a turn's own grant lasts, in whole seconds. |
| `proposal-seconds` | How long a change waits for an answer, in whole seconds. |

**`name` is matched exactly.** Grants are matched exactly everywhere in alo OS
(ADR 0001 §3), so a name that differs from the one the grants were made to is a
machine on which every turn is refused. It may not be empty.

**Neither length of time may be `0`, and neither may be longer than 86400** —
one day. The ceiling is alo OS's rather than an organisation's, and the reason
is `CLAUDE.md`: *what a person approves is that sentence, and an approval is
never a session.* A proposal that stands for a week is an approval given on
Monday running on Friday's machine. Something too long is **refused, never
shortened**: a machine that quietly clamped a week to a day would be running
under a description nobody wrote.

## `[record]`

| Field | Meaning |
|---|---|
| `path` | The file `alo-agentd` writes what happened into. |
| `keeping` | How long it is kept: `"forever"`, or `{ for-days = n }`. |

**`path` must be absolute.** A relative one would put the evidence of what an
agent did wherever the service happened to be started from, and somewhere else
the next time. The file itself is `docs/contracts/record-file.md`.

**`keeping` is the retention rule**, and `n` may not be `0` — a record kept for
no days is a record deleted as it is written, which is the record turned off
wearing a retention setting's clothes. ADR 0004 gives this to the organisation
that manages the machine; alo OS ships no number of days of its own, because how
long an organisation may keep a record of what its staff's machines did has a
legal answer in some places and a cultural one in others.

## What is **not** in it

**Where the socket goes.** That is `$XDG_RUNTIME_DIR`, which the person's
session sets when they sign in and empties when they sign out, so it is a fact
about the session rather than a decision anybody wrote down. The socket is
`alo/agentd.sock` beneath it, and `docs/contracts/daemon-protocol.md` says so.

When the variable is not set, `alo-agentd` **refuses to start** rather than
guessing. `/tmp` is writable by everybody, so a directory there is one anybody
can create first; `/run/user/<uid>` worked out from the login is the session
manager's to make, and a service that made it because it wanted the name would
be standing in for a session that has not started.

**Anything secret.** There are two login numbers, two lengths of time and two
paths in this file and nothing else. A provider's key lives in the keyring and
never in a settings file (`crates/alo-models/src/provider.rs`), and that is not
relaxed here. The file may be world-readable — `0644` in `/etc` is the ordinary
case — and alo OS does not check that it is not, because checking would teach
whoever writes it that secrets may go in.

## Who may write it

The description names which login is the agent. Whoever can rewrite it can name
themselves this machine's agent — and then every read the person's grants permit
is theirs, on a service behaving exactly as it was told to. So before anything
in the file is parsed:

- it is **not a symbolic link**;
- it belongs to **root or to the person `alo-agentd` runs as**, and to nobody
  else. Both are ordinary: an organisation writes it into `/etc` as root, and a
  person whose machine it is may keep their own;
- **nobody else can write it** — a description that is group-writable or
  world-writable is one the group or the world describes this machine with.

These are checked on the open file rather than on the path, so a description
that is checked and a description that is read cannot be two different files.

## What changes additively

New keys may be added and `format` stays `1` for as long as an older service
reading the file without them would still describe the same machine. Anything
else — a key removed, a meaning changed, a default introduced — is a new
`format`, and a service refuses a number it does not read.
