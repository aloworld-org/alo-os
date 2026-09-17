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

**alo OS's own image ships one**, at `image/etc/alo/agentd.toml`, and that is not
an exception to the sentence above: the image is what installs that machine. What
it may write is only what it really decided — the two login numbers are the
accounts it creates, and `record.keeping` is `"forever"` because `ADR 0004` gives
retention to the organisation and alo OS ships no number of days of its own. An
organisation with a retention rule replaces the file with theirs.
`crates/alo-image` is what holds the shipped one to the accounts beside it.

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
| `format` | Which shape this description is in. Required. `1`; `2` or later when it carries `[questions]`; `3` when it carries `[applications]`. |

A description that says a number this service does not read is **refused rather
than guessed at**,
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

**The file is made if it is not there; the folder holding it is not.** A machine
that has never run has no record and gets one on its first start, with its first
line already written. The directory above it is whoever installs the machine's,
and `alo-agentd` refuses to start rather than making one — a typo in this key
would otherwise become a second record nobody is reading.

**`keeping` is the retention rule**, and `n` may not be `0` — a record kept for
no days is a record deleted as it is written, which is the record turned off
wearing a retention setting's clothes. ADR 0004 gives this to the organisation
that manages the machine; alo OS ships no number of days of its own, because how
long an organisation may keep a record of what its staff's machines did has a
legal answer in some places and a cultural one in others.

## `[questions]`

Where an organisation permits a person's questions to be answered — ADR 0016's
bound, which the organisation sets and the person chooses within.

| Field | Meaning |
|---|---|
| `may-go` | `"anywhere"`, `"in-the-building"`, `"this-machine-only"`, or `"in-a-region"`. Required when the section is present. |
| `region` | Which region, and **only** when `may-go` is `"in-a-region"`. A provider that has not stated where it runs never satisfies it, and is refused as *unknown* — the sentence says the provider did not say, never that it runs outside the region. |

**The section is optional and its absence is the common case.** A machine no
organisation manages has no policy at all — *not empty, not permissive by
default, absent* (ADR 0016) — and a person on it chooses freely. Absence is not
written as `"anywhere"`: the two permit exactly the same things, and what they do
not share is somebody to name in a refusal.

**A section that is present and does not hold is refused, and the service does
not start.** It is never read as unrestricted, which is the one failure worth
avoiding here: an organisation that wrote a policy and got no policy, with
nothing on the machine saying so. So each of these refuses —

- a `may-go` this service does not know;
- `may-go = "in-a-region"` with no `region`;
- a `region` beside any other `may-go`, because a key that does nothing is a key
  somebody believes is doing something;
- `[questions]` with no `may-go` at all.

**It bounds where a question may be answered and nothing else.** It does not name
a provider, choose a model, or touch what the person picked: their choices live
in their own settings (`docs/contracts/person-settings.md`) and are never
rewritten from here. What it can do is refuse one — and then the person is told
what the rule is and that an administrator set it.

**Who is named in that refusal is decided by who owns this file**, and by nothing
about the rule itself. *Who may write it* below permits two owners, and they are
two different people: root is an organisation's configuration system (ADR 0004)
or whoever installed the machine, so there is an administrator to name; the
person alo-agentd runs as is the owner of their own machine, so there is not. A
person who writes `may-go = "this-machine-only"` into their own description is
bounded exactly as strictly and is told no administrator did it — a restrictive
value is never, on its own, evidence that somebody else set it. **An organisation
that wants its rule attributed to it writes this file as root**, which is where
its configuration system writes into `/etc` anyway.

**Nothing here is fleet management.** There is no enrolment, no identity, no
reporting and no key naming a server. This file is read from the disk it is on,
as it always was, and by the same rules in *Who may write it* below.

### It requires `format = 2`

This is the first of the two places the additive rule below does not reach, and the reason is
worth stating plainly. An older service reading a description that carries
`[questions]` would ignore the section and go on sending questions wherever the
person chose — **an organisation's policy silently not enforced**, which is not
"the same machine" under any reading.

So a description carrying `[questions]` says `format = 2`, and a service that
reads only `1` refuses it and does not start. That is the fail-closed direction:
a managed machine whose alo OS is too old to understand its policy does not run
unmanaged — it does not run.

