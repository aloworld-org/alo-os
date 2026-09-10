# The daemon's environment is the session's

**Date:** 2026-09-10
**Workstream:** v0.01 lane B — accounts and session entry (`docs/autonomy/v0-01-lane-b-plan.md`, task 2; `docs/autonomy/v0-01-delivery-plan.md`, task 5)
**Contributor:** Claude Code worker, checkout `C:\dev\alo-os-b`
**Status:** ready for integration

## What this was

`alo-agentd` runs as the signed-in person and finds their bus at
`/run/user/<uid>`. `crates/alo-secrets/tests/a_session_that_really_ended.rs`
measured what that directory and that bus really do across a sign-in and a
sign-out, and nothing was wired to any of it. The image started the daemon from
`multi-user.target`: at boot, before anybody had signed in, with no session, no
bus and no person — a service running *as* the person rather than *for* them.

The task was the wiring: the daemon starts under the session task 1 creates,
reaches that session's bus, and stops when the session ends.

## What changed

### `crates/alo-entering` — what signing in hands the daemon (new)

One small crate, and it is the join the repository did not have. It derives from
one number — the uid of `alo_accounts::Session`, which cannot differ from the one
the machine description names — the four strings a session is made of:

| | |
|---|---|
| `runtime_directory()` | `/run/user/<uid>` |
| `bus()`, `bus_address()` | `/run/user/<uid>/bus`, `unix:path=/run/user/<uid>/bus` |
| `their_manager()` | `user@<uid>.service`: what starts the daemon and what stops it |
| `variables()` | `XDG_RUNTIME_DIR` and `DBUS_SESSION_BUS_ADDRESS`, and there is no third |
| `names_their_session()`, `names_their_bus()` | Whether an environment names theirs — the daemon's refusal |

It **derives and never observes**: nothing in it reads a variable, opens a socket
or asks the machine whether anybody is signed in. Whether a bus is really there,
is really a socket and is really the person's stays exactly where it was —
`alo_secrets::TheBus`, from a uid, with no parameter an environment variable
could arrive through. Two linux-only tests hold the two crates to one spelling of
where a bus is and how it is said to a client, so the duplication is checked
rather than hoped for.

### `image/usr/lib/systemd/system/alo-agentd.service` — started by signing in

- `WantedBy=user@1000.service` instead of `multi-user.target`. Signing in is what
  starts it.
- `BindsTo=user@1000.service` and `After=user@1000.service`. Signing out stops
  it; and it does not start before `/run/user/1000` and the bus in it exist.
- `Environment=XDG_RUNTIME_DIR=…` and `Environment=DBUS_SESSION_BUS_ADDRESS=…`.
  A system unit inherits no session, so what these lines say is the whole of what
  the service is told about one.

Everything else about the unit is unchanged: it still holds nothing, still waits
for the boundary, still gets the person's door made for it, still is not
restarted.

### `crates/alo-image` — the lines a build cannot check

`Service::bound_to()` and `Service::environment()`, and one new check with three
`Wrong`s: started by signing in rather than by booting, bound to and ordered
after the person's manager, and an environment that is exactly the person's
session — every string derived from `[logins].person` through `alo-entering`, so
moving the person in the description without moving the unit is caught. Six new
tests, each breaking one line of a copy of the shipped image.

### `crates/alo-agentd/src/session.rs` — the refusal (new)

At start-up, directly after the description (which is what says who the person
is) and before anything is opened: if the environment names a session, it must be
the person's, or the process stops and says which variable, what it said, and
what it would have to say.

**It checks the variables; it never reads one to decide anything.** Where this
daemon's bus is still comes from its own uid. The difference is the point: a
daemon that read `DBUS_SESSION_BUS_ADDRESS` could be pointed at root's bus by one
edited line in a unit file; a daemon that checks it cannot be pointed anywhere,
and the worst an edited line can do is stop the service.

## Decisions I made, and why

**A system unit bound to the person's manager, rather than a systemd *user*
unit.** A user unit gets the session environment for free, which is tempting. It
also moves `RuntimeDirectory=` from `/run/alo/1000` to under the runtime
directory, which is ADR 0017's door in a different place, and it cannot be
ordered after a system unit at all — so `Requires=alo-boundaryd.service`, which
is ADR 0015 in one line, would have had to become prose. A system unit pulled in
by `user@<uid>.service` keeps both ADRs untouched and still starts and stops with
the session. That is configuring systemd rather than rearranging alo OS around
it.

**An absent variable is allowed; a wrong one is not.** The rule is exactly *say
nothing, or say the person's session*. Two reasons, both about a real machine:
nothing in the daemon uses these variables, so an absent one leaves it precisely
where it already was — `Unavailable` when a store is opened, the state
`a_session_that_really_ended.rs` measures — and the person's bus appears when
their user manager gets to it, not at the instant the manager is called started.
A service that refused to run because a socket was three hundred milliseconds
late would boot to a sentence about D-Bus, and this unit is deliberately not
restarted.

**Equality, not parsing.** `names_their_bus` accepts the address and the bare
path and nothing else. A rule that pulled a uid out of whatever it was handed
would have an opinion about strings nobody on this machine writes; a rule that
compares against the one spelling this machine uses refuses
`unix:path=/run/user/1000/bus;unix:path=/run/user/0/bus` — a D-Bus address's `;`
means *and if that fails, try this one* — without having to understand any of it.

