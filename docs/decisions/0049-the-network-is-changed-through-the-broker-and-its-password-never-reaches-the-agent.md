# ADR 0049 — The network is changed through the broker, and its password never reaches the agent

**Status:** accepted, 2026-09-16. Written by task 3 of
`docs/autonomy/v0-5-the-broker-and-the-disk-plan.md` (*Network, through the
broker*), whose code is built on it, as ADR 0042 was by the task built on it.
**Date:** 2026-09-16
**Context:** [ADR 0001](0001-the-capability-model.md) (§2: operations needing
privilege sit behind a broker with no free-form parameters; §5: a change is
proposed as one sentence and approved once);
[ADR 0009](0009-a-good-computer-without-the-agent.md) (what an agent verb does, a
person does by hand); [ADR 0011](0011-the-base-is-rented-and-the-image-is-a-container.md)
(NetworkManager is rented, configured and never patched);
[ADR 0016](0016-the-organisation-bounds-and-the-person-chooses.md) (an
organisation's setting is theirs and a person is told so);
[ADR 0022](0022-where-a-providers-key-is-kept.md) (a credential is never a field
in a setting); `docs/features.md` v0.5 ★ *System verbs through the privileged
broker* and *Corporate proxy support, machine-wide*.

**Numbering.** 0047 is taken by the printers' decision, written on 2026-09-16 and
held back unpublished with that task. This record takes the next number so the
two do not collide when that one is published.

## The question in one line

**What does an agent name a network by, where does a Wi-Fi password go, who sets
the proxy, and how does a person know a change will cut the conversation they
are having?**

## §1 A network is named by the name it announces, and compared by what was reported

**Options.** (A) The agent names the network by a device, a file or a connection
identifier — machinery a person cannot judge in a sentence. (B) By the name it
announces, matched exactly when the change is carried out. **B.**

What crosses the broker's door is never the name. It is the SHA-256 of what the
network manager reported: for a network in range, its name **and how it is
protected** (open, a password, an organisation's sign-in); for a saved network,
the network manager's own identifier. The protection is part of the identity so
that an open network announcing the name of a protected one next door is a
different network, and an approval of one is never carried out on the other. A
name matching no network, or two, changes nothing and sends the person to
Settings, where each network shows whether it asks for a password.

## §2 None of the three verbs needs a grant

A grant is over a path or an application (ADR 0001 §3), and a network is
neither: `join_network`, `forget_network` and `switch_wireless` change the
machine's own network settings. What protects the person is the approval — one
sentence naming the network and what the change does to the conversation,
answered once. The reason is carried in each declaration, as
`docs/contracts/agent-verbs.md` rule 5 asks.

## §3 The proxy is set by a person, and no agent proposes one in v0.5

**Options.**

- **(A) An agent verb taking a proxy address.** Every connection this machine
  makes would go through an address a model wrote, and *your IT department says
  to use proxy.example.net* is exactly the sentence a document can persuade a
  model to propose and a person to approve. The approval would be doing all of
  the work against an attack aimed at the approval.
- **(B) A person sets it in Settings, through the broker's `network.set-proxy`.**
  The address is one somebody handed them, typed where they would look for it.
  **Chosen.**

The broker's door takes no text, so a person's choice is handed over as a file in
`/run/alo-broker/wanted`, a folder the broker makes `0770` in the person's group,
and the request carries the digest of exactly those bytes. The broker reads it
without following a link, only as a plain file owned by the person, rebuilds it
through `alo-proxy`'s own checks, refuses it over a proxy an organisation set
(ADR 0016), and writes the machine's proxy file
(`docs/contracts/machine-proxy-file.md`), `0644` in `/etc/alo-proxy`, whole.

**Consequence.** An agent verb for the proxy is additive later, if a shape is
found that does not let a model choose where everything goes.

## §4 A Wi-Fi password is asked of the person by the network manager's own protocol

**Options.**

- **(A) A password argument on `join_network`.** Forbidden by the plan, and the
  password would pass through the model, the turn and the record.
- **(B) Settings asks, and hands the password to the broker.** The broker would
  hold a secret, and its door would need a field that carries text — the one
  thing ADR 0001 §2 says it has none of.
- **(C) The network manager asks.** Joining hands it no password and no settings
  of ours; it calls `GetSecrets` on the secret agents registered on the system
  bus, and the one alo OS registers runs in the person's own session, asks the
  person there, and answers. **Chosen.**

The agent (`alo_networks::secret_agent`) answers **only the connection that owns
the network manager's name, running as the network manager's user**; anything
else that can send to the person's connection — the agent's own login among it —
is refused before the person is asked, because otherwise it could make the
machine ask the person for a password and be handed the answer. It asks only when
the network manager says the person may be asked, and answers only a password
network's key. `WifiPassword` has no way out but that one answer.

## §5 What a change does to the conversation is part of the sentence, and must be true

A person approving *turn Wi-Fi off* while the assistant is answered by a provider
over that Wi-Fi would watch the conversation stop and not know why.

**Options.** (A) A warning drawn beside the sentence. What a person approves is
the sentence (ADR 0001 §5), so a warning outside it is not part of what was
approved and is not in the record. (B) An argument, `this_conversation`, that the
sentence names — *…, and this conversation loses its connection until this
machine is connected again*. **B.**

The agent supplies it, so it is checked: `alo_changing_network::proposable`
refuses a call whose value is not what `alo_changing_network::would` works out
from the network manager's report and from where the conversation is answered,
in either direction; and carrying out refuses an approval that is no longer true
— a cable unplugged between the approval and the change — without asking the
broker. A conversation answered on this machine keeps its connection whatever
the Wi-Fi does, and is never told otherwise.

## §6 Not in v0.5

- **No VPN configuration verb**, on the agent's list or the broker's. Its absence
  is recorded here, as the plan asks.
- **No network asking for an organisation's sign-in (802.1X) is joined** through
  the broker: joining one needs settings a person cannot judge in a sentence, and
  the secret agent does not answer for one.
- **No verb lists the networks nearby.** The networks around a machine say where
  it is.

## Consequences

- `crates/alo-networks` holds what the network manager reports, the client, the
  secret agent and the proxy file's shape; `crates/alo-brokerd` carries the four
  network verbs out; `crates/alo-changing-network` declares the three agent verbs
  and holds the one road from an approval, or a person's pick, to the door.
- No turn offers these verbs yet, and the shell neither registers the secret
  agent nor draws the password surface yet. Both are owed and named in the task's
  report; until the secret agent is registered, joining a protected network the
  machine has no saved password for is `not-carried`.
- Whether NetworkManager on a certified machine asks a secret agent in the
  person's session for a connection root activated is **not measured**, and is
  the first thing to check on a booted image.
