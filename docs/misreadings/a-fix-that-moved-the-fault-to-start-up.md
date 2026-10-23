# A fix that moved the fault to start-up

## What was concluded

On 2026-10-16, after `alo-portald` had crash-looped 78 times on a machine whose
image shipped no Secret Service:

> Now the Secret portal answers `Unavailable` — the word that contract already
> had for *the keyring is not there* — and everything else works. An application
> asking for its secret is told no; it is not told the portal is missing,
> because it is not.

Every sentence of that is true, and the change it describes was the right one.
It was written beside this:

```rust
let keyring: Arc<dyn KeepsSecrets> = match TheKeyring::opened(&bus) {
    Ok(keyring) => Arc::new(keyring),
    Err(why) => { /* ...  */ Arc::new(NoKeyringHere) }
};
```

## What was true

`NoKeyringHere` was installed for the **life of the process**. The keyring and
`alo-portald` are both user units and nothing orders them, so on any boot where
the backend started first, every application that asked for its secret for the
rest of that session was told *this machine has no keyring* — about a machine
that had one by then.

Measured by giving the fixture the order nobody had tested
(`alo-keyring-fixture`: `a_keyring_arrives`), and running the losing order
against the old code:

```
test a_keyring_that_was_not_there_when_this_was_made_answers_once_it_is ... FAILED
  the keyring is on the bus now[...]: Unavailable
test asking_twice_on_a_machine_with_a_keyring_answers_twice ... ok
```

The second line is what makes the first one mean something: the fixture was
working, and only the order was broken.

## The mechanism

**A fix is scoped to the failure that prompted it, and the failure that prompted
it was permanent.** The machine of 2026-10-16 had *no keyring in the image at
all*. For that machine, reading once and keeping the answer is not a
simplification — it is exactly correct, because the answer could never change.
So the code was right about the machine in front of it, the reasoning written
beside it was sound, and it passed review.

What changed was the world, not the code: a later change put a keyring in the
image, and the permanent condition became a race. **Nothing in the fix said it
depended on the keyring never arriving**, because at the time that was not an
assumption anybody was making — it was just the situation.

This is why it is not caught by reading. The comment explains why the service
must not exit and why other portals must keep working, and both are still true.
There is no wrong sentence to find. The assumption is in the *timing*, which
prose about behaviour does not mention and a test that starts everything in one
order cannot see.

## The cure

**When a capability is found to be absent, ask whether it is absent or merely
absent yet.** The two look identical at the moment of the check and differ
entirely afterwards. If the thing can appear later — a unit, a device, a
network, a mount — the answer belongs at the point of use, not at start-up.

And **a fixture that only ever builds one order tests one order.** Where two
components have no ordering between them, both orders are ordinary and the
fixture needs a way to produce each. `a_bus_with_no_keyring_on_it` already
existed for the permanent case; what was missing was the ability to go from it
to a bus that has one, which is three lines and the whole difference between
this being caught and not.

The narrow check: **grep for a capability opened once and stored.** `OnceLock`,
a field holding `Option<Thing>` filled at construction, or a `match` on
`::opened` whose `Err` arm substitutes a permanent stand-in. Each is a reading
published as a standing property — the shape `CLAUDE.md` already forbids for
measurements in reports, in code instead of prose.
