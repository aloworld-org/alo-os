# A machine says it exists, and says nothing else

- Date: 2026-09-13
- Workstream: v0.5 the local network, task 1 (`docs/autonomy/v0-5-the-local-network-plan.md`)
- Contributor: Claude Code, by hand — no worker could run
- Status: **the first half of ADR 0003, and the half where the leak would be**

`docs/features.md` promises, at v0.5: *machines find each other with zero
configuration — no addresses typed, no accounts.* ADR 0003 sets the limit that
makes it safe to keep: **discovery reveals presence and nothing else.**

The whole risk of the promise is in one place. Everything on a network can read
an advertisement — a machine nobody has paired with, a machine nobody owns, a
café. So this task is not *make discovery work*; discovery is forty lines of
DNS-SD that everybody's printer already speaks. It is *decide what an
advertisement may carry, and make anything else impossible rather than
unlikely.*

## What changed

A new crate, `crates/alo-nearby`. `Presence` is what a machine says about
itself and it holds two things: a `MachineId` and a port. There is no field for
the person, the organisation, the models on the machine, what it has granted,
what it is doing, or what it is called.

There is also **no field for whether it has paired with anything**, which is
the one that would not have been noticed. Presence that changed shape when a
pairing was made would tell a network watching it that a pairing had been made,
and *who is working with whom* is not presence.

The list is closed in the type. A later change that wanted to say more has to
add a field to a struct whose entire documented subject is that it does not,
rather than append a key in a packet builder.

## The identity, and the two things it is not

Sixteen random bytes from the kernel, written to a file once and read
afterwards. Every obvious alternative is refused, for two different reasons.

**A serial outlives a reinstall**, and an identifier that survives somebody
wiping their machine is a tracker. A MAC address, a disk serial or a TPM key
would each mean that a person who reinstalls alo OS to stop being recognised
does not stop being recognised. The identity lives in a file so that removing
the installation removes it, and
`the_identity_survives_a_restart_and_outlives_nothing_else` holds both halves
of that sentence at once.

**A name a person chose is a person's name.** DNS-SD's instance name is the
field ordinarily filled with human words — *Disan's printer* — and
`DISAN-LAPTOP` on a café network tells everybody in the café who is in the
café. The instance name here is the identity, and so is the host name.

## What is deliberately not in the packet

Three records — a `PTR`, an `SRV` and a `TXT` carrying one key, `v=1`.

**There is no `A` record**, and that is a decision rather than an omission. An
`A` record is where a responder lists the addresses it can be reached at, and a
machine with a wired connection, a wireless one and a virtual machine bridge
lists three — telling a network watching it how this machine is attached to the
world. The address a machine is reachable at is the source address of its own
answer, which the packet revealed by arriving; nothing further is owed. A
resolver that insists on an `A` record gets no answer from us, which is the
correct trade in the direction of saying less.

## The refusal that is not the ordinary thing

A `TXT` key that is not on the list is refused, **not skipped**.

Skipping it is what every DNS-SD reader does, and it is why the format has
lasted thirty years. It is refused here because the failure being guarded
against is not a stranger's packet. It is *this* machine, two years from now,
advertising the person's name in a key the reader would have ignored, with
nothing in the workspace failing. `NotNearby::SaysMoreThanPresence` names the
key it refused.

## What is not here

**Nothing connects to anything it finds.** A `Found` is a fact written down;
the port it carries is a number, not a dial. Turning a found machine into one
this machine will talk to is ADR 0003's mutual, deliberate pairing, which is
task 2. `this_crate_dials_nothing_and_has_no_setting` reads the crate's own
shipped source and fails if a `TcpStream` appears in it.

**And there is no setting.** No *advertise as*, no *discovery off for this
network*, no subnet rule, no *remember this machine*. ADR 0003 names each as
the whole vulnerability, and a switch added here would be the trusted-network
setting arriving by the back door. The same test fails if this crate learns to
read one.

**`Standing` has one arm.** Every machine found is `NotPaired`, because nothing
can be anything else yet. It is an enum at one arm rather than a `bool` or
nothing at all, because a reader looking for *what does finding a machine give
it* should find the answer written down rather than have to notice that nothing
gives it anything.

## Two bugs worth recording

**Half a DNS header.** The first version of `a_question_in` stepped over three
of the header's six numbers and then read the question's name out of the middle
of the counts. It failed exactly once, in the one test that puts both sides on
a socket — every test that built a packet and read it back was green, because
both halves agreed with each other. The road being walked is what caught it,
and it is the argument for `looking.rs` having tests at all.

**A source scan that read its own tests.** `this_crate_dials_nothing_and_has_no
_setting` failed on `reading.rs`, which reads `COMPUTERNAME` from the
environment — inside a test, in order to prove the packet does not carry it.
The scan now stops at `#[cfg(test)]`: what ships is what is scanned.

## Verified

Windows, this development machine:

| Target | Result |
|---|---|
| `alo-nearby` unit tests | 31 passed, 0 failed |
| `the_local_network_says_no_more_than_a_machine_exists` | 5 passed, 0 failed |
| doctest | 1 passed |
| `cargo fmt --all --check` | clean |
| `cargo clippy -p alo-nearby --all-targets -D warnings` | clean |
| the rest of the workspace | green but for `alo-recounting` |

`alo-recounting`'s failures are the platform split recorded in
`docs/quirks.md`: they are Windows-only and pass on Linux, and nothing in this
change touches that crate.

**What no test here shows is that a second physical machine on an office
network hears this one.** Both sides are on this host, over ordinary datagrams,
which shows the packets are right and the road is walked. Multicast on a real
link — an interface that drops it, a switch that does not forward it, a
Windows firewall that eats it — is owed to two machines, and this crate takes
its socket from the caller rather than opening one so that the day there are
two machines, nothing in it has to change.

**CHANGELOG.md** — nothing user-visible: there is no surface that calls this.
**ROADMAP.md** — the v0.5 line *machines find each other with zero
configuration* is built rather than unstarted; no tick moves *on the machine*.
