# A bus with nothing on it

- Date: 2026-09-09
- Workstream: model selection and configuration (`alo-secrets`)
- Contributor: Claude Code
- Task: The last unproven refusal, and the hang it was hiding
- Status: **all four refusal states are now proved against a real bus.** The gap
  closed, and closing it found a fault worth more than the test.

## The gap was framed wrong

Two reports have carried this as open work: a *real bus that answers* with no
keyring on it, which is the machine whose image ships no Secret Service. The
fixture "could not stop only the keyring" — signalling the daemon left something
still serving `org.freedesktop.secrets`, so the test asserted a state it had not
produced, and it was withdrawn rather than left passing for the wrong reason.

That was the right call on a wrongly-posed problem. The state wanted is *a bus
with no keyring on it*, and the way to get it is **not to start one**. Killing a
running daemon was the hard road to a state that could simply be declined.

`AKeyringOfOurOwn::a_bus_with_no_keyring_on_it` starts the bus and stops there.

## The controls carry this test

`bus::tests` already proves `Unavailable` for a socket that is absent, is not a
socket, is not the person's, or cannot be said as an address. **A test that only
asserted `Unavailable` here would be a fifth spelling of those**, and would pass
just as well against a dead socket. So before asserting anything, the test
proves the state it claims to be in:

1. the bus answers its own `ListNames`, so it is genuinely alive;
2. `org.freedesktop.secrets` is **not** among the names it returns.

Only then is the refusal asserted.

## What it found: a two-minute hang

The test passed on its first run and took **125 seconds**.

That is not a slow test. A D-Bus proxy can be built for a name **nobody owns**,
and every call against it then waits out the method timeout. On a machine whose
image ships no Secret Service, `TheKeyring::opened` sat through stacked timeouts
before returning `Unavailable` — a person's daemon appearing to hang for two
minutes before telling them there is no store.

`store.rs` now asks the **bus** who owns the name, before building anything:

```rust
if !anybody_is_serving(&connection) {
    return Err(NotStored::Unavailable);
}
```

`NameHasOwner` is answered by `dbus-daemon` itself, immediately, whatever is or
is not running behind it. A bus that will not answer that question is treated
the same way — either is `Unavailable`, and neither is worth waiting on.

**125 seconds → 3.25 seconds**, for the whole eight-test file.

The promptness is now asserted, because it is a promise to a person rather than
a preference about test runtime. The bound is deliberately loose — 10 seconds —
so it guards against the timeout returning, not against a slow machine.

`get_all_collections` stays where it is: a name can be owned by something that is
not a Secret Service, and that case is still refused at opening rather than at
the first key somebody wanted.

## Where the four states stand now

| State | How it is reached | Real? |
|---|---|---|
| `Unavailable` | socket absent / not a socket / not the person's / unsayable | yes, `bus::tests` |
| `Unavailable` | **a live bus with nothing serving the name** | **yes, new** |
| `Locked` | a collection locked through `gnome-keyring-daemon` | yes |
| `Missing` | a real store, open, with nothing under that reference | yes |
| `Denied` | the running bus refuses delivery, by reloaded policy | yes |

**No state in `NotStored` is now unproven, and none is simulated.**

## What is preserved

Explicit bus selection from the daemon's own uid; the encrypted `Dh` session;
redaction; refusal without fallback; agent and daemon identity separation. The
new check *adds* a question to the bus and removes nothing: a machine that has a
Secret Service reaches it exactly as before.

## Proposed integration updates

**CHANGELOG.md** — nothing user-visible yet. **ROADMAP.md** — no tick.

**docs/autonomy/QUEUE.md** — credential store: every refusal state now
evidenced against a real bus; authenticated HTTPS next.

**docs/autonomy/STATE.md** — the empty-bus `Unavailable` path is proved, by not
starting a keyring rather than by stopping one; it exposed a 125-second hang on
machines with no Secret Service, now answered immediately by asking the bus who
owns the name, with the promptness asserted.
