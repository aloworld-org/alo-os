# ADR 0060 — A person's own proxy password is set with the proxy, in one act

**Status:** accepted, 2026-09-20. Written by task 13 of
`docs/autonomy/v0-5-software-and-the-web-plan.md` (*A person's own proxy
password, set on the machine that is theirs*), whose code is built on it — the
shape [ADR 0049](0049-the-network-is-changed-through-the-broker-and-its-password-never-reaches-the-agent.md)
and [ADR 0059](0059-where-a-machine-wide-proxy-password-is-kept.md) were written
in by the tasks built on them.
**Date:** 2026-09-20
**Proposed by:** the software-and-the-web workstream
**Context:** [ADR 0059](0059-where-a-machine-wide-proxy-password-is-kept.md)
(where a machine-wide proxy password is kept, and the thing it explicitly did
not decide);
[ADR 0049](0049-the-network-is-changed-through-the-broker-and-its-password-never-reaches-the-agent.md)
§3 (the proxy is set by a person through `network.set-proxy`, and no agent
proposes one in v0.5); [ADR 0016](0016-the-organisation-bounds-and-the-person-chooses.md)
(the organisation bounds and the person chooses — and is their own machine's
organisation); [ADR 0011](0011-the-base-is-rented-and-the-image-is-a-container.md)
(the base is rented, configured and never patched);
[ADR 0001](0001-the-capability-model.md) §2 (the broker's closed list and its
lack of free-form parameters) and §5 (one approval, one execution);
[ADR 0022](0022-where-a-providers-key-is-kept.md) (a credential is never a field
in a setting); `crates/alo-proxy/src/provisioned.rs`,
`docs/contracts/machine-proxy-file.md`, `docs/features.md` v0.5 *corporate proxy
support, machine-wide and honoured by applications*.

## The question in one line

**A person on their own machine sets a proxy that asks who they are. Where does
the password they type go, what carries it across the broker's door, and who
writes the credential ADR 0059 chose?**

ADR 0059 settled the store and settled it for the case that pays for it: an
organisation writes the credential with the machine, beside the `[proxy]`
section in the description. **On a personal machine there is nobody else to
write it.** ADR 0016 says the person is their own machine's organisation, and
ADR 0049 §3 says the proxy is set by a person through the broker — so a person
could already set a proxy in Settings and could not give it the password it asks
for without opening a terminal and running `systemd-creds` by hand. That is a
settings panel with a field that works and a field beside it that does not, and
ADR 0059 named the gap rather than closing it: *writing a credential from
Settings is not decided here and is not built … a door in the broker for it is a
later task.* This is that record.

## §1 One act, and no twelfth verb

**Options.**

- **(A) A second broker verb, `network.set-proxy-password`.** Two verbs, two
  approvals, two carriers — and an order in which one of them can fail. Whichever
  order is chosen, a machine can be left with a proxy set that it cannot sign in
  to, or with a credential for a proxy that is not set. The plan for this work
  asks in so many words that a machine where the write fails *changes nothing,
  rather than leaving a proxy set that cannot be signed in to*, and two verbs
  cannot promise that without a transaction across two approvals, which is
  exactly the thing ADR 0001 §5 refuses: an approval is never a session.
- **(B) The existing `network.set-proxy` carries both.** **Chosen.** A person
  hands over the proxy they chose and, beside it, the password it asks for; one
  approval covers both; the broker writes the credential **first** and the
  machine's proxy file **only if that succeeded**. A failure leaves the machine
  exactly as it was.

The broker's closed list does not grow, its argument stays thirty-two bytes, and
`crates/alo-broker` is not edited by this change — which matters twice over,
because that crate belongs to `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`
and because a privileged list that grows for every new thing is a privileged
list nobody audits.

**The consequence, stated plainly:** the meaning of `network.set-proxy` is now
*set the machine's proxy, and the password it signs in with*. It is the same
change to the same setting, asked for by the same person, in one act. Nothing
else about the verb moves.

