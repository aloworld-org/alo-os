# Contract — the file a machine keeps its grants in

**Status:** contract. Additive changes only; a break requires versioning and a
deprecation period. See `CLAUDE.md`, "Contracts outlive code".

This is the file that says what has been granted on this machine: to which
agent or application, over which folder, file, application or facility, when it
was granted and when it ends. It is written by the person's side of the machine
— the folder picker and the surface that lists and revokes grants — and read by
`alo-agentd` at start and whenever the person's door knocks to say it changed.
An agent's door has no road to it.

Read `docs/decisions/0001-the-capability-model.md` first. What a grant *is* —
enumerated, deliberate, revocable, expiring, and never to `/` — is that
decision's and `crates/alo-capability`'s. What an application's grant is over,
and why it survives declining the agent, is
`docs/decisions/0040-what-an-applications-grant-is-over.md`.
`crates/alo-remembering`'s `written.rs` is the shape and `keeping.rs` is the
file.

## Where it is

```
/var/lib/alo/grants.toml
```

In the folder the image makes for what an agent did on this machine
(`0700 alo alo`), beside the record. The path is a constant: where a machine
keeps its grants is nobody's policy.

## What it looks like

```toml
format = 2
next = 2

[[grant]]
handle = 0
agent = "@files"
folder = "/home/ada/Invoices"
granted = 1760000000
expires = 1760003600

[[grant]]
handle = 1
applicant = "org.gnome.Cheese"
facility = "camera"
granted = 1760000000
expires = 1760086400
```

| Key | Meaning |
|---|---|
| `format` | Which shape the file is in. Required. `1` or `2`. |
| `next` | The handle the next grant made on this machine is given. A handle is never reused. |
| `handle` | What a person revokes this grant by. Unique in the file, and below `next`. |
| `agent` | The agent the grant is for, by its name. |
| `applicant` | The application the grant is for, by its identifier. **Format 2 only.** |
| `folder` | A folder and everything in it, by its full path. |
| `file` | Exactly one file, by its full path. |
| `application` | One installed application, by its identifier. |
| `facility` | Something this machine has that is not a path (below). **Format 2 only.** |
| `granted` | When it was made, in whole seconds since 1970. |
| `expires` | When it stops, in whole seconds since 1970. Never absent. |

Every grant names **exactly one** of `agent` and `applicant`, and **exactly
one** of `folder`, `file`, `application` and `facility`. A key nobody declared
is refused, not skipped.

### Facilities

A facility is matched exactly, and is granted only to an application — a file
that grants one to an agent is refused.

| Name | What it is |
|---|---|
| `camera` | The camera — whichever camera the machine has, never a device number |
| `microphone` | The microphone |
| `screen-once` | One picture of the screen |
| `screen-continuously` | Recording or sharing the screen |
| `notifications` | Sending the person notifications |
| `clipboard` | What the person copied |
| `desktop-background` | The desktop background |
| `appearance-settings` | Light or dark, accent and contrast |
| `sleep` | Keeping the machine awake |
| `network-state` | Whether the machine is connected, and how |
| `power-profile` | The power profile |
| `secrets` | A place for the application's own passwords in the person's keyring (added 2026-09-15) |

The list is closed. Adding a facility is additive; renaming or removing one
needs a new format.

## The two formats

**Format 1** holds agents' grants over a folder, a file or an application.
**Format 2** adds `applicant` and `facility` (ADR 0040). Both are read.

**The lowest format that holds the list is written.** A machine on which
nothing is granted to an application writes format 1, exactly as an alo OS
before format 2 did, so rolling an update back does not lose the folders a
person granted. A list with any application's grant writes format 2, and an
alo OS that reads only format 1 refuses it whole rather than guessing.

A file that says `format = 1` and names `applicant` or `facility` is refused:
no alo OS wrote it.

## Every grant is made again on the way in

Nothing deserialises a grant. Each table is handed to
`alo_capability::Grant::checked_for`, the road a person's pick takes, so a file
hand-edited into granting `/`, a relative path, a path with `..` in it, a grant
that ends before it begins, or a facility granted to an agent is refused in
that crate's words. A file with **one** such grant is refused **whole**: there
is no partial list, because a list that silently lost a grant, or kept one
nobody could check, lies about what is granted.

Expired grants are dropped as the file is read, before anybody has the list,
and are not written.

## Who may have written it

Believed only from root or from the login it belongs to, only when nobody else
can write it, and only as a real file rather than a symbolic link. Replaced
whole, never edited in place. These are `crates/alo-remembering`'s
`believing.rs`, shared with the pairings file.
