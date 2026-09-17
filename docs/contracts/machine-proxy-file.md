# Contract — the machine's proxy file

**Status:** contract. Additive changes only; a break requires versioning and a
deprecation period. See `CLAUDE.md`, "Contracts outlive code".

`docs/features.md` v0.5 promises a proxy **machine-wide, and honoured by
applications**. `alo-proxy` holds the one setting and decides which way every
road out goes; this is the file that setting is kept in, so that everything on
the machine reads one answer. ADR 0049 §3 is the decision.

## Where it is, and who writes it

| | |
|---|---|
| Path | `/etc/alo-proxy/proxy.json` (`alo_networks::proxy_file::THE_MACHINES_PROXY`) |
| Directory | `/etc/alo-proxy`, `0755`, root's — made by the broker's unit (`ConfigurationDirectory=alo-proxy`) |
| File | `0644`, root's |
| Writer | `alo-brokerd`, and nothing else, carrying out `network.set-proxy` |
| Readers | every road out of the machine, and whatever publishes the proxy to applications |

The broker replaces it whole: the new file is written beside it, synced, and
renamed over it, so a reader never reads half of one. A machine where it has
never been written has no proxy, and it is the person's to set.

## What it holds

One JSON object — `alo_proxy::Kept` as `serde_json` writes it — and a newline:

```json
{"proxy":{"proxy":"manual","http":{"spoken_to":"http","host":"proxy.example.com","port":3128},"https":{"spoken_to":"http","host":"proxy.example.com","port":3128},"exceptions":["intranet.example.com"]},"set_by":"this-person"}
```

| Field | |
|---|---|
| `proxy` | `alo_proxy::TheProxy`: `{"proxy":"none"}`, `{"proxy":"manual", "http"?, "https"?, "exceptions"}`, or `{"proxy":"automatic","at":"<address of a configuration>"}` |
| `proxy.http`, `proxy.https` | `spoken_to` (`http` or `https`), `host`, `port`, and — only together — `name` and `password`, where `password` is **the name the password is kept under in the keyring, never a password** (ADR 0022) |
| `set_by` | `an-organisation` or `this-person` |

**A reader rebuilds what it reads** through `alo-proxy`'s checked constructors
(`alo_networks::proxy_file::kept_on_this_machine`) and treats a file that does not
come back identical as no file it can use — never as *no proxy*, which on a
company network is a machine that silently goes nowhere.

## How a person's proxy reaches it

The broker's door takes no text (`agent-verbs.md`, *The privileged broker*). So
Settings writes the chosen `alo_proxy::TheProxy`, as `serde_json` writes it and
nothing more, to `/run/alo-broker/wanted/proxy.json`
(`alo_networks::proxy_file::THE_WANTED_PROXY`) — a folder the broker makes `0770`
in the person's group at start-up — and asks for `network.set-proxy` with the
SHA-256 of exactly those bytes. The broker sets it only when:

1. the handed-over file is a plain file owned by the person, opened without
   following a link, and no longer than 64 KiB;
2. its bytes digest to the approved identity;
3. it rebuilds identically through `alo-proxy`'s checks;
4. the machine's current proxy was not set by an organisation.

Then it writes this file with `set_by` `this-person`, and removes the handed-over
one. In every other case this file is left exactly as it was, and the broker's
record says `not-carried`.

## What is not here

- **An organisation's proxy** is read from the machine description by
  `alo-agentd` (`alo_proxy::Kept`'s own documentation); how it comes to be written
  into this file is not decided by this contract.
- **Who reads it today.** No road out reads this file yet: `alo-software`,
  `alo-updating` and `alo-models` are handed a `TheProxy` by their callers. Wiring
  each reader to this file is owed, and named in the report for the network task.
