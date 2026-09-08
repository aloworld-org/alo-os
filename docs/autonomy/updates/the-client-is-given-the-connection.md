# The client is given the connection

- Date: 2026-09-08
- Workstream: model selection and configuration (`alo-secrets`)
- Contributor: Claude Code
- Task: Record ADR 0022's amendment and continue the store — steps 1 to 2
- Status: **amendment recorded; the explicit connection is built and its routing
  is proved.** Steps 3 to 5 are blocked on one thing, named below.

## The amendment, recorded

ADR 0022's binding is now `secret-service` over `zbus`, approved 2026-09-08.
**Client library only** — the store is still the Secret Service, the bus is still
`/run/user/<uid>/bus` derived from the daemon's own uid, and the security policy
and release scope are untouched. It removes a C dependency rather than adding
one; `libsecret-1-dev` stays installed, as instructed.

## Step 1 — the explicit connection

`crates/alo-secrets/src/store.rs`:

```rust
let connection = zbus::blocking::connection::Builder::address(bus.as_an_address().as_str())?
    .build()?;
let service = SecretService::connect_with_existing(EncryptionType::Dh, connection)?;
```

**Nothing in this crate calls `Connection::session()` or `SecretService::connect`**,
which are the two environment-chosen defaults. The address comes from `TheBus`,
which comes from a uid.

The session is **encrypted** — `EncryptionType::Dh`, never `Plain` — because a
secret crossing the bus in clear is a secret available to whatever else can read
that bus. `crypto-rust` rather than `crypto-openssl`, so the encryption brings no
second C library. Neither was weakened for a test.

Retrieval is written: search by `xdg:schema = dev.alo.Provider` and the reference,
**locked before missing** (an entry that will not open is a different thing for a
person to do about than one never added), and **nothing is unlocked by the
daemon** — unlocking is a person answering a prompt. The service's own message is
never carried into a refusal, for `41c9f1e`'s reason.

## Step 2 — routing, observed at the listeners

`crates/alo-secrets/tests/which_bus_is_reached.rs`,
**`the_intended_bus_is_reached_and_an_environment_decoy_is_not`** — passing.

Two listeners this test owns. A child process is given
`DBUS_SESSION_BUS_ADDRESS` pointing at the **decoy** and told the person's bus is
the **intended** one. Then:

- opening the keyring **our way** → the **intended** listener receives the
  connection, and the decoy does not;
- the same child asking zbus for the **session** bus, the environment's own way →
  the **decoy** receives it.

**The second half is the control and it is why this measures anything.** A decoy
nothing ever contacted would pass on a machine where the variable was ignored
entirely. Because the environment's own way really does land on the decoy, the
first result is a choice rather than an accident.

The environment variable is set on a **child**, never on the test process:
`std::env::set_var` is `unsafe` in this edition, and a test that changed the
environment of the process running every other test would reach beyond itself.

Nothing needs a real message bus for this: a listener that accepts and closes is
enough to say *the connection arrived here*, and what the client makes of the
closed connection afterwards is not the subject.

## What is not claimed

**No libsecret singleton claim was carried over.** That behaviour was
libsecret's. What a `zbus` connection does across turns and under concurrent
retrievals is **step 5, and it is a measurement I have not made** — the crate's
own documentation says so rather than inheriting an answer.

What *is* already known, and unchanged: a connection held open across a turn is
reachable from inside it, whoever opened it —
`alo-bounding`'s `a_connection_made_before_the_boundary_stays_usable_inside_it`.

## The blocker for steps 3 to 5

**There is no Secret Service on the build machine to retrieve from.** Retrieval,
the four refusal states against a real store, authenticated HTTPS through
`alo-agentd`, and the lifetime and concurrency measurements all need one running
on a bus.

Two ways, and neither is mine to take alone:

1. **Another package install** — `dbus-daemon` plus `gnome-keyring` (or another
   Secret Service), run headless on a private bus with a known password. The
   authorisation I was given named `libsecret-1-dev` specifically, and installing
   a keyring daemon on the shared machine is a further change to it.
2. **The image work with the desktop worker** — a Secret Service package and unit,
   which is already on their list, plus sign-in.

A stub Secret Service written in the test is the third option and is a poor one:
the encrypted session means implementing the DH handshake, and a store whose
crypto we wrote to satisfy our own test proves less than it appears to.

**`Unavailable` is the one state provable today** and it is proved — a bus that
is not there, is not a socket, is not the person's, or would not survive D-Bus
address grammar. The other three need a service that can be locked, be empty, and
refuse.

## What is preserved

Redaction (`41c9f1e`); refusal without fallback — the four states, none of which
sends anything; agent and daemon identity separation, untouched; and
`/run/alo/<uid>/agentd.sock` with every check in `place.rs`, which nothing here
goes near. `alo-agentd` is unchanged: a provider needing a credential is still
chosen, persisted and refused at the moment of asking, with nothing sent.

## Coordination with the desktop worker

Their checkout untouched. Still theirs, and now on the critical path for steps 3
to 5: a **Secret Service package and unit in the image**, and **sign-in**.

## Proposed integration updates

**CHANGELOG.md** — nothing user-visible.

**ROADMAP.md** — no tick, no tier moved.

**docs/autonomy/QUEUE.md** — the credential-store item is unblocked as a decision
and blocked on a running Secret Service to test against.

**docs/autonomy/STATE.md** — three facts. ADR 0022's binding is amended to
`secret-service` over `zbus`, client library only. The daemon's client is now
**given** its connection, and the routing is observed at two listeners: the
intended bus receives it and an environment-named decoy does not, with a control
proving the decoy was live. And steps 3 to 5 need a Secret Service running on the
build machine or in the image — no libsecret singleton claim was carried into the
new implementation, and connection lifetime is an unmade measurement rather than
an assumed answer.
