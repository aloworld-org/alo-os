# ADR 0059 — Where a machine-wide proxy password is kept

**Status:** accepted, 2026-09-20. Written by task 12 of
`docs/autonomy/v0-5-software-and-the-web-plan.md` (*A proxy that asks who you
are, signed in to on every road*), whose code is built on it — the shape
[ADR 0042](0042-installing-an-application-is-an-errand-and-an-agent-only-proposes-it.md)
and [ADR 0049](0049-the-network-is-changed-through-the-broker-and-its-password-never-reaches-the-agent.md)
were written in by the tasks built on them, and the shape that task's own
constraint asks for: *the decision is written first, because a store chosen
inside a crate would be a store four roads then have to agree with.*
**Date:** 2026-09-20
**Proposed by:** the software-and-the-web workstream
**Context:** [ADR 0016](0016-the-organisation-bounds-and-the-person-chooses.md)
(an organisation bounds and the person chooses);
[ADR 0017](0017-the-agents-door-is-ours-and-not-in-the-session.md) (the agent is
a different login and is refused the person's runtime directory);
[ADR 0022](0022-where-a-providers-key-is-kept.md) (where a **provider's** key is
kept, and the store this one is not);
[ADR 0011](0011-the-base-is-rented-and-the-image-is-a-container.md) (the base is
rented, configured and never patched);
[ADR 0053](0053-an-update-is-carried-out-by-a-unit-the-broker-starts-never-by-the-broker.md)
(an update is carried out by a unit the broker starts);
`crates/alo-proxy/src/password.rs`, `crates/alo-agentd/src/machine_wide_proxy.rs`,
`docs/contracts/machine-description.md`, `docs/contracts/machine-proxy-file.md`,
`docs/features.md` v0.5 *corporate proxy support, machine-wide and honoured by
applications*.

## The question in one line

**A company proxy asks who you are. Where does that password live, whose is it,
and what does a road out do when nobody has signed in yet?**

`[proxy]` in `/etc/alo/agentd.toml` already names `sign-in-as` and
`password-in-keyring`; `alo_proxy::ProxyAddress` already carries both; and
`alo_proxy::Carried::with_the_password` is already the one door a proxy
credential becomes text through. **No road in this workspace called it.** So on a
network whose own header says there is no other route out, a machine reached its
company's proxy as somebody with no password and was refused by the proxy. This
record is what the four roads read the password from.

## Why ADR 0022's store is not this store

ADR 0022 is settled and this record does not touch it: a **provider's** key
lives in the Secret Service, reached over the person's own session bus at
`/run/user/<uid>/bus`, derived from the daemon's own uid. That is right for a key
somebody typed into their own settings panel, and it is wrong here for three
reasons that are about *when* and *whose* rather than about taste.

| | A provider's key (ADR 0022) | A machine-wide proxy password |
|---|---|---|
| **Whose** | the person's, typed by them | the machine's, set by whoever administers it — an organisation on a managed machine (ADR 0016) |
| **When it is wanted** | inside a turn, with the person signed in | at `multi-user.target`, before anybody has signed in |
| **Who wants it** | `alo-agentd`, as the person | `alo-agentd` as the person, `alo-looking-once` as the person with no session, and root units besides |

The third row is decisive. `alo-looking-once` runs at `multi-user.target` and
asks whether there is an update; on a machine whose proxy asks for a name, a
store that needs a session means that check never happens on any machine that
boots to a login screen and waits. `/run/user/<uid>` does not exist until
`logind` makes it, and ADR 0022 records exactly that state as **unavailable**.

The second reason is not a scheduling detail either. **A machine-wide credential
kept in one person's keyring is the organisation's secret held by whoever
happens to log in**, which is the wrong answer to *whose is it* even on a
machine where it would work.

## The options

**(A) The person's Secret Service — ADR 0022's store.** Rejected above: no
session, no bus, no answer, on the roads that most need one. It would also make
the machine's proxy stop working the moment the person logs out, and work
differently for a second person on the same machine.

**(B) A file alo OS owns, root-owned and `0600`.** This is *the thing not to
invent*, in ADR 0022's own words, and it fails on its own terms here: two of the
four roads are taken by processes running as the person, so a root-only file
cannot be read by them, and a file they *can* read is a proxy password readable
by everything the person runs. `[proxy]` already refuses a password written into
`/etc/alo/agentd.toml`; keeping one in the file beside it would make that
refusal decorative.

**(C) The kernel keyring (`@u`).** No daemon, no bus, no session — and nothing
survives a reboot, so a machine's proxy password would be typed again after
every restart by somebody who may not be in the room. It is also per-uid, so the
root units and the person's units would look in different keyrings for the same
machine-wide setting.

**(D) The machine's own credentials, provisioned to each unit by systemd.**
**Chosen.** `systemd-creds` is a mechanism the base already has (ADR 0011: the
base is rented, and this is renting it rather than writing one). A credential is
encrypted at rest with a root-only host key, the TPM, or both; **PID 1 decrypts
it at unit start** and places it in a per-unit directory as a file readable only
by the unit's own user; it is gone when the unit stops. It needs no session, no
bus and nobody signed in, and it is provisioned by exactly the person who sets
`[proxy]` — whoever administers the machine.

ADR 0022 names `systemd-creds` and rules it out, and the sentence it rules it
out with is the sentence that selects it here: *"it is for credentials an
**administrator provisions to a unit** … A person adding a provider at runtime
cannot write one."* A machine-wide proxy password is not a person adding
something at runtime. It is an administrator provisioning one to a unit.

## The decision

**Store.** The machine's own credentials, read at
**`/run/credentials/<unit>/<entry>`**, where `<entry>` is the name
`password-in-keyring` already gives. `crates/alo-proxy/src/provisioned.rs` is
the reader, and `alo_proxy::signed_in` is the one function that puts what it
answers on a road.

**Whose.** The machine's, set by whoever administers it: an organisation on a
machine it manages, and the person on their own machine acting as their own
administrator — ADR 0016's split, unchanged. Nothing about *which* proxy is set
moves: `alo_proxy::Kept` still answers who set it, and a person is still refused
a change on a machine an organisation manages.

**What a road does when nobody has signed in.** It takes the road. That is the
property this store was chosen for: a credential delivered to a unit at its
start does not wait for a person. A unit that was **not** given one **refuses
the road in words** — never reaching the proxy as somebody with no password, and
never going straight out around it.

**How it reaches a unit**, which is a line in a unit file rather than code:

```
LoadCredentialEncrypted=the company proxy:/etc/credstore.encrypted/the-company-proxy
```

with the credential written once by whoever administers the machine:

```
systemd-creds encrypt --name="the company proxy" - /etc/credstore.encrypted/the-company-proxy
```

**Which unit is named, never worked out.** `alo_proxy::TheMachinesPasswords::given_to`
takes the unit's name and nothing else, and each road's crate writes its own as
a constant. This is deliberately the shape `alo_secrets::TheBus::of` has, for the
reason that crate gives: *a function that takes a name cannot be pointed
somewhere by a variable.* systemd also exports `$CREDENTIALS_DIRECTORY`, and
**reading it is rejected here** — it would be one environment variable deciding
where alo OS looks for a credential, which is the thing ADR 0022 refused
`DBUS_SESSION_BUS_ADDRESS` for. Naming the unit has a second benefit worth
having: the constant in the crate is the list of unit files that must carry the
line, written where somebody adding a road will see it.

## What is refused, and none of it falls back

Six states, told apart, each refused in words that say what to do and **quote
nothing that was stored**:

| What the machine finds | What happens |
|---|---|
| the unit was given no credentials at all | refused — this machine has not been given the password |
| credentials, and nothing under that name | refused — the same sentence; the machine does not have it either way |
| a name that is not one thing can be looked up by | refused — a name with a separator in it would reach outside the directory, and does not |
| it is readable by anybody but its owner | refused, **in its own sentence**, because setting it again is a different thing to do |
| it is there and cannot be read | refused — the machine could not use what it has |
| what is kept is not a password — blank, unsendable, or longer than one | refused — the same sentence, with what was wrong kept for whoever administers the machine and never shown |

**None of the six goes straight out, and none of them reaches the proxy without
the credential.** That is the rule `alo-proxy` already applies to an automatic
configuration it cannot work out, and the rule `alo-egress` applies to a
destination its policy cannot permit: a road that quietly went around the
company's own rule, with nobody told, is the failure all three exist to prevent.

**A proxy that asks for no name is untouched by every word of this.** The store
is not opened, not looked at and not required, and a machine on an ordinary
network never meets any of it.

## What this record does not decide

- **Nothing about ADR 0022.** A provider's key stays in the Secret Service, on
  the person's bus, derived from the daemon's uid. Neither store reads the
  other's, and no credential moves between them.
- **Nothing about `[proxy]`'s shape.** `password-in-keyring` keeps its spelling
  and its meaning — *the name the password is kept under, never the password* —
  because it is a published contract and a rename would break every description
  already written against it. What this record settles is where that name is
  looked up for a **machine-wide** proxy.
  `docs/contracts/machine-description.md` says so where the key is documented.
- **A password written into the description stays refused**, by name, in the
  sentence `alo_agentd::machine_wide_proxy` already has. This widens nothing.
- **Writing a credential from Settings is not decided here and is not built.**
  A person on their own machine who wants a proxy that asks for a name
  provisions it as an administrator does, and a door in the broker for it is a
  later task rather than a thing invented in this one — it would be a privileged
  write, and ADR 0049 §3 is where the broker's proxy door lives.
- **The unit lines are the installer lane's**, as every line in `image/` is.
  This record names them exactly; the crates name the units they need them on;
  and until they are there, a machine whose proxy asks for a name is refused in
  words by alo OS rather than by the proxy — which is the state this record
  improves on and does not yet finish.

## What is documentation and what this repository measured

The description of `systemd-creds` above — encryption with a host key or the
TPM, decryption by PID 1 at unit start, a per-unit directory whose files belong
to the unit's user, removal when the unit stops — is **systemd's documented
behaviour, not a measurement made here**, and it is written that way on purpose:
ADR 0022 recorded a paragraph of libsecret's behaviour as though it were ours
and had to withdraw it.

What this repository does measure is the reading, and it measures it without
believing any of the above: the reader checks for itself that what it found is a
regular file nobody but its owner may read, and refuses if it is not.
`crates/alo-proxy/src/provisioned.rs` holds those checks and its tests hold each
refusal. **Acceptance against a real `systemd-creds` on a booted machine is
outstanding**, and is named in this task's report rather than claimed here.
