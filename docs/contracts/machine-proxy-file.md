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

## And how its password reaches the machine's own credentials

**Added 2026-09-20 by ADR 0060, additively: a proxy handed over with no password
is asked for under exactly its own bytes, as above, and nothing here changes for
a machine that does not use it.**

A person on their own machine may give their proxy the password it asks for,
from the same place they set the proxy. It is a second handed-over file, in the
same folder, `0600` and the person's:

| | |
|---|---|
| Path | `/run/alo-broker/wanted/proxy-password` (`alo_networks::proxy_password::THE_WANTED_PASSWORD`) |
| Holds | 32 bytes of the kernel's randomness, then the password (`alo_proxy::provisioning::handed_over`) |
| Asked for as | the SHA-256 of the proxy's bytes **followed by** these (`alo_networks::proxy_password::handed_over_together`) |

The nonce is there because the identity is written into the broker's record and
a digest of a password is a password (ADR 0060 §3).

**One act, one approval.** There is no second verb: `network.set-proxy` writes
the credential **first** and this file only if that succeeded, so a machine is
never left with a proxy set that it cannot sign in to. The broker sets it only
when, beside the four above:

5. the proxy signs in under `the-proxy-on-this-machine`
   (`alo_proxy::THE_PERSONS_PROXY_PASSWORD`) — the one name a person's own
   machine keeps a proxy password by, which is what a unit file's static
   `LoadCredentialEncrypted=` line names (ADR 0060 §2). Any other name is
   refused and nothing is written;
6. the credential is written into `/etc/credstore.encrypted`, `0600` and root's,
   by `systemd-creds` and by no encryption of ours (ADR 0011).

The handed-over password is removed whichever way the act went, refusals
included.

**How a settings panel knows a password is set:** this file names where the
password is kept, and the broker writes this file only after the credential is
written. So `proxy.http.password` (or `proxy.https.password`) being
`the-proxy-on-this-machine` **is** the machine's statement that one is set. It
is never read back out; replacing it is giving a new one.

## What is not here

- **An organisation's proxy** is read from the machine description by
  `alo-agentd` (`alo_proxy::Kept`'s own documentation); how it comes to be written
  into this file is not decided by this contract.
- **Who reads it today**, corrected 2026-09-20. Two roads out read this file:
  `alo-looking-once` (the one look a machine takes on its way up) and
  `alo-software` (installing an application and updating one), each through
  `alo_networks::proxy_file::kept_on_this_machine` and neither with a reader of
  its own. **`alo-updating` and `alo-models` are still handed a `TheProxy` by
  their callers**, and wiring those two remains owed. The line this replaces
  said *no road out reads this file yet*, which stopped being true when
  `alo-looking-once` landed and was not corrected then.