A description **without** `[questions]` means the same thing under either number,
so both are read and neither is a migration anybody performs. `1` is every
description that exists today and it goes on working untouched; `2` without a
policy is the same machine, which matters when an organisation **takes** a bound
off — deleting the section is the whole edit, and nothing has to be renumbered
back. What is refused is `format = 1` carrying a `[questions]` section, because
that is a file claiming an older service could have read it correctly when it
could not.

alo OS's own image ships `format = 1` and no `[questions]`: the image installs an
unmanaged machine, and an organisation with a policy replaces the file with
theirs. `crates/alo-image` reads the shipped file for the handful of things the
image is answerable for and knows only `1` — which is correct for what it checks,
and is why a policy in a built image would need that reader taught the section
first.

## `[applications]`

Which places an organisation permits applications to come from — the same
ADR 0016 bound as `[questions]`, for installing and updating an application
rather than for answering a question.

```toml
format = 3

[applications]
may-come-from = ["acme-apps", "flathub"]
```

| Field | Meaning |
|---|---|
| `may-come-from` | Every place an application may be installed or updated from, by the name this machine knows it by. Required when the section is present. |

**A name is what this machine calls a place applications come from** — the name
the place was set up under, which is the name a person installs by. Letters,
digits, `.`, `_` and `-`, not beginning with `-`, at most 64 characters. Names
are matched exactly, as grants are: `Flathub` is not `flathub`. Naming a place
here does **not** set one up; a place that is permitted and was never set up on
the machine is still not somewhere anything is installed from, and a place that
is set up and not named is refused.

**The section is optional and its absence is the common case**, exactly as
`[questions]`: no section is no rule — *absent, not permissive by default* — and
a person on that machine installs from any place set up on it.

**An empty list is a rule, and it keeps every place out.** An organisation that
wrote `may-come-from = []` wrote a rule; it is never read as no section.

**A section that is present and does not hold is refused, and the service does
not start** — never read as unrestricted. Each of these refuses:

- `[applications]` with no `may-come-from`, or with one that is not a list of
  strings;
- a key in the section this service does not know;
- a name that could never be a place's name — empty, with a space or a `/` in
  it, or beginning with `-`;
- the same place named twice, which is most often a line copied and not
  edited, with the place that was meant missing.

**When a place is kept out, the person is told who set the rule, and who set it
is decided by who owns this file** — the rule `[questions]` states, for the same
reason. Root's file is an organisation's, and the refusal says *the organisation
that manages this machine does not permit installing applications from …*. The
person's own file is theirs, and the refusal says no organisation set it. A
restrictive list is never, on its own, evidence that somebody else wrote it.

It bounds **where** applications come from and nothing else. It does not set up
a place, choose an application, or turn a place's signature checking on or off;
a place set up without checking is refused whatever this list says.

### It requires `format = 3`

For `[questions]`' reason, one shape later. An older service reading a
description that carries `[applications]` would not enforce it and would install
from any place the person set up — **an organisation's rule silently not
enforced**. So a description carrying `[applications]` says `format = 3`; a
service that reads only `1` and `2` refuses it and does not start, and a `1` or
`2` carrying the section is refused as a file claiming an older service could
have read it correctly.

A description without the section means the same machine under `1`, `2` and `3`,
and none of them is a migration anybody performs. `[questions]` is read in a `3`
exactly as in a `2`.

## What is **not** in it

**Where the socket goes.** It is `/run/alo/<uid>/agentd.sock` for the person
this file already names, and `docs/contracts/daemon-protocol.md` says so. A key
for it would be a second answer to a question the two logins above settle — and
a key naming a directory is a key somebody can point at one they own.

The parent, `/run/alo`, is the image's: it is made at boot through `tmpfiles.d`,
and `alo-agentd` **refuses to start** rather than create it (ADR 0017). Until
2026-09-04 the socket was `$XDG_RUNTIME_DIR/alo/agentd.sock`, which was
unreachable by the agent on any real machine; the ADR is why it moved and what
was rejected on the way.

**Anything secret.** There are login numbers, two lengths of time, a path, a
retention rule and, where an organisation set them, two bounds in this file and
nothing else — the names of places are not secrets. A provider's key lives in the keyring and
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

New keys may be added and `format` stays as it is for as long as an older
service reading the file without them would still describe the same machine.
`[questions]` and `[applications]` are the two sections that fail that test and
say why, above: ignoring a policy is not describing the same machine. Anything
else — a key removed, a meaning changed, a default introduced — is a new
`format`, and a service refuses a number it does not read.
