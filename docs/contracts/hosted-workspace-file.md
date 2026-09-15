# Contract — the file that says which workspace an alo machine hosts

**Status:** contract. Additive changes only; a break requires versioning and a
deprecation period. See `CLAUDE.md`, "Contracts outlive code". Added 2026-09-14.

This is the file that tells `alo-agentd` a workspace — `alo-workplace`'s mail,
files, chat and documents — is served on this machine, and on which port, so that
the machine's own discovery responder answers for it on the local network under
the machine's own identity (`docs/contracts/local-network-wire.md`, *A workspace
on the network*). It is the whole of what an alo machine is told about a workspace
it hosts.

Read `docs/decisions/0003-the-network-is-not-authority.md` first: discovery
reveals presence and nothing else, and **hosting a workspace grants nothing and
pairs nothing.** `crates/alo-agentd`'s `hosting.rs` is the reader, and
`crates/alo-nearby`'s `answering.rs` the answer it causes.

## Who writes it, and who reads it

- **Written by the package that installs the workspace server, as root** — which
  is `alo-workplace`'s, outside this repository. Written when the server is
  installed, changed when its port changes, and removed when the server is
  removed.
- **Read by `alo-agentd`, once, as it starts.** A change takes effect the next
  time `alo-agentd` starts; the package that changes the file restarts the
  service it installed beside, or the change waits for the next sign-in. The
  daemon never writes it, and **nothing on either of its doors — the person's or
  the agent's — writes, names or changes it**: there is no request carrying a
  port and no request naming this file. An agent cannot reach it.

## Where it is

```
/etc/alo/workspace.toml
```

In `/etc/alo`, beside the machine description (`agentd.toml`), because it is
installed configuration rather than something a person made. The path is a
constant.

## What it looks like

```toml
port = 8443
```

| Key | Meaning |
|---|---|
| `port` | The TCP port the hosted workspace answers on: a whole number from 1 to 65535, and not `7610`, the port `alo-agentd` itself answers proposals, verbs and questions on. Required. |

**One key.** There is no key for the workspace's name, its organisation, a URL, a
login page, a certificate, an address or an identity. The identity advertised is
always the machine's own (`/var/lib/alo/machine-id`), and the address is measured
by whoever hears the answer. A file carrying any other key is refused rather than
read around, because what it says is spoken to everything on the network.

## The rules it is held to

Asked of the open file, not of the path, so the file checked and the file read
are the same file:

- **not a symbolic link**;
- **a regular file**;
- **owned by root** (uid 0), and by nobody else — not the person `alo-agentd`
  runs as, and not the agent's login;
- **writable by nobody but its owner** — a group- or world-writable file is
  refused.

Then, of what it says: valid TOML, exactly the one key `port`, a whole number,
from 1 to 65535, and not `7610`.

## What happens when it is absent, or refused

- **No file** is a machine hosting no workspace. Nothing is advertised, and the
  question for workspaces is stepped over. This is every alo machine without a
  workspace server, and it is not an error.
- **A file that fails any rule above** is refused with one line in the service
  log naming the file and what to change, and **the machine advertises no
  workspace**. `alo-agentd` still starts and serves the person: a wrong workspace
  file is not a reason to stop, and it is never a reason to advertise a guess.
- **The person is told either way** (added 2026-09-15, additively): the person's
  door answers `advertised` (`docs/contracts/daemon-protocol.md`) with the port
  the running service advertises, `hosts-none` for no file, or — for a refused
  file — one of three sentences in the person's language saying no workspace is
  advertised and what to do, naming no path, owner or mode. It is what the
  service read at start: asking reads nothing again.

## Versioning

There is no `format` key, and version one of this file is the one key above.
Anything added later is added as a new key that version one refuses — so a file
written for a later alo OS is refused by an earlier one rather than half-read,
and an earlier machine advertises nothing rather than something the file did not
mean.