**And the useful corollary.** Because the proxy file is written only after the
credential is, **the machine's proxy file saying the proxy signs in is the
statement that a password is set.** A settings panel needs no second place to
look and no way to read the credential: it reads the file it already reads
(`0644`, everybody's to read) and shows *a password is set* with an offer to
replace it.

## §2 One reserved name, because the unit line is not written per machine

ADR 0059 delivers the credential to a unit with a **static** line in a unit
file:

```
LoadCredentialEncrypted=<name>:/etc/credstore.encrypted/<file>
```

An organisation writing a machine's description picks its own name and
provisions that credential itself; the line is written with the machine. **A
person choosing a name in Settings cannot be, because nothing rewrites the unit
files afterwards.** A name a person typed would need a unit line nobody will
ever write, and the credential would be delivered to no unit at all.

So alo OS reserves **one** name for the password a person sets on their own
machine:

```
the-proxy-on-this-machine
```

`alo_proxy::THE_PERSONS_PROXY_PASSWORD` is that name and
`alo_proxy::WhereThePasswordIs::on_this_machine` is it as a setting holds it.
Settings puts it in `password-in-keyring`; every road out looks it up under it;
and the broker **refuses to write a credential under any other name**, which is
also what keeps a root process from writing a file somebody else chose the name
of into the machine's credential store. The unit line is fixed for the life of
the product and the installer lane writes it once:

```
LoadCredentialEncrypted=the-proxy-on-this-machine:/etc/credstore.encrypted/the-proxy-on-this-machine
```

No spaces, no mapping to compute, one segment, and identical on both sides of
the colon — a name chosen so that whoever adds a road can copy the line without
thinking about it. `password-in-keyring` keeps its spelling and its meaning
(ADR 0059), and an organisation's own name keeps working exactly as it did.

## §3 The password crosses by identity, and the record keeps no digest of it

The broker's door takes no text (ADR 0001 §2), so the password is handed over
the way the proxy already is: written into `/run/alo-broker/wanted`, the folder
the broker makes `0770` in the person's group, and named across the door by the
digest of the bytes. `alo_networks::proxy_password::THE_WANTED_PASSWORD` is the
file; `alo_networks::proxy_password::handed_over_together` is the one place the
two files are bound into the identity that is asked for, **the proxy's bytes
followed by the password file's** — which for a proxy handed over with no
password is the proxy's bytes exactly, so the sentence
`docs/contracts/machine-proxy-file.md` already publishes stays literally true.

**A digest of a password is a password.** The identity crosses the door and is
written into the broker's record, and SHA-256 of what somebody typed is a
dictionary attack away from what they typed. So the handed-over password file
is **thirty-two bytes of the kernel's randomness followed by the password**,
and the identity is taken over that. The record then names an act and reveals
nothing about the credential, which is what *the password never reaches the
record* has to mean to be worth saying.

**It is removed whichever way the act goes.** The handed-over proxy is removed
when it is used, as it always was; the password is removed on every path,
including every refusal, because bytes left in `/run` are a credential waiting
for somebody to read them and a person retyping a password is a smaller cost
than that.

## §4 The credential is written by the tool the base already has

`systemd-creds encrypt`, started directly, with a cleared environment, the
password on its **standard input** and never in an argument — `/proc` is
readable and an argument list is not a secret. ADR 0011: the base is rented, and
this is renting it. **No encryption of ours**, and ADR 0022's exclusion of a
credential-transfer protocol is untouched: nothing here moves a credential
between machines or between stores.

It is written **beside** its place in the store, `0600`, synced and renamed
over, so a store that already held a credential still holds that one if
anything fails. `alo_proxy::TheMachinesCredentials` is the writer and it is the
counterpart of `alo_proxy::TheMachinesPasswords`, in the crate that owns the
question *where is a machine-wide proxy password kept*.

`/etc/credstore.encrypted` is made `0700` and root's if it is not there. What a
person's own programs can read is the encrypted file's absence: the store is
root's, the credential inside it is `0600`, and the decrypted copy exists only
in a unit's own credentials directory, which ADR 0059 already describes.

## §5 A managed machine is refused before anything is handed over

ADR 0016 is already settled and `alo_proxy::NotChanged::AnOrganisationSetIt` is
already its sentence — *your organisation set the proxy for this machine, so it
cannot be changed here; ask whoever manages it*. What was missing is that a
person never read it: the broker refused, in English, into its own log, and
Settings turned every refusal into *the proxy was not set*.

So the road a person's own change takes (`alo_changing_network::set_proxy_by_hand`)
reads the machine's proxy file **first**, and on a machine an organisation
manages refuses in that existing sentence — with nothing written into the
handed-over folder at all, so the password they typed never leaves the process
they typed it into. The broker still refuses independently, as it must: this is
a better sentence in front of a check that was already there, never a check
moved out of the privileged component.

## §6 It is never read back

There is no function anywhere that answers with a proxy password.
`alo_proxy::Password` still goes in and does not come out — its bytes have
exactly two readers inside `alo-proxy`, the road out (`Carried`) and this
writer, and neither is reachable from outside the crate. The credential store is
write-and-decrypt-at-unit-start; nothing in this repository decrypts one. A
settings panel shows *a password is set* from §1's corollary and offers to
replace it, which is the whole of what a surface can do with one.

## What this record does not decide

- **The unit lines in `image/`.** ADR 0059 named them and they are the
  installer lane's, as every line in `image/` is. §2 fixes the name so that
  those lines can be written once and never again.
- **Anything about an agent.** ADR 0049 §3 stands unchanged: no agent verb
  proposes a proxy in v0.5, and none proposes a password. Nothing declared in
  `alo_changing_network::verbs` reaches any of this, and a test says so.
- **Anything about ADR 0022 or ADR 0059's store.** A provider's key stays in the
  Secret Service on the person's bus. A machine-wide proxy password stays in the
  machine's own credentials. Neither reads the other's.
- **An organisation's own credential.** A description that names its own
  `password-in-keyring` is provisioned by whoever administers that machine,
  exactly as before. This record adds a way for a person to provision **their
  own machine's**, and adds nothing to what an organisation may write.

## What is documentation and what this repository measured

That `systemd-creds encrypt` writes a credential the machine can later decrypt,
and that `LoadCredentialEncrypted=` delivers it to a unit, is **systemd's
documented behaviour**. What this repository measured is the writing: the tests
run the real `systemd-creds` where the machine running them has one, and check
that what was written decrypts back to what was handed over; where it has none,
the same tests run against a stand-in and the real tool's absence is itself a
refusal with its own test. **Acceptance on a booted alo OS image, with the unit
lines in place, is outstanding** and is named in this task's report rather than
claimed here.
