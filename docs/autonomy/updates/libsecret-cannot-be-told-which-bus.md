# libsecret cannot be told which bus

- Date: 2026-09-08
- Workstream: model selection and configuration (`alo-secrets`, ADR 0022)
- Contributor: Claude Code
- Task: Implement the approved credential store — steps 1 and 2
- Status: **step 1 done and verified. Step 2 stopped on a measured finding**, with
  a narrow amendment proposed. Steps 3 to 5 wait on it.

## Step 1 — the package, installed and verified

Coordinated first: the other checkout had **no build running** (`pgrep -c cargo`
= 0) when the install ran.

**The transaction was inspected before anything happened.**
`apt-get install --dry-run libsecret-1-dev` on Ubuntu 26.04:

```
Inst libsecret-common  Inst libsecret-1-0     Inst gir1.2-secret-1
Inst libgpg-error-dev  Inst libgcrypt20-dev   Inst libsecret-1-dev
```

**Six packages in, zero removed, nothing reconfigured beyond the six, no service
restarted, and no Windows configuration touched.**

Verified after:

- `pkg-config --modversion libsecret-1` → **0.21.7**
- a minimal C program including `<libsecret/secret.h>` and referencing
  `secret_service_get_sync`, compiled and linked with
  `gcc $(pkg-config --cflags --libs libsecret-1)` → **LINKED OK**

So the library is present and usable. **No Rust dependency was added**, because
the next check changed the answer.

## Step 2 — the finding that stops it

ADR 0022 requires the daemon to reach **the person's own bus at
`/run/user/<uid>/bus`, derived from its own uid**, and the instruction was
explicit: *passing an address to our own wrapper does not prove libsecret uses
it; do not silently use a default connection.*

**It does not, and it cannot.** From the installed headers:

```
$ grep -rn "GDBusConnection" /usr/include/libsecret-1/libsecret/*.h
(nothing)

secret_service_open_sync (GType service_gtype,
                          const gchar *service_bus_name,
                          SecretServiceFlags flags,
                          GCancellable *cancellable,
                          GError **error);
```

**No libsecret API accepts a connection.** The only thing `open` takes is a bus
*name* — `org.freedesktop.secrets` — not a bus. The connection always comes from
`g_bus_get_sync(G_BUS_TYPE_SESSION)`, and GDBus reads that address from the
environment.

So a daemon built on libsecret connects to whatever `DBUS_SESSION_BUS_ADDRESS`
names. That is precisely the default connection ADR 0022 refuses, and precisely
what `TheBus::of` taking a uid and nothing else was built to prevent. Handing our
derived address to our own wrapper and calling libsecret anyway would look like
compliance and be none of it — so it was not done.

## What can honour it

**`zbus` 5.19.0 has `Builder::address(...)`**, in both the async and blocking
builders:

```
zbus-5.19.0/src/connection/builder.rs:191:          pub fn address<A>(address: A) -> Result<Self>
zbus-5.19.0/src/blocking/connection/builder.rs:82:  pub fn address<A>(address: A) -> Result<Self>
```

A connection is made to **the address it is handed** and to no other — which is
what makes step 3's routing test possible at all: an intended bus that receives
the connection and an environment-selected decoy that does not cannot be told
apart if the library picks the bus itself.

`secret-service` 5.2.0 speaks the same Secret Service protocol on top of zbus.
Both fetch on this machine; the sparse index is reachable and `cargo fetch`
brought them down.

## The amendment proposed, and it is narrow

**The store stays the Secret Service. The bus stays `/run/user/<uid>/bus` derived
from the daemon's own uid. Only the binding changes** — from libsecret to a Rust
client that can be given a connection.

It *removes* a C dependency rather than adding one. `libsecret-1-dev` stays
installed on the build machine: it is what made this measurable and it harms
nothing.

One consequence worth recording: **the shared-singleton limitation ADR 0022
carries is libsecret's.** A client we construct per connection may not have it,
which turns connection lifetime from something inherited into something to
measure — the acceptance plan already asks for that measurement, and it becomes a
real question rather than a foregone one.

**This is a decision, so it was not taken.** Swapping the binding named in an
accepted ADR is exactly the kind of quiet substitution that makes an accepted
decision meaningless.

## What is blocked, and what is not

Steps 3, 4 and 5 — routing with owned bus fixtures, the four store states against
a real store, and authenticated HTTPS through the production daemon path — are
each built on whichever client is chosen, so all three wait.

Nothing was stubbed and no half-store was wired into `alo-agentd`. A provider that
needs a credential is still chosen, persisted, and refused at the moment of
asking, with nothing sent.

## Evidence distinctions, kept

Both cautions from the instruction are recorded in the plan rather than assumed
away:

- **A socket with the wrong owner is not the separate agent user.**
  `a_bus_that_belongs_to_somebody_else_is_refused` tests the ownership check and
  nothing more. That a process running as `alo-agent` cannot reach the person's
  real bus is an operating-system access test, on a machine with two logins, and
  it is not something a root box can show.
- **`nothing_was_sent` is not proof of network non-transmission.** It says a
  refusal happens before a question is put. What proves nothing left is a
  listener that never accepted, which is how
  `a_provider_that_needs_a_key_is_refused_and_nothing_is_sent` is written and how
  the four store states will be.

## Coordination with the desktop worker

Unchanged and untouched in their checkout. Still theirs: a Secret Service package
and unit in the image, and sign-in. The build-machine package is now done and is
no longer a dependency on them.

## Proposed integration updates

**CHANGELOG.md** — nothing user-visible.

**ROADMAP.md** — no tick, no tier moved.

**docs/autonomy/QUEUE.md** — the credential-store item is blocked on one
amendment: which client speaks to the Secret Service.

**docs/autonomy/STATE.md** — three facts. `libsecret-1-dev` is installed on the
build machine and links. **libsecret accepts no `GDBusConnection`**, so it cannot
reach a bus chosen by the daemon rather than by the environment, and ADR 0022
cannot be implemented through it. A narrow amendment is proposed — the store and
the bus derivation unchanged, the binding changed to a Rust client with
`Builder::address` — and steps 3 to 5 wait on it.
