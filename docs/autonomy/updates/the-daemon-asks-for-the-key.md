# The daemon asks for the key

- Date: 2026-09-09
- Workstream: model selection and configuration (`alo-agentd`, `alo-secrets`)
- Contributor: Claude Code
- Task: Retrieval wired into the production daemon path
- Status: **the daemon now asks the person's keyring for a provider's key**, and
  refuses in four different sentences when it does not get one. Sending the key
  to a provider over TLS is the next task, not this one.

## What changed

`crate::doing` used to refuse **every** provider that needed a credential, with
one sentence saying this machine has nowhere to keep a key. That was true when
it was written and is not any more. It now looks the key up, and only refuses if
the store will not give it up:

```rust
Some(reference) => match Questions::a_key_from(&keyring, reference) {
    Ok(secret) => { held = secret; Some(&held) }
    Err(why) => return ToAnAgent::refused(&strings.say(&said_about(why).key(), ..)),
}
```

`said_about` is a `match` **with no wildcard**, so a fifth state added to
`alo_secrets` fails to compile here rather than silently becoming whichever arm
a catch-all named.

## Four sentences, which is the whole point of four states

`alo_secrets::NotStored` deliberately carried no words — `refusing.rs` says the
sentences "belong beside the daemon that says them, and there is nothing here to
say them about until a lookup exists". The lookup exists, so they are now in
`alo-agentd`'s vocabulary:

| State | Where it sends a person |
|---|---|
| `Unavailable` | there is nowhere on this machine to ask for one |
| `Locked` | your keyring is locked — unlock it and ask again |
| `Missing` | there is no key saved for that provider — add one |
| `Denied` | this machine's keyring refused — nothing else was asked |

None of them names the provider. That is the existing rule kept: no sentence
this service says has a gap in it, because a gap is the one road text somebody
else wrote could take into a sentence a person reads.

A test renders all four and asserts **no two are the same**. Two states sharing
a sentence would put back exactly the confusion the states exist to remove, and
nothing else in the system would notice.

## A test-isolation fault, found by the daemon's own suite

Wiring this in made an existing test fail — and the failure was the useful part.
The refusal came back as **`Missing`** rather than `Unavailable`, which means
`TheBus::of_this_process()` had succeeded and **a real `gnome-keyring-daemon`
had answered**. There is one running on `/run/user/0/bus` in the build
environment, D-Bus-activated by the `org.freedesktop.secrets.service` file the
package installed.

Two things follow.

**A correction.** `a-real-keyring-answers.md` said those activation files would
not take effect because the machine has no user session. That was wrong: there
is a session bus at `/run/user/0`, and the keyring was activated through it.

**A design gap.** A unit test was reaching the machine's real credential store.
It could not be pointed elsewhere either, because `of_this_process` reads a uid
from the kernel and is *deliberately* unredirectable — that is the property ADR
0022 wants, and it is also what made the daemon untestable.

So *whose* keyring is now an input, following the pattern `Questions` already
had for the environment (`of_this_process` for production, `of_a_session` for a
session named):

```rust
pub enum WhoseKeyring { Nobodys, ThisProcess, On(TheBus) }
```

Three states rather than an `Option`, and **`Nobodys` is the default**, so a test
that says nothing about credentials reaches no store at all. An `Option` whose
`None` meant *the usual place* is precisely how the real keyring got touched.
`Nobodys` is also an honest machine: one whose image ships no Secret Service
behaves exactly like it.

Production is unchanged. `Questions::of_this_process` selects `ThisProcess`, and
the bus is resolved **at the moment a key is wanted** rather than once at
startup — `/run/user/<uid>` does not exist before somebody signs in, and a daemon
that resolved it early would go on telling a person who signed in afterwards that
their machine has no keyring, for as long as it ran.

## A test that asserted wording

The same test quoted the English of the refusal, so rewording a sentence broke a
test whose subject had not changed. It now renders the expected sentence from
the **vocabulary**. A test that asserts wording is a test about the wording.

## Where the credential lives

In one `let` binding inside `put_to_a_model`, for the length of the ask.
`Hosted::provider` borrows it, so it must outlive the call and nothing else. It
is an `alo_models::Secret`: no accessor, no `Display`, no `Serialize`, no
`Clone`. This is the only place in the daemon where a credential exists at all.

## What is not done

**Nothing has been sent to a provider with a key on it yet.** The daemon now
*holds* the key at the point of asking; proving it reaches an owned TLS server
with a synthetic credential — and measuring connection lifetime and concurrent
retrievals — is the next task. Until that runs, the wiring is proved to the
point of the ask and no further, which is what the tests here claim and no more.

The daemon's own end-to-end retrieval test needs the keyring fixture, which
currently lives in `alo-secrets`' test directory and is not shared. Moving it
somewhere both crates can reach is part of that next task.

## Proposed integration updates

**CHANGELOG.md** — nothing user-visible until a question can actually reach a
keyed provider. **ROADMAP.md** — no tick.

**docs/autonomy/QUEUE.md** — credential store: retrieval wired into the daemon;
authenticated HTTPS next.

**docs/autonomy/STATE.md** — the daemon asks the person's keyring for a
provider's key and words all four refusals distinctly; whose keyring is an input
with a hermetic default, after a unit test was found reaching the build
machine's real store; the build environment does have a session bus and an
activated keyring, correcting an earlier report.