**A crate rather than a module in `alo-accounts`.** Three callers, and they have
nothing else in common: the checker (`alo-image`, portable, on whoever's laptop),
the daemon (`alo-agentd`, Linux), and eventually the sign-in surface. Putting it
in `alo-accounts` would give that crate a second reason to change and would drag
a portable checker behind an account store.

**The new crate is not Linux-only.** `alo-agentd` and `alo-secrets` compile to
nothing off Linux for good reasons. This one is text about paths and its other
caller is the crate that checks the shipped image — a check that vanished on the
host where the files are edited would be a check nobody runs.

## Verification

Run in this checkout, on Windows 11 (`C:\dev\alo-os-b`), on the finished tree:

| Command | Result |
|---|---|
| `cargo fmt --all` | clean |
| `cargo clippy --all-targets --workspace -- -D warnings` | zero warnings |
| `cargo test --workspace` | all green, no failures |
| `cargo doc --workspace --no-deps` | 62 warnings, the same 62 as before this change — the known Windows `broken_intra_doc_links` about `cfg(target_os = "linux")` modules that the root `Cargo.toml` records. None from the files here. |

Each acceptance criterion's test was then run on its own with `--exact`:

- **Starts under the session** — `cargo test -p alo-image --lib -- --exact
  checking::tests::an_agent_started_before_anybody_signs_in_is_caught`, and
  `…::the_image_this_repository_ships_agrees_with_itself` for the shipped unit.
- **Reaches that session's bus** — `cargo test -p alo-entering --lib -- --exact
  environment::tests::what_a_session_hands_a_daemon_is_derived_from_the_session`
  and `cargo test -p alo-image --lib -- --exact
  checking::tests::an_agent_pointed_at_another_logins_session_is_caught`.
- **Stops when the session ends** — `cargo test -p alo-image --lib -- --exact
  checking::tests::an_agent_that_would_outlive_the_session_is_caught`.

### Refusal paths, tested beside the legitimate ones

- `alo-entering`: another person's session and bus under either spelling; a name
  that merely begins the same (`/run/user/1000` for uid 100, `…/100/../0`,
  `unix:abstract=…`, a second address after a `;`).
- `alo-agentd`: another login's runtime directory; another login's bus; the two
  variables refused separately; and the machine that says nothing, which starts.
- `alo-image`: pulled in by `multi-user.target`; `BindsTo=` removed; `After=`
  removed; the environment line removed; the environment line pointed at uid 0;
  and the description's person moved without the unit.

### Not run here, and named rather than implied

**`crates/alo-entering/tests/a_daemon_in_the_persons_session.rs`** — the three
states from a real sign-in, on a machine with `logind`. It signs a person in with
`su` against a store of `alo-accounts`' own on a temporary disk, and measures
that the runtime directory the environment names is there, that the bus it names
is a socket that accepts a connection and is the one `alo-secrets` derives from
the uid, and that both are gone after a logout. It is `#[ignore]`d for
`a_session_that_really_ended.rs`'s reasons — it needs root, needs `logind`, and
needs to be the only thing testing at the time:

```text
cargo test -p alo-entering --test a_daemon_in_the_persons_session -- --ignored --test-threads=1
```

It is **not** offered as evidence, deliberately: the supervisor runs evidence
tests with `--include-ignored`, and this one signs a person in and out of
whatever machine that is. It belongs in a coordinated window, alongside the
`a_session_that_really_ended.rs` cases it extends.

**It does not start `alo-agentd`, and this is written down rather than left to be
discovered.** The daemon reads `/etc/alo/agentd.toml` before anything a test here
could ask it about, so starting it would need a machine description installed on
the machine running the test — which is the image's job, and delivery-plan task
10. What the daemon does with the environment it is handed is tested in
`crates/alo-agentd/src/session.rs` against every string this file measures; that
the unit hands it those strings at all is tested in `crates/alo-image`. What
remains unmeasured until task 10 is the two joined on a booted machine, and the
image task is where that belongs.

**The image is not built here.** `image/Containerfile` already runs `systemctl
enable alo-agentd.service`, which now writes into `user@1000.service.wants/`
rather than `multi-user.target.wants/`. Nothing in this change alters the build
steps, and the checks in `crates/alo-image` read the files the build copies.

## Limitations

- The person's number is `1000` in the unit file, as it is in
  `RuntimeDirectory=alo/1000`, for the reason `docs/quirks.md` records: `%U` in a
  system unit expands to 0 before `User=` is resolved. `crates/alo-image` is what
  holds every one of those numbers to the machine description. A machine whose
  person is not 1000 is a different image, and that is unchanged by this work.
- The daemon's session check is a refusal and not a discovery: it cannot tell a
  machine that was started outside any session from one that was started before
  the bus appeared. Both say nothing, and both are allowed, and the reasoning is
  in `session.rs`.

## Proposed shared-document updates

I did not edit `CHANGELOG.md`, `ROADMAP.md`, `docs/autonomy/QUEUE.md` or
`docs/autonomy/STATE.md`. For the integration owner:

**CHANGELOG.md**, under the unreleased section:

> **The agent service starts when you sign in, and stops when you sign out.**
> Until now it started with the machine, before anybody had signed in — running
> as the person without being in their session. It is now started by the
> person's own session, told where that session is, and stopped with it. If it
> is ever pointed at somebody else's session, it refuses to run and says so
> rather than quietly using it.

**ROADMAP.md / QUEUE.md:** v0.01 delivery-plan task 5 is done, so task 10 (the
image carries the shell, the session and the daemon) no longer waits on session
work. Lane B's task 3 is written and ready: where a machine keeps its grants
between one sign-in and the next.
