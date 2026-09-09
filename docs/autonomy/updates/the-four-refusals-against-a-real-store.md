# The four refusals, against a real store

- Date: 2026-09-09
- Workstream: model selection and configuration (`alo-secrets`)
- Contributor: Claude Code
- Task: Locked and denied, proved rather than described
- Status: **all four refusal states now hold against a real Secret Service.**
  One of them could not happen at all before this change.

## A state no code path could reach

`NotStored::Denied` has been declared since ADR 0022 and **was unreachable**.

`secret_service::Error` has no denied variant — there is no `AccessDenied` in
that enum at all. A refusal is a *bus* error, and `as_not_stored` sent every
unrecognised error to its `_ => Unavailable` arm. So a person the store had
refused was told **there is nothing here to ask**, and would have gone looking
for a service that was running the whole time. Those are the two states the type
exists to keep apart.

It is now matched, and the shape was **measured, not assumed**. The first fix
matched `ZbusFdo(AccessDenied)`, which is the obvious guess and is wrong; the
test still failed. A real denial arrives as:

```
Zbus(MethodError(OwnedErrorName("org.freedesktop.DBus.Error.AccessDenied"), Some("Rejected send message, 2 matched rules; ... uid=0 pid=8173 comm=... "), Msg { .. }))
```

The match is on the **error name**, which is the interoperable part of the D-Bus
specification. Deliberately **not** on the message beside it: that text is
`dbus-daemon`'s own English and carries the caller's uid, pid and executable
path. It is exactly what `41c9f1e` established must never be repeated back, and
`NotStored` still carries no words at all.

## How a real refusal is produced

Not by constructing an error. `AKeyringOfOurOwn::started_where_the_bus_can_refuse`
starts its `dbus-daemon` from a configuration file, and
`stop_letting_anyone_reach_the_keyring` rewrites that file with

```xml
<deny send_destination="org.freedesktop.secrets"/>
```

after the allow rules, since the last matching rule wins, and asks the bus to
`ReloadConfig` over its own connection. The keyring **stays running and stays
holding the key** — what changes is that the bus will not carry the message.

That distinction is the point. This is a refusal, not an outage, and the
difference between the two is what the test asserts. An injected transport
failure would have proved neither.

## What is proved now

`crates/alo-secrets` — **7 real-store tests, plus routing and the unit tests.**

| State | How it is reached |
|---|---|
| **`Denied`** | the running bus refuses to deliver, by reloaded policy |
| **`Locked`** | a real collection locked through `gnome-keyring-daemon` |
| **`Missing`** | a real store, open, with nothing under that reference |
| **`Unavailable`** | socket-shaped cases only — see the gap below |

Two further guarantees, each beside the refusal it constrains:

- **`a_refused_lookup_leaves_the_collection_locked`** — `store.rs` says unlocking
  is a person answering a prompt and not a daemon deciding for them. This is
  that sentence as a check: after a refused lookup the collection is still
  locked, so nothing on our side quietly opened it to get an answer.
- **`a_denied_lookup_carries_neither_the_key_nor_what_the_bus_said`** — the
  refusal reaches no provider, does not render the synthetic key, and does not
  repeat the bus's message.

Everything the fixture arranges — storing, locking — now goes through a client
of its own rather than through `TheKeyring`, so no test proves itself by driving
the object it is checking.

## Still open, unchanged and not papered over

**A real bus with no Secret Service on it.** `Unavailable` is proved only in its
socket-shaped forms. The `get_all_collections` check in `TheKeyring::opened` is
the code for the empty-bus case and **remains unproven**, because the fixture
still cannot stop only the keyring. The withdrawn test is still documented where
it would live.

Items 3 to 6 of the credential work — daemon wiring, authenticated HTTPS through
`alo-agentd`, connection lifetime and concurrency — are next, not done.

## What is preserved

Explicit bus selection from the daemon's own uid; the encrypted `Dh` session;
redaction; refusal without fallback; agent and daemon identity separation;
`/run/alo/<uid>/agentd.sock` untouched. No access control was weakened to make a
test pass — the denied test *adds* an access control and then proves we report
it honestly.

## Proposed integration updates

**CHANGELOG.md** — nothing user-visible yet. **ROADMAP.md** — no tick.

**docs/autonomy/QUEUE.md** — credential store: all four refusal states now
evidenced; authenticated HTTPS next.

**docs/autonomy/STATE.md** — `NotStored::Denied` was unreachable and is now
matched on the D-Bus error name; locked and denied are proved against a real
store with a real bus-policy refusal; the empty-bus `Unavailable` path remains
unproven.
